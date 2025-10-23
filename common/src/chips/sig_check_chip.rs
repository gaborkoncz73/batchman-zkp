use halo2_proofs::{
    circuit::{ Layouter, AssignedCell},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Selector},
};

use crate::chips::{
    rlc_chip::{RlcFixedChip, RlcFixedConfig},
    sig_rlc_chip::SigRlcChip,
    sig_or_compare_chip::SigOrCompareChip,
};
use crate::utils_2::common_helpers::MAX_CANDIDATES;

/// ─────────────────────────────
/// CONFIG
/// ─────────────────────────────
#[derive(Clone, Debug)]
pub struct SigCheckConfig {
    pub sig_name: Column<Advice>,
    pub sig_arity: Column<Advice>,
    pub flag: Column<Advice>,
    pub q_bool: Column<Fixed>,
    pub q_sum: Selector,
    pub rlc_cfg: RlcFixedConfig,
}

#[derive(Clone, Debug)]
pub struct SigCheckChip {
    pub cfg: SigCheckConfig,
}

impl SigCheckChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>, alpha: Fp) -> SigCheckConfig {
        let sig_name  = meta.advice_column();
        let sig_arity = meta.advice_column();
        let flag      = meta.advice_column();
        let q_bool    = meta.fixed_column();
        let q_sum     = meta.selector();

        // Equality permisson
        meta.enable_equality(sig_name);
        meta.enable_equality(sig_arity);
        meta.enable_equality(flag);

        // Booleanity gate
        meta.create_gate("flag booleanity", |meta| {
            let q  = meta.query_fixed(q_bool);
            let b  = meta.query_advice(flag, halo2_proofs::poly::Rotation::cur());
            vec![ q * b.clone() * (b - halo2_proofs::plonk::Expression::Constant(Fp::one())) ]
        });

        // Sum flags
        meta.create_gate("sum flags == 1", |meta| {
            let q  = meta.query_selector(q_sum);
            let mut sum_expr = meta.query_advice(flag, halo2_proofs::poly::Rotation::cur());
            for i in 1..MAX_CANDIDATES {
                sum_expr = sum_expr + meta.query_advice(flag, halo2_proofs::poly::Rotation(i as i32));
            }
            vec![ q * (sum_expr - halo2_proofs::plonk::Expression::Constant(Fp::one())) ]
        });


        let rlc_cfg = RlcFixedChip::configure(meta, alpha);

        SigCheckConfig { sig_name, sig_arity, flag, q_bool, q_sum, rlc_cfg }
    }

    pub fn construct(cfg: SigCheckConfig) -> Self {
        Self { cfg }
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        proof_pairs: &[(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)],
        candidate_pairs_all: &[Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>],
        is_fact: &AssignedCell<Fp, Fp>,
    ) -> Result<Vec<AssignedCell<Fp, Fp>>, Error> { 
        let sig_rlc_chip = SigRlcChip::construct(self.cfg.rlc_cfg.clone());
        let or_chip = SigOrCompareChip::construct(self.cfg.clone(), sig_rlc_chip);

        let b_flags = or_chip.check_membership_or(
            layouter.namespace(|| "SigCheck main OR"),
            proof_pairs,
            candidate_pairs_all,
            is_fact,
        )?;

        Ok(b_flags)
    }

}
