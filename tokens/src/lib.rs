pub mod fact_registry;
pub mod frontend;
pub mod lowering;
pub mod parser;
pub mod trusted_roots;

pub use fact_registry::parse_fact_registry;
pub use frontend::{parse_goal, parse_policy, to_canonical_json};
pub use trusted_roots::parse_trusted_root_goals;
