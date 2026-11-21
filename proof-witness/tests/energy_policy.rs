//! The witness of the repository's proof tree (`input/proof_tree.json`).

use policy_constraints::{BranchKind, Cell, ConstraintSystem, Position, Slot, compile};
use policy_parser::parse_policy;
use proof_witness::{FactRegistry, ProofTree, ProofWitness, TrustedRoots, build};

const POLICY: &str = include_str!("../../prolog/policy.pl");
const TREE: &str = include_str!("../../input/proof_tree.json");
const FACTS: &str = include_str!("../../issue/src/facts.yaml");
const ROOTS: &str = include_str!("../../input/trusted_roots.json");

fn system() -> ConstraintSystem {
    compile(&parse_policy(POLICY, "policy.pl").unwrap()).unwrap()
}

fn witness_with(tree: &str, facts: &str, roots: &str) -> anyhow::Result<ProofWitness> {
    build(
        &system(),
        &ProofTree::from_json(tree, "proof_tree")?,
        &FactRegistry::from_yaml(facts, "facts")?,
        &TrustedRoots::from_json(roots, "trusted_roots")?,
    )
}

fn error_with(tree: &str, facts: &str, roots: &str) -> String {
    format!("{:#}", witness_with(tree, facts, roots).unwrap_err())
}

#[test]
fn every_node_gets_an_assignment_and_a_satisfied_branch() {
    let system = system();
    let witness = witness_with(TREE, FACTS, ROOTS).unwrap();
    assert_eq!(witness.nodes.len(), 50);
    assert_eq!(witness.roots, [0]);
    assert_eq!(witness.nodes[0].text, "endPrice(931220)");

    for node in &witness.nodes {
        assert_eq!(node.assignment.len(), system.layout().width());
        let branch = &system.branches()[node.branch];
        assert!(branch.is_satisfied_by(&node.assignment), "{}", node.text);
        match &branch.kind {
            BranchKind::Builtin(check) => assert!(check.holds(&node.goal).unwrap()),
            BranchKind::ExternalFact { .. } => {}
            BranchKind::Clause { children, .. } => assert_eq!(*children, node.children.len()),
        }
    }
}

#[test]
fn nodes_are_in_pre_order_and_subtrees_are_contiguous() {
    let witness = witness_with(TREE, FACTS, ROOTS).unwrap();
    for (index, node) in witness.nodes.iter().enumerate() {
        for &child in &node.children {
            assert!(child > index);
            assert_eq!(witness.nodes[child].parent, Some(index));
        }
        // The subtree of `index` is `index..end`.
        let mut end = index + 1;
        while end < witness.nodes.len()
            && std::iter::successors(witness.nodes[end].parent, |&p| witness.nodes[p].parent)
                .any(|ancestor| ancestor == index)
        {
            end += 1;
        }
        let outside = witness.nodes[end..].iter().any(|later| {
            std::iter::successors(later.parent, |&p| witness.nodes[p].parent).any(|a| a == index)
        });
        assert!(!outside, "subtree of node {index} is not contiguous");
    }
}

/// The digest of child slot k in the parent's assignment is the digest of the
/// goal in the child's own assignment. This is the link between nodes.
#[test]
fn parent_and_child_assignments_agree_on_the_child_digest() {
    let system = system();
    let layout = system.layout();
    let goal_digest = layout
        .column(&Cell::Digest(Position::root(Slot::Parent)))
        .unwrap();
    let witness = witness_with(TREE, FACTS, ROOTS).unwrap();
    for node in &witness.nodes {
        for (slot, &child) in node.children.iter().enumerate() {
            let column = layout
                .column(&Cell::Digest(Position::root(Slot::Child(slot as u8))))
                .unwrap();
            assert_eq!(
                node.assignment[column],
                witness.nodes[child].assignment[goal_digest]
            );
        }
    }
}

#[test]
fn an_untrusted_root_is_rejected() {
    let roots = r#"{ "root_goals": ["endPrice(931221)"] }"#;
    let error = error_with(TREE, FACTS, roots);
    assert!(
        error.contains("endPrice(931220), which is not a trusted root"),
        "{error}"
    );
}

#[test]
fn a_fact_missing_from_the_registry_is_rejected() {
    let facts = FACTS.replace(
        "support_matrix(low,mid,percent,10)",
        "support_matrix(low,mid,percent,11)",
    );
    let error = error_with(TREE, &facts, ROOTS);
    assert!(
        error.contains("node 41 support_matrix(low,mid,percent,10): external fact support_matrix/4: not in the fact registry"),
        "{error}"
    );
}

#[test]
fn a_changed_result_breaks_the_parent_first() {
    // The subtraction claims 941221 while the parent still says 941220. Node
    // 42 (the parent) comes first in pre-order, and its shared variable row
    // no longer holds.
    let tree = TREE.replace("941220 is 1045800-104580", "941221 is 1045800-104580");
    let error = error_with(&tree, FACTS, ROOTS);
    assert!(
        error.contains("node 42 applySupport(1045800,percent,10,941220): no branch matches"),
        "{error}"
    );
}
