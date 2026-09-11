//! Checked baseline for the current circuit's constraint and row cost.
//!
//! Deliberate circuit changes should update these numbers in the same commit.
//! Unexpected changes then show up as a regression instead of silently making
//! proofs larger or weakening the constraint system.

use common::{
    circuit_dialect::CircuitLimits, data::UnificationInputFp,
    policy_compiler::compile_policy_for_current_circuit, policy_ir::PolicyFile,
    unification_checker_circuit::UnificationCircuit,
};
use halo2_proofs::{
    dev::{CircuitCost, CircuitGates},
    pasta::{Eq, Fp},
};
use std::collections::BTreeMap;

fn current_policy_circuit() -> UnificationCircuit {
    let policy: PolicyFile = serde_json::from_str(include_str!(
        "../../tests/fixtures/current-policy.rules.json"
    ))
    .unwrap();
    let limits: CircuitLimits =
        serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap();
    let (rules, typed_policy) = compile_policy_for_current_circuit(&policy, &limits)
        .unwrap()
        .into_circuit_parts();
    UnificationCircuit {
        rules,
        typed_policy,
        unif: UnificationInputFp::default(),
    }
}

#[test]
fn gate_count_baseline_is_explicit() {
    let summary = CircuitGates::collect::<Fp, UnificationCircuit>().to_string();
    let metrics = summary
        .lines()
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim(), value.trim()))
        .collect::<BTreeMap<_, _>>();

    assert_eq!(metrics.get("Total gates"), Some(&"30"));
    assert_eq!(
        metrics.get("Total custom constraint polynomials"),
        Some(&"165")
    );
    assert_eq!(metrics.get("Total negations"), Some(&"173"));
    assert_eq!(metrics.get("Total additions"), Some(&"351"));
    assert_eq!(metrics.get("Total multiplications"), Some(&"400"));
}

#[test]
fn current_policy_row_and_column_cost_baseline_is_explicit() {
    let cost = CircuitCost::<Eq, UnificationCircuit>::measure(16, &current_policy_circuit());
    let debug = format!("{cost:?}");

    let field = |name: &str| {
        let marker = format!("{name}: ");
        let start = debug
            .find(&marker)
            .unwrap_or_else(|| panic!("missing {name:?} in {debug}"))
            + marker.len();
        debug[start..].split([',', '}']).next().unwrap().trim()
    };
    for (name, expected) in [
        ("k", "16"),
        ("max_deg", "6"),
        ("lookups", "1"),
        ("permutation_cols", "42"),
        ("max_rows", "17168"),
        ("max_advice_rows", "17168"),
        ("max_fixed_rows", "17167"),
        ("num_fixed_columns", "22"),
        ("num_advice_columns", "37"),
        ("num_instance_columns", "1"),
        ("num_total_columns", "60"),
    ] {
        assert_eq!(
            field(name),
            expected,
            "changed circuit-cost field {name}; full measurement: {debug}"
        );
    }

    // This is halo2's analytical estimate, not the serialized proof length.
    let estimated_proof_bytes: usize = cost.proof_size(1).into();
    assert_eq!(estimated_proof_bytes, 9088);
}
