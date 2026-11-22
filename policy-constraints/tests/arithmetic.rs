//! Builtin arithmetic as step slots: the compiled steps, the completed
//! assignment, and the reference check that enforces the flagged operations.

mod support;

use ff::Field;
use policy_constraints::arith::{StepCell, StepOp, decode, holds};
use policy_constraints::term::integer;
use policy_constraints::{ArithmeticOp, Branch, BranchKind, ConstraintSystem, Fp};
use support::{assignment, completed, satisfied, system};

const POLICY: &str = "\
half_square(X, Y) :- Y is X * X div 2.
bigger(X, Y) :- X > Y.
";

fn builtin<'a>(system: &'a ConstraintSystem, label: &str) -> &'a Branch {
    system
        .branches()
        .iter()
        .find(|branch| {
            matches!(branch.kind, BranchKind::Builtin(_)) && branch.kind.to_string() == label
        })
        .unwrap_or_else(|| panic!("no branch {label}"))
}

fn step_cell(system: &ConstraintSystem, step: usize, part: StepCell) -> usize {
    system.layout().step_column(step, part)
}

#[test]
fn integers_decode_back_from_their_field_encoding() {
    for value in [0, 1, -1, 12, -931_220, i64::MAX, i64::MIN] {
        assert_eq!(decode(integer(value)), Some(value), "{value}");
    }
    assert_eq!(decode(Fp::from(1_u64 << 63)), None, "2^63 is too large");
    assert_eq!(
        decode(-Fp::from(1_u64 << 63) - Fp::ONE),
        None,
        "-2^63 - 1 is too small"
    );
}

#[test]
fn floor_division_agrees_with_the_builtin() {
    for x in [-9, -7, -1, 0, 1, 7, 9, i64::MIN, i64::MAX] {
        for y in [-3, -2, -1, 1, 2, 3] {
            let Ok(z) = ArithmeticOp::FloorDivide.apply(x, y) else {
                continue;
            };
            assert!(holds(StepOp::FloorDivide, x, y, z), "{x} div {y} = {z}");
            for wrong in [z.checked_add(1), z.checked_sub(1)].into_iter().flatten() {
                assert!(
                    !holds(StepOp::FloorDivide, x, y, wrong),
                    "{x} div {y} ≠ {wrong}"
                );
            }
        }
    }
    assert!(
        !holds(StepOp::FloorDivide, 5, 0, 0),
        "division by zero never holds"
    );
}

#[test]
fn an_expression_compiles_to_steps_in_post_order() {
    let system = system(POLICY);
    assert_eq!(system.layout().steps(), 2);
    let branch = builtin(&system, "builtin _ is ((_ * _) div _)");
    let ops: Vec<StepOp> = branch.steps.iter().map(|step| step.op).collect();
    assert_eq!(ops, [StepOp::Multiply, StepOp::FloorDivide]);

    let rows: Vec<&str> = branch
        .rows
        .iter()
        .map(|row| row.description.as_str())
        .collect();
    for expected in [
        "payload(P.0) = z[1]   (the value of the expression)",
        "x[0] = payload(P.1.0.0)   (operand)",
        "y[0] = payload(P.1.0.1)   (operand)",
        "mul[0] = 1",
        "x[1] = z[0]   (operand)",
        "y[1] = payload(P.1.1)   (operand)",
        "div[1] = 1",
        "add[1] = 0",
    ] {
        assert!(
            rows.contains(&expected),
            "{expected} missing from {rows:#?}"
        );
    }

    // `X > Y` is `Y < X`: one step, operands swapped, no result.
    let compare = builtin(&system, "builtin _ > _");
    assert_eq!(compare.steps.len(), 1);
    assert_eq!(compare.steps[0].op, StepOp::Less);
    let rows: Vec<&str> = compare
        .rows
        .iter()
        .map(|row| row.description.as_str())
        .collect();
    assert!(
        rows.contains(&"x[0] = payload(P.1)   (operand)"),
        "{rows:#?}"
    );
    assert!(rows.contains(&"z[0] = 0"), "{rows:#?}");
    assert!(
        rows.contains(&"x[1] = 0"),
        "the unused slot is zero: {rows:#?}"
    );
}

#[test]
fn honest_builtin_nodes_complete_and_pass() {
    let system = system(POLICY);
    for (goal, label) in [
        ("4 is 3*3 div 2", "builtin _ is ((_ * _) div _)"),
        ("-4 is -7*1 div 2", "builtin _ is ((_ * _) div _)"),
        ("5 > 3", "builtin _ > _"),
    ] {
        let honest = assignment(&system, goal, &[]);
        assert_eq!(satisfied(&system, &honest), [label], "{goal}");
        let w = completed(&system, builtin(&system, label), &honest).unwrap();
        system.layout().check_arithmetic(&w).unwrap();
    }
}

#[test]
fn a_wrong_quotient_is_caught_by_the_step() {
    // 5 is 3*3 div 2, with the rows satisfied: z[1] follows the false result.
    let system = system(POLICY);
    let branch = builtin(&system, "builtin _ is ((_ * _) div _)");
    let mut w = completed(&system, branch, &assignment(&system, "5 is 3*3 div 2", &[])).unwrap();
    // complete wrote the true z[1] = 4 although the goal says 5: a row breaks …
    assert!(!branch.is_satisfied_by(&w));
    // … so a cheating prover writes 5 there, and the step breaks instead.
    w[step_cell(&system, 1, StepCell::Z)] = integer(5);
    assert!(branch.is_satisfied_by(&w));
    let error = system
        .layout()
        .check_arithmetic(&w)
        .unwrap_err()
        .to_string();
    assert_eq!(error, "step 1: div does not hold for x = 9, y = 2, z = 5");
}

#[test]
fn a_false_comparison_is_caught_by_the_step() {
    let system = system(POLICY);
    let branch = builtin(&system, "builtin _ > _");
    let w = completed(&system, branch, &assignment(&system, "3 > 5", &[])).unwrap();
    assert!(branch.is_satisfied_by(&w), "the form of 3 > 5 is fine");
    let error = system
        .layout()
        .check_arithmetic(&w)
        .unwrap_err()
        .to_string();
    assert_eq!(error, "step 0: lt does not hold for x = 5, y = 3");
}

#[test]
fn an_overflowing_product_has_no_valid_assignment() {
    let system = system(POLICY);
    let big = 1_i64 << 40;
    let goal = format!("0 is {big}*{big} div 2");
    let honest = assignment(&system, &goal, &[]);
    let branch = builtin(&system, "builtin _ is ((_ * _) div _)");
    assert!(
        completed(&system, branch, &honest).is_none(),
        "the prover cannot compute it"
    );

    // Forging z[0] as the field product (2^80) is not a 64-bit integer.
    let mut forged = honest.clone();
    for (part, value) in [
        (StepCell::X, integer(big)),
        (StepCell::Y, integer(big)),
        (StepCell::Z, integer(big) * integer(big)),
        (StepCell::Flag(StepOp::Multiply), Fp::ONE),
    ] {
        forged[step_cell(&system, 0, part)] = value;
    }
    let error = system
        .layout()
        .check_arithmetic(&forged)
        .unwrap_err()
        .to_string();
    assert_eq!(error, "step 0: z is not a 64-bit integer");
}
