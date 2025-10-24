use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Read, Write},
    path::Path, sync::Mutex,
};
use num_bigint::BigUint;
use once_cell::sync::Lazy;
use halo2_proofs::pasta::Fp;
use serde::{Serialize, Deserialize};
use base64::{engine::general_purpose, Engine as _};
use halo2curves::ff::PrimeField;
#[derive(Serialize, Deserialize)]
struct ProofEntry {
    proof_b64: String,
    inputs_b64: Vec<Vec<String>>,
}

pub fn init_output_dir() -> anyhow::Result<()> {
    let out_dir = Path::new("out");
    if out_dir.exists() {
        fs::remove_dir_all(out_dir)?;
    }
    fs::create_dir_all(out_dir)?;
    Ok(())
}

static FILE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));


pub fn write_proof(name: &str, proof_bytes: &[u8], instances: &[&[&[Fp]]]) -> anyhow::Result<()> {
    // 🔒 Mutex lock – csak egy szál írhat egyszerre
    let _guard = FILE_LOCK.lock().unwrap();

    let out_dir = Path::new("out");
    fs::create_dir_all(out_dir)?;
    let file_path = out_dir.join(format!("{}_proofs.json", name));

    let inputs_b64: Vec<Vec<String>> = instances
        .iter()
        .flat_map(|circuits| circuits.iter()) // iterate per circuit
        .map(|cols| {
        cols.iter()
            .map(|fp| {
                let bytes = fp.to_repr(); // 32-byte representation
                let int = BigUint::from_bytes_le(bytes.as_ref());
                int.to_str_radix(10) // decimal string
            })
            .collect::<Vec<String>>()
    })
    .collect();

    let proof_b64 = general_purpose::STANDARD.encode(proof_bytes);
    let entry = ProofEntry { proof_b64, inputs_b64 };

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
