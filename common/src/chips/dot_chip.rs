use halo2_proofs::{
    circuit::{Chip, Layouter, Region, Value}, pasta::Fp, plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed, Instance}, poly::Rotation
};

#[derive(Clone, Debug)]
pub struct DotConfig {
    pub adv_w: Column<Advice>,      // w[i] witnesses
    pub adv_c: Column<Advice>,      // w[i] witnesses
    pub adv_acc: Column<Advice>,    // accumulation
    pub adv_flag: Column<Advice>,    // accumulation
    pub fixed_q: Column<Fixed>,     // selector
    pub fixed_last: Column<Fixed>,  // last-row flag
    pub fixed_first: Column<Fixed>, // first-row flag
    //pub instance_c: Column<Instance>,   // public inputs
    //pub instance_flag: Column<Instance>,// public flags
}

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
    pub fn construct(config: DotConfig) -> Self {
        Self { config }
    }

    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> DotConfig {
        let adv_w = meta.advice_column();
        let adv_c = meta.advice_column();
        let adv_acc = meta.advice_column();
        let adv_flag = meta.advice_column();
        let fixed_q = meta.fixed_column();
        let fixed_last = meta.fixed_column();
        let fixed_first = meta.fixed_column();
        //let instance_c = meta.instance_column();
        //let instance_flag = meta.instance_column();

        meta.enable_equality(adv_w);
        meta.enable_equality(adv_acc);
        //meta.enable_equality(instance_c);
        //meta.enable_equality(instance_flag);

        // First row: acc_0 = w_0 * c_0
        meta.create_gate("first row acc0 = w0*c0", |meta| {
            let q_first = meta.query_fixed(fixed_first);
            let w0 = meta.query_advice(adv_w, Rotation::cur());
            //let c0 = meta.query_instance(instance_c, Rotation::cur());
            let c0 = meta.query_advice(adv_c, Rotation::cur());
            //let c0 = Expression::Constant(Fp::one());
            let acc0 = meta.query_advice(adv_acc, Rotation::cur());
            vec![q_first * (acc0 - w0 * c0)]
        });

        // Running sum acc_i = acc_{i-1} + w_i*c_i for non-first rows
        meta.create_gate("running sum", |meta| {
            let q = meta.query_fixed(fixed_q);
            let is_first = meta.query_fixed(fixed_first);
            let sel = q * (Expression::Constant(Fp::one()) - is_first);
            let wi = meta.query_advice(adv_w, Rotation::cur());
            let ci = meta.query_advice(adv_c, Rotation::cur());
            //let ci = meta.query_instance(instance_c, Rotation::cur());
            //let ci = Expression::Constant(Fp::one()); 
            let acci = meta.query_advice(adv_acc, Rotation::cur());
            let accp = meta.query_advice(adv_acc, Rotation::prev());
            vec![sel * (acci - accp - wi * ci)]
        });

        // Boolean constraint: w*(w-1)=0 for non-last rows if flag=1
        meta.create_gate("boolean non-last (optional)", |meta| {
            let q = meta.query_fixed(fixed_q);
            let is_last = meta.query_fixed(fixed_last);
            let sel = q * (Expression::Constant(Fp::one()) - is_last);
            let w = meta.query_advice(adv_w, Rotation::cur());
            //let enforce = meta.query_instance(instance_flag, Rotation::cur());
            let enforce = meta.query_advice(adv_flag, Rotation::cur());
            vec![sel * enforce* w.clone() * (w - Expression::Constant(Fp::one()))]
        });

        // Last row: w_last = 1, acc_last = 0
        meta.create_gate("last row constraints", |meta| {
            let q = meta.query_fixed(fixed_q);
            let is_last = meta.query_fixed(fixed_last);
            let sel = q * is_last;
            let w_last = meta.query_advice(adv_w, Rotation::cur());
            let acc_last = meta.query_advice(adv_acc, Rotation::cur());
            vec![
                sel.clone() * (w_last - Expression::Constant(Fp::one())),
                sel * acc_last,
            ]
        });

         DotConfig {
            adv_w,
            adv_c,
            adv_acc,
            adv_flag,
            fixed_q,
            fixed_last,
            fixed_first,
        }
    }

    /// Assigns a dot-product constraint region
    pub fn assign_dot_check(
        &self,
        mut layouter: impl Layouter<Fp>,
        w_vec: &[Fp],
        c_vec: &[Fp],
        flag_value: Fp, // single scalar
    ) -> Result<(), Error> {
        assert_eq!(w_vec.len(), c_vec.len());
        let n = w_vec.len();
        let cfg = self.config();

        layouter.assign_region(
            || "dot check region (single flag)",
            |mut region: Region<'_, Fp>| {
                let mut acc = Fp::zero();

                for i in 0..n {
                    region.assign_fixed(|| "q", cfg.fixed_q, i, || Value::known(Fp::one()))?;
                    region.assign_fixed(
                        || "first",
                        cfg.fixed_first,
                        i,
                        || Value::known(if i == 0 { Fp::one() } else { Fp::zero() }),
                    )?;
                    region.assign_fixed(
                        || "last",
                        cfg.fixed_last,
                        i,
                        || Value::known(if i + 1 == n { Fp::one() } else { Fp::zero() }),
                    )?;

                    let wi = w_vec[i];
                    let ci = c_vec[i];
                    acc = if i == 0 { wi * ci } else { acc + wi * ci };

                    region.assign_advice(|| "w", cfg.adv_w, i, || Value::known(wi))?;
                    region.assign_advice(|| "c", cfg.adv_c, i, || Value::known(ci))?;
                    region.assign_advice(|| "acc", cfg.adv_acc, i, || Value::known(acc))?;
                    // same flag each row
                    region.assign_advice(|| "flag", cfg.adv_flag, i, || Value::known(flag_value))?;
                }

                Ok(())
            },
        )
    }
}
