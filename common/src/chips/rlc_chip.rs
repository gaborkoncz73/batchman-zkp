use halo2_proofs::{
    circuit::{Chip, Layouter, Value, AssignedCell},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
    poly::Rotation,
};

use crate::{ utils_2::common_helpers::{MAX_ARITY}};

#[derive(Clone, Debug)]
pub struct RlcFixedConfig {
    pub token: Column<Advice>,  // input tokenek: [name, arg0, arg1, ...]
    pub acc: Column<Advice>,    // akkumulátor
    pub q: Column<Fixed>,       // selector
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

        RlcFixedConfig { token, acc, q, alpha }
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

    pub fn fold_one_term_as_rlc_from_cells(
        &self,
        mut layouter: impl Layouter<Fp>,
        name_cell: &AssignedCell<Fp, Fp>,
        arg_cells: &[AssignedCell<Fp, Fp>],
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        // összefűzzük a name + args cellákat egy vektorba
        let mut all_inputs = Vec::with_capacity(1 + arg_cells.len());
        all_inputs.push(name_cell.clone());
        all_inputs.extend_from_slice(arg_cells);

        // reuse-oljuk az assign_from_cells() függvényt,
        // ami a token oszlopba visszaköti az inputokat és végigfoldolja őket
        let (combined, _token_cells) = self.assign_from_cells(
            layouter.namespace(|| "RLC(term from cells)"),
            &all_inputs,
        )?;

        Ok(combined)
    }
   pub fn assign_from_cells(
        &self,
        mut layouter: impl Layouter<Fp>,
        inputs: &[AssignedCell<Fp, Fp>],
    ) -> Result<(AssignedCell<Fp, Fp>, Vec<AssignedCell<Fp, Fp>>), Error> {
        use halo2_proofs::circuit::Value;

        let cfg = self.config();
        let n = inputs.len();

        layouter.assign_region(
            || "RLC(from cells)",
            |mut region| {
                // q
                for r in 0..=n {
                    let qv = if r < n { Fp::one() } else { Fp::zero() };
                    region.assign_fixed(|| "q", cfg.q, r, || Value::known(qv))?;
                }

                // acc_0
                let mut acc = Value::known(Fp::zero());
                region.assign_advice(|| "acc_0", cfg.acc, 0, || acc)?;

                // token-ek + fold, és visszakötés
                let mut token_cells = Vec::with_capacity(n);
                for i in 0..n {
                    let t = region.assign_advice(
                        || "token",
                        cfg.token,
                        i,
                        || inputs[i].value().copied(),
                    )?;
                    // wiring: token == eredeti input cell
                    region.constrain_equal(t.cell(), inputs[i].cell())?;

                    token_cells.push(t);

                    acc = acc.zip(inputs[i].value()).map(|(a, v)| a * cfg.alpha + v);
                    region.assign_advice(|| "acc", cfg.acc, i + 1, || acc)?;
                }

                let out = region.assign_advice(|| "acc_last", cfg.acc, n, || acc)?;
                Ok((out, token_cells))
            },
        )
    }
}



