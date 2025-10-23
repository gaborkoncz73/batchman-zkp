use halo2_proofs::{
    circuit::{Layouter, AssignedCell},
    pasta::Fp,
    plonk::Error,
};
use crate::chips::sig_rlc_chip::SigRlcChip;
use crate::chips::sig_check_chip::SigCheckConfig;

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
    ) -> Result<(), Error> {
        use halo2_proofs::circuit::Value;
        let cfg = &self.cfg;

        // --- 1️⃣ proof RLC
        let proof_rlc = self.sig_rlc_chip.fold_sig_list(
            layouter.namespace(|| "sig RLC(proof)"),
            proof_pairs,
        )?;

        // --- 2️⃣ candidate RLC-k
        let cand_rlcs: Vec<AssignedCell<Fp, Fp>> = candidate_pairs_all
            .iter()
            .enumerate()
            .map(|(i, cand)| {
                self.sig_rlc_chip
                    .fold_sig_list(layouter.namespace(|| format!("sig RLC(cand {i})")), cand)
            })
            .collect::<Result<Vec<_>, _>>()?;

        // --- 3️⃣ compute OR( matches )  —  legalább egy cand egyezik
        let or_cell = layouter.assign_region(
            || "any match OR",
            |mut region| {
                let mut acc = Value::known(Fp::zero());
                for (i, cand_rlc) in cand_rlcs.iter().enumerate() {
                    let match_val = proof_rlc
                        .value()
                        .zip(cand_rlc.value())
                        .map(|(p, c)| if *p == *c { Fp::one() } else { Fp::zero() });
                    // OR accumulator: acc = 1 - (1-acc)*(1-match)
                    acc = acc.zip(match_val).map(|(a, m)| {
                        Fp::one() - (Fp::one() - a) * (Fp::one() - m)
                    });
                    region.assign_advice(|| format!("or_step_{}", i), cfg.sig_arity, i, || acc)?;
                }
                // végső OR érték:
                region.assign_advice(|| "or_final", cfg.sig_arity, cand_rlcs.len(), || acc)
            },
        )?;

        // --- 4️⃣ final_ok = OR(matches) + is_fact - OR*is_fact  (logikai OR)
        let final_ok = layouter.assign_region(
            || "final ok = match OR is_fact",
            |mut region| {
                let val = or_cell
                    .value()
                    .zip(is_fact.value())
                    .map(|(m, f)| *m + *f - (*m) * (*f));
                region.assign_advice(|| "final_ok", cfg.sig_arity, 0, || val)
            },
        )?;

        // --- 5️⃣ Enforce final_ok == 1
        layouter.assign_region(
            || "final_ok == 1",
            |mut region| {
                let diff_val = final_ok.value().map(|v| *v - Fp::one());
                let diff_cell = region.assign_advice(|| "diff", cfg.sig_arity, 0, || diff_val)?;
                let zero = region.assign_advice(|| "zero", cfg.sig_arity, 1, || Value::known(Fp::zero()))?;
                region.constrain_equal(diff_cell.cell(), zero.cell())
            },
        )?;

        Ok(())
    }
}
