//! Semantic IR of a parsed policy: the format of `input/rules.json`.
//!
//! `policy-parser` produces it from Prolog source; `policy-constraints` reads
//! it. Clauses keep their source order, variables are numbered per clause,
//! and operators stay ordinary compound terms (`A + B` is `+(A, B)`).

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyFile {
    pub predicates: Vec<PredicateDef>,
}

/// A predicate and its clauses in source order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateDef {
    pub indicator: PredicateIndicator,
    pub clauses: Vec<Clause>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateIndicator {
    pub name: String,
    pub arity: usize,
}

/// One clause `head :- body` of the containing predicate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clause {
    /// Source name of each variable; `Term::Variable { id }` indexes this
    /// list. `None` is an anonymous `_`.
    pub variables: Vec<Option<String>>,
    /// The head arguments; the functor is the containing predicate's.
    pub head: Vec<Term>,
    pub body: Term,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Term {
    Variable {
        id: u32,
    },
    Atom {
        value: String,
    },
    Integer {
        decimal: String,
    },
    Compound {
        functor: String,
        args: Vec<Term>,
    },
    List {
        items: Vec<Term>,
        tail: Option<Box<Term>>,
    },
}

impl Term {
    pub fn compound(functor: impl Into<String>, args: Vec<Term>) -> Self {
        Self::Compound {
            functor: functor.into(),
            args,
        }
    }

    pub fn atom(value: impl Into<String>) -> Self {
        Self::Atom {
            value: value.into(),
        }
    }

    pub fn as_compound(&self) -> Option<(&str, &[Term])> {
        match self {
            Self::Compound { functor, args } => Some((functor, args)),
            _ => None,
        }
    }
}

impl PolicyFile {
    pub fn validate(&self) -> Result<(), String> {
        // Strictly sorted also means no predicate is defined twice.
        let mut previous_indicator: Option<&PredicateIndicator> = None;
        for predicate in &self.predicates {
            let PredicateIndicator { name, arity } = &predicate.indicator;
            if previous_indicator.is_some_and(|previous| previous >= &predicate.indicator) {
                return Err(format!(
                    "predicates must be strictly sorted by (name, arity) and unique; {name}/{arity} is out of place"
                ));
            }
            previous_indicator = Some(&predicate.indicator);

            for (index, clause) in predicate.clauses.iter().enumerate() {
                if clause.head.len() != *arity {
                    return Err(format!(
                        "{name}/{arity} clause {index}: the head has {} arguments",
                        clause.head.len()
                    ));
                }
                validate_clause_variables(clause)
                    .map_err(|message| format!("{name}/{arity} clause {index}: {message}"))?;
            }
        }
        Ok(())
    }
}

fn validate_clause_variables(clause: &Clause) -> Result<(), String> {
    let mut names = BTreeSet::new();
    for name in clause.variables.iter().flatten() {
        if name.is_empty() || !names.insert(name) {
            return Err(format!("variable name {name:?} is empty or repeated"));
        }
    }

    let mut occurrences = vec![0_usize; clause.variables.len()];
    for term in clause.head.iter().chain(std::iter::once(&clause.body)) {
        count_variable_occurrences(term, &mut occurrences)?;
    }
    for (id, (name, count)) in clause.variables.iter().zip(&occurrences).enumerate() {
        match (name, count) {
            (_, 0) => return Err(format!("variable {id} is declared but never used")),
            (None, 2..) => return Err(format!("anonymous variable {id} occurs more than once")),
            _ => {}
        }
    }
    Ok(())
}

fn count_variable_occurrences(term: &Term, occurrences: &mut [usize]) -> Result<(), String> {
    match term {
        Term::Variable { id } => {
            let count = occurrences
                .get_mut(*id as usize)
                .ok_or_else(|| format!("term refers to undeclared variable {id}"))?;
            *count += 1;
        }
        Term::Compound { args, .. } => {
            for arg in args {
                count_variable_occurrences(arg, occurrences)?;
            }
        }
        Term::List { items, tail } => {
            for item in items.iter().chain(tail.as_deref()) {
                count_variable_occurrences(item, occurrences)?;
            }
        }
        Term::Atom { .. } | Term::Integer { .. } => {}
    }
    Ok(())
}
