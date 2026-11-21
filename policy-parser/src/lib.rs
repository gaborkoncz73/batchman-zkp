//! Batchman policy-Prolog frontend: parses `.pl` policies and ground goals into
//! the semantic IR of `policy-ir`.
//!
//! The `legacy` feature adds the fact-registry and trusted-root parsing of the
//! legacy pipeline, which produces the types of the `common` crate.

pub mod parser;

#[cfg(feature = "legacy")]
mod goals;
mod policy;
mod syntax;

#[cfg(feature = "legacy")]
pub use goals::{parse_fact_registry, parse_trusted_root_goals};
pub use policy::{parse_policy, to_canonical_json};
pub use syntax::parse_goal;
