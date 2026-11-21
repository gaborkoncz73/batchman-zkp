//! Parsing and semantic validation for verifier-trusted root goals.

use anyhow::{Result, bail};
use common::{
    circuit_dialect::{CircuitLimits, CircuitPolicyValidation},
    proof_tree::TrustedRootsV1,
    witness_ir::GroundCall,
};
use policy_ir::PolicyFile;

use super::parse_ground_predicate;

pub fn parse_trusted_root_goals(
    statement: &TrustedRootsV1,
    policy: &PolicyFile,
    limits: &CircuitLimits,
    source_name: &str,
) -> Result<Vec<GroundCall>> {
    statement.validate_shape(limits)?;
    let analysis = policy
        .validate_for_circuit(limits)
        .map_err(anyhow::Error::new)?;

    statement
        .root_goals
        .iter()
        .enumerate()
        .map(|(index, source)| {
            let label = format!("{source_name}.root_goals[{index}]");
            let goal = parse_ground_predicate(source, &label, limits, "a trusted root")?;
            if !analysis.defined_predicates.contains(&goal.predicate) {
                bail!(
                    "{label}: predicate {}/{} is not defined by the policy",
                    goal.predicate.name,
                    goal.predicate.arity
                );
            }
            Ok(goal)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use common::{
        circuit_dialect::CircuitLimits, policy_ir::PolicyFile, proof_tree::TrustedRootsV1,
    };

    use super::parse_trusted_root_goals;

    fn inputs() -> (PolicyFile, CircuitLimits) {
        (
            serde_json::from_str(include_str!(
                "../../../tests/fixtures/current-policy.rules.json"
            ))
            .unwrap(),
            serde_json::from_str(include_str!("../../../config/circuit-limits.json")).unwrap(),
        )
    }

    fn statement(goal: &str) -> TrustedRootsV1 {
        TrustedRootsV1 {
            schema_version: 1,
            dialect: "batchman_trusted_roots_v1".to_owned(),
            root_goals: vec![goal.to_owned()],
        }
    }

    #[test]
    fn accepts_the_current_root() {
        let (policy, limits) = inputs();
        let roots = parse_trusted_root_goals(
            &statement("endPrice(931220)"),
            &policy,
            &limits,
            "trusted_roots",
        )
        .unwrap();
        assert_eq!(roots[0].predicate.name, "endPrice");
    }

    #[test]
    fn rejects_builtin_and_undefined_roots() {
        let (policy, limits) = inputs();
        assert!(
            parse_trusted_root_goals(&statement("1=1"), &policy, &limits, "trusted_roots").is_err()
        );
        assert!(
            parse_trusted_root_goals(
                &statement("currentConsumption(1400)"),
                &policy,
                &limits,
                "trusted_roots"
            )
            .is_err()
        );
    }
}
