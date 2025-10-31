mod writer;
pub mod helpers;

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{collections::HashMap, fs};
use std::sync::Arc;
use anyhow::Result;
use common::data::{GoalEntry, ProofNode, RuleTemplateFileFp, TermFp};
use common::utils_2::common_helpers::to_fp_value;
//use common::utils_2::off_circuit_poseidon::poseidon_hash_list_native;
use rand_core::OsRng;
use rayon::prelude::*;

use halo2_proofs::{
    pasta::{EqAffine, Fp},
    plonk::{keygen_pk, keygen_vk, ProvingKey, VerifyingKey},
    poly::commitment::Params,
    transcript::{Blake2bWrite, Challenge255},
};

use common::{data, data::UnificationInputFp};
//use common::unification_checker_circuit::UnificationCircuit;
use common::io::read_fact_hashes::read_fact_hashes;
use data::Config;
use helpers::{build_fact_map/*, unification_input_from_goal_and_facts*/};

use serde::Serialize;
use serde_json::to_string_pretty;
use writer::{write_proof};

use crate::helpers::{encode_str_to_termfp, unification_input_from_goal_and_facts};
use crate::writer::remove_proofs_file;

/*fn main() -> Result<()> {
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
    
    // Flar rules for rule hash commitment
    //let rules_vec_fp = RuleTemplateFileFp::to_flat_vec(&rules_fp);

    // Processing the proof tree
    let proof_text = fs::read_to_string("input/proof_tree.json")?;
    let tree: Vec<data::ProofNode> = serde_json::from_str(&proof_text)?;

    // Public input hashes
    let path = Path::new("output/fact_hashes.json");
    let public_facts_hashes: Vec<Fp> = read_fact_hashes(path)?;
    //let public_rules_hashes = poseidon_hash_list_native(&rules_vec_fp);

    // Creating the public inputs
    let instance_columns: &[&[Fp]] = &[
        &public_facts_hashes/*,   // first instance column
        std::slice::from_ref(&public_rules_hashes),  // second instance column*/
    ];

    // Wrap into &[&[&[Fp]]] for create_proof
    let public_inputs: &[&[&[Fp]]] = &[instance_columns];
    // Debug
    println!(
        "Loaded {} predicates, {} facts, {} proof nodes.",
        rules.predicates.len(),
        rules.facts.len(),
        tree.len()
    );
    
    // Params + keygen
    /*let params: Params<EqAffine> = Params::new(8);
    let shape = UnificationCircuit {
        rules: rules_fp.clone(),
        unif: UnificationInputFp::default(),
    };
    let vk: VerifyingKey<EqAffine> = keygen_vk(&params, &shape)?;
    let pk: ProvingKey<EqAffine> = keygen_pk(&params, vk.clone(), &shape)?;

    let params = Arc::new(params);
    let pk = Arc::new(pk);*/

    // Clearing the unif_proofs.json
    //remove_proofs_file("unif_proofs.json")?;

    // Starting the proving from the root
    /*tree.iter()
        .try_for_each(|node|prove_tree(&rules_fp, node, &params,  &pk, &facts, &public_inputs))?;

    println!("All unification goals proof saved!");*/
    Ok(())
}

// Recursive proving function
/*fn prove_tree(
    rules_fp: &data::RuleTemplateFileFp,
    node: &data::ProofNode,
    params: &Arc<Params<EqAffine>>,
    pk: &Arc<ProvingKey<EqAffine>>,
    facts: &HashMap<String, Fp>,
    public_inputs: &[&[&[Fp]]],
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

        
        halo2_proofs::plonk::create_proof(
            params.as_ref(),
            pk.as_ref(),
            &[circuit],
            &public_inputs,
            OsRng,
            &mut transcript,
        )?;
        let proof = transcript.finalize();

        write_proof("unif", &proof)?;

        // Recursion
        g.subtree.par_iter()
            .try_for_each(|sub| prove_tree(rules_fp, sub, params, pk, facts, &public_inputs))?;
    }
    Ok(())
}*/ */



// -----------------------------------------------------------------------------
// A hierarchikus (rekurzívan kibontott) kimeneti csomópont
// -----------------------------------------------------------------------------

// A hierarchikus kimeneti fa (egy goal + a saját subtree-je)
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting recursive unification input export...");

    // ✅ facts.yaml
    let config_text = fs::read_to_string("input/facts.yaml")?;
    let fact_configs: Vec<data::Config> = serde_yaml::from_str(&config_text)?;
    let facts: HashMap<String, Fp> = build_fact_map(&fact_configs);

    // ✅ proof_tree.json
    let tree_text = fs::read_to_string("input/proof_tree.json")?;
    let proof_nodes: Vec<ProofNode> = serde_json::from_str(&tree_text)?;

    println!("📌 Loaded {} proof nodes", proof_nodes.len());

    // ✅ DFS traversal of all proof nodes
    let mut results: Vec<UnificationInputFp> = Vec::new();
    for node in proof_nodes.iter() {
        collect_unification_inputs(node, &facts, &mut results);
    }

    // ✅ Output JSON
    let json_output = to_string_pretty(&results)?;
    let mut file = File::create("output/unification_inputs.json")?;
    file.write_all(json_output.as_bytes())?;

    println!("✅ Saved {} unification inputs → output/unification_inputs.json", results.len());
    Ok(())
}

/// Recursively walk DFS on the proof tree
fn collect_unification_inputs(
    node: &ProofNode,
    facts: &HashMap<String, Fp>,
    results: &mut Vec<UnificationInputFp>,
) {
    if let ProofNode::GoalNode(g) = node {

        // ✅ Encode this goal itself
        let unif = build_unification_from_goal(g, facts);
        results.push(unif);

        // ✅ Recurse deeply into subtree
        for list in g.subtree.iter() {
                collect_unification_inputs(list, facts, results);
        
        }
    }
}

/// Build a full unification input for ONE goal node
fn build_unification_from_goal(
    g: &GoalEntry,
    facts: &HashMap<String, Fp>,
) -> UnificationInputFp {

    // ✅ goal_name: Vec<TermFp>
    let goal_name_terms = encode_str_to_termfp(&g.goal, facts);

    // ✅ subtree_goals: Vec<Vec<TermFp>>
    let subtree_terms: Vec<Vec<TermFp>> = g.subtree
    .iter()
    .map(|subnode| {
        if let ProofNode::GoalNode(subg) = subnode {
            // ✅ encode this immediate subtree goal
            encode_str_to_termfp(&subg.goal, facts)
        } else {
            Vec::new()
        }
    })
    .collect();

    UnificationInputFp {
        goal_name: goal_name_terms,
        subtree_goals: subtree_terms,
    }
}