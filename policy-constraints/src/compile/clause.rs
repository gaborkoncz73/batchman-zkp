//! Branches of policy clauses.

use std::collections::BTreeMap;

use policy_ir::{Clause, PredicateIndicator, Term};

use super::body::{self, Goal};
use super::draft::BranchDraft;
use super::show::show;
use crate::branch::BranchKind;
use crate::error::ModelError;
use crate::layout::Cell;
use crate::position::{Position, Slot};
use crate::row::RowKind;
use crate::term::{GroundTerm, MAX_ARITY, TermTag};

/// The branch of one alternative of a clause, whose body goals are `goals`.
pub(super) fn clause_branch(
    predicate: &PredicateIndicator,
    ordinal: usize,
    clause: &Clause,
    alternative: usize,
    goals: &[&Term],
) -> Result<BranchDraft, ModelError> {
    check_arity(&predicate.name, predicate.arity)?;
    let goals: Vec<(Slot, Goal)> = goals
        .iter()
        .enumerate()
        .map(|(index, term)| {
            let goal = body::goal(term)?;
            check_arity(goal.name, goal.args.len())?;
            let slot = u8::try_from(index)
                .map(Slot::Child)
                .map_err(|_| ModelError::new("a clause body has more than 255 goals"))?;
            Ok((slot, goal))
        })
        .collect::<Result<_, ModelError>>()?;

    let kind = BranchKind::Clause {
        predicate: predicate.clone(),
        clause: ordinal,
        alternative,
        children: goals.len(),
    };
    let mut draft = BranchDraft::new(kind, goals.len());
    draft.goal_signature(Slot::Parent, &predicate.name, predicate.arity);
    draft.child_count();
    for (slot, goal) in &goals {
        draft.goal_signature(*slot, goal.name, goal.args.len());
    }

    let mut patterns = Patterns {
        draft: &mut draft,
        variables: BTreeMap::new(),
    };
    let goal_arguments = std::iter::once((Slot::Parent, clause.head.as_slice()))
        .chain(goals.iter().map(|(slot, goal)| (*slot, goal.args)));
    for (slot, args) in goal_arguments {
        for (index, argument) in args.iter().enumerate() {
            patterns.add(argument, Position::root(slot).child(index))?;
        }
    }
    let variables = patterns.variables;

    for (id, occurrences) in variables {
        let name = clause
            .variables
            .get(id as usize)
            .and_then(Option::as_deref)
            .unwrap_or("_");
        let (first, others) = occurrences.split_first().expect("recorded at least once");
        for other in others {
            draft.require_equal(
                RowKind::SharedVariable,
                Cell::Digest(first.clone()),
                Cell::Digest(other.clone()),
                format!("variable {name}"),
            );
        }
    }
    Ok(draft)
}

fn check_arity(name: &str, arity: usize) -> Result<(), ModelError> {
    if arity > MAX_ARITY {
        return Err(ModelError::new(format!(
            "{name}/{arity} exceeds the maximum arity {MAX_ARITY}"
        )));
    }
    Ok(())
}

/// Turns the terms written in a clause into rows: a variable-free subterm is
/// compared by digest, a constructor that contains variables by its shape,
/// and variable occurrences are collected for the equality rows.
struct Patterns<'d> {
    draft: &'d mut BranchDraft,
    variables: BTreeMap<u32, Vec<Position>>,
}

impl Patterns<'_> {
    fn add(&mut self, term: &Term, at: Position) -> Result<(), ModelError> {
        if !contains_variable(term) {
            let ground = GroundTerm::from_term(term).map_err(|error| error.context(&at))?;
            self.draft.require(
                RowKind::ExactSubterm,
                Cell::Digest(at),
                ground.digest(),
                format!("H({})", show(term)),
            );
            return Ok(());
        }
        match term {
            Term::Variable { id } => {
                self.variables.entry(*id).or_default().push(at);
                Ok(())
            }
            Term::Compound { functor, args } => {
                check_arity(functor, args.len()).map_err(|error| error.context(&at))?;
                self.draft.node_shape(
                    RowKind::Shape,
                    &at,
                    TermTag::Compound,
                    Some(functor),
                    args.len(),
                );
                for (index, argument) in args.iter().enumerate() {
                    self.add(argument, at.child(index))?;
                }
                Ok(())
            }
            Term::List { items, tail } => match items.split_first() {
                None => {
                    let tail = tail
                        .as_deref()
                        .expect("a list without items or tail is ground");
                    self.add(tail, at)
                }
                Some((head, rest)) => {
                    self.draft
                        .node_shape(RowKind::Shape, &at, TermTag::Cons, None, 2);
                    self.add(head, at.child(0))?;
                    let rest = Term::List {
                        items: rest.to_vec(),
                        tail: tail.clone(),
                    };
                    self.add(&rest, at.child(1))
                }
            },
            Term::Atom { .. } | Term::Integer { .. } => unreachable!("a leaf contains no variable"),
        }
    }
}

fn contains_variable(term: &Term) -> bool {
    match term {
        Term::Variable { .. } => true,
        Term::Compound { args, .. } => args.iter().any(contains_variable),
        Term::List { items, tail } => {
            items.iter().any(contains_variable) || tail.as_deref().is_some_and(contains_variable)
        }
        Term::Atom { .. } | Term::Integer { .. } => false,
    }
}
