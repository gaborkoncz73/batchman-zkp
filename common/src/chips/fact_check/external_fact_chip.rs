//! Sound membership in the fixed public external-fact registry.
//!
//! Given an already constrained fact hash `h`, this chip evaluates
//! `prod_i (h - public_hash[i])` with an algebraically constrained running
//! product.  A selected external-fact branch requires the final product to be
//! zero, which is equivalent to membership because the field has no zero
//! divisors.

use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Instance, Selector},
    poly::Rotation,
};

use crate::utils_2::common_helpers::MAX_FACTS_HASHES;

#[derive(Clone, Debug)]
pub struct ExternalFactConfig {
    branch: Column<Advice>,
    hash: Column<Advice>,
    public_hash: Column<Advice>,
    accumulator: Column<Advice>,
    public_inputs: Column<Instance>,
    q_first: Selector,
    q_step: Selector,
    q_last: Selector,
}

#[derive(Clone, Debug)]
pub struct ExternalFactChip {
    config: ExternalFactConfig,
}

impl Chip<Fp> for ExternalFactChip {
    type Config = ExternalFactConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl ExternalFactChip {
    pub fn configure(
        meta: &mut ConstraintSystem<Fp>,
        public_inputs: Column<Instance>,
    ) -> ExternalFactConfig {
        let branch = meta.advice_column();
        let hash = meta.advice_column();
        let public_hash = meta.advice_column();
        let accumulator = meta.advice_column();
        let q_first = meta.selector();
        let q_step = meta.selector();
        let q_last = meta.selector();

        for column in [branch, hash, public_hash, accumulator] {
            meta.enable_equality(column);
        }
        meta.enable_equality(public_inputs);

        meta.create_gate(
            "external fact: boolean branch and product identity",
            |meta| {
                let q = meta.query_selector(q_first);
                let branch = meta.query_advice(branch, Rotation::cur());
                let accumulator = meta.query_advice(accumulator, Rotation::cur());
                vec![
                    q.clone()
                        * branch.clone()
                        * (branch - halo2_proofs::plonk::Expression::Constant(Fp::one())),
                    q * (accumulator - halo2_proofs::plonk::Expression::Constant(Fp::one())),
                ]
            },
        );

        meta.create_gate("external fact: running membership product", |meta| {
            let q = meta.query_selector(q_step);
            let hash = meta.query_advice(hash, Rotation::cur());
            let public_hash = meta.query_advice(public_hash, Rotation::cur());
            let accumulator_current = meta.query_advice(accumulator, Rotation::cur());
            let accumulator_next = meta.query_advice(accumulator, Rotation::next());
            vec![q * (accumulator_next - accumulator_current * (hash - public_hash))]
        });

        meta.create_gate("external fact: selected branch is a member", |meta| {
            let q = meta.query_selector(q_last);
            let branch = meta.query_advice(branch, Rotation::cur());
            let accumulator = meta.query_advice(accumulator, Rotation::cur());
            vec![q * branch * accumulator]
        });

        ExternalFactConfig {
            branch,
            hash,
            public_hash,
            accumulator,
            public_inputs,
            q_first,
            q_step,
            q_last,
        }
    }

    pub fn construct(config: ExternalFactConfig) -> Self {
        Self { config }
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        computed_hash: &AssignedCell<Fp, Fp>,
        branch_hint: Value<Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let config = self.config();

        layouter.assign_region(
            || "external fact public membership",
            |mut region| {
                let mut accumulator_value = Value::known(Fp::one());
                let mut branch_cell = None;

                for index in 0..=MAX_FACTS_HASHES {
                    let _accumulator = region.assign_advice(
                        || format!("accumulator[{index}]"),
                        config.accumulator,
                        index,
                        || accumulator_value,
                    )?;
                    if index == 0 {
                        config.q_first.enable(&mut region, index)?;
                        let branch = region.assign_advice(
                            || "external fact branch",
                            config.branch,
                            index,
                            || branch_hint,
                        )?;
                        branch_cell = Some(branch);
                    }

                    if index < MAX_FACTS_HASHES {
                        config.q_step.enable(&mut region, index)?;
                        let hash = region.assign_advice(
                            || format!("computed hash[{index}]"),
                            config.hash,
                            index,
                            || computed_hash.value().copied(),
                        )?;
                        region.constrain_equal(hash.cell(), computed_hash.cell())?;
                        let public_hash = region.assign_advice_from_instance(
                            || format!("public hash[{index}]"),
                            config.public_inputs,
                            index,
                            config.public_hash,
                            index,
                        )?;
                        accumulator_value = accumulator_value
                            .zip(computed_hash.value().copied())
                            .zip(public_hash.value().copied())
                            .map(|((accumulator, hash), public)| accumulator * (hash - public));
                    } else {
                        config.q_last.enable(&mut region, index)?;
                        let branch = region.assign_advice(
                            || "final external fact branch",
                            config.branch,
                            index,
                            || branch_hint,
                        )?;
                        region.constrain_equal(
                            branch.cell(),
                            branch_cell.as_ref().ok_or(Error::Synthesis)?.cell(),
                        )?;
                    }
                }

                branch_cell.ok_or(Error::Synthesis)
            },
        )
    }
}
