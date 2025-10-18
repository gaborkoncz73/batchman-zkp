use std::fs;
use std::path::Path;

use halo2_proofs::pasta::Fp;
use halo2_proofs::pasta::group::ff::PrimeField;
use serde::{Serialize, Deserialize};
use base64::{engine::general_purpose, Engine as _};

/// Ugyanaz a szerkezet, mint amit a writer írt
#[derive(Serialize, Deserialize)]
pub struct ProofEntry {
    pub proof_b64: String,
    pub inputs_b64: Vec<Vec<String>>, // 2D structure: [ [c_vec], [flag_vec] ]
}

/// Loads proofs from out/<name>_proofs.json.
/// Returns a vector of (Vec<Vec<Fp>>, Vec<u8>) tuples.
pub fn read_proofs(name: &str) -> anyhow::Result<Vec<(Vec<Vec<Fp>>, Vec<u8>)>> {
    let file_path = Path::new("out").join(format!("{}_proofs.json", name));

    if !file_path.exists() {
        anyhow::bail!("File not found: {}", file_path.display());
    }

    let content = fs::read_to_string(&file_path)?;
    let entries: Vec<ProofEntry> = serde_json::from_str(&content)?;

    let proofs: Vec<(Vec<Vec<Fp>>, Vec<u8>)> = entries
        .into_iter()
        .map(|entry| {
            // Decode proof bytes from base64
            let proof_bytes = general_purpose::STANDARD
                .decode(&entry.proof_b64)
                .expect("Invalid base64 in proof_b64");

            // Decode all columns: [ [c_vec], [flag_vec] ]
            let inputs: Vec<Vec<Fp>> = entry.inputs_b64
                .iter()
                .map(|col| {
                    col.iter()
                        .map(|b64| {
                            let bytes = general_purpose::STANDARD
                                .decode(b64)
                                .expect("Invalid base64 in inputs_b64");

                            assert_eq!(
                                bytes.len(),
                                32,
                                "Invalid Fp length (expected 32 bytes, got {})",
                                bytes.len()
                            );

                            let mut repr = <Fp as PrimeField>::Repr::default();
                            repr.as_mut().copy_from_slice(&bytes);
                            Fp::from_repr(repr).expect("Invalid Fp repr")
                        })
                        .collect::<Vec<Fp>>()
                })
                .collect::<Vec<Vec<Fp>>>();

            (inputs, proof_bytes)
        })
        .collect();

    Ok(proofs)
}
