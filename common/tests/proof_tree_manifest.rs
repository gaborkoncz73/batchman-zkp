use common::{
    circuit_dialect::CircuitLimits,
    commitments::{LocalStepCommitments, ordered_children_commitment_from_hashes},
    proof_tree::{ProofTreeManifestV1, ProofTreeNodeV1, validate_compositional_proof_tree},
};
use halo2_proofs::pasta::Fp;

fn limits() -> CircuitLimits {
    serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap()
}

fn honest_tree() -> (ProofTreeManifestV1, Vec<LocalStepCommitments>, Vec<Fp>, Fp) {
    let policy = Fp::from(99);
    let root = Fp::from(10);
    let left = Fp::from(11);
    let right = Fp::from(12);
    let empty = ordered_children_commitment_from_hashes(&[]).unwrap();
    let commitments = vec![
        LocalStepCommitments {
            policy,
            parent_goal: root,
            ordered_children: ordered_children_commitment_from_hashes(&[left, right]).unwrap(),
            resolution_tag: Fp::from(1),
        },
        LocalStepCommitments {
            policy,
            parent_goal: left,
            ordered_children: empty,
            resolution_tag: Fp::from(2),
        },
        LocalStepCommitments {
            policy,
            parent_goal: right,
            ordered_children: empty,
            resolution_tag: Fp::from(3),
        },
    ];
    let manifest = ProofTreeManifestV1::new(
        vec![0],
        vec![
            ProofTreeNodeV1 {
                child_indices: vec![1, 2],
            },
            ProofTreeNodeV1 {
                child_indices: vec![],
            },
            ProofTreeNodeV1 {
                child_indices: vec![],
            },
        ],
    );
    (manifest, commitments, vec![root], policy)
}

fn validate(
    manifest: &ProofTreeManifestV1,
    commitments: &[LocalStepCommitments],
    roots: &[Fp],
    policy: Fp,
) -> anyhow::Result<()> {
    validate_compositional_proof_tree(manifest, commitments, roots, policy, &limits())
}

#[test]
fn accepts_an_exact_ordered_tree_anchored_at_the_trusted_root() {
    let (manifest, commitments, roots, policy) = honest_tree();
    validate(&manifest, &commitments, &roots, policy).unwrap();
}

#[test]
fn rejects_wrong_root_policy_and_proof_count() {
    let (manifest, commitments, roots, policy) = honest_tree();
    assert!(validate(&manifest, &commitments, &[Fp::from(404)], policy).is_err());
    assert!(validate(&manifest, &commitments, &roots, Fp::from(100)).is_err());
    assert!(
        validate(
            &manifest,
            &commitments[..commitments.len() - 1],
            &roots,
            policy
        )
        .is_err()
    );
}

#[test]
fn rejects_reordered_or_forged_edges() {
    let (manifest, commitments, roots, policy) = honest_tree();

    let mut reordered = manifest.clone();
    reordered.nodes[0].child_indices.swap(0, 1);
    assert!(validate(&reordered, &commitments, &roots, policy).is_err());

    let mut duplicate = manifest.clone();
    duplicate.nodes[0].child_indices = vec![1, 1];
    assert!(validate(&duplicate, &commitments, &roots, policy).is_err());

    let mut out_of_range = manifest.clone();
    out_of_range.nodes[0].child_indices[1] = 3;
    assert!(validate(&out_of_range, &commitments, &roots, policy).is_err());
}

#[test]
fn rejects_orphans_cycles_and_noncanonical_entry_order() {
    let (manifest, commitments, roots, policy) = honest_tree();

    let mut orphan = manifest.clone();
    orphan.nodes[0].child_indices = vec![1];
    let mut orphan_commitments = commitments.clone();
    orphan_commitments[0].ordered_children =
        ordered_children_commitment_from_hashes(&[commitments[1].parent_goal]).unwrap();
    assert!(validate(&orphan, &orphan_commitments, &roots, policy).is_err());

    let mut cycle = manifest.clone();
    cycle.nodes[1].child_indices = vec![0];
    let mut cycle_commitments = commitments.clone();
    cycle_commitments[1].resolution_tag = Fp::from(1);
    cycle_commitments[1].ordered_children =
        ordered_children_commitment_from_hashes(&[commitments[0].parent_goal]).unwrap();
    assert!(validate(&cycle, &cycle_commitments, &roots, policy).is_err());

    let mut noncanonical = manifest.clone();
    noncanonical.nodes[0].child_indices = vec![2, 1];
    let mut noncanonical_commitments = commitments.clone();
    noncanonical_commitments[0].ordered_children = ordered_children_commitment_from_hashes(&[
        commitments[2].parent_goal,
        commitments[1].parent_goal,
    ])
    .unwrap();
    assert!(validate(&noncanonical, &noncanonical_commitments, &roots, policy).is_err());
}

#[test]
fn rejects_children_on_external_fact_or_builtin_nodes() {
    let (mut manifest, mut commitments, roots, policy) = honest_tree();
    manifest.nodes[1].child_indices = vec![2];
    commitments[1].ordered_children =
        ordered_children_commitment_from_hashes(&[commitments[2].parent_goal]).unwrap();
    commitments[0].ordered_children =
        ordered_children_commitment_from_hashes(&[commitments[1].parent_goal]).unwrap();
    manifest.nodes[0].child_indices = vec![1];
    assert!(validate(&manifest, &commitments, &roots, policy).is_err());
}
