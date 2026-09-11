//! Algebraically constrained evaluator for the bounded legacy builtin layout.
//!
//! Every opcode, branch decision, arithmetic transition, comparison, and
//! division witness below is queried by a `create_gate` constraint. The input
//! remains the legacy four-predicate matrix; typed term tags must be constrained
//! by the caller when this chip is integrated into the main circuit.

use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};
use halo2curves::ff::{Field, PrimeField};

use crate::{
    chips::fact_check::u32_range_chip::{U32RangeChip, U32RangeConfig},
    utils_2::common_helpers::{MAX_ARITY, MAX_PRED_LIST, to_fp_value},
};

const REL_EQ: usize = 0;
const REL_IS: usize = 1;
const REL_LT: usize = 2;
const REL_LE: usize = 3;
const REL_GT: usize = 4;
const REL_GE: usize = 5;
const OP_ADD: usize = 0;
const OP_SUB: usize = 1;
const OP_MUL: usize = 2;
const OP_DIV: usize = 3;
const OP_PAD: usize = 4;

#[derive(Clone, Debug)]
pub struct SoundBuiltinConfig {
    branch: Column<Advice>,
    name: Column<Advice>,
    arity: Column<Advice>,
    selectors: [Column<Advice>; 6],
    work: [Column<Advice>; 4],
    q_relation: Selector,
    q_equality: Selector,
    q_instruction: Selector,
    q_first_instruction: Selector,
    q_arithmetic: Selector,
    q_comparison: Selector,
    q_final: Selector,
    range: U32RangeConfig,
}

#[derive(Clone, Debug)]
pub struct SoundBuiltinChip {
    config: SoundBuiltinConfig,
}

/// Cells of the fixed legacy parent row consumed by the evaluator.
pub struct LegacyBuiltinCells<'a> {
    pub names: &'a [AssignedCell<Fp, Fp>],
    pub arities: &'a [AssignedCell<Fp, Fp>],
    pub args: &'a [Vec<Vec<AssignedCell<Fp, Fp>>>],
}

#[derive(Clone)]
struct RelationCells {
    branch: AssignedCell<Fp, Fp>,
    selectors: [AssignedCell<Fp, Fp>; 6],
    numeric: AssignedCell<Fp, Fp>,
    comparison: AssignedCell<Fp, Fp>,
}

#[derive(Clone)]
struct InstructionCells {
    selectors: [AssignedCell<Fp, Fp>; 5],
    active: AssignedCell<Fp, Fp>,
    pad: AssignedCell<Fp, Fp>,
}

#[derive(Clone)]
struct ArithmeticCells {
    next: AssignedCell<Fp, Fp>,
    quotient: AssignedCell<Fp, Fp>,
    remainder: AssignedCell<Fp, Fp>,
    difference: AssignedCell<Fp, Fp>,
}

impl Chip<Fp> for SoundBuiltinChip {
    type Config = SoundBuiltinConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl SoundBuiltinChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> SoundBuiltinConfig {
        let branch = meta.advice_column();
        let name = meta.advice_column();
        let arity = meta.advice_column();
        let selectors = [(); 6].map(|_| meta.advice_column());
        let work = [(); 4].map(|_| meta.advice_column());
        let q_relation = meta.selector();
        let q_equality = meta.selector();
        let q_instruction = meta.selector();
        let q_first_instruction = meta.selector();
        let q_arithmetic = meta.selector();
        let q_comparison = meta.selector();
        let q_final = meta.selector();

        for column in [branch, name, arity]
            .into_iter()
            .chain(selectors)
            .chain(work)
        {
            meta.enable_equality(column);
        }

        let relation_constants = ["=", "is", "<", "<=", ">", ">="].map(to_fp_value);
        meta.create_gate("builtin: exact relation opcode", |meta| {
            let q = meta.query_selector(q_relation);
            let branch_expr = meta.query_advice(branch, Rotation::cur());
            let name_expr = meta.query_advice(name, Rotation::cur());
            let arity_expr = meta.query_advice(arity, Rotation::cur());
            let selected = selectors.map(|column| meta.query_advice(column, Rotation::cur()));
            let numeric = meta.query_advice(work[0], Rotation::cur());
            let comparison = meta.query_advice(work[1], Rotation::cur());
            let one = Expression::Constant(Fp::one());
            let selector_sum = selected
                .iter()
                .cloned()
                .fold(Expression::Constant(Fp::zero()), |sum, value| sum + value);
            let mut constraints = vec![
                q.clone() * branch_expr.clone() * (branch_expr.clone() - one.clone()),
                q.clone() * (selector_sum - branch_expr.clone()),
                q.clone() * branch_expr * (arity_expr - Expression::Constant(Fp::from(2))),
                q.clone()
                    * (numeric
                        - selected[REL_IS].clone()
                        - selected[REL_LT].clone()
                        - selected[REL_LE].clone()
                        - selected[REL_GT].clone()
                        - selected[REL_GE].clone()),
                q.clone()
                    * (comparison
                        - selected[REL_LT].clone()
                        - selected[REL_LE].clone()
                        - selected[REL_GT].clone()
                        - selected[REL_GE].clone()),
            ];
            for (selector, constant) in selected.iter().zip(relation_constants) {
                constraints.push(
                    q.clone()
                        * selector.clone()
                        * (name_expr.clone() - Expression::Constant(constant)),
                );
                constraints.push(q.clone() * selector.clone() * (selector.clone() - one.clone()));
            }
            constraints
        });

        meta.create_gate("builtin: ground equality slots", |meta| {
            let q = meta.query_selector(q_equality);
            let left = meta.query_advice(branch, Rotation::cur());
            let right = meta.query_advice(name, Rotation::cur());
            let enabled = meta.query_advice(arity, Rotation::cur());
            vec![q * enabled * (left - right)]
        });

        let operation_constants = [
            to_fp_value("+"),
            to_fp_value("-"),
            to_fp_value("*"),
            to_fp_value("div"),
            Fp::zero(),
        ];
        meta.create_gate("builtin: exact arithmetic instruction", |meta| {
            let q = meta.query_selector(q_instruction);
            let branch_expr = meta.query_advice(branch, Rotation::cur());
            let name_expr = meta.query_advice(name, Rotation::cur());
            let arity_expr = meta.query_advice(arity, Rotation::cur());
            let selected = std::array::from_fn::<_, 5, _>(|index| {
                meta.query_advice(selectors[index], Rotation::cur())
            });
            let is_relation = meta.query_advice(work[0], Rotation::cur());
            let previous_pad = meta.query_advice(work[1], Rotation::cur());
            let active = meta.query_advice(work[2], Rotation::cur());
            let one = Expression::Constant(Fp::one());
            let selector_sum = selected
                .iter()
                .cloned()
                .fold(Expression::Constant(Fp::zero()), |sum, value| sum + value);
            let active_sum = selected[OP_ADD].clone()
                + selected[OP_SUB].clone()
                + selected[OP_MUL].clone()
                + selected[OP_DIV].clone();
            let mut constraints = vec![
                q.clone() * (selector_sum - branch_expr),
                q.clone() * (active.clone() - active_sum),
                q.clone() * active.clone() * (one.clone() - is_relation),
                q.clone() * previous_pad * active.clone(),
                q.clone() * active * (arity_expr.clone() - one.clone()),
                q.clone() * selected[OP_PAD].clone() * arity_expr,
            ];
            for (selector, constant) in selected.iter().zip(operation_constants) {
                constraints.push(
                    q.clone()
                        * selector.clone()
                        * (name_expr.clone() - Expression::Constant(constant)),
                );
                constraints.push(q.clone() * selector.clone() * (selector.clone() - one.clone()));
            }
            constraints
        });

        meta.create_gate("builtin: first instruction has no preceding pad", |meta| {
            let q = meta.query_selector(q_first_instruction);
            let previous_pad = meta.query_advice(work[1], Rotation::cur());
            vec![q * previous_pad]
        });

        meta.create_gate("builtin: arithmetic transition", |meta| {
            let q_gate = meta.query_selector(q_arithmetic);
            let selected = std::array::from_fn::<_, 5, _>(|index| {
                meta.query_advice(selectors[index], Rotation::cur())
            });
            let accumulator = meta.query_advice(branch, Rotation::cur());
            let operand = meta.query_advice(name, Rotation::cur());
            let next = meta.query_advice(arity, Rotation::cur());
            let quotient = meta.query_advice(work[0], Rotation::cur());
            let remainder = meta.query_advice(work[1], Rotation::cur());
            let inverse = meta.query_advice(work[2], Rotation::cur());
            let difference = meta.query_advice(work[3], Rotation::cur());
            let one = Expression::Constant(Fp::one());
            vec![
                q_gate.clone()
                    * selected[OP_ADD].clone()
                    * (next.clone() - accumulator.clone() - operand.clone()),
                q_gate.clone()
                    * selected[OP_SUB].clone()
                    * (next.clone() - accumulator.clone() + operand.clone()),
                q_gate.clone()
                    * selected[OP_MUL].clone()
                    * (next.clone() - accumulator.clone() * operand.clone()),
                q_gate.clone() * selected[OP_DIV].clone() * (next.clone() - quotient.clone()),
                q_gate.clone()
                    * selected[OP_DIV].clone()
                    * (operand.clone() * inverse - one.clone()),
                q_gate.clone()
                    * selected[OP_DIV].clone()
                    * (accumulator.clone() - quotient * operand.clone() - remainder.clone()),
                q_gate.clone()
                    * selected[OP_DIV].clone()
                    * (operand - remainder - one - difference),
                q_gate * selected[OP_PAD].clone() * (next - accumulator),
            ]
        });

        meta.create_gate("builtin: bounded comparison", |meta| {
            let q = meta.query_selector(q_comparison);
            let selected = selectors.map(|column| meta.query_advice(column, Rotation::cur()));
            let left = meta.query_advice(branch, Rotation::cur());
            let right = meta.query_advice(name, Rotation::cur());
            let difference = meta.query_advice(arity, Rotation::cur());
            let one = Expression::Constant(Fp::one());
            vec![
                q.clone()
                    * selected[REL_LT].clone()
                    * (right.clone() - left.clone() - one.clone() - difference.clone()),
                q.clone()
                    * selected[REL_LE].clone()
                    * (right.clone() - left.clone() - difference.clone()),
                q.clone()
                    * selected[REL_GT].clone()
                    * (left.clone() - right.clone() - one - difference.clone()),
                q * selected[REL_GE].clone() * (left - right - difference),
            ]
        });

        meta.create_gate("builtin: is result", |meta| {
            let q = meta.query_selector(q_final);
            let left = meta.query_advice(branch, Rotation::cur());
            let accumulator = meta.query_advice(name, Rotation::cur());
            let is_relation = meta.query_advice(arity, Rotation::cur());
            vec![q * is_relation * (left - accumulator)]
        });

        let range =
            U32RangeChip::configure_with_columns(meta, selectors[5], work[3], work[2], work[1]);
        SoundBuiltinConfig {
            branch,
            name,
            arity,
            selectors,
            work,
            q_relation,
            q_equality,
            q_instruction,
            q_first_instruction,
            q_arithmetic,
            q_comparison,
            q_final,
            range,
        }
    }

    pub fn construct(config: SoundBuiltinConfig) -> Self {
        Self { config }
    }

    /// Enforces the selected legacy builtin and returns the constrained branch
    /// bit so the caller can bind it to the public local-step resolution tag.
    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        inputs: LegacyBuiltinCells<'_>,
        branch_hint: Value<Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        validate_shape(&inputs)?;
        let relation = self.assign_relation(
            layouter.namespace(|| "builtin relation selector"),
            &inputs.names[0],
            &inputs.arities[0],
            branch_hint,
        )?;
        let lhs = &inputs.args[0][0][0];
        let rhs = &inputs.args[0][1][0];
        let range = U32RangeChip::construct(self.config.range.clone());
        range.assign(
            layouter.namespace(|| "builtin lhs u32"),
            lhs,
            &relation.numeric,
        )?;
        range.assign(
            layouter.namespace(|| "builtin rhs u32"),
            rhs,
            &relation.numeric,
        )?;

        self.assign_ground_equality(
            layouter.namespace(|| "builtin ground equality"),
            &inputs.args[0][0],
            &inputs.args[0][1],
            &relation.selectors[REL_EQ],
        )?;

        let mut accumulator = rhs.clone();
        let mut previous_pad = None;
        for predicate in 1..MAX_PRED_LIST {
            let instruction = self.assign_instruction(
                layouter.namespace(|| format!("builtin instruction {predicate}")),
                &inputs.names[predicate],
                &inputs.arities[predicate],
                &relation.branch,
                &relation.selectors[REL_IS],
                previous_pad.as_ref(),
                predicate == 1,
            )?;
            let operand = &inputs.args[predicate][0][0];
            let arithmetic = self.assign_arithmetic(
                layouter.namespace(|| format!("builtin arithmetic {predicate}")),
                &accumulator,
                operand,
                &instruction.selectors,
            )?;
            range.assign(
                layouter.namespace(|| format!("operand {predicate} u32")),
                operand,
                &instruction.active,
            )?;
            range.assign(
                layouter.namespace(|| format!("accumulator {predicate} u32")),
                &arithmetic.next,
                &relation.selectors[REL_IS],
            )?;
            range.assign(
                layouter.namespace(|| format!("quotient {predicate} u32")),
                &arithmetic.quotient,
                &instruction.selectors[OP_DIV],
            )?;
            range.assign(
                layouter.namespace(|| format!("remainder {predicate} u32")),
                &arithmetic.remainder,
                &instruction.selectors[OP_DIV],
            )?;
            range.assign(
                layouter.namespace(|| format!("division difference {predicate} u32")),
                &arithmetic.difference,
                &instruction.selectors[OP_DIV],
            )?;
            accumulator = arithmetic.next;
            previous_pad = Some(instruction.pad);
        }

        self.assign_is_result(
            layouter.namespace(|| "builtin is final result"),
            lhs,
            &accumulator,
            &relation.selectors[REL_IS],
        )?;
        let comparison_difference = self.assign_comparison(
            layouter.namespace(|| "builtin comparison"),
            lhs,
            rhs,
            &relation.selectors,
        )?;
        range.assign(
            layouter.namespace(|| "comparison difference u32"),
            &comparison_difference,
            &relation.comparison,
        )?;
        Ok(relation.branch)
    }

    fn assign_relation(
        &self,
        mut layouter: impl Layouter<Fp>,
        name: &AssignedCell<Fp, Fp>,
        arity: &AssignedCell<Fp, Fp>,
        branch_hint: Value<Fp>,
    ) -> Result<RelationCells, Error> {
        let config = self.config();
        let constants = ["=", "is", "<", "<=", ">", ">="].map(to_fp_value);
        layouter.assign_region(
            || "relation opcode",
            |mut region| {
                config.q_relation.enable(&mut region, 0)?;
                let branch =
                    region.assign_advice(|| "builtin branch", config.branch, 0, || branch_hint)?;
                copy_cell(&mut region, "relation name", config.name, 0, name)?;
                copy_cell(&mut region, "relation arity", config.arity, 0, arity)?;
                let selector_values = constants.map(|constant| {
                    branch_hint
                        .zip(name.value().copied())
                        .map(|(branch, name)| {
                            if branch == Fp::one() && name == constant {
                                Fp::one()
                            } else {
                                Fp::zero()
                            }
                        })
                });
                let mut selector_cells = Vec::with_capacity(6);
                for (index, value) in selector_values.iter().enumerate() {
                    selector_cells.push(region.assign_advice(
                        || format!("relation selector[{index}]"),
                        config.selectors[index],
                        0,
                        || *value,
                    )?);
                }
                let numeric_value = sum_values(&[
                    selector_values[REL_IS],
                    selector_values[REL_LT],
                    selector_values[REL_LE],
                    selector_values[REL_GT],
                    selector_values[REL_GE],
                ]);
                let comparison_value = sum_values(&[
                    selector_values[REL_LT],
                    selector_values[REL_LE],
                    selector_values[REL_GT],
                    selector_values[REL_GE],
                ]);
                let numeric = region.assign_advice(
                    || "numeric builtin",
                    config.work[0],
                    0,
                    || numeric_value,
                )?;
                let comparison = region.assign_advice(
                    || "comparison builtin",
                    config.work[1],
                    0,
                    || comparison_value,
                )?;
                Ok(RelationCells {
                    branch,
                    selectors: selector_cells.try_into().map_err(|_| Error::Synthesis)?,
                    numeric,
                    comparison,
                })
            },
        )
    }

    fn assign_ground_equality(
        &self,
        mut layouter: impl Layouter<Fp>,
        left: &[AssignedCell<Fp, Fp>],
        right: &[AssignedCell<Fp, Fp>],
        enabled: &AssignedCell<Fp, Fp>,
    ) -> Result<(), Error> {
        if left.len() != MAX_PRED_LIST || right.len() != MAX_PRED_LIST {
            return Err(Error::Synthesis);
        }
        let config = self.config();
        layouter.assign_region(
            || "cell-wise legacy equality",
            |mut region| {
                for index in 0..MAX_PRED_LIST {
                    config.q_equality.enable(&mut region, index)?;
                    copy_cell(&mut region, "left", config.branch, index, &left[index])?;
                    copy_cell(&mut region, "right", config.name, index, &right[index])?;
                    copy_cell(
                        &mut region,
                        "equality enabled",
                        config.arity,
                        index,
                        enabled,
                    )?;
                }
                Ok(())
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn assign_instruction(
        &self,
        mut layouter: impl Layouter<Fp>,
        name: &AssignedCell<Fp, Fp>,
        arity: &AssignedCell<Fp, Fp>,
        branch: &AssignedCell<Fp, Fp>,
        is_relation: &AssignedCell<Fp, Fp>,
        previous_pad: Option<&AssignedCell<Fp, Fp>>,
        first: bool,
    ) -> Result<InstructionCells, Error> {
        let config = self.config();
        let constants = [
            to_fp_value("+"),
            to_fp_value("-"),
            to_fp_value("*"),
            to_fp_value("div"),
            Fp::zero(),
        ];
        layouter.assign_region(
            || "arithmetic opcode",
            |mut region| {
                config.q_instruction.enable(&mut region, 0)?;
                if first {
                    config.q_first_instruction.enable(&mut region, 0)?;
                }
                copy_cell(&mut region, "builtin branch", config.branch, 0, branch)?;
                copy_cell(&mut region, "instruction name", config.name, 0, name)?;
                copy_cell(&mut region, "instruction arity", config.arity, 0, arity)?;
                copy_cell(&mut region, "is relation", config.work[0], 0, is_relation)?;
                if let Some(previous_pad) = previous_pad {
                    copy_cell(&mut region, "previous pad", config.work[1], 0, previous_pad)?;
                } else {
                    region.assign_advice(
                        || "no previous pad",
                        config.work[1],
                        0,
                        || Value::known(Fp::zero()),
                    )?;
                }
                let selector_values = constants.map(|constant| {
                    branch
                        .value()
                        .copied()
                        .zip(name.value().copied())
                        .map(|(branch, name)| {
                            if branch == Fp::one() && name == constant {
                                Fp::one()
                            } else {
                                Fp::zero()
                            }
                        })
                });
                let mut selector_cells = Vec::with_capacity(5);
                for (index, value) in selector_values.iter().enumerate() {
                    selector_cells.push(region.assign_advice(
                        || format!("operation selector[{index}]"),
                        config.selectors[index],
                        0,
                        || *value,
                    )?);
                }
                let active_value = sum_values(&selector_values[..4]);
                let active = region.assign_advice(
                    || "active instruction",
                    config.work[2],
                    0,
                    || active_value,
                )?;
                let selectors: [AssignedCell<Fp, Fp>; 5] =
                    selector_cells.try_into().map_err(|_| Error::Synthesis)?;
                Ok(InstructionCells {
                    pad: selectors[OP_PAD].clone(),
                    selectors,
                    active,
                })
            },
        )
    }

    fn assign_arithmetic(
        &self,
        mut layouter: impl Layouter<Fp>,
        accumulator: &AssignedCell<Fp, Fp>,
        operand: &AssignedCell<Fp, Fp>,
        selectors: &[AssignedCell<Fp, Fp>; 5],
    ) -> Result<ArithmeticCells, Error> {
        let config = self.config();
        let quotient_value = accumulator
            .value()
            .copied()
            .zip(operand.value().copied())
            .map(|(accumulator, operand)| {
                let divisor = low_u64(operand);
                if divisor == 0 {
                    Fp::zero()
                } else {
                    Fp::from(low_u64(accumulator) / divisor)
                }
            });
        let remainder_value = accumulator
            .value()
            .copied()
            .zip(quotient_value)
            .zip(operand.value().copied())
            .map(|((accumulator, quotient), operand)| accumulator - quotient * operand);
        let inverse_value = operand
            .value()
            .map(|operand| Option::<Fp>::from(operand.invert()).unwrap_or(Fp::zero()));
        let difference_value = operand
            .value()
            .copied()
            .zip(remainder_value)
            .map(|(operand, remainder)| operand - remainder - Fp::one());
        let add = accumulator
            .value()
            .copied()
            .zip(operand.value().copied())
            .map(|(a, b)| a + b);
        let sub = accumulator
            .value()
            .copied()
            .zip(operand.value().copied())
            .map(|(a, b)| a - b);
        let mul = accumulator
            .value()
            .copied()
            .zip(operand.value().copied())
            .map(|(a, b)| a * b);
        let candidates = [add, sub, mul, quotient_value, accumulator.value().copied()];
        let mut next_value = Value::known(Fp::zero());
        for (selector, candidate) in selectors.iter().zip(candidates) {
            next_value = next_value
                .zip(selector.value().copied())
                .zip(candidate)
                .map(|((sum, selector), candidate)| sum + selector * candidate);
        }

        layouter.assign_region(
            || "constrained arithmetic transition",
            |mut region| {
                config.q_arithmetic.enable(&mut region, 0)?;
                for (index, selector) in selectors.iter().enumerate() {
                    copy_cell(
                        &mut region,
                        "operation selector",
                        config.selectors[index],
                        0,
                        selector,
                    )?;
                }
                copy_cell(&mut region, "accumulator", config.branch, 0, accumulator)?;
                copy_cell(&mut region, "operand", config.name, 0, operand)?;
                let next =
                    region.assign_advice(|| "next accumulator", config.arity, 0, || next_value)?;
                let quotient = region.assign_advice(
                    || "division quotient",
                    config.work[0],
                    0,
                    || quotient_value,
                )?;
                let remainder = region.assign_advice(
                    || "division remainder",
                    config.work[1],
                    0,
                    || remainder_value,
                )?;
                region.assign_advice(|| "divisor inverse", config.work[2], 0, || inverse_value)?;
                let difference = region.assign_advice(
                    || "divisor - remainder - 1",
                    config.work[3],
                    0,
                    || difference_value,
                )?;
                Ok(ArithmeticCells {
                    next,
                    quotient,
                    remainder,
                    difference,
                })
            },
        )
    }

    fn assign_is_result(
        &self,
        mut layouter: impl Layouter<Fp>,
        lhs: &AssignedCell<Fp, Fp>,
        accumulator: &AssignedCell<Fp, Fp>,
        is_relation: &AssignedCell<Fp, Fp>,
    ) -> Result<(), Error> {
        let config = self.config();
        layouter.assign_region(
            || "is equality",
            |mut region| {
                config.q_final.enable(&mut region, 0)?;
                copy_cell(&mut region, "lhs", config.branch, 0, lhs)?;
                copy_cell(&mut region, "accumulator", config.name, 0, accumulator)?;
                copy_cell(&mut region, "is selector", config.arity, 0, is_relation)?;
                Ok(())
            },
        )
    }

    fn assign_comparison(
        &self,
        mut layouter: impl Layouter<Fp>,
        left: &AssignedCell<Fp, Fp>,
        right: &AssignedCell<Fp, Fp>,
        selectors: &[AssignedCell<Fp, Fp>; 6],
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let difference_value = selectors[REL_LT]
            .value()
            .copied()
            .zip(selectors[REL_LE].value().copied())
            .zip(selectors[REL_GT].value().copied())
            .zip(selectors[REL_GE].value().copied())
            .zip(left.value().copied())
            .zip(right.value().copied())
            .map(
                |(((((less, less_equal), greater), greater_equal), left), right)| {
                    less * (right - left - Fp::one())
                        + less_equal * (right - left)
                        + greater * (left - right - Fp::one())
                        + greater_equal * (left - right)
                },
            );
        let config = self.config();
        layouter.assign_region(
            || "bounded comparison relation",
            |mut region| {
                config.q_comparison.enable(&mut region, 0)?;
                for (index, selector) in selectors.iter().enumerate() {
                    copy_cell(
                        &mut region,
                        "relation selector",
                        config.selectors[index],
                        0,
                        selector,
                    )?;
                }
                copy_cell(&mut region, "left", config.branch, 0, left)?;
                copy_cell(&mut region, "right", config.name, 0, right)?;
                region.assign_advice(
                    || "comparison difference",
                    config.arity,
                    0,
                    || difference_value,
                )
            },
        )
    }
}

fn copy_cell(
    region: &mut halo2_proofs::circuit::Region<'_, Fp>,
    annotation: &'static str,
    column: Column<Advice>,
    row: usize,
    source: &AssignedCell<Fp, Fp>,
) -> Result<AssignedCell<Fp, Fp>, Error> {
    let copied = region.assign_advice(|| annotation, column, row, || source.value().copied())?;
    region.constrain_equal(copied.cell(), source.cell())?;
    Ok(copied)
}

fn sum_values(values: &[Value<Fp>]) -> Value<Fp> {
    values
        .iter()
        .copied()
        .fold(Value::known(Fp::zero()), |sum, value| {
            sum.zip(value).map(|(sum, value)| sum + value)
        })
}

fn low_u64(value: Fp) -> u64 {
    let repr = value.to_repr();
    u64::from_le_bytes(
        repr.as_ref()[..8]
            .try_into()
            .expect("field repr has eight bytes"),
    )
}

fn validate_shape(inputs: &LegacyBuiltinCells<'_>) -> Result<(), Error> {
    if inputs.names.len() != MAX_PRED_LIST
        || inputs.arities.len() != MAX_PRED_LIST
        || inputs.args.len() != MAX_PRED_LIST
        || inputs.args.iter().any(|arguments| {
            arguments.len() != MAX_ARITY
                || arguments
                    .iter()
                    .any(|list_slots| list_slots.len() != MAX_PRED_LIST)
        })
    {
        return Err(Error::Synthesis);
    }
    Ok(())
}
