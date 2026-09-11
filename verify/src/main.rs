mod reader;

use common::circuit_dialect::CircuitLimits;
use common::circuit_witness::TypedGoalFp;
use common::commitments::{
    LOCAL_STEP_PUBLIC_INPUT_COUNT, append_local_step_public_inputs, circuit_policy_commitment,
};
use common::data::UnificationInputFp;
use common::io::read_fact_hashes::{read_fact_hashes, validate_fact_hashes_for_current_circuit};
use common::policy_compiler::compile_policy_for_current_circuit;
use common::policy_ir::PolicyFile;
use common::proof_tree::{TrustedRootsV1, validate_compositional_proof_tree};
use common::unification_checker_circuit::UnificationCircuit;
//use common::utils_2::off_circuit_poseidon::poseidon_hash_list_native;
use reader::read_proofs;

use anyhow::{Result, bail, ensure};
use rayon::prelude::*;
use std::sync::Arc;
use std::{fs, path::Path};

use halo2_proofs::{
    pasta::{EqAffine, Fp},
    plonk::{SingleVerifier, VerifyingKey, keygen_vk, verify_proof},
    poly::commitment::Params,
    transcript::{Blake2bRead, Challenge255},
};

fn main() -> Result<()> {
    // Load proofs
    let proof_tree = read_proofs("unif")?;
    let proofs = &proof_tree.proofs;
    if proofs.is_empty() {
        bail!("proof set is empty");
    }
    // Load public hashes
    let path = Path::new("input/fact_hashes.json");
    let public_hashes = read_fact_hashes(path)?;

    // Debug (12 bytes/proof)
    println!("Verifying {} unification proofs", proofs.len());

    // The semantic policy is the only persisted rule source. Prover and
    // verifier call the same deterministic in-memory compiler.
    let limits_text = fs::read_to_string("config/circuit-limits.json")?;
    let limits: CircuitLimits = serde_json::from_str(&limits_text)?;
    validate_fact_hashes_for_current_circuit(&public_hashes, &limits)?;
    let policy_text = fs::read_to_string("input/rules.json")?;
    let policy: PolicyFile = serde_json::from_str(&policy_text)?;
    let trusted_roots_text = fs::read_to_string("input/trusted_roots.json")?;
    let trusted_roots: TrustedRootsV1 = serde_json::from_str(&trusted_roots_text)?;
    let trusted_root_goals = tokens::parse_trusted_root_goals(
        &trusted_roots,
        &policy,
        &limits,
        "input/trusted_roots.json",
    )?;
    let trusted_root_commitments = trusted_root_goals
        .iter()
        .map(|goal| TypedGoalFp::from_ground_call(goal).map(|typed| typed.hash()))
        .collect::<Result<Vec<_>, _>>()?;
    let compiled_policy = compile_policy_for_current_circuit(&policy, &limits)?;
    let (rules_fp, typed_policy) = compiled_policy.into_circuit_parts();
    let expected_policy_commitment = circuit_policy_commitment(&rules_fp, &typed_policy);
    let proof_commitments = proofs
        .iter()
        .map(|proof| proof.commitments)
        .collect::<Vec<_>>();
    validate_compositional_proof_tree(
        &proof_tree.manifest,
        &proof_commitments,
        &trusted_root_commitments,
        expected_policy_commitment,
        &limits,
    )?;
    let public_instances = proofs
        .iter()
        .enumerate()
        .map(|(index, proof)| {
            ensure!(
                proof.commitments.policy == expected_policy_commitment,
                "proof entry {index} belongs to a different policy"
            );
            let public_instance =
                append_local_step_public_inputs(&public_hashes, &proof.commitments);
            ensure!(
                public_instance.len() == LOCAL_STEP_PUBLIC_INPUT_COUNT,
                "proof entry {index} produced {} public inputs; expected {LOCAL_STEP_PUBLIC_INPUT_COUNT}",
                public_instance.len()
            );
            Ok(public_instance)
        })
        .collect::<Result<Vec<_>>>()?;
    //let flatten_rules_fp = RuleTemplateFileFp::to_flat_vec(&rules_fp);
    //let public_rules_hashes = poseidon_hash_list_native(&flatten_rules_fp);
    // Same params + vkgen
    let params: Params<EqAffine> = Params::new(16);
    let shape = UnificationCircuit {
        rules: rules_fp,
        typed_policy,
        unif: UnificationInputFp::default(),
    };

    let vk: VerifyingKey<EqAffine> = keygen_vk(&params, &shape)?;
    let params = Arc::new(params);
    let vk = Arc::new(vk);

    // Parallel verification
    let ok = proofs
        .par_iter()
        .zip(public_instances.par_iter())
        .all(|(proof, public_instance)| {
            let instance_columns: &[&[Fp]] = &[public_instance];
            let public_inputs: &[&[&[Fp]]] = &[instance_columns];
            let mut transcript =
                Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof.proof_bytes[..]);
            let strategy = SingleVerifier::new(params.as_ref());
            verify_proof(
                params.as_ref(),
                vk.as_ref(),
                strategy,
                &public_inputs,
                &mut transcript,
            )
            .is_ok()
        });

    if ok {
        println!("All proofs verified successfully!");
    } else {
        bail!("one or more proofs failed verification");
    }

    Ok(())
}
