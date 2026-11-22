//! Step 4a: the arithmetic chip, the circuit form of
//! [`policy_constraints::Layout::check_arithmetic`].
//!
//! Every node has the same arithmetic step slots (see
//! [`policy_constraints::arith`]); the branch's rows set their operands and
//! flags, and this chip enforces the flagged operations on every node alike,
//! so it does not reveal which builtin a node is.
//!
//! A step is a block of two rows in eight columns, its `x`, `y`, `z` and flags
//! copied from the node's `w`:
//!
//! ```text
//!          [0]  [1]   [2]   [3]   [4]   [5]   [6]   [7]
//! row 0    x    y     z     r     σ     inv   d₁    d₂
//! row 1    d₃   add   sub   mul   div   lt    le    ne
//! ```
//!
//! with the gates
//!
//! ```text
//! add·(z − x − y) = 0    sub·(z − x + y) = 0    mul·(z − x·y) = 0
//! div·(x − z·y − r) = 0  ne·((x − y)·inv − 1) = 0
//! σ ∈ {0, 1},  (1 − div)·σ = 0,  (1 − div)·r = 0
//! d₁ = div·(ŷ − 1) + lt·(y − x − 1) + le·(y − x)
//! d₂ = div·r̂,  d₃ = div·(ŷ − r̂ − 1)      where ŷ = (1 − 2σ)·y, r̂ = (1 − 2σ)·r
//! ```
//!
//! and the range checks `x + 2⁶³, y + 2⁶³, z + 2⁶³ ∈ [0, 2⁶⁴)` (64-bit
//! integers) and `d₁, d₂, d₃ ∈ [0, 2⁶⁴)`. With every value that small the
//! field equations are integer equations, so `div` is floor division (the
//! remainder has the sign of `y` and `|r| < |y|`, and `y ≠ 0`), `lt` is `x < y`
//! and `le` is `x ≤ y`.
//!
//! A range check takes nine rows of one column: the value, then the running
//! sum `rcᵢ₊₁ = (rcᵢ + offsetᵢ − limbᵢ) / 256` with every limb looked up in a
//! table of `0..256`, ending at zero. The fixed `offset` is `2⁶³` on the first
//! row of a signed value and zero elsewhere.

use ff::PrimeField;
use halo2_proofs::circuit::{AssignedCell, Layouter, Value};
use halo2_proofs::plonk::{
    Advice, Column, ConstraintSystem, Error, Expression, Fixed, Selector, TableColumn,
};
use halo2_proofs::poly::Rotation;
use policy_constraints::Layout;
use policy_constraints::arith::{StepCell, StepOp};

const BLOCK: usize = 8;
// Row 0 of a step block.
const X: usize = 0;
const Y: usize = 1;
const Z: usize = 2;
const REMAINDER: usize = 3;
const SIGN: usize = 4;
const INVERSE: usize = 5;
const D1: usize = 6;
const D2: usize = 7;
// Row 1 of a step block: d₃, then the flags in `StepOp::ALL` order.
const D3: usize = 0;
const FLAGS: usize = 1;

/// Bits per limb, and limbs per 64-bit value.
const LIMB_BITS: u32 = 8;
const LIMBS: usize = 64 / LIMB_BITS as usize;

#[derive(Debug, Clone)]
pub struct ArithConfig {
    block: [Column<Advice>; BLOCK],
    running: Column<Advice>,
    offset: Column<Fixed>,
    q_step: Selector,
    q_limb: Selector,
    q_end: Selector,
    limbs: TableColumn,
}

impl ArithConfig {
    pub fn configure<F: PrimeField>(meta: &mut ConstraintSystem<F>) -> Self {
        let block: [Column<Advice>; BLOCK] = std::array::from_fn(|_| meta.advice_column());
        for column in block {
            meta.enable_equality(column);
        }
        let running = meta.advice_column();
        meta.enable_equality(running);
        let offset = meta.fixed_column();
        let q_step = meta.selector();
        let q_limb = meta.complex_selector();
        let q_end = meta.selector();
        let limbs = meta.lookup_table_column();

        meta.create_gate("arithmetic step", |meta| {
            let q = meta.query_selector(q_step);
            let one = Expression::Constant(F::ONE);
            let two = Expression::Constant(F::from(2));
            let top = |meta: &mut halo2_proofs::plonk::VirtualCells<'_, F>, column: usize| {
                meta.query_advice(block[column], Rotation::cur())
            };
            let bottom = |meta: &mut halo2_proofs::plonk::VirtualCells<'_, F>, column: usize| {
                meta.query_advice(block[column], Rotation::next())
            };
            let (x, y, z) = (top(meta, X), top(meta, Y), top(meta, Z));
            let (r, sign, inverse) = (top(meta, REMAINDER), top(meta, SIGN), top(meta, INVERSE));
            let (d1, d2, d3) = (top(meta, D1), top(meta, D2), bottom(meta, D3));
            let flag = |meta: &mut halo2_proofs::plonk::VirtualCells<'_, F>, op: StepOp| {
                let index = StepOp::ALL
                    .iter()
                    .position(|each| *each == op)
                    .expect("an op");
                bottom(meta, FLAGS + index)
            };
            let add = flag(meta, StepOp::Add);
            let sub = flag(meta, StepOp::Subtract);
            let mul = flag(meta, StepOp::Multiply);
            let div = flag(meta, StepOp::FloorDivide);
            let lt = flag(meta, StepOp::Less);
            let le = flag(meta, StepOp::LessOrEqual);
            let ne = flag(meta, StepOp::NotEqual);

            let unsign = one.clone() - two * sign.clone();
            let y_abs = unsign.clone() * y.clone();
            let r_abs = unsign * r.clone();
            vec![
                add * (z.clone() - x.clone() - y.clone()),
                sub * (z.clone() - x.clone() + y.clone()),
                mul * (z.clone() - x.clone() * y.clone()),
                div.clone() * (x.clone() - z * y.clone() - r.clone()),
                ne * ((x.clone() - y.clone()) * inverse - one.clone()),
                sign.clone() * (one.clone() - sign.clone()),
                (one.clone() - div.clone()) * sign,
                (one.clone() - div.clone()) * r,
                d1 - (div.clone() * (y_abs.clone() - one.clone())
                    + lt * (y.clone() - x.clone() - one.clone())
                    + le * (y - x)),
                d2 - div.clone() * r_abs.clone(),
                d3 - div * (y_abs - r_abs - one),
            ]
            .into_iter()
            .map(|constraint| q.clone() * constraint)
            .collect::<Vec<_>>()
        });

        meta.lookup("limb of a range check", |meta| {
            let q = meta.query_selector(q_limb);
            let cur = meta.query_advice(running, Rotation::cur());
            let next = meta.query_advice(running, Rotation::next());
            let offset = meta.query_fixed(offset, Rotation::cur());
            let base = Expression::Constant(F::from(1 << LIMB_BITS));
            vec![(q * (cur + offset - base * next), limbs)]
        });

        meta.create_gate("a range check ends at zero", |meta| {
            let q = meta.query_selector(q_end);
            vec![q * meta.query_advice(running, Rotation::cur())]
        });

        Self {
            block,
            running,
            offset,
            q_step,
            q_limb,
            q_end,
            limbs,
        }
    }
}

/// A cell to range check, and whether it is signed (checked as `value + 2⁶³`).
type Ranged<F> = (AssignedCell<F, F>, bool);

#[derive(Debug, Clone)]
pub struct ArithChip {
    config: ArithConfig,
}

impl ArithChip {
    pub fn construct(config: ArithConfig) -> Self {
        Self { config }
    }

    /// The table of limbs `0..256`. Once per circuit.
    pub fn load_table<F: PrimeField>(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error> {
        let column = self.config.limbs;
        layouter.assign_table(
            || "limbs",
            |mut table| {
                for limb in 0..1_u64 << LIMB_BITS {
                    table.assign_cell(
                        || "limb",
                        column,
                        limb as usize,
                        || Value::known(F::from(limb)),
                    )?;
                }
                Ok(())
            },
        )
    }

    /// Checks every step slot of one node, whose `w` cells (in `layout`
    /// order) are `w`.
    pub fn assign_node<F: PrimeField>(
        &self,
        mut layouter: impl Layouter<F>,
        layout: &Layout,
        w: &[AssignedCell<F, F>],
    ) -> Result<(), Error> {
        for step in 0..layout.steps() {
            let cell = |part: StepCell| &w[layout.step_column(step, part)];
            let ranged = layouter.assign_region(
                || format!("step {step}"),
                |mut region| self.assign_step(&mut region, &cell),
            )?;
            for (index, (value, signed)) in ranged.into_iter().enumerate() {
                layouter.assign_region(
                    || format!("step {step}: range {index}"),
                    |mut region| self.assign_range(&mut region, &value, signed),
                )?;
            }
        }
        Ok(())
    }

    /// The step block; returns the cells to range check, and whether each is
    /// signed (checked as `value + 2⁶³`).
    fn assign_step<'a, F: PrimeField>(
        &self,
        region: &mut halo2_proofs::circuit::Region<'_, F>,
        cell: &impl Fn(StepCell) -> &'a AssignedCell<F, F>,
    ) -> Result<Vec<Ranged<F>>, Error> {
        let config = &self.config;
        config.q_step.enable(region, 0)?;
        let x = cell(StepCell::X).copy_advice(|| "x", region, config.block[X], 0)?;
        let y = cell(StepCell::Y).copy_advice(|| "y", region, config.block[Y], 0)?;
        let z = cell(StepCell::Z).copy_advice(|| "z", region, config.block[Z], 0)?;
        let mut flags = Vec::new();
        for (index, op) in StepOp::ALL.into_iter().enumerate() {
            let flag = cell(StepCell::Flag(op)).copy_advice(
                || op.name(),
                region,
                config.block[FLAGS + index],
                1,
            )?;
            flags.push(flag.value().copied());
        }
        let flag = |op: StepOp| {
            let index = StepOp::ALL
                .iter()
                .position(|each| *each == op)
                .expect("an op");
            flags[index]
        };

        let (xv, yv, zv) = (x.value().copied(), y.value().copied(), z.value().copied());
        let div = flag(StepOp::FloorDivide);
        // σ = 1 for a negative divisor; the remainder of a division.
        let sign = div.zip(yv).map(|(div, y)| {
            F::from(u64::from(
                div != F::ZERO && signed_i64(y).is_some_and(|y| y < 0),
            ))
        });
        let remainder = div
            .zip(xv)
            .zip(yv.zip(zv))
            .map(|((div, x), (y, z))| if div == F::ZERO { F::ZERO } else { x - z * y });
        let inverse = xv.zip(yv).map(|(x, y)| (x - y).invert().unwrap_or(F::ZERO));
        let unsign = sign.map(|sign| F::ONE - sign.double());
        let y_abs = unsign.zip(yv).map(|(unsign, y)| unsign * y);
        let r_abs = unsign.zip(remainder).map(|(unsign, r)| unsign * r);
        let d1 = div
            .zip(flag(StepOp::Less).zip(flag(StepOp::LessOrEqual)))
            .zip(xv.zip(yv).zip(y_abs))
            .map(|((div, (lt, le)), ((x, y), y_abs))| {
                div * (y_abs - F::ONE) + lt * (y - x - F::ONE) + le * (y - x)
            });
        let d2 = div.zip(r_abs).map(|(div, r_abs)| div * r_abs);
        let d3 = div
            .zip(y_abs.zip(r_abs))
            .map(|(div, (y_abs, r_abs))| div * (y_abs - r_abs - F::ONE));

        region.assign_advice(|| "r", config.block[REMAINDER], 0, || remainder)?;
        region.assign_advice(|| "σ", config.block[SIGN], 0, || sign)?;
        region.assign_advice(|| "inv", config.block[INVERSE], 0, || inverse)?;
        let d1 = region.assign_advice(|| "d1", config.block[D1], 0, || d1)?;
        let d2 = region.assign_advice(|| "d2", config.block[D2], 0, || d2)?;
        let d3 = region.assign_advice(|| "d3", config.block[D3], 1, || d3)?;
        Ok(vec![
            (x, true),
            (y, true),
            (z, true),
            (d1, false),
            (d2, false),
            (d3, false),
        ])
    }

    /// `value (+ 2⁶³ if signed) ∈ [0, 2⁶⁴)`: nine rows of the running sum.
    fn assign_range<F: PrimeField>(
        &self,
        region: &mut halo2_proofs::circuit::Region<'_, F>,
        value: &AssignedCell<F, F>,
        signed: bool,
    ) -> Result<(), Error> {
        let config = &self.config;
        let shift = if signed { F::from(1 << 63) } else { F::ZERO };
        let base_inverse = F::from(1 << LIMB_BITS).invert().expect("256 is not zero");
        value.copy_advice(|| "value", region, config.running, 0)?;
        let shifted = value.value().map(|value| *value + shift);
        // The limbs of the low 64 bits; if the value is larger, the running
        // sum does not end at zero.
        let limbs = shifted.map(|value| low_u64(value).to_le_bytes());
        let mut running = shifted;
        for row in 0..LIMBS {
            config.q_limb.enable(region, row)?;
            let offset = if row == 0 { shift } else { F::ZERO };
            region.assign_fixed(|| "offset", config.offset, row, || Value::known(offset))?;
            running = running
                .zip(limbs)
                .map(|(running, limbs)| (running - F::from(u64::from(limbs[row]))) * base_inverse);
            region.assign_advice(|| "running sum", config.running, row + 1, || running)?;
        }
        config.q_end.enable(region, LIMBS)?;
        Ok(())
    }
}

/// The low 64 bits of a field element (little-endian representation, as for
/// the Pasta and BN254 fields).
fn low_u64<F: PrimeField>(value: F) -> u64 {
    let repr = value.to_repr();
    u64::from_le_bytes(repr.as_ref()[..8].try_into().expect("eight bytes"))
}

/// The `i64` a field element encodes (negative values as `p − |n|`), if any.
fn signed_i64<F: PrimeField>(value: F) -> Option<i64> {
    let fits = |value: F| {
        let repr = value.to_repr();
        repr.as_ref()[8..]
            .iter()
            .all(|byte| *byte == 0)
            .then(|| low_u64(value))
    };
    if let Some(magnitude) = fits(value) {
        return i64::try_from(magnitude).ok();
    }
    i64::try_from(-i128::from(fits(-value)?)).ok()
}
