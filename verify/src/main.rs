mod reader;

use common::data::{RuleTemplateFile, RuleTemplateFileFp, UnificationInputFp};
use common::unification_checker_circuit::UnificationCircuit;
use common::io::read_fact_hashes::read_fact_hashes;
//use common::utils_2::off_circuit_poseidon::poseidon_hash_list_native;
use reader::read_proofs_bytes;

use std::{fs, path::Path};
use std::sync::Arc;
use rayon::prelude::*;
use anyhow::Result;

use halo2_proofs::{
    pasta::{EqAffine, Fp},
    plonk::{keygen_vk, verify_proof, SingleVerifier, VerifyingKey},
    poly::commitment::Params,
    transcript::{Blake2bRead, Challenge255},
};

fn main() -> Result<()> {
    // Load proofs
    let proofs: Vec<Vec<u8>> = read_proofs_bytes("unif")?;
    // Load public hashes
    let path = Path::new("output/fact_hashes.json");
    let public_hashes = read_fact_hashes(path)?;

    // Debug (12 bytes/proof)
    println!("Verifying {} unification proofs", proofs.len());

    // Load Rules
    let rules_text = fs::read_to_string("input/rules.json")?;
    let rules: RuleTemplateFile = serde_json::from_str(&rules_text)?;

    let rules_fp = RuleTemplateFileFp::from(&rules);
    //let flatten_rules_fp = RuleTemplateFileFp::to_flat_vec(&rules_fp);
    //let public_rules_hashes = poseidon_hash_list_native(&flatten_rules_fp);
    // Same params + vkgen
    let params: Params<EqAffine> = Params::new(16);
    let shape = UnificationCircuit {
        rules: rules_fp,
        unif: UnificationInputFp::default(),
    };

    let vk: VerifyingKey<EqAffine> = keygen_vk(&params, &shape)?;
    let params = Arc::new(params);
    let vk = Arc::new(vk);

    // Constructing the public inputs
    let instance_columns: &[&[Fp]] = &[
        &public_hashes/*,   // first instance column
        std::slice::from_ref(&public_rules_hashes), // second instance column*/
    ];

    // Wrap into &[&[&[Fp]]] for create_proof
    let public_inputs: &[&[&[Fp]]] = &[instance_columns];

    // Parallel verification
    let ok = proofs
        .par_iter()
        .all(|proof| {
            let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof[..]);
            let strategy = SingleVerifier::new(params.as_ref());
            verify_proof(params.as_ref(), vk.as_ref(), strategy, &public_inputs, &mut transcript).is_ok()
        });

    if ok {
        println!("All proofs verified successfully!");
    } else {
        println!("Some proofs failed verification!");
    }

    Ok(())
}