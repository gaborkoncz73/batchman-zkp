//! The reference check: honest witnesses pass, and every kind of tampering
//! with the assignments is caught by the rule that is meant to catch it.

use ff::Field;
use halo2_proofs::pasta::Fp;
use policy_constraints::term::symbol;
use policy_constraints::{BranchKind, Cell, ConstraintSystem, GroundTerm, Position, Slot, compile};
use policy_parser::{parse_goal, parse_policy};
use proof_witness::{
    FactRegistry, NodeWitness, ProofTree, ProofWitness, TrustedRoots, build, check,
};

const POLICY: &str = include_str!("../../prolog/policy.pl");
const TREE: &str = include_str!("../../input/proof_tree.json");
const FACTS: &str = include_str!("../../issue/src/facts.yaml");
const ROOTS: &str = include_str!("../../input/trusted_roots.json");

struct Energy {
    system: ConstraintSystem,
    witness: ProofWitness,
    roots: TrustedRoots,
}

fn energy() -> Energy {
    let system = compile(&parse_policy(POLICY, "policy.pl").unwrap()).unwrap();
    let roots = TrustedRoots::from_json(ROOTS, "trusted_roots").unwrap();
    let witness = build(
        &system,
        &ProofTree::from_json(TREE, "proof_tree").unwrap(),
        &FactRegistry::from_yaml(FACTS, "facts").unwrap(),
        &roots,
    )
    .unwrap();
    Energy {
        system,
        witness,
        roots,
    }
}

/// A fixed stand-in for the verifier's challenge, so that the tests are
/// deterministic.
fn challenge() -> Fp {
    symbol("test challenge")
}

fn ground(goal: &str) -> GroundTerm {
    GroundTerm::from_term(&parse_goal(goal, "goal").unwrap()).unwrap()
}

impl Energy {
    fn check(&self) -> anyhow::Result<()> {
        check(&self.system, &self.witness, &self.roots, challenge())
    }

    fn error(&self) -> String {
        format!("{:#}", self.check().unwrap_err())
    }

    fn column(&self, cell: Cell) -> usize {
        self.system.layout().column(&cell).unwrap()
    }

    /// The node whose goal is written as `text`.
    fn node(&self, text: &str) -> usize {
        self.witness
            .nodes
            .iter()
            .position(|node| node.text == text)
            .unwrap_or_else(|| panic!("no node {text}"))
    }

    /// Replaces the assignment of `node` with an honest assignment of other
    /// goals, keeping its branch and its place in the tree.
    fn reassign(&mut self, node: usize, parent: &str, children: &[&str]) {
        let children: Vec<GroundTerm> = children.iter().map(|goal| ground(goal)).collect();
        self.witness.nodes[node].assignment = self
            .system
            .layout()
            .assign(&ground(parent), &children)
            .unwrap();
    }
}

#[test]
fn the_honest_witness_passes() {
    let energy = energy();
    energy.check().unwrap();
    // Any challenge works for an honest witness, even a degenerate one.
    for challenge in [Fp::ZERO, Fp::ONE, -Fp::ONE] {
        check(&energy.system, &energy.witness, &energy.roots, challenge).unwrap();
    }
}

/// For every node and every branch, the compressed branch is zero exactly
/// when all rows hold: one random combination decides like all the rows.
#[test]
fn compression_decides_like_the_rows() {
    let energy = energy();
    let width = energy.system.layout().width();
    let mut holding = 0;
    for branch in energy.system.branches() {
        let weights: Vec<Fp> =
            std::iter::successors(Some(Fp::ONE), |power| Some(*power * challenge()))
                .take(branch.rows.len())
                .collect();
        let compressed = branch.compress(width, &weights);
        for node in &energy.witness.nodes {
            let rows_hold = branch.is_satisfied_by(&node.assignment);
            let compressed_holds = compressed.evaluate(&node.assignment) == Fp::ZERO;
            assert_eq!(
                rows_hold, compressed_holds,
                "{} on {}",
                branch.kind, node.text
            );
            holding += usize::from(rows_hold);
        }
    }
    // Every node satisfies at least its own branch, and most pairs fail.
    assert!(holding >= energy.witness.nodes.len());
    assert!(holding < energy.witness.nodes.len() * energy.system.branches().len() / 2);
}

#[test]
fn a_cell_that_disagrees_with_its_digest_is_rejected() {
    let mut energy = energy();
    // The integer 931220 in endPrice(931220) becomes 1, digest left alone.
    let payload = energy.column(Cell::Payload(Position::root(Slot::Parent).child(0)));
    energy.witness.nodes[0].assignment[payload] = Fp::ONE;
    let error = energy.error();
    assert!(
        error.contains("node 0: subterm P.0: the digest is not the hash"),
        "{error}"
    );
}

#[test]
fn swapping_a_fact_breaks_the_link_to_the_parent() {
    // The leaf claims another, unissued support_matrix entry. Its own
    // assignment is honest and satisfies the fact branch, but the parent
    // still asks for the issued one.
    let mut energy = energy();
    let leaf = energy.node("support_matrix(low,mid,percent,10)");
    energy.reassign(leaf, "support_matrix(low,mid,percent,11)", &[]);
    let parent = energy.witness.nodes[leaf].parent.unwrap();
    let error = energy.error();
    assert!(
        error.contains(&format!(
            "node {parent}: child slot 0 does not hold the goal of node {leaf}"
        )),
        "{error}"
    );
}

#[test]
fn a_wrong_branch_choice_is_rejected_with_its_violated_rows() {
    let mut energy = energy();
    let fact = energy
        .system
        .branches()
        .iter()
        .find(|branch| matches!(branch.kind, BranchKind::ExternalFact { .. }))
        .unwrap()
        .id;
    energy.witness.nodes[0].branch = fact;
    let error = energy.error();
    assert!(error.contains("node 0: external fact"), "{error}");
    assert!(
        error.contains("does not hold, violating payload(P) ="),
        "{error}"
    );
}

#[test]
fn the_child_count_cell_must_match_the_tree() {
    let mut energy = energy();
    let count = energy.column(Cell::ChildCount);
    energy.witness.nodes[0].assignment[count] += Fp::ONE;
    assert!(
        energy
            .error()
            .contains("node 0: the child count cell does not match"),
        "{}",
        energy.error()
    );
}

#[test]
fn the_root_must_prove_the_trusted_goal() {
    // An honest-looking root for another end price.
    let mut energy = energy();
    let root = &energy.witness.nodes[0];
    let children: Vec<String> = root
        .children
        .iter()
        .map(|&child| energy.witness.nodes[child].text.clone())
        .collect();
    let children: Vec<&str> = children.iter().map(String::as_str).collect();
    energy.reassign(0, "endPrice(931221)", &children);
    let error = energy.error();
    // The changed price no longer matches the price the last body goal
    // computed.
    assert!(
        error.contains("node 0: clause endPrice/1 #0 alternative 0 does not hold")
            && error.contains("(variable Price)"),
        "{error}"
    );

    let energy = Energy {
        roots: TrustedRoots::new(&["endPrice(931221)"], "roots").unwrap(),
        ..self::energy()
    };
    assert!(
        energy
            .error()
            .contains("root node 0 proves a goal that is not trusted"),
        "{}",
        energy.error()
    );
}

#[test]
fn the_tree_shape_must_be_consistent() {
    let mut energy = energy();
    let child = energy.witness.nodes[0].children[0];
    energy.witness.nodes[child].parent = None;
    assert!(
        energy
            .error()
            .contains(&format!("node 0: child {child} names another parent")),
        "{}",
        energy.error()
    );

    let mut energy = self::energy();
    energy.witness.nodes[0].branch = usize::MAX;
    assert!(energy.error().contains("does not exist"));
}

#[test]
fn a_root_proved_by_a_fact_is_rejected() {
    let system = compile(&parse_policy("p :- q(1).\n", "test.pl").unwrap()).unwrap();
    let fact = system
        .branches()
        .iter()
        .find(|branch| matches!(branch.kind, BranchKind::ExternalFact { .. }))
        .unwrap()
        .id;
    let goal = ground("q(1)");
    let witness = ProofWitness {
        nodes: vec![NodeWitness {
            text: "q(1)".to_owned(),
            assignment: system.layout().assign(&goal, &[]).unwrap(),
            goal,
            parent: None,
            children: Vec::new(),
            branch: fact,
        }],
        roots: vec![0],
    };
    let roots = TrustedRoots::new(&["q(1)"], "roots").unwrap();
    let error = format!(
        "{:#}",
        check(&system, &witness, &roots, challenge()).unwrap_err()
    );
    assert!(
        error.contains("root node 0 is proved by external fact q/1"),
        "{error}"
    );
}
