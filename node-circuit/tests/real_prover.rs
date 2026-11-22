//! The node circuit with the real prover and verifier (IPA over Pasta), not
//! only MockProver: the real prover sees less than MockProver (for example
//! not the values of fixed cells), so a circuit can pass one and fail the
//! other.

use ff::Field;
use halo2_poseidon::poseidon::primitives::P128Pow5T3;
use halo2_proofs::halo2curves::pasta::EqAffine;
use halo2_proofs::plonk::{Circuit, create_proof, keygen_pk, keygen_vk, verify_proof};
use halo2_proofs::poly::VerificationStrategy;
use halo2_proofs::poly::commitment::ParamsProver;
use halo2_proofs::poly::ipa::commitment::{IPACommitmentScheme, ParamsIPA};
use halo2_proofs::poly::ipa::multiopen::{ProverIPA, VerifierIPA};
use halo2_proofs::poly::ipa::strategy::SingleStrategy;
use halo2_proofs::transcript::{
    Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
};
use node_circuit::{
    BranchTable, Claim, NodeBranch, NodeCircuit, TreeCircuit, minimum_k, root_digests,
};
use policy_constraints::term::fact_commitment;
use policy_constraints::{Fp, GroundTerm, compile};
use policy_parser::{parse_goal, parse_policy};
use proof_witness::{NodeWitness, ProofWitness, TrustedRoots};
use rand_core::OsRng;

/// One node proving `goal` from `children` by the branch printed as `label`,
/// completed with `salt`; proved with the real prover against `published`,
/// then verified. False if the prover gives up (a lookup input it cannot
/// find in its table) or the verifier rejects the proof.
fn prove(
    policy: &str,
    goal: &str,
    children: &[&str],
    label: &str,
    salt: Fp,
    published: &[Fp],
) -> bool {
    let system = compile(&parse_policy(policy, "test.pl").unwrap()).unwrap();
    let branch = system
        .branches()
        .iter()
        .find(|branch| branch.kind.to_string() == label)
        .unwrap()
        .id;
    let ground = |goal: &str| GroundTerm::from_term(&parse_goal(goal, "goal").unwrap()).unwrap();
    let children: Vec<GroundTerm> = children.iter().map(|goal| ground(goal)).collect();
    let mut w = system.layout().assign(&ground(goal), &children).unwrap();
    system.complete(branch, &mut w, salt).unwrap();
    let circuit = NodeCircuit::<Fp, P128Pow5T3>::new(
        system.layout().clone(),
        BranchTable::from_policy(&system),
        vec![NodeBranch {
            assignment: w,
            claim: Claim::Branch(branch),
        }],
    );
    prove_and_verify(circuit, &[published.to_vec()])
}

/// Proves `circuit` with the real prover against the instance columns
/// `instances`, then verifies the proof. False if the prover gives up or the
/// verifier rejects the proof.
fn prove_and_verify<C: Circuit<Fp>>(circuit: C, instances: &[Vec<Fp>]) -> bool {
    let columns: Vec<&[Fp]> = instances.iter().map(Vec::as_slice).collect();
    let params: ParamsIPA<EqAffine> = ParamsIPA::new(minimum_k(&circuit));
    let vk = keygen_vk(&params, &circuit).unwrap();
    let pk = keygen_pk(&params, vk, &circuit).unwrap();
    let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
    if let Err(error) = create_proof::<IPACommitmentScheme<EqAffine>, ProverIPA<EqAffine>, _, _, _, _>(
        &params,
        &pk,
        &[circuit],
        &[&columns],
        OsRng,
        &mut transcript,
    ) {
        eprintln!("the prover gives up: {error:?}");
        return false;
    }
    let proof = transcript.finalize();

    let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof[..]);
    let verified = verify_proof::<IPACommitmentScheme<EqAffine>, VerifierIPA<EqAffine>, _, _, _>(
        &params,
        pk.get_vk(),
        SingleStrategy::new(&params),
        &[&columns],
        &mut transcript,
    );
    if let Err(error) = &verified {
        eprintln!("the verifier rejects the proof: {error:?}");
    }
    verified.is_ok()
}

const FACTS: &str = "p :- q(1).\n";

fn commitment(goal: &str, salt: Fp) -> Fp {
    let goal = GroundTerm::from_term(&parse_goal(goal, "goal").unwrap()).unwrap();
    fact_commitment(goal.digest(), salt)
}

#[test]
fn an_arithmetic_node_is_proved_and_verified() {
    assert!(prove(
        "p(X) :- X is 2 + 3.\n",
        "5 is 2+3",
        &[],
        "builtin _ is (_ + _)",
        Fp::ZERO,
        &[],
    ));
}

#[test]
fn a_published_fact_is_proved_and_verified() {
    let salt = Fp::random(OsRng);
    assert!(prove(
        FACTS,
        "q(1)",
        &[],
        "external fact q/1",
        salt,
        &[commitment("q(1)", salt)],
    ));
}

#[test]
fn a_clause_node_of_a_policy_with_facts_is_proved_and_verified() {
    assert!(prove(
        FACTS,
        "p",
        &["q(1)"],
        "clause p/0 #0 alternative 0",
        Fp::ZERO,
        &[commitment("q(1)", Fp::ONE)],
    ));
}

#[test]
fn an_unpublished_fact_does_not_verify() {
    let salt = Fp::random(OsRng);
    assert!(!prove(
        FACTS,
        "q(2)",
        &[],
        "external fact q/1",
        salt,
        &[commitment("q(1)", salt)],
    ));
}

#[test]
fn a_false_sum_does_not_verify() {
    // The prover runs on a false witness, but the proof does not verify.
    assert!(!prove(
        "p(X) :- X is 2 + 3.\n",
        "6 is 2+3",
        &[],
        "builtin _ is (_ + _)",
        Fp::ZERO,
        &[],
    ));
}

#[test]
fn a_clause_node_without_its_body_goal_does_not_verify() {
    assert!(!prove(
        FACTS,
        "p",
        &[],
        "clause p/0 #0 alternative 0",
        Fp::ZERO,
        &[commitment("q(1)", Fp::ONE)],
    ));
}

/// A tree of `(goal, body, branch)` nodes for `policy`, the first one the
/// root, laid out in 4 node places; proved with the real prover and verified
/// with `public_root` as the trusted root.
fn prove_tree(
    policy: &str,
    nodes: &[(&str, &[&str], &str)],
    published: &[Fp],
    public_root: &str,
) -> bool {
    let system = compile(&parse_policy(policy, "test.pl").unwrap()).unwrap();
    let ground = |goal: &str| GroundTerm::from_term(&parse_goal(goal, "goal").unwrap()).unwrap();
    let nodes = nodes
        .iter()
        .map(|&(goal, body, label)| {
            let branch = system
                .branches()
                .iter()
                .find(|branch| branch.kind.to_string() == label)
                .unwrap()
                .id;
            let body: Vec<GroundTerm> = body.iter().map(|goal| ground(goal)).collect();
            let mut assignment = system.layout().assign(&ground(goal), &body).unwrap();
            system.complete(branch, &mut assignment, Fp::ZERO).unwrap();
            NodeWitness {
                text: goal.to_owned(),
                goal: ground(goal),
                parent: None,
                children: Vec::new(),
                branch,
                assignment,
            }
        })
        .collect();
    let witness = ProofWitness {
        nodes,
        roots: vec![0],
    };
    let roots = TrustedRoots::new(&[witness.nodes[0].text.as_str()], "roots").unwrap();
    let circuit =
        TreeCircuit::<Fp, P128Pow5T3>::from_witness(&system, &witness, &roots, 4).unwrap();
    let public = TrustedRoots::new(&[public_root], "public roots").unwrap();
    prove_and_verify(
        circuit,
        &[published.to_vec(), root_digests(&public), Vec::new()],
    )
}

const TREE: &str = "r :- p.\np :- p.\np :- q(1).\n";

#[test]
fn a_whole_tree_is_proved_and_verified() {
    let published = [commitment("q(1)", Fp::ZERO)];
    assert!(prove_tree(
        TREE,
        &[
            ("r", &["p"], "clause r/0 #0 alternative 0"),
            ("p", &["q(1)"], "clause p/0 #1 alternative 0"),
            ("q(1)", &[], "external fact q/1"),
        ],
        &published,
        "r",
    ));
}

#[test]
fn a_circular_tree_does_not_verify() {
    let published = [commitment("q(1)", Fp::ZERO)];
    assert!(!prove_tree(
        TREE,
        &[
            ("r", &["p"], "clause r/0 #0 alternative 0"),
            ("p", &["p"], "clause p/0 #0 alternative 0"),
        ],
        &published,
        "r",
    ));
}

#[test]
fn a_tree_for_another_root_does_not_verify() {
    // An honest tree for r, verified against the trusted root p: the root's
    // digest is not the public one.
    let published = [commitment("q(1)", Fp::ZERO)];
    let nodes: &[(&str, &[&str], &str)] = &[
        ("r", &["p"], "clause r/0 #0 alternative 0"),
        ("p", &["q(1)"], "clause p/0 #1 alternative 0"),
        ("q(1)", &[], "external fact q/1"),
    ];
    assert!(!prove_tree(TREE, nodes, &published, "p"));
}
