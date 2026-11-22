//! Translates a policy into the linear constraints that one proof-tree node
//! must satisfy.
//!
//! A node of the proof tree claims: "this goal follows from these child goals
//! by one rule of the policy". The claim is checked on a *node assignment*
//! `w`, a vector of field elements whose cells are described by the shared
//! [`Layout`]:
//!
//! * the number of child goals;
//! * for every goal position that some branch inspects, the subterm found
//!   there: its digest, and for opened positions also its tag, payload
//!   (functor symbol or integer value) and arity.
//!
//! Every way of proving a goal is a [`Branch`]: one per clause alternative of
//! the policy, one per builtin call form, and one per external fact
//! predicate. A branch is a list of [`LinearRow`]s of the form
//! `Σ coefficient·w[column] + constant = 0`, i.e. the rows of the Batchman
//! topology matrix `Mᵢ` applied to `(w, 1)`. The rows check the goal
//! signatures, the child count, the constants and constructor shapes written
//! in the clause, and that repeated variables are bound to equal subterms.
//!
//! The layout is the same for every branch, so an assignment does not reveal
//! which branch it satisfies. Two checks are independent of the branch and are
//! therefore not rows of any branch:
//!
//! * well-formedness of every opened node: its digest is the Poseidon hash of
//!   its tag, payload, arity and child digests (see [`term::node_digest`]),
//!   or the node is absent and all its cells are zero
//!   ([`Layout::check_well_formed`]);
//! * the semantics attached to a branch kind: arithmetic for builtins and
//!   registry membership for external facts.

mod compile;
mod error;

pub mod arith;
pub mod branch;
pub mod layout;
pub mod position;
pub mod row;
pub mod term;

pub use branch::{
    ArithmeticOp, Branch, BranchKind, BuiltinCheck, Comparison, CompressedBranch, Expression,
};
pub use compile::{ConstraintSystem, compile};
pub use error::ModelError;
pub use layout::{Cell, Layout, OpenedNode, SlotLink};
/// The field every cell, coefficient and digest lives in (the Pallas base
/// field; the scalar field of the Vesta curve the circuits commit with).
pub use pasta_curves::Fp;
pub use position::{Position, Slot};
pub use row::{LinearRow, RowKind};
pub use term::{GroundTerm, MAX_ARITY, TermTag};
