use std::{fs, path::Path};
use anyhow::Result;
use halo2_proofs::pasta::Fp;
use halo2curves::ff::PrimeField;
use num_bigint::BigUint;    

pub fn read_fact_hashes(path: &Path) -> Result<Vec<Fp>> {
    // Read JSON file
    let content = fs::read_to_string(path)?;
    
    // Deserialize as Vec<String>
    let hash_strings: Vec<String> = serde_json::from_str(&content)?;
    
    // Convert decimal strings back to Fp
    let hashes_fp: Vec<Fp> = hash_strings
        .into_iter()
        .map(|s| {
            let int_val = BigUint::parse_bytes(s.as_bytes(), 10)
                .expect("Invalid number in fact_hashes.json");
            let mut bytes = [0u8; 32];
            let int_bytes = int_val.to_bytes_le();
            bytes[..int_bytes.len()].copy_from_slice(&int_bytes);
            Fp::from_repr(bytes).expect("Invalid Fp conversion")
        })
        .collect();


    Ok(hashes_fp)
}