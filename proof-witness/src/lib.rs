//! Turns a proof tree into what the prover needs for every node.
//!
//! The SWI meta-interpreter writes the proof tree (`input/proof_tree.json`):
//! every node is a goal and the goals it was derived from. For every node this
//! crate computes
//!
//! * the node assignment `w` (see [`policy_constraints::Layout::assign`]), and
//! * the branch that proves the node.
//!
//! A branch proves a node when the node assignment satisfies its rows and,
//! for builtins, the arithmetic holds, and for external facts, the goal is in
//! the fact registry. The roots of the tree must be exactly the goals the
//! verifier trusts, and each root must be proved by a clause of the policy.
//!
//! Nodes are independent of each other, so they are processed in parallel.
//! The nodes are numbered in pre-order: a parent comes before its children,
//! and every subtree is a contiguous range of node indices.
//!
//! [`check`] is the reference for the circuit: it checks a witness from the
//! node assignments alone, without the goals and without the shape of the
//! tree (body goals are matched with the nodes that prove them as a multiset).
//! [`plan`] splits a witness into shards proved independently, linked by
//! salted commitments ([`shard`]).

mod check;
mod facts;
mod goal;
mod roots;
pub mod shard;
mod tree;
mod witness;

pub use check::check;
pub use facts::{
    Fact, FactRegistry, commitments_to_json, decimal, field_element, parse_commitments,
};
pub use roots::TrustedRoots;
pub use shard::{Link, Shard, ShardPublic, check_links_between, check_shard, check_sharded, plan};
pub use tree::{ProofTree, TreeNode};
pub use witness::{NodeWitness, ProofWitness, build};
