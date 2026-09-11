pub mod helpers;
mod legacy_circuit_v1;
mod writer;

use anyhow::{Result, ensure};
use common::circuit_dialect::CircuitLimits;
use common::circuit_witness::TypedGoalFp;
use common::commitments::{
    LOCAL_STEP_PUBLIC_INPUT_COUNT, append_local_step_public_inputs, circuit_policy_commitment,
    local_step_commitments,
};
use common::policy_compiler::compile_policy_for_current_circuit;
use common::policy_ir::PolicyFile;
use common::proof_tree::{
    ProofTreeManifestV1, ProofTreeNodeV1, TrustedRootsV1, validate_compositional_proof_tree,
};
use common::typed_policy::TypedPolicyPlan;
use common::unification_checker_circuit::UnificationCircuit;
use std::fs;
use std::path::Path;
use std::sync::Arc;
//use common::utils_2::off_circuit_poseidon::poseidon_hash_list_native;
use rand_core::OsRng;
use rayon::prelude::*;

use halo2_proofs::{
    pasta::{EqAffine, Fp},
    plonk::{ProvingKey, VerifyingKey, keygen_pk, keygen_vk},
    poly::commitment::Params,
    transcript::{Blake2bWrite, Challenge255},
};

use common::{data, data::UnificationInputFp};
//use common::unification_checker_circuit::UnificationCircuit;
use common::io::read_fact_hashes::{read_fact_hashes, validate_fact_hashes_for_current_circuit};
use data::FactEntry;
use helpers::{ValidatedProofNode, semantic_local_step, validate_proof_tree};
use legacy_circuit_v1::{LegacyFactRegistry, encode_local_step_for_legacy_circuit};

use writer::write_proofs;

fn main() -> Result<()> {
    // Fact struct

    // facts.yaml is the MNB case study factsX X = the test we want
    let config_file = "issue/src/facts.yaml";

    let file_content = fs::read_to_string(config_file).expect("Failed to read the YAML file.");
    let fact_configs: Vec<FactEntry> =
        serde_yaml::from_str(&file_content).expect("Wrong YAML format");

    let limits_text = fs::read_to_string("config/circuit-limits.json")?;
    let limits: CircuitLimits = serde_json::from_str(&limits_text)?;
    let typed_facts = tokens::parse_fact_registry(&fact_configs, &limits, "issue/src/facts.yaml")?;

    // The compatibility bridge and typed public registry share this exact
    // parser-derived set of canonical ground calls.
    let facts = LegacyFactRegistry::new(&typed_facts)?;

    // The semantic policy is the only persisted rule source.  The bounded
    // representation required by the current circuit is compiled in memory.
    let policy_text = fs::read_to_string("input/rules.json")?;
    let policy: PolicyFile = serde_json::from_str(&policy_text)?;
    let compiled_policy = compile_policy_for_current_circuit(&policy, &limits)?;
    let predicate_count = compiled_policy.analysis().predicate_count;
    let legacy_policy = compiled_policy.layout().clone();
    let (rules_fp, typed_policy) = compiled_policy.into_circuit_parts();

    // Flar rules for rule hash commitment
    //let rules_vec_fp = RuleTemplateFileFp::to_flat_vec(&rules_fp);

    // Processing the proof tree
    let proof_text = fs::read_to_string("input/proof_tree.json")?;

    let raw_tree: Vec<data::ProofNode> = serde_json::from_str(&proof_text)?;
    let tree = validate_proof_tree(&raw_tree, &policy, &typed_facts, &limits)?;
    let trusted_roots_text = fs::read_to_string("input/trusted_roots.json")?;
    let trusted_roots: TrustedRootsV1 = serde_json::from_str(&trusted_roots_text)?;
    let trusted_root_goals = tokens::parse_trusted_root_goals(
        &trusted_roots,
        &policy,
        &limits,
        "input/trusted_roots.json",
    )?;
    ensure!(
        tree.roots.len() == trusted_root_goals.len(),
        "proof-tree root count {} does not match trusted root count {}",
        tree.roots.len(),
        trusted_root_goals.len()
    );
    for (index, (root, trusted)) in tree.roots.iter().zip(&trusted_root_goals).enumerate() {
        let ValidatedProofNode::Goal(root) = root else {
            unreachable!("validated proof roots are always goals")
        };
        ensure!(
            root.ground_call == *trusted,
            "proof-tree root {index} does not match input/trusted_roots.json"
        );
    }

    // Public input hashes
    let path = Path::new("input/fact_hashes.json");
    let public_facts_hashes: Vec<Fp> = read_fact_hashes(path)?;
    validate_fact_hashes_for_current_circuit(&public_facts_hashes, &limits)?;
    let mut expected_fact_hashes = typed_facts
        .iter()
        .map(|fact| fact.commitment().map_err(anyhow::Error::new))
        .collect::<Result<Vec<_>>>()?;
    expected_fact_hashes.resize(
        common::utils_2::common_helpers::MAX_FACTS_HASHES,
        Fp::zero(),
    );
    ensure!(
        public_facts_hashes == expected_fact_hashes,
        "input/fact_hashes.json does not match the parser-derived typed fact registry; regenerate it with `cargo run -p issue`"
    );
    //let public_rules_hashes = poseidon_hash_list_native(&rules_vec_fp);

    // Normalize and bridge every step before expensive key generation. This
    // also fixes the proof order to a deterministic depth-first index.
    let (typed_steps, manifest) = collect_semantic_forest(&tree.roots)?;
    let unification_inputs = typed_steps
        .iter()
        .map(|step| encode_local_step_for_legacy_circuit(step, &facts, &legacy_policy))
        .collect::<Result<Vec<_>>>()?;

    println!(
        "Loaded {} predicates, {} proof nodes.",
        predicate_count,
        unification_inputs.len()
    );

    // Params + keygen
    //let params: Params<EqAffine> = Params::new(16);

    //Testing

    //for tests
    let params: Params<EqAffine> = Params::new(16);

    let shape = UnificationCircuit {
        rules: rules_fp.clone(),
        typed_policy: typed_policy.clone(),
        unif: UnificationInputFp::default(),
    };
    let vk: VerifyingKey<EqAffine> = keygen_vk(&params, &shape)?;
    let pk: ProvingKey<EqAffine> = keygen_pk(&params, vk.clone(), &shape)?;

    let params = Arc::new(params);
    let pk = Arc::new(pk);

    let pool = rayon::ThreadPoolBuilder::new().num_threads(9).build()?;
    let proofs = pool.install(|| {
        unification_inputs
            .par_iter()
            .enumerate()
            .map(|(index, unif)| {
                let commitments = local_step_commitments(&rules_fp, &typed_policy, unif)?;
                let public_instance =
                    append_local_step_public_inputs(&public_facts_hashes, &commitments);
                ensure!(
                    public_instance.len() == LOCAL_STEP_PUBLIC_INPUT_COUNT,
                    "proof entry {index} produced {} public inputs; expected {LOCAL_STEP_PUBLIC_INPUT_COUNT}",
                    public_instance.len()
                );
                let proof = create_local_proof(
                    &rules_fp,
                    &typed_policy,
                    unif,
                    &params,
                    &pk,
                    &public_instance,
                )?;
                Ok((proof, commitments))
            })
            .collect::<Result<Vec<_>>>()
    })?;
    let proof_commitments = proofs
        .iter()
        .map(|(_, commitments)| *commitments)
        .collect::<Vec<_>>();
    let trusted_root_commitments = trusted_root_goals
        .iter()
        .map(|goal| TypedGoalFp::from_ground_call(goal).map(|typed| typed.hash()))
        .collect::<Result<Vec<_>, _>>()?;
    validate_compositional_proof_tree(
        &manifest,
        &proof_commitments,
        &trusted_root_commitments,
        circuit_policy_commitment(&rules_fp, &typed_policy),
        &limits,
    )?;
    write_proofs("unif", &manifest, &proofs)?;

    println!("All unification goals proof saved!");
    Ok(())
}

fn collect_semantic_forest(
    roots: &[ValidatedProofNode],
) -> Result<(
    Vec<common::witness_ir::LocalStepWitness>,
    ProofTreeManifestV1,
)> {
    fn collect_node(
        node: &ValidatedProofNode,
        steps: &mut Vec<common::witness_ir::LocalStepWitness>,
        nodes: &mut Vec<ProofTreeNodeV1>,
    ) -> Result<Option<usize>> {
        let ValidatedProofNode::Goal(goal) = node else {
            return Ok(None);
        };
        let index = steps.len();
        steps.push(semantic_local_step(goal)?);
        nodes.push(ProofTreeNodeV1 {
            child_indices: Vec::new(),
        });

        let mut child_indices = Vec::new();
        for child in &goal.subtree {
            if let Some(child_index) = collect_node(child, steps, nodes)? {
                child_indices.push(child_index);
            }
        }
        nodes[index].child_indices = child_indices;
        Ok(Some(index))
    }

    let mut steps = Vec::new();
    let mut nodes = Vec::new();
    let mut root_indices = Vec::with_capacity(roots.len());
    for root in roots {
        root_indices.push(
            collect_node(root, &mut steps, &mut nodes)?
                .expect("validated proof roots are always goals"),
        );
    }
    Ok((steps, ProofTreeManifestV1::new(root_indices, nodes)))
}

fn create_local_proof(
    rules_fp: &data::RuleTemplateFileFp,
    typed_policy: &TypedPolicyPlan,
    unif: &UnificationInputFp,
    params: &Arc<Params<EqAffine>>,
    pk: &Arc<ProvingKey<EqAffine>>,
    public_instance: &[Fp],
) -> Result<Vec<u8>> {
    let circuit = UnificationCircuit {
        rules: rules_fp.clone(),
        typed_policy: typed_policy.clone(),
        unif: unif.clone(),
    };
    let mut transcript: Blake2bWrite<Vec<u8>, EqAffine, Challenge255<EqAffine>> =
        Blake2bWrite::<Vec<u8>, _, Challenge255<_>>::init(Vec::new());
    let instance_columns: &[&[Fp]] = &[public_instance];
    let public_inputs: &[&[&[Fp]]] = &[instance_columns];
    halo2_proofs::plonk::create_proof(
        params.as_ref(),
        pk.as_ref(),
        &[circuit],
        public_inputs,
        OsRng,
        &mut transcript,
    )?;
    Ok(transcript.finalize())
}

#[cfg(test)]
mod composition_tests {
    use common::{
        circuit_dialect::CircuitLimits,
        circuit_witness::TypedGoalFp,
        commitments::{circuit_policy_commitment, local_step_commitments},
        data::{FactEntry, ProofNode},
        policy_compiler::compile_policy_for_current_circuit,
        policy_ir::PolicyFile,
        proof_tree::{TrustedRootsV1, validate_compositional_proof_tree},
    };

    use super::{collect_semantic_forest, validate_proof_tree};
    use crate::legacy_circuit_v1::{LegacyFactRegistry, encode_local_step_for_legacy_circuit};

    #[test]
    fn current_tree_forms_one_exact_manifest_anchored_at_the_trusted_root() {
        let policy: PolicyFile = serde_json::from_str(include_str!(
            "../../tests/fixtures/current-policy.rules.json"
        ))
        .unwrap();
        let limits: CircuitLimits =
            serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap();
        let raw_facts: Vec<FactEntry> =
            serde_yaml::from_str(include_str!("../../issue/src/facts.yaml")).unwrap();
        let facts = tokens::parse_fact_registry(&raw_facts, &limits, "facts.yaml").unwrap();
        let raw_tree: Vec<ProofNode> =
            serde_json::from_str(include_str!("../../tests/fixtures/current-proof-tree.json"))
                .unwrap();
        let tree = validate_proof_tree(&raw_tree, &policy, &facts, &limits).unwrap();
        let trusted: TrustedRootsV1 = serde_json::from_str(include_str!(
            "../../tests/fixtures/current-trusted-roots.json"
        ))
        .unwrap();
        let trusted_goals =
            tokens::parse_trusted_root_goals(&trusted, &policy, &limits, "trusted_roots").unwrap();

        let compiled = compile_policy_for_current_circuit(&policy, &limits).unwrap();
        let legacy_facts = LegacyFactRegistry::new(&facts).unwrap();
        let legacy_policy = compiled.layout().clone();
        let (rules, typed_policy) = compiled.into_circuit_parts();
        let (steps, manifest) = collect_semantic_forest(&tree.roots).unwrap();
        let commitments = steps
            .iter()
            .map(|step| {
                let unif =
                    encode_local_step_for_legacy_circuit(step, &legacy_facts, &legacy_policy)
                        .unwrap();
                local_step_commitments(&rules, &typed_policy, &unif).unwrap()
            })
            .collect::<Vec<_>>();
        let roots = trusted_goals
            .iter()
            .map(|goal| TypedGoalFp::from_ground_call(goal).unwrap().hash())
            .collect::<Vec<_>>();

        // One root and 49 authenticated parent-child edges.
        assert_eq!(manifest.nodes.len(), 50);
        validate_compositional_proof_tree(
            &manifest,
            &commitments,
            &roots,
            circuit_policy_commitment(&rules, &typed_policy),
            &limits,
        )
        .unwrap();
    }
}
