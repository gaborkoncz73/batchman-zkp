mod data;
mod utils;
mod proofs;
mod logic;
mod writer;

use anyhow::Result;
use std::fs;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use common::*;
use data::*;
use logic::*;
use proofs::*;
use utils::*;
use writer::write_proof;

pub const MAX_DOT_DIM: usize = 7;


fn main() -> Result<()> {
    // Load rules and proof tree
    let rules_text = fs::read_to_string("input/rules_template.json")?;
    let rules: RuleTemplateFile = serde_json::from_str(&rules_text)?;
    let proof_text = fs::read_to_string("input/proof_tree.json")?;
    let tree: Vec<ProofNode> = serde_json::from_str(&proof_text)?;

    let id_map = build_predicate_id_map(&rules);

    let proofs = ProofStore {
        dot_proofs: Arc::new(Mutex::new(Vec::new())),
        cons_proofs: Arc::new(Mutex::new(Vec::new())),
    };

    let pk_store = Arc::new(ProvingKeyStore::new(MAX_DOT_DIM, 5));

    

    println!(
        "Loaded {} predicates, {} facts, {} proof nodes.",
        rules.predicates.len(),
        rules.facts.len(),
        tree.len()
    );

    // Main recursion
    tree.par_iter()
        .filter_map(|n| if let ProofNode::GoalNode(g) = n { Some(g) } else { None })
        .map(|g| prove_node(g, &rules, &id_map, &proofs, &pk_store, 0))
        .collect::<Result<Vec<_>>>()?;

    // Proof export
    let dot_proofs = proofs.dot_proofs.lock().unwrap().clone();
    let cons_proofs = proofs.cons_proofs.lock().unwrap().clone();

    // cleaning the output folder
    writer::init_output_dir()?;

    // DOT proofs
    for (inputs, proof_bytes) in dot_proofs.iter() {
        write_proof("dot", proof_bytes, Some(inputs))?;
    }

    // CONSISTENCY proofs (no public inputs)
    for proof_bytes in cons_proofs.iter() {
    write_proof("cons", proof_bytes.as_slice(), None)?;
}
    println!("Proof generation and exportation finished!");
    Ok(())
}
