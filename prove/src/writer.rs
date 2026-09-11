use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use common::commitments::{LocalStepCommitments, ProofCommitmentsV3};
use common::proof_tree::ProofTreeManifestV1;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ProofEntry {
    proof_b64: String,
    commitments: ProofCommitmentsV3,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProofTreeArtifactV1 {
    manifest: ProofTreeManifestV1,
    proofs: Vec<ProofEntry>,
}

/// Writes the complete, DFS-indexed proof set in one atomic replacement.
/// Callers collect indexed parallel results before invoking this function, so
/// scheduling cannot reorder entries and a failed proof cannot leave a partial
/// output file behind.
pub fn write_proofs(
    name: &str,
    manifest: &ProofTreeManifestV1,
    proofs: &[(Vec<u8>, LocalStepCommitments)],
) -> Result<()> {
    anyhow::ensure!(
        manifest.nodes.len() == proofs.len(),
        "manifest node count {} does not match proof count {}",
        manifest.nodes.len(),
        proofs.len()
    );
    let entries = proofs
        .iter()
        .map(|(proof, commitments)| ProofEntry {
            proof_b64: general_purpose::STANDARD.encode(proof),
            commitments: ProofCommitmentsV3::from_fields(*commitments),
        })
        .collect::<Vec<_>>();
    let artifact = ProofTreeArtifactV1 {
        manifest: manifest.clone(),
        proofs: entries,
    };
    let mut json = serde_json::to_string_pretty(&artifact)?;
    json.push('\n');

    let output_directory = Path::new("output");
    fs::create_dir_all(output_directory)?;
    let destination = output_directory.join(format!("{name}_proofs.json"));
    let temporary = output_directory.join(format!(".{name}_proofs.json.tmp"));
    let file = File::create(&temporary)
        .with_context(|| format!("failed to create {}", temporary.display()))?;
    let mut writer = BufWriter::new(file);
    writer.write_all(json.as_bytes())?;
    writer.flush()?;
    writer.get_ref().sync_all()?;
    drop(writer);
    fs::rename(&temporary, &destination).with_context(|| {
        format!(
            "failed to atomically replace {} with {}",
            destination.display(),
            temporary.display()
        )
    })?;
    Ok(())
}
