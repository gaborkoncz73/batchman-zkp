//! End-to-end cryptographic smoke test.
//!
//! This is ignored in the fast suite because parameter generation and proving
//! at k=16 are intentionally expensive.  Run it in release mode with:
//!
//! `cargo test --release -p common --test real_proof -- --ignored --nocapture`

use common::{
    circuit_dialect::CircuitLimits,
    circuit_witness::{TypedGoalFp, TypedLocalStepFp},
    commitments::{append_local_step_public_inputs, local_step_commitments},
    data::{LocalStepKind, TermFp, UnificationInputFp},
    policy_compiler::compile_policy_for_current_circuit,
    policy_ir::{Clause, PolicyFile, PredicateDef, PredicateIndicator, Term},
    unification_checker_circuit::UnificationCircuit,
    utils_2::common_helpers::{MAX_ARITY, MAX_FACTS_HASHES, MAX_PRED_LIST, to_fp_value},
    witness_ir::{GroundCall, GroundTerm},
};
use halo2_proofs::{
    pasta::{EqAffine, Fp},
    plonk::{SingleVerifier, create_proof, keygen_pk, keygen_vk, verify_proof},
    poly::commitment::Params,
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use rand_core::OsRng;

const TEST_K: u32 = 16;

fn term(name: &str, argument: &str) -> TermFp {
    let mut args = vec![vec![-Fp::one(); MAX_PRED_LIST]; MAX_ARITY];
    args[0][0] = to_fp_value(argument);
    TermFp {
        name: to_fp_value(name),
        args,
        fact_hashes: Fp::zero(),
    }
}

fn one_rule_policy(parent: &str, child: &str) -> PolicyFile {
    let indicator = PredicateIndicator {
        name: parent.to_owned(),
        arity: 1,
    };
    PolicyFile {
        predicates: vec![PredicateDef {
            indicator: indicator.clone(),
            clauses: vec![Clause {
                variables: vec![Some("X".to_owned())],
                head: vec![Term::Variable { id: 0 }],
                body: Term::compound(child, vec![Term::Variable { id: 0 }]),
            }],
        }],
    }
}

fn ground_goal(name: &str, argument: &str) -> GroundCall {
    GroundCall {
        predicate: PredicateIndicator {
            name: name.to_owned(),
            arity: 1,
        },
        args: vec![GroundTerm::Atom {
            value: argument.to_owned(),
        }],
    }
}

fn typed_local_step(parent: GroundCall, children: Vec<GroundCall>) -> TypedLocalStepFp {
    TypedLocalStepFp {
        goal: TypedGoalFp::from_ground_call(&parent).unwrap(),
        children: children
            .iter()
            .map(|child| TypedGoalFp::from_ground_call(child).unwrap())
            .collect(),
        ground_goal: parent,
        ground_children: children,
    }
}

fn rule_circuit(parent: &str, child: &str) -> UnificationCircuit {
    let limits: CircuitLimits =
        serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap();
    let (rules, typed_policy) =
        compile_policy_for_current_circuit(&one_rule_policy(parent, child), &limits)
            .unwrap()
            .into_circuit_parts();
    UnificationCircuit {
        rules,
        typed_policy,
        unif: UnificationInputFp {
            goal_name: vec![term(parent, "a")],
            subtree_goals: vec![vec![term(child, "a")]],
            resolution: LocalStepKind::Rule,
            typed: typed_local_step(ground_goal(parent, "a"), vec![ground_goal(child, "a")]),
        },
    }
}

fn honest_rule_circuit() -> UnificationCircuit {
    rule_circuit("p", "q")
}

fn verifies(
    params: &Params<EqAffine>,
    vk: &halo2_proofs::plonk::VerifyingKey<EqAffine>,
    proof: &[u8],
    instances: &[Fp],
) -> bool {
    let instance_columns: &[&[Fp]] = &[instances];
    let public_inputs: &[&[&[Fp]]] = &[instance_columns];
    let strategy = SingleVerifier::new(params);
    let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(proof);
    verify_proof(params, vk, strategy, public_inputs, &mut transcript).is_ok()
}

#[test]
#[ignore = "slow cryptographic integration test; run the command documented at the top of this file"]
fn real_proof_round_trip_and_public_input_binding() {
    let params: Params<EqAffine> = Params::new(TEST_K);
    let circuit = honest_rule_circuit();
    let shape = UnificationCircuit {
        rules: circuit.rules.clone(),
        typed_policy: circuit.typed_policy.clone(),
        unif: UnificationInputFp::default(),
    };
    let vk = keygen_vk(&params, &shape).expect("verifying-key generation must succeed");
    let pk = keygen_pk(&params, vk.clone(), &shape).expect("proving-key generation must succeed");

    let public_fact_hashes = vec![Fp::zero(); MAX_FACTS_HASHES];
    let commitments =
        local_step_commitments(&circuit.rules, &circuit.typed_policy, &circuit.unif).unwrap();
    let public_inputs_values = append_local_step_public_inputs(&public_fact_hashes, &commitments);
    let instance_columns: &[&[Fp]] = &[&public_inputs_values];
    let public_inputs: &[&[&[Fp]]] = &[instance_columns];
    let mut transcript = Blake2bWrite::<Vec<u8>, EqAffine, Challenge255<_>>::init(Vec::new());
    create_proof(
        &params,
        &pk,
        &[circuit],
        public_inputs,
        OsRng,
        &mut transcript,
    )
    .expect("real proof creation must succeed");
    let proof = transcript.finalize();

    assert_eq!(proof.len(), 9088, "serialized proof-size baseline changed");
    assert!(verifies(&params, &vk, &proof, &public_inputs_values));

    let mut corrupted_proof = proof.clone();
    let middle = corrupted_proof.len() / 2;
    corrupted_proof[middle] ^= 1;
    assert!(
        !verifies(&params, &vk, &corrupted_proof, &public_inputs_values),
        "modified proof bytes must be rejected"
    );

    for row in 0..public_inputs_values.len() {
        let mut tampered_public_inputs = public_inputs_values.clone();
        tampered_public_inputs[row] += Fp::one();
        assert!(
            !verifies(&params, &vk, &proof, &tampered_public_inputs),
            "the same proof must not verify after changing public-input row {row}"
        );
    }
}

#[test]
#[ignore = "slow cryptographic cross-policy integration test"]
fn a_proof_for_one_policy_must_not_verify_under_another_policy() {
    let params: Params<EqAffine> = Params::new(TEST_K);
    let policy_a = rule_circuit("p", "q");
    let policy_b = rule_circuit("r", "s");
    let vk_a = keygen_vk(&params, &policy_a).unwrap();
    let pk_a = keygen_pk(&params, vk_a, &policy_a).unwrap();
    let vk_b = keygen_vk(&params, &policy_b).unwrap();

    let public_fact_hashes = vec![Fp::zero(); MAX_FACTS_HASHES];
    let commitments_a =
        local_step_commitments(&policy_a.rules, &policy_a.typed_policy, &policy_a.unif).unwrap();
    let public_inputs_a = append_local_step_public_inputs(&public_fact_hashes, &commitments_a);
    let instance_columns: &[&[Fp]] = &[&public_inputs_a];
    let public_inputs: &[&[&[Fp]]] = &[instance_columns];
    let mut transcript = Blake2bWrite::<Vec<u8>, EqAffine, Challenge255<_>>::init(Vec::new());
    create_proof(
        &params,
        &pk_a,
        &[policy_a],
        public_inputs,
        OsRng,
        &mut transcript,
    )
    .unwrap();
    let proof = transcript.finalize();

    assert!(
        !verifies(&params, &vk_b, &proof, &public_inputs_a),
        "a proof created for policy A must be rejected by policy B"
    );
}
