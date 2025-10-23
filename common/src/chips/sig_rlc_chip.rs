use halo2_proofs::{circuit::{AssignedCell, Layouter}, pasta::Fp, plonk::Error};
use crate::chips::rlc_chip::{RlcFixedChip, RlcFixedConfig};
use crate::utils_2::common_helpers::MAX_SIG_TOKENS;

#[derive(Clone, Debug)]
pub struct SigRlcChip {
    pub cfg: RlcFixedConfig,
}

impl SigRlcChip {
    pub fn construct(cfg: RlcFixedConfig) -> Self { Self { cfg } }

    pub fn fold_sig_list(
        &self,
        mut layouter: impl Layouter<Fp>,
        pairs: &[(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)],
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        use halo2_proofs::circuit::Value;
        let rlc = RlcFixedChip::construct(self.cfg.clone());
        let sep = Fp::from(0x534947u64); // "SIG" domain separator

        let tokens: Vec<AssignedCell<Fp, Fp>> = layouter.assign_region(
            || "sig tokens",
            |mut region| {
                let mut toks = Vec::new();
                let sep_cell = region.assign_advice(|| "sep", self.cfg.token, 0, || Value::known(sep))?;
                toks.push(sep_cell);
                let mut row = 1;

                for (n, a) in pairs.iter() {
                    let n_tok = region.assign_advice(|| "name_tok", self.cfg.token, row, || n.value().copied())?;
                    region.constrain_equal(n_tok.cell(), n.cell())?;
                    row += 1;

                    let a_tok = region.assign_advice(|| "arity_tok", self.cfg.token, row, || a.value().copied())?;
                    region.constrain_equal(a_tok.cell(), a.cell())?;
                    row += 1;

                    toks.push(n_tok);
                    toks.push(a_tok);
                }

                while toks.len() < MAX_SIG_TOKENS {
                    let pad = region.assign_advice(|| "pad", self.cfg.token, row, || Value::known(Fp::zero()))?;
                    toks.push(pad);
                    row += 1;
                }

                Ok(toks)
            },
        )?;

        let (combined, _) = rlc.assign_from_cells(
            layouter.namespace(|| "RLC(sig full)"),
            &tokens,
        )?;
        Ok(combined)
    }
}
