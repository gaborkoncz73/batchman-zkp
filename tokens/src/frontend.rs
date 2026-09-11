use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use antlr4rust::error_listener::ErrorListener;
use antlr4rust::errors::ANTLRError;
use antlr4rust::parser::Parser;
use antlr4rust::parser_rule_context::ParserRuleContext;
use antlr4rust::recognizer::Recognizer;
use antlr4rust::token::{TOKEN_EOF, Token};
use antlr4rust::tree::{ParseTree, TerminalNode};
use antlr4rust::{InputStream, common_token_stream::CommonTokenStream};
use anyhow::{Context, Result, anyhow, bail};
use common::policy_ir::{
    Call, Clause, ModuleDecl, POLICY_DIALECT, POLICY_SCHEMA_VERSION, PolicyFile, PredicateDef,
    PredicateIndicator, Term, VariableDecl,
};
use num_bigint::BigInt;

use crate::parser::prologlexer::prologLexer;
use crate::parser::prologparser::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyntaxDiagnostic {
    line: usize,
    column: usize,
    message: String,
}

#[derive(Clone, Default)]
struct CollectingErrorListener {
    diagnostics: Rc<RefCell<Vec<SyntaxDiagnostic>>>,
}

impl<'input, T> ErrorListener<'input, T> for CollectingErrorListener
where
    T: Recognizer<'input>,
{
    fn syntax_error(
        &self,
        _recognizer: &T,
        _offending_symbol: Option<
            &<T::TF as antlr4rust::token_factory::TokenFactory<'input>>::Inner,
        >,
        line: isize,
        column: isize,
        message: &str,
        _error: Option<&ANTLRError>,
    ) {
        self.diagnostics.borrow_mut().push(SyntaxDiagnostic {
            line: line.max(0) as usize,
            column: column.max(0) as usize,
            message: message.to_owned(),
        });
    }
}

#[derive(Default)]
struct ClauseVariables {
    named: BTreeMap<String, u32>,
    declarations: Vec<VariableDecl>,
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
        let id = self.declarations.len() as u32;
        self.declarations.push(VariableDecl { id, source_name });
        id
    }
}

struct ParsedClause {
    variables: Vec<VariableDecl>,
    head: Call,
    body: Term,
}

pub fn parse_policy(source: &str, source_name: &str) -> Result<PolicyFile> {
    let errors = CollectingErrorListener::default();

    let mut lexer = prologLexer::new(InputStream::new(source));
    lexer.remove_error_listeners();
    lexer.add_error_listener(Box::new(errors.clone()));

    let token_stream = CommonTokenStream::new(lexer);
    let mut parser = prologParser::new(token_stream);
    parser.remove_error_listeners();
    parser.add_error_listener(Box::new(errors.clone()));

    let tree_result = parser.p_text();
    let diagnostics = errors.diagnostics.borrow().clone();
    if !diagnostics.is_empty() {
        bail!(format_diagnostics(source_name, &diagnostics));
    }
    let tree = tree_result.map_err(|error| anyhow!("{source_name}: parser failure: {error:?}"))?;

    let mut directives = Vec::new();
    let mut parsed_clauses = Vec::new();
    for item in tree.source_item_all() {
        if let Some(directive) = item.directive() {
            let mut variables = ClauseVariables::default();
            let body = build_body(
                &*required(directive.body(), "directive body")?,
                &mut variables,
            )?;
            if !variables.declarations.is_empty() {
                bail!(
                    "{}:{}:{}: variables in directives are not supported by {}",
                    source_name,
                    directive.start().get_line(),
                    directive.start().get_column(),
                    POLICY_DIALECT
                );
            }
            if !matches!(
                &body,
                Term::Compound { functor, args } if functor == "module" && args.len() == 2
            ) {
                bail!(
                    "{}:{}:{}: unsupported directive in {}: {:?}",
                    source_name,
                    directive.start().get_line(),
                    directive.start().get_column(),
                    POLICY_DIALECT,
                    body
                );
            }
            directives.push(body);
        } else if let Some(clause) = item.clause() {
            parsed_clauses.push(build_clause(&clause)?);
        } else {
            bail!("{source_name}: internal parser error: empty source item");
        }
    }

    let module = extract_module_decl(&directives)?;
    let predicates = group_predicates(parsed_clauses);
    let policy = PolicyFile {
        schema_version: POLICY_SCHEMA_VERSION,
        dialect: POLICY_DIALECT.to_owned(),
        module,
        directives,
        predicates,
    };
    policy.validate().map_err(|error| anyhow!(error))?;
    Ok(policy)
}

/// Parses one proof-tree goal with the same grammar and AST builder as policy
/// bodies. The complete input must be consumed; partial parses are errors.
pub fn parse_goal(source: &str, source_name: &str) -> Result<Term> {
    if source.trim().is_empty() {
        bail!("{source_name}: goal must not be empty");
    }

    let errors = CollectingErrorListener::default();
    let mut lexer = prologLexer::new(InputStream::new(source));
    lexer.remove_error_listeners();
    lexer.add_error_listener(Box::new(errors.clone()));

    let token_stream = CommonTokenStream::new(lexer);
    let mut parser = prologParser::new(token_stream);
    parser.remove_error_listeners();
    parser.add_error_listener(Box::new(errors.clone()));

    let tree_result = parser.body();
    let diagnostics = errors.diagnostics.borrow().clone();
    if !diagnostics.is_empty() {
        bail!(format_diagnostics(source_name, &diagnostics));
    }
    let tree = tree_result.map_err(|error| anyhow!("{source_name}: parser failure: {error:?}"))?;
    let trailing = parser.get_current_token();
    if trailing.get_token_type() != TOKEN_EOF {
        bail!(
            "{}:{}:{}: unexpected trailing input {:?}",
            source_name,
            trailing.get_line(),
            trailing.get_column(),
            trailing.get_text()
        );
    }

    let mut variables = ClauseVariables::default();
    build_body(&tree, &mut variables)
}

pub fn to_canonical_json(policy: &PolicyFile) -> Result<String> {
    policy.validate().map_err(|error| anyhow!(error))?;
    let mut json = serde_json::to_string_pretty(policy)?;
    json.push('\n');
    Ok(json)
}

fn build_clause(ctx: &ClauseContextAll<'_>) -> Result<ParsedClause> {
    let mut variables = ClauseVariables::default();
    let head = build_callable(&*required(ctx.callable(), "clause head")?, &mut variables)?;
    let body = match ctx.body() {
        Some(body) => build_body(&body, &mut variables)?,
        None => Term::atom("true"),
    };

    Ok(ParsedClause {
        variables: variables.declarations,
        head,
        body,
    })
}

fn build_callable(ctx: &CallableContextAll<'_>, variables: &mut ClauseVariables) -> Result<Call> {
    let name = build_atom(&*required(ctx.atom(), "callable atom")?)?;
    let args = match ctx.argument_list() {
        Some(arguments) => build_argument_list(&arguments, variables)?,
        None => Vec::new(),
    };
    Ok(Call {
        predicate: PredicateIndicator {
            name,
            arity: args.len(),
        },
        args,
    })
}

fn build_body(ctx: &BodyContextAll<'_>, variables: &mut ClauseVariables) -> Result<Term> {
    build_disjunction(
        &*required(ctx.disjunction(), "body disjunction")?,
        variables,
    )
}

fn build_disjunction(
    ctx: &DisjunctionContextAll<'_>,
    variables: &mut ClauseVariables,
) -> Result<Term> {
    let left = build_if_then(
        &*required(ctx.if_then(), "disjunction left side")?,
        variables,
    )?;
    match ctx.disjunction() {
        Some(right) => Ok(Term::compound(
            ";",
            vec![left, build_disjunction(&right, variables)?],
        )),
        None => Ok(left),
    }
}

fn build_if_then(ctx: &If_thenContextAll<'_>, variables: &mut ClauseVariables) -> Result<Term> {
    let left = build_conjunction(
        &*required(ctx.conjunction(), "if-then left side")?,
        variables,
    )?;
    match ctx.if_then() {
        Some(right) => Ok(Term::compound(
            "->",
            vec![left, build_if_then(&right, variables)?],
        )),
        None => Ok(left),
    }
}

fn build_conjunction(
    ctx: &ConjunctionContextAll<'_>,
    variables: &mut ClauseVariables,
) -> Result<Term> {
    let left = build_term999(
        &*required(ctx.term999(), "conjunction left side")?,
        variables,
    )?;
    match ctx.conjunction() {
        Some(right) => Ok(Term::compound(
            ",",
            vec![left, build_conjunction(&right, variables)?],
        )),
        None => Ok(left),
    }
}

fn build_term999(ctx: &Term999ContextAll<'_>, variables: &mut ClauseVariables) -> Result<Term> {
    if ctx.NOT().is_some() {
        return Ok(Term::compound(
            "\\+",
            vec![build_term999(
                &*required(ctx.term999(), "negated term")?,
                variables,
            )?],
        ));
    }
    build_relation(
        &*required(ctx.relation_expr(), "priority-999 term")?,
        variables,
    )
}

fn build_relation(
    ctx: &Relation_exprContextAll<'_>,
    variables: &mut ClauseVariables,
) -> Result<Term> {
    let expressions = ctx.qualified_expr_all();
    let left = build_qualified(
        expressions
            .first()
            .context("relation is missing its left operand")?,
        variables,
    )?;
    match ctx.relation_op() {
        Some(operator) => {
            let right = build_qualified(
                expressions
                    .get(1)
                    .context("relation is missing its right operand")?,
                variables,
            )?;
            Ok(Term::compound(
                relation_operator(&operator)?,
                vec![left, right],
            ))
        }
        None => Ok(left),
    }
}

fn build_qualified(
    ctx: &Qualified_exprContextAll<'_>,
    variables: &mut ClauseVariables,
) -> Result<Term> {
    let left = build_additive(
        &*required(ctx.additive_expr(), "qualified expression")?,
        variables,
    )?;
    match ctx.qualified_expr() {
        Some(right) => Ok(Term::compound(
            ":",
            vec![left, build_qualified(&right, variables)?],
        )),
        None => Ok(left),
    }
}

fn build_additive(
    ctx: &Additive_exprContextAll<'_>,
    variables: &mut ClauseVariables,
) -> Result<Term> {
    let operands = ctx.multiplicative_expr_all();
    let operators = ordered_operators(vec![
        ctx.get_tokens(prolog_PLUS),
        ctx.get_tokens(prolog_MINUS),
        ctx.get_tokens(prolog_BIT_AND),
        ctx.get_tokens(prolog_BIT_OR),
    ]);
    fold_left(operands, operators, variables, build_multiplicative)
}

fn build_multiplicative(
    ctx: &Multiplicative_exprContextAll<'_>,
    variables: &mut ClauseVariables,
) -> Result<Term> {
    let operands = ctx.power_expr_all();
    let operators = ordered_operators(vec![
        ctx.get_tokens(prolog_STAR),
        ctx.get_tokens(prolog_SLASH),
        ctx.get_tokens(prolog_INT_DIV),
        ctx.get_tokens(prolog_DIV),
        ctx.get_tokens(prolog_MOD),
        ctx.get_tokens(prolog_REM),
        ctx.get_tokens(prolog_SHL),
        ctx.get_tokens(prolog_SHR),
    ]);
    fold_left(operands, operators, variables, build_power)
}

fn build_power(ctx: &Power_exprContextAll<'_>, variables: &mut ClauseVariables) -> Result<Term> {
    build_unary(
        &*required(ctx.unary_expr(), "power expression operand")?,
        variables,
    )
}

fn build_unary(ctx: &Unary_exprContextAll<'_>, variables: &mut ClauseVariables) -> Result<Term> {
    if ctx.PLUS().is_some() || ctx.MINUS().is_some() || ctx.BIT_NOT().is_some() {
        let operator = if ctx.PLUS().is_some() {
            "+"
        } else if ctx.MINUS().is_some() {
            "-"
        } else if ctx.BIT_NOT().is_some() {
            "\\"
        } else {
            bail!("unary expression is missing its operator");
        };
        return Ok(Term::compound(
            operator,
            vec![build_power(
                &*required(ctx.power_expr(), "unary operand")?,
                variables,
            )?],
        ));
    }

    let primaries = ctx.primary_all();
    let left = build_primary(
        primaries
            .first()
            .context("power expression has no left operand")?,
        variables,
    )?;
    if ctx.POW_XFX().is_some() {
        let right = build_primary(
            primaries
                .get(1)
                .context("** expression has no right operand")?,
            variables,
        )?;
        Ok(Term::compound("**", vec![left, right]))
    } else if ctx.POW_XFY().is_some() {
        Ok(Term::compound(
            "^",
            vec![
                left,
                build_power(&*required(ctx.power_expr(), "^ right operand")?, variables)?,
            ],
        ))
    } else {
        Ok(left)
    }
}

fn build_primary(ctx: &PrimaryContextAll<'_>, variables: &mut ClauseVariables) -> Result<Term> {
    match ctx {
        PrimaryContextAll::CompoundTermContext(compound) => {
            let functor = build_atom(&*required(compound.atom(), "compound functor")?)?;
            let args = build_argument_list(
                &*required(compound.argument_list(), "compound arguments")?,
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
        PrimaryContextAll::FloatTermContext(float) => Ok(Term::Float {
            lexeme: required(float.FLOAT(), "float token")?
                .get_text()
                .to_owned(),
        }),
        PrimaryContextAll::AtomTermContext(atom) => Ok(Term::atom(build_atom(&*required(
            atom.atom(),
            "atom token",
        )?)?)),
        PrimaryContextAll::StringTermContext(string) => Ok(Term::String {
            value: decode_quoted(
                &required(string.DOUBLE_QUOTED_STRING(), "string token")?.get_text(),
                '"',
            )?,
        }),
        PrimaryContextAll::BackStringTermContext(string) => Ok(Term::BackQuoted {
            value: decode_quoted(
                &required(string.BACK_QUOTED_STRING(), "back-quoted token")?.get_text(),
                '`',
            )?,
        }),
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
    match ctx {
        ListContextAll::EmptyListContext(_) => Ok(Term::List {
            items: Vec::new(),
            tail: None,
        }),
        ListContextAll::ProperListContext(list) => Ok(Term::List {
            items: build_list_items(
                &*required(list.list_items(), "proper-list items")?,
                variables,
            )?,
            tail: None,
        }),
        ListContextAll::ImproperListContext(list) => Ok(Term::List {
            items: build_list_items(
                &*required(list.list_items(), "improper-list items")?,
                variables,
            )?,
            tail: Some(Box::new(build_term999(
                &*required(list.term999(), "improper-list tail")?,
                variables,
            )?)),
        }),
        ListContextAll::Error(_) => bail!("ANTLR produced an error list context"),
    }
}

fn build_argument_list(
    ctx: &Argument_listContextAll<'_>,
    variables: &mut ClauseVariables,
) -> Result<Vec<Term>> {
    ctx.term999_all()
        .iter()
        .map(|term| build_term999(term, variables))
        .collect()
}

fn build_list_items(
    ctx: &List_itemsContextAll<'_>,
    variables: &mut ClauseVariables,
) -> Result<Vec<Term>> {
    ctx.term999_all()
        .iter()
        .map(|term| build_term999(term, variables))
        .collect()
}

fn build_atom(ctx: &AtomContextAll<'_>) -> Result<String> {
    if let Some(atom) = ctx.ATOM_ID() {
        Ok(atom.get_text().to_owned())
    } else if let Some(atom) = ctx.QUOTED_ATOM() {
        decode_quoted(&atom.get_text(), '\'')
    } else if let Some(atom) = ctx.GRAPHIC_ATOM() {
        Ok(atom.get_text().to_owned())
    } else {
        bail!("atom context has no token")
    }
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

fn relation_operator(ctx: &Relation_opContextAll<'_>) -> Result<&'static str> {
    let operator = if ctx.UNIFY().is_some() {
        "="
    } else if ctx.NOT_UNIFIABLE().is_some() {
        "\\="
    } else if ctx.IDENTICAL().is_some() {
        "=="
    } else if ctx.NOT_IDENTICAL().is_some() {
        "\\=="
    } else if ctx.TERM_LT().is_some() {
        "@<"
    } else if ctx.TERM_LE().is_some() {
        "@=<"
    } else if ctx.TERM_GT().is_some() {
        "@>"
    } else if ctx.TERM_GE().is_some() {
        "@>="
    } else if ctx.UNIV().is_some() {
        "=.."
    } else if ctx.IS().is_some() {
        "is"
    } else if ctx.ARITH_EQ().is_some() {
        "=:="
    } else if ctx.ARITH_NE().is_some() {
        "=\\="
    } else if ctx.LT().is_some() {
        "<"
    } else if ctx.LE().is_some() {
        "=<"
    } else if ctx.GT().is_some() {
        ">"
    } else if ctx.GE().is_some() {
        ">="
    } else {
        bail!("relation operator context has no operator");
    };
    Ok(operator)
}

fn fold_left<'input, C, F>(
    operands: Vec<Rc<C>>,
    operators: Vec<String>,
    variables: &mut ClauseVariables,
    build_operand: F,
) -> Result<Term>
where
    F: Fn(&C, &mut ClauseVariables) -> Result<Term>,
{
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
    for (operator, operand) in operators.into_iter().zip(operands) {
        result = Term::compound(operator, vec![result, build_operand(&operand, variables)?]);
    }
    Ok(result)
}

fn ordered_operators<'input>(
    groups: Vec<Vec<Rc<TerminalNode<'input, prologParserContextType>>>>,
) -> Vec<String> {
    let mut operators: Vec<_> = groups.into_iter().flatten().collect();
    operators.sort_by_key(|operator| operator.symbol.get_token_index());
    operators
        .into_iter()
        .map(|operator| operator.get_text().to_owned())
        .collect()
}

fn group_predicates(clauses: Vec<ParsedClause>) -> Vec<PredicateDef> {
    let mut grouped = BTreeMap::<PredicateIndicator, Vec<ParsedClause>>::new();
    for clause in clauses {
        grouped
            .entry(clause.head.predicate.clone())
            .or_default()
            .push(clause);
    }

    grouped
        .into_iter()
        .map(|(indicator, clauses)| PredicateDef {
            indicator,
            clauses: clauses
                .into_iter()
                .enumerate()
                .map(|(ordinal, clause)| Clause {
                    ordinal,
                    variables: clause.variables,
                    head: clause.head,
                    body: clause.body,
                })
                .collect(),
        })
        .collect()
}

fn extract_module_decl(directives: &[Term]) -> Result<Option<ModuleDecl>> {
    let mut module = None;
    for directive in directives {
        let Term::Compound { functor, args } = directive else {
            continue;
        };
        if functor != "module" {
            continue;
        }
        if module.is_some() {
            bail!("multiple module/2 directives are not allowed");
        }
        if args.len() != 2 {
            bail!("module directive must have arity 2");
        }
        let name = atom_value(&args[0])?.to_owned();
        let Term::List {
            items: exports,
            tail: None,
        } = &args[1]
        else {
            bail!("module export list must be a proper list");
        };
        let exports = exports
            .iter()
            .map(predicate_indicator)
            .collect::<Result<Vec<_>>>()?;
        module = Some(ModuleDecl { name, exports });
    }
    Ok(module)
}

fn predicate_indicator(term: &Term) -> Result<PredicateIndicator> {
    let Term::Compound { functor, args } = term else {
        bail!("module export must be a Name/Arity predicate indicator");
    };
    if functor != "/" || args.len() != 2 {
        bail!("module export must be a Name/Arity predicate indicator");
    }
    let name = atom_value(&args[0])?.to_owned();
    let Term::Integer { decimal } = &args[1] else {
        bail!("predicate-indicator arity must be an integer");
    };
    let arity = decimal
        .parse::<usize>()
        .context("predicate-indicator arity does not fit usize")?;
    Ok(PredicateIndicator { name, arity })
}

fn atom_value(term: &Term) -> Result<&str> {
    match term {
        Term::Atom { value } => Ok(value),
        _ => bail!("expected an atom"),
    }
}

fn canonical_integer(text: &str, radix: u32, prefix: &str) -> Result<String> {
    let digits = text
        .strip_prefix(prefix)
        .context("integer has an invalid radix prefix")?;
    let value = BigInt::parse_bytes(digits.as_bytes(), radix)
        .context("integer literal is outside the supported syntax")?;
    Ok(value.to_string())
}

fn decode_character_code(text: &str) -> Result<u32> {
    let encoded = text
        .strip_prefix("0'")
        .context("invalid character-code constant")?;
    let decoded = decode_content(encoded, None)?;
    let mut characters = decoded.chars();
    let character = characters.next().context("empty character-code constant")?;
    if characters.next().is_some() {
        bail!("character-code constant must decode to exactly one character");
    }
    Ok(character as u32)
}

fn decode_quoted(text: &str, quote: char) -> Result<String> {
    if !text.starts_with(quote) || !text.ends_with(quote) || text.len() < 2 {
        bail!("invalid quoted literal");
    }
    let inner = &text[quote.len_utf8()..text.len() - quote.len_utf8()];
    decode_content(inner, Some(quote))
}

fn decode_content(input: &str, doubled_quote: Option<char>) -> Result<String> {
    let chars: Vec<char> = input.chars().collect();
    let mut output = String::new();
    let mut index = 0;
    while index < chars.len() {
        let current = chars[index];
        if Some(current) == doubled_quote && chars.get(index + 1).copied() == doubled_quote {
            output.push(current);
            index += 2;
            continue;
        }
        if current != '\\' {
            output.push(current);
            index += 1;
            continue;
        }

        let escaped = *chars.get(index + 1).context("trailing escape")?;
        match escaped {
            '\n' => index += 2,
            '\r' if chars.get(index + 2) == Some(&'\n') => index += 3,
            'a' => {
                output.push('\u{0007}');
                index += 2;
            }
            'b' => {
                output.push('\u{0008}');
                index += 2;
            }
            'r' => {
                output.push('\r');
                index += 2;
            }
            'f' => {
                output.push('\u{000c}');
                index += 2;
            }
            't' => {
                output.push('\t');
                index += 2;
            }
            'n' => {
                output.push('\n');
                index += 2;
            }
            'v' => {
                output.push('\u{000b}');
                index += 2;
            }
            'x' => {
                let (character, next) = decode_radix_escape(&chars, index + 2, 16)?;
                output.push(character);
                index = next;
            }
            '0'..='7' => {
                let (character, next) = decode_radix_escape(&chars, index + 1, 8)?;
                output.push(character);
                index = next;
            }
            other => {
                output.push(other);
                index += 2;
            }
        }
    }
    Ok(output)
}

fn decode_radix_escape(chars: &[char], start: usize, radix: u32) -> Result<(char, usize)> {
    let mut end = start;
    while end < chars.len() && chars[end] != '\\' {
        end += 1;
    }
    if end == start || end == chars.len() {
        bail!("numeric escape must end with a backslash");
    }
    let digits: String = chars[start..end].iter().collect();
    let value = u32::from_str_radix(&digits, radix).context("invalid numeric escape")?;
    let character = char::from_u32(value).context("numeric escape is not a Unicode scalar")?;
    Ok((character, end + 1))
}

fn required<T>(value: Option<T>, description: &str) -> Result<T> {
    value.with_context(|| format!("ANTLR parse tree is missing {description}"))
}

fn format_diagnostics(source_name: &str, diagnostics: &[SyntaxDiagnostic]) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| {
            format!(
                "{source_name}:{}:{}: {}",
                diagnostic.line, diagnostic.column, diagnostic.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_literals_are_decoded() {
        assert_eq!(decode_quoted("'can''t'", '\'').unwrap(), "can't");
        assert_eq!(decode_quoted(r#""line\nnext""#, '"').unwrap(), "line\nnext");
        assert_eq!(decode_quoted(r#"'\x41\'"#, '\'').unwrap(), "A");
    }

    #[test]
    fn radix_integers_are_canonical_decimal() {
        assert_eq!(canonical_integer("0b00101", 2, "0b").unwrap(), "5");
        assert_eq!(canonical_integer("0x10", 16, "0x").unwrap(), "16");
    }
}
