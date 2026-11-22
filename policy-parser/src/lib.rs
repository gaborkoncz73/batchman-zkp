//! Batchman policy-Prolog frontend: parses `.pl` policies and ground goals into
//! the semantic IR of `policy-ir`.

pub mod parser;

mod policy;
mod syntax;

pub use policy::{parse_policy, to_canonical_json};
pub use syntax::parse_goal;
