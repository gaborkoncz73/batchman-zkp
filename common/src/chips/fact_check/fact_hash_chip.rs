use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Instance},
};
use halo2curves::ff::Field;

use crate::{chips::fact_check::poseidon_hash::{PoseidonHashChip, PoseidonHashConfig}, utils_2::common_helpers::{MAX_CANDIDATES, MAX_FACTS_HASHES, to_fp_value}};

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
        goal_name_cell: &[AssignedCell<Fp, Fp>],
        goal_name_args_cells: &[Vec<Vec<AssignedCell<Fp, Fp>>>],
        goal_name_salt: &AssignedCell<Fp, Fp>,
        is_fact: &AssignedCell<Fp, Fp>,
        flags:  &Vec<AssignedCell<Fp,Fp>>,   
    ) -> Result<(), Error> {
        let cfg = &self.config;
        let pos_chip = PoseidonHashChip::construct(cfg.pos_cfg.clone());
        let mut tokens: Vec<AssignedCell<Fp, Fp>> =
        vec![goal_name_cell[0].clone()];
        // goal_argument_cells[p][a][l]
        if let Some(args_matrix) = goal_name_args_cells.get(0) {
            for arg_row in args_matrix {
                if let Some(first_arg_cell) = arg_row.get(0) {
                    tokens.push(first_arg_cell.clone());
                    
                }
            }
        }
        
        // finally salt
        tokens.push(goal_name_salt.clone());
        let hashed = pos_chip.hash_list(
            layouter.namespace(|| "Poseidon(fact||salt)"),
            &tokens,
        )?;
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
                //let mut sum_bits_cell = zero.clone();

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
                    /*sum_bits_cell = region.assign_advice(
                        || format!("sum_bits[{i}]"),
                        cfg.fact,
                        MAX_FACTS_HASHES + 1 + i,
                        || sum_bits_val,
                    )?;*/

                    // Update Σ(bit_i * pub_i)
                    pub_sel_val = pub_sel_val.zip(bit_i.value()).zip(pub_local.value()).map(|((acc, b), p)| acc + *b * *p);
                    pub_sel_cell = region.assign_advice(
                        || format!("pub_sel[{i}]"),
                        cfg.hash_advice,
                        MAX_FACTS_HASHES  + 1 + i,
                        || pub_sel_val,
                    )?;
                }

                // ✅ Compute product of NOT flags = Π (1 - flag[i])
                let mut prod_not_flag_val = Value::known(Fp::ONE);
                let mut prod_not_flag_cell = zero.clone();

                for (i, b) in flags.iter().enumerate() {
                    prod_not_flag_val = prod_not_flag_val.zip(b.value()).map(|(acc, bi)| {
                        acc * (Fp::ONE - *bi)
                    });

                    prod_not_flag_cell = region.assign_advice(
                        || format!("prod_not_flag[{i}]"),
                        cfg.salt,
                        i + MAX_CANDIDATES + MAX_FACTS_HASHES + 1, // ✅ just use unused rows
                        || prod_not_flag_val,
                    )?;
                }

                // ✅ any_flag = 1 - Π(1 - flag_i)  (1 ha volt legalább egy 1-es)
                let any_flag_cell = region.assign_advice(
                    || "any_flag",
                    cfg.salt,
                    MAX_CANDIDATES*2 + MAX_FACTS_HASHES + 1,
                    || prod_not_flag_cell.value().map(|p| Fp::ONE - *p),
                )?;

                
                // ✅ OR(logic): either hash matches OR rule_flags justify it
                let final_ok_val = is_fact_local.value()
                    .zip(any_flag_cell.value())
                    .zip(hashed_local.value())
                    .zip(pub_sel_cell.value())
                    .map(|(((f, af), h), ps)| {
                        // OK if:
                        //   (f & af) == 1    (flag condition OK)
                        //   OR
                        //   (f & (h == ps))  (normal hashed membership OK)
                        let flag_ok = *f * *af;
                        let hash_ok = *f * (if h == ps { Fp::ONE } else { Fp::ZERO });
                        flag_ok + hash_ok
                    });

                let final_ok_cell = region.assign_advice(
                    || "final_ok",
                    cfg.fact,
                    210,
                    || final_ok_val,
                )?;

                // ✅ must equal 1
                let diff = region.assign_advice(
                    || "diff_final_ok",
                    cfg.salt,
                    211,
                    || final_ok_cell.value().map(|v| *v - Fp::ONE),
                )?;
                region.constrain_equal(diff.cell(), zero.cell())?;


                Ok(())
            },
        )?;

        Ok(())
    }


}

