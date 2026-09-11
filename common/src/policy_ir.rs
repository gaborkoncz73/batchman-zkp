use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

pub const POLICY_SCHEMA_VERSION: u32 = 2;
pub const POLICY_DIALECT: &str = "batchman_policy_prolog_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyFile {
    pub schema_version: u32,
    pub dialect: String,
    pub module: Option<ModuleDecl>,
    pub directives: Vec<Term>,
    pub predicates: Vec<PredicateDef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleDecl {
    pub name: String,
    pub exports: Vec<PredicateIndicator>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateIndicator {
    pub name: String,
    pub arity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateDef {
    pub indicator: PredicateIndicator,
    pub clauses: Vec<Clause>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clause {
    pub ordinal: usize,
    pub variables: Vec<VariableDecl>,
    pub head: Call,
    pub body: Term,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariableDecl {
    pub id: u32,
    pub source_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Call {
    pub predicate: PredicateIndicator,
    pub args: Vec<Term>,
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
    Float {
        lexeme: String,
    },
    String {
        value: String,
    },
    BackQuoted {
        value: String,
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
        if self.schema_version != POLICY_SCHEMA_VERSION {
            return Err(format!(
                "unsupported policy schema version {}; expected {POLICY_SCHEMA_VERSION}",
                self.schema_version
            ));
        }
        if self.dialect != POLICY_DIALECT {
            return Err(format!(
                "unsupported policy dialect {:?}; expected {POLICY_DIALECT:?}",
                self.dialect
            ));
        }

        if let Some(module) = &self.module {
            let mut exports = BTreeSet::new();
            for export in &module.exports {
                if !exports.insert(export) {
                    return Err(format!(
                        "duplicate module export {}/{}",
                        export.name, export.arity
                    ));
                }
            }
        }

        let mut previous_indicator: Option<&PredicateIndicator> = None;
        let mut indicators = BTreeSet::new();
        for predicate in &self.predicates {
            if let Some(previous) = previous_indicator {
                if previous >= &predicate.indicator {
                    return Err("predicates must be strictly sorted by (name, arity)".to_owned());
                }
            }
            previous_indicator = Some(&predicate.indicator);

            if !indicators.insert(&predicate.indicator) {
                return Err(format!(
                    "duplicate predicate definition {}/{}",
                    predicate.indicator.name, predicate.indicator.arity
                ));
            }

            for (expected_ordinal, clause) in predicate.clauses.iter().enumerate() {
                if clause.ordinal != expected_ordinal {
                    return Err(format!(
                        "non-contiguous clause ordinal for {}/{}: expected {}, got {}",
                        predicate.indicator.name,
                        predicate.indicator.arity,
                        expected_ordinal,
                        clause.ordinal
                    ));
                }
                if clause.head.predicate != predicate.indicator {
                    return Err(format!(
                        "clause head {}/{} does not match containing predicate {}/{}",
                        clause.head.predicate.name,
                        clause.head.predicate.arity,
                        predicate.indicator.name,
                        predicate.indicator.arity
                    ));
                }
                if clause.head.args.len() != predicate.indicator.arity {
                    return Err(format!(
                        "head arity mismatch for {}/{}",
                        predicate.indicator.name, predicate.indicator.arity
                    ));
                }
                validate_clause_variables(clause)?;
            }
        }

        Ok(())
    }
}

fn validate_clause_variables(clause: &Clause) -> Result<(), String> {
    let declarations: BTreeMap<u32, &VariableDecl> = clause
        .variables
        .iter()
        .map(|decl| (decl.id, decl))
        .collect();
    if declarations.len() != clause.variables.len() {
        return Err("duplicate variable ID in clause".to_owned());
    }
    for (expected_id, declaration) in clause.variables.iter().enumerate() {
        if declaration.id as usize != expected_id {
            return Err(format!(
                "variable IDs must be contiguous from zero; expected {expected_id}, got {}",
                declaration.id
            ));
        }
    }

    let mut occurrences = BTreeMap::<u32, usize>::new();
    for term in clause.head.args.iter().chain(std::iter::once(&clause.body)) {
        collect_variable_occurrences(term, &mut occurrences);
    }
    for id in occurrences.keys() {
        if !declarations.contains_key(id) {
            return Err(format!("term refers to undeclared variable ID {id}"));
        }
    }
    for declaration in &clause.variables {
        if !occurrences.contains_key(&declaration.id) {
            return Err(format!(
                "variable ID {} is declared but never used",
                declaration.id
            ));
        }
        if declaration.source_name.is_none()
            && occurrences
                .get(&declaration.id)
                .copied()
                .unwrap_or_default()
                != 1
        {
            return Err(format!(
                "anonymous variable ID {} must occur exactly once",
                declaration.id
            ));
        }
    }

    Ok(())
}

fn collect_variable_occurrences(term: &Term, occurrences: &mut BTreeMap<u32, usize>) {
    match term {
        Term::Variable { id } => *occurrences.entry(*id).or_default() += 1,
        Term::Compound { args, .. } => {
            for arg in args {
                collect_variable_occurrences(arg, occurrences);
            }
        }
        Term::List { items, tail } => {
            for item in items {
                collect_variable_occurrences(item, occurrences);
            }
            if let Some(tail) = tail {
                collect_variable_occurrences(tail, occurrences);
            }
        }
        Term::Atom { .. }
        | Term::Integer { .. }
        | Term::Float { .. }
        | Term::String { .. }
        | Term::BackQuoted { .. } => {}
    }
}
