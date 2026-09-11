//! Challenge-compressed matrix/dot-product check for typed rule variables.
//!
//! For one policy candidate, let every compiler-produced variable equality be
//! one row of an incidence matrix `A`: the referenced left occurrence has
//! coefficient `+1`, the right occurrence has coefficient `-1`, and all other
//! entries are zero.  If `W` is the vector of authenticated typed-term
//! digests, the equality residual vector is `A * W`.
//!
//! The chip submits the selected candidate's fixed-width residual tuple
//!
//! ```text
//! flag * (left[0] - right[0]), ..., flag * (left[m-1] - right[m-1])
//! ```
//!
//! to a one-row zero lookup. Halo2 compresses the tuple internally as a dot
//! product with transcript challenge powers:
//!
//! ```text
//! flag * <(1, theta, ..., theta^(m-1)), A * W> = 0.
//! ```
//!
//! `theta` is sampled after the advice columns containing `W` have been
//! committed. A malicious prover therefore cannot choose unequal occurrences
//! that cancel for a known coefficient vector. For at most `MAX_EQUALITIES`
//! rows, a non-zero residual vector passes with probability at most
//! `(MAX_EQUALITIES - 1) / |Fp|`, in addition to the proof system's soundness
//! error.
//!
//! This is not the historical gate-free `DotExistsChip`: every operand is
//! copied from an authenticated typed opening, and the already-constrained
//! candidate flag is copied through Halo2's permutation argument. Booleanity
//! remains the caller's responsibility so it is not duplicated here.

use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Selector, TableColumn},
    poly::Rotation,
};
use halo2curves::ff::Field;

use crate::utils_2::common_helpers::MAX_EQUALITIES;

#[derive(Clone, Debug)]
pub struct TypedVariableDotConfig {
    flag: Column<Advice>,
    left: Column<Advice>,
    right: Column<Advice>,
    q_candidate: Selector,
    zero_table: TableColumn,
}

#[derive(Clone, Debug)]
pub struct TypedVariableDotChip {
    config: TypedVariableDotConfig,
}

#[derive(Clone, Debug)]
pub struct TypedVariableEquality {
    pub left: AssignedCell<Fp, Fp>,
    pub right: AssignedCell<Fp, Fp>,
}

impl TypedVariableEquality {
    pub fn new(left: AssignedCell<Fp, Fp>, right: AssignedCell<Fp, Fp>) -> Self {
        Self { left, right }
    }
}

#[derive(Clone, Debug)]
pub struct TypedVariableCandidate {
    /// A clause-selection bit already constrained by `TypedRuleChip`.
    pub flag: AssignedCell<Fp, Fp>,
    /// The compiler emits a minimal `n - 1` spanning set per source variable.
    pub equalities: Vec<TypedVariableEquality>,
}

impl TypedVariableDotChip {
    /// Shares the typed rule chip's three advice columns. The dot-product
    /// argument therefore adds no advice commitment of its own.
    pub fn configure_with_columns(
        meta: &mut ConstraintSystem<Fp>,
        flag: Column<Advice>,
        left: Column<Advice>,
        right: Column<Advice>,
    ) -> TypedVariableDotConfig {
        for column in [flag, left, right] {
            meta.enable_equality(column);
        }
        let q_candidate = meta.complex_selector();
        let zero_table = meta.lookup_table_column();

        // Repeating the same table column deliberately defines the singleton
        // tuple {(0, ..., 0)}. The real proof system challenge-compresses both
        // sides with theta only after committing the advice columns.
        meta.lookup(|meta| {
            let q = meta.query_selector(q_candidate);
            let selected = meta.query_advice(flag, Rotation::cur());
            (0..MAX_EQUALITIES)
                .map(|index| {
                    let rotation = Rotation(index as i32);
                    let left = meta.query_advice(left, rotation);
                    let right = meta.query_advice(right, rotation);
                    (q.clone() * selected.clone() * (left - right), zero_table)
                })
                .collect()
        });

        TypedVariableDotConfig {
            flag,
            left,
            right,
            q_candidate,
            zero_table,
        }
    }

    pub fn construct(config: TypedVariableDotConfig) -> Self {
        Self { config }
    }

    /// Loads the singleton zero tuple. Halo2 pads the table with its row-zero
    /// value, so assigning one zero is sufficient.
    pub fn load_zero_table(&self, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        layouter.assign_table(
            || "typed variable dot-product zero table",
            |mut table| {
                table.assign_cell(
                    || "zero",
                    self.config.zero_table,
                    0,
                    || Value::known(Fp::ZERO),
                )?;
                Ok(())
            },
        )
    }

    /// Assigns one fixed-width matrix block for every actual policy candidate.
    /// Padding is deterministic and contributes zero to the compressed dot
    /// product. No candidate/equality count depends on the private witness.
    pub fn assign_candidates(
        &self,
        mut layouter: impl Layouter<Fp>,
        candidates: &[TypedVariableCandidate],
    ) -> Result<(), Error> {
        if candidates
            .iter()
            .any(|candidate| candidate.equalities.len() > MAX_EQUALITIES)
        {
            return Err(Error::Synthesis);
        }

        layouter.assign_region(
            || "typed variable matrix dot products",
            |mut region| {
                for (candidate_index, candidate) in candidates.iter().enumerate() {
                    let block_start = candidate_index * MAX_EQUALITIES;
                    self.config.q_candidate.enable(&mut region, block_start)?;
                    candidate.flag.copy_advice(
                        || format!("candidate flag[{candidate_index}]"),
                        &mut region,
                        self.config.flag,
                        block_start,
                    )?;

                    for equality_index in 0..MAX_EQUALITIES {
                        let row = block_start + equality_index;
                        if equality_index != 0 {
                            // Only the block's first flag is queried under the
                            // enabled selector. Explicit zero padding keeps the
                            // assignment total and deterministic.
                            region.assign_advice(
                                || format!("candidate flag padding[{candidate_index}][{equality_index}]"),
                                self.config.flag,
                                row,
                                || Value::known(Fp::ZERO),
                            )?;
                        }
                        if let Some(equality) = candidate.equalities.get(equality_index) {
                            equality.left.copy_advice(
                                || format!("left occurrence[{candidate_index}][{equality_index}]"),
                                &mut region,
                                self.config.left,
                                row,
                            )?;
                            equality.right.copy_advice(
                                || format!("right occurrence[{candidate_index}][{equality_index}]"),
                                &mut region,
                                self.config.right,
                                row,
                            )?;
                        } else {
                            region.assign_advice(
                                || format!("left padding[{candidate_index}][{equality_index}]"),
                                self.config.left,
                                row,
                                || Value::known(Fp::ZERO),
                            )?;
                            region.assign_advice(
                                || format!("right padding[{candidate_index}][{equality_index}]"),
                                self.config.right,
                                row,
                                || Value::known(Fp::ZERO),
                            )?;
                        }
                    }
                }
                Ok(())
            },
        )
    }
}
