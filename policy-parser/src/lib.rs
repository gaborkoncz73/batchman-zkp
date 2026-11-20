//! Batchman policy-Prolog frontend: parses `.pl` policies, proof-tree goals,
//! external facts and trusted roots into the semantic IR of `common`.

pub mod parser;

mod goals;
mod policy;
mod syntax;

pub use goals::{parse_fact_registry, parse_trusted_root_goals};
pub use policy::{parse_policy, to_canonical_json};
pub use syntax::parse_goal;
