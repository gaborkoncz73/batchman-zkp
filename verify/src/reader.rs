use std::{fs, path::Path};

use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use common::commitments::{LocalStepCommitments, ProofCommitmentsV3};
use common::proof_tree::ProofTreeManifestV1;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

pub struct BoundProof {
    pub proof_bytes: Vec<u8>,
    pub commitments: LocalStepCommitments,
}

pub struct BoundProofTree {
    pub manifest: ProofTreeManifestV1,
    pub proofs: Vec<BoundProof>,
}

pub fn read_proofs(name: &str) -> Result<BoundProofTree> {
    let file_path = Path::new("output").join(format!("{name}_proofs.json"));
    if !file_path.exists() {
        anyhow::bail!("file not found: {}", file_path.display());
    }

    let content = fs::read_to_string(&file_path)?;
    let artifact: ProofTreeArtifactV1 = serde_json::from_str(&content).with_context(|| {
        format!(
            "invalid compositional proof-tree artifact {}: regenerate legacy or independent-proof artifacts that do not contain a manifest",
            file_path.display()
        )
    })?;
    let proofs = artifact
        .proofs
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            let proof_bytes = general_purpose::STANDARD
                .decode(&entry.proof_b64)
                .with_context(|| format!("proof entry {index} is not valid base64"))?;
            let commitments = entry
                .commitments
                .to_fields()
                .with_context(|| format!("proof entry {index} has invalid commitments"))?;
            Ok(BoundProof {
                proof_bytes,
                commitments,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(BoundProofTree {
        manifest: artifact.manifest,
        proofs,
    })
}
