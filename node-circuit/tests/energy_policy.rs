//! The well-formedness chip on nodes of the repository's proof tree.

use ff::Field;
use halo2_poseidon::poseidon::primitives::P128Pow5T3;
use halo2_proofs::dev::MockProver;
use node_circuit::{WellFormedCircuit, minimum_k};
use policy_constraints::{Cell, ConstraintSystem, Fp, Position, Slot, compile};
use policy_parser::parse_policy;
use proof_witness::{FactRegistry, ProofTree, ProofWitness, TrustedRoots, build};

const POLICY: &str = include_str!("../../prolog/policy.pl");
const TREE: &str = include_str!("../../input/proof_tree.json");
const FACTS: &str = include_str!("../../registry/facts.yaml");
const ROOTS: &str = include_str!("../../input/trusted_roots.json");

fn energy() -> (ConstraintSystem, ProofWitness) {
    let system = compile(&parse_policy(POLICY, "policy.pl").unwrap()).unwrap();
    let tree = ProofTree::from_json(TREE, "proof_tree.json").unwrap();
    let facts = FactRegistry::from_yaml(FACTS, "facts.yaml").unwrap();
    let roots = TrustedRoots::from_json(ROOTS, "trusted_roots.json").unwrap();
    let witness = build(&system, &tree, &facts, &roots).unwrap();
    (system, witness)
}

fn verify(system: &ConstraintSystem, nodes: Vec<Vec<Fp>>) -> Result<(), Vec<String>> {
    let circuit = WellFormedCircuit::<Fp, P128Pow5T3>::new(system.layout().clone(), nodes);
    MockProver::run(minimum_k(&circuit), &circuit, vec![])
        .unwrap()
        .verify()
        .map_err(|failures| failures.iter().map(ToString::to_string).collect())
}

#[test]
fn the_first_nodes_of_the_tree_are_well_formed() {
    // endPrice, monthlyConsumptions and a sumOfMonthlyConsumptions node.
    let (system, witness) = energy();
    let nodes = witness.nodes[..3]
        .iter()
        .map(|node| node.assignment.clone())
        .collect();
    assert_eq!(verify(&system, nodes), Ok(()));
}

#[test]
fn a_cell_that_disagrees_with_its_digest_is_rejected() {
    // As in proof-witness' reference check: the integer 931220 in
    // endPrice(931220) becomes 1, its digest is left alone.
    let (system, witness) = energy();
    let mut w = witness.nodes[0].assignment.clone();
    let payload = system
        .layout()
        .column(&Cell::Payload(Position::root(Slot::Parent).child(0)))
        .unwrap();
    w[payload] = Fp::ONE;
    assert!(system.layout().check_well_formed(&w).is_err());

    let failures = verify(&system, vec![w]).unwrap_err();
    assert!(
        failures
            .iter()
            .all(|failure| failure.contains("parent-side digest is the hash")
                && failure.contains("'subterm P.0'")),
        "{failures:#?}"
    );
}
