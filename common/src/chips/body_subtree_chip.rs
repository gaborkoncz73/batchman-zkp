use halo2_proofs::{
    circuit::{AssignedCell, Layouter},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error},
};
use crate::{
    data::TermFp,
    utils_2::common_helpers::MAX_ARITY,
};
#[derive(Debug, Clone)]
pub struct UnifCompareConfig {
    pub body_name: Column<Advice>,
    pub body_args: [Column<Advice>; MAX_ARITY],
    pub subtree_name: Column<Advice>,
    pub subtree_args: [Column<Advice>; MAX_ARITY],
}

impl UnifCompareConfig {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> Self {
        let body_name = meta.advice_column();
        let body_args = array_init::array_init(|_| meta.advice_column());
        let subtree_name = meta.advice_column();
        let subtree_args = array_init::array_init(|_| meta.advice_column());
        
        meta.enable_equality(body_name);
        for c in body_args.iter() {
            meta.enable_equality(*c);
        }

        meta.enable_equality(subtree_name);
        for c in subtree_args.iter() {
            meta.enable_equality(*c);
        }
        Self { body_name, body_args, subtree_name, subtree_args }
    }
}
pub struct BodySubtreeChip {
    pub cfg: UnifCompareConfig,
}

impl BodySubtreeChip {
    pub fn construct(cfg: UnifCompareConfig) -> Self {
        Self { cfg }
    }

    /// Beköti a body és subtree termeket, kitöltve 0-val MAX_PAIRS-ig.
    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        body: &[TermFp],
        subtree: &[TermFp],
    ) -> Result<
        (
            Vec<(AssignedCell<Fp, Fp>, Vec<AssignedCell<Fp, Fp>>)>, // body_pairs
            Vec<(AssignedCell<Fp, Fp>, Vec<AssignedCell<Fp, Fp>>)>, // subtree_pairs
        ),
        Error,
    > {
        use halo2_proofs::circuit::Value;
        use crate::utils_2::common_helpers::MAX_PAIRS;

        layouter.assign_region(
            || "BodySubtreeChip region",
            |mut region| {
                let mut body_pairs = Vec::with_capacity(MAX_PAIRS);
                let mut subtree_pairs = Vec::with_capacity(MAX_PAIRS);

                // BODY
                for i in 0..MAX_PAIRS {
                    let row0 = i * MAX_ARITY;
                    let term = body.get(i);

                    // TERM NAME
                    let name_val = Value::known(term.map(|t| t.name).unwrap_or(Fp::zero()));
                    let name_cell = region.assign_advice(
                        || format!("body[{i}].name"),
                        self.cfg.body_name,
                        i,
                        || name_val,
                    )?;

                    // BODY ARGS
                    let mut args_cells = Vec::with_capacity(MAX_ARITY);
                    for j in 0..MAX_ARITY {
                        let aval = Value::known(match term {
                            Some(t) => *t.args.get(j).unwrap_or(&Fp::zero()),
                            None => Fp::zero(),
                        });
                        let c = region.assign_advice(
                            || format!("body[{i}].arg{j}"),
                            self.cfg.body_args[j],
                            row0 + j,
                            || aval,
                        )?;
                        args_cells.push(c);
                    }
                    body_pairs.push((name_cell, args_cells));
                }

                // SUBTREE

                for i in 0..MAX_PAIRS {
                    let row0 = i * MAX_ARITY;
                    let term = subtree.get(i);

                    let name_val = Value::known(term.map(|t| t.name).unwrap_or(Fp::zero()));
                    let name_cell = region.assign_advice(
                        || format!("subtree[{i}].name"),
                        self.cfg.subtree_name,
                        i,
                        || name_val,
                    )?;

                    let mut args_cells = Vec::with_capacity(MAX_ARITY);
                    for j in 0..MAX_ARITY {
                        let aval = Value::known(match term {
                            Some(t) => *t.args.get(j).unwrap_or(&Fp::zero()),
                            None => Fp::zero(),
                        });
                        let c = region.assign_advice(
                            || format!("subtree[{i}].arg{j}"),
                            self.cfg.subtree_args[j],
                            row0 + j,
                            || aval,
                        )?;
                        args_cells.push(c);
                    }
                    subtree_pairs.push((name_cell, args_cells));
                }
                Ok((body_pairs, subtree_pairs))
            },
        )
    }
}