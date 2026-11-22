//! Halo2 circuit for the nodes of a proof tree.
//!
//! The checks mirror `proof_witness::check`, node by node:
//!
//! * step 3a ([`well_formed`]): every opened subterm of a node assignment is
//!   well-formed, and the parent-side ones hash to their digests;
//! * step 3b ([`branch`]): the node satisfies one branch, chosen in secret,
//!   checked with two challenges (`c` compresses a branch to a vector, `γ`
//!   the vector to a number looked up among the branches);
//! * step 4a ([`arith`]): the arithmetic step slots — the flagged operation
//!   holds and every value is a 64-bit integer — on every node alike;
//! * step 4b ([`fact`]): a fact node's salted commitment is in the public
//!   list of the registry, again checked on every node alike;
//! * [`NodeCircuit`]: all of these, on one `w` per node;
//! * step 5a ([`link`]): the body goals of the nodes are matched with the
//!   nodes that prove them, as a shuffle of compressed goals, so the nodes
//!   form a tree;
//! * step 5b ([`boundary`]): links to other shard proofs, published as
//!   salted commitments of the shared goals;
//! * [`TreeCircuit`]: a whole proof tree, or one shard of it, in one proof,
//!   with its roots the trusted goals;
//! * [`pipeline`]: keys, proving and verifying with the real prover, and the
//!   proof file format (the `prove` and `verify` binaries).
//!
//! The circuits are generic over the field `F` and the Poseidon [`Spec`]; the
//! policy instantiates them with `policy_constraints::Fp` and `P128Pow5T3`.
//!
//! [`Spec`]: halo2_poseidon::poseidon::primitives::Spec

pub mod arith;
pub mod boundary;
pub mod branch;
pub mod circuit;
pub mod fact;
pub mod link;
pub mod pipeline;
pub mod rows;
pub mod shape;
pub mod well_formed;

pub use arith::{ArithChip, ArithConfig};
pub use boundary::{BoundaryChip, BoundaryConfig, Gate};
pub use branch::{BranchChip, BranchConfig, BranchTable, Claim, Entry, NodeBranch};
pub use circuit::{
    BranchCircuit, NodeCircuit, NodeConfig, TreeCircuit, TreeConfig, TreeNode, WellFormedCircuit,
    link_commitments, root_digests, shard_public,
};
pub use fact::{FactChip, FactConfig};
pub use link::{LinkChip, LinkConfig, Role, Side};
pub use pipeline::{
    ShardKey, ShardProof, Sizes, Verified, VerifierKeys, prove_shard, verify_proofs,
    verify_proofs_with, verifying_key,
};
pub use rows::{minimum_k, rows_used};
pub use shape::Shape;
pub use well_formed::{Shaped, WellFormedChip, WellFormedConfig};
