//! Branches of builtin calls (`=/2`, `is/2` and the arithmetic comparisons).

use policy_ir::Term;

use super::body::Goal;
use super::draft::BranchDraft;
use super::show::show;
use crate::branch::{ArithmeticOp, BranchKind, BuiltinCheck, Comparison, Expression};
use crate::error::ModelError;
use crate::layout::Cell;
use crate::position::{Position, Slot};
use crate::row::RowKind;
use crate::term::TermTag;

pub(super) fn is_builtin(name: &str, arity: usize) -> bool {
    arity == 2 && (name == "=" || name == "is" || comparison(name).is_some())
}

fn comparison(name: &str) -> Option<Comparison> {
    Comparison::ALL
        .into_iter()
        .find(|relation| relation.functor() == name)
}

/// The form of a builtin call as written in a clause body. Variables are
/// operands whose values are only known in the proof tree.
pub(super) fn builtin_check(goal: &Goal) -> Result<BuiltinCheck, ModelError> {
    let parent = Position::root(Slot::Parent);
    let [left, right] = goal.args else {
        unreachable!("builtins have arity 2")
    };
    match goal.name {
        "=" => Ok(BuiltinCheck::Unify),
        "is" => {
            if !matches!(left, Term::Variable { .. } | Term::Integer { .. }) {
                return Err(ModelError::new(format!(
                    "the left side of is/2 must be a variable or an integer, not {}",
                    show(left)
                )));
            }
            Ok(BuiltinCheck::Evaluate {
                result: parent.child(0),
                expression: expression(right, parent.child(1))?,
            })
        }
        name => Ok(BuiltinCheck::Compare {
            relation: comparison(name).expect("is_builtin accepted the name"),
            left: expression(left, parent.child(0))?,
            right: expression(right, parent.child(1))?,
        }),
    }
}

fn expression(term: &Term, at: Position) -> Result<Expression, ModelError> {
    match term {
        Term::Variable { .. } | Term::Integer { .. } => Ok(Expression::Integer { at }),
        Term::Compound { functor, args } if args.len() == 2 => {
            let operator = ArithmeticOp::ALL
                .into_iter()
                .find(|operator| operator.functor() == functor)
                .ok_or_else(|| unsupported_expression(term))?;
            Ok(Expression::Apply {
                operator,
                left: Box::new(expression(&args[0], at.child(0))?),
                right: Box::new(expression(&args[1], at.child(1))?),
                at,
            })
        }
        _ => Err(unsupported_expression(term)),
    }
}

fn unsupported_expression(term: &Term) -> ModelError {
    ModelError::new(format!(
        "{} is not a supported arithmetic expression (integers, variables, + - * div)",
        show(term)
    ))
}

pub(super) fn builtin_branch(check: BuiltinCheck) -> BranchDraft {
    let mut draft = BranchDraft::new(BranchKind::Builtin(check.clone()), 0);
    draft.goal_signature(Slot::Parent, check.functor(), 2);
    draft.child_count();
    let parent = Position::root(Slot::Parent);
    match &check {
        BuiltinCheck::Unify => draft.require_equal(
            RowKind::Unification,
            Cell::Digest(parent.child(0)),
            Cell::Digest(parent.child(1)),
            "the arguments of =",
        ),
        BuiltinCheck::Evaluate { result, expression } => {
            integer_operand(&mut draft, result);
            expression_rows(&mut draft, expression);
        }
        BuiltinCheck::Compare { left, right, .. } => {
            expression_rows(&mut draft, left);
            expression_rows(&mut draft, right);
        }
    }
    draft
}

fn integer_operand(draft: &mut BranchDraft, at: &Position) {
    draft.require(
        RowKind::IntegerOperand,
        Cell::Tag(at.clone()),
        TermTag::Integer.field(),
        TermTag::Integer.name(),
    );
}

fn expression_rows(draft: &mut BranchDraft, expression: &Expression) {
    match expression {
        Expression::Integer { at } => integer_operand(draft, at),
        Expression::Apply {
            at,
            operator,
            left,
            right,
        } => {
            draft.node_shape(
                RowKind::Shape,
                at,
                TermTag::Compound,
                Some(operator.functor()),
                2,
            );
            expression_rows(draft, left);
            expression_rows(draft, right);
        }
    }
}
