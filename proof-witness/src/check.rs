//! The reference check of a witness: what the circuit will check, computed
//! from the node assignments alone.
//!
//! The check never looks at the goals or texts of the nodes, nor at the shape
//! of the tree (`parent` and `children` are the prover's bookkeeping). It sees
//!
//! * the assignment `w`, the chosen branch, and whether it is a root, for every
//!   node (private),
//! * the digests of the trusted roots and the fact commitments (public).
//!
//! It checks, for every node:
//!
//! 1. every opened subterm of `w` is well-formed, and the parent-side ones
//!    hash to their digests ([`Layout::check_well_formed`]);
//! 2. the chosen branch, compressed with the powers of `challenge`, evaluates
//!    to zero on `w` (Batchman's `⟨sᵀMᵢ, (w,1)⟩ = 0`);
//! 3. the arithmetic step slots hold 64-bit integers and every flagged
//!    operation holds ([`Layout::check_arithmetic`]); the branch's rows set
//!    the flags, so a builtin node is checked by its builtin and no other;
//! 4. if the node is an external fact (`is_fact`, set by the branch), the
//!    commitment of its goal digest and salt is a published one;
//!
//! that the body goals are matched with the nodes ([`check_links`]): the
//! present child slots of all nodes and the parent goals of all non-root nodes
//! are the same multiset of cell lists; and that the root nodes are proved by
//! clauses (`is_clause`) and their goal digests are exactly the trusted ones.
//!
//! The matching makes every non-root node the child of exactly one slot, so
//! the nodes reachable from the roots form a tree whose every body goal is
//! proved by a node of it (a cycle of nodes can balance itself, but no root
//! reaches it). The circuit compares the same lists, each compressed into one
//! field element.
//!
//! A shard of a tree ([`crate::shard`]) is checked the same way, with links to
//! other shards taking part in the matching: an incoming link stands in for a
//! node of another shard, an outgoing link for a body goal of another shard.
//!
//! [`Layout::check_well_formed`]: policy_constraints::Layout::check_well_formed
//! [`Layout::check_arithmetic`]: policy_constraints::Layout::check_arithmetic

use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};

use anyhow::{Result, bail};
use ff::{Field, PrimeField};
use policy_constraints::Fp;
use policy_constraints::term::fact_commitment;
use policy_constraints::{
    Cell, CompressedBranch, ConstraintSystem, Layout, Position, Slot, TermTag,
};
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
    commitments: &[Fp],
    challenge: Fp,
) -> Result<()> {
    check_part(system, witness, &[], &[], commitments, challenge)?;
    check_roots(system, witness, trusted)
}

/// Every check but the roots' ([`check_roots`]): the nodes one by one, then
/// the matching of body goals with goals, where `incoming` are further goals
/// proved elsewhere and `outgoing` further body goals elsewhere (goal cell
/// lists, see [`Layout::goal_columns`]).
pub(crate) fn check_part(
    system: &ConstraintSystem,
    witness: &ProofWitness,
    incoming: &[Vec<Fp>],
    outgoing: &[Vec<Fp>],
    commitments: &[Fp],
    challenge: Fp,
) -> Result<()> {
    check_indices(system, witness)?;

    let layout = system.layout();
    let goal_digest = goal_digest(layout);
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

            layout
                .check_arithmetic(w)
                .map_err(|error| anyhow::anyhow!("node {index}: {error}"))?;
            if let Some((is_fact, salt)) = layout.fact_cells()
                && w[is_fact] != Fp::ZERO
                && !commitments.contains(&fact_commitment(w[goal_digest], w[salt]))
            {
                bail!("node {index}: the fact's commitment is not a published one");
            }
            Ok(())
        })
        // Report the first failing node, whatever order the threads finish in.
        .collect::<Vec<Result<()>>>()
        .into_iter()
        .collect::<Result<()>>()?;

    for (index, goal) in incoming.iter().enumerate() {
        layout
            .check_goal(goal)
            .map_err(|error| anyhow::anyhow!("incoming link {index}: {error}"))?;
    }
    check_links(layout, witness, incoming, outgoing)
}

/// Branch indices are valid, and the roots are distinct nodes.
fn check_indices(system: &ConstraintSystem, witness: &ProofWitness) -> Result<()> {
    for (index, node) in witness.nodes.iter().enumerate() {
        if node.branch >= system.branches().len() {
            bail!("node {index}: branch {} does not exist", node.branch);
        }
    }
    for (position, &root) in witness.roots.iter().enumerate() {
        if root >= witness.nodes.len() {
            bail!("root {root} is not a node");
        }
        if witness.roots[..position].contains(&root) {
            bail!("root {root} is listed twice");
        }
    }
    Ok(())
}

/// The cells of a goal, compared as a whole: one key per cell list.
type Goal = Vec<<Fp as PrimeField>::Repr>;

fn key(goal: impl IntoIterator<Item = Fp>) -> Goal {
    goal.into_iter().map(|value| value.to_repr()).collect()
}

/// Where a goal comes from (an export) or is used (an import).
#[derive(Debug, Clone, Copy)]
enum End {
    /// The parent goal of a non-root node.
    Node(usize),
    /// A goal proved by a node of another shard.
    Incoming(usize),
    /// A body goal of a node.
    Slot(usize, usize),
    /// A body goal of a node of another shard.
    Outgoing(usize),
}

impl Display for End {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Node(index) => write!(
                formatter,
                "node {index} proves a goal that no body goal uses"
            ),
            Self::Incoming(index) => {
                write!(
                    formatter,
                    "incoming link {index} proves a goal that no body goal uses"
                )
            }
            Self::Slot(index, slot) => write!(
                formatter,
                "node {index}: no node proves the body goal in child slot {slot}"
            ),
            Self::Outgoing(index) => {
                write!(formatter, "outgoing link {index}: no node proves its goal")
            }
        }
    }
}

/// Every present child slot of every node (its tag is not absent) and every
/// outgoing link is matched with the parent goal of one non-root node or one
/// incoming link, and the other way around: the two are the same multiset of
/// cell lists ([`Layout::goal_columns`], [`Layout::slot_links`]).
///
/// The circuit exports the compressed goals, imports the compressed body
/// goals, and checks that the two lists are a shuffle of each other.
fn check_links(
    layout: &Layout,
    witness: &ProofWitness,
    incoming: &[Vec<Fp>],
    outgoing: &[Vec<Fp>],
) -> Result<()> {
    let mut exports: Vec<(End, Goal)> = (0..witness.nodes.len())
        .filter(|index| !witness.roots.contains(index))
        .map(|index| {
            (
                End::Node(index),
                key(layout.goal_of(&witness.nodes[index].assignment)),
            )
        })
        .collect();
    exports.extend(
        incoming
            .iter()
            .enumerate()
            .map(|(index, goal)| (End::Incoming(index), key(goal.iter().copied()))),
    );

    let mut imports: Vec<(End, Goal)> = Vec::new();
    for (index, node) in witness.nodes.iter().enumerate() {
        let w = &node.assignment;
        for slot in 0..layout.child_slots() {
            let root = Position::root(Slot::Child(slot as u8));
            let tag = layout
                .column(&Cell::Tag(root))
                .expect("the layout opens every child slot");
            if w[tag] == TermTag::Absent.field() {
                continue;
            }
            let goal = key(layout.slot_links(slot).iter().map(|link| w[link.in_parent]));
            imports.push((End::Slot(index, slot), goal));
        }
    }
    imports.extend(
        outgoing
            .iter()
            .enumerate()
            .map(|(index, goal)| (End::Outgoing(index), key(goal.iter().copied()))),
    );

    let mut unused: BTreeMap<&Goal, usize> = BTreeMap::new();
    for (_, goal) in &exports {
        *unused.entry(goal).or_default() += 1;
    }
    for (end, goal) in &imports {
        match unused.get_mut(goal) {
            Some(count) if *count > 0 => *count -= 1,
            _ => bail!("{end}"),
        }
    }
    // Blame the leftover exports of a goal one by one, in order.
    for (end, goal) in &exports {
        let count = unused.get_mut(goal).expect("every export is counted");
        if *count > 0 {
            bail!("{end}");
        }
    }
    Ok(())
}

/// Every root is proved by a clause, and the root goal digests are the
/// trusted digests, each once.
pub(crate) fn check_roots(
    system: &ConstraintSystem,
    witness: &ProofWitness,
    trusted: &TrustedRoots,
) -> Result<()> {
    let goal_digest = goal_digest(system.layout());
    let is_clause = system.layout().is_clause();
    let trusted: Vec<Fp> = trusted.goals().map(|goal| goal.digest()).collect();
    let mut proved: Vec<Fp> = Vec::with_capacity(witness.roots.len());
    for &root in &witness.roots {
        let w = &witness.nodes[root].assignment;
        if w[is_clause] != Fp::ONE {
            let kind = &system.branches()[witness.nodes[root].branch].kind;
            bail!("root node {root} is proved by {kind}, not by a clause of the policy");
        }
        let digest = w[goal_digest];
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

fn goal_digest(layout: &Layout) -> usize {
    layout
        .column(&Cell::Digest(Position::root(Slot::Parent)))
        .expect("the layout always opens the parent goal")
}

/// `1, c, c², …`: one weight per row of a branch.
fn powers(challenge: Fp, count: usize) -> Vec<Fp> {
    std::iter::successors(Some(Fp::ONE), |power| Some(*power * challenge))
        .take(count)
        .collect()
}
