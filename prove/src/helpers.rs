use std::collections::BTreeSet;

use anyhow::{Context, Result, bail};
use common::{
    circuit_dialect::{
        CircuitGoalKind, CircuitLimits, CircuitPolicyValidation, CircuitValidationError,
        validate_ground_goal_for_circuit,
    },
    data::ProofNode,
    policy_ir::{PolicyFile, PredicateIndicator},
    typed_fact::TypedFact,
    witness_ir::{GoalResolution, GroundCall, LocalStepWitness, ResolvedGoal, StepTransition},
};
use policy_parser::parse_goal;

#[derive(Debug, Clone)]
pub(crate) struct ValidatedProofTree {
    pub(crate) roots: Vec<ValidatedProofNode>,
}

#[derive(Debug, Clone)]
pub(crate) enum ValidatedProofNode {
    Goal(ValidatedGoal),
    True,
}

#[derive(Debug, Clone)]
pub(crate) struct ValidatedGoal {
    pub(crate) ground_call: GroundCall,
    pub(crate) resolution: GoalResolution,
    pub(crate) subtree: Vec<ValidatedProofNode>,
}

pub(crate) fn validate_proof_tree(
    raw_roots: &[ProofNode],
    policy: &PolicyFile,
    facts: &[TypedFact],
    limits: &CircuitLimits,
) -> Result<ValidatedProofTree> {
    limits.validate()?;
    let analysis = policy.validate_for_circuit(limits)?;
    if raw_roots.is_empty() {
        bail!("proof_tree: at least one root goal is required");
    }
    if raw_roots.len() > limits.max_proof_roots {
        bail!(
            "proof_tree: {} roots exceed configured maximum {}",
            raw_roots.len(),
            limits.max_proof_roots
        );
    }
    if facts.len() > limits.max_external_facts {
        bail!(
            "facts: {} entries exceed configured maximum {}",
            facts.len(),
            limits.max_external_facts
        );
    }

    let fact_predicates: BTreeSet<_> = facts
        .iter()
        .map(|fact| fact.goal.predicate.clone())
        .collect();
    for required in &analysis.required_external_predicates {
        if !fact_predicates.contains(required) {
            bail!(
                "facts: policy requires external predicate {}/{}, but the fact registry does not provide it",
                required.name,
                required.arity
            );
        }
    }

    let context = ProofValidationContext {
        policy_predicates: analysis.defined_predicates,
        external_predicates: analysis.required_external_predicates,
        fact_predicates,
        limits,
    };
    let mut node_count = 0;
    let mut roots = Vec::with_capacity(raw_roots.len());
    for (index, root) in raw_roots.iter().enumerate() {
        let path = format!("proof_tree[{index}]");
        let ProofNode::GoalNode(_) = root else {
            bail!("{path}: a proof-tree root must be a goal");
        };
        let validated = validate_proof_node(root, &context, &path, 1, true, &mut node_count)?;
        roots.push(validated);
    }
    Ok(ValidatedProofTree { roots })
}

struct ProofValidationContext<'a> {
    policy_predicates: BTreeSet<PredicateIndicator>,
    external_predicates: BTreeSet<PredicateIndicator>,
    fact_predicates: BTreeSet<PredicateIndicator>,
    limits: &'a CircuitLimits,
}

fn validate_proof_node(
    raw: &ProofNode,
    context: &ProofValidationContext<'_>,
    path: &str,
    depth: usize,
    is_root: bool,
    node_count: &mut usize,
) -> Result<ValidatedProofNode> {
    if depth > context.limits.max_proof_depth {
        bail!(
            "{path}: proof depth {depth} exceeds configured maximum {}",
            context.limits.max_proof_depth
        );
    }

    match raw {
        ProofNode::True(true) => Ok(ValidatedProofNode::True),
        ProofNode::True(false) => bail!("{path}: false is not a valid proof marker"),
        ProofNode::GoalNode(goal) => {
            *node_count += 1;
            if *node_count > context.limits.max_proof_nodes {
                bail!(
                    "{path}: proof tree exceeds configured maximum of {} goal nodes",
                    context.limits.max_proof_nodes
                );
            }
            if goal.goal.len() > context.limits.max_goal_bytes {
                bail!(
                    "{path}.goal: {} bytes exceed configured maximum {}",
                    goal.goal.len(),
                    context.limits.max_goal_bytes
                );
            }

            let label = format!("{path}.goal");
            let term = parse_goal(&goal.goal, &label)
                .with_context(|| format!("failed to parse {label}"))?;
            let kind = validate_ground_goal_for_circuit(&term, context.limits)
                .map_err(|error| prefix_circuit_error(&label, error))?;

            let resolution = match &kind {
                CircuitGoalKind::Predicate(indicator) => {
                    let is_policy = context.policy_predicates.contains(indicator);
                    let is_external_fact = context.external_predicates.contains(indicator)
                        && context.fact_predicates.contains(indicator);
                    if !is_policy && !is_external_fact {
                        bail!(
                            "{label}: predicate {}/{} is neither defined by the policy nor present in the fact registry",
                            indicator.name,
                            indicator.arity
                        );
                    }
                    if is_root && !is_policy {
                        bail!(
                            "{label}: root predicate {}/{} is not defined by the policy",
                            indicator.name,
                            indicator.arity
                        );
                    }
                    if is_policy {
                        GoalResolution::Policy {
                            predicate: indicator.clone(),
                        }
                    } else {
                        GoalResolution::ExternalFact {
                            predicate: indicator.clone(),
                        }
                    }
                }
                CircuitGoalKind::Builtin(_) if is_root => {
                    bail!("{label}: a proof-tree root must be a policy-defined predicate");
                }
                CircuitGoalKind::Builtin(builtin) => GoalResolution::Builtin { builtin: *builtin },
            };

            let ground_call = GroundCall::from_goal_term(&term)
                .with_context(|| format!("failed to normalize typed witness for {label}"))?;

            if goal.subtree.is_empty() {
                bail!("{path}.subtree: a goal must contain either [true] or child goals");
            }
            if goal.subtree.len() > context.limits.max_children_per_node {
                bail!(
                    "{path}.subtree: {} children exceed configured maximum {}",
                    goal.subtree.len(),
                    context.limits.max_children_per_node
                );
            }

            let marker_count = goal
                .subtree
                .iter()
                .filter(|node| matches!(node, ProofNode::True(_)))
                .count();
            if marker_count > 0 && !(goal.subtree.len() == 1 && marker_count == 1) {
                bail!("{path}.subtree: true is allowed only as the sole leaf marker [true]");
            }
            let must_be_leaf = matches!(
                resolution,
                GoalResolution::Builtin { .. } | GoalResolution::ExternalFact { .. }
            );
            if must_be_leaf && !matches!(goal.subtree.as_slice(), [ProofNode::True(true)]) {
                let category = match resolution {
                    GoalResolution::Builtin { .. } => "builtin",
                    GoalResolution::ExternalFact { .. } => "external fact",
                    GoalResolution::Policy { .. } => unreachable!(),
                };
                bail!("{path}.subtree: a {category} goal must use the leaf marker [true]");
            }

            let mut subtree = Vec::with_capacity(goal.subtree.len());
            for (index, child) in goal.subtree.iter().enumerate() {
                subtree.push(validate_proof_node(
                    child,
                    context,
                    &format!("{path}.subtree[{index}]"),
                    depth + 1,
                    false,
                    node_count,
                )?);
            }
            Ok(ValidatedProofNode::Goal(ValidatedGoal {
                ground_call,
                resolution,
                subtree,
            }))
        }
    }
}

fn prefix_circuit_error(prefix: &str, error: CircuitValidationError) -> anyhow::Error {
    anyhow::anyhow!("{prefix}.{}: {}", error.path, error.message)
}

pub(crate) fn semantic_local_step(g: &ValidatedGoal) -> Result<LocalStepWitness> {
    let transition = match &g.resolution {
        GoalResolution::Policy { .. } => {
            let mut children = Vec::new();
            for child in &g.subtree {
                match child {
                    ValidatedProofNode::Goal(child) => children.push(
                        ResolvedGoal::new(child.ground_call.clone(), child.resolution.clone())
                            .map_err(anyhow::Error::new)?,
                    ),
                    ValidatedProofNode::True if g.subtree.len() == 1 => {}
                    ValidatedProofNode::True => {
                        bail!("typed rule transition contains a mixed true marker")
                    }
                }
            }
            StepTransition::Rule { children }
        }
        GoalResolution::ExternalFact { .. } => StepTransition::ExternalFact,
        GoalResolution::Builtin { builtin } => StepTransition::Builtin { builtin: *builtin },
    };

    LocalStepWitness::new(g.ground_call.clone(), g.resolution.clone(), transition)
        .map_err(anyhow::Error::new)
}

#[cfg(test)]
mod validation_tests {
    use common::{
        circuit_dialect::CircuitLimits,
        commitments::{
            append_local_step_public_inputs, goal_commitment_for_node, local_step_commitments,
        },
        data::{FactEntry, GoalEntry, ProofNode},
        io::read_fact_hashes::parse_fact_hashes,
        policy_compiler::compile_policy_for_current_circuit,
        policy_ir::PolicyFile,
        typed_fact::TypedFact,
        unification_checker_circuit::UnificationCircuit,
        utils_2::common_helpers::MAX_FACTS_HASHES,
        witness_ir::{GoalResolution, LocalStepWitness},
    };
    use halo2_proofs::dev::MockProver;

    use super::{ValidatedProofNode, semantic_local_step, validate_proof_tree};
    use crate::legacy_circuit_v1::{LegacyFactRegistry, encode_local_step_for_legacy_circuit};

    fn inputs() -> (PolicyFile, Vec<TypedFact>, CircuitLimits) {
        let policy = serde_json::from_str(include_str!(
            "../../tests/fixtures/current-policy.rules.json"
        ))
        .unwrap();
        let raw_facts: Vec<FactEntry> =
            serde_yaml::from_str(include_str!("../../issue/src/facts.yaml")).unwrap();
        let limits =
            serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap();
        let facts = policy_parser::parse_fact_registry(&raw_facts, &limits, "facts.yaml").unwrap();
        (policy, facts, limits)
    }

    #[test]
    fn current_proof_tree_is_validated_before_witness_encoding() {
        let (policy, facts, limits) = inputs();
        let raw: Vec<ProofNode> =
            serde_json::from_str(include_str!("../../tests/fixtures/current-proof-tree.json"))
                .unwrap();
        let validated = validate_proof_tree(&raw, &policy, &facts, &limits).unwrap();
        assert_eq!(validated.roots.len(), 1);
    }

    #[test]
    fn current_fact_hash_fixture_matches_the_tracked_fact_registry() {
        let (_, facts, _) = inputs();
        let mut computed = facts
            .iter()
            .map(|fact| fact.commitment().unwrap())
            .collect::<Vec<_>>();
        computed.resize(MAX_FACTS_HASHES, halo2_proofs::pasta::Fp::zero());

        let expected = parse_fact_hashes(include_str!(
            "../../tests/fixtures/current-fact-hashes.json"
        ))
        .unwrap();
        assert_eq!(computed, expected);
    }

    #[test]
    fn every_current_goal_has_a_deterministic_typed_local_step() {
        fn visit(node: &ValidatedProofNode, output: &mut Vec<LocalStepWitness>) {
            if let ValidatedProofNode::Goal(goal) = node {
                output.push(semantic_local_step(goal).unwrap());
                for child in &goal.subtree {
                    visit(child, output);
                }
            }
        }

        let (policy, facts, limits) = inputs();
        let raw: Vec<ProofNode> =
            serde_json::from_str(include_str!("../../tests/fixtures/current-proof-tree.json"))
                .unwrap();
        let validated = validate_proof_tree(&raw, &policy, &facts, &limits).unwrap();
        let mut steps = Vec::new();
        for root in &validated.roots {
            visit(root, &mut steps);
        }

        assert_eq!(steps.len(), 50);
        assert!(
            steps
                .iter()
                .any(|step| matches!(step.resolution, GoalResolution::Policy { .. }))
        );
        assert!(
            steps
                .iter()
                .any(|step| matches!(step.resolution, GoalResolution::ExternalFact { .. }))
        );
        assert!(
            steps
                .iter()
                .any(|step| matches!(step.resolution, GoalResolution::Builtin { .. }))
        );

        let first = serde_json::to_string_pretty(&steps).unwrap();
        let second = serde_json::to_string_pretty(&steps).unwrap();
        assert_eq!(first, second);
        assert!(!first.contains("legacy_source"));
    }

    #[test]
    fn every_current_typed_step_has_a_checked_legacy_bridge_encoding() {
        fn visit(
            node: &ValidatedProofNode,
            facts: &LegacyFactRegistry,
            policy: &common::data::RuleTemplateFile,
            count: &mut usize,
        ) {
            if let ValidatedProofNode::Goal(goal) = node {
                let step = semantic_local_step(goal).unwrap();
                let encoded = encode_local_step_for_legacy_circuit(&step, facts, policy).unwrap();
                assert!(!encoded.goal_name.is_empty());
                *count += 1;
                for child in &goal.subtree {
                    visit(child, facts, policy, count);
                }
            }
        }

        let (policy, fact_entries, limits) = inputs();
        let registry = LegacyFactRegistry::new(&fact_entries).unwrap();
        let raw: Vec<ProofNode> =
            serde_json::from_str(include_str!("../../tests/fixtures/current-proof-tree.json"))
                .unwrap();
        let validated = validate_proof_tree(&raw, &policy, &fact_entries, &limits).unwrap();
        let compiled = compile_policy_for_current_circuit(&policy, &limits).unwrap();
        let mut count = 0;
        for root in &validated.roots {
            visit(root, &registry, compiled.layout(), &mut count);
        }
        assert_eq!(count, 50);
    }

    #[test]
    fn production_bridge_preserves_typed_goal_hash_on_every_current_tree_edge() {
        fn visit(
            node: &ValidatedProofNode,
            facts: &LegacyFactRegistry,
            policy: &common::data::RuleTemplateFile,
            edge_count: &mut usize,
        ) {
            let ValidatedProofNode::Goal(goal) = node else {
                return;
            };
            let parent_step = semantic_local_step(goal).unwrap();
            let parent_unif =
                encode_local_step_for_legacy_circuit(&parent_step, facts, policy).unwrap();
            let proof_child_count = goal
                .subtree
                .iter()
                .filter(|child| matches!(child, ValidatedProofNode::Goal(_)))
                .count();
            assert_eq!(
                parent_unif.typed.children.len(),
                proof_child_count,
                "native typed bridge child count diverged for {:?}",
                goal.ground_call,
            );
            let mut typed_child_index = 0;

            for child in &goal.subtree {
                let ValidatedProofNode::Goal(child_goal) = child else {
                    continue;
                };
                let child_step = semantic_local_step(child_goal).unwrap();
                let child_unif =
                    encode_local_step_for_legacy_circuit(&child_step, facts, policy).unwrap();
                assert_eq!(
                    parent_unif.typed.children[typed_child_index].hash(),
                    child_unif.typed.goal.hash(),
                    "native typed bridge hash diverged on edge {:?} -> {:?}",
                    goal.ground_call,
                    child_goal.ground_call,
                );
                typed_child_index += 1;
                *edge_count += 1;
                visit(child, facts, policy, edge_count);
            }
        }

        let (policy, fact_entries, limits) = inputs();
        let registry = LegacyFactRegistry::new(&fact_entries).unwrap();
        let raw: Vec<ProofNode> =
            serde_json::from_str(include_str!("../../tests/fixtures/current-proof-tree.json"))
                .unwrap();
        let validated = validate_proof_tree(&raw, &policy, &fact_entries, &limits).unwrap();
        let compiled = compile_policy_for_current_circuit(&policy, &limits).unwrap();
        let mut edge_count = 0;
        for root in &validated.roots {
            visit(root, &registry, compiled.layout(), &mut edge_count);
        }
        assert_eq!(edge_count, 49);
    }

    #[test]
    fn typed_circuit_goal_commitments_match_across_current_tree_edges() {
        fn visit(
            node: &ValidatedProofNode,
            facts: &LegacyFactRegistry,
            policy: &common::data::RuleTemplateFile,
            edge_count: &mut usize,
        ) {
            let ValidatedProofNode::Goal(goal) = node else {
                return;
            };
            let parent_step = semantic_local_step(goal).unwrap();
            let parent = encode_local_step_for_legacy_circuit(&parent_step, facts, policy).unwrap();
            let mut child_row = 1;
            for child in &goal.subtree {
                let ValidatedProofNode::Goal(child_goal) = child else {
                    continue;
                };
                let child_step = semantic_local_step(child_goal).unwrap();
                let child_projection =
                    encode_local_step_for_legacy_circuit(&child_step, facts, policy).unwrap();
                assert_eq!(
                    goal_commitment_for_node(&parent, child_row).unwrap(),
                    goal_commitment_for_node(&child_projection, 0).unwrap(),
                    "parent {:?} child row {child_row} {:?} and child canonical typed commitment diverged",
                    goal.ground_call,
                    child_goal.ground_call,
                );
                *edge_count += 1;
                child_row += 1;
                visit(child, facts, policy, edge_count);
            }
        }

        let (policy, fact_entries, limits) = inputs();
        let registry = LegacyFactRegistry::new(&fact_entries).unwrap();
        let raw: Vec<ProofNode> =
            serde_json::from_str(include_str!("../../tests/fixtures/current-proof-tree.json"))
                .unwrap();
        let validated = validate_proof_tree(&raw, &policy, &fact_entries, &limits).unwrap();
        let compiled = compile_policy_for_current_circuit(&policy, &limits).unwrap();
        let mut edge_count = 0;
        for root in &validated.roots {
            visit(root, &registry, compiled.layout(), &mut edge_count);
        }
        assert_eq!(edge_count, 49);
    }

    #[test]
    fn current_fact_and_recursive_rule_pass_through_the_new_pipeline() {
        fn find<'a>(
            node: &'a ValidatedProofNode,
            predicate: &str,
        ) -> Option<&'a super::ValidatedGoal> {
            let ValidatedProofNode::Goal(goal) = node else {
                return None;
            };
            if goal.ground_call.predicate.name == predicate {
                return Some(goal);
            }
            goal.subtree.iter().find_map(|child| find(child, predicate))
        }

        let (policy, fact_entries, limits) = inputs();
        let registry = LegacyFactRegistry::new(&fact_entries).unwrap();
        let compiled = compile_policy_for_current_circuit(&policy, &limits).unwrap();
        let legacy_policy = compiled.layout().clone();
        let (rules, typed_policy) = compiled.into_circuit_parts();
        let raw: Vec<ProofNode> =
            serde_json::from_str(include_str!("../../tests/fixtures/current-proof-tree.json"))
                .unwrap();
        let validated = validate_proof_tree(&raw, &policy, &fact_entries, &limits).unwrap();
        let public_hashes = parse_fact_hashes(include_str!(
            "../../tests/fixtures/current-fact-hashes.json"
        ))
        .unwrap();

        for predicate in [
            "currentConsumption",
            "sumOfMonthlyConsumptions",
            "consumptionClass",
            "savingsClass",
        ] {
            let goal = validated
                .roots
                .iter()
                .find_map(|root| find(root, predicate))
                .unwrap();
            let step = semantic_local_step(goal).unwrap();
            let unif =
                encode_local_step_for_legacy_circuit(&step, &registry, &legacy_policy).unwrap();
            let commitments = local_step_commitments(&rules, &typed_policy, &unif).unwrap();
            let public_inputs = append_local_step_public_inputs(&public_hashes, &commitments);
            let circuit = UnificationCircuit {
                rules: rules.clone(),
                typed_policy: typed_policy.clone(),
                unif,
            };
            assert!(
                MockProver::run(16, &circuit, vec![public_inputs])
                    .unwrap()
                    .verify()
                    .is_ok(),
                "new pipeline rejected current {predicate} witness"
            );
        }
    }

    #[test]
    fn prolog_less_than_or_equal_signature_matches_the_legacy_bridge() {
        let limits: CircuitLimits =
            serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap();
        let policy = policy_parser::parse_policy("p(X) :- X =< 10.", "le-policy.pl").unwrap();
        let raw = vec![ProofNode::GoalNode(GoalEntry {
            goal: "p(10)".to_owned(),
            subtree: vec![ProofNode::GoalNode(GoalEntry {
                goal: "10 =< 10".to_owned(),
                subtree: vec![ProofNode::True(true)],
            })],
        })];
        let validated = validate_proof_tree(&raw, &policy, &[], &limits).unwrap();
        let ValidatedProofNode::Goal(root) = &validated.roots[0] else {
            unreachable!()
        };
        let step = semantic_local_step(root).unwrap();
        let compiled = compile_policy_for_current_circuit(&policy, &limits).unwrap();
        let unif = encode_local_step_for_legacy_circuit(
            &step,
            &LegacyFactRegistry::new(&[]).unwrap(),
            compiled.layout(),
        )
        .unwrap();
        let (rules, typed_policy) = compiled.into_circuit_parts();
        let commitments = local_step_commitments(&rules, &typed_policy, &unif).unwrap();
        let public_inputs = append_local_step_public_inputs(
            &vec![halo2_proofs::pasta::Fp::zero(); MAX_FACTS_HASHES],
            &commitments,
        );
        let circuit = UnificationCircuit {
            rules,
            typed_policy,
            unif,
        };
        assert!(
            MockProver::run(16, &circuit, vec![public_inputs])
                .unwrap()
                .verify()
                .is_ok()
        );
    }

    #[test]
    fn list_unification_uses_one_common_projection_for_both_operands() {
        let limits: CircuitLimits =
            serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap();
        let policy =
            policy_parser::parse_policy("p(L) :- L = [a,b].", "list-unify-policy.pl").unwrap();
        let raw = vec![ProofNode::GoalNode(GoalEntry {
            goal: "p([a,b])".to_owned(),
            subtree: vec![ProofNode::GoalNode(GoalEntry {
                goal: "[a,b] = [a,b]".to_owned(),
                subtree: vec![ProofNode::True(true)],
            })],
        })];
        let validated = validate_proof_tree(&raw, &policy, &[], &limits).unwrap();
        let ValidatedProofNode::Goal(root) = &validated.roots[0] else {
            unreachable!()
        };
        let step = semantic_local_step(root).unwrap();
        let compiled = compile_policy_for_current_circuit(&policy, &limits).unwrap();
        let unif = encode_local_step_for_legacy_circuit(
            &step,
            &LegacyFactRegistry::new(&[]).unwrap(),
            compiled.layout(),
        )
        .unwrap();
        assert_eq!(
            unif.subtree_goals[0][0].args[0],
            unif.subtree_goals[0][0].args[1]
        );
        let (rules, typed_policy) = compiled.into_circuit_parts();
        let commitments = local_step_commitments(&rules, &typed_policy, &unif).unwrap();
        let public_inputs = append_local_step_public_inputs(
            &vec![halo2_proofs::pasta::Fp::zero(); MAX_FACTS_HASHES],
            &commitments,
        );
        let circuit = UnificationCircuit {
            rules,
            typed_policy,
            unif,
        };
        assert!(
            MockProver::run(16, &circuit, vec![public_inputs])
                .unwrap()
                .verify()
                .is_ok()
        );
    }

    #[test]
    fn invalid_markers_and_tree_shapes_are_rejected() {
        let (policy, facts, limits) = inputs();
        assert!(validate_proof_tree(&[], &policy, &facts, &limits).is_err());

        let false_leaf = vec![ProofNode::GoalNode(GoalEntry {
            goal: "endPrice(1)".to_owned(),
            subtree: vec![ProofNode::True(false)],
        })];
        assert!(
            validate_proof_tree(&false_leaf, &policy, &facts, &limits)
                .unwrap_err()
                .to_string()
                .contains("false")
        );

        let mixed = vec![ProofNode::GoalNode(GoalEntry {
            goal: "endPrice(1)".to_owned(),
            subtree: vec![
                ProofNode::True(true),
                ProofNode::GoalNode(GoalEntry {
                    goal: "priceBase(1)".to_owned(),
                    subtree: vec![ProofNode::True(true)],
                }),
            ],
        })];
        assert!(
            validate_proof_tree(&mixed, &policy, &facts, &limits)
                .unwrap_err()
                .to_string()
                .contains("sole leaf marker")
        );
    }

    #[test]
    fn unknown_predicates_and_extra_json_fields_are_rejected() {
        let (policy, facts, limits) = inputs();
        let unknown = vec![ProofNode::GoalNode(GoalEntry {
            goal: "endPrice(1)".to_owned(),
            subtree: vec![ProofNode::GoalNode(GoalEntry {
                goal: "misspelledFact(1)".to_owned(),
                subtree: vec![ProofNode::True(true)],
            })],
        })];
        assert!(
            validate_proof_tree(&unknown, &policy, &facts, &limits)
                .unwrap_err()
                .to_string()
                .contains("neither defined")
        );

        let extra = r#"[{"goal":"endPrice(1)","subtree":[true],"extra":1}]"#;
        assert!(serde_json::from_str::<Vec<ProofNode>>(extra).is_err());
    }
}
