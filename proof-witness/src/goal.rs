//! Ground goals written as Prolog text.

use anyhow::{Result, bail};
use policy_constraints::{GroundTerm, TermTag};
use policy_parser::parse_goal;

/// Parses a ground, callable goal such as `support_matrix(low,low,nominal,500)`.
pub(crate) fn ground_goal(text: &str, label: &str) -> Result<GroundTerm> {
    let term = parse_goal(text, label)?;
    let goal = GroundTerm::from_term(&term).map_err(|error| anyhow::anyhow!("{label}: {error}"))?;
    if !matches!(goal.tag(), TermTag::Atom | TermTag::Compound) {
        bail!("{label}: {text} is not a callable goal");
    }
    Ok(goal)
}
