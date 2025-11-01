use halo2_proofs::{
    circuit::{Layouter, AssignedCell, Value},
    pasta::Fp,
    plonk::Error,
};
use crate::{chips::finding_rule::sig_rlc_chip::SigRlcChip, utils_2::common_helpers::to_fp_value};
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
    // NESTED proof: Vec<row>[ (name,arity), ... ]
    proof_pairs_nested: &[Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>],
    // NESTED candidates: Vec<candidate>[ Vec<row>[ (name,arity), ... ] ]
    candidate_pairs_nested: &[Vec<Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>>],
    is_fact: &AssignedCell<Fp, Fp>,
) -> Result<Vec<AssignedCell<Fp, Fp>>, Error> {
    let cfg = &self.cfg;
    //println!("LIST: \n {:?}", proof_pairs_nested);
    let mut proof_flat: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();
    for row in proof_pairs_nested {
        for pair in row {
            // AssignedCell<Fp,Fp> Clone: csak a hivatkozást klónozzuk, nem hozunk létre új tanút
            proof_flat.push(pair.clone());
        }
    }

    let mut candidates_flat: Vec<Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>> = Vec::new();
    for cand in candidate_pairs_nested {
        let mut flat: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();
        for row in cand {
            for pair in row {
                flat.push(pair.clone());
            }
        }
        candidates_flat.push(flat);
    }
    // proof RLC
    let proof_rlc = self.sig_rlc_chip.fold_sig_list(
        layouter.namespace(|| "sig RLC(proof)"),
        &proof_flat,
    )?;
    
    // candidate RLC-k és match flag-ek (b_i)
    let mut b_flags: Vec<AssignedCell<Fp, Fp>> = Vec::new();

    for (i, cand) in candidates_flat.iter().enumerate() {
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

    // Compute product: prod(1 - b_i)
    let mut prod_not_b = Value::known(Fp::one());

    for b in b_flags.iter() {
        prod_not_b = prod_not_b.zip(b.value()).map(|(acc, bi)| {
            acc * (Fp::one() - *bi)
        });
    }   
    // Constraint: prod_not_b * (1 - is_fact) == 0
    layouter.assign_region(
        || "OR(success) constraint",
        |mut region| {
            let v = prod_not_b.zip(is_fact.value()).map(|(prod, f)| prod * (Fp::one() - *f));
            let check = region.assign_advice(|| "or_check", cfg.flag, 0, || v)?;
            let zero = region.assign_advice(|| "zero", cfg.flag, 1, || Value::known(Fp::zero()))?;
            region.constrain_equal(check.cell(), zero.cell())
        },
    )?;


    Ok(b_flags)
}

}
