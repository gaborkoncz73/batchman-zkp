//! Public commitments for one bounded local step.
//!
//! Parent and child identities use the canonical typed-goal hash.  They are
//! therefore independent of the temporary legacy matrix projection and remain
//! identical when the same Prolog goal occurs as a parent in one proof and as
//! a child in another.

use anyhow::{Context, Result, bail};
use halo2_gadgets::poseidon::primitives::{ConstantLength, Hash as PoseidonHash, P128Pow5T3};
use halo2_proofs::pasta::Fp;
use halo2curves::ff::{Field, PrimeField};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

use crate::{
    circuit_witness::TypedGoalFp,
    data::{RuleTemplateFileFp, TermSideFp, UnificationInputFp},
    typed_policy::{TypedPolicyPlan, typed_policy_commitment},
    utils_2::{
        common_helpers::{MAX_ARITY, MAX_CHILDREN, MAX_FACTS_HASHES, MAX_PRED_LIST},
        off_circuit_poseidon::poseidon_hash_list_native,
    },
};

pub const POLICY_COMMITMENT_INSTANCE_ROW: usize = MAX_FACTS_HASHES;
pub const PARENT_GOAL_COMMITMENT_INSTANCE_ROW: usize = MAX_FACTS_HASHES + 1;
pub const ORDERED_CHILDREN_COMMITMENT_INSTANCE_ROW: usize = MAX_FACTS_HASHES + 2;
pub const RESOLUTION_TAG_INSTANCE_ROW: usize = MAX_FACTS_HASHES + 3;
pub const LOCAL_STEP_PUBLIC_INPUT_COUNT: usize = MAX_FACTS_HASHES + 4;

pub const CHILDREN_COMMITMENT_INPUTS: usize = 2 + MAX_CHILDREN;

const COMMITMENT_SCHEMA_VERSION: u32 = 3;
const COMMITMENT_DIALECT: &str = "batchman_typed_local_step_commitments_v3";

// These small protocol tags are field elements, not hashed source strings.
// ConstantLength also separates the 73-element goal hash from the 12-element
// ordered-children hash.
pub const CHILDREN_COMMITMENT_DOMAIN_TAG: u64 = 0x4241_5454_4348_4c01;
const POLICY_MAGIC: u64 = 0x4241_5443_484d_414e;
const POLICY_VERSION: u64 = 1;
const COMPOSITE_POLICY_MAGIC: u64 = 0x4241_5454_504f_4c43;
const COMPOSITE_POLICY_VERSION: u64 = 3;
const TAG_POLICY: u64 = 0x60;
const TAG_PREDICATE: u64 = 0x61;
const TAG_CLAUSE: u64 = 0x62;
const TAG_CHILD_ROW: u64 = 0x63;
const TAG_EQUALITY: u64 = 0x64;
const TAG_REF: u64 = 0x65;
const TAG_VALUE: u64 = 0x66;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalStepCommitments {
    pub policy: Fp,
    pub parent_goal: Fp,
    pub ordered_children: Fp,
    pub resolution_tag: Fp,
}

/// JSON-safe representation stored beside each independent proof.
///
/// Field elements use canonical unsigned decimal strings, matching the
/// project's existing public fact-hash format.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofCommitmentsV3 {
    pub schema_version: u32,
    pub dialect: String,
    pub policy_commitment: String,
    pub parent_goal_commitment: String,
    pub ordered_children_commitment: String,
    pub resolution_tag: String,
}

impl ProofCommitmentsV3 {
    pub fn from_fields(commitments: LocalStepCommitments) -> Self {
        Self {
            schema_version: COMMITMENT_SCHEMA_VERSION,
            dialect: COMMITMENT_DIALECT.to_owned(),
            policy_commitment: fp_to_decimal(commitments.policy),
            parent_goal_commitment: fp_to_decimal(commitments.parent_goal),
            ordered_children_commitment: fp_to_decimal(commitments.ordered_children),
            resolution_tag: fp_to_decimal(commitments.resolution_tag),
        }
    }

    pub fn to_fields(&self) -> Result<LocalStepCommitments> {
        if self.schema_version != COMMITMENT_SCHEMA_VERSION {
            bail!(
                "unsupported proof commitment schema version {}; expected {COMMITMENT_SCHEMA_VERSION}",
                self.schema_version
            );
        }
        if self.dialect != COMMITMENT_DIALECT {
            bail!(
                "unsupported proof commitment dialect {:?}; expected {COMMITMENT_DIALECT:?}",
                self.dialect
            );
        }

        Ok(LocalStepCommitments {
            policy: fp_from_decimal(&self.policy_commitment)
                .context("invalid policy_commitment")?,
            parent_goal: fp_from_decimal(&self.parent_goal_commitment)
                .context("invalid parent_goal_commitment")?,
            ordered_children: fp_from_decimal(&self.ordered_children_commitment)
                .context("invalid ordered_children_commitment")?,
            resolution_tag: fp_from_decimal(&self.resolution_tag)
                .context("invalid resolution_tag")?,
        })
    }
}

pub fn append_local_step_public_inputs(
    public_fact_hashes: &[Fp],
    commitments: &LocalStepCommitments,
) -> Vec<Fp> {
    let mut public_inputs = Vec::with_capacity(LOCAL_STEP_PUBLIC_INPUT_COUNT);
    public_inputs.extend_from_slice(public_fact_hashes);
    public_inputs.extend([
        commitments.policy,
        commitments.parent_goal,
        commitments.ordered_children,
        commitments.resolution_tag,
    ]);
    public_inputs
}

pub fn local_step_commitments(
    rules: &RuleTemplateFileFp,
    typed_policy: &TypedPolicyPlan,
    unif: &UnificationInputFp,
) -> Result<LocalStepCommitments> {
    validate_unification_shape(unif)?;
    validate_typed_statement(unif)?;
    let parent_goal = unif.typed.goal.hash();

    let child_hashes = unif
        .typed
        .children
        .iter()
        .map(TypedGoalFp::hash)
        .collect::<Vec<_>>();

    Ok(LocalStepCommitments {
        policy: circuit_policy_commitment(rules, typed_policy),
        parent_goal,
        ordered_children: ordered_children_commitment_from_hashes(&child_hashes)?,
        resolution_tag: unif.resolution.protocol_field(),
    })
}

/// Reconstructs the ordered-child commitment from child proof statements.
/// This is shared by local commitment creation and verifier-side tree
/// composition, preventing the two paths from drifting.
pub fn ordered_children_commitment_from_hashes(child_hashes: &[Fp]) -> Result<Fp> {
    if child_hashes.len() > MAX_CHILDREN {
        bail!(
            "ordered-child commitment has {} children, maximum is {MAX_CHILDREN}",
            child_hashes.len()
        );
    }
    let empty_goal_hash = TypedGoalFp::default().hash();
    let mut padded = [empty_goal_hash; MAX_CHILDREN];
    padded[..child_hashes.len()].copy_from_slice(child_hashes);

    let mut children_inputs = [Fp::ZERO; CHILDREN_COMMITMENT_INPUTS];
    children_inputs[0] = Fp::from(CHILDREN_COMMITMENT_DOMAIN_TAG);
    children_inputs[1] = Fp::from(child_hashes.len() as u64);
    children_inputs[2..].copy_from_slice(&padded);
    Ok(poseidon_hash_fixed_native(children_inputs))
}

/// Returns the canonical typed goal hash for node zero (the parent) or node
/// `1..=MAX_CHILDREN`. Missing child slots use the protocol's canonical empty
/// typed-goal hash so the circuit shape remains fixed.
pub fn goal_commitment_for_node(unif: &UnificationInputFp, node: usize) -> Result<Fp> {
    validate_unification_shape(unif)?;
    validate_typed_statement(unif)?;
    if node > MAX_CHILDREN {
        bail!("goal commitment node {node} exceeds maximum {MAX_CHILDREN}");
    }
    Ok(if node == 0 {
        unif.typed.goal.hash()
    } else {
        unif.typed
            .children
            .get(node - 1)
            .cloned()
            .unwrap_or_default()
            .hash()
    })
}

/// Canonical commitment to the complete representation understood by the
/// current circuit compiler.  It is a policy-specific fixed/VK parameter in
/// the circuit, so hashing it natively does not turn it into free witness data.
pub fn circuit_policy_commitment(rules: &RuleTemplateFileFp, typed_policy: &TypedPolicyPlan) -> Fp {
    poseidon_hash_list_native(&[
        Fp::from(COMPOSITE_POLICY_MAGIC),
        Fp::from(COMPOSITE_POLICY_VERSION),
        legacy_policy_commitment(rules),
        typed_policy_commitment(typed_policy),
    ])
}

fn legacy_policy_commitment(rules: &RuleTemplateFileFp) -> Fp {
    let mut tokens = vec![
        Fp::from(POLICY_MAGIC),
        Fp::from(POLICY_VERSION),
        Fp::from(TAG_POLICY),
        Fp::from(rules.predicates.len() as u64),
    ];

    for (predicate_index, predicate) in rules.predicates.iter().enumerate() {
        tokens.extend([
            Fp::from(TAG_PREDICATE),
            Fp::from(predicate_index as u64),
            predicate.name,
            predicate.arity,
            Fp::from(predicate.clauses.len() as u64),
        ]);

        for (clause_index, clause) in predicate.clauses.iter().enumerate() {
            tokens.extend([
                Fp::from(TAG_CLAUSE),
                Fp::from(clause_index as u64),
                Fp::from(clause.children.len() as u64),
            ]);
            for (row_index, row) in clause.children.iter().enumerate() {
                tokens.extend([
                    Fp::from(TAG_CHILD_ROW),
                    Fp::from(row_index as u64),
                    Fp::from(row.len() as u64),
                ]);
                for child in row {
                    tokens.extend([child.name, child.arity]);
                }
            }

            tokens.push(Fp::from(clause.equalities.len() as u64));
            for (equality_index, equality) in clause.equalities.iter().enumerate() {
                tokens.extend([Fp::from(TAG_EQUALITY), Fp::from(equality_index as u64)]);
                append_side_tokens(&mut tokens, &equality.left);
                append_side_tokens(&mut tokens, &equality.right);
            }
        }
    }

    poseidon_hash_list_native(&tokens)
}

pub fn poseidon_hash_fixed_native<const L: usize>(values: [Fp; L]) -> Fp {
    PoseidonHash::<Fp, P128Pow5T3, ConstantLength<L>, 3, 2>::init().hash(values)
}

fn append_side_tokens(tokens: &mut Vec<Fp>, side: &TermSideFp) {
    match side {
        TermSideFp::Ref(reference) => tokens.extend([
            Fp::from(TAG_REF),
            reference.children_node_list,
            reference.predicate,
            reference.arg,
            reference.list_index,
        ]),
        TermSideFp::Value(value) => tokens.extend([Fp::from(TAG_VALUE), *value]),
    }
}

fn validate_unification_shape(unif: &UnificationInputFp) -> Result<()> {
    if unif.goal_name.len() > MAX_PRED_LIST {
        bail!(
            "parent predicate list has {} entries, maximum is {MAX_PRED_LIST}",
            unif.goal_name.len()
        );
    }
    if unif.subtree_goals.len() > MAX_CHILDREN {
        bail!(
            "local step has {} child rows, maximum is {MAX_CHILDREN}",
            unif.subtree_goals.len()
        );
    }
    for (node_index, row) in std::iter::once(&unif.goal_name)
        .chain(unif.subtree_goals.iter())
        .enumerate()
    {
        if row.len() > MAX_PRED_LIST {
            bail!(
                "node {node_index} predicate list has {} entries, maximum is {MAX_PRED_LIST}",
                row.len()
            );
        }
        for (predicate_index, term) in row.iter().enumerate() {
            if term.args.len() > MAX_ARITY {
                bail!(
                    "node {node_index} predicate {predicate_index} has {} argument rows, maximum is {MAX_ARITY}",
                    term.args.len()
                );
            }
            for (argument_index, values) in term.args.iter().enumerate() {
                if values.len() > MAX_PRED_LIST {
                    bail!(
                        "node {node_index} predicate {predicate_index} argument {argument_index} has {} list slots, maximum is {MAX_PRED_LIST}",
                        values.len()
                    );
                }
            }
        }
    }
    Ok(())
}

fn validate_typed_statement(unif: &UnificationInputFp) -> Result<()> {
    let derived_parent =
        TypedGoalFp::from_ground_call(&unif.typed.ground_goal).map_err(anyhow::Error::new)?;
    if derived_parent != unif.typed.goal {
        bail!("typed parent field view does not match its canonical ground goal");
    }
    if unif.typed.children.len() != unif.typed.ground_children.len() {
        bail!(
            "typed local step has {} child field views but {} canonical ground children",
            unif.typed.children.len(),
            unif.typed.ground_children.len()
        );
    }
    if unif.typed.children.len() > MAX_CHILDREN {
        bail!(
            "typed local step has {} semantic children, maximum is {MAX_CHILDREN}",
            unif.typed.children.len()
        );
    }
    for (index, (fields, ground)) in unif
        .typed
        .children
        .iter()
        .zip(&unif.typed.ground_children)
        .enumerate()
    {
        let derived = TypedGoalFp::from_ground_call(ground).map_err(anyhow::Error::new)?;
        if &derived != fields {
            bail!("typed child {index} field view does not match its canonical ground goal");
        }
    }
    Ok(())
}

fn fp_to_decimal(value: Fp) -> String {
    BigUint::from_bytes_le(value.to_repr().as_ref()).to_str_radix(10)
}

fn fp_from_decimal(decimal: &str) -> Result<Fp> {
    let Some(integer) = BigUint::parse_bytes(decimal.as_bytes(), 10) else {
        bail!("value is not an unsigned decimal integer");
    };
    if integer.to_str_radix(10) != decimal {
        bail!("value is not in canonical decimal form");
    }
    let bytes = integer.to_bytes_le();
    if bytes.len() > 32 {
        bail!("value does not fit the field representation");
    }
    let mut repr = [0_u8; 32];
    repr[..bytes.len()].copy_from_slice(&bytes);
    Option::<Fp>::from(Fp::from_repr(repr)).context("value is not a canonical field element")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        circuit_witness::TypedLocalStepFp,
        data::{ChildSigFp, ClauseTemplateFp, EqualityFp, PredicateTemplateFp, TermRefFp},
        policy_ir::PredicateIndicator,
        typed_policy::{TypedCandidateId, TypedClausePlan, TypedPolicyPlan, TypedPredicatePlan},
        typed_term::symbol_to_field,
        utils_2::common_helpers::to_fp_value,
        witness_ir::{GroundCall, GroundTerm},
    };

    fn ground_goal(name: &str, argument: &str) -> GroundCall {
        GroundCall {
            predicate: PredicateIndicator {
                name: name.to_owned(),
                arity: 1,
            },
            args: vec![GroundTerm::Atom {
                value: argument.to_owned(),
            }],
        }
    }

    fn typed_step(parent: GroundCall, children: Vec<GroundCall>) -> TypedLocalStepFp {
        TypedLocalStepFp {
            goal: TypedGoalFp::from_ground_call(&parent).unwrap(),
            children: children
                .iter()
                .map(|child| TypedGoalFp::from_ground_call(child).unwrap())
                .collect(),
            ground_goal: parent,
            ground_children: children,
        }
    }

    fn unification(parent: GroundCall, children: Vec<GroundCall>) -> UnificationInputFp {
        UnificationInputFp {
            typed: typed_step(parent, children),
            ..UnificationInputFp::default()
        }
    }

    fn empty_typed_policy() -> TypedPolicyPlan {
        TypedPolicyPlan {
            predicates: Vec::new(),
        }
    }

    #[test]
    fn proof_commitments_use_canonical_decimal_fields() {
        let fields = LocalStepCommitments {
            policy: Fp::from(1),
            parent_goal: Fp::from(2),
            ordered_children: Fp::from(3),
            resolution_tag: Fp::from(4),
        };
        let encoded = ProofCommitmentsV3::from_fields(fields);
        assert_eq!(encoded.to_fields().unwrap(), fields);
        assert_eq!(encoded.policy_commitment, "1");
    }

    #[test]
    fn proof_commitments_reject_legacy_or_malformed_v3_metadata() {
        let fields = LocalStepCommitments {
            policy: Fp::from(1),
            parent_goal: Fp::from(2),
            ordered_children: Fp::from(3),
            resolution_tag: Fp::from(1),
        };
        let valid = ProofCommitmentsV3::from_fields(fields);

        let mut legacy = valid.clone();
        legacy.schema_version = 1;
        assert!(legacy.to_fields().is_err());

        let mut wrong_dialect = valid.clone();
        wrong_dialect.dialect = "batchman_legacy_local_step_commitments_v1".to_owned();
        assert!(wrong_dialect.to_fields().is_err());

        let mut noncanonical = valid.clone();
        noncanonical.resolution_tag = "01".to_owned();
        assert!(noncanonical.to_fields().is_err());

        let mut missing_resolution = serde_json::to_value(valid).unwrap();
        missing_resolution
            .as_object_mut()
            .unwrap()
            .remove("resolution_tag");
        assert!(serde_json::from_value::<ProofCommitmentsV3>(missing_resolution).is_err());
    }

    #[test]
    fn goal_commitment_is_common_for_parent_and_child_rows() {
        let child = ground_goal("q", "a");
        let parent_step = unification(ground_goal("p", "a"), vec![child.clone()]);
        let child_step = unification(child, Vec::new());

        assert_eq!(
            goal_commitment_for_node(&parent_step, 1).unwrap(),
            goal_commitment_for_node(&child_step, 0).unwrap()
        );
    }

    #[test]
    fn ordered_children_commitment_binds_order_count_and_arguments() {
        let rules = RuleTemplateFileFp {
            predicates: Vec::new(),
        };
        let typed_policy = empty_typed_policy();
        let step = |children| unification(ground_goal("p", "root"), children);
        let original = local_step_commitments(
            &rules,
            &typed_policy,
            &step(vec![ground_goal("q", "a"), ground_goal("r", "b")]),
        )
        .unwrap()
        .ordered_children;

        for changed in [
            step(vec![ground_goal("r", "b"), ground_goal("q", "a")]),
            step(vec![ground_goal("q", "a")]),
            step(vec![ground_goal("q", "changed"), ground_goal("r", "b")]),
        ] {
            assert_ne!(
                original,
                local_step_commitments(&rules, &typed_policy, &changed)
                    .unwrap()
                    .ordered_children
            );
        }
    }

    #[test]
    fn policy_commitment_covers_literals_and_child_order() {
        let reference = TermSideFp::Ref(TermRefFp {
            children_node_list: Fp::ZERO,
            predicate: Fp::ZERO,
            arg: Fp::ZERO,
            list_index: Fp::ZERO,
        });
        let policy = |literal: &str, children: [&str; 2]| RuleTemplateFileFp {
            predicates: vec![PredicateTemplateFp {
                name: to_fp_value("p"),
                arity: Fp::ONE,
                clauses: vec![ClauseTemplateFp {
                    children: children
                        .into_iter()
                        .map(|name| {
                            vec![ChildSigFp {
                                name: to_fp_value(name),
                                arity: Fp::ONE,
                            }]
                        })
                        .collect(),
                    equalities: vec![EqualityFp {
                        left: reference.clone(),
                        right: TermSideFp::Value(to_fp_value(literal)),
                    }],
                }],
            }],
        };

        let typed_policy = empty_typed_policy();
        let original = circuit_policy_commitment(&policy("a", ["q", "r"]), &typed_policy);
        assert_ne!(
            original,
            circuit_policy_commitment(&policy("b", ["q", "r"]), &typed_policy)
        );
        assert_ne!(
            original,
            circuit_policy_commitment(&policy("a", ["r", "q"]), &typed_policy)
        );

        let typed_variant = TypedPolicyPlan {
            predicates: vec![TypedPredicatePlan {
                name: symbol_to_field("p"),
                arity: 1,
                candidates: vec![TypedClausePlan {
                    id: TypedCandidateId {
                        source_clause_ordinal: 0,
                        branch_ordinal: 0,
                    },
                    children: Vec::new(),
                    equalities: Vec::new(),
                    patterns: Vec::new(),
                }],
            }],
        };
        assert_ne!(
            original,
            circuit_policy_commitment(&policy("a", ["q", "r"]), &typed_variant),
            "the composite policy identity must bind the typed compiler plan"
        );
    }
}
