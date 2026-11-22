//! The arithmetic chip decides like the reference
//! (`Layout::check_arithmetic`): honest builtin nodes pass, and every false
//! or out-of-range step fails, in the regions of that step.

use ff::Field;
use halo2_poseidon::poseidon::primitives::P128Pow5T3;
use halo2_proofs::dev::MockProver;
use node_circuit::{BranchTable, Claim, NodeBranch, NodeCircuit, minimum_k};
use policy_constraints::arith::{StepCell, StepOp};
use policy_constraints::term::integer;
use policy_constraints::{ConstraintSystem, Fp, GroundTerm, compile};
use policy_parser::{parse_goal, parse_policy};

const POLICY: &str = "\
half_square(X, Y) :- Y is X * X div 2.
bigger(X, Y) :- X > Y.
next(X, Y) :- Y is X + 1.
";
const DIV: &str = "builtin _ is ((_ * _) div _)";
const GREATER: &str = "builtin _ > _";

fn system() -> ConstraintSystem {
    compile(&parse_policy(POLICY, "test.pl").unwrap()).unwrap()
}

fn branch(system: &ConstraintSystem, label: &str) -> usize {
    system
        .branches()
        .iter()
        .find(|branch| branch.kind.to_string() == label)
        .unwrap_or_else(|| panic!("no branch {label}"))
        .id
}

/// The goal cells of `goal`; not yet completed for a branch.
fn goals(system: &ConstraintSystem, goal: &str) -> Vec<Fp> {
    let goal = GroundTerm::from_term(&parse_goal(goal, "goal").unwrap()).unwrap();
    system.layout().assign(&goal, &[]).unwrap()
}

fn completed(system: &ConstraintSystem, goal: &str, branch: usize) -> Vec<Fp> {
    let mut w = goals(system, goal);
    system.complete(branch, &mut w, Fp::ZERO).unwrap();
    w
}

fn set(system: &ConstraintSystem, w: &mut [Fp], step: usize, part: StepCell, value: Fp) {
    w[system.layout().step_column(step, part)] = value;
}

/// The circuit's failures on one node proved by `branch`.
fn failures(system: &ConstraintSystem, w: Vec<Fp>, branch: usize) -> Vec<String> {
    let circuit = NodeCircuit::<Fp, P128Pow5T3>::new(
        system.layout().clone(),
        BranchTable::from_policy(system),
        vec![NodeBranch {
            assignment: w,
            claim: Claim::Branch(branch),
        }],
    );
    // The policy has no external facts: the public list is empty.
    match MockProver::run(minimum_k(&circuit), &circuit, vec![vec![]])
        .unwrap()
        .verify()
    {
        Ok(()) => Vec::new(),
        Err(failures) => failures.iter().map(ToString::to_string).collect(),
    }
}

/// The reference rejects `w` with `reference`, and the circuit fails only in
/// the regions of `step`, one of them `region`.
fn both_reject(
    system: &ConstraintSystem,
    w: Vec<Fp>,
    branch: usize,
    reference: &str,
    region: &str,
) {
    let error = system
        .layout()
        .check_arithmetic(&w)
        .unwrap_err()
        .to_string();
    assert_eq!(error, reference);
    let failures = failures(system, w, branch);
    let step = &region[..6];
    assert!(!failures.is_empty(), "the circuit accepts: {reference}");
    assert!(
        failures
            .iter()
            .all(|failure| failure.contains(&format!("('{step}"))),
        "{failures:#?}"
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains(&format!("('{region}')"))),
        "{failures:#?}"
    );
}

#[test]
fn honest_builtin_nodes_are_accepted() {
    let system = system();
    for (goal, label) in [
        ("4 is 3*3 div 2", DIV),
        ("-4 is -7*1 div 2", DIV),
        ("-4 is 7*1 div -2", DIV),
        ("3 is -7*1 div -2", DIV),
        ("5 > 3", GREATER),
        ("-3 > -5", GREATER),
        (
            "9223372036854775807 is 9223372036854775806 + 1",
            "builtin _ is (_ + _)",
        ),
        (
            "-9223372036854775808 is -9223372036854775807 + -1",
            "builtin _ is (_ + _)",
        ),
    ] {
        let id = branch(&system, label);
        let w = completed(&system, goal, id);
        system.layout().check_arithmetic(&w).unwrap();
        assert_eq!(failures(&system, w, id), Vec::<String>::new(), "{goal}");
    }
}

#[test]
fn a_wrong_quotient_is_rejected() {
    // 5 is 3*3 div 2 with z[1] = 5: the remainder 9 − 5·2 = −1 is negative.
    let system = system();
    let id = branch(&system, DIV);
    let mut w = completed(&system, "5 is 3*3 div 2", id);
    set(&system, &mut w, 1, StepCell::Z, integer(5));
    both_reject(
        &system,
        w,
        id,
        "step 1: div does not hold for x = 9, y = 2, z = 5",
        "step 1: range 4",
    );
}

#[test]
fn a_false_comparison_is_rejected() {
    // 3 > 5 is 5 < 3: the difference 3 − 5 − 1 is negative.
    let system = system();
    let id = branch(&system, GREATER);
    let w = completed(&system, "3 > 5", id);
    both_reject(
        &system,
        w,
        id,
        "step 0: lt does not hold for x = 5, y = 3",
        "step 0: range 3",
    );
}

#[test]
fn a_division_by_zero_is_rejected() {
    // 0 is 5*1 div 0: the prover cannot compute it, so it forges the step.
    let system = system();
    let id = branch(&system, DIV);
    let mut w = goals(&system, "0 is 5*1 div 0");
    assert!(system.complete(id, &mut w, Fp::ZERO).is_err());
    for (step, x, y) in [(0, 5, 1), (1, 5, 0)] {
        set(&system, &mut w, step, StepCell::X, integer(x));
        set(&system, &mut w, step, StepCell::Y, integer(y));
    }
    set(&system, &mut w, 0, StepCell::Z, integer(5));
    set(
        &system,
        &mut w,
        0,
        StepCell::Flag(StepOp::Multiply),
        Fp::ONE,
    );
    set(
        &system,
        &mut w,
        1,
        StepCell::Flag(StepOp::FloorDivide),
        Fp::ONE,
    );
    both_reject(
        &system,
        w,
        id,
        "step 1: div does not hold for x = 5, y = 0, z = 0",
        "step 1: range 3",
    );
}

#[test]
fn an_overflowing_result_is_rejected() {
    // 2⁴⁰ · 2⁴⁰ = 2⁸⁰ in the field, but not as a 64-bit integer.
    let system = system();
    let id = branch(&system, DIV);
    let big = 1_i64 << 40;
    let mut w = goals(&system, &format!("0 is {big}*{big} div 2"));
    assert!(system.complete(id, &mut w, Fp::ZERO).is_err());
    for (step, part, value) in [
        (0, StepCell::X, integer(big)),
        (0, StepCell::Y, integer(big)),
        (0, StepCell::Z, integer(big) * integer(big)),
        (0, StepCell::Flag(StepOp::Multiply), Fp::ONE),
    ] {
        set(&system, &mut w, step, part, value);
    }
    assert_eq!(
        system
            .layout()
            .check_arithmetic(&w)
            .unwrap_err()
            .to_string(),
        "step 0: z is not a 64-bit integer"
    );
    let failures = failures(&system, w, id);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("('step 0: range 2')")),
        "{failures:#?}"
    );
}
