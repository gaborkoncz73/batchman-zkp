//! Checked bridge from the typed witness IR to the unchanged circuit-v1
//! `TermFp` matrix.
//!
//! This module is intentionally named `legacy`: list subtrees are compressed
//! into host-side scalar digests and literal type tags are lost by the current
//! field codec.  It exists only to keep the current circuit operational while
//! the typed term-node circuit is introduced.

use anyhow::{Result, bail};
use common::{
    circuit_dialect::BuiltinKind,
    circuit_witness::TypedLocalStepFp,
    data::{
        ClauseTemplate, LocalStepKind, RuleTemplateFile, TermFp, TermRefComplex, TermSide,
        UnificationInputFp,
    },
    typed_fact::TypedFact,
    utils_2::common_helpers::{MAX_ARITY, MAX_PRED_LIST, to_fp_atom, to_fp_value},
    witness_ir::{
        GoalResolution, GroundCall, GroundTerm, LocalStepWitness, ResolvedGoal, StepTransition,
    },
};
use halo2_proofs::pasta::Fp;

#[derive(Debug, Clone)]
struct EncodedFact {
    name: Fp,
    arity: usize,
    args: Vec<Vec<Fp>>,
    salt: Fp,
}

#[derive(Debug, Clone)]
pub(crate) struct LegacyFactRegistry {
    entries: Vec<EncodedFact>,
}

impl LegacyFactRegistry {
    pub(crate) fn new(facts: &[TypedFact]) -> Result<Self> {
        let mut entries: Vec<EncodedFact> = Vec::with_capacity(facts.len());
        for (fact_index, fact) in facts.iter().enumerate() {
            if fact.goal.args.len() > MAX_ARITY {
                bail!(
                    "facts[{fact_index}]: arity {} exceeds legacy circuit maximum {MAX_ARITY}",
                    fact.goal.args.len()
                );
            }
            let encoded = EncodedFact {
                name: to_fp_value(&fact.goal.predicate.name),
                arity: fact.goal.predicate.arity,
                args: legacy_fact_argument_matrix(&fact.goal)?,
                salt: fact.salt,
            };
            if entries.iter().any(|entry| {
                entry.name == encoded.name
                    && entry.arity == encoded.arity
                    && entry.args == encoded.args
            }) {
                bail!(
                    "facts[{fact_index}]: typed fact {}/{} collides in the temporary legacy projection",
                    fact.goal.predicate.name,
                    fact.goal.predicate.arity
                );
            }
            entries.push(encoded);
        }
        Ok(Self { entries })
    }

    fn salt_for(&self, name: Fp, arity: usize, args: &[Vec<Fp>]) -> Option<Fp> {
        self.entries
            .iter()
            .find(|entry| entry.name == name && entry.arity == arity && entry.args == args)
            .map(|entry| entry.salt)
    }
}

fn legacy_fact_argument_matrix(goal: &GroundCall) -> Result<Vec<Vec<Fp>>> {
    if goal.args.len() > MAX_ARITY {
        bail!(
            "external fact {}/{} exceeds legacy maximum arity {MAX_ARITY}",
            goal.predicate.name,
            goal.predicate.arity
        );
    }
    let mut args = padding_matrix();
    for (argument, term) in goal.args.iter().enumerate() {
        args[argument] = legacy_fact_argument_row(term)?;
    }
    Ok(args)
}

/// Deterministic compatibility projection derived from the typed fact itself.
/// Flat proper lists keep the case-study tuple layout when it fits; larger or
/// improper lists use the historical textual digest, and arbitrary nested
/// terms fall back to the type-tagged compatibility digest.
fn legacy_fact_argument_row(term: &GroundTerm) -> Result<Vec<Fp>> {
    match term {
        GroundTerm::Atom { .. } | GroundTerm::Integer { .. } | GroundTerm::Nil => {
            row_from_values(&[scalar_value(term)?])
        }
        GroundTerm::Cons { .. } => flat_fact_list_row(term)
            .or_else(|_| legacy_digest(term).and_then(|value| row_from_values(&[value])))
            .or_else(|_| typed_term_digest(term).and_then(|value| row_from_values(&[value]))),
        GroundTerm::Compound { .. } => row_from_values(&[typed_term_digest(term)?]),
    }
}

pub(crate) fn encode_local_step_for_legacy_circuit(
    step: &LocalStepWitness,
    facts: &LegacyFactRegistry,
    policy: &RuleTemplateFile,
) -> Result<UnificationInputFp> {
    step.validate().map_err(anyhow::Error::new)?;
    let typed = TypedLocalStepFp::from_semantic_step(step).map_err(anyhow::Error::new)?;
    match &step.transition {
        StepTransition::Rule { children } => encode_policy_step(step, children, facts, policy),
        StepTransition::ExternalFact | StepTransition::Builtin { .. } => {
            let mut candidates = encode_call_candidates(&step.goal, &step.resolution, facts)?;
            Ok(UnificationInputFp {
                goal_name: candidates.remove(0),
                subtree_goals: vec![vec![true_marker()]],
                resolution: match &step.transition {
                    StepTransition::ExternalFact => LocalStepKind::ExternalFact,
                    StepTransition::Builtin { builtin } => LocalStepKind::Builtin(*builtin),
                    StepTransition::Rule { .. } => unreachable!(),
                },
                typed,
            })
        }
    }
}

fn encode_policy_step(
    step: &LocalStepWitness,
    children: &[ResolvedGoal],
    facts: &LegacyFactRegistry,
    policy: &RuleTemplateFile,
) -> Result<UnificationInputFp> {
    let predicate = policy
        .predicates
        .iter()
        .find(|predicate| {
            predicate.name == step.goal.predicate.name
                && predicate.arity == step.goal.predicate.arity
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "compiled policy has no predicate {}/{}",
                step.goal.predicate.name,
                step.goal.predicate.arity
            )
        })?;
    let parent_candidates = encode_call_candidates(&step.goal, &step.resolution, facts)?;
    let child_candidates = children
        .iter()
        .map(|child| encode_call_candidates(&child.goal, &child.resolution, facts))
        .collect::<Result<Vec<_>>>()?;
    let child_signatures = children
        .iter()
        .map(|child| legacy_signature(&child.goal))
        .collect::<Result<Vec<_>>>()?;
    let mut attempts = 0_usize;

    for clause in &predicate.clauses {
        if !signatures_match(&clause.children, &child_signatures) {
            continue;
        }
        for parent in &parent_candidates {
            let mut selected_children = Vec::with_capacity(child_candidates.len());
            if choose_child_projections(
                clause,
                parent,
                &child_candidates,
                0,
                &mut selected_children,
                &mut attempts,
            )? {
                let subtree_goals = if selected_children.is_empty() {
                    vec![vec![true_marker()]]
                } else {
                    selected_children
                };
                return Ok(UnificationInputFp {
                    goal_name: parent.clone(),
                    subtree_goals,
                    resolution: LocalStepKind::Rule,
                    typed: TypedLocalStepFp::from_semantic_step(step)
                        .map_err(anyhow::Error::new)?,
                });
            }
        }
    }

    bail!(
        "typed local step for {}/{} does not match any compiled legacy clause",
        step.goal.predicate.name,
        step.goal.predicate.arity
    )
}

fn encode_call_candidates(
    call: &GroundCall,
    resolution: &GoalResolution,
    facts: &LegacyFactRegistry,
) -> Result<Vec<Vec<TermFp>>> {
    if call.predicate.name == "is" {
        return Ok(vec![encode_is(call)?]);
    }
    if matches!(call.predicate.name.as_str(), "<" | "=<" | ">" | ">=") {
        return Ok(vec![encode_scalar_relation(call)?]);
    }

    if call.predicate.name == "=" {
        if call.args.len() != 2 {
            bail!("legacy unification encoding requires arity 2");
        }
        if call.args[0] != call.args[1] {
            bail!("ground unification builtin is false in the typed witness");
        }
        let name = to_fp_value("=");
        let candidates = argument_matrix_candidates(&call.args)?
            .into_iter()
            .filter(|args| args[0] == args[1])
            .map(|args| {
                vec![TermFp {
                    name,
                    args,
                    fact_hashes: Fp::zero(),
                }]
            })
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            bail!("ground unification has no common legacy projection");
        }
        return Ok(candidates);
    }

    let name = to_fp_value(&call.predicate.name);
    let matrices = argument_matrix_candidates(&call.args)?;
    match resolution {
        GoalResolution::ExternalFact { .. } => {
            let matches = matrices
                .into_iter()
                .filter_map(|args| {
                    facts
                        .salt_for(name, call.predicate.arity, &args)
                        .map(|salt| (args, salt))
                })
                .collect::<Vec<_>>();
            match matches.as_slice() {
                [] => bail!(
                    "external fact {}/{} has no exact entry in the legacy fact registry",
                    call.predicate.name,
                    call.predicate.arity
                ),
                [(args, salt)] => Ok(vec![vec![TermFp {
                    name,
                    args: args.clone(),
                    fact_hashes: *salt,
                }]]),
                _ => bail!(
                    "external fact {}/{} has multiple valid legacy projections; the encoding is ambiguous",
                    call.predicate.name,
                    call.predicate.arity
                ),
            }
        }
        GoalResolution::Policy { .. } | GoalResolution::Builtin { .. } => Ok(matrices
            .into_iter()
            .map(|args| {
                vec![TermFp {
                    name,
                    args,
                    fact_hashes: Fp::zero(),
                }]
            })
            .collect()),
    }
}

fn encode_is(call: &GroundCall) -> Result<Vec<TermFp>> {
    if call.args.len() != 2 {
        bail!("legacy is encoding requires arity 2");
    }
    let lhs = scalar_value(&call.args[0])?;
    let mut steps = Vec::new();
    let initial = arithmetic_left_spine(&call.args[1], &mut steps)?;

    let mut terms = vec![term_with_scalars("is", &[lhs, scalar_value(initial)?])?];
    for (operator, operand) in steps {
        terms.push(term_with_scalars(operator, &[scalar_value(operand)?])?);
    }
    Ok(terms)
}

fn encode_scalar_relation(call: &GroundCall) -> Result<Vec<TermFp>> {
    if call.args.len() != 2 {
        bail!(
            "legacy relation {}/{} requires arity 2",
            call.predicate.name,
            call.args.len()
        );
    }
    let builtin = match call.predicate.name.as_str() {
        "<" => BuiltinKind::LessThan,
        "=<" => BuiltinKind::LessThanOrEqual,
        ">" => BuiltinKind::GreaterThan,
        ">=" => BuiltinKind::GreaterThanOrEqual,
        _ => unreachable!("relation functor was checked by the caller"),
    };
    let legacy_name = builtin.legacy_circuit_functor();
    Ok(vec![term_with_scalars(
        legacy_name,
        &[scalar_value(&call.args[0])?, scalar_value(&call.args[1])?],
    )?])
}

fn legacy_signature(call: &GroundCall) -> Result<Vec<(String, usize)>> {
    if call.predicate.name == "is" {
        if call.args.len() != 2 {
            bail!("legacy is signature requires arity 2");
        }
        let mut steps = Vec::new();
        arithmetic_left_spine(&call.args[1], &mut steps)?;
        let mut signature = vec![("is".to_owned(), 2)];
        signature.extend(
            steps
                .into_iter()
                .map(|(operator, _)| (operator.to_owned(), 1)),
        );
        return Ok(signature);
    }
    let name = match call.predicate.name.as_str() {
        "<" => BuiltinKind::LessThan.legacy_circuit_functor(),
        "=<" => BuiltinKind::LessThanOrEqual.legacy_circuit_functor(),
        ">" => BuiltinKind::GreaterThan.legacy_circuit_functor(),
        ">=" => BuiltinKind::GreaterThanOrEqual.legacy_circuit_functor(),
        name => name,
    };
    Ok(vec![(name.to_owned(), call.predicate.arity)])
}

fn signatures_match(
    expected: &[Vec<common::data::ChildSig>],
    actual: &[Vec<(String, usize)>],
) -> bool {
    expected.len() == actual.len()
        && expected
            .iter()
            .zip(actual)
            .all(|(expected_row, actual_row)| {
                expected_row.len() == actual_row.len()
                    && expected_row
                        .iter()
                        .zip(actual_row)
                        .all(|(expected, actual)| {
                            expected.name == actual.0 && expected.arity == actual.1
                        })
            })
}

const MAX_PROJECTION_SEARCH_ATTEMPTS: usize = 100_000;

fn choose_child_projections(
    clause: &ClauseTemplate,
    parent: &[TermFp],
    candidates: &[Vec<Vec<TermFp>>],
    child_index: usize,
    selected: &mut Vec<Vec<TermFp>>,
    attempts: &mut usize,
) -> Result<bool> {
    if !resolved_equalities_hold(clause, parent, selected)? {
        return Ok(false);
    }
    if child_index == candidates.len() {
        return Ok(true);
    }

    for candidate in &candidates[child_index] {
        *attempts += 1;
        if *attempts > MAX_PROJECTION_SEARCH_ATTEMPTS {
            bail!(
                "legacy list projection search exceeded {MAX_PROJECTION_SEARCH_ATTEMPTS} deterministic attempts"
            );
        }
        selected.push(candidate.clone());
        if choose_child_projections(
            clause,
            parent,
            candidates,
            child_index + 1,
            selected,
            attempts,
        )? {
            return Ok(true);
        }
        selected.pop();
    }
    Ok(false)
}

fn resolved_equalities_hold(
    clause: &ClauseTemplate,
    parent: &[TermFp],
    children: &[Vec<TermFp>],
) -> Result<bool> {
    for equality in &clause.equalities {
        let left = resolve_side(&equality.left, parent, children)?;
        let right = resolve_side(&equality.right, parent, children)?;
        if let (Some(left), Some(right)) = (left, right) {
            if left != right {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

fn resolve_side(
    side: &TermSide,
    parent: &[TermFp],
    children: &[Vec<TermFp>],
) -> Result<Option<Fp>> {
    match side {
        TermSide::Value(value) => Ok(Some(to_fp_value(value))),
        TermSide::Ref(reference) => resolve_reference(reference, parent, children),
    }
}

fn resolve_reference(
    reference: &TermRefComplex,
    parent: &[TermFp],
    children: &[Vec<TermFp>],
) -> Result<Option<Fp>> {
    let row = if reference.children_node_list == 0 {
        parent
    } else {
        let Some(row) = children.get(reference.children_node_list - 1) else {
            return Ok(None);
        };
        row
    };
    let value = row
        .get(reference.predicate)
        .and_then(|term| term.args.get(reference.arg))
        .and_then(|argument| argument.get(reference.list_index))
        .copied()
        .ok_or_else(|| {
            anyhow::anyhow!("compiled legacy equality contains an out-of-range reference")
        })?;
    Ok(Some(value))
}

fn arithmetic_left_spine<'a>(
    term: &'a GroundTerm,
    steps: &mut Vec<(&'a str, &'a GroundTerm)>,
) -> Result<&'a GroundTerm> {
    match term {
        GroundTerm::Compound { functor, args }
            if matches!(functor.as_str(), "+" | "-" | "*" | "div") && args.len() == 2 =>
        {
            let initial = arithmetic_left_spine(&args[0], steps)?;
            if matches!(
                &args[1],
                GroundTerm::Compound { functor, .. }
                    if matches!(functor.as_str(), "+" | "-" | "*" | "div")
            ) {
                bail!("right-nested arithmetic cannot be encoded by legacy circuit-v1");
            }
            steps.push((functor, &args[1]));
            Ok(initial)
        }
        GroundTerm::Atom { .. }
        | GroundTerm::Integer { .. }
        | GroundTerm::Nil
        | GroundTerm::Cons { .. }
        | GroundTerm::Compound { .. } => Ok(term),
    }
}

fn term_with_scalars(name: &str, values: &[Fp]) -> Result<TermFp> {
    if values.len() > MAX_ARITY {
        bail!(
            "legacy term {name}/{} exceeds maximum arity {MAX_ARITY}",
            values.len()
        );
    }
    let mut args = padding_matrix();
    for (argument, value) in values.iter().enumerate() {
        args[argument][0] = *value;
    }
    Ok(TermFp {
        name: to_fp_value(name),
        args,
        fact_hashes: Fp::zero(),
    })
}

fn argument_matrix_candidates(args: &[GroundTerm]) -> Result<Vec<Vec<Vec<Fp>>>> {
    if args.len() > MAX_ARITY {
        bail!(
            "goal arity {} exceeds legacy circuit maximum {MAX_ARITY}",
            args.len()
        );
    }

    let mut matrices = vec![padding_matrix()];
    for (argument_index, argument) in args.iter().enumerate() {
        let rows = argument_row_candidates(argument)?;
        let mut next = Vec::new();
        for matrix in matrices {
            for row in &rows {
                let mut candidate = matrix.clone();
                candidate[argument_index] = row.clone();
                if !next.contains(&candidate) {
                    next.push(candidate);
                }
            }
        }
        matrices = next;
    }
    Ok(matrices)
}

fn argument_row_candidates(term: &GroundTerm) -> Result<Vec<Vec<Fp>>> {
    let mut rows = match term {
        GroundTerm::Atom { .. } | GroundTerm::Integer { .. } | GroundTerm::Nil => {
            vec![row_from_values(&[scalar_value(term)?])?]
        }
        GroundTerm::Cons { .. } => {
            // The first candidate is a canonical, type-tagged digest that can
            // carry arbitrary nested ground data through variable positions.
            // Legacy candidates remain available for existing clause literals,
            // list destructuring, and the string-matrix external fact format.
            let mut rows = vec![row_from_values(&[typed_term_digest(term)?])?];
            if let Ok(whole) = legacy_digest(term).and_then(|value| row_from_values(&[value])) {
                rows.push(whole);
            }
            if let Ok(structural) = structural_list_rows(term) {
                rows.extend(structural);
            }
            if let Ok(flat) = flat_fact_list_row(term) {
                rows.push(flat);
            }
            rows
        }
        GroundTerm::Compound { .. } => {
            vec![row_from_values(&[typed_term_digest(term)?])?]
        }
    };
    rows.dedup();
    Ok(rows)
}

fn typed_term_digest(term: &GroundTerm) -> Result<Fp> {
    let canonical = serde_json::to_string(term)?;
    Ok(to_fp_value(&format!("batchman-ground-term-v1:{canonical}")))
}

fn structural_list_rows(term: &GroundTerm) -> Result<Vec<Vec<Fp>>> {
    let mut rows = Vec::new();
    let mut values = Vec::new();
    let mut cursor = term;
    while let GroundTerm::Cons { head, tail } = cursor {
        flatten_comma_values(head, &mut values)?;
        cursor = tail;
        let mut projection = values.clone();
        projection.push(legacy_digest(cursor)?);
        if projection.len() <= MAX_PRED_LIST {
            rows.push(row_from_values(&projection)?);
        }
    }
    Ok(rows)
}

fn flat_fact_list_row(term: &GroundTerm) -> Result<Vec<Fp>> {
    let mut values = Vec::new();
    let mut cursor = term;
    while let GroundTerm::Cons { head, tail } = cursor {
        flatten_comma_values(head, &mut values)?;
        cursor = tail;
    }
    if !matches!(cursor, GroundTerm::Nil) {
        bail!("improper list cannot use the flat legacy fact projection");
    }
    row_from_values(&values)
}

fn flatten_comma_values(term: &GroundTerm, output: &mut Vec<Fp>) -> Result<()> {
    match term {
        GroundTerm::Compound { functor, args } if functor == "," && args.len() == 2 => {
            flatten_comma_values(&args[0], output)?;
            flatten_comma_values(&args[1], output)
        }
        GroundTerm::Atom { .. } | GroundTerm::Integer { .. } => {
            output.push(scalar_value(term)?);
            Ok(())
        }
        other => bail!("legacy list item {other:?} is not a scalar or comma tuple"),
    }
}

fn row_from_values(values: &[Fp]) -> Result<Vec<Fp>> {
    if values.len() > MAX_PRED_LIST {
        bail!(
            "legacy list projection needs {} slots, maximum is {MAX_PRED_LIST}",
            values.len()
        );
    }
    let mut row = vec![-Fp::one(); MAX_PRED_LIST];
    row[..values.len()].copy_from_slice(values);
    Ok(row)
}

fn scalar_value(term: &GroundTerm) -> Result<Fp> {
    match term {
        GroundTerm::Atom { value } => Ok(to_fp_atom(value)),
        GroundTerm::Integer { value } => Ok(Fp::from(*value)),
        GroundTerm::Nil => Ok(to_fp_value("[]")),
        other => bail!("legacy scalar slot cannot represent {other:?}"),
    }
}

fn legacy_digest(term: &GroundTerm) -> Result<Fp> {
    Ok(to_fp_value(&legacy_text(term)?))
}

fn legacy_text(term: &GroundTerm) -> Result<String> {
    match term {
        GroundTerm::Atom { value } => Ok(value.clone()),
        GroundTerm::Integer { value } => Ok(value.to_string()),
        GroundTerm::Nil => Ok("[]".to_owned()),
        GroundTerm::Cons { .. } => {
            let mut items = Vec::new();
            let mut cursor = term;
            while let GroundTerm::Cons { head, tail } = cursor {
                items.push(legacy_text(head)?);
                cursor = tail;
            }
            let mut text = items.join(",");
            if !matches!(cursor, GroundTerm::Nil) {
                text.push('|');
                text.push_str(&legacy_text(cursor)?);
            }
            Ok(text)
        }
        GroundTerm::Compound { functor, args } if functor == "," && args.len() == 2 => Ok(format!(
            "{},{}",
            legacy_text(&args[0])?,
            legacy_text(&args[1])?
        )),
        other => bail!("legacy digest cannot represent nested term {other:?}"),
    }
}

fn padding_matrix() -> Vec<Vec<Fp>> {
    vec![vec![-Fp::one(); MAX_PRED_LIST]; MAX_ARITY]
}

fn true_marker() -> TermFp {
    TermFp {
        name: Fp::zero(),
        args: padding_matrix(),
        fact_hashes: Fp::zero(),
    }
}

#[cfg(test)]
mod tests {
    use common::{
        circuit_dialect::CircuitLimits,
        policy_compiler::compile_policy_for_current_circuit,
        typed_fact::TypedFact,
        utils_2::common_helpers::to_fp_value,
        witness_ir::{GoalResolution, GroundCall, LocalStepWitness, ResolvedGoal, StepTransition},
    };
    use halo2_proofs::pasta::Fp;
    use policy_parser::parse_goal;

    use super::{LegacyFactRegistry, encode_local_step_for_legacy_circuit};

    fn call(source: &str) -> GroundCall {
        GroundCall::from_goal_term(&parse_goal(source, "legacy-bridge-test").unwrap()).unwrap()
    }

    fn fact(goal: &str, salt: u64) -> TypedFact {
        TypedFact {
            goal: call(goal),
            salt: Fp::from(salt),
        }
    }

    fn external_step(goal: GroundCall) -> LocalStepWitness {
        LocalStepWitness::new(
            goal.clone(),
            GoalResolution::ExternalFact {
                predicate: goal.predicate.clone(),
            },
            StepTransition::ExternalFact,
        )
        .unwrap()
    }

    fn policy_fact_step(goal: GroundCall) -> LocalStepWitness {
        LocalStepWitness::new(
            goal.clone(),
            GoalResolution::Policy {
                predicate: goal.predicate.clone(),
            },
            StepTransition::Rule {
                children: Vec::new(),
            },
        )
        .unwrap()
    }

    fn policy(source: &str) -> common::data::RuleTemplateFile {
        let semantic = policy_parser::parse_policy(source, "legacy-bridge-policy.pl").unwrap();
        let limits: CircuitLimits =
            serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap();
        compile_policy_for_current_circuit(&semantic, &limits)
            .unwrap()
            .layout()
            .clone()
    }

    fn no_policy() -> common::data::RuleTemplateFile {
        common::data::RuleTemplateFile {
            predicates: Vec::new(),
        }
    }

    #[test]
    fn scalar_external_fact_projection_is_exact() {
        let registry = LegacyFactRegistry::new(&[fact("p(a,1)", 7)]).unwrap();
        let encoded = encode_local_step_for_legacy_circuit(
            &external_step(call("p(a, 1)")),
            &registry,
            &no_policy(),
        )
        .unwrap();

        assert_eq!(encoded.goal_name[0].name, to_fp_value("p"));
        assert_eq!(encoded.goal_name[0].args[0][0], to_fp_value("a"));
        assert_eq!(encoded.goal_name[0].args[1][0], Fp::from(1));
        assert_eq!(encoded.goal_name[0].fact_hashes, Fp::from(7));
        assert_eq!(encoded.subtree_goals.len(), 1);
        assert_eq!(encoded.subtree_goals[0][0].name, Fp::zero());
    }

    #[test]
    fn proper_list_and_tuple_list_match_their_fact_rows() {
        let registry = LegacyFactRegistry::new(&[
            fact("months([a,b])", 8),
            fact("credential([(role,nominal,10000)])", 9),
            fact("open_list([a|tail])", 10),
        ])
        .unwrap();

        let list = encode_local_step_for_legacy_circuit(
            &external_step(call("months([a,b])")),
            &registry,
            &no_policy(),
        )
        .unwrap();
        assert_eq!(list.goal_name[0].args[0][0], to_fp_value("a"));
        assert_eq!(list.goal_name[0].args[0][1], to_fp_value("b"));
        assert_eq!(list.goal_name[0].fact_hashes, Fp::from(8));

        let tuple = encode_local_step_for_legacy_circuit(
            &external_step(call("credential([(role,nominal,10000)])")),
            &registry,
            &no_policy(),
        )
        .unwrap();
        assert_eq!(tuple.goal_name[0].args[0][0], to_fp_value("role"));
        assert_eq!(tuple.goal_name[0].args[0][1], to_fp_value("nominal"));
        assert_eq!(tuple.goal_name[0].args[0][2], Fp::from(10000));
        assert_eq!(tuple.goal_name[0].fact_hashes, Fp::from(9));

        let open = encode_local_step_for_legacy_circuit(
            &external_step(call("open_list([a|tail])")),
            &registry,
            &no_policy(),
        )
        .unwrap();
        assert_eq!(open.goal_name[0].args[0][0], to_fp_value("a|tail"));
        assert_eq!(open.goal_name[0].fact_hashes, Fp::from(10));
    }

    #[test]
    fn the_same_external_fact_projects_identically_as_parent_and_child() {
        let registry = LegacyFactRegistry::new(&[fact("months([a,b])", 8)]).unwrap();
        let external = external_step(call("months([a,b])"));
        let external_encoded =
            encode_local_step_for_legacy_circuit(&external, &registry, &no_policy()).unwrap();

        let parent = call("q([a,b])");
        let child = external.goal.clone();
        let policy_step = LocalStepWitness::new(
            parent.clone(),
            GoalResolution::Policy {
                predicate: parent.predicate.clone(),
            },
            StepTransition::Rule {
                children: vec![
                    ResolvedGoal::new(
                        child.clone(),
                        GoalResolution::ExternalFact {
                            predicate: child.predicate.clone(),
                        },
                    )
                    .unwrap(),
                ],
            },
        )
        .unwrap();
        let child_encoded = encode_local_step_for_legacy_circuit(
            &policy_step,
            &registry,
            &policy("q(L) :- months(L)."),
        )
        .unwrap();

        let parent_fact = &external_encoded.goal_name[0];
        let policy_parent = &child_encoded.goal_name[0];
        let child_fact = &child_encoded.subtree_goals[0][0];
        assert_eq!(policy_parent.args, child_fact.args);
        assert_eq!(parent_fact.name, child_fact.name);
        assert_eq!(parent_fact.args, child_fact.args);
        assert_eq!(parent_fact.fact_hashes, child_fact.fact_hashes);
    }

    #[test]
    fn exact_proper_and_improper_list_patterns_select_the_matching_prefix() {
        let registry = LegacyFactRegistry::new(&[]).unwrap();

        let proper = encode_local_step_for_legacy_circuit(
            &policy_fact_step(call("p([a,b])")),
            &registry,
            &policy("p([a,b])."),
        )
        .unwrap();
        assert_eq!(proper.goal_name[0].args[0][0], to_fp_value("a"));
        assert_eq!(proper.goal_name[0].args[0][1], to_fp_value("b"));
        assert_eq!(proper.goal_name[0].args[0][2], to_fp_value("[]"));

        let improper = encode_local_step_for_legacy_circuit(
            &policy_fact_step(call("p([a,b|tail])")),
            &registry,
            &policy("p([a,b|tail])."),
        )
        .unwrap();
        assert_eq!(improper.goal_name[0].args[0][0], to_fp_value("a"));
        assert_eq!(improper.goal_name[0].args[0][1], to_fp_value("b"));
        assert_eq!(improper.goal_name[0].args[0][2], to_fp_value("tail"));
    }

    #[test]
    fn nested_ground_data_can_flow_through_policy_variables() {
        let registry = LegacyFactRegistry::new(&[]).unwrap();
        let layout = policy("p(X) :- q(X). q(_).");

        for (parent_source, child_source) in [("p(f(a))", "q(f(a))"), ("p([[a]])", "q([[a]])")] {
            let parent = call(parent_source);
            let child = call(child_source);
            let step = LocalStepWitness::new(
                parent.clone(),
                GoalResolution::Policy {
                    predicate: parent.predicate.clone(),
                },
                StepTransition::Rule {
                    children: vec![
                        ResolvedGoal::new(
                            child.clone(),
                            GoalResolution::Policy {
                                predicate: child.predicate.clone(),
                            },
                        )
                        .unwrap(),
                    ],
                },
            )
            .unwrap();
            let encoded = encode_local_step_for_legacy_circuit(&step, &registry, &layout).unwrap();
            assert_eq!(encoded.goal_name[0].args, encoded.subtree_goals[0][0].args);
        }
    }

    #[test]
    fn missing_and_legacy_colliding_fact_projections_are_rejected() {
        let missing_registry = LegacyFactRegistry::new(&[]).unwrap();
        assert!(
            encode_local_step_for_legacy_circuit(
                &external_step(call("months([a,b])")),
                &missing_registry,
                &no_policy(),
            )
            .unwrap_err()
            .to_string()
            .contains("no exact entry")
        );

        assert!(
            LegacyFactRegistry::new(&[fact("p([])", 8), fact("p('[]')", 9)])
                .unwrap_err()
                .to_string()
                .contains("collides")
        );
    }

    #[test]
    fn policy_literals_are_checked_before_the_lossy_circuit_projection() {
        let error = encode_local_step_for_legacy_circuit(
            &policy_fact_step(call("p(b)")),
            &LegacyFactRegistry::new(&[]).unwrap(),
            &policy("p(a)."),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("does not match any compiled legacy clause"));
    }
}
