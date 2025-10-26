use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use base64::{engine::general_purpose, Engine as _};
use anyhow::Result;

#[derive(Serialize, Deserialize)]
pub struct ProofEntry {
    pub proof_b64: String,
}

pub fn read_proofs_bytes(name: &str) -> Result<Vec<Vec<u8>>> {
    let file_path = Path::new("output").join(format!("{}_proofs.json", name));
    if !file_path.exists() {
        anyhow::bail!("File not found: {}", file_path.display());
    }

    let content = fs::read_to_string(&file_path)?;
    let entries: Vec<ProofEntry> = serde_json::from_str(&content)?;

    let proofs_bytes: Vec<Vec<u8>> = entries
        .into_iter()
        .map(|entry| general_purpose::STANDARD.decode(&entry.proof_b64).unwrap())
        .collect();

    Ok(proofs_bytes)
}