use std::fs;
use std::path::Path;
use halo2_proofs::pasta::Fp;
use serde::{Serialize, Deserialize};
use base64::{engine::general_purpose, Engine as _};

#[derive(Serialize, Deserialize)]
pub struct ProofEntry {
    pub proof_b64: String,
    pub inputs_b64: Vec<Vec<String>>,
}

pub fn read_proofs(name: &str) -> anyhow::Result<Vec<(Vec<Vec<Fp>>, Vec<u8>)>> {
    let file_path = Path::new("out").join(format!("{}_proofs.json", name));
    if !file_path.exists() {
        anyhow::bail!("File not found: {}", file_path.display());
    }

    let content = fs::read_to_string(&file_path)?;
    let entries: Vec<ProofEntry> = serde_json::from_str(&content)?;

    let proofs: Vec<(Vec<Vec<Fp>>, Vec<u8>)> = entries.into_iter().map(|entry| {
        let proof_bytes = general_purpose::STANDARD.decode(&entry.proof_b64).unwrap();
        (vec![], proof_bytes)
    }).collect();

    Ok(proofs)
}
