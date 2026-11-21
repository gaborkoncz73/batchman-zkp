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
//! node assignments alone, without the goals.

mod check;
mod facts;
mod goal;
mod roots;
mod tree;
mod witness;

pub use check::check;
pub use facts::{Fact, FactRegistry};
pub use roots::TrustedRoots;
pub use tree::{ProofTree, TreeNode};
pub use witness::{NodeWitness, ProofWitness, build};
