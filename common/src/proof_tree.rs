//! Verifier-side composition of independently proven local resolution steps.
//!
//! The manifest contains only topology.  Its edges are authenticated by the
//! ordered-child commitment already exposed by every local Halo2 proof, while
//! its roots are anchored in a separate verifier-trusted statement.

use std::collections::BTreeSet;

use anyhow::{Result, bail, ensure};
use halo2_proofs::pasta::Fp;
use serde::{Deserialize, Serialize};

use crate::{
    circuit_dialect::CircuitLimits,
    commitments::{LocalStepCommitments, ordered_children_commitment_from_hashes},
};

pub const PROOF_TREE_SCHEMA_VERSION: u32 = 1;
pub const PROOF_TREE_DIALECT: &str = "batchman_compositional_proof_tree_v1";
pub const TRUSTED_ROOTS_SCHEMA_VERSION: u32 = 1;
pub const TRUSTED_ROOTS_DIALECT: &str = "batchman_trusted_roots_v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofTreeNodeV1 {
    pub child_indices: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofTreeManifestV1 {
    pub schema_version: u32,
    pub dialect: String,
    pub root_indices: Vec<usize>,
    pub nodes: Vec<ProofTreeNodeV1>,
}

impl ProofTreeManifestV1 {
    pub fn new(root_indices: Vec<usize>, nodes: Vec<ProofTreeNodeV1>) -> Self {
        Self {
            schema_version: PROOF_TREE_SCHEMA_VERSION,
            dialect: PROOF_TREE_DIALECT.to_owned(),
            root_indices,
            nodes,
        }
    }

    fn validate_envelope(&self) -> Result<()> {
        ensure!(
            self.schema_version == PROOF_TREE_SCHEMA_VERSION,
            "unsupported proof-tree schema version {}; expected {PROOF_TREE_SCHEMA_VERSION}",
            self.schema_version
        );
        ensure!(
            self.dialect == PROOF_TREE_DIALECT,
            "unsupported proof-tree dialect {:?}; expected {PROOF_TREE_DIALECT:?}",
            self.dialect
        );
        Ok(())
    }
}

/// Human-readable roots supplied by the verifier, independently from the
/// prover-produced proof artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedRootsV1 {
    pub schema_version: u32,
    pub dialect: String,
    pub root_goals: Vec<String>,
}

impl TrustedRootsV1 {
    pub fn validate_shape(&self, limits: &CircuitLimits) -> Result<()> {
        ensure!(
            self.schema_version == TRUSTED_ROOTS_SCHEMA_VERSION,
            "unsupported trusted-roots schema version {}; expected {TRUSTED_ROOTS_SCHEMA_VERSION}",
            self.schema_version
        );
        ensure!(
            self.dialect == TRUSTED_ROOTS_DIALECT,
            "unsupported trusted-roots dialect {:?}; expected {TRUSTED_ROOTS_DIALECT:?}",
            self.dialect
        );
        ensure!(!self.root_goals.is_empty(), "trusted root list is empty");
        ensure!(
            self.root_goals.len() <= limits.max_proof_roots,
            "{} trusted roots exceed configured maximum {}",
            self.root_goals.len(),
            limits.max_proof_roots
        );
        for (index, goal) in self.root_goals.iter().enumerate() {
            ensure!(!goal.is_empty(), "trusted root {index} is empty");
            ensure!(
                goal.len() <= limits.max_goal_bytes,
                "trusted root {index} has {} bytes, maximum is {}",
                goal.len(),
                limits.max_goal_bytes
            );
        }
        Ok(())
    }
}

/// Checks that the independently verified local statements form exactly one
/// bounded, ordered forest rooted in the verifier's trusted goals.
pub fn validate_compositional_proof_tree(
    manifest: &ProofTreeManifestV1,
    commitments: &[LocalStepCommitments],
    trusted_root_commitments: &[Fp],
    expected_policy_commitment: Fp,
    limits: &CircuitLimits,
) -> Result<()> {
    limits.validate().map_err(anyhow::Error::new)?;
    manifest.validate_envelope()?;

    let node_count = manifest.nodes.len();
    ensure!(node_count > 0, "proof-tree manifest contains no nodes");
    ensure!(
        node_count <= limits.max_proof_nodes,
        "proof-tree manifest has {node_count} nodes, maximum is {}",
        limits.max_proof_nodes
    );
    ensure!(
        commitments.len() == node_count,
        "proof count {} does not match manifest node count {node_count}",
        commitments.len()
    );
    ensure!(
        manifest.root_indices.len() == trusted_root_commitments.len(),
        "manifest root count {} does not match trusted root count {}",
        manifest.root_indices.len(),
        trusted_root_commitments.len()
    );
    ensure!(
        !manifest.root_indices.is_empty(),
        "proof-tree manifest has no roots"
    );
    ensure!(
        manifest.root_indices.len() <= limits.max_proof_roots,
        "manifest root count {} exceeds configured maximum {}",
        manifest.root_indices.len(),
        limits.max_proof_roots
    );

    for (index, commitment) in commitments.iter().enumerate() {
        ensure!(
            commitment.policy == expected_policy_commitment,
            "proof node {index} belongs to a different policy"
        );
        ensure!(
            matches!(commitment.resolution_tag, tag if tag == Fp::from(1) || tag == Fp::from(2) || tag == Fp::from(3)),
            "proof node {index} has invalid resolution tag"
        );
    }

    let roots = manifest
        .root_indices
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    ensure!(
        roots.len() == manifest.root_indices.len(),
        "proof-tree manifest contains a duplicate root index"
    );
    let mut indegree = vec![0_usize; node_count];

    for (node_index, node) in manifest.nodes.iter().enumerate() {
        ensure!(
            node.child_indices.len() <= limits.max_children_per_node,
            "proof node {node_index} has {} children, maximum is {}",
            node.child_indices.len(),
            limits.max_children_per_node
        );
        if commitments[node_index].resolution_tag != Fp::from(1) {
            ensure!(
                node.child_indices.is_empty(),
                "external-fact or builtin proof node {node_index} must be a leaf"
            );
        }

        let mut local_children = BTreeSet::new();
        let mut child_hashes = Vec::with_capacity(node.child_indices.len());
        for &child_index in &node.child_indices {
            ensure!(
                child_index < node_count,
                "proof node {node_index} references out-of-range child index {child_index}"
            );
            ensure!(
                local_children.insert(child_index),
                "proof node {node_index} references child {child_index} more than once"
            );
            indegree[child_index] = indegree[child_index]
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("proof node {child_index} indegree overflow"))?;
            child_hashes.push(commitments[child_index].parent_goal);
        }
        let reconstructed = ordered_children_commitment_from_hashes(&child_hashes)?;
        ensure!(
            reconstructed == commitments[node_index].ordered_children,
            "proof node {node_index} ordered-child commitment does not match its manifest edges"
        );
    }

    for (root_position, (&root_index, trusted_commitment)) in manifest
        .root_indices
        .iter()
        .zip(trusted_root_commitments)
        .enumerate()
    {
        ensure!(
            root_index < node_count,
            "trusted root position {root_position} references out-of-range node {root_index}"
        );
        ensure!(
            commitments[root_index].parent_goal == *trusted_commitment,
            "proof root {root_position} does not match the verifier-trusted goal"
        );
    }

    for (node_index, degree) in indegree.iter().copied().enumerate() {
        let expected = usize::from(!roots.contains(&node_index));
        ensure!(
            degree == expected,
            "proof node {node_index} has indegree {degree}, expected {expected}"
        );
    }

    // Require the canonical preorder used by the prover. Besides deterministic
    // artifacts, this visits every node once and rejects cycles/orphans.
    let mut traversal = Vec::with_capacity(node_count);
    let mut seen = vec![false; node_count];
    let mut stack = manifest
        .root_indices
        .iter()
        .rev()
        .map(|&index| (index, 1_usize))
        .collect::<Vec<_>>();
    while let Some((node_index, depth)) = stack.pop() {
        if seen[node_index] {
            bail!("proof-tree traversal reaches node {node_index} more than once");
        }
        ensure!(
            depth <= limits.max_proof_depth,
            "proof-tree depth {depth} exceeds configured maximum {}",
            limits.max_proof_depth
        );
        seen[node_index] = true;
        traversal.push(node_index);
        stack.extend(
            manifest.nodes[node_index]
                .child_indices
                .iter()
                .rev()
                .map(|&child| (child, depth + 1)),
        );
    }
    ensure!(
        seen.iter().all(|visited| *visited),
        "proof-tree manifest contains a node unreachable from the trusted roots"
    );
    ensure!(
        traversal.iter().copied().eq(0..node_count),
        "proof entries are not in canonical depth-first preorder"
    );
    Ok(())
}
