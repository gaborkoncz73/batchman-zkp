//! Selected-clause checks over canonical typed Prolog term trees.
//!
//! Candidate flags come from the bounded rule-selection fold.  Every policy
//! reference is opened at a compiler-fixed path; the opening itself is always
//! authenticated, while its connection to the proof goal and every pattern is
//! gated by that candidate's constrained selection bit. Variable occurrences
//! are checked by a transcript-challenge-compressed matrix dot product.

use std::collections::{BTreeMap, BTreeSet};

use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
};
use halo2curves::ff::Field;

use crate::{
    chips::{
        typed_term_chip::{AssignedTypedGoal, AssignedTypedOpening, TypedTermChip},
        typed_variable_dot_chip::{
            TypedVariableCandidate, TypedVariableDotChip, TypedVariableDotConfig,
            TypedVariableEquality,
        },
    },
    data::RuleTemplateFileFp,
    typed_policy::{
        TypedClausePlan, TypedGoalSlot, TypedPatternCheck, TypedPolicyPlan, TypedPredicatePlan,
        TypedTermRef,
    },
    typed_term::{
        TermPathStep, TypedNodePreimage, build_dummy_term_opening, build_term_opening,
        hash_ground_term, symbol_to_field,
    },
    witness_ir::{GroundCall, GroundTerm},
};

#[derive(Clone, Debug)]
pub struct TypedRuleConfig {
    flag: Column<Advice>,
    left: Column<Advice>,
    right: Column<Advice>,
    q_selected_equal: Selector,
    q_flag_first: Selector,
    q_flag_fold: Selector,
    q_rule_ok: Selector,
    variable_dot: TypedVariableDotConfig,
}

#[derive(Clone, Debug)]
pub struct TypedRuleChip {
    config: TypedRuleConfig,
}

#[derive(Clone, Debug)]
pub struct TypedRuleResult {
    pub candidate_flags: Vec<AssignedCell<Fp, Fp>>,
    pub rule_ok: AssignedCell<Fp, Fp>,
}

pub struct TypedRuleInputs<'a> {
    pub parent: &'a AssignedTypedGoal,
    /// Fixed-width assigned child slots. Missing semantic children contain the
    /// canonical empty typed goal.
    pub children: &'a [AssignedTypedGoal],
    pub semantic_child_count: &'a AssignedCell<Fp, Fp>,
    pub ground_parent: &'a GroundCall,
    pub ground_children: &'a [GroundCall],
}

impl TypedRuleChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> TypedRuleConfig {
        let flag = meta.advice_column();
        let left = meta.advice_column();
        let right = meta.advice_column();
        for column in [flag, left, right] {
            meta.enable_equality(column);
        }
        let q_selected_equal = meta.selector();
        let q_flag_first = meta.selector();
        let q_flag_fold = meta.selector();
        let q_rule_ok = meta.selector();
        let variable_dot = TypedVariableDotChip::configure_with_columns(meta, flag, left, right);
        meta.create_gate("selected typed policy component equality", |meta| {
            let q = meta.query_selector(q_selected_equal);
            let flag = meta.query_advice(flag, Rotation::cur());
            let left = meta.query_advice(left, Rotation::cur());
            let right = meta.query_advice(right, Rotation::cur());
            vec![q * flag * (left - right)]
        });
        meta.create_gate("typed rule candidate flag fold", |meta| {
            let q = meta.query_selector(q_flag_fold);
            let flag = meta.query_advice(flag, Rotation::cur());
            let accumulator = meta.query_advice(left, Rotation::cur());
            let next = meta.query_advice(left, Rotation::next());
            vec![
                q.clone()
                    * flag.clone()
                    * (flag.clone() - halo2_proofs::plonk::Expression::Constant(Fp::ONE)),
                q * (next - accumulator - flag),
            ]
        });
        meta.create_gate("typed rule candidate fold starts at zero", |meta| {
            let q = meta.query_selector(q_flag_first);
            let accumulator = meta.query_advice(left, Rotation::cur());
            vec![q * accumulator]
        });
        meta.create_gate("typed rule_ok is boolean", |meta| {
            let q = meta.query_selector(q_rule_ok);
            let rule_ok = meta.query_advice(left, Rotation::cur());
            vec![
                q * rule_ok.clone()
                    * (rule_ok - halo2_proofs::plonk::Expression::Constant(Fp::ONE)),
            ]
        });
        TypedRuleConfig {
            flag,
            left,
            right,
            q_selected_equal,
            q_flag_first,
            q_flag_fold,
            q_rule_ok,
            variable_dot,
        }
    }

    pub fn construct(config: TypedRuleConfig) -> Self {
        Self { config }
    }

    /// Deterministically chooses the first typed candidate satisfied by the
    /// canonical ground statement, constrains a boolean at-most-one flag fold,
    /// and applies all authenticated typed checks to those exact flag cells.
    pub fn select_and_assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        typed_term: &TypedTermChip,
        legacy_policy: &RuleTemplateFileFp,
        policy: &TypedPolicyPlan,
        inputs: TypedRuleInputs<'_>,
        enable_rule: bool,
    ) -> Result<TypedRuleResult, Error> {
        policy.validate().map_err(|_| Error::Synthesis)?;
        validate_candidate_alignment(legacy_policy, policy)?;
        let candidates = flattened_candidates(policy);
        let selected = if enable_rule {
            candidates
                .iter()
                .position(|(predicate, candidate)| candidate_matches(predicate, candidate, &inputs))
        } else {
            None
        };

        let (candidate_flags, rule_ok) = layouter.assign_region(
            || "typed rule candidate selection fold",
            |mut region| {
                let mut flags = Vec::with_capacity(candidates.len());
                let mut accumulator = Fp::ZERO;
                for candidate_index in 0..candidates.len() {
                    self.config
                        .q_flag_fold
                        .enable(&mut region, candidate_index)?;
                    if candidate_index == 0 {
                        self.config
                            .q_flag_first
                            .enable(&mut region, candidate_index)?;
                    }
                    region.assign_advice(
                        || format!("typed candidate accumulator[{candidate_index}]"),
                        self.config.left,
                        candidate_index,
                        || Value::known(accumulator),
                    )?;
                    let value = if selected == Some(candidate_index) {
                        Fp::ONE
                    } else {
                        Fp::ZERO
                    };
                    flags.push(region.assign_advice(
                        || format!("typed candidate flag[{candidate_index}]"),
                        self.config.flag,
                        candidate_index,
                        || Value::known(value),
                    )?);
                    accumulator += value;
                }
                let final_row = candidates.len();
                self.config.q_rule_ok.enable(&mut region, final_row)?;
                let rule_ok = region.assign_advice(
                    || "typed rule_ok",
                    self.config.left,
                    final_row,
                    || Value::known(accumulator),
                )?;
                region.assign_advice(
                    || "typed candidate flag final padding",
                    self.config.flag,
                    final_row,
                    || Value::known(Fp::ZERO),
                )?;
                Ok((flags, rule_ok))
            },
        )?;

        self.assign(
            layouter.namespace(|| "typed selected candidate checks"),
            typed_term,
            legacy_policy,
            policy,
            &candidate_flags,
            inputs,
        )?;
        Ok(TypedRuleResult {
            candidate_flags,
            rule_ok,
        })
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        typed_term: &TypedTermChip,
        legacy_policy: &RuleTemplateFileFp,
        policy: &TypedPolicyPlan,
        candidate_flags: &[AssignedCell<Fp, Fp>],
        inputs: TypedRuleInputs<'_>,
    ) -> Result<(), Error> {
        policy.validate().map_err(|_| Error::Synthesis)?;
        validate_candidate_alignment(legacy_policy, policy)?;
        let candidates = flattened_candidates(policy);
        if candidates.len() != candidate_flags.len() {
            return Err(Error::Synthesis);
        }

        // Authenticate each distinct typed coordinate once. Candidate-local
        // root links and policy checks remain separately gated, but repeated
        // paths no longer repeat their Poseidon opening work.
        let all_references = candidates
            .iter()
            .flat_map(|(_predicate, candidate)| candidate_references(candidate))
            .collect::<BTreeSet<_>>();
        let mut opened = BTreeMap::<TypedTermRef, AssignedTypedOpening>::new();
        for (reference_index, reference) in all_references.into_iter().enumerate() {
            let opening = opening_for_reference(&reference, &inputs)?;
            let assigned = typed_term.verify_opening_detached(
                layouter.namespace(|| format!("shared typed opening {reference_index}")),
                &opening,
                &reference.path,
            )?;
            opened.insert(reference, assigned);
        }

        let variable_dot = TypedVariableDotChip::construct(self.config.variable_dot.clone());
        variable_dot
            .load_zero_table(layouter.namespace(|| "typed variable dot-product zero table"))?;
        let mut variable_candidates = Vec::with_capacity(candidates.len());

        for (candidate_index, ((predicate, candidate), flag)) in
            candidates.iter().copied().zip(candidate_flags).enumerate()
        {
            self.bind_constant_if(
                layouter.namespace(|| format!("typed candidate {candidate_index} parent name")),
                flag,
                &inputs.parent.predicate,
                predicate.name,
            )?;
            self.bind_constant_if(
                layouter.namespace(|| format!("typed candidate {candidate_index} parent arity")),
                flag,
                &inputs.parent.arity,
                Fp::from(u64::from(predicate.arity)),
            )?;
            self.bind_constant_if(
                layouter.namespace(|| format!("typed candidate {candidate_index} child count")),
                flag,
                inputs.semantic_child_count,
                Fp::from(candidate.children.len() as u64),
            )?;

            for (child_index, signature) in candidate.children.iter().enumerate() {
                let child = inputs.children.get(child_index).ok_or(Error::Synthesis)?;
                self.bind_constant_if(
                    layouter.namespace(|| {
                        format!("typed candidate {candidate_index} child {child_index} name")
                    }),
                    flag,
                    &child.predicate,
                    signature.name,
                )?;
                self.bind_constant_if(
                    layouter.namespace(|| {
                        format!("typed candidate {candidate_index} child {child_index} arity")
                    }),
                    flag,
                    &child.arity,
                    Fp::from(u64::from(signature.arity)),
                )?;
            }

            for (reference_index, reference) in
                candidate_references(candidate).into_iter().enumerate()
            {
                let expected_root = assigned_root(&reference, &inputs)?;
                let assigned = opened.get(&reference).ok_or(Error::Synthesis)?;
                typed_term.bind_opening_root_if(
                    layouter.namespace(|| {
                        format!("typed candidate {candidate_index} root link {reference_index}")
                    }),
                    flag,
                    assigned,
                    expected_root,
                )?;
            }

            let mut variable_equalities = Vec::with_capacity(candidate.equalities.len());
            for equality in &candidate.equalities {
                let left = &opened.get(&equality.left).ok_or(Error::Synthesis)?.target;
                let right = &opened.get(&equality.right).ok_or(Error::Synthesis)?.target;
                variable_equalities.push(TypedVariableEquality::new(
                    left.digest.clone(),
                    right.digest.clone(),
                ));
            }
            variable_candidates.push(TypedVariableCandidate {
                flag: flag.clone(),
                equalities: variable_equalities,
            });

            for (pattern_index, pattern) in candidate.patterns.iter().enumerate() {
                match pattern {
                    TypedPatternCheck::Exact { at, digest } => {
                        let target = &opened.get(at).ok_or(Error::Synthesis)?.target;
                        self.bind_constant_if(
                            layouter.namespace(|| {
                                format!(
                                    "typed candidate {candidate_index} exact pattern {pattern_index}"
                                )
                            }),
                            flag,
                            &target.digest,
                            *digest,
                        )?;
                    }
                    TypedPatternCheck::Shape {
                        at,
                        tag,
                        payload,
                        arity,
                    } => {
                        let target = &opened.get(at).ok_or(Error::Synthesis)?.target;
                        for (field, expected, label) in [
                            (&target.preimage.tag, Fp::from(tag.as_u64()), "tag"),
                            (&target.preimage.payload, *payload, "payload"),
                            (&target.preimage.arity, Fp::from(u64::from(*arity)), "arity"),
                        ] {
                            self.bind_constant_if(
                                layouter.namespace(|| {
                                    format!(
                                        "typed candidate {candidate_index} shape {pattern_index} {label}"
                                    )
                                }),
                                flag,
                                field,
                                expected,
                            )?;
                        }
                    }
                }
            }
        }
        variable_dot.assign_candidates(
            layouter.namespace(|| "typed candidate variable dot products"),
            &variable_candidates,
        )?;
        Ok(())
    }

    fn bind_constant_if(
        &self,
        mut layouter: impl Layouter<Fp>,
        flag: &AssignedCell<Fp, Fp>,
        actual: &AssignedCell<Fp, Fp>,
        expected: Fp,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "candidate-gated typed constant equality",
            |mut region| {
                self.config.q_selected_equal.enable(&mut region, 0)?;
                flag.copy_advice(|| "candidate flag", &mut region, self.config.flag, 0)?;
                actual.copy_advice(|| "typed value", &mut region, self.config.left, 0)?;
                region.assign_advice_from_constant(
                    || "policy-fixed typed value",
                    self.config.right,
                    0,
                    expected,
                )?;
                Ok(())
            },
        )
    }
}

fn flattened_candidates(policy: &TypedPolicyPlan) -> Vec<(&TypedPredicatePlan, &TypedClausePlan)> {
    policy
        .predicates
        .iter()
        .flat_map(|predicate| {
            predicate
                .candidates
                .iter()
                .map(move |candidate| (predicate, candidate))
        })
        .collect()
}

fn candidate_matches(
    predicate: &TypedPredicatePlan,
    candidate: &TypedClausePlan,
    inputs: &TypedRuleInputs<'_>,
) -> bool {
    if symbol_to_field(&inputs.ground_parent.predicate.name) != predicate.name
        || inputs.ground_parent.predicate.arity != usize::from(predicate.arity)
        || inputs.ground_children.len() != candidate.children.len()
    {
        return false;
    }
    if inputs
        .ground_children
        .iter()
        .zip(&candidate.children)
        .any(|(actual, expected)| {
            symbol_to_field(&actual.predicate.name) != expected.name
                || actual.predicate.arity != usize::from(expected.arity)
        })
    {
        return false;
    }

    for equality in &candidate.equalities {
        let Some(left) = target_term(&equality.left, inputs) else {
            return false;
        };
        let Some(right) = target_term(&equality.right, inputs) else {
            return false;
        };
        if hash_ground_term(left).ok() != hash_ground_term(right).ok() {
            return false;
        }
    }
    for pattern in &candidate.patterns {
        match pattern {
            TypedPatternCheck::Exact { at, digest } => {
                if target_term(at, inputs).and_then(|term| hash_ground_term(term).ok())
                    != Some(*digest)
                {
                    return false;
                }
            }
            TypedPatternCheck::Shape {
                at,
                tag,
                payload,
                arity,
            } => {
                let Some(node) = target_term(at, inputs)
                    .and_then(|term| TypedNodePreimage::from_ground_term(term).ok())
                else {
                    return false;
                };
                let (actual_payload, actual_arity) = match &node {
                    TypedNodePreimage::Cons { .. } => (Fp::ZERO, 2),
                    TypedNodePreimage::Compound { functor, arity, .. } => (*functor, *arity),
                    TypedNodePreimage::Atom { symbol } => (*symbol, 0),
                    TypedNodePreimage::Integer { .. } | TypedNodePreimage::Nil => (Fp::ZERO, 0),
                };
                if node.tag() != *tag || actual_payload != *payload || actual_arity != *arity {
                    return false;
                }
            }
        }
    }
    true
}

fn target_term<'a>(
    reference: &TypedTermRef,
    inputs: &'a TypedRuleInputs<'_>,
) -> Option<&'a GroundTerm> {
    let mut current = ground_term(reference, inputs)?;
    for step in &reference.path.steps {
        current = match (current, step) {
            (GroundTerm::Cons { head, .. }, TermPathStep::Head) => head,
            (GroundTerm::Cons { tail, .. }, TermPathStep::Tail) => tail,
            (GroundTerm::Compound { args, .. }, TermPathStep::Argument { index }) => {
                args.get(usize::from(*index))?
            }
            _ => return None,
        };
    }
    Some(current)
}

fn validate_candidate_alignment(
    legacy: &RuleTemplateFileFp,
    typed: &TypedPolicyPlan,
) -> Result<(), Error> {
    if legacy.predicates.len() != typed.predicates.len() {
        return Err(Error::Synthesis);
    }
    for (legacy_predicate, typed_predicate) in legacy.predicates.iter().zip(&typed.predicates) {
        if legacy_predicate.arity != Fp::from(u64::from(typed_predicate.arity))
            || legacy_predicate.clauses.len() != typed_predicate.candidates.len()
        {
            return Err(Error::Synthesis);
        }
        for (legacy_clause, typed_candidate) in legacy_predicate
            .clauses
            .iter()
            .zip(&typed_predicate.candidates)
        {
            if legacy_clause.children.len() != typed_candidate.children.len() {
                return Err(Error::Synthesis);
            }
        }
    }
    Ok(())
}

fn candidate_references(
    candidate: &crate::typed_policy::TypedClausePlan,
) -> BTreeSet<TypedTermRef> {
    let mut references = BTreeSet::new();
    for equality in &candidate.equalities {
        references.insert(equality.left.clone());
        references.insert(equality.right.clone());
    }
    for pattern in &candidate.patterns {
        references.insert(match pattern {
            TypedPatternCheck::Exact { at, .. } | TypedPatternCheck::Shape { at, .. } => at.clone(),
        });
    }
    references
}

fn assigned_goal<'a>(
    slot: TypedGoalSlot,
    inputs: &'a TypedRuleInputs<'_>,
) -> Result<&'a AssignedTypedGoal, Error> {
    match slot {
        TypedGoalSlot::Parent => Ok(inputs.parent),
        TypedGoalSlot::Child { index } => inputs
            .children
            .get(usize::from(index))
            .ok_or(Error::Synthesis),
    }
}

fn assigned_root<'a>(
    reference: &TypedTermRef,
    inputs: &'a TypedRuleInputs<'_>,
) -> Result<&'a AssignedCell<Fp, Fp>, Error> {
    assigned_goal(reference.goal, inputs)?
        .arguments
        .get(usize::from(reference.argument))
        .ok_or(Error::Synthesis)
}

fn ground_term<'a>(
    reference: &TypedTermRef,
    inputs: &'a TypedRuleInputs<'_>,
) -> Option<&'a GroundTerm> {
    let goal = match reference.goal {
        TypedGoalSlot::Parent => inputs.ground_parent,
        TypedGoalSlot::Child { index } => inputs.ground_children.get(usize::from(index))?,
    };
    goal.args.get(usize::from(reference.argument))
}

fn opening_for_reference(
    reference: &TypedTermRef,
    inputs: &TypedRuleInputs<'_>,
) -> Result<crate::typed_term::TypedTermOpening, Error> {
    if let Some(term) = ground_term(reference, inputs) {
        if let Ok(opening) = build_term_opening(term, reference.path.clone()) {
            return Ok(opening);
        }
    }
    build_dummy_term_opening(&reference.path).map_err(|_| Error::Synthesis)
}
