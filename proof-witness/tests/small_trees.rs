//! Small hand-written proof trees: honest ones and cheats.

use policy_constraints::{ConstraintSystem, compile};
use policy_parser::parse_policy;
use proof_witness::{FactRegistry, ProofTree, ProofWitness, TrustedRoots, build};

const POLICY: &str = "\
p(X, Y) :- q(X), Y is X * 2 + 1, Y > 3.
d(X, Y) :- Y is X div 0.
";

fn system() -> ConstraintSystem {
    compile(&parse_policy(POLICY, "test.pl").unwrap()).unwrap()
}

/// A tree with one root `goal` whose children are leaves.
fn tree(goal: &str, leaves: &[&str]) -> String {
    let leaves: Vec<String> = leaves
        .iter()
        .map(|leaf| format!(r#"{{"goal": "{leaf}", "subtree": [true]}}"#))
        .collect();
    format!(
        r#"[{{"goal": "{goal}", "subtree": [{}]}}]"#,
        leaves.join(", ")
    )
}

fn facts(goals: &[&str]) -> String {
    goals
        .iter()
        .enumerate()
        .map(|(index, goal)| format!("- goal: \"{goal}\"\n  salt: \"{}\"\n", index + 1))
        .collect()
}

fn run(tree: &str, facts: &str, roots: &[&str]) -> anyhow::Result<ProofWitness> {
    build(
        &system(),
        &ProofTree::from_json(tree, "tree")?,
        &FactRegistry::from_yaml(facts, "facts")?,
        &TrustedRoots::new(roots, "roots")?,
    )
}

fn error(tree: &str, facts: &str, roots: &[&str]) -> String {
    format!("{:#}", run(tree, facts, roots).unwrap_err())
}

fn honest_tree() -> String {
    tree("p(5,11)", &["q(5)", "11 is 5*2+1", "11>3"])
}

#[test]
fn honest_tree_gets_one_branch_per_node() {
    let system = system();
    let witness = run(&honest_tree(), &facts(&["q(5)"]), &["p(5,11)"]).unwrap();
    let branches: Vec<String> = witness
        .nodes
        .iter()
        .map(|node| system.branches()[node.branch].kind.to_string())
        .collect();
    assert_eq!(
        branches,
        [
            "clause p/2 #0 alternative 0",
            "external fact q/1",
            "builtin _ is ((_ * _) + _)",
            "builtin _ > _",
        ]
    );
    assert_eq!(witness.nodes[0].children, [1, 2, 3]);
}

#[test]
fn wrong_arithmetic_is_rejected_at_the_builtin() {
    let tree = tree("p(5,12)", &["q(5)", "12 is 5*2+1", "12>3"]);
    let error = error(&tree, &facts(&["q(5)"]), &["p(5,12)"]);
    assert!(
        error.contains("node 2 12 is 5*2+1: builtin _ is ((_ * _) + _): the builtin is false"),
        "{error}"
    );
}

#[test]
fn false_comparison_is_rejected() {
    let tree = tree("p(1,3)", &["q(1)", "3 is 1*2+1", "3>3"]);
    let error = error(&tree, &facts(&["q(1)"]), &["p(1,3)"]);
    assert!(
        error.contains("node 3 3>3: builtin _ > _: the builtin is false"),
        "{error}"
    );
}

#[test]
fn division_by_zero_is_rejected() {
    let tree = tree("d(4,0)", &["0 is 4 div 0"]);
    let error = error(&tree, "[]", &["d(4,0)"]);
    assert!(error.contains("division by zero"), "{error}");
}

#[test]
fn a_fact_that_was_not_issued_is_rejected() {
    let error = error(&honest_tree(), &facts(&["q(6)"]), &["p(5,11)"]);
    assert!(
        error.contains("node 1 q(5): external fact q/1: not in the fact registry"),
        "{error}"
    );
}

#[test]
fn a_missing_child_matches_no_branch() {
    let tree = tree("p(5,11)", &["q(5)", "11 is 5*2+1"]);
    let error = error(&tree, &facts(&["q(5)"]), &["p(5,11)"]);
    assert!(
        error.contains("node 0 p(5,11): no branch matches"),
        "{error}"
    );
    assert!(
        error.contains("closest is clause p/2 #0 alternative 0, violating children = 3"),
        "{error}"
    );
}

#[test]
fn more_children_than_the_layout_has_slots_are_rejected() {
    let tree = tree("p(5,11)", &["q(5)", "11 is 5*2+1", "11>3", "q(5)"]);
    let error = error(&tree, &facts(&["q(5)"]), &["p(5,11)"]);
    assert!(
        error.contains("the node has 4 body goals, the layout has 3 slots"),
        "{error}"
    );
}

#[test]
fn roots_must_be_exactly_the_trusted_goals() {
    let facts = facts(&["q(5)"]);
    let error1 = error(&honest_tree(), &facts, &["p(5,12)"]);
    assert!(
        error1.contains("p(5,11), which is not a trusted root"),
        "{error1}"
    );

    let error2 = error(&honest_tree(), &facts, &["p(5,11)", "p(6,13)"]);
    assert!(error2.contains("trusted root 1 is not proved"), "{error2}");

    let two_roots = format!(
        "[{}, {}]",
        honest_tree().trim_matches(['[', ']']),
        honest_tree().trim_matches(['[', ']'])
    );
    let error3 = error(&two_roots, &facts, &["p(5,11)"]);
    assert!(error3.contains("proves p(5,11) twice"), "{error3}");
}

#[test]
fn a_root_must_be_proved_by_a_clause() {
    let tree = tree("q(5)", &[]);
    let error = error(&tree, &facts(&["q(5)"]), &["q(5)"]);
    assert!(
        error.contains("q(5) is proved by external fact q/1, not by a clause"),
        "{error}"
    );
}

#[test]
fn malformed_inputs_are_rejected() {
    let parse_facts =
        |yaml: &str| format!("{:#}", FactRegistry::from_yaml(yaml, "facts").unwrap_err());
    assert!(parse_facts(&facts(&["q(5)", "q(5)"])).contains("is already fact 0"));
    assert!(parse_facts("- goal: \"q(X)\"\n  salt: \"1\"\n").contains("not ground"));
    assert!(parse_facts("- goal: \"5\"\n  salt: \"1\"\n").contains("not a callable goal"));
    assert!(parse_facts("- goal: \"q(5)\"\n  salt: \"01\"\n").contains("canonical decimal"));
    assert!(parse_facts("- goal: \"q(5)\"\n  salt: \"-1\"\n").contains("unsigned decimal"));
    let modulus_plus_one =
        "28948022309329048855892746252171976963363056481941560715954676764349967630338";
    assert!(
        parse_facts(&format!(
            "- goal: \"q(5)\"\n  salt: \"{modulus_plus_one}\"\n"
        ))
        .contains("field modulus")
    );

    let parse_tree = |json: &str| format!("{:#}", ProofTree::from_json(json, "tree").unwrap_err());
    assert!(parse_tree("[]").contains("no root"));
    assert!(parse_tree(r#"[{"goal": "p(X)", "subtree": [true]}]"#).contains("not ground"));
    assert!(parse_tree(r#"[{"goal": "p(1)", "subtree": [false]}]"#).contains("`false`"));

    assert!(TrustedRoots::new(&["p(1)", "p(1)"], "roots").is_err());
    assert!(TrustedRoots::new(&[] as &[&str], "roots").is_err());
}
