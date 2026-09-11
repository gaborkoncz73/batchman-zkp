//! Type-preserving bridge for the legacy builtin arithmetic core.
//!
//! `SoundBuiltinChip` remains the bounded u32 evaluator.  This chip makes its
//! scalar/opcode cells authoritative for the same canonical typed goal that is
//! publicly committed by `UnificationCircuit`.  Every node and instruction
//! slot is assigned in a fixed shape; relation and instruction selectors only
//! gate algebraic links and never Rust control flow.

use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};
use halo2curves::ff::Field;

use crate::{
    chips::typed_term_chip::{AssignedHashedNode, AssignedTypedGoal, TypedTermChip},
    typed_term::{MAX_TYPED_COMPOUND_ARITY, TypedNodePreimage, hash_ground_term, symbol_to_field},
    utils_2::common_helpers::{MAX_ARITY, MAX_PRED_LIST, to_fp_value},
    witness_ir::{GroundCall, GroundTerm},
};

const REL_EQ: usize = 0;
const REL_IS: usize = 1;
const REL_LT: usize = 2;
const REL_LE: usize = 3;
const REL_GT: usize = 4;
const REL_GE: usize = 5;
const RELATION_COUNT: usize = 6;

const OP_ADD: usize = 0;
const OP_SUB: usize = 1;
const OP_MUL: usize = 2;
const OP_DIV: usize = 3;
const OP_PAD: usize = 4;
const OP_COUNT: usize = 5;
const MAX_INSTRUCTIONS: usize = MAX_PRED_LIST - 1;

#[derive(Clone, Debug)]
pub struct TypedBuiltinConfig {
    value: Column<Advice>,
    q_relation: Selector,
    q_integer_link: Selector,
    q_integer_payload: Selector,
    q_instruction: Selector,
    q_final: Selector,
}

#[derive(Clone, Debug)]
pub struct TypedBuiltinChip {
    config: TypedBuiltinConfig,
}

pub struct TypedBuiltinLegacyCells<'a> {
    pub names: &'a [AssignedCell<Fp, Fp>],
    pub arities: &'a [AssignedCell<Fp, Fp>],
    pub args: &'a [Vec<Vec<AssignedCell<Fp, Fp>>>],
}

struct RelationCells {
    selectors: [AssignedCell<Fp, Fp>; RELATION_COUNT],
    numeric: AssignedCell<Fp, Fp>,
    comparison: AssignedCell<Fp, Fp>,
}

struct InstructionCells {
    active: AssignedCell<Fp, Fp>,
    next_digest: AssignedCell<Fp, Fp>,
}

#[derive(Clone)]
struct TraceStep {
    functor: String,
    operand: GroundTerm,
    expression: GroundTerm,
}

impl TypedBuiltinChip {
    /// Reuses the typed-term vertical advice column. Regions do not overlap,
    /// while avoiding another committed advice column solely for bridge rows.
    pub fn configure(meta: &mut ConstraintSystem<Fp>, value: Column<Advice>) -> TypedBuiltinConfig {
        meta.enable_equality(value);
        let q_relation = meta.selector();
        let q_integer_link = meta.selector();
        let q_integer_payload = meta.selector();
        let q_instruction = meta.selector();
        let q_final = meta.selector();

        // row 0..14: branch, legacy name/arity, typed predicate/arity,
        // six relation selectors, two argument roots, numeric, comparison.
        meta.create_gate("typed builtin relation identity", |meta| {
            let q = meta.query_selector(q_relation);
            let mut at = |rotation| meta.query_advice(value, Rotation(rotation));
            let branch = at(0);
            let legacy_name = at(1);
            let legacy_arity = at(2);
            let typed_predicate = at(3);
            let typed_arity = at(4);
            let selectors = [at(5), at(6), at(7), at(8), at(9), at(10)];
            let argument_0 = at(11);
            let argument_1 = at(12);
            let numeric = at(13);
            let comparison = at(14);
            let one = Expression::Constant(Fp::ONE);
            let legacy_constants = ["=", "is", "<", "<=", ">", ">="].map(to_fp_value);
            let typed_constants = ["=", "is", "<", "=<", ">", ">="].map(symbol_to_field);

            let selector_sum = selectors
                .iter()
                .cloned()
                .fold(Expression::Constant(Fp::ZERO), |sum, bit| sum + bit);
            let mut constraints = vec![
                selector_sum - branch.clone(),
                branch.clone() * (legacy_arity - Expression::Constant(Fp::from(2))),
                branch.clone() * (typed_arity - Expression::Constant(Fp::from(2))),
                numeric
                    - selectors[REL_IS].clone()
                    - selectors[REL_LT].clone()
                    - selectors[REL_LE].clone()
                    - selectors[REL_GT].clone()
                    - selectors[REL_GE].clone(),
                comparison
                    - selectors[REL_LT].clone()
                    - selectors[REL_LE].clone()
                    - selectors[REL_GT].clone()
                    - selectors[REL_GE].clone(),
                selectors[REL_EQ].clone() * (argument_0 - argument_1),
            ];
            for index in 0..RELATION_COUNT {
                constraints
                    .push(selectors[index].clone() * (selectors[index].clone() - one.clone()));
                constraints.push(
                    selectors[index].clone()
                        * (legacy_name.clone() - Expression::Constant(legacy_constants[index])),
                );
                constraints.push(
                    selectors[index].clone()
                        * (typed_predicate.clone() - Expression::Constant(typed_constants[index])),
                );
            }
            constraints
                .into_iter()
                .map(|constraint| q.clone() * constraint)
                .collect::<Vec<_>>()
        });

        // active, integer payload, legacy scalar, integer digest, goal root.
        meta.create_gate("typed builtin integer root and payload", |meta| {
            let q = meta.query_selector(q_integer_link);
            let active = meta.query_advice(value, Rotation::cur());
            let payload = meta.query_advice(value, Rotation::next());
            let scalar = meta.query_advice(value, Rotation(2));
            let digest = meta.query_advice(value, Rotation(3));
            let root = meta.query_advice(value, Rotation(4));
            vec![
                q.clone() * active.clone() * (payload - scalar),
                q * active * (digest - root),
            ]
        });

        // active, integer payload, legacy scalar. Used by the initial `is`
        // value and every instruction operand; their digests are linked by the
        // expression chain instead of directly to a goal argument root.
        meta.create_gate("typed builtin integer payload", |meta| {
            let q = meta.query_selector(q_integer_payload);
            let active = meta.query_advice(value, Rotation::cur());
            let payload = meta.query_advice(value, Rotation::next());
            let scalar = meta.query_advice(value, Rotation(2));
            vec![q * active * (payload - scalar)]
        });

        // row 0..16: is-selector, legacy name/arity, typed functor/arity,
        // previous/child0, operand/child1, op digest, next digest, five op
        // selectors, active.
        meta.create_gate("typed builtin expression instruction", |meta| {
            let q = meta.query_selector(q_instruction);
            let mut at = |rotation| meta.query_advice(value, Rotation(rotation));
            let is_relation = at(0);
            let legacy_name = at(1);
            let legacy_arity = at(2);
            let typed_functor = at(3);
            let typed_arity = at(4);
            let previous_digest = at(5);
            let child_0 = at(6);
            let operand_digest = at(7);
            let child_1 = at(8);
            let operation_digest = at(9);
            let next_digest = at(10);
            let selectors = [at(11), at(12), at(13), at(14), at(15)];
            let active = at(16);
            let one = Expression::Constant(Fp::ONE);
            let legacy_constants = ["+", "-", "*", "div"].map(to_fp_value);
            let typed_constants = ["+", "-", "*", "div"].map(symbol_to_field);
            let selector_sum = selectors
                .iter()
                .cloned()
                .fold(Expression::Constant(Fp::ZERO), |sum, bit| sum + bit);
            let active_sum = selectors[..4]
                .iter()
                .cloned()
                .fold(Expression::Constant(Fp::ZERO), |sum, bit| sum + bit);

            let mut constraints = vec![
                selector_sum - is_relation,
                active.clone() - active_sum,
                active.clone() * (legacy_arity.clone() - one.clone()),
                active.clone() * (typed_arity - Expression::Constant(Fp::from(2))),
                active.clone() * (child_0 - previous_digest.clone()),
                active.clone() * (child_1 - operand_digest),
                next_digest
                    - active.clone() * operation_digest
                    - selectors[OP_PAD].clone() * previous_digest,
                selectors[OP_PAD].clone() * legacy_name.clone(),
                selectors[OP_PAD].clone() * legacy_arity,
            ];
            for selector in &selectors {
                constraints.push(selector.clone() * (selector.clone() - one.clone()));
            }
            for index in 0..4 {
                constraints.push(
                    selectors[index].clone()
                        * (legacy_name.clone() - Expression::Constant(legacy_constants[index])),
                );
                constraints.push(
                    selectors[index].clone()
                        * (typed_functor.clone() - Expression::Constant(typed_constants[index])),
                );
            }
            constraints
                .into_iter()
                .map(|constraint| q.clone() * constraint)
                .collect::<Vec<_>>()
        });

        meta.create_gate("typed builtin final expression root", |meta| {
            let q = meta.query_selector(q_final);
            let is_relation = meta.query_advice(value, Rotation::cur());
            let expression_digest = meta.query_advice(value, Rotation::next());
            let argument_root = meta.query_advice(value, Rotation(2));
            vec![q * is_relation * (expression_digest - argument_root)]
        });

        TypedBuiltinConfig {
            value,
            q_relation,
            q_integer_link,
            q_integer_payload,
            q_instruction,
            q_final,
        }
    }

    pub fn construct(config: TypedBuiltinConfig) -> Self {
        Self { config }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        typed_term: &TypedTermChip,
        typed_goal: &AssignedTypedGoal,
        ground_goal: &GroundCall,
        legacy: TypedBuiltinLegacyCells<'_>,
        builtin_branch: &AssignedCell<Fp, Fp>,
    ) -> Result<(), Error> {
        validate_legacy_shape(&legacy)?;
        let relation = self.assign_relation(
            layouter.namespace(|| "typed builtin relation"),
            typed_goal,
            ground_goal,
            &legacy,
            builtin_branch,
        )?;

        let mut trace = Vec::new();
        let initial = ground_goal
            .args
            .get(1)
            .map(|term| arithmetic_left_spine(term, &mut trace))
            .unwrap_or_else(|| GroundTerm::Integer { value: 0 });
        let lhs_node = integer_preimage(ground_goal.args.first());
        let initial_node = integer_preimage(Some(&initial));
        let assigned_lhs = typed_term.assign_and_hash_node(
            layouter.namespace(|| "typed builtin lhs integer"),
            &lhs_node,
        )?;
        let assigned_initial = typed_term.assign_and_hash_node(
            layouter.namespace(|| "typed builtin initial/rhs integer"),
            &initial_node,
        )?;

        self.assign_integer_link(
            layouter.namespace(|| "typed builtin lhs root"),
            &relation.numeric,
            &assigned_lhs,
            &legacy.args[0][0][0],
            &typed_goal.arguments[0],
        )?;
        self.assign_integer_link(
            layouter.namespace(|| "typed builtin comparison rhs root"),
            &relation.comparison,
            &assigned_initial,
            &legacy.args[0][1][0],
            &typed_goal.arguments[1],
        )?;
        self.assign_integer_payload(
            layouter.namespace(|| "typed builtin is initial payload"),
            &relation.selectors[REL_IS],
            &assigned_initial,
            &legacy.args[0][1][0],
        )?;

        let mut current_digest = assigned_initial.digest.clone();
        for instruction in 0..MAX_INSTRUCTIONS {
            let step = trace.get(instruction);
            let operand_preimage = integer_preimage(step.map(|step| &step.operand));
            let operation_preimage = operation_preimage(step);
            let operand = typed_term.assign_and_hash_node(
                layouter.namespace(|| format!("typed builtin operand {instruction}")),
                &operand_preimage,
            )?;
            let operation = typed_term.assign_and_hash_node(
                layouter.namespace(|| format!("typed builtin operation {instruction}")),
                &operation_preimage,
            )?;
            let assigned_instruction = self.assign_instruction(
                layouter.namespace(|| format!("typed builtin instruction {instruction}")),
                &relation.selectors[REL_IS],
                legacy.names.get(instruction + 1).ok_or(Error::Synthesis)?,
                legacy
                    .arities
                    .get(instruction + 1)
                    .ok_or(Error::Synthesis)?,
                &current_digest,
                &operand,
                &operation,
                step.and_then(|step| operation_index(&step.functor)),
            )?;
            self.assign_integer_payload(
                layouter.namespace(|| format!("typed builtin operand payload {instruction}")),
                &assigned_instruction.active,
                &operand,
                &legacy.args[instruction + 1][0][0],
            )?;
            current_digest = assigned_instruction.next_digest;
        }

        self.assign_final_root(
            layouter.namespace(|| "typed builtin final expression"),
            &relation.selectors[REL_IS],
            &current_digest,
            &typed_goal.arguments[1],
        )
    }

    fn assign_relation(
        &self,
        mut layouter: impl Layouter<Fp>,
        typed_goal: &AssignedTypedGoal,
        ground_goal: &GroundCall,
        legacy: &TypedBuiltinLegacyCells<'_>,
        branch: &AssignedCell<Fp, Fp>,
    ) -> Result<RelationCells, Error> {
        let selected = relation_index(&ground_goal.predicate.name, ground_goal.predicate.arity);
        layouter.assign_region(
            || "typed/legacy relation identity",
            |mut region| {
                self.config.q_relation.enable(&mut region, 0)?;
                copy(&mut region, self.config.value, 0, "builtin branch", branch)?;
                copy(
                    &mut region,
                    self.config.value,
                    1,
                    "legacy relation name",
                    &legacy.names[0],
                )?;
                copy(
                    &mut region,
                    self.config.value,
                    2,
                    "legacy relation arity",
                    &legacy.arities[0],
                )?;
                copy(
                    &mut region,
                    self.config.value,
                    3,
                    "typed relation predicate",
                    &typed_goal.predicate,
                )?;
                copy(
                    &mut region,
                    self.config.value,
                    4,
                    "typed relation arity",
                    &typed_goal.arity,
                )?;

                let mut selectors = Vec::with_capacity(RELATION_COUNT);
                let mut selector_values = Vec::with_capacity(RELATION_COUNT);
                for index in 0..RELATION_COUNT {
                    let value = branch.value().map(|branch| {
                        if selected == Some(index) {
                            *branch
                        } else {
                            Fp::ZERO
                        }
                    });
                    selector_values.push(value);
                    selectors.push(region.assign_advice(
                        || format!("typed relation selector {index}"),
                        self.config.value,
                        5 + index,
                        || value,
                    )?);
                }
                copy(
                    &mut region,
                    self.config.value,
                    11,
                    "typed argument 0 root",
                    &typed_goal.arguments[0],
                )?;
                copy(
                    &mut region,
                    self.config.value,
                    12,
                    "typed argument 1 root",
                    &typed_goal.arguments[1],
                )?;
                let numeric_value = sum_values(&selector_values[REL_IS..]);
                let comparison_value = sum_values(&selector_values[REL_LT..]);
                let numeric = region.assign_advice(
                    || "typed numeric relation",
                    self.config.value,
                    13,
                    || numeric_value,
                )?;
                let comparison = region.assign_advice(
                    || "typed comparison relation",
                    self.config.value,
                    14,
                    || comparison_value,
                )?;
                Ok(RelationCells {
                    selectors: selectors.try_into().map_err(|_| Error::Synthesis)?,
                    numeric,
                    comparison,
                })
            },
        )
    }

    fn assign_integer_link(
        &self,
        mut layouter: impl Layouter<Fp>,
        active: &AssignedCell<Fp, Fp>,
        integer: &AssignedHashedNode,
        scalar: &AssignedCell<Fp, Fp>,
        root: &AssignedCell<Fp, Fp>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "typed integer linked to scalar and root",
            |mut region| {
                self.config.q_integer_link.enable(&mut region, 0)?;
                copy(&mut region, self.config.value, 0, "active", active)?;
                copy(
                    &mut region,
                    self.config.value,
                    1,
                    "integer payload",
                    &integer.preimage.payload,
                )?;
                copy(&mut region, self.config.value, 2, "legacy scalar", scalar)?;
                copy(
                    &mut region,
                    self.config.value,
                    3,
                    "integer digest",
                    &integer.digest,
                )?;
                copy(&mut region, self.config.value, 4, "argument root", root)?;
                Ok(())
            },
        )
    }

    fn assign_integer_payload(
        &self,
        mut layouter: impl Layouter<Fp>,
        active: &AssignedCell<Fp, Fp>,
        integer: &AssignedHashedNode,
        scalar: &AssignedCell<Fp, Fp>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "typed integer linked to scalar",
            |mut region| {
                self.config.q_integer_payload.enable(&mut region, 0)?;
                copy(&mut region, self.config.value, 0, "active", active)?;
                copy(
                    &mut region,
                    self.config.value,
                    1,
                    "integer payload",
                    &integer.preimage.payload,
                )?;
                copy(&mut region, self.config.value, 2, "legacy scalar", scalar)?;
                Ok(())
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn assign_instruction(
        &self,
        mut layouter: impl Layouter<Fp>,
        is_relation: &AssignedCell<Fp, Fp>,
        legacy_name: &AssignedCell<Fp, Fp>,
        legacy_arity: &AssignedCell<Fp, Fp>,
        previous_digest: &AssignedCell<Fp, Fp>,
        operand: &AssignedHashedNode,
        operation: &AssignedHashedNode,
        operation_kind: Option<usize>,
    ) -> Result<InstructionCells, Error> {
        layouter.assign_region(
            || "typed expression instruction",
            |mut region| {
                self.config.q_instruction.enable(&mut region, 0)?;
                copy(
                    &mut region,
                    self.config.value,
                    0,
                    "is selector",
                    is_relation,
                )?;
                copy(
                    &mut region,
                    self.config.value,
                    1,
                    "legacy operation name",
                    legacy_name,
                )?;
                copy(
                    &mut region,
                    self.config.value,
                    2,
                    "legacy operation arity",
                    legacy_arity,
                )?;
                copy(
                    &mut region,
                    self.config.value,
                    3,
                    "typed operation functor",
                    &operation.preimage.payload,
                )?;
                copy(
                    &mut region,
                    self.config.value,
                    4,
                    "typed operation arity",
                    &operation.preimage.arity,
                )?;
                copy(
                    &mut region,
                    self.config.value,
                    5,
                    "previous expression digest",
                    previous_digest,
                )?;
                copy(
                    &mut region,
                    self.config.value,
                    6,
                    "operation child 0",
                    &operation.preimage.children[0],
                )?;
                copy(
                    &mut region,
                    self.config.value,
                    7,
                    "operand digest",
                    &operand.digest,
                )?;
                copy(
                    &mut region,
                    self.config.value,
                    8,
                    "operation child 1",
                    &operation.preimage.children[1],
                )?;
                copy(
                    &mut region,
                    self.config.value,
                    9,
                    "operation digest",
                    &operation.digest,
                )?;

                let selected = operation_kind.unwrap_or(OP_PAD);
                let mut selector_values = Vec::with_capacity(OP_COUNT);
                let mut selector_cells = Vec::with_capacity(OP_COUNT);
                for index in 0..OP_COUNT {
                    let value = is_relation.value().map(|is_relation| {
                        if selected == index {
                            *is_relation
                        } else {
                            Fp::ZERO
                        }
                    });
                    selector_values.push(value);
                    selector_cells.push(region.assign_advice(
                        || format!("typed operation selector {index}"),
                        self.config.value,
                        11 + index,
                        || value,
                    )?);
                }
                let active_value = sum_values(&selector_values[..4]);
                let active = region.assign_advice(
                    || "active typed instruction",
                    self.config.value,
                    16,
                    || active_value,
                )?;
                let next_value = active_value
                    .zip(operation.digest.value().copied())
                    .zip(selector_values[OP_PAD])
                    .zip(previous_digest.value().copied())
                    .map(|(((active, operation), pad), previous)| {
                        active * operation + pad * previous
                    });
                let next_digest = region.assign_advice(
                    || "next typed expression digest",
                    self.config.value,
                    10,
                    || next_value,
                )?;
                Ok(InstructionCells {
                    active,
                    next_digest,
                })
            },
        )
    }

    fn assign_final_root(
        &self,
        mut layouter: impl Layouter<Fp>,
        is_relation: &AssignedCell<Fp, Fp>,
        expression_digest: &AssignedCell<Fp, Fp>,
        argument_root: &AssignedCell<Fp, Fp>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "typed final expression root",
            |mut region| {
                self.config.q_final.enable(&mut region, 0)?;
                copy(
                    &mut region,
                    self.config.value,
                    0,
                    "is selector",
                    is_relation,
                )?;
                copy(
                    &mut region,
                    self.config.value,
                    1,
                    "expression digest",
                    expression_digest,
                )?;
                copy(
                    &mut region,
                    self.config.value,
                    2,
                    "typed argument 1 root",
                    argument_root,
                )?;
                Ok(())
            },
        )
    }
}

fn validate_legacy_shape(inputs: &TypedBuiltinLegacyCells<'_>) -> Result<(), Error> {
    if inputs.names.len() != MAX_PRED_LIST
        || inputs.arities.len() != MAX_PRED_LIST
        || inputs.args.len() != MAX_PRED_LIST
        || inputs.args.iter().any(|args| {
            args.len() != MAX_ARITY || args.iter().any(|row| row.len() != MAX_PRED_LIST)
        })
    {
        return Err(Error::Synthesis);
    }
    Ok(())
}

fn relation_index(name: &str, arity: usize) -> Option<usize> {
    if arity != 2 {
        return None;
    }
    match name {
        "=" => Some(REL_EQ),
        "is" => Some(REL_IS),
        "<" => Some(REL_LT),
        "=<" => Some(REL_LE),
        ">" => Some(REL_GT),
        ">=" => Some(REL_GE),
        _ => None,
    }
}

fn operation_index(name: &str) -> Option<usize> {
    match name {
        "+" => Some(OP_ADD),
        "-" => Some(OP_SUB),
        "*" => Some(OP_MUL),
        "div" => Some(OP_DIV),
        _ => None,
    }
}

fn integer_preimage(term: Option<&GroundTerm>) -> TypedNodePreimage {
    match term {
        Some(GroundTerm::Integer { value }) => TypedNodePreimage::Integer { value: *value },
        _ => TypedNodePreimage::Integer { value: 0 },
    }
}

fn arithmetic_left_spine(term: &GroundTerm, steps: &mut Vec<TraceStep>) -> GroundTerm {
    match term {
        GroundTerm::Compound { functor, args }
            if operation_index(functor).is_some() && args.len() == 2 =>
        {
            let initial = arithmetic_left_spine(&args[0], steps);
            steps.push(TraceStep {
                functor: functor.clone(),
                operand: args[1].clone(),
                expression: term.clone(),
            });
            initial
        }
        _ => term.clone(),
    }
}

fn operation_preimage(step: Option<&TraceStep>) -> TypedNodePreimage {
    if let Some(step) = step {
        if let Ok(node) = TypedNodePreimage::from_ground_term(&step.expression) {
            return node;
        }
    }
    let zero = GroundTerm::Integer { value: 0 };
    let zero_hash = hash_ground_term(&zero).expect("the fixed integer-zero node is canonical");
    let mut children = [Fp::ZERO; MAX_TYPED_COMPOUND_ARITY];
    children[0] = zero_hash;
    children[1] = zero_hash;
    TypedNodePreimage::Compound {
        functor: symbol_to_field("+"),
        arity: 2,
        children,
    }
}

fn copy(
    region: &mut halo2_proofs::circuit::Region<'_, Fp>,
    column: Column<Advice>,
    row: usize,
    annotation: &'static str,
    source: &AssignedCell<Fp, Fp>,
) -> Result<AssignedCell<Fp, Fp>, Error> {
    source.copy_advice(|| annotation, region, column, row)
}

fn sum_values(values: &[Value<Fp>]) -> Value<Fp> {
    values.iter().fold(Value::known(Fp::ZERO), |sum, value| {
        sum.zip(*value).map(|(sum, value)| sum + value)
    })
}
