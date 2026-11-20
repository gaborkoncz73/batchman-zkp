//! Parser-derived external fact registry normalization.

use anyhow::{Result, anyhow, bail};
use common::{
    circuit_dialect::CircuitLimits,
    data::FactEntry,
    typed_fact::{TypedFact, parse_typed_fact_salt},
};

use super::parse_ground_predicate;

/// Parses and validates every external fact exactly once.
///
/// Distinct facts may share a predicate indicator, but the same canonical
/// ground call may occur only once, independently of its salt.
pub fn parse_fact_registry(
    entries: &[FactEntry],
    limits: &CircuitLimits,
    source_name: &str,
) -> Result<Vec<TypedFact>> {
    limits.validate()?;
    if entries.len() > limits.max_external_facts {
        bail!(
            "{source_name}: {} facts exceed configured maximum {}",
            entries.len(),
            limits.max_external_facts
        );
    }

    let mut parsed: Vec<TypedFact> = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let path = format!("{source_name}[{index}]");
        if entry.goal.len() > limits.max_goal_bytes {
            bail!(
                "{path}.goal: {} bytes exceed configured maximum {}",
                entry.goal.len(),
                limits.max_goal_bytes
            );
        }
        let goal = parse_ground_predicate(
            &entry.goal,
            &format!("{path}.goal"),
            limits,
            "an external fact",
        )?;
        if let Some(previous) = parsed.iter().position(|fact| fact.goal == goal) {
            bail!(
                "{path}.goal duplicates the canonical ground fact at {source_name}[{previous}].goal"
            );
        }
        let salt = parse_typed_fact_salt(&entry.salt)
            .map_err(|message| anyhow!("{path}.salt: {message}"))?;
        parsed.push(TypedFact { goal, salt });
    }
    Ok(parsed)
}
