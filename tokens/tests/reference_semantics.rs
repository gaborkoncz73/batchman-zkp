//! A small, circuit-independent oracle for local Prolog rule steps.
//!
//! This is deliberately test-only code.  It gives the soundness tests a
//! trustworthy expected result without reusing the legacy lowering or any
//! circuit witness calculation.

use std::collections::BTreeMap;

use common::{
    circuit_dialect::CircuitLimits,
    policy_compiler::compile_policy_for_current_circuit,
    policy_ir::{Call, Clause, Term},
};
use tokens::{lowering::to_circuit_v1_json, parse_goal, parse_policy, to_canonical_json};

#[derive(Clone, Debug, PartialEq, Eq)]
enum NormalTerm {
    Variable(u32),
    Atom(String),
    Integer(String),
    Float(String),
    String(String),
    BackQuoted(String),
    Compound(String, Vec<NormalTerm>),
    Nil,
    Cons(Box<NormalTerm>, Box<NormalTerm>),
}

impl NormalTerm {
    fn is_ground(&self) -> bool {
        match self {
            Self::Variable(_) => false,
            Self::Compound(_, args) => args.iter().all(Self::is_ground),
            Self::Cons(head, tail) => head.is_ground() && tail.is_ground(),
            Self::Atom(_)
            | Self::Integer(_)
            | Self::Float(_)
            | Self::String(_)
            | Self::BackQuoted(_)
            | Self::Nil => true,
        }
    }
}

fn normalize(term: &Term) -> NormalTerm {
    match term {
        Term::Variable { id } => NormalTerm::Variable(*id),
        Term::Atom { value } => NormalTerm::Atom(value.clone()),
        Term::Integer { decimal } => NormalTerm::Integer(decimal.clone()),
        Term::Float { lexeme } => NormalTerm::Float(lexeme.clone()),
        Term::String { value } => NormalTerm::String(value.clone()),
        Term::BackQuoted { value } => NormalTerm::BackQuoted(value.clone()),
        Term::Compound { functor, args } => {
            NormalTerm::Compound(functor.clone(), args.iter().map(normalize).collect())
        }
        Term::List { items, tail } => {
            let tail = tail.as_deref().map(normalize).unwrap_or(NormalTerm::Nil);
            items.iter().rev().fold(tail, |tail, item| {
                NormalTerm::Cons(Box::new(normalize(item)), Box::new(tail))
            })
        }
    }
}

fn normalize_call(call: &Call) -> NormalTerm {
    NormalTerm::Compound(
        call.predicate.name.clone(),
        call.args.iter().map(normalize).collect(),
    )
}

fn matches_pattern(
    pattern: &NormalTerm,
    value: &NormalTerm,
    bindings: &mut BTreeMap<u32, NormalTerm>,
) -> bool {
    if !value.is_ground() {
        return false;
    }

    match (pattern, value) {
        (NormalTerm::Variable(id), value) => match bindings.get(id) {
            Some(bound) => bound == value,
            None => {
                bindings.insert(*id, value.clone());
                true
            }
        },
        (NormalTerm::Atom(left), NormalTerm::Atom(right))
        | (NormalTerm::Integer(left), NormalTerm::Integer(right))
        | (NormalTerm::Float(left), NormalTerm::Float(right))
        | (NormalTerm::String(left), NormalTerm::String(right))
        | (NormalTerm::BackQuoted(left), NormalTerm::BackQuoted(right)) => left == right,
        (
            NormalTerm::Compound(left_name, left_args),
            NormalTerm::Compound(right_name, right_args),
        ) => {
            left_name == right_name
                && left_args.len() == right_args.len()
                && left_args
                    .iter()
                    .zip(right_args)
                    .all(|(left, right)| matches_pattern(left, right, bindings))
        }
        (NormalTerm::Nil, NormalTerm::Nil) => true,
        (NormalTerm::Cons(left_head, left_tail), NormalTerm::Cons(right_head, right_tail)) => {
            matches_pattern(left_head, right_head, bindings)
                && matches_pattern(left_tail, right_tail, bindings)
        }
        _ => false,
    }
}

fn body_branches(term: &Term) -> Vec<Vec<&Term>> {
    match term {
        Term::Compound { functor, args } if functor == ";" && args.len() == 2 => {
            let mut branches = body_branches(&args[0]);
            branches.extend(body_branches(&args[1]));
            branches
        }
        Term::Compound { functor, args } if functor == "," && args.len() == 2 => {
            let left = body_branches(&args[0]);
            let right = body_branches(&args[1]);
            left.into_iter()
                .flat_map(|left_branch| {
                    right.iter().cloned().map(move |right_branch| {
                        let mut branch = left_branch.clone();
                        branch.extend(right_branch);
                        branch
                    })
                })
                .collect()
        }
        Term::Atom { value } if value == "true" => vec![Vec::new()],
        goal => vec![vec![goal]],
    }
}

fn reference_accepts(clause: &Clause, parent: &Term, ordered_children: &[Term]) -> bool {
    let parent = normalize(parent);
    if !parent.is_ground() {
        return false;
    }

    body_branches(&clause.body).into_iter().any(|branch| {
        if branch.len() != ordered_children.len() {
            return false;
        }

        let mut bindings = BTreeMap::new();
        matches_pattern(&normalize_call(&clause.head), &parent, &mut bindings)
            && branch.iter().zip(ordered_children).all(|(pattern, value)| {
                matches_pattern(&normalize(pattern), &normalize(value), &mut bindings)
            })
    })
}

fn bounded_integer(value: u128, integer_bits: usize) -> Result<u128, &'static str> {
    let maximum = (1_u128 << integer_bits) - 1;
    if value <= maximum {
        Ok(value)
    } else {
        Err("integer is outside the configured unsigned range")
    }
}

fn eval_arithmetic(term: &Term, integer_bits: usize) -> Result<u128, &'static str> {
    let result = match term {
        Term::Integer { decimal } => decimal
            .parse::<u128>()
            .map_err(|_| "invalid integer literal")?,
        Term::Compound { functor, args } if args.len() == 2 => {
            let left = eval_arithmetic(&args[0], integer_bits)?;
            let right = eval_arithmetic(&args[1], integer_bits)?;
            match functor.as_str() {
                "+" => left.checked_add(right).ok_or("integer overflow")?,
                "-" => left.checked_sub(right).ok_or("integer overflow")?,
                "*" => left.checked_mul(right).ok_or("integer overflow")?,
                "div" if right == 0 => return Err("division by zero"),
                "div" => left.checked_div(right).ok_or("integer overflow")?,
                _ => return Err("unsupported arithmetic operator"),
            }
        }
        _ => return Err("arithmetic expression is not a ground integer expression"),
    };

    bounded_integer(result, integer_bits)
}

fn reference_builtin(term: &Term, integer_bits: usize) -> Result<bool, &'static str> {
    let Term::Compound { functor, args } = term else {
        return Err("builtin must be a compound term");
    };
    if args.len() != 2 {
        return Err("builtin must have arity two");
    }

    if functor == "=" {
        let left = normalize(&args[0]);
        let right = normalize(&args[1]);
        if !left.is_ground() || !right.is_ground() {
            return Err("reference builtin input must be ground");
        }
        return Ok(left == right);
    }

    let right = eval_arithmetic(&args[1], integer_bits)?;
    if functor == "is" {
        return Ok(eval_arithmetic(&args[0], integer_bits)? == right);
    }
    let left = eval_arithmetic(&args[0], integer_bits)?;
    match functor.as_str() {
        "<" => Ok(left < right),
        "=<" => Ok(left <= right),
        ">" => Ok(left > right),
        ">=" => Ok(left >= right),
        _ => Err("unsupported relation builtin"),
    }
}

fn goal(source: &str) -> Term {
    parse_goal(source, "reference-goal").unwrap()
}

fn only_clause(source: &str) -> Clause {
    parse_policy(source, "reference-policy.pl")
        .unwrap()
        .predicates
        .into_iter()
        .next()
        .unwrap()
        .clauses
        .into_iter()
        .next()
        .unwrap()
}

#[test]
fn current_policy_outputs_match_the_golden_files() {
    let policy = parse_policy(include_str!("../../prolog/policy.pl"), "prolog/policy.pl").unwrap();
    assert_eq!(
        to_canonical_json(&policy).unwrap(),
        include_str!("../../tests/fixtures/current-policy.rules.json")
    );

    let limits: CircuitLimits =
        serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap();
    let circuit_v1 = compile_policy_for_current_circuit(&policy, &limits).unwrap();
    assert_eq!(
        to_circuit_v1_json(circuit_v1.layout()).unwrap(),
        include_str!("../../tests/fixtures/current-policy.circuit-v1.json")
    );
}

#[test]
fn reference_oracle_rejects_literal_and_literal_type_changes() {
    let clause = only_clause("p(a, 1, \"1\") :- q(a).");
    assert!(reference_accepts(
        &clause,
        &goal("p(a, 1, \"1\")"),
        &[goal("q(a)")],
    ));
    assert!(!reference_accepts(
        &clause,
        &goal("p(b, 1, \"1\")"),
        &[goal("q(a)")],
    ));
    assert!(!reference_accepts(
        &clause,
        &goal("p(a, '1', \"1\")"),
        &[goal("q(a)")],
    ));
}

#[test]
fn reference_oracle_enforces_repeated_variables_and_ordered_children() {
    let clause = only_clause("p(X) :- q(X), r(X).");
    let parent = goal("p(a)");

    assert!(reference_accepts(
        &clause,
        &parent,
        &[goal("q(a)"), goal("r(a)")],
    ));
    assert!(!reference_accepts(
        &clause,
        &parent,
        &[goal("q(b)"), goal("r(a)")],
    ));
    assert!(!reference_accepts(
        &clause,
        &parent,
        &[goal("r(a)"), goal("q(a)")],
    ));
    assert!(!reference_accepts(&clause, &parent, &[goal("q(a)")]));
    assert!(!reference_accepts(
        &clause,
        &parent,
        &[goal("q(a)"), goal("r(a)"), goal("r(a)")],
    ));
}

#[test]
fn reference_oracle_preserves_open_list_tails_and_nested_compounds() {
    let clause = only_clause("p([H|T], f(X)) :- q(H, X), r(T).");
    let parent = goal("p([a,b], f(z))");

    assert!(reference_accepts(
        &clause,
        &parent,
        &[goal("q(a,z)"), goal("r([b])")],
    ));
    assert!(!reference_accepts(
        &clause,
        &parent,
        &[goal("q(a,z)"), goal("r([])")],
    ));
    assert!(!reference_accepts(
        &clause,
        &goal("p([a,b], g(z))"),
        &[goal("q(a,z)"), goal("r([b])")],
    ));
}

#[test]
fn reference_arithmetic_rejects_wrong_results_division_by_zero_and_range_overflow() {
    const INTEGER_BITS: usize = 32;

    assert_eq!(
        reference_builtin(&goal("10 is 2 * 5"), INTEGER_BITS),
        Ok(true)
    );
    assert_eq!(
        reference_builtin(&goal("9 is 2 * 5"), INTEGER_BITS),
        Ok(false)
    );
    assert_eq!(reference_builtin(&goal("1 < 2"), INTEGER_BITS), Ok(true));
    assert_eq!(reference_builtin(&goal("2 < 1"), INTEGER_BITS), Ok(false));
    assert_eq!(
        reference_builtin(&goal("0 is 1 div 0"), INTEGER_BITS),
        Err("division by zero")
    );
    assert_eq!(
        reference_builtin(&goal("4294967295 is 4294967295 + 1"), INTEGER_BITS),
        Err("integer is outside the configured unsigned range")
    );
}
