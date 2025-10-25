use halo2_proofs::{
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem},
};
#[derive(Debug, Clone)]
pub struct UnifCompareConfig {
    pub proof_pairs: Column<Advice>,
    pub candidate_pairs: Column<Advice>,
}

impl UnifCompareConfig {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> Self {
        let proof_pairs = meta.advice_column();
        let candidate_pairs = meta.advice_column();
        
        meta.enable_equality(proof_pairs);
        meta.enable_equality(candidate_pairs);

        Self { proof_pairs, candidate_pairs }
    }
}
pub struct BodySubtreeChip {
    pub cfg: UnifCompareConfig,
}

impl BodySubtreeChip {
    pub fn construct(cfg: UnifCompareConfig) -> Self {
        Self { cfg }
    }
}