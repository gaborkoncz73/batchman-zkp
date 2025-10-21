use halo2_proofs::{
    circuit::{Chip, Layouter, Region, Value, AssignedCell},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
    poly::Rotation,
};

#[derive(Clone, Debug)]
pub struct RlcFixedConfig {
    pub token: Column<Advice>,  // input tokenek: [name, arg0, arg1, ...]
    pub acc: Column<Advice>,    // akkumulátor
    pub q: Column<Fixed>,       // selector
    pub q_last: Column<Fixed>,  // utolsó sor jelző (ha akarsz rá ellenőrzést)
    pub alpha: Fp,              // FIX konstans α
}

pub struct RlcFixedChip {
    cfg: RlcFixedConfig,
}

impl Chip<Fp> for RlcFixedChip {
    type Config = RlcFixedConfig;
    type Loaded = ();
    fn config(&self) -> &Self::Config { &self.cfg }
    fn loaded(&self) -> &Self::Loaded { &() }
}

impl RlcFixedChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>, alpha: Fp) -> RlcFixedConfig {
        let token  = meta.advice_column();
        let acc    = meta.advice_column();
        let q      = meta.fixed_column();
        let q_last = meta.fixed_column();

        meta.enable_equality(token);
        meta.enable_equality(acc);

        // Gate: acc_{i+1} = acc_i * α + token_i
        meta.create_gate("RLC fixed-alpha fold", |meta| {
            let q     = meta.query_fixed(q);
            let acc_c = meta.query_advice(acc, Rotation::cur());
            let acc_n = meta.query_advice(acc, Rotation::next());
            let tok   = meta.query_advice(token, Rotation::cur());
            let alpha = halo2_proofs::plonk::Expression::Constant(alpha);
            vec![ q * (acc_n - (acc_c * alpha + tok)) ]
        });

        RlcFixedConfig { token, acc, q, q_last, alpha }
    }

    pub fn construct(cfg: RlcFixedConfig) -> Self { Self { cfg } }

    /// tokens = [name, arg0, arg1, ...] → returns combined cell (acc_last)
    pub fn assign(
    &self,
    mut layouter: impl Layouter<Fp>,
    tokens: &[Value<Fp>],
) -> Result<(AssignedCell<Fp, Fp>, Vec<AssignedCell<Fp, Fp>>), Error> {
    assert!(!tokens.is_empty());
    let cfg = self.config();
    let n = tokens.len();

    let (last, token_cells) = layouter.assign_region(
        || "RLC single region (Value version)",
        |mut region| {
            // q / q_last
            for r in 0..=n {
                let q_val = if r < n { Fp::one() } else { Fp::zero() };
                region.assign_fixed(|| "q", cfg.q, r, || Value::known(q_val))?;
                region.assign_fixed(|| "q_last", cfg.q_last, r,
                    || Value::known(if r == n { Fp::one() } else { Fp::zero() }))?;
            }

            // acc_0
            let mut acc_val = Value::known(Fp::zero());
            region.assign_advice(|| "acc_0", cfg.acc, 0, || acc_val)?;

            // token-ek + fold
            let mut token_cells = Vec::with_capacity(n);
            for i in 0..n {
                let t_cell = region.assign_advice(|| "token", cfg.token, i, || tokens[i].clone())?;
                token_cells.push(t_cell);
                acc_val = acc_val.zip(tokens[i]).map(|(acc, t)| acc * cfg.alpha + t);
                region.assign_advice(|| "acc", cfg.acc, i + 1, || acc_val)?;
            }

            // output
            let last_cell = region.assign_advice(|| "acc_last_out", cfg.acc, n, || acc_val)?;
            Ok((last_cell, token_cells))
        },
    )?;

    Ok((last, token_cells))
}

}
