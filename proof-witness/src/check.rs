//! The reference check of a witness: what the circuit will check, computed
//! from the node assignments alone.
//!
//! The check never looks at the goals or texts of the nodes, only at
//!
//! * the assignment `w` and the chosen branch of every node (private),
//! * the shape of the tree: which node is the child of which (private),
//! * the digests of the trusted roots (public).
//!
//! It checks, for every node:
//!
//! 1. every opened subterm of `w` is well-formed
//!    ([`Layout::check_well_formed`]);
//! 2. the child count cell is the number of children, and the digest of child
//!    slot `k` is the goal digest in the assignment of the `k`-th child;
//! 3. the chosen branch, compressed with the powers of `challenge`, evaluates
//!    to zero on `w` (Batchman's `⟨sᵀMᵢ, (w,1)⟩ = 0`);
//!
//! and for the whole tree that the roots are proved by clauses and their goal
//! digests are exactly the trusted ones.
//!
//! Builtin arithmetic and fact-registry membership are not checked here; they
//! become circuit gadgets in step 4.

use anyhow::{Result, bail};
use ff::Field;
use halo2_proofs::pasta::Fp;
use policy_constraints::{BranchKind, Cell, CompressedBranch, ConstraintSystem, Position, Slot};
use rayon::prelude::*;

use crate::roots::TrustedRoots;
use crate::witness::ProofWitness;

/// Checks `witness` against the policy and the trusted roots.
///
/// `challenge` stands in for the verifier's random challenge; it must be
/// chosen after the witness is fixed.
pub fn check(
    system: &ConstraintSystem,
    witness: &ProofWitness,
    trusted: &TrustedRoots,
    challenge: Fp,
) -> Result<()> {
    check_shape(system, witness)?;

    let layout = system.layout();
    let goal_digest = column(system, Cell::Digest(Position::root(Slot::Parent)));
    let child_count = column(system, Cell::ChildCount);
    let compressed: Vec<CompressedBranch> = system
        .branches()
        .iter()
        .map(|branch| branch.compress(layout.width(), &powers(challenge, branch.rows.len())))
        .collect();

    witness
        .nodes
        .par_iter()
        .enumerate()
        .map(|(index, node)| -> Result<()> {
            let w = &node.assignment;
            layout
                .check_well_formed(w)
                .map_err(|error| anyhow::anyhow!("node {index}: {error}"))?;

            if w[child_count] != Fp::from(node.children.len() as u64) {
                bail!(
                    "node {index}: the child count cell does not match its {} children",
                    node.children.len()
                );
            }
            for (slot, &child) in node.children.iter().enumerate() {
                let slot_digest = column(system, Cell::Digest(Position::root(child_slot(slot))));
                if w[slot_digest] != witness.nodes[child].assignment[goal_digest] {
                    bail!("node {index}: child slot {slot} does not hold the goal of node {child}");
                }
            }

            if compressed[node.branch].evaluate(w) != Fp::ZERO {
                let branch = &system.branches()[node.branch];
                let violated: Vec<&str> = branch
                    .rows
                    .iter()
                    .filter(|row| !row.is_satisfied_by(w))
                    .map(|row| row.description.as_str())
                    .collect();
                bail!(
                    "node {index}: {} does not hold, violating {}",
                    branch.kind,
                    violated.join(", ")
                );
            }
            Ok(())
        })
        // Report the first failing node, whatever order the threads finish in.
        .collect::<Vec<Result<()>>>()
        .into_iter()
        .collect::<Result<()>>()?;

    check_roots(system, witness, trusted, goal_digest)
}

/// Branch indices are valid, and parents and children point at each other
/// with every child after its parent, so the nodes form a forest whose roots
/// are `witness.roots`.
fn check_shape(system: &ConstraintSystem, witness: &ProofWitness) -> Result<()> {
    let nodes = &witness.nodes;
    for (index, node) in nodes.iter().enumerate() {
        if node.branch >= system.branches().len() {
            bail!("node {index}: branch {} does not exist", node.branch);
        }
        if node.children.len() > system.layout().child_slots() {
            bail!(
                "node {index}: {} children, the layout has {} slots",
                node.children.len(),
                system.layout().child_slots()
            );
        }
        for &child in &node.children {
            if child <= index || child >= nodes.len() {
                bail!("node {index}: child {child} is not a later node");
            }
            if nodes[child].parent != Some(index) {
                bail!("node {index}: child {child} names another parent");
            }
        }
        if let Some(parent) = node.parent
            && !nodes
                .get(parent)
                .is_some_and(|p| p.children.contains(&index))
        {
            bail!("node {index}: parent {parent} does not list it as a child");
        }
    }
    let roots: Vec<usize> = (0..nodes.len())
        .filter(|&index| nodes[index].parent.is_none())
        .collect();
    if roots != witness.roots {
        bail!(
            "the roots are {:?}, but the nodes without a parent are {roots:?}",
            witness.roots
        );
    }
    Ok(())
}

/// Every root is proved by a clause, and the root goal digests are the
/// trusted digests, each once.
fn check_roots(
    system: &ConstraintSystem,
    witness: &ProofWitness,
    trusted: &TrustedRoots,
    goal_digest: usize,
) -> Result<()> {
    let trusted: Vec<Fp> = trusted.goals().map(|goal| goal.digest()).collect();
    let mut proved: Vec<Fp> = Vec::with_capacity(witness.roots.len());
    for &root in &witness.roots {
        let kind = &system.branches()[witness.nodes[root].branch].kind;
        if !matches!(kind, BranchKind::Clause { .. }) {
            bail!("root node {root} is proved by {kind}, not by a clause of the policy");
        }
        let digest = witness.nodes[root].assignment[goal_digest];
        if !trusted.contains(&digest) {
            bail!("root node {root} proves a goal that is not trusted");
        }
        if proved.contains(&digest) {
            bail!("root node {root} proves a trusted goal that another root proves");
        }
        proved.push(digest);
    }
    if let Some(missing) = trusted.iter().position(|digest| !proved.contains(digest)) {
        bail!("trusted root {missing} is not proved by any root node");
    }
    Ok(())
}

/// `1, c, c², …`: one weight per row of a branch.
fn powers(challenge: Fp, count: usize) -> Vec<Fp> {
    std::iter::successors(Some(Fp::ONE), |power| Some(*power * challenge))
        .take(count)
        .collect()
}

fn child_slot(slot: usize) -> Slot {
    Slot::Child(u8::try_from(slot).expect("check_shape bounds the child count"))
}

fn column(system: &ConstraintSystem, cell: Cell) -> usize {
    system
        .layout()
        .column(&cell)
        .expect("the layout always opens the goal and the child slots")
}
