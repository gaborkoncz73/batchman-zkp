//! The whole per-node circuit (3c) on the repository's proof tree, against
//! the per-node cheats of proof-witness' reference check.

use ff::Field;
use halo2_poseidon::poseidon::primitives::P128Pow5T3;
use halo2_proofs::dev::MockProver;
use node_circuit::{BranchTable, Claim, NodeBranch, NodeCircuit, minimum_k};
use policy_constraints::arith::StepCell;
use policy_constraints::term::fact_commitment;
use policy_constraints::{Cell, ConstraintSystem, Fp, GroundTerm, Position, Slot, compile};
use policy_parser::{parse_goal, parse_policy};
use proof_witness::{
    FactRegistry, ProofTree, ProofWitness, TrustedRoots, build, parse_commitments,
};
use rand_core::OsRng;

const POLICY: &str = include_str!("../../prolog/policy.pl");
const TREE: &str = include_str!("../../input/proof_tree.json");
const FACTS: &str = include_str!("../../registry/facts.yaml");
const ROOTS: &str = include_str!("../../input/trusted_roots.json");
const COMMITMENTS: &str = include_str!("../../registry/commitments.json");
const FACT_LOOKUP: &str = "a fact's commitment is published";

/// The first nodes: endPrice, monthlyConsumptions and the start of the
/// sumOfMonthlyConsumptions recursion (the whole tree runs in the example).
const NODES: usize = 4;

fn energy() -> (ConstraintSystem, ProofWitness) {
    let system = compile(&parse_policy(POLICY, "policy.pl").unwrap()).unwrap();
    let tree = ProofTree::from_json(TREE, "proof_tree.json").unwrap();
    let facts = FactRegistry::from_yaml(FACTS, "facts.yaml").unwrap();
    let roots = TrustedRoots::from_json(ROOTS, "trusted_roots.json").unwrap();
    let witness = build(&system, &tree, &facts, &roots).unwrap();
    (system, witness)
}

fn honest(witness: &ProofWitness, count: usize) -> Vec<NodeBranch<Fp>> {
    witness.nodes[..count]
        .iter()
        .map(|node| NodeBranch {
            assignment: node.assignment.clone(),
            claim: Claim::Branch(node.branch),
        })
        .collect()
}

/// The registry's public commitment list.
fn published() -> Vec<Fp> {
    parse_commitments(COMMITMENTS, "commitments.json").unwrap()
}

fn verify(circuit: NodeCircuit<Fp, P128Pow5T3>) -> Vec<String> {
    verify_with(circuit, published())
}

fn verify_with(circuit: NodeCircuit<Fp, P128Pow5T3>, commitments: Vec<Fp>) -> Vec<String> {
    match MockProver::run(minimum_k(&circuit), &circuit, vec![commitments])
        .unwrap()
        .verify()
    {
        Ok(()) => Vec::new(),
        Err(failures) => failures.iter().map(ToString::to_string).collect(),
    }
}

fn circuit(system: &ConstraintSystem, nodes: Vec<NodeBranch<Fp>>) -> NodeCircuit<Fp, P128Pow5T3> {
    NodeCircuit::new(
        system.layout().clone(),
        BranchTable::from_policy(system),
        nodes,
    )
}

fn column(system: &ConstraintSystem, cell: Cell) -> usize {
    system.layout().column(&cell).unwrap()
}

fn any(failures: &[String], expected: &str) {
    assert!(
        failures.iter().any(|failure| failure.contains(expected)),
        "{failures:#?}"
    );
}

#[test]
fn honest_nodes_are_accepted() {
    let (system, witness) = energy();
    assert_eq!(
        verify(circuit(&system, honest(&witness, NODES))),
        Vec::<String>::new()
    );
}

#[test]
fn a_cell_that_disagrees_with_its_digest_is_rejected() {
    // The integer 931220 in endPrice(931220) becomes 1, digest left alone.
    let (system, witness) = energy();
    let mut nodes = honest(&witness, NODES);
    let payload = column(
        &system,
        Cell::Payload(Position::root(Slot::Parent).child(0)),
    );
    nodes[0].assignment[payload] = Fp::ONE;
    any(
        &verify(circuit(&system, nodes)),
        "parent-side digest is the hash",
    );
}

#[test]
fn a_wrong_branch_choice_is_rejected() {
    let (system, witness) = energy();
    let mut nodes = honest(&witness, NODES);
    nodes[0].claim = Claim::Branch(17);
    any(&verify(circuit(&system, nodes)), "node: ⟨cv, (w, 1)⟩ = 0");
}

#[test]
fn a_child_count_the_branch_does_not_allow_is_rejected() {
    // The reference compares the cell with the tree (step 5); per node, the
    // branch's ChildCount row already rejects it.
    let (system, witness) = energy();
    let mut nodes = honest(&witness, NODES);
    nodes[0].assignment[column(&system, Cell::ChildCount)] += Fp::ONE;
    any(&verify(circuit(&system, nodes)), "node: ⟨cv, (w, 1)⟩ = 0");
}

#[test]
fn the_two_chips_must_see_the_same_assignment() {
    // Each chip on its own is satisfied: well-formedness sees node 1's honest
    // assignment, the branch check sees node 0's. Only the links object.
    let (system, witness) = energy();
    let branch = honest(&witness, 1);
    let well_formed = vec![witness.nodes[1].assignment.clone()];
    let failures = verify(NodeCircuit::with_assignments(
        system.layout().clone(),
        BranchTable::from_policy(&system),
        well_formed,
        branch,
    ));
    assert!(!failures.is_empty());
    assert!(
        failures
            .iter()
            .all(|failure| failure.contains("Equality constraint not satisfied")),
        "{failures:#?}"
    );
}

#[test]
fn a_false_sum_is_rejected_by_the_arithmetic_chip() {
    // `24006 is 22005+2001` becomes `24007 is 22005+2001`, and the prover
    // writes 24007 into the step's z as well, so every row of the
    // `_ is _ + _` branch holds. Only the step's `add` is false.
    let (system, witness) = energy();
    let index = witness
        .nodes
        .iter()
        .position(|node| node.text == "24006 is 22005+2001")
        .unwrap();
    let ground = |goal: &str| GroundTerm::from_term(&parse_goal(goal, "goal").unwrap()).unwrap();
    let branch = witness.nodes[index].branch;
    let mut false_goal = system
        .layout()
        .assign(&ground("24007 is 22005+2001"), &[])
        .unwrap();
    system.complete(branch, &mut false_goal, Fp::ZERO).unwrap();
    false_goal[system.layout().step_column(0, StepCell::Z)] = Fp::from(24007);
    assert!(
        system.layout().check_arithmetic(&false_goal).is_err(),
        "the reference objects"
    );
    let nodes = vec![NodeBranch {
        assignment: false_goal,
        claim: Claim::Branch(branch),
    }];
    let failures = verify(circuit(&system, nodes));
    assert!(!failures.is_empty());
    assert!(
        failures
            .iter()
            .all(|failure| failure.contains("'arithmetic step'") && failure.contains("('step 0')")),
        "{failures:#?}"
    );
}

/// Every failure is the fact lookup, in the fact region of a node.
fn only_the_fact_lookup(failures: &[String]) {
    assert!(!failures.is_empty(), "the circuit accepts");
    assert!(
        failures
            .iter()
            .all(|failure| failure.contains(FACT_LOOKUP) && failure.contains("('fact')")),
        "{failures:#?}"
    );
}

#[test]
fn a_fact_with_a_wrong_salt_is_rejected() {
    // Node 1, monthlyConsumptions(…), is issued, but under another salt.
    let (system, witness) = energy();
    let mut nodes = honest(&witness, NODES);
    let (_, salt) = system.layout().fact_cells().unwrap();
    nodes[1].assignment[salt] += Fp::ONE;
    only_the_fact_lookup(&verify(circuit(&system, nodes)));
}

#[test]
fn the_commitment_list_is_the_public_input() {
    // The same honest nodes against a list without monthlyConsumptions'
    // commitment: only the fact node fails.
    let (system, witness) = energy();
    let fact = &witness.nodes[1];
    let (_, salt) = system.layout().fact_cells().unwrap();
    let digest = column(&system, Cell::Digest(Position::root(Slot::Parent)));
    let commitment = fact_commitment(fact.assignment[digest], fact.assignment[salt]);
    let without: Vec<Fp> = published()
        .into_iter()
        .filter(|published| *published != commitment)
        .collect();
    assert_eq!(without.len(), published().len() - 1);
    only_the_fact_lookup(&verify_with(
        circuit(&system, honest(&witness, NODES)),
        without,
    ));
}

#[test]
fn the_salt_of_a_node_that_is_no_fact_does_not_matter() {
    // Node 0 (endPrice) is proved by a clause: is_fact = 0, the hash of its
    // digest and any salt is not looked up.
    let (system, witness) = energy();
    let mut nodes = honest(&witness, NODES);
    let (_, salt) = system.layout().fact_cells().unwrap();
    nodes[0].assignment[salt] = Fp::random(OsRng);
    assert_eq!(verify(circuit(&system, nodes)), Vec::<String>::new());
}

#[test]
fn a_fact_node_cannot_claim_it_is_no_fact() {
    // The branch's rows fix is_fact: setting it to 0 to skip the lookup breaks
    // the external fact branch.
    let (system, witness) = energy();
    let mut nodes = honest(&witness, NODES);
    let (is_fact, _) = system.layout().fact_cells().unwrap();
    nodes[1].assignment[is_fact] = Fp::ZERO;
    let failures = verify(circuit(&system, nodes));
    assert!(
        failures
            .iter()
            .all(|failure| failure.contains("node: ⟨cv, (w, 1)⟩ = 0")),
        "{failures:#?}"
    );
}

#[test]
fn an_unpublished_fact_is_rejected() {
    // q(2) has the form of the fact branch, but only q(1) is published.
    let system = compile(&parse_policy("p :- q(1).\n", "test.pl").unwrap()).unwrap();
    let fact = system
        .branches()
        .iter()
        .find(|branch| branch.kind.to_string() == "external fact q/1")
        .unwrap()
        .id;
    let ground = |goal: &str| GroundTerm::from_term(&parse_goal(goal, "goal").unwrap()).unwrap();
    let salt = Fp::random(OsRng);
    let mut w = system.layout().assign(&ground("q(2)"), &[]).unwrap();
    system.complete(fact, &mut w, salt).unwrap();
    let nodes = vec![NodeBranch {
        assignment: w,
        claim: Claim::Branch(fact),
    }];
    let list = vec![fact_commitment(ground("q(1)").digest(), salt)];
    only_the_fact_lookup(&verify_with(circuit(&system, nodes), list));
}
