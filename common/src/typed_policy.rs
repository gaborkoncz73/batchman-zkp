//! Circuit-ready, type-preserving policy matching plan.
//!
//! The semantic `PolicyFile` remains the only persisted policy format.  This
//! module describes the deterministic in-memory plan used to match a selected
//! clause against canonical typed parent and child goals.  In particular, a
//! reference starts at one goal argument root and then follows a `TermPath`;
//! it has no dependency on the legacy predicate-list or list-slot projection.

use halo2_proofs::pasta::Fp;
use halo2curves::ff::Field;

use crate::typed_term::{MAX_TYPED_COMPOUND_ARITY, TermPath, TermPathStep, TermTag};
use crate::utils_2::off_circuit_poseidon::poseidon_hash_list_native;

/// Version of the canonical typed-policy token stream.
pub const TYPED_POLICY_COMMITMENT_SCHEMA_VERSION: u32 = 1;

/// Domain separator for the typed-policy commitment protocol (`BATTPOL\x01`).
pub const TYPED_POLICY_COMMITMENT_DOMAIN_TAG: u64 = 0x4241_5454_504f_4c01;

const TAG_POLICY: u64 = 0x100;
const TAG_PREDICATES: u64 = 0x101;
const TAG_PREDICATE: u64 = 0x102;
const TAG_CANDIDATES: u64 = 0x103;
const TAG_CANDIDATE: u64 = 0x104;
const TAG_CANDIDATE_ID: u64 = 0x105;
const TAG_CHILDREN: u64 = 0x106;
const TAG_CHILD: u64 = 0x107;
const TAG_EQUALITIES: u64 = 0x108;
const TAG_EQUALITY: u64 = 0x109;
const TAG_LEFT: u64 = 0x10a;
const TAG_RIGHT: u64 = 0x10b;
const TAG_PATTERNS: u64 = 0x10c;
const TAG_EXACT: u64 = 0x10d;
const TAG_SHAPE: u64 = 0x10e;
const TAG_TERM_REF: u64 = 0x10f;
const TAG_PARENT: u64 = 0x110;
const TAG_CHILD_SLOT: u64 = 0x111;
const TAG_PATH: u64 = 0x112;
const TAG_PATH_STEP: u64 = 0x113;
const TAG_HEAD: u64 = 0x114;
const TAG_TAIL: u64 = 0x115;
const TAG_ARGUMENT: u64 = 0x116;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedPolicyPlan {
    pub predicates: Vec<TypedPredicatePlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedPredicatePlan {
    pub name: Fp,
    pub arity: u8,
    pub candidates: Vec<TypedClausePlan>,
}

/// Stable identity of one branch produced from a source clause.
///
/// Branches are numbered in the deterministic left-to-right order used by the
/// compiler when it distributes conjunctions over disjunctions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypedCandidateId {
    pub source_clause_ordinal: usize,
    pub branch_ordinal: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedClausePlan {
    pub id: TypedCandidateId,
    /// Signatures of the original body goals.  An `is/2` expression therefore
    /// remains one typed child; it is not legacy accumulator bytecode.
    pub children: Vec<TypedGoalSignature>,
    pub equalities: Vec<TypedEquality>,
    pub patterns: Vec<TypedPatternCheck>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypedGoalSignature {
    pub name: Fp,
    pub arity: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypedGoalSlot {
    Parent,
    Child { index: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypedTermRef {
    pub goal: TypedGoalSlot,
    pub argument: u8,
    pub path: TermPath,
}

impl TypedTermRef {
    pub fn root(goal: TypedGoalSlot, argument: usize) -> Result<Self, String> {
        let argument = u8::try_from(argument)
            .map_err(|_| format!("typed argument index {argument} does not fit u8"))?;
        Ok(Self {
            goal,
            argument,
            path: TermPath::root(),
        })
    }

    pub fn pushed(&self, step: TermPathStep) -> Self {
        Self {
            goal: self.goal,
            argument: self.argument,
            path: self.path.pushed(step),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedEquality {
    pub left: TypedTermRef,
    pub right: TypedTermRef,
}

/// Minimal recursive decomposition of one source pattern.
///
/// A maximal variable-free subtree is represented by one `Exact` digest.  A
/// constructor containing variables cannot be hashed ahead of time, so `Shape`
/// binds its tag, functor (payload), and exact arity while its children are
/// handled recursively.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedPatternCheck {
    Exact {
        at: TypedTermRef,
        digest: Fp,
    },
    Shape {
        at: TypedTermRef,
        tag: TermTag,
        payload: Fp,
        arity: u8,
    },
}

impl TypedPolicyPlan {
    /// Checks compiler-plan invariants without consulting prover-controlled
    /// data.  This is intentionally stricter than merely having valid indices:
    /// candidate IDs and all term paths must be canonical as well.
    pub fn validate(&self) -> Result<(), String> {
        for (predicate_index, predicate) in self.predicates.iter().enumerate() {
            let predicate_path = format!("typed.predicates[{predicate_index}]");
            if usize::from(predicate.arity) > MAX_TYPED_COMPOUND_ARITY {
                return Err(format!(
                    "{predicate_path}.arity {} exceeds typed maximum {MAX_TYPED_COMPOUND_ARITY}",
                    predicate.arity
                ));
            }
            if predicate.candidates.is_empty() {
                return Err(format!("{predicate_path} has no candidates"));
            }

            let mut expected_source = 0;
            let mut expected_branch = 0;
            for (candidate_index, candidate) in predicate.candidates.iter().enumerate() {
                let candidate_path = format!("{predicate_path}.candidates[{candidate_index}]");
                if candidate.id.source_clause_ordinal != expected_source
                    || candidate.id.branch_ordinal != expected_branch
                {
                    return Err(format!(
                        "{candidate_path} has non-canonical ID ({}, {}), expected ({expected_source}, {expected_branch})",
                        candidate.id.source_clause_ordinal, candidate.id.branch_ordinal
                    ));
                }

                let next = predicate
                    .candidates
                    .get(candidate_index + 1)
                    .map(|next| next.id);
                if matches!(next, Some(next) if next.source_clause_ordinal == expected_source) {
                    expected_branch += 1;
                } else {
                    expected_source += 1;
                    expected_branch = 0;
                }

                for (child_index, child) in candidate.children.iter().enumerate() {
                    if usize::from(child.arity) > MAX_TYPED_COMPOUND_ARITY {
                        return Err(format!(
                            "{candidate_path}.children[{child_index}].arity {} exceeds typed maximum {MAX_TYPED_COMPOUND_ARITY}",
                            child.arity
                        ));
                    }
                }
                for (index, equality) in candidate.equalities.iter().enumerate() {
                    validate_ref(
                        &equality.left,
                        predicate,
                        candidate,
                        &format!("{candidate_path}.equalities[{index}].left"),
                    )?;
                    validate_ref(
                        &equality.right,
                        predicate,
                        candidate,
                        &format!("{candidate_path}.equalities[{index}].right"),
                    )?;
                }
                for (index, pattern) in candidate.patterns.iter().enumerate() {
                    let pattern_path = format!("{candidate_path}.patterns[{index}]");
                    match pattern {
                        TypedPatternCheck::Exact { at, .. } => {
                            validate_ref(at, predicate, candidate, &format!("{pattern_path}.at"))?;
                        }
                        TypedPatternCheck::Shape {
                            at,
                            tag,
                            payload,
                            arity,
                        } => {
                            validate_ref(at, predicate, candidate, &format!("{pattern_path}.at"))?;
                            match tag {
                                TermTag::Cons
                                    if *payload == Fp::ZERO && usize::from(*arity) == 2 => {}
                                TermTag::Compound
                                    if (1..=MAX_TYPED_COMPOUND_ARITY)
                                        .contains(&usize::from(*arity)) => {}
                                _ => {
                                    return Err(format!(
                                        "{pattern_path} is not a canonical variable-bearing constructor shape"
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// Produces the canonical, self-delimiting field-token stream committed by
/// [`typed_policy_commitment`].
///
/// Every variable-length collection carries its item count, every record has a
/// tag and field-length prefix, and every ordered element carries its index.
/// Consequently no concatenation ambiguity can reinterpret a child as a
/// candidate, one path step as two fields, or the same children in a different
/// order. The function is total so it can also diagnose mutations of an
/// otherwise invalid plan; production compiler output is separately checked by
/// [`TypedPolicyPlan::validate`].
pub fn typed_policy_commitment_tokens(plan: &TypedPolicyPlan) -> Vec<Fp> {
    let mut predicate_items = Vec::new();
    for (predicate_index, predicate) in plan.predicates.iter().enumerate() {
        append_predicate(&mut predicate_items, predicate_index, predicate);
    }
    let mut predicates = vec![fp_usize(plan.predicates.len())];
    predicates.extend(predicate_items);

    let mut policy_payload = Vec::new();
    append_record(&mut policy_payload, TAG_PREDICATES, predicates);

    let mut policy = Vec::new();
    append_record(&mut policy, TAG_POLICY, policy_payload);

    let mut tokens = vec![
        Fp::from(TYPED_POLICY_COMMITMENT_DOMAIN_TAG),
        Fp::from(TYPED_POLICY_COMMITMENT_SCHEMA_VERSION as u64),
        fp_usize(policy.len()),
    ];
    tokens.extend(policy);
    tokens
}

/// Native Poseidon commitment to every field of a typed compiler plan.
pub fn typed_policy_commitment(plan: &TypedPolicyPlan) -> Fp {
    poseidon_hash_list_native(&typed_policy_commitment_tokens(plan))
}

fn append_predicate(output: &mut Vec<Fp>, predicate_index: usize, predicate: &TypedPredicatePlan) {
    let mut candidates = vec![fp_usize(predicate.candidates.len())];
    for (candidate_index, candidate) in predicate.candidates.iter().enumerate() {
        append_candidate(&mut candidates, candidate_index, candidate);
    }

    let mut payload = vec![
        fp_usize(predicate_index),
        predicate.name,
        Fp::from(predicate.arity as u64),
    ];
    append_record(&mut payload, TAG_CANDIDATES, candidates);
    append_record(output, TAG_PREDICATE, payload);
}

fn append_candidate(output: &mut Vec<Fp>, candidate_index: usize, candidate: &TypedClausePlan) {
    let mut id = vec![
        fp_usize(candidate.id.source_clause_ordinal),
        fp_usize(candidate.id.branch_ordinal),
    ];
    let mut payload = vec![fp_usize(candidate_index)];
    append_record(&mut payload, TAG_CANDIDATE_ID, std::mem::take(&mut id));

    let mut children = vec![fp_usize(candidate.children.len())];
    for (child_index, child) in candidate.children.iter().enumerate() {
        append_record(
            &mut children,
            TAG_CHILD,
            vec![
                fp_usize(child_index),
                child.name,
                Fp::from(child.arity as u64),
            ],
        );
    }
    append_record(&mut payload, TAG_CHILDREN, children);

    let mut equalities = vec![fp_usize(candidate.equalities.len())];
    for (equality_index, equality) in candidate.equalities.iter().enumerate() {
        let mut equality_payload = vec![fp_usize(equality_index)];
        let mut left = Vec::new();
        append_term_ref(&mut left, &equality.left);
        append_record(&mut equality_payload, TAG_LEFT, left);
        let mut right = Vec::new();
        append_term_ref(&mut right, &equality.right);
        append_record(&mut equality_payload, TAG_RIGHT, right);
        append_record(&mut equalities, TAG_EQUALITY, equality_payload);
    }
    append_record(&mut payload, TAG_EQUALITIES, equalities);

    let mut patterns = vec![fp_usize(candidate.patterns.len())];
    for (pattern_index, pattern) in candidate.patterns.iter().enumerate() {
        append_pattern(&mut patterns, pattern_index, pattern);
    }
    append_record(&mut payload, TAG_PATTERNS, patterns);

    append_record(output, TAG_CANDIDATE, payload);
}

fn append_pattern(output: &mut Vec<Fp>, pattern_index: usize, pattern: &TypedPatternCheck) {
    match pattern {
        TypedPatternCheck::Exact { at, digest } => {
            let mut payload = vec![fp_usize(pattern_index)];
            append_term_ref(&mut payload, at);
            payload.push(*digest);
            append_record(output, TAG_EXACT, payload);
        }
        TypedPatternCheck::Shape {
            at,
            tag,
            payload: node_payload,
            arity,
        } => {
            let mut payload = vec![fp_usize(pattern_index)];
            append_term_ref(&mut payload, at);
            payload.extend([
                Fp::from(tag.as_u64()),
                *node_payload,
                Fp::from(*arity as u64),
            ]);
            append_record(output, TAG_SHAPE, payload);
        }
    }
}

fn append_term_ref(output: &mut Vec<Fp>, reference: &TypedTermRef) {
    let mut payload = Vec::new();
    match reference.goal {
        TypedGoalSlot::Parent => append_record(&mut payload, TAG_PARENT, Vec::new()),
        TypedGoalSlot::Child { index } => {
            append_record(&mut payload, TAG_CHILD_SLOT, vec![Fp::from(index as u64)])
        }
    }
    payload.push(Fp::from(reference.argument as u64));

    let mut path = vec![fp_usize(reference.path.steps.len())];
    for (step_index, step) in reference.path.steps.iter().enumerate() {
        let mut step_payload = vec![fp_usize(step_index)];
        match step {
            TermPathStep::Head => append_record(&mut step_payload, TAG_HEAD, Vec::new()),
            TermPathStep::Tail => append_record(&mut step_payload, TAG_TAIL, Vec::new()),
            TermPathStep::Argument { index } => append_record(
                &mut step_payload,
                TAG_ARGUMENT,
                vec![Fp::from(*index as u64)],
            ),
        }
        append_record(&mut path, TAG_PATH_STEP, step_payload);
    }
    append_record(&mut payload, TAG_PATH, path);
    append_record(output, TAG_TERM_REF, payload);
}

fn append_record(output: &mut Vec<Fp>, tag: u64, payload: Vec<Fp>) {
    output.extend([Fp::from(tag), fp_usize(payload.len())]);
    output.extend(payload);
}

fn fp_usize(value: usize) -> Fp {
    Fp::from(value as u64)
}

fn validate_ref(
    reference: &TypedTermRef,
    predicate: &TypedPredicatePlan,
    candidate: &TypedClausePlan,
    path: &str,
) -> Result<(), String> {
    let arity = match reference.goal {
        TypedGoalSlot::Parent => predicate.arity,
        TypedGoalSlot::Child { index } => {
            candidate
                .children
                .get(usize::from(index))
                .ok_or_else(|| format!("{path}.goal points to missing child {index}"))?
                .arity
        }
    };
    if reference.argument >= arity {
        return Err(format!(
            "{path}.argument {} is outside goal arity {arity}",
            reference.argument
        ));
    }
    for (index, step) in reference.path.steps.iter().enumerate() {
        if matches!(step, TermPathStep::Argument { index } if usize::from(*index) >= MAX_TYPED_COMPOUND_ARITY)
        {
            return Err(format!(
                "{path}.path.steps[{index}] exceeds typed compound arity"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod commitment_tests {
    use super::*;

    fn reference(goal: TypedGoalSlot, argument: u8, steps: Vec<TermPathStep>) -> TypedTermRef {
        TypedTermRef {
            goal,
            argument,
            path: TermPath::new(steps),
        }
    }

    fn sample_plan() -> TypedPolicyPlan {
        TypedPolicyPlan {
            predicates: vec![TypedPredicatePlan {
                name: Fp::from(10),
                arity: 2,
                candidates: vec![TypedClausePlan {
                    id: TypedCandidateId {
                        source_clause_ordinal: 0,
                        branch_ordinal: 0,
                    },
                    children: vec![
                        TypedGoalSignature {
                            name: Fp::from(20),
                            arity: 2,
                        },
                        TypedGoalSignature {
                            name: Fp::from(21),
                            arity: 1,
                        },
                    ],
                    equalities: vec![TypedEquality {
                        left: reference(
                            TypedGoalSlot::Parent,
                            0,
                            vec![TermPathStep::Head, TermPathStep::Argument { index: 1 }],
                        ),
                        right: reference(
                            TypedGoalSlot::Child { index: 0 },
                            1,
                            vec![TermPathStep::Tail],
                        ),
                    }],
                    patterns: vec![
                        TypedPatternCheck::Exact {
                            at: reference(
                                TypedGoalSlot::Child { index: 1 },
                                0,
                                vec![TermPathStep::Argument { index: 0 }],
                            ),
                            digest: Fp::from(30),
                        },
                        TypedPatternCheck::Shape {
                            at: reference(TypedGoalSlot::Parent, 1, Vec::new()),
                            tag: TermTag::Compound,
                            payload: Fp::from(31),
                            arity: 2,
                        },
                    ],
                }],
            }],
        }
    }

    fn assert_mutation_is_bound(original: &TypedPolicyPlan, mutated: TypedPolicyPlan) {
        assert_ne!(
            typed_policy_commitment_tokens(original),
            typed_policy_commitment_tokens(&mutated)
        );
        assert_ne!(
            typed_policy_commitment(original),
            typed_policy_commitment(&mutated)
        );
    }

    #[test]
    fn typed_policy_tokens_are_deterministic_versioned_and_self_delimiting() {
        let plan = sample_plan();
        let first = typed_policy_commitment_tokens(&plan);
        let second = typed_policy_commitment_tokens(&plan);
        assert_eq!(first, second);
        assert_eq!(first[0], Fp::from(TYPED_POLICY_COMMITMENT_DOMAIN_TAG));
        assert_eq!(
            first[1],
            Fp::from(TYPED_POLICY_COMMITMENT_SCHEMA_VERSION as u64)
        );
        assert_eq!(first[2], fp_usize(first.len() - 3));
        assert_eq!(
            typed_policy_commitment(&plan),
            poseidon_hash_list_native(&first)
        );
    }

    #[test]
    fn commitment_binds_predicate_candidate_and_ordered_child_fields() {
        let original = sample_plan();

        let mut mutated = original.clone();
        mutated.predicates[0].name += Fp::ONE;
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].arity = 1;
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].candidates[0].id.source_clause_ordinal = 1;
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].candidates[0].id.branch_ordinal = 1;
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].candidates[0].children[0].name += Fp::ONE;
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].candidates[0].children[0].arity = 1;
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].candidates[0].children.swap(0, 1);
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].candidates[0].children.pop();
        assert_mutation_is_bound(&original, mutated);
    }

    #[test]
    fn commitment_binds_both_equality_refs_and_every_path_step_variant() {
        let original = sample_plan();

        let mut mutated = original.clone();
        let equality = &mut mutated.predicates[0].candidates[0].equalities[0];
        std::mem::swap(&mut equality.left, &mut equality.right);
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].candidates[0].equalities[0].left.goal =
            TypedGoalSlot::Child { index: 1 };
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].candidates[0].equalities[0].right.goal =
            TypedGoalSlot::Child { index: 1 };
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].candidates[0].equalities[0]
            .left
            .argument = 1;
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].candidates[0].equalities[0]
            .left
            .path
            .steps[0] = TermPathStep::Tail;
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].candidates[0].equalities[0]
            .right
            .path
            .steps[0] = TermPathStep::Head;
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].candidates[0].equalities[0]
            .left
            .path
            .steps[1] = TermPathStep::Argument { index: 0 };
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].candidates[0].equalities[0]
            .left
            .path
            .steps
            .swap(0, 1);
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].candidates[0].equalities.clear();
        assert_mutation_is_bound(&original, mutated);
    }

    #[test]
    fn commitment_binds_exact_and_shape_patterns_and_their_order() {
        let original = sample_plan();

        let mut mutated = original.clone();
        let TypedPatternCheck::Exact { digest, .. } =
            &mut mutated.predicates[0].candidates[0].patterns[0]
        else {
            unreachable!()
        };
        *digest += Fp::ONE;
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        let TypedPatternCheck::Exact { at, .. } =
            &mut mutated.predicates[0].candidates[0].patterns[0]
        else {
            unreachable!()
        };
        at.path.steps[0] = TermPathStep::Argument { index: 1 };
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        let TypedPatternCheck::Shape { tag, .. } =
            &mut mutated.predicates[0].candidates[0].patterns[1]
        else {
            unreachable!()
        };
        *tag = TermTag::Cons;
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        let TypedPatternCheck::Shape { payload, .. } =
            &mut mutated.predicates[0].candidates[0].patterns[1]
        else {
            unreachable!()
        };
        *payload += Fp::ONE;
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        let TypedPatternCheck::Shape { arity, .. } =
            &mut mutated.predicates[0].candidates[0].patterns[1]
        else {
            unreachable!()
        };
        *arity = 1;
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].candidates[0].patterns.swap(0, 1);
        assert_mutation_is_bound(&original, mutated);

        let mut mutated = original.clone();
        mutated.predicates[0].candidates[0].patterns.pop();
        assert_mutation_is_bound(&original, mutated);
    }

    #[test]
    fn commitment_binds_predicate_candidate_and_equality_collection_order() {
        let mut two_candidates = sample_plan();
        let mut second_candidate = two_candidates.predicates[0].candidates[0].clone();
        second_candidate.id.branch_ordinal = 1;
        second_candidate.children[0].name += Fp::ONE;
        two_candidates.predicates[0]
            .candidates
            .push(second_candidate);
        let mut reordered_candidates = two_candidates.clone();
        reordered_candidates.predicates[0].candidates.swap(0, 1);
        assert_mutation_is_bound(&two_candidates, reordered_candidates);

        let mut two_predicates = sample_plan();
        let mut second_predicate = two_predicates.predicates[0].clone();
        second_predicate.name += Fp::ONE;
        two_predicates.predicates.push(second_predicate);
        let mut reordered_predicates = two_predicates.clone();
        reordered_predicates.predicates.swap(0, 1);
        assert_mutation_is_bound(&two_predicates, reordered_predicates);

        let mut two_equalities = sample_plan();
        let mut second_equality = two_equalities.predicates[0].candidates[0].equalities[0].clone();
        second_equality.right.argument = 0;
        two_equalities.predicates[0].candidates[0]
            .equalities
            .push(second_equality);
        let mut reordered_equalities = two_equalities.clone();
        reordered_equalities.predicates[0].candidates[0]
            .equalities
            .swap(0, 1);
        assert_mutation_is_bound(&two_equalities, reordered_equalities);
    }
}
