use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Read, Write},
    path::Path, sync::Mutex,
};
use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use base64::{engine::general_purpose, Engine as _};
#[derive(Serialize, Deserialize)]
struct ProofEntry {
    proof_b64: String,
}
pub fn remove_proofs_file(name: &str) -> Result<()> {
    let file_path = Path::new("output").join(format!("{}", name));
    if file_path.exists() {
        fs::remove_file(file_path)?;
    }
    Ok(())
}

static FILE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub fn write_proof(name: &str, proof_bytes: &[u8]) -> anyhow::Result<()> {
    // Mutex lock – one thread can write at a time
    let _guard = FILE_LOCK.lock().unwrap();

    let out_dir = Path::new("output");
    fs::create_dir_all(out_dir)?;
    let file_path = out_dir.join(format!("{}_proofs.json", name));

    let proof_b64 = general_purpose::STANDARD.encode(proof_bytes);
    let entry = ProofEntry { proof_b64 };

    let mut existing: Vec<ProofEntry> = if file_path.exists() {
        let mut content = String::new();
        fs::File::open(&file_path)?.read_to_string(&mut content)?;
        if content.trim().is_empty() { Vec::new() } else { serde_json::from_str(&content)? }
    } else {
        Vec::new()
    };

    existing.push(entry);

    let json = serde_json::to_string_pretty(&existing)?;
    let file = OpenOptions::new()
        .write(true).create(true).truncate(true).open(&file_path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(json.as_bytes())?;
    writer.flush()?;
    Ok(())
}