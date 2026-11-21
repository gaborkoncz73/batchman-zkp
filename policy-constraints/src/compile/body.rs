//! Clause bodies: alternatives and the classification of body goals.

use std::collections::BTreeSet;

use policy_ir::{PredicateIndicator, Term};

use super::builtin::is_builtin;
use super::show::show;
use crate::error::ModelError;

/// System predicates and control constructs the model does not support. A
/// policy may neither call nor redefine them. (`,`, `;` and `true` are
/// handled by [`alternatives`].)
const UNSUPPORTED: &[&str] = &[
    "!",
    "->",
    "*->",
    "\\+",
    "call",
    "not",
    "fail",
    "false",
    "once",
    "ignore",
    "forall",
    "findall",
    "bagof",
    "setof",
    "aggregate_all",
    "catch",
    "throw",
    "repeat",
    "halt",
    "\\=",
    "==",
    "\\==",
    "@<",
    "@>",
    "@=<",
    "@>=",
    "=..",
    "compare",
    "var",
    "nonvar",
    "atom",
    "number",
    "integer",
    "float",
    "atomic",
    "compound",
    "callable",
    "is_list",
    "ground",
    "functor",
    "arg",
    "copy_term",
    "length",
    "msort",
    "sort",
    "succ",
    "plus",
    "between",
    "atom_length",
    "atom_concat",
    "atom_codes",
    "atom_chars",
    "atom_number",
    "number_codes",
    "sub_atom",
    "assert",
    "asserta",
    "assertz",
    "retract",
    "write",
    "writeln",
    "print",
    "format",
    "nl",
];

/// The goal sequences of a body, one per way through its disjunctions,
/// ordered left to right. `true` contributes no goal.
pub(super) fn alternatives(body: &Term) -> Vec<Vec<&Term>> {
    match body {
        Term::Compound { functor, args } if functor == ";" && args.len() == 2 => {
            let mut alternatives = alternatives(&args[0]);
            alternatives.extend(self::alternatives(&args[1]));
            alternatives
        }
        Term::Compound { functor, args } if functor == "," && args.len() == 2 => {
            let right = alternatives(&args[1]);
            alternatives(&args[0])
                .into_iter()
                .flat_map(|left| {
                    right.iter().map(move |right| {
                        let mut goals = left.clone();
                        goals.extend(right);
                        goals
                    })
                })
                .collect()
        }
        Term::Atom { value } if value == "true" => vec![Vec::new()],
        goal => vec![vec![goal]],
    }
}

/// A callable term split into predicate name and arguments.
pub(super) struct Goal<'t> {
    pub name: &'t str,
    pub args: &'t [Term],
}

impl Goal<'_> {
    pub(super) fn indicator(&self) -> PredicateIndicator {
        PredicateIndicator {
            name: self.name.to_owned(),
            arity: self.args.len(),
        }
    }
}

pub(super) fn goal(term: &Term) -> Result<Goal<'_>, ModelError> {
    match term {
        Term::Atom { value } => Ok(Goal {
            name: value,
            args: &[],
        }),
        Term::Compound { functor, args } => Ok(Goal {
            name: functor,
            args,
        }),
        Term::Variable { .. } => Err(ModelError::new("a variable cannot be called as a goal")),
        other => Err(ModelError::new(format!("{} is not callable", show(other)))),
    }
}

/// Who answers a body goal.
pub(super) enum Callee {
    Policy,
    Builtin,
    External,
}

pub(super) fn classify(
    goal: &Goal,
    defined: &BTreeSet<PredicateIndicator>,
) -> Result<Callee, ModelError> {
    if is_builtin(goal.name, goal.args.len()) {
        Ok(Callee::Builtin)
    } else if is_reserved(goal.name) {
        Err(ModelError::new(format!(
            "{}/{} is not supported",
            goal.name,
            goal.args.len()
        )))
    } else if defined.contains(&goal.indicator()) {
        Ok(Callee::Policy)
    } else {
        Ok(Callee::External)
    }
}

/// A name the policy must not define a predicate with.
pub(super) fn is_reserved(name: &str) -> bool {
    UNSUPPORTED.contains(&name) || matches!(name, "," | ";" | "true" | "=" | "is")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conj(left: Term, right: Term) -> Term {
        Term::compound(",", vec![left, right])
    }

    fn disj(left: Term, right: Term) -> Term {
        Term::compound(";", vec![left, right])
    }

    fn names(alternatives: Vec<Vec<&Term>>) -> Vec<Vec<String>> {
        alternatives
            .into_iter()
            .map(|goals| goals.into_iter().map(show).collect())
            .collect()
    }

    #[test]
    fn conjunction_distributes_over_disjunction() {
        let a = Term::atom("a");
        let body = conj(
            disj(Term::atom("a"), Term::atom("b")),
            disj(Term::atom("c"), Term::atom("true")),
        );
        assert_eq!(
            names(alternatives(&body)),
            [vec!["a", "c"], vec!["a"], vec!["b", "c"], vec!["b"]]
        );
        assert_eq!(names(alternatives(&a)), [vec!["a"]]);
        assert_eq!(
            names(alternatives(&Term::atom("true"))),
            [Vec::<String>::new()]
        );
    }
}
