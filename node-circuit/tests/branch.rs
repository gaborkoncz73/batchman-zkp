//! The branch chip on the repository's policy and proof tree: honest nodes
//! pass, and a node that satisfies no branch, or claims a vector that is no
//! branch, fails.

use ff::Field;
use halo2_proofs::dev::MockProver;
use node_circuit::{BranchCircuit, BranchTable, Claim, NodeBranch, minimum_k};
use policy_constraints::{Cell, ConstraintSystem, Fp, compile};
use policy_parser::parse_policy;
use proof_witness::{FactRegistry, ProofTree, ProofWitness, TrustedRoots, build};
use rand_core::OsRng;

const POLICY: &str = include_str!("../../prolog/policy.pl");
const TREE: &str = include_str!("../../input/proof_tree.json");
const FACTS: &str = include_str!("../../registry/facts.yaml");
const ROOTS: &str = include_str!("../../input/trusted_roots.json");

const DOT: &str = "node: ⟨cv, (w, 1)⟩ = 0";
const CLAIM: &str = "node: the compressed claim is a branch";

fn energy() -> (ConstraintSystem, ProofWitness) {
    let system = compile(&parse_policy(POLICY, "policy.pl").unwrap()).unwrap();
    let tree = ProofTree::from_json(TREE, "proof_tree.json").unwrap();
    let facts = FactRegistry::from_yaml(FACTS, "facts.yaml").unwrap();
    let roots = TrustedRoots::from_json(ROOTS, "trusted_roots.json").unwrap();
    let witness = build(&system, &tree, &facts, &roots).unwrap();
    (system, witness)
}

fn honest(witness: &ProofWitness) -> Vec<NodeBranch<Fp>> {
    witness
        .nodes
        .iter()
        .map(|node| NodeBranch {
            assignment: node.assignment.clone(),
            claim: Claim::Branch(node.branch),
        })
        .collect()
}

/// The failures of the circuit over these nodes, as MockProver prints them.
fn failures(table: BranchTable<Fp>, nodes: Vec<NodeBranch<Fp>>) -> Vec<String> {
    let circuit = BranchCircuit::new(table, nodes);
    match MockProver::run(minimum_k(&circuit), &circuit, vec![])
        .unwrap()
        .verify()
    {
        Ok(()) => Vec::new(),
        Err(failures) => failures.iter().map(ToString::to_string).collect(),
    }
}

/// Every failure names `expected` and the region of node `node` (regions 0
/// and 1 are the powers and the branch table).
fn only(failures: &[String], expected: &str, node: usize) {
    let region = format!("Region {} ('node branch')", node + 2);
    assert!(!failures.is_empty(), "the circuit accepts");
    assert!(
        failures
            .iter()
            .all(|failure| failure.contains(expected) && failure.contains(&region)),
        "{failures:#?}"
    );
}

#[test]
fn the_table_compresses_like_policy_constraints() {
    let (system, _) = energy();
    let table = BranchTable::from_policy(&system);
    let (c, gamma) = (Fp::random(OsRng), Fp::random(OsRng));
    for branch in system.branches() {
        let weights: Vec<Fp> = std::iter::successors(Some(Fp::ONE), |power| Some(*power * c))
            .take(branch.rows.len())
            .collect();
        let expected = branch.compress(system.layout().width(), &weights);
        let mut dense = expected.coefficients.clone();
        dense.push(expected.constant);
        assert_eq!(table.compress(branch.id, c), dense, "branch {}", branch.id);

        let folded = dense
            .iter()
            .rev()
            .fold(Fp::ZERO, |sum, value| sum * gamma + value);
        assert_eq!(
            table.fold(branch.id, c, gamma),
            folded,
            "branch {}",
            branch.id
        );
    }
}

#[test]
fn the_whole_tree_is_accepted() {
    let (system, witness) = energy();
    let table = BranchTable::from_policy(&system);
    assert_eq!(failures(table, honest(&witness)), Vec::<String>::new());
}

#[test]
fn a_tampered_assignment_is_rejected() {
    // Node 2 (a sumOfMonthlyConsumptions node) claims one more child.
    let (system, witness) = energy();
    let mut nodes = honest(&witness);
    let child_count = system.layout().column(&Cell::ChildCount).unwrap();
    nodes[2].assignment[child_count] += Fp::ONE;
    only(&failures(BranchTable::from_policy(&system), nodes), DOT, 2);
}

#[test]
fn claiming_another_branch_is_rejected() {
    // Node 0 (endPrice) claims the base case of sumOfMonthlyConsumptions.
    let (system, witness) = energy();
    let mut nodes = honest(&witness);
    nodes[0].claim = Claim::Branch(17);
    only(&failures(BranchTable::from_policy(&system), nodes), DOT, 0);
}

#[test]
fn the_zero_vector_is_not_a_branch() {
    // cv = 0 makes the dot product zero for every w; its fold is 0, which is
    // no Tᵢ (the rows outside the branch ends do not count: their flag is 0).
    let (system, witness) = energy();
    let width = system.layout().width();
    let mut nodes = honest(&witness);
    nodes[0].claim = Claim::Forged(vec![Fp::ZERO; width + 1]);
    only(
        &failures(BranchTable::from_policy(&system), nodes),
        CLAIM,
        0,
    );
}

#[test]
fn a_vector_outside_the_table_is_rejected() {
    // cv = e₀ − w[0]·e_W: ⟨cv, (w, 1)⟩ = w[0] − w[0] = 0, but cv is no branch.
    let (system, witness) = energy();
    let width = system.layout().width();
    let mut nodes = honest(&witness);
    let mut forged = vec![Fp::ZERO; width + 1];
    forged[0] = Fp::ONE;
    forged[width] = -nodes[5].assignment[0];
    nodes[5].claim = Claim::Forged(forged);
    only(
        &failures(BranchTable::from_policy(&system), nodes),
        CLAIM,
        5,
    );
}
