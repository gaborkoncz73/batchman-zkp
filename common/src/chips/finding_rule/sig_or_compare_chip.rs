use halo2_proofs::{
    circuit::{Layouter, AssignedCell, Value},
    pasta::Fp,
    plonk::Error,
};
use crate::chips::finding_rule::sig_rlc_chip::SigRlcChip;
use crate::chips::finding_rule::sig_check_chip::SigCheckConfig;

#[derive(Clone, Debug)]
pub struct SigOrCompareChip {
    pub cfg: SigCheckConfig,
    pub sig_rlc_chip: SigRlcChip,
}

impl SigOrCompareChip {
    pub fn construct(cfg: SigCheckConfig, sig_rlc_chip: SigRlcChip) -> Self {
        Self { cfg, sig_rlc_chip }
    }

    pub fn check_membership_or(
        &self,
        mut layouter: impl Layouter<Fp>,
        proof_pairs: &[(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)],
        candidate_pairs_all: &[Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>],
        is_fact: &AssignedCell<Fp, Fp>,
    ) -> Result<Vec<AssignedCell<Fp, Fp>>, Error> {
        let cfg = &self.cfg;

        // proof RLC
        let proof_rlc = self.sig_rlc_chip.fold_sig_list(
            layouter.namespace(|| "sig RLC(proof)"),
            proof_pairs,
        )?;

        // candidate RLC-k és match flag-ek (b_i)
        let mut b_flags: Vec<AssignedCell<Fp, Fp>> = Vec::new();

        for (i, cand) in candidate_pairs_all.iter().enumerate() {
            let cand_rlc = self.sig_rlc_chip.fold_sig_list(
                layouter.namespace(|| format!("sig RLC(cand {i})")),
                cand,
            )?;

            // Boolean match flag = 1 if proof_rlc == cand_rlc else 0
            let b_i = layouter.assign_region(
                || format!("match flag for candidate {i}"),
                |mut region| {
                    let val = proof_rlc
                        .value()
                        .zip(cand_rlc.value())
                        .map(|(p, c)| if *p == *c { Fp::one() } else { Fp::zero() });
                    region.assign_advice(|| "b_i", cfg.flag, 0, || val)
                },
            )?;

            b_flags.push(b_i);
        }

        // Enforce booleanity: b_i * (1 - b_i) == 0
        for (i, b) in b_flags.iter().enumerate() {
            layouter.assign_region(
                || format!("b_{} booleanity", i),
                |mut region| {
                    let val = b.value().map(|v| *v * (Fp::one() - *v));
                    let cell = region.assign_advice(|| "bool_check", cfg.flag, 0, || val)?;
                    let zero =
                        region.assign_advice(|| "zero", cfg.flag, 1, || Value::known(Fp::zero()))?;
                    region.constrain_equal(cell.cell(), zero.cell())
                },
            )?;
        }

        // Enforce Σ b_i + is_fact - Σb_i*is_fact == 1
        let sum_b_val = b_flags.iter().fold(Value::known(Fp::zero()), |acc, b| {
            acc.zip(b.value()).map(|(a, bi)| a + *bi)
        });

        let final_ok = layouter.assign_region(
            || "final ok = OR(matches) ∨ is_fact",
            |mut region| {
                let val = sum_b_val
                    .zip(is_fact.value())
                    .map(|(sum_b, f)| sum_b + *f - (sum_b) * (*f));
                region.assign_advice(|| "final_ok", cfg.sig_arity, 0, || val)
            },
        )?;

        // Enforce final_ok 
        layouter.assign_region(
            || "final_ok == 1",
            |mut region| {
                let diff_val = final_ok.value().map(|v| *v - Fp::one());
                let diff_cell = region.assign_advice(|| "diff", cfg.sig_arity, 0, || diff_val)?;
                let zero =
                    region.assign_advice(|| "zero", cfg.sig_arity, 1, || Value::known(Fp::zero()))?;
                region.constrain_equal(diff_cell.cell(), zero.cell())
            },
        )?;

        Ok(b_flags)
    }
}
