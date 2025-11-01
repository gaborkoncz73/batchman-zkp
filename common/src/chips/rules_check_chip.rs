use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Instance},
};
use halo2curves::ff::Field;

use crate::{chips::fact_check::poseidon_hash::{PoseidonHashChip, PoseidonHashConfig}, utils_2::common_helpers::MAX_RULE_COMPONENTS};

#[derive(Clone, Debug)]
pub struct RulesConfig {
    pub rules: Column<Advice>,
    pub public_rules_hash: Column<Instance>,
    pub pos_cfg: PoseidonHashConfig,
}

#[derive(Clone, Debug)]
pub struct RulesChip {
    config: RulesConfig,
}

impl Chip<Fp> for RulesChip {
    type Config = RulesConfig;
    type Loaded = ();
    fn config(&self) -> &Self::Config { &self.config }
    fn loaded(&self) -> &Self::Loaded { &() }
}

impl RulesChip {
    pub fn construct (config: RulesConfig) -> Self { Self { config }}

    pub fn configure(meta: &mut ConstraintSystem<Fp>, public_rules_hash: Column<Instance>) -> RulesConfig {
        let rules = meta.advice_column();

        meta.enable_equality(rules); 
        meta.enable_equality(public_rules_hash);

        let pos_cfg = PoseidonHashChip::configure(meta);

        RulesConfig { rules, public_rules_hash, pos_cfg }
    }
    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        flatten_rules: &[AssignedCell<Fp, Fp>],
    ) -> Result<(), Error> {
        let cfg = &self.config;
        let pos_chip = PoseidonHashChip::construct(cfg.pos_cfg.clone());


        let hashed = pos_chip.hash_list(layouter.namespace(|| "Poseidon(rules_flatten)"), &flatten_rules)?;

        // Membership check (gated by is_fact)
        layouter.assign_region(
            || "membership check (is_fact-gated)",
            |mut region| {
                // zero cell for equality constraints
                let zero = region.assign_advice(
                    || "zero",
                    cfg.rules,
                    MAX_RULE_COMPONENTS+1,
                    || Value::known(Fp::ZERO),
                )?;

                let pub_rules_local = region.assign_advice_from_instance(
                    || format!("pub_hash]"),
                    cfg.public_rules_hash,
                    0,
                    cfg.rules,
                    MAX_RULE_COMPONENTS+2,
                )?;

                let hashed_local = region.assign_advice(
                    || "hashed local",
                    cfg.rules,
                    MAX_RULE_COMPONENTS+3,
                    || hashed.value().copied(),
                )?;

                // Equals?
                let equal = region.assign_advice(
                    || format!("check"),
                    cfg.rules,
                    MAX_RULE_COMPONENTS+4,
                    || hashed_local.value()
                            .zip(pub_rules_local.value())
                            .map(|(h, p)| {
                                if *h == *p { Fp::ZERO } else { Fp::ONE }
                    }),
                )?;
                region.constrain_equal(equal.cell(), zero.cell())?;
                Ok(())
            },
        )?;
        Ok(())
    }
}