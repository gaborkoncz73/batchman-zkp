use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Region, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed},
    poly::Rotation,
};

#[derive(Clone, Debug)]
pub struct DotConfig {
    pub adv_w: Column<Advice>,      // w[i]
    pub adv_c: Column<Advice>,      // c[i]
    pub adv_acc: Column<Advice>,    // running accumulator
    pub adv_flag: Column<Advice>,   // boolean-enforcement flag (0/1)
    pub adv_fact: Column<Advice>,   // is_fact flag (0 = rule, 1 = fact) — gates all checks
    pub fixed_q: Column<Fixed>,     // selector
    pub fixed_last: Column<Fixed>,  // last-row flag
    pub fixed_first: Column<Fixed>, // first-row flag
}

#[derive(Clone, Debug)]
pub struct DotChip {
    config: DotConfig,
}

impl Chip<Fp> for DotChip {
    type Config = DotConfig;
    type Loaded = ();
    fn config(&self) -> &Self::Config { &self.config }
    fn loaded(&self) -> &Self::Loaded { &() }
}

impl DotChip {
    pub fn construct(config: DotConfig) -> Self { Self { config } }

    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> DotConfig {
        let adv_w      = meta.advice_column();
        let adv_c      = meta.advice_column();
        let adv_acc    = meta.advice_column();
        let adv_flag   = meta.advice_column();
        let adv_fact   = meta.advice_column();
        let fixed_q    = meta.fixed_column();
        let fixed_last = meta.fixed_column();
        let fixed_first= meta.fixed_column();

        meta.enable_equality(adv_w);
        meta.enable_equality(adv_c);
        meta.enable_equality(adv_acc);
        meta.enable_equality(adv_flag);
        meta.enable_equality(adv_fact);

        // (1) Accumulation: acc = (is_first ? w*c : acc_prev + w*c), gated by (1 - is_fact)
        meta.create_gate("dot accumulation (gated)", |meta| {
            let q        = meta.query_fixed(fixed_q);
            let is_first = meta.query_fixed(fixed_first);
            let w        = meta.query_advice(adv_w, Rotation::cur());
            let c        = meta.query_advice(adv_c, Rotation::cur());
            let acc      = meta.query_advice(adv_acc, Rotation::cur());
            let acc_prev = meta.query_advice(adv_acc, Rotation::prev());
            let fact     = meta.query_advice(adv_fact, Rotation::cur());
            let en       = Expression::Constant(Fp::one()) - fact; // enable when not fact

            vec![
                en * q * (acc - (acc_prev * (Expression::Constant(Fp::one()) - is_first) + w * c))
            ]
        });

        // (2) Optional booleanity on w for non-last rows, gated by (1 - is_fact) and flag
        meta.create_gate("boolean w (gated)", |meta| {
            let q      = meta.query_fixed(fixed_q);
            let is_last= meta.query_fixed(fixed_last);
            let w      = meta.query_advice(adv_w, Rotation::cur());
            let flag   = meta.query_advice(adv_flag, Rotation::cur());
            let fact   = meta.query_advice(adv_fact, Rotation::cur());
            let en     = (Expression::Constant(Fp::one()) - fact)
                        * (Expression::Constant(Fp::one()) - is_last);

            vec![
                en * q * flag * w.clone() * (w - Expression::Constant(Fp::one()))
            ]
        });

        // (3) Last row: enforce acc_last=0, gated by (1 - is_fact)
        meta.create_gate("last row checks (gated)", |meta| {
            let q      = meta.query_fixed(fixed_q);
            let is_last= meta.query_fixed(fixed_last);
            //let w_last = meta.query_advice(adv_w, Rotation::cur());
            let acc_l  = meta.query_advice(adv_acc, Rotation::cur());
            let fact   = meta.query_advice(adv_fact, Rotation::cur());
            let en     = (Expression::Constant(Fp::one()) - fact) * is_last;

            vec![
                en * q * acc_l,
            ]
        });

        DotConfig { adv_w, adv_c, adv_acc, adv_flag, adv_fact, fixed_q, fixed_last, fixed_first }
    }

    /// on-circuit dot-product: ⟨w,c⟩ == 0 when is_fact = 0; fully disabled when is_fact = 1.
    pub fn assign_dot_check(
        &self,
        mut layouter: impl Layouter<Fp>,
        w_vec: &[AssignedCell<Fp, Fp>],
        c_vec: &[AssignedCell<Fp, Fp>],
        flag_cell: &AssignedCell<Fp, Fp>, // 0/1, controls boolean constraint on w
        fact_cell: &AssignedCell<Fp, Fp>, // 0 for rule, 1 for fact (disables all checks)
    ) -> Result<(), Error> {
        assert_eq!(w_vec.len(), c_vec.len());
        let n = w_vec.len();
        let cfg = self.config();

        layouter.assign_region(
            || "dot-product (gated by is_fact)",
            |mut region: Region<'_, Fp>| {
                let mut acc_val: Value<Fp> = Value::known(Fp::zero());

                for i in 0..n {
                    // Fixed selectors
                    region.assign_fixed(|| "q",     cfg.fixed_q,     i, || Value::known(Fp::one()))?;
                    region.assign_fixed(|| "first", cfg.fixed_first, i, || Value::known(if i == 0 { Fp::one() } else { Fp::zero() }))?;
                    region.assign_fixed(|| "last",  cfg.fixed_last,  i, || Value::known(if i + 1 == n { Fp::one() } else { Fp::zero() }))?;

                    // Values
                    let wi = w_vec[i].value();
                    let ci = c_vec[i].value();

                    // Running accumulation
                    acc_val = if i == 0 {
                        wi.zip(ci).map(|(w, c)| *w * *c)
                    } else {
                        acc_val.zip(wi).zip(ci).map(|((a, w), c)| a + *w * *c)
                    };

                    // Assign w, c, acc
                    let w_local = region.assign_advice(|| "w",   cfg.adv_w,   i, || wi.copied())?;
                    let c_local = region.assign_advice(|| "c",   cfg.adv_c,   i, || ci.copied())?;
                    let a_local = region.assign_advice(|| "acc", cfg.adv_acc, i, || acc_val)?;

                    // Copy in flag and fact, and constrain-equal to the originals
                    let flag_local = region.assign_advice(
                        || "flag (copy)",
                        cfg.adv_flag,
                        i,
                        || flag_cell.value().copied(),
                    )?;
                    region.constrain_equal(flag_local.cell(), flag_cell.cell())?;

                    let fact_local = region.assign_advice(
                        || "fact (copy)",
                        cfg.adv_fact,
                        i,
                        || fact_cell.value().copied(),
                    )?;
                    region.constrain_equal(fact_local.cell(), fact_cell.cell())?;

                     //Optional debug:
                     if i + 1 == n {
                         println!("DOT last row: acc={:?}, w_last={:?}, is_fact={:?}",
                             acc_val, w_vec.last().unwrap().value(), fact_cell.value());
                     }

                    // silence unused locals in release builds
                    let _ = (w_local, c_local, a_local);
                }

                Ok(())
            },
        )
    }
}
