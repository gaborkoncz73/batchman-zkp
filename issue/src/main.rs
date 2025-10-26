use std::{fs, path::Path};
use anyhow::Result;
use halo2_proofs::pasta::Fp;
use num_bigint::BigUint;

use common::{
    data::Config,
    utils_2::{common_helpers::MAX_FACTS_HASHES, off_circuit_poseidon::fact_hash_native_salted},
};
use halo2curves::ff::PrimeField;

fn main() -> Result<()> {
    let config_file = "input/facts.yaml";
    let file_content = fs::read_to_string(config_file)?;
    let fact_configs: Vec<Config> = serde_yaml::from_str(&file_content)?;

    // Calculate the hashes as decimal strings
    let mut fact_hashes: Vec<String> = fact_configs
        .iter()
        .map(|f| {
            let args_ref: Vec<&str> = f.args.iter().map(|s| s.as_str()).collect();
            let hash_fp: Fp = fact_hash_native_salted(&f.predicate, &args_ref, &f.salt);
            let hash_bytes = hash_fp.to_repr();
            let hash_int = BigUint::from_bytes_le(hash_bytes.as_ref());
            hash_int.to_str_radix(10)
        })
        .collect();
    
    while fact_hashes.len() < MAX_FACTS_HASHES {
        fact_hashes.push("0".to_string());
    }
    
    // Write JSON file
    let out_path = Path::new("output/fact_hashes.json");
    fs::create_dir_all(out_path.parent().unwrap())?;
    let json = serde_json::to_string_pretty(&fact_hashes)?;
    fs::write(out_path, json)?;

    println!("Saved {} fact hashes to {:?}", fact_hashes.len(), out_path);
    Ok(())
}
