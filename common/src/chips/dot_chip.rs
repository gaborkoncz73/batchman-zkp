use halo2_proofs::{
    circuit::{Chip, Layouter, Region, Value}, pasta::Fp, plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed}, poly::Rotation
};

// SUBCIRCUIT for dot production
// Total constraints: 2n + 1 (n is the length of the vectors)
#[derive(Clone, Debug)]
pub struct DotConfig {
    pub adv_w: Column<Advice>,      // w[i] witnesses, vector for the actual values
    pub adv_c: Column<Advice>,      // w[i] witnesses, vector for the rules
    pub adv_acc: Column<Advice>,    // accumulation for the dot product computation
    pub adv_flag: Column<Advice>,    // flag to differ if the witness vector elements should be only 0 and 1 or anything else
    pub fixed_q: Column<Fixed>,     // selector
    pub fixed_last: Column<Fixed>,  // last-row flag
    pub fixed_first: Column<Fixed>, // first-row flag
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

        meta.enable_equality(adv_w);
        meta.enable_equality(adv_acc);

        // Unified accumulation rule:
        // For first row (is_first=1):    acc = w * c
        // For later rows (is_first=0):   acc = acc_prev + w * c
        // Builds the running dot product sum across all rows
        // Constraints: n
        meta.create_gate("unified accumulation", |meta| {
            let q = meta.query_fixed(fixed_q);
            let is_first = meta.query_fixed(fixed_first);
            let w = meta.query_advice(adv_w, Rotation::cur());
            let c = meta.query_advice(adv_c, Rotation::cur());
            let acc = meta.query_advice(adv_acc, Rotation::cur());
            let acc_prev = meta.query_advice(adv_acc, Rotation::prev());

            vec![
                q * (acc - acc_prev * (Expression::Constant(Fp::one()) - is_first) - w * c)
            ]
        });

        // Boolean constraint for non-last rows:
        // Enforces w ∈ {0,1} (w*(w-1)=0) when flag=1 and not the last row
        // This ensures that intermediate witness values are boolean selectors if required
        // Constraints: n - 1
        meta.create_gate("boolean non-last (optional)", |meta| {
            let q = meta.query_fixed(fixed_q);
            let is_last = meta.query_fixed(fixed_last);
            let sel = q * (Expression::Constant(Fp::one()) - is_last);
            let w = meta.query_advice(adv_w, Rotation::cur());
            let enforce = meta.query_advice(adv_flag, Rotation::cur());
            vec![sel * enforce* w.clone() * (w - Expression::Constant(Fp::one()))]
        });

        // Final row constraints:
        // Applies only on the last row (is_last=1)
        // Enforces that w_last = 1 (normalization constant) and acc_last = 0 (dot product result is zero)
        // Constraints: 2
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
                    region.assign_advice(|| "flag", cfg.adv_flag, i, || Value::known(flag_value))?;
                }
                Ok(())
            },
        )
    }
}