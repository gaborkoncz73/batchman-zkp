use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
};

use crate::data::LocalStepKind;

/// Binds the three semantic resolution branches to one public protocol tag.
///
/// Tags are `1 = policy rule`, `2 = external fact`, `3 = builtin`. The
/// builtin functor itself remains bound by the goal commitment and is checked
/// by the builtin chip.
#[derive(Clone, Debug)]
pub struct ResolutionConfig {
    branch_rule: Column<Advice>,
    branch_external: Column<Advice>,
    branch_builtin: Column<Advice>,
    child_count: Column<Advice>,
    leaf: Column<Advice>,
    tag: Column<Advice>,
    q_resolution: Selector,
}

impl ResolutionConfig {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> Self {
        let branch_rule = meta.advice_column();
        let branch_external = meta.advice_column();
        let branch_builtin = meta.advice_column();
        let child_count = meta.advice_column();
        let leaf = meta.advice_column();
        let tag = meta.advice_column();
        for column in [
            branch_rule,
            branch_external,
            branch_builtin,
            child_count,
            leaf,
            tag,
        ] {
            meta.enable_equality(column);
        }
        let q_resolution = meta.selector();
        meta.create_gate("one semantic resolution and public tag", |meta| {
            let q = meta.query_selector(q_resolution);
            let rule = meta.query_advice(branch_rule, Rotation::cur());
            let external = meta.query_advice(branch_external, Rotation::cur());
            let builtin = meta.query_advice(branch_builtin, Rotation::cur());
            let child_count_expr = meta.query_advice(child_count, Rotation::cur());
            let leaf_expr = meta.query_advice(leaf, Rotation::cur());
            let tag = meta.query_advice(tag, Rotation::cur());
            let one = halo2_proofs::plonk::Expression::Constant(Fp::one());
            vec![
                q.clone() * rule.clone() * (rule.clone() - one.clone()),
                q.clone() * external.clone() * (external.clone() - one.clone()),
                q.clone() * builtin.clone() * (builtin.clone() - one.clone()),
                q.clone() * (rule.clone() + external.clone() + builtin.clone() - one),
                q.clone() * (leaf_expr - external.clone() - builtin.clone()),
                // Typed topology has no synthetic `[true]` child. External
                // facts and builtins are therefore semantic zero-child leaves.
                q.clone() * (external.clone() + builtin.clone()) * child_count_expr,
                q * (tag
                    - rule
                    - halo2_proofs::plonk::Expression::Constant(Fp::from(2)) * external
                    - halo2_proofs::plonk::Expression::Constant(Fp::from(3)) * builtin),
            ]
        });

        Self {
            branch_rule,
            branch_external,
            branch_builtin,
            child_count,
            leaf,
            tag,
            q_resolution,
        }
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        rule: &AssignedCell<Fp, Fp>,
        external: &AssignedCell<Fp, Fp>,
        builtin: &AssignedCell<Fp, Fp>,
        child_count: &AssignedCell<Fp, Fp>,
        resolution: LocalStepKind,
    ) -> Result<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>), Error> {
        layouter.assign_region(
            || "bind local resolution tag",
            |mut region| {
                self.q_resolution.enable(&mut region, 0)?;
                let rule_local = region.assign_advice(
                    || "rule branch",
                    self.branch_rule,
                    0,
                    || rule.value().copied(),
                )?;
                let external_local = region.assign_advice(
                    || "external branch",
                    self.branch_external,
                    0,
                    || external.value().copied(),
                )?;
                let builtin_local = region.assign_advice(
                    || "builtin branch",
                    self.branch_builtin,
                    0,
                    || builtin.value().copied(),
                )?;
                region.constrain_equal(rule_local.cell(), rule.cell())?;
                region.constrain_equal(external_local.cell(), external.cell())?;
                region.constrain_equal(builtin_local.cell(), builtin.cell())?;
                let child_count_local = region.assign_advice(
                    || "semantic child count",
                    self.child_count,
                    0,
                    || child_count.value().copied(),
                )?;
                region.constrain_equal(child_count_local.cell(), child_count.cell())?;
                let leaf = region.assign_advice(
                    || "leaf branch",
                    self.leaf,
                    0,
                    || {
                        external
                            .value()
                            .copied()
                            .zip(builtin.value().copied())
                            .map(|(external, builtin)| external + builtin)
                    },
                )?;
                let tag = region.assign_advice(
                    || "resolution tag",
                    self.tag,
                    0,
                    || Value::known(resolution.protocol_field()),
                )?;
                Ok((tag, leaf))
            },
        )
    }
}
