//! The branch circuit on the repository's policy and proof tree, and on toy
//! tables that show why the compression needs a challenge.

use ff::Field;
use halo2_proofs::dev::{FailureLocation, MockProver, VerifyFailure};
use halo2_proofs::halo2curves::pasta::Fp;
use policy_constraints::Cell;
use rand_core::OsRng;
use spike_challenge::{BranchCircuit, BranchTable, Claim, NodeInput, minimum_k, repository_inputs};

fn mock(circuit: &BranchCircuit) -> Result<(), Vec<VerifyFailure>> {
    let k = minimum_k(circuit.rows());
    MockProver::run(k, circuit, vec![]).expect("synthesis").verify()
}

/// The names of the failed gates and lookups, for the assertions.
fn failures(result: Result<(), Vec<VerifyFailure>>) -> Vec<String> {
    result
        .expect_err("the circuit must reject")
        .into_iter()
        .map(|failure| match failure {
            VerifyFailure::ConstraintNotSatisfied { constraint, .. } => format!("gate {constraint}"),
            VerifyFailure::Lookup { name, location, .. } => {
                let row = match location {
                    FailureLocation::InRegion { offset, .. } => offset,
                    FailureLocation::OutsideRegion { row } => row,
                };
                format!("lookup {name} @ {row}")
            }
            other => format!("{other:?}"),
        })
        .collect()
}

fn honest_nodes() -> (policy_constraints::ConstraintSystem, Vec<NodeInput>) {
    let (policy, witness) = repository_inputs().unwrap();
    let nodes = witness
        .nodes
        .iter()
        .map(|node| NodeInput {
            assignment: node.assignment.clone(),
            claim: Claim::Branch(node.branch),
        })
        .collect();
    (policy, nodes)
}

#[test]
fn the_in_circuit_compression_matches_policy_constraints() {
    let (policy, _) = repository_inputs().unwrap();
    let table = BranchTable::from_policy(&policy);
    let c = Fp::random(OsRng);
    for branch in policy.branches() {
        let weights: Vec<Fp> =
            std::iter::successors(Some(Fp::ONE), |power| Some(*power * c)).take(branch.rows.len()).collect();
        let expected = branch.compress(policy.layout().width(), &weights);
        let mut expected_dense = expected.coefficients.clone();
        expected_dense.push(expected.constant);
        assert_eq!(table.compress(branch.id, c), expected_dense, "branch {}", branch.id);
    }
}

#[test]
fn the_repository_tree_is_accepted() {
    let (policy, witness) = repository_inputs().unwrap();
    let circuit = BranchCircuit::from_witness(&policy, &witness);
    assert_eq!(mock(&circuit), Ok(()));
}

#[test]
fn a_tampered_assignment_is_rejected() {
    let (policy, mut nodes) = honest_nodes();
    // Node 2 (a sumOfMonthlyConsumptions node) now claims one child more.
    let child_count = policy.layout().column(&Cell::ChildCount).unwrap();
    nodes[2].assignment[child_count] += Fp::ONE;
    let circuit = BranchCircuit::new(BranchTable::from_policy(&policy), nodes);
    let failed = failures(mock(&circuit));
    assert!(failed.iter().all(|failure| failure.contains("⟨cv, (w, 1)⟩ = 0")), "{failed:?}");
}

#[test]
fn claiming_another_branch_is_rejected() {
    let (policy, mut nodes) = honest_nodes();
    // Node 0 (endPrice) claims the base case of sumOfMonthlyConsumptions.
    nodes[0].claim = Claim::Branch(17);
    let circuit = BranchCircuit::new(BranchTable::from_policy(&policy), nodes);
    let failed = failures(mock(&circuit));
    assert!(failed.iter().all(|failure| failure.contains("⟨cv, (w, 1)⟩ = 0")), "{failed:?}");
}

#[test]
fn the_zero_vector_is_not_in_the_table() {
    // cv = 0 makes ⟨cv, (w, 1)⟩ = 0 for every w. The non-first table rows hold
    // all-zero x tuples, so without the leading q_table entry this would pass.
    let (policy, mut nodes) = honest_nodes();
    let width = policy.layout().width();
    nodes[0].claim = Claim::Forged(vec![Fp::ZERO; width + 1]);
    let circuit = BranchCircuit::new(BranchTable::from_policy(&policy), nodes);
    let failed = failures(mock(&circuit));
    let table_rows = BranchTable::from_policy(&policy).rows();
    assert_eq!(failed, vec![format!("lookup node: cv is a branch of the table @ 0")], "table has {table_rows} rows");
}

#[test]
fn a_vector_outside_the_table_is_rejected() {
    // cv = e₀ − w[0]·e_W: ⟨cv, (w, 1)⟩ = w[0] − w[0] = 0, but cv is no branch.
    let (policy, mut nodes) = honest_nodes();
    let width = policy.layout().width();
    let mut forged = vec![Fp::ZERO; width + 1];
    forged[0] = Fp::ONE;
    forged[width] = -nodes[5].assignment[0];
    nodes[5].claim = Claim::Forged(forged);
    let circuit = BranchCircuit::new(BranchTable::from_policy(&policy), nodes);
    assert_eq!(failures(mock(&circuit)), vec!["lookup node: cv is a branch of the table @ 5".to_owned()]);
}

/// One branch over `w = (w₀)` with the rows `w₀ − 5 = 0` and `−w₀ + 5 = 0`.
fn cancelling_table() -> BranchTable {
    let five = Fp::from(5);
    BranchTable::new(1, vec![vec![vec![Fp::ONE, -five], vec![-Fp::ONE, five]]])
}

#[test]
fn cancelling_rows_need_the_challenge() {
    let table = cancelling_table();
    let node = |value: u64| NodeInput { assignment: vec![Fp::from(value)], claim: Claim::Branch(0) };

    assert_eq!(mock(&BranchCircuit::new(table.clone(), vec![node(5)])), Ok(()));

    // w₀ = 7 violates both rows: 2 and −2. With weights 1, 1 they cancel …
    let rows = [Fp::from(7) - Fp::from(5), Fp::from(5) - Fp::from(7)];
    assert_eq!(rows[0] + rows[1], Fp::ZERO);
    // … with weights 1, c they do not, and the circuit rejects.
    let failed = failures(mock(&BranchCircuit::new(table, vec![node(7)])));
    assert!(failed.iter().all(|failure| failure.contains("⟨cv, (w, 1)⟩ = 0")), "{failed:?}");
}
