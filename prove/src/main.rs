mod writer;
pub mod helpers;

use std::path::Path;
use std::{collections::HashMap, fs};
use std::sync::Arc;
use anyhow::Result;
use common::data::RuleTemplateFileFp;
use rand_core::OsRng;
use rayon::prelude::*;

use halo2_proofs::{
    pasta::{EqAffine, Fp},
    plonk::{keygen_pk, keygen_vk, ProvingKey, VerifyingKey},
    poly::commitment::Params,
    transcript::{Blake2bWrite, Challenge255},
};

use common::{data, data::UnificationInputFp};
use common::unification_checker_circuit::UnificationCircuit;
use common::io::read_fact_hashes::read_fact_hashes;
use data::Config;
use helpers::{build_fact_map, unification_input_from_goal_and_facts};

use writer::{write_proof};

use crate::writer::remove_proofs_file;

fn main() -> Result<()> {
    // Fact struct
    let config_file = "input/facts.yaml";
    let file_content = fs::read_to_string(config_file)
        .expect("Failed to read the YAML file.");
    let fact_configs: Vec<Config> = serde_yaml::from_str(&file_content)
        .expect("Wrong YAML format");

    // Building fact HashMap
    let facts = build_fact_map(&fact_configs);

    // Processing the rules
    let rules_text = fs::read_to_string("input/rules_template.json")?;
    let rules: data::RuleTemplateFile = serde_json::from_str(&rules_text)?;
    let rules_fp = RuleTemplateFileFp::from(&rules);

    // Processing the proof tree
    let proof_text = fs::read_to_string("input/proof_tree.json")?;
    let tree: Vec<data::ProofNode> = serde_json::from_str(&proof_text)?;

    // Public input hashes
    let path = Path::new("output/fact_hashes.json");
    let public_hashes: Vec<Fp> = read_fact_hashes(path)?;

    // Debug
    println!(
        "Loaded {} predicates, {} facts, {} proof nodes.",
        rules.predicates.len(),
        rules.facts.len(),
        tree.len()
    );
    
    // Params + keygen
    let params: Params<EqAffine> = Params::new(8);
    let shape = UnificationCircuit {
        rules: rules_fp.clone(),
        unif: UnificationInputFp::default(),
    };
    let vk: VerifyingKey<EqAffine> = keygen_vk(&params, &shape)?;
    let pk: ProvingKey<EqAffine> = keygen_pk(&params, vk.clone(), &shape)?;

    let params = Arc::new(params);
    let pk = Arc::new(pk);

    // Clearing the unif_proofs.json
    remove_proofs_file("unif_proofs.json")?;

    // Starting the proving from the root
    tree.iter()
        .try_for_each(|node|prove_tree(&rules_fp, node, &params,  &pk, &facts, &public_hashes))?;

    println!("All unification goals proof saved!");
    Ok(())
}

// Recursive proving function
fn prove_tree(
    rules_fp: &data::RuleTemplateFileFp,
    node: &data::ProofNode,
    params: &Arc<Params<EqAffine>>,
    pk: &Arc<ProvingKey<EqAffine>>,
    facts: &HashMap<String, Fp>,
    public_hashes: &Vec<Fp>,
) -> Result<()> {
    if let data::ProofNode::GoalNode(g) = node {
        
        // Constructing the Unification inputs from the goal node and the facts hashmap
        let unif_input_fp = unification_input_from_goal_and_facts(g, facts);

        // Circuit Fp with proper inputs
        let circuit = UnificationCircuit {
            rules: rules_fp.clone(),
            unif: unif_input_fp,
        };

        // Proof generation
        let mut transcript = Blake2bWrite::<Vec<u8>, _, Challenge255<_>>::init(vec![]);

        let public_hashes_slice: &[Fp] = &public_hashes;        
        let instances: &[&[&[Fp]]] = &[&[public_hashes_slice]]; 
        halo2_proofs::plonk::create_proof(
            params.as_ref(),
            pk.as_ref(),
            &[circuit],
            &instances,
            OsRng,
            &mut transcript,
        )?;
        let proof = transcript.finalize();

        write_proof("unif", &proof)?;

        // Recursion
        g.subtree.par_iter()
            .try_for_each(|sub| prove_tree(rules_fp, sub, params, pk, facts, &public_hashes))?;
    }
    Ok(())
}