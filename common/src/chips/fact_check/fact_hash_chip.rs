use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Instance},
};
use halo2curves::ff::Field;

use crate::{chips::fact_check::{built_in_check_chip::{BuiltinExprChip, BuiltinExprConfig}, poseidon_hash::{PoseidonHashChip, PoseidonHashConfig}}, utils_2::common_helpers::{MAX_CANDIDATES, MAX_FACTS_HASHES, to_fp_value}};

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

    pub builtin_cfg: BuiltinExprConfig,
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

        meta.enable_equality(name);
        meta.enable_equality(args);

        let pos_cfg = PoseidonHashChip::configure(meta);
        let builtin_cfg = BuiltinExprChip::configure(meta); 
        FactConfig { name, args, fact, salt, hash_public, hash_advice, is_fact, pos_cfg, builtin_cfg, }
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

    // 1) Beépített kifejezéslánc ellenőrzése (külön chip, külön namespace!)
    let builtin_chip = BuiltinExprChip::construct(cfg.builtin_cfg.clone());
    let builtin_ok = builtin_chip.eval_chain_equal(
        layouter.namespace(|| "builtin expr"),   // külön namespace
        &goal_name_cell,                         // p lista
        &goal_name_args_cells,                   // p -> a -> l
        false,                                   // itt nem kényszerítjük ok==1-re
    )?;
    println!("builtin: {:?}", builtin_ok);
    // 2) Hash-elés tokenláncról (név + minden arg[0] + salt)
    let pos_chip = PoseidonHashChip::construct(cfg.pos_cfg.clone());

    let mut tokens: Vec<AssignedCell<Fp, Fp>> = vec![goal_name_cell[0].clone()];

    if let Some(args_matrix) = goal_name_args_cells.get(0) {
        for arg_row in args_matrix {
            if let Some(first_arg_cell) = arg_row.get(0) {
                tokens.push(first_arg_cell.clone());
            }
        }
    }
    tokens.push(goal_name_salt.clone());

    let hashed = pos_chip.hash_list(
        layouter.namespace(|| "Poseidon(fact||salt)"),
        &tokens,
    )?;

    // 3) Membership + flags + builtin kombináció EGY régióban
    layouter.assign_region(
        || "final decision (membership OR flags OR builtin) gated by is_fact",
        |mut region| {
            // zero
            let zero = region.assign_advice(
                || "zero",
                cfg.salt, 1,
                || Value::known(Fp::ZERO),
            )?;

            // is_fact lokális másolat + kötés
            let is_fact_local = region.assign_advice(
                || "is_fact local",
                cfg.is_fact, 1,
                || is_fact.value().copied(),
            )?;
            region.constrain_equal(is_fact_local.cell(), is_fact.cell())?;

            // hashed lokális másolat + opcionális equality a külső 'hashed'-hez
            let hashed_local = region.assign_advice(
                || "hashed local",
                cfg.hash_advice, 0,
                || hashed.value().copied(),
            )?;
            region.constrain_equal(hashed_local.cell(), hashed.cell())?;

            // publikushash-szelekció (Σ bit_i*pub_i), és Σbit_i tanúk
            let mut pub_sel_val = Value::known(Fp::ZERO);
            let mut pub_sel_cell = zero.clone();

            for i in 0..MAX_FACTS_HASHES {
                let pub_local = region.assign_advice_from_instance(
                    || format!("pub[{i}]"),
                    cfg.hash_public,
                    i,
                    cfg.hash_advice,
                    i + 1,
                )?;

                let bit_i = region.assign_advice(
                    || format!("bit[{i}]"),
                    cfg.fact, i + 1,
                    || hashed_local.value().zip(pub_local.value())
                        .map(|(h,p)| if *h == *p { Fp::ONE } else { Fp::ZERO })
                )?;
                // bit_i booleanitás (kapuzva is_fact-tal)
                let bool_expr = region.assign_advice(
                    || format!("bit_bool[{i}]"),
                    cfg.salt, i + 2,
                    || is_fact_local.value().zip(bit_i.value())
                         .map(|(f,b)| f * *b * (*b - Fp::ONE)),
                )?;
                region.constrain_equal(bool_expr.cell(), zero.cell())?;

                pub_sel_val = pub_sel_val.zip(bit_i.value())
                    .zip(pub_local.value())
                    .map(|((acc,b),p)| acc + *b * *p);

                pub_sel_cell = region.assign_advice(
                    || format!("pub_sel[{i}]"),
                    cfg.hash_advice, MAX_FACTS_HASHES + 1 + i,
                    || pub_sel_val,
                )?;
            }

            // flags: any_flag = 1 - Π(1 - flag_i)
            let mut prod_not_flag_val = Value::known(Fp::ONE);
            let mut prod_not_flag_cell = zero.clone();
            for (i, b) in flags.iter().enumerate() {
                prod_not_flag_val = prod_not_flag_val.zip(b.value())
                    .map(|(acc, bi)| acc * (Fp::ONE - *bi));
                prod_not_flag_cell = region.assign_advice(
                    || format!("prod_not_flag[{i}]"),
                    cfg.salt, MAX_FACTS_HASHES*2 + 10 + i,
                    || prod_not_flag_val,
                )?;
            }
            let any_flag_cell = region.assign_advice(
                || "any_flag",
                cfg.salt, MAX_FACTS_HASHES*2 + 100,
                || prod_not_flag_cell.value().map(|p| Fp::ONE - *p),
            )?;

            // builtin_ok lokális másolat (hogy ebben a régióban is lásd)
            let builtin_ok_local = region.assign_advice(
                || "builtin_ok local",
                cfg.fact, MAX_FACTS_HASHES*2 + 101,
                || builtin_ok.value().copied(),
            )?;

            //  OR logika ZKP-biztosan: OR(a,b,c) = 1 - (1-a)(1-b)(1-c)
            let fact_ok_cell = region.assign_advice(
                || "fact_ok = [hashed==pub_sel]",
                cfg.fact, MAX_FACTS_HASHES*2 + 102,
                || hashed_local.value().zip(pub_sel_cell.value())
                     .map(|(h,ps)| if *h == *ps { Fp::ONE } else { Fp::ZERO })
            )?;

            let or_abc = region.assign_advice(
                || "or_abc",
                cfg.fact, MAX_FACTS_HASHES*2 + 103,
                || fact_ok_cell.value()
                    .zip(any_flag_cell.value())
                    .zip(builtin_ok_local.value())
                    .map(|((a,b),c)| {
                        let na = Fp::ONE - *a;
                        let nb = Fp::ONE - *b;
                        let nc = Fp::ONE - *c;
                        Fp::ONE - (na * nb * nc)
                    })
            )?;

            // final_ok = is_fact * or_abc
            let final_ok_cell = region.assign_advice(
                || "final_ok",
                cfg.fact, MAX_FACTS_HASHES*2 + 104,
                || is_fact_local.value().zip(or_abc.value())
                     .map(|(f, o)| *f * *o)
            )?;

            // final_ok == 1
            let diff = region.assign_advice(
                || "diff_final_ok",
                cfg.salt, MAX_FACTS_HASHES*2 + 105,
                || final_ok_cell.value().map(|v| *v - Fp::ONE),
            )?;
            region.constrain_equal(diff.cell(), zero.cell())?;

            Ok(())
        }
    )?;

    Ok(())
}


}