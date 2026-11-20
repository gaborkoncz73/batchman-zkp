//! Ground goals for the circuit: external facts and verifier-trusted roots.

mod facts;
mod trusted_roots;

use anyhow::{Context, Result, anyhow, bail};
use common::circuit_dialect::{CircuitGoalKind, CircuitLimits, validate_ground_goal_for_circuit};
use common::witness_ir::GroundCall;

use crate::syntax::parse_goal;

pub use facts::parse_fact_registry;
pub use trusted_roots::parse_trusted_root_goals;

/// Parses one ground policy-predicate goal (an external fact or a trusted
/// root) and normalizes it for the circuit. `role` names the goal in the error
/// raised for builtins, e.g. "an external fact".
fn parse_ground_predicate(
    source: &str,
    label: &str,
    limits: &CircuitLimits,
    role: &str,
) -> Result<GroundCall> {
    let term = parse_goal(source, label).with_context(|| format!("failed to parse {label}"))?;
    let indicator = match validate_ground_goal_for_circuit(&term, limits)
        .map_err(|error| anyhow!("{label}.{}: {}", error.path, error.message))?
    {
        CircuitGoalKind::Predicate(indicator) => indicator,
        CircuitGoalKind::Builtin(builtin) => bail!(
            "{label}: builtin {}/{} cannot be {role}",
            builtin.functor(),
            builtin.arity()
        ),
    };
    let goal = GroundCall::from_goal_term(&term)
        .with_context(|| format!("failed to normalize {label}"))?;
    if goal.predicate != indicator {
        bail!("{label}: parsed predicate indicator is inconsistent");
    }
    Ok(goal)
}
