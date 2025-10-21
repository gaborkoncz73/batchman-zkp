use halo2_proofs::{
    circuit::{Chip, Layouter, Value, AssignedCell},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error},
};

use crate::{chips::rlc_chip::{RlcFixedChip, RlcFixedConfig}, utils_2::common_helpers::MAX_ARITY};

#[derive(Clone, Debug)]
pub struct RlcGoalCheckConfig {
    pub goal_name: Column<Advice>,
    pub unif_goal: Column<Advice>,
    pub rlc_cfg: RlcFixedConfig,
}

pub struct RlcGoalCheckChip {
    cfg: RlcGoalCheckConfig,
}

impl Chip<Fp> for RlcGoalCheckChip {
    type Config = RlcGoalCheckConfig;
    type Loaded = ();
    fn config(&self) -> &Self::Config { &self.cfg }
    fn loaded(&self) -> &Self::Loaded { &() }
}

impl RlcGoalCheckChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>, alpha: Fp) -> RlcGoalCheckConfig {
        let goal_name = meta.advice_column();
        let unif_goal = meta.advice_column();
        meta.enable_equality(goal_name);
        meta.enable_equality(unif_goal);

        let rlc_cfg = RlcFixedChip::configure(meta, alpha);

        RlcGoalCheckConfig { goal_name, unif_goal, rlc_cfg }
    }

    pub fn construct(cfg: RlcGoalCheckConfig) -> Self { Self { cfg } }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        goal_cell: &AssignedCell<Fp, Fp>,
        unif_goal_cell: &AssignedCell<Fp, Fp>,
        goal_term_name_cell: &AssignedCell<Fp, Fp>,
        goal_term_arg_cells: &[AssignedCell<Fp, Fp>],
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        use halo2_proofs::circuit::Value;

        // --- 1️⃣ Előkészítjük a token-értékeket (Value-k) ---
        let mut tokens_val: Vec<Value<Fp>> = std::iter::once(goal_term_name_cell.value().map(|v| *v))
            .chain(goal_term_arg_cells.iter().map(|a| a.value().map(|v| *v)))
            .collect();

        while tokens_val.len() < MAX_ARITY {
            tokens_val.push(Value::known(Fp::zero()));
        }

        // --- 2️⃣ Lefuttatjuk az RLC foldot (kapjuk: acc_last és a token cellák) ---
        let rlc_chip = RlcFixedChip::construct(self.cfg.rlc_cfg.clone());
        let (combined_cell, rlc_token_cells) =
            rlc_chip.assign(layouter.namespace(|| "RLC(term_name,args)"), &tokens_val)?;

        // --- 3️⃣ Kapcsoljuk össze a goal/unif mezőket az RLC outputjával ---
        println!("goal: {:?}", goal_cell.value());
        println!("term: {:?}", combined_cell.value());
        println!("unif: {:?}", unif_goal_cell.value());
        layouter.assign_region(
            || "goal/unif == combined",
            |mut region| {
                region.constrain_equal(goal_cell.cell(), combined_cell.cell())?;
                region.constrain_equal(unif_goal_cell.cell(), combined_cell.cell())?;
                Ok(())
            },
        )?;

        // --- 4️⃣ Kapcsoljuk össze a term_name és term_args cellákat az RLC token cellákkal ---
        layouter.assign_region(
            || "bind term_name/args to RLC tokens",
            |mut region| {
                // 0. token = term_name
                region.constrain_equal(goal_term_name_cell.cell(), rlc_token_cells[0].cell())?;

                // a többi token = term_args[i]
                for (i, arg_cell) in goal_term_arg_cells.iter().enumerate() {
                    region.constrain_equal(arg_cell.cell(), rlc_token_cells[i + 1].cell())?;
                }

                Ok(())
            },
        )?;

        // --- 5️⃣ Visszaadjuk a combined cell-t ---
        Ok(combined_cell)
    }



}
