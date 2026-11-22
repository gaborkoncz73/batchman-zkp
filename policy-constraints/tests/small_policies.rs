//! The model on small hand-written policies.

mod support;

use ff::Field;
use policy_constraints::Fp;
use policy_constraints::{Branch, ConstraintSystem};
use policy_constraints::{BranchKind, RowKind};
use support::{
    assignment, compile_error, completed, compressed_residual, satisfied, system, violated,
};

const EXAMPLE: &str = "p(X, a) :- q(X).\n";

/// The assignment of a node proving `parent` from `children`, completed for
/// `branch` (so that its `is_clause` and `is_fact` cells are the branch's).
fn assigned(
    system: &ConstraintSystem,
    branch: &Branch,
    parent: &str,
    children: &[&str],
) -> Vec<Fp> {
    completed(system, branch, &assignment(system, parent, children)).expect("steps compute")
}

#[test]
fn example_clause_has_the_expected_rows() {
    let system = system(EXAMPLE);
    let labels: Vec<String> = system
        .branches()
        .iter()
        .map(|b| b.kind.to_string())
        .collect();
    assert_eq!(labels, ["clause p/2 #0 alternative 0", "external fact q/1"]);

    let rows: Vec<&str> = system.branches()[0]
        .rows
        .iter()
        .map(|row| row.description.as_str())
        .collect();
    assert_eq!(
        rows,
        [
            "tag(P) = compound",
            "payload(P) = p",
            "arity(P) = 2",
            "children = 1",
            "tag(C0) = compound",
            "payload(C0) = q",
            "arity(C0) = 1",
            "digest(P.1) = H(a)",
            "digest(P.0) = digest(C0.0)   (variable X)",
            // q is an external fact, so every branch says whether it is one.
            "is_fact = 0",
            "is_clause = 1",
        ]
    );

    let fact_rows: Vec<&str> = system.branches()[1]
        .rows
        .iter()
        .map(|row| row.description.as_str())
        .collect();
    assert_eq!(
        fact_rows,
        [
            "tag(P) = compound",
            "payload(P) = q",
            "arity(P) = 1",
            "children = 0",
            "tag(C0) = absent",
            "is_fact = 1",
            "is_clause = 0",
        ]
    );
}

#[test]
fn honest_node_satisfies_exactly_its_branch() {
    let system = system(EXAMPLE);
    let honest = assignment(&system, "p(5, a)", &["q(5)"]);
    assert_eq!(satisfied(&system, &honest), ["clause p/2 #0 alternative 0"]);
    let fact = assignment(&system, "q(5)", &[]);
    assert_eq!(satisfied(&system, &fact), ["external fact q/1"]);
}

#[test]
fn each_kind_of_cheat_breaks_the_row_that_checks_it() {
    let system = system(EXAMPLE);
    let clause = &system.branches()[0];
    let cases: [(&str, &[&str], &[&str]); 4] = [
        ("p(5, b)", &["q(5)"], &["digest(P.1) = H(a)"]),
        (
            "p(5, a)",
            &["q(7)"],
            &["digest(P.0) = digest(C0.0)   (variable X)"],
        ),
        ("p(5, a)", &["r(5)"], &["payload(C0) = q"]),
        (
            "p(5, a)",
            &[],
            &[
                "children = 1",
                "tag(C0) = compound",
                "payload(C0) = q",
                "arity(C0) = 1",
                "digest(P.0) = digest(C0.0)   (variable X)",
            ],
        ),
    ];
    for (parent, children, expected) in cases {
        let cheat = assigned(&system, clause, parent, children);
        assert_eq!(violated(clause, &cheat), expected, "{parent} {children:?}");
        assert!(
            satisfied(&system, &cheat).is_empty(),
            "{parent} {children:?}"
        );
    }
}

#[test]
fn compressed_branch_agrees_with_the_rows() {
    let system = system(EXAMPLE);
    let clause = &system.branches()[0];
    let honest = assigned(&system, clause, "p(5, a)", &["q(5)"]);
    assert_eq!(compressed_residual(&system, clause, &honest), Fp::ZERO);
    for cheat in [
        assigned(&system, clause, "p(5, b)", &["q(5)"]),
        assigned(&system, clause, "p(5, a)", &["q(7)"]),
    ] {
        assert_ne!(compressed_residual(&system, clause, &cheat), Fp::ZERO);
    }
}

#[test]
fn compression_cannot_cancel_opposite_residuals() {
    // Two equalities whose residuals are +d and −d cancel in a plain sum, but
    // not in a randomly weighted one.
    let system = system("p(X, X, Y, Y).\n");
    let clause = &system.branches()[0];
    let cheat = assigned(&system, clause, "p(1, 2, 2, 1)", &[]);
    let rows: Vec<Fp> = clause.rows.iter().map(|row| row.evaluate(&cheat)).collect();
    let shared: Vec<Fp> = clause
        .rows
        .iter()
        .zip(&rows)
        .filter(|(row, _)| row.kind == RowKind::SharedVariable)
        .map(|(_, residual)| *residual)
        .collect();
    assert_eq!(shared.len(), 2);
    assert_eq!(shared[0] + shared[1], Fp::ZERO);
    assert_ne!(compressed_residual(&system, clause, &cheat), Fp::ZERO);
}

#[test]
fn quoted_atom_and_integer_are_different_constants() {
    let system = system("p('1').\n");
    assert!(satisfied(&system, &assignment(&system, "p('1')", &[])).len() == 1);
    assert!(satisfied(&system, &assignment(&system, "p(1)", &[])).is_empty());
}

#[test]
fn body_goals_are_checked_in_order() {
    let system = system("p :- q, r.\nq.\nr.\n");
    assert_eq!(
        satisfied(&system, &assignment(&system, "p", &["q", "r"])),
        ["clause p/0 #0 alternative 0"]
    );
    assert!(satisfied(&system, &assignment(&system, "p", &["r", "q"])).is_empty());
    assert!(satisfied(&system, &assignment(&system, "p", &["q"])).is_empty());
}

#[test]
fn disjunctions_become_alternatives() {
    let system = system("p(X) :- (q(X) ; r(X), s), t.\n");
    let alternatives: Vec<usize> = system
        .clause_branches("p", 1)
        .map(|branch| match branch.kind {
            BranchKind::Clause { children, .. } => children,
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(alternatives, [2, 3]);
    assert_eq!(
        satisfied(&system, &assignment(&system, "p(1)", &["r(1)", "s", "t"])),
        ["clause p/1 #0 alternative 1"]
    );
}

#[test]
fn lists_and_nested_patterns_are_opened_by_shape() {
    let system = system("p([(_, T, V)|Rest], T, V, Rest).\n");
    let clause = system.clause_branches("p", 4).next().unwrap();
    let shapes: Vec<&str> = clause
        .rows
        .iter()
        .filter(|row| row.kind == RowKind::Shape)
        .map(|row| row.description.as_str())
        .filter(|description| description.starts_with("payload"))
        .collect();
    assert_eq!(
        shapes,
        [
            "payload(P.0) = 0",
            "payload(P.0.0) = ,",
            "payload(P.0.0.1) = ,"
        ]
    );

    let honest = assigned(
        &system,
        clause,
        "p([(c, nominal, 10), x], nominal, 10, [x])",
        &[],
    );
    assert!(clause.is_satisfied_by(&honest));
    let cheat = assigned(
        &system,
        clause,
        "p([(c, nominal, 10), x], percent, 10, [x])",
        &[],
    );
    assert!(!clause.is_satisfied_by(&cheat));
}

#[test]
fn builtin_branches_fix_the_form_of_the_call() {
    let system = system("p(X, Y, Z) :- Y is X * 2 + 1, Y > 3, Z = Y.\n");
    let forms: Vec<String> = system
        .builtin_checks()
        .map(|check| check.to_string())
        .collect();
    assert_eq!(forms, ["_ is ((_ * _) + _)", "_ > _", "_ = _"]);

    for (node, expected) in [
        ("7 is 3 * 2 + 1", "builtin _ is ((_ * _) + _)"),
        ("7 > 3", "builtin _ > _"),
        ("seven = seven", "builtin _ = _"),
    ] {
        assert_eq!(
            satisfied(&system, &assignment(&system, node, &[])),
            [expected]
        );
    }
    for wrong_form in [
        "7 is 3 + 2 * 1",
        "7 is (3 * 2) * 1",
        "a > 3",
        "seven = eight",
    ] {
        assert!(
            satisfied(&system, &assignment(&system, wrong_form, &[])).is_empty(),
            "{wrong_form}"
        );
    }
}

#[test]
fn unsupported_policies_are_rejected_with_a_reason() {
    for (source, reason) in [
        ("p :- !.\n", "!/0 is not supported"),
        ("p(X) :- X \\= a.\n", "\\=/2 is not supported"),
        ("p(X) :- \\+ q(X).\n", "\\+/1 is not supported"),
        ("p(X) :- (q(X) -> r ; s).\n", "->/2 is not supported"),
        (
            "p(X, Y) :- Y is X mod 2.\n",
            "is not a supported arithmetic expression",
        ),
        ("p(X) :- f(X) is 1.\n", "left side of is/2"),
        ("p(X) :- X.\n", "a variable cannot be called"),
        ("p(a, b, c, d, e).\n", "exceeds the maximum arity"),
        ("p(f(a, b, c, d, e)).\n", "exceeds the maximum arity"),
        ("length(a, b).\n", "redefines the system predicate length/2"),
    ] {
        let error = compile_error(source);
        assert!(error.contains(reason), "{source:?}: {error}");
    }
}

#[test]
fn every_opened_node_has_its_child_digests_in_the_layout() {
    let system = system("p([A|T], f(A, g(B)), B) :- q(T), A > 1.\n");
    let layout = system.layout();
    let width = layout.width();
    for node in layout.opened_nodes() {
        for column in [node.digest, node.tag, node.payload, node.arity]
            .into_iter()
            .chain(node.children)
        {
            assert!(column < width);
        }
    }
    // Honest assignments recompute every opened digest from its cells.
    let honest = assignment(&system, "p([2, 3], f(2, g(b)), b)", &["q([3])", "2 > 1"]);
    for node in layout.opened_nodes() {
        let digest = honest[node.digest];
        if honest[node.tag] == Fp::ZERO {
            assert_eq!(digest, Fp::ZERO, "{}", node.position);
        } else {
            assert_ne!(digest, Fp::ZERO, "{}", node.position);
        }
    }
}
