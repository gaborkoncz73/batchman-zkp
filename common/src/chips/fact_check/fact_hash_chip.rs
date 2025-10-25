use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Instance},
};
use halo2curves::ff::Field;

use crate::{chips::fact_check::poseidon_hash::{PoseidonHashChip, PoseidonHashConfig}, utils_2::common_helpers::MAX_FACTS_HASHES};

#[derive(Clone, Debug)]
pub struct FactConfig {
    pub name: Column<Advice>,
    pub args: Column<Advice>,
    pub fact: Column<Advice>,
    pub salt: Column<Advice>,
    pub hash_public: Column<Instance>,
    pub hash_advice: Column<Advice>,
    pub is_fact: Column<Advice>,
    pub pos_cfg: PoseidonHashConfig,
}

#[derive(Clone, Debug)]
pub struct FactChip {
    config: FactConfig,
}

impl Chip<Fp> for FactChip {
    type Config = FactConfig;
    type Loaded = ();
    fn config(&self) -> &Self::Config { &self.config }
    fn loaded(&self) -> &Self::Loaded { &() }
}

impl FactChip {
    pub fn construct (config: FactConfig) -> Self { Self { config }}

    pub fn configure(meta: &mut ConstraintSystem<Fp>, hash_public: Column<Instance>) -> FactConfig {
        let fact = meta.advice_column();
        let salt = meta.advice_column();
        let hash_advice = meta.advice_column();
        let name = meta.advice_column();
        let args = meta.advice_column();
        //let hash_public = meta.instance_column();
        let is_fact = meta.advice_column();
        meta.enable_equality(hash_public); 
        meta.enable_equality(fact);
        meta.enable_equality(salt);
        meta.enable_equality(hash_advice);
        meta.enable_equality(is_fact);

        let pos_cfg = PoseidonHashChip::configure(meta);
        FactConfig { name, args, fact, salt, hash_public, hash_advice, is_fact, pos_cfg }
    }
 pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        goal_name_cell: &AssignedCell<Fp, Fp>,
        goal_name_args_cells: &[AssignedCell<Fp, Fp>],
        goal_name_salt: &AssignedCell<Fp, Fp>,
        is_fact: &AssignedCell<Fp, Fp>,
    ) -> Result<(), Error> {
        let cfg = &self.config;
        let pos_chip = PoseidonHashChip::construct(cfg.pos_cfg.clone());

        // 1️⃣ Hash the fact (Poseidon(fact || salt))
        let tokens: Vec<AssignedCell<Fp, Fp>> = std::iter::once(goal_name_cell.clone())
            .chain(goal_name_args_cells.iter().cloned())
            .chain(std::iter::once(goal_name_salt.clone()))
            .collect();

        let hashed = pos_chip.hash_list(layouter.namespace(|| "Poseidon(fact||salt)"), &tokens)?;

        // 2️⃣ Membership check (gated by is_fact)
        layouter.assign_region(
            || "membership check (is_fact-gated)",
            |mut region| {
                // zero cell for equality constraints
                let zero = region.assign_advice(
                    || "zero",
                    cfg.salt,
                    1,
                    || Value::known(Fp::ZERO),
                )?;

                // Local copy of is_fact for gating
                let is_fact_local = region.assign_advice(
                    || "is_fact local",
                    cfg.is_fact,
                    1,
                    || is_fact.value().copied(),
                )?;
                region.constrain_equal(is_fact_local.cell(), is_fact.cell())?;

                // Local copy of hashed
                let hashed_local = region.assign_advice(
                    || "hashed local",
                    cfg.hash_advice,
                    0,
                    || hashed.value().copied(),
                )?;
                region.constrain_equal(hashed_local.cell(), hashed.cell())?;

                // Σ(bit_i) accumulator
                let mut sum_bits_val = Value::known(Fp::ZERO);
                let mut sum_bits_cell = zero.clone();

                // Σ(bit_i * pub_i) accumulator
                let mut pub_sel_val = Value::known(Fp::ZERO);
                let mut pub_sel_cell = zero.clone();
                 for i in 0..MAX_FACTS_HASHES {
                    // 1. Public hash copy from instance
                    let pub_local = region.assign_advice_from_instance(
                        || format!("pub[{i}]"),
                        cfg.hash_public,
                        i,
                        cfg.hash_advice,
                        i + 1,
                    )?;

                    // 2. Compute bit_i = 1 if hashed==pub_local else 0 (as witness)
                    let bit_i = region.assign_advice(
                        || format!("bit[{i}]"),
                        cfg.fact,
                        i + 1,
                        || hashed_local.value().zip(pub_local.value()).map(|(h, p)| {
                            if *h == *p { Fp::ONE } else { Fp::ZERO }
                        }),
                    )?;

                    // 3. Boolean constraint: is_fact * bit_i * (bit_i - 1) = 0
                    let bool_expr = region.assign_advice(
                        || format!("bool_expr[{i}]"),
                        cfg.salt,
                        i + 2,
                        || is_fact.value().zip(bit_i.value()).map(|(f, b)| f * *b * (*b - Fp::ONE)),
                    )?;
                    region.constrain_equal(bool_expr.cell(), zero.cell())?;

                    // Update Σbit_i
                    sum_bits_val = sum_bits_val.zip(bit_i.value()).map(|(acc, b)| acc + *b);
                    sum_bits_cell = region.assign_advice(
                        || format!("sum_bits[{i}]"),
                        cfg.fact,
                        MAX_FACTS_HASHES + 1 + i,
                        || sum_bits_val,
                    )?;

                    // Update Σ(bit_i * pub_i)
                    pub_sel_val = pub_sel_val.zip(bit_i.value()).zip(pub_local.value()).map(|((acc, b), p)| acc + *b * *p);
                    pub_sel_cell = region.assign_advice(
                        || format!("pub_sel[{i}]"),
                        cfg.hash_advice,
                        MAX_FACTS_HASHES + 1 + i,
                        || pub_sel_val,
                    )?;
                }


                
                // 4. Enforce exactly one match if is_fact=1: is_fact * (sum_bits - 1) = 0
                let sum_check = region.assign_advice(
                    || "sum_bits_check",
                    cfg.fact,
                    3 * MAX_FACTS_HASHES + 2,
                    || is_fact_local.value().zip(sum_bits_cell.value()).map(|(f, s)| f * (s - Fp::ONE)),
                )?;
                region.constrain_equal(sum_check.cell(), zero.cell())?;

                
                // 5. Enforce membership: is_fact * (hashed_local - pub_sel) = 0
                let mem_check = region.assign_advice(
                    || "membership check",
                    cfg.salt,
                    2 * MAX_FACTS_HASHES + 3,
                    || {
                        is_fact_local.value()
                            .zip(hashed_local.value())
                            .zip(pub_sel_cell.value())
                            .map(|((f, h), ps)| f * (*h - ps))
                    },
                )?;
                region.constrain_equal(mem_check.cell(), zero.cell())?;

                Ok(())
            },
        )?;

        Ok(())
    }


}

