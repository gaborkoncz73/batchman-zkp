//! Algebraic unsigned 32-bit range checks.
//!
//! The old builtin code converted field elements to host integers.  That does
//! not constrain the prover.  This chip instead opens a field element into 32
//! boolean little-endian bits and reconstructs the original cell in-circuit.

use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Selector},
    poly::Rotation,
};
use halo2curves::ff::PrimeField;

pub const U32_BITS: usize = 32;

#[derive(Clone, Debug)]
pub struct U32RangeConfig {
    bit: Column<Advice>,
    accumulator: Column<Advice>,
    enabled: Column<Advice>,
    value: Column<Advice>,
    weight: Column<Fixed>,
    q_first: Selector,
    q_step: Selector,
    q_last: Selector,
}

#[derive(Clone, Debug)]
pub struct U32RangeChip {
    config: U32RangeConfig,
}

impl Chip<Fp> for U32RangeChip {
    type Config = U32RangeConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl U32RangeChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> U32RangeConfig {
        let bit = meta.advice_column();
        let accumulator = meta.advice_column();
        let enabled = meta.advice_column();
        let value = meta.advice_column();
        Self::configure_with_columns(meta, bit, accumulator, enabled, value)
    }

    /// Configures the range gates on caller-owned advice columns.  The range
    /// regions are disjoint from the caller's other gates, so reusing columns
    /// avoids four permanent columns in composite chips without weakening any
    /// copy or range constraint.
    pub fn configure_with_columns(
        meta: &mut ConstraintSystem<Fp>,
        bit: Column<Advice>,
        accumulator: Column<Advice>,
        enabled: Column<Advice>,
        value: Column<Advice>,
    ) -> U32RangeConfig {
        let weight = meta.fixed_column();
        let q_first = meta.selector();
        let q_step = meta.selector();
        let q_last = meta.selector();

        for column in [bit, accumulator, enabled, value] {
            meta.enable_equality(column);
        }

        meta.create_gate("u32 range: boolean enable and zero accumulator", |meta| {
            let q = meta.query_selector(q_first);
            let enabled = meta.query_advice(enabled, Rotation::cur());
            let accumulator = meta.query_advice(accumulator, Rotation::cur());
            vec![
                q.clone()
                    * enabled.clone()
                    * (enabled - halo2_proofs::plonk::Expression::Constant(Fp::one())),
                q * accumulator,
            ]
        });

        meta.create_gate("u32 range: bit and reconstruction step", |meta| {
            let q = meta.query_selector(q_step);
            let bit = meta.query_advice(bit, Rotation::cur());
            let accumulator_current = meta.query_advice(accumulator, Rotation::cur());
            let accumulator_next = meta.query_advice(accumulator, Rotation::next());
            let weight = meta.query_fixed(weight);
            vec![
                q.clone()
                    * bit.clone()
                    * (bit.clone() - halo2_proofs::plonk::Expression::Constant(Fp::one())),
                q * (accumulator_next - accumulator_current - bit * weight),
            ]
        });

        meta.create_gate("u32 range: reconstructed value", |meta| {
            let q = meta.query_selector(q_last);
            let enabled = meta.query_advice(enabled, Rotation::cur());
            let accumulator = meta.query_advice(accumulator, Rotation::cur());
            let value = meta.query_advice(value, Rotation::cur());
            vec![q * enabled * (value - accumulator)]
        });

        U32RangeConfig {
            bit,
            accumulator,
            enabled,
            value,
            weight,
            q_first,
            q_step,
            q_last,
        }
    }

    pub fn construct(config: U32RangeConfig) -> Self {
        Self { config }
    }

    /// Constrains `value` to the unsigned 32-bit domain when `enabled == 1`.
    /// `enabled` is constrained to be boolean by this chip.  When it is zero,
    /// the value is intentionally left outside the range relation.
    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        value: &AssignedCell<Fp, Fp>,
        enabled: &AssignedCell<Fp, Fp>,
    ) -> Result<(), Error> {
        let config = self.config();
        let bits = value.value().map(|field| {
            let repr = field.to_repr();
            let bytes = repr.as_ref();
            std::array::from_fn::<_, U32_BITS, _>(|index| {
                ((bytes[index / 8] >> (index % 8)) & 1) as u64
            })
        });

        layouter.assign_region(
            || "conditional u32 decomposition",
            |mut region| {
                let mut accumulator = Value::known(Fp::zero());

                for row in 0..U32_BITS {
                    config.q_step.enable(&mut region, row)?;
                    if row == 0 {
                        config.q_first.enable(&mut region, row)?;
                    }

                    if row == 0 {
                        let enabled_local = region.assign_advice(
                            || "enabled",
                            config.enabled,
                            row,
                            || enabled.value().copied(),
                        )?;
                        region.constrain_equal(enabled_local.cell(), enabled.cell())?;
                    }

                    region.assign_advice(
                        || format!("accumulator[{row}]"),
                        config.accumulator,
                        row,
                        || accumulator,
                    )?;

                    let bit_value = bits.map(|bits| Fp::from(bits[row]));
                    region.assign_advice(
                        || format!("bit[{row}]"),
                        config.bit,
                        row,
                        || bit_value,
                    )?;
                    let weight = Fp::from(1_u64 << row);
                    region.assign_fixed(
                        || format!("2^{row}"),
                        config.weight,
                        row,
                        || Value::known(weight),
                    )?;
                    accumulator = accumulator
                        .zip(bit_value)
                        .map(|(accumulator, bit)| accumulator + bit * weight);
                }

                let final_row = U32_BITS;
                config.q_last.enable(&mut region, final_row)?;
                region.assign_advice(
                    || "final accumulator",
                    config.accumulator,
                    final_row,
                    || accumulator,
                )?;
                let enabled_local = region.assign_advice(
                    || "final enabled",
                    config.enabled,
                    final_row,
                    || enabled.value().copied(),
                )?;
                region.constrain_equal(enabled_local.cell(), enabled.cell())?;
                let value_local = region.assign_advice(
                    || "range-checked value",
                    config.value,
                    final_row,
                    || value.value().copied(),
                )?;
                region.constrain_equal(value_local.cell(), value.cell())
            },
        )
    }
}
