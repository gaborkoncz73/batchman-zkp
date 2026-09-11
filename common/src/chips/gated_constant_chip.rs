use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Selector},
    poly::Rotation,
};

/// Reusable conditional canonical-shape check: `enabled * (value-constant)=0`.
#[derive(Clone, Debug)]
pub struct GatedConstantConfig {
    enabled: Column<Advice>,
    value: Column<Advice>,
    expected: Column<Fixed>,
    q_check: Selector,
}

impl GatedConstantConfig {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> Self {
        let enabled = meta.advice_column();
        let value = meta.advice_column();
        let expected = meta.fixed_column();
        let q_check = meta.selector();
        meta.enable_equality(enabled);
        meta.enable_equality(value);
        meta.create_gate("selected canonical field", |meta| {
            let q = meta.query_selector(q_check);
            let enabled = meta.query_advice(enabled, Rotation::cur());
            let value = meta.query_advice(value, Rotation::cur());
            let expected = meta.query_fixed(expected);
            vec![q * enabled * (value - expected)]
        });
        Self {
            enabled,
            value,
            expected,
            q_check,
        }
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        enabled: &AssignedCell<Fp, Fp>,
        checks: &[(AssignedCell<Fp, Fp>, Fp)],
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "gated canonical shape",
            |mut region| {
                for (row, (source, expected)) in checks.iter().enumerate() {
                    self.q_check.enable(&mut region, row)?;
                    let enabled_local = region.assign_advice(
                        || format!("enabled[{row}]"),
                        self.enabled,
                        row,
                        || enabled.value().copied(),
                    )?;
                    region.constrain_equal(enabled_local.cell(), enabled.cell())?;
                    let value_local = region.assign_advice(
                        || format!("value[{row}]"),
                        self.value,
                        row,
                        || source.value().copied(),
                    )?;
                    region.constrain_equal(value_local.cell(), source.cell())?;
                    region.assign_fixed(
                        || format!("expected[{row}]"),
                        self.expected,
                        row,
                        || Value::known(*expected),
                    )?;
                }
                Ok(())
            },
        )
    }
}
