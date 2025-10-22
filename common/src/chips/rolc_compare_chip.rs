use halo2_proofs::{circuit::{AssignedCell, Layouter}, pasta::Fp, plonk::Error};
use crate::chips::rlc_chip::{RlcFixedChip, RlcFixedConfig};

pub struct RlcCompareChip {
    pub cfg: RlcFixedConfig,
}

impl RlcCompareChip {
    pub fn construct(cfg: RlcFixedConfig) -> Self {
        Self { cfg }
    }

    pub fn assign_pairwise(
        &self,
        mut layouter: impl Layouter<Fp>,
        left: &[(AssignedCell<Fp, Fp>, Vec<AssignedCell<Fp, Fp>>)],
        right: &[(AssignedCell<Fp, Fp>, Vec<AssignedCell<Fp, Fp>>)],
    ) -> Result<(), Error> {
        let rlc = RlcFixedChip::construct(self.cfg.clone());
        rlc.cmp_term_lists_pairwise_with_rlc_cells(
            layouter.namespace(|| "RLC body-subtree compare"),
            left,
            right,
        )
    }
}
