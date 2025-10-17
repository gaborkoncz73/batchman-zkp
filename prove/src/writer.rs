use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Write, Read},
    path::Path,
};

use halo2_proofs::pasta::Fp;
use halo2_proofs::pasta::group::ff::PrimeField;
use serde::{Serialize, Deserialize};
use base64::{engine::general_purpose, Engine as _};

#[derive(Serialize, Deserialize)]
struct ProofEntry {
    proof_b64: String,
    inputs_b64: Vec<String>,
}

/// Removes and recreates out/ each run
pub fn init_output_dir() -> anyhow::Result<()> {
    let out_dir = Path::new("out");
    if out_dir.exists() {
        fs::remove_dir_all(out_dir)?;
    }
    fs::create_dir_all(out_dir)?;
    Ok(())
}

/// Appends a _proof entry to a JSON array file: out/dot_proofs.json
pub fn write_proof(name: &str, proof_bytes: &[u8], inputs: Option<&[Fp]>) -> anyhow::Result<()> {
    let out_dir = Path::new("out");
    fs::create_dir_all(out_dir)?;
    let file_path = out_dir.join(format!("{}_proofs.json", name));

    // Encode proof and inputs in Base64
    let proof_b64 = general_purpose::STANDARD.encode(proof_bytes);
    let inputs_b64: Vec<String> = inputs
        .map(|vals| {
            vals.iter()
                .map(|x| {
                    let repr = x.to_repr(); // canonical 32-byte form
                    general_purpose::STANDARD.encode(repr.as_ref())
                })
                .collect()
        })
        .unwrap_or_default();

    let entry = ProofEntry { proof_b64, inputs_b64 };

    // Read existing file content if any
    let mut existing: Vec<ProofEntry> = if file_path.exists() {
        let mut content = String::new();
        fs::File::open(&file_path)?.read_to_string(&mut content)?;
        if content.trim().is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&content)?
        }
    } else {
        Vec::new()
    };

    // Append new entry
    existing.push(entry);

    // Overwrite file with a valid JSON array
    let json = serde_json::to_string_pretty(&existing)?;
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&file_path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(json.as_bytes())?;
    writer.flush()?;
    Ok(())
}
