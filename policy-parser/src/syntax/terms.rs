//! ANTLR parse tree → [`Term`]: one builder per grammar rule.

use std::collections::BTreeMap;
use std::rc::Rc;

use antlr4rust::parser_rule_context::ParserRuleContext;
use antlr4rust::token::Token;
use antlr4rust::token_factory::TokenFactory;
use antlr4rust::tree::ParseTree;
use anyhow::{Context, Result, anyhow, bail};
use policy_ir::{Clause, PredicateIndicator, Term};

use super::literals::{canonical_integer, decode_character_code, decode_quoted};
use super::required;
use crate::parser::prologparser::*;

#[derive(Default)]
struct ClauseVariables {
    named: BTreeMap<String, u32>,
    names: Vec<Option<String>>,
}

impl ClauseVariables {
    fn intern(&mut self, source_name: &str) -> Term {
        let id = if source_name == "_" {
            self.declare(None)
        } else if let Some(id) = self.named.get(source_name) {
            *id
        } else {
            let id = self.declare(Some(source_name.to_owned()));
            self.named.insert(source_name.to_owned(), id);
            id
        };
        Term::Variable { id }
    }

    fn declare(&mut self, source_name: Option<String>) -> u32 {
        let id = self.names.len() as u32;
        self.names.push(source_name);
        id
    }
}

/// Builds a standalone goal; its variables are local to the goal.
pub(super) fn build_goal(ctx: &BodyContextAll<'_>) -> Result<Term> {
    build_body(ctx, &mut ClauseVariables::default())
}

/// Builds a directive body.
pub(crate) fn build_directive(ctx: &DirectiveContextAll<'_>) -> Result<Term> {
    build_body(
        &*required(ctx.body(), "directive body")?,
        &mut ClauseVariables::default(),
    )
}

/// Builds a clause together with the predicate it belongs to.
pub(crate) fn build_clause(ctx: &ClauseContextAll<'_>) -> Result<(PredicateIndicator, Clause)> {
    let mut variables = ClauseVariables::default();
    let (predicate, head) =
        build_callable(&*required(ctx.callable(), "clause head")?, &mut variables)?;
    let body = match ctx.body() {
        Some(body) => build_body(&body, &mut variables)?,
        None => Term::atom("true"),
    };
    let clause = Clause {
        variables: variables.names,
        head,
        body,
    };
    Ok((predicate, clause))
}

fn build_callable(
    ctx: &CallableContextAll<'_>,
    variables: &mut ClauseVariables,
) -> Result<(PredicateIndicator, Vec<Term>)> {
    let atom = required(ctx.atom(), "callable atom")?;
    let name = build_atom(&atom)?;
    let args = match ctx.term_list() {
        Some(arguments) => {
            let open = required(ctx.LPAREN(), "clause head '('")?;
            require_functional_notation(atom.stop().get_stop(), &*open.symbol)?;
            build_terms(&arguments, variables)?
        }
        None => Vec::new(),
    };
    let predicate = PredicateIndicator {
        name,
        arity: args.len(),
    };
    Ok((predicate, args))
}

/// Completes a `left (OPERATOR right)?` rule: `OPERATOR(left, right)` when the
/// optional part is present, otherwise `left` unchanged.
fn infix(left: Term, operator: Option<String>, right: Option<Result<Term>>) -> Result<Term> {
    match (operator, right.transpose()?) {
        (Some(operator), Some(right)) => Ok(Term::compound(operator, vec![left, right])),
        (None, None) => Ok(left),
        _ => bail!("ANTLR parse tree has an operator without its operand"),
    }
}

fn build_body(ctx: &BodyContextAll<'_>, variables: &mut ClauseVariables) -> Result<Term> {
    let left = build_if_then(&*required(ctx.if_then(), "body operand")?, variables)?;
    infix(
        left,
        ctx.SEMI().map(|operator| operator.get_text()),
        ctx.body().map(|right| build_body(&right, variables)),
    )
}

fn build_if_then(ctx: &If_thenContextAll<'_>, variables: &mut ClauseVariables) -> Result<Term> {
    let left = build_conjunction(&*required(ctx.conjunction(), "if-then operand")?, variables)?;
    infix(
        left,
        ctx.IF_THEN().map(|operator| operator.get_text()),
        ctx.if_then().map(|right| build_if_then(&right, variables)),
    )
}

fn build_conjunction(
    ctx: &ConjunctionContextAll<'_>,
    variables: &mut ClauseVariables,
) -> Result<Term> {
    let left = build_term999(&*required(ctx.term999(), "conjunction operand")?, variables)?;
    infix(
        left,
        ctx.COMMA().map(|operator| operator.get_text()),
        ctx.conjunction()
            .map(|right| build_conjunction(&right, variables)),
    )
}

fn build_term999(ctx: &Term999ContextAll<'_>, variables: &mut ClauseVariables) -> Result<Term> {
    if let Some(not) = ctx.NOT() {
        let operand = build_term999(&*required(ctx.term999(), "negated term")?, variables)?;
        return Ok(Term::compound(not.get_text(), vec![operand]));
    }
    build_relation(&*required(ctx.relation_expr(), "relation")?, variables)
}

fn build_relation(
    ctx: &Relation_exprContextAll<'_>,
    variables: &mut ClauseVariables,
) -> Result<Term> {
    let operands = ctx.qualified_expr_all();
    let mut operands = operands.iter();
    let left = build_qualified(required(operands.next(), "relation operand")?, variables)?;
    infix(
        left,
        ctx.relation_op().map(|operator| operator.get_text()),
        operands
            .next()
            .map(|right| build_qualified(right, variables)),
    )
}

fn build_qualified(
    ctx: &Qualified_exprContextAll<'_>,
    variables: &mut ClauseVariables,
) -> Result<Term> {
    let left = build_additive(
        &*required(ctx.additive_expr(), "qualified operand")?,
        variables,
    )?;
    infix(
        left,
        ctx.COLON().map(|operator| operator.get_text()),
        ctx.qualified_expr()
            .map(|right| build_qualified(&right, variables)),
    )
}

fn build_additive(
    ctx: &Additive_exprContextAll<'_>,
    variables: &mut ClauseVariables,
) -> Result<Term> {
    fold_left(
        ctx.multiplicative_expr_all(),
        &ctx.ops,
        variables,
        build_multiplicative,
    )
}

fn build_multiplicative(
    ctx: &Multiplicative_exprContextAll<'_>,
    variables: &mut ClauseVariables,
) -> Result<Term> {
    fold_left(ctx.unary_expr_all(), &ctx.ops, variables, build_unary)
}

fn build_unary(ctx: &Unary_exprContextAll<'_>, variables: &mut ClauseVariables) -> Result<Term> {
    if let Some(operator) = &ctx.op {
        let operand = required(ctx.unary_expr(), "prefix operand")?;
        // `-1` is the integer -1, `- 1` is -(1) (as in ISO and SWI-Prolog).
        // The number is the leftmost primary of the operand, so `-1^2` is
        // ^(-1, 2) while `- 1^2` is -(1^2).
        if operator.get_text() == "-"
            && operand.op.is_none()
            && let Some(primary) = operand.primary()
            && let Some(number) = number_literal(&primary)?
            && operator.get_stop() + 1 == primary.start().get_start()
        {
            return build_power(&operand, negate(number), variables);
        }
        let operand = build_unary(&operand, variables)?;
        return Ok(Term::compound(operator.get_text(), vec![operand]));
    }

    let left = build_primary(&*required(ctx.primary(), "power operand")?, variables)?;
    build_power(ctx, left, variables)
}

/// Completes `primary (** power_operand | ^ unary_expr)?` given the built primary.
fn build_power(
    ctx: &Unary_exprContextAll<'_>,
    left: Term,
    variables: &mut ClauseVariables,
) -> Result<Term> {
    let right = match ctx.power_operand() {
        Some(right) => Some(build_power_operand(&right, variables)),
        None => ctx.unary_expr().map(|right| build_unary(&right, variables)),
    };
    infix(
        left,
        ctx.POW_XFX()
            .or(ctx.POW_XFY())
            .map(|operator| operator.get_text()),
        right,
    )
}

/// The right operand of the xfx `**` must have priority < 200: a primary or a
/// negative number literal, but not the prefix-operator term `- 1`.
fn build_power_operand(
    ctx: &Power_operandContextAll<'_>,
    variables: &mut ClauseVariables,
) -> Result<Term> {
    if let Some(primary) = ctx.primary() {
        return build_primary(&primary, variables);
    }
    let minus = required(ctx.MINUS(), "negative number sign")?;
    let number = required(ctx.number(), "negative number")?;
    if minus.symbol.get_stop() + 1 != number.start().get_start() {
        bail!(
            "{}:{}: operator priority clash: the right operand of ** cannot be a prefix-operator term",
            minus.symbol.get_line(),
            minus.symbol.get_column()
        );
    }
    let number = match number.integer() {
        Some(integer) => build_integer(&integer)?,
        None => {
            return Err(unsupported_float(
                &*required(number.FLOAT(), "float token")?.symbol,
            ));
        }
    };
    Ok(negate(number))
}

/// The value of an unparenthesized number-literal primary, if it is one.
fn number_literal(ctx: &PrimaryContextAll<'_>) -> Result<Option<Term>> {
    match ctx {
        PrimaryContextAll::IntegerTermContext(integer) => Ok(Some(build_integer(&*required(
            integer.integer(),
            "integer token",
        )?)?)),
        PrimaryContextAll::FloatTermContext(float) => Err(unsupported_float(
            &*required(float.FLOAT(), "float token")?.symbol,
        )),
        _ => Ok(None),
    }
}

/// Negates a number literal; `-0` is the integer 0, as in SWI-Prolog.
fn negate(number: Term) -> Term {
    match number {
        Term::Integer { decimal } if decimal == "0" => Term::Integer { decimal },
        Term::Integer { decimal } => Term::Integer {
            decimal: format!("-{decimal}"),
        },
        other => unreachable!("negate called on non-number {other:?}"),
    }
}

/// Functional notation requires '(' immediately after the functor; the lexer
/// already turns '(' after whitespace into LAYOUT_LPAREN, this also rejects an
/// intervening comment.
fn require_functional_notation(functor_stop: isize, open: &impl Token) -> Result<()> {
    if functor_stop + 1 != open.get_start() {
        bail!(
            "{}:{}: operator expected: a functor must be followed immediately by '('",
            open.get_line(),
            open.get_column()
        );
    }
    Ok(())
}

fn build_primary(ctx: &PrimaryContextAll<'_>, variables: &mut ClauseVariables) -> Result<Term> {
    match ctx {
        PrimaryContextAll::CompoundTermContext(compound) => {
            let functor_ctx = required(compound.functor(), "compound functor")?;
            let open = required(compound.LPAREN(), "compound '('")?;
            require_functional_notation(functor_ctx.stop().get_stop(), &*open.symbol)?;
            let functor = match functor_ctx.atom() {
                Some(atom) => build_atom(&atom)?,
                None => required(functor_ctx.operator_name(), "operator functor")?.get_text(),
            };
            let args = build_terms(
                &*required(compound.term_list(), "compound arguments")?,
                variables,
            )?;
            Ok(Term::compound(functor, args))
        }
        PrimaryContextAll::VariableTermContext(variable) => {
            let token = required(variable.VARIABLE(), "variable token")?;
            Ok(variables.intern(&token.get_text()))
        }
        PrimaryContextAll::IntegerTermContext(integer) => {
            build_integer(&*required(integer.integer(), "integer token")?)
        }
        PrimaryContextAll::FloatTermContext(float) => Err(unsupported_float(
            &*required(float.FLOAT(), "float token")?.symbol,
        )),
        PrimaryContextAll::AtomTermContext(atom) => Ok(Term::atom(build_atom(&*required(
            atom.atom(),
            "atom token",
        )?)?)),
        PrimaryContextAll::StringTermContext(string) => Err(unsupported_text(
            &*required(string.DOUBLE_QUOTED_STRING(), "string token")?.symbol,
        )),
        PrimaryContextAll::BackStringTermContext(string) => Err(unsupported_text(
            &*required(string.BACK_QUOTED_STRING(), "back-quoted token")?.symbol,
        )),
        PrimaryContextAll::ListTermContext(list) => {
            build_list(&*required(list.list(), "list")?, variables)
        }
        PrimaryContextAll::ParenthesizedTermContext(parenthesized) => build_body(
            &*required(parenthesized.body(), "parenthesized body")?,
            variables,
        ),
        PrimaryContextAll::BracedTermContext(braced) => match braced.body() {
            Some(body) => Ok(Term::compound("{}", vec![build_body(&body, variables)?])),
            None => Ok(Term::atom("{}")),
        },
        PrimaryContextAll::CutTermContext(_) => Ok(Term::atom("!")),
        PrimaryContextAll::Error(_) => bail!("ANTLR produced an error primary context"),
    }
}

fn build_list(ctx: &ListContextAll<'_>, variables: &mut ClauseVariables) -> Result<Term> {
    let (items, tail) = match ctx {
        ListContextAll::EmptyListContext(_) => (None, None),
        ListContextAll::ProperListContext(list) => (list.term_list(), None),
        ListContextAll::ImproperListContext(list) => (list.term_list(), list.term999()),
        ListContextAll::Error(_) => bail!("ANTLR produced an error list context"),
    };
    Ok(Term::List {
        items: match items {
            Some(items) => build_terms(&items, variables)?,
            None => Vec::new(),
        },
        tail: match tail {
            Some(tail) => Some(Box::new(build_term999(&tail, variables)?)),
            None => None,
        },
    })
}

fn build_terms(
    ctx: &Term_listContextAll<'_>,
    variables: &mut ClauseVariables,
) -> Result<Vec<Term>> {
    ctx.term999_all()
        .iter()
        .map(|term| build_term999(term, variables))
        .collect()
}

fn build_atom(ctx: &AtomContextAll<'_>) -> Result<String> {
    match ctx.QUOTED_ATOM() {
        Some(quoted) => decode_quoted(&quoted.get_text(), '\''),
        None => Ok(ctx.get_text()),
    }
}

/// Floats have no exact field representation; policies use fixed-point integers.
fn unsupported_float(token: &impl Token<Data = str>) -> anyhow::Error {
    anyhow!(
        "{}:{}: float {} is not supported; write it as a fixed-point integer (e.g. 1.25 as 125 hundredths)",
        token.get_line(),
        token.get_column(),
        token.get_text()
    )
}

/// `"..."` strings and `` `...` `` code lists are not supported; identifiers are atoms.
fn unsupported_text(token: &impl Token<Data = str>) -> anyhow::Error {
    anyhow!(
        "{}:{}: {} is not supported; write identifiers as atoms in single quotes",
        token.get_line(),
        token.get_column(),
        token.get_text()
    )
}

fn build_integer(ctx: &IntegerContextAll<'_>) -> Result<Term> {
    let decimal = if let Some(token) = ctx.DECIMAL() {
        canonical_integer(&token.get_text(), 10, "")?
    } else if let Some(token) = ctx.BINARY() {
        canonical_integer(&token.get_text(), 2, "0b")?
    } else if let Some(token) = ctx.OCTAL() {
        canonical_integer(&token.get_text(), 8, "0o")?
    } else if let Some(token) = ctx.HEX() {
        canonical_integer(&token.get_text(), 16, "0x")?
    } else if let Some(token) = ctx.CHARACTER_CODE_CONSTANT() {
        decode_character_code(&token.get_text())?.to_string()
    } else {
        bail!("integer context has no token");
    };
    Ok(Term::Integer { decimal })
}

/// Left-associative `operand (OPERATOR operand)*` fold, using the operator
/// tokens collected by the grammar's `ops+=` label in source order.
fn fold_left<'input, C>(
    operands: Vec<Rc<C>>,
    operators: &[<LocalTokenFactory<'input> as TokenFactory<'input>>::Tok],
    variables: &mut ClauseVariables,
    build_operand: impl Fn(&C, &mut ClauseVariables) -> Result<Term>,
) -> Result<Term> {
    if operands.len() != operators.len() + 1 {
        bail!(
            "operator/operand count mismatch: {} operands, {} operators",
            operands.len(),
            operators.len()
        );
    }
    let mut operands = operands.into_iter();
    let first = operands.next().context("expression has no operand")?;
    let mut result = build_operand(&first, variables)?;
    for (operator, operand) in operators.iter().zip(operands) {
        result = Term::compound(
            operator.get_text(),
            vec![result, build_operand(&operand, variables)?],
        );
    }
    Ok(result)
}
