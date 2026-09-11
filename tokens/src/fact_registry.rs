//! Parser-derived external fact registry normalization.

use anyhow::{Context, Result, bail};
use common::{
    circuit_dialect::{CircuitGoalKind, CircuitLimits, validate_ground_goal_for_circuit},
    data::FactEntry,
    typed_fact::{TypedFact, parse_typed_fact_salt},
    witness_ir::GroundCall,
};

use crate::frontend::parse_goal;

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
        let term = parse_goal(&entry.goal, &format!("{path}.goal"))
            .with_context(|| format!("failed to parse {path}.goal"))?;
        let indicator = match validate_ground_goal_for_circuit(&term, limits)
            .map_err(|error| anyhow::anyhow!("{path}.goal.{}: {}", error.path, error.message))?
        {
            CircuitGoalKind::Predicate(indicator) => indicator,
            CircuitGoalKind::Builtin(builtin) => {
                bail!(
                    "{path}.goal: builtin {}/{} cannot be an external fact",
                    builtin.functor(),
                    builtin.arity()
                )
            }
        };
        let goal = GroundCall::from_goal_term(&term)
            .with_context(|| format!("failed to normalize {path}.goal"))?;
        if goal.predicate != indicator {
            bail!("{path}.goal: parsed predicate indicator is inconsistent");
        }
        if let Some(previous) = parsed.iter().position(|fact| fact.goal == goal) {
            bail!(
                "{path}.goal duplicates the canonical ground fact at {source_name}[{previous}].goal"
            );
        }
        let salt = parse_typed_fact_salt(&entry.salt)
            .map_err(|message| anyhow::anyhow!("{path}.salt: {message}"))?;
        parsed.push(TypedFact { goal, salt });
    }
    Ok(parsed)
}
