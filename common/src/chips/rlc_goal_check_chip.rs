use halo2_proofs::{
    circuit::{Chip, Layouter, AssignedCell},
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
    goal_name_cell: &AssignedCell<Fp, Fp>,
    goal_name_arg_cells: &[AssignedCell<Fp, Fp>],
    goal_term_name_cell: &AssignedCell<Fp, Fp>,
    goal_term_arg_cells: &[AssignedCell<Fp, Fp>],
    unif_goal_name_cell: &AssignedCell<Fp, Fp>,
    unif_goal_arg_cells: &[AssignedCell<Fp, Fp>],
) -> Result<AssignedCell<Fp, Fp>, Error> {
    use halo2_proofs::circuit::Value;

    let rlc_chip = RlcFixedChip::construct(self.cfg.rlc_cfg.clone());

    // --- 1️⃣ Goal name + args fold ---
    let goal_tokens: Vec<Value<Fp>> = std::iter::once(goal_name_cell.value().map(|v| *v))
        .chain(goal_name_arg_cells.iter().map(|c| c.value().map(|v| *v)))
        .collect();
    let (goal_combined, goal_token_cells) =
        rlc_chip.assign(layouter.namespace(|| "RLC(goal_name,args)"), &goal_tokens)?;

    // --- 2️⃣ Term name + args fold ---
    let term_tokens: Vec<Value<Fp>> = std::iter::once(goal_term_name_cell.value().map(|v| *v))
        .chain(goal_term_arg_cells.iter().map(|c| c.value().map(|v| *v)))
        .collect();
    let (term_combined, term_token_cells) =
        rlc_chip.assign(layouter.namespace(|| "RLC(term_name,args)"), &term_tokens)?;

    // --- 3️⃣ Unif goal name + args fold ---
    let unif_tokens: Vec<Value<Fp>> = std::iter::once(unif_goal_name_cell.value().map(|v| *v))
        .chain(unif_goal_arg_cells.iter().map(|c| c.value().map(|v| *v)))
        .collect();
    let (unif_combined, unif_token_cells) =
        rlc_chip.assign(layouter.namespace(|| "RLC(unif_goal_name,args)"), &unif_tokens)?;

    // --- 4️⃣ Egyenlőségi feltételek ---
    layouter.assign_region(
        || "goal == term == unif consistency check",
        |mut region| {
            // goal == term
            region.constrain_equal(goal_combined.cell(), term_combined.cell())?;
            // goal == unif
            region.constrain_equal(goal_combined.cell(), unif_combined.cell())?;
            Ok(())
        },
    )?;

    // --- 5️⃣ Token binding (opcionális, ha debugolni akarsz) ---
    layouter.assign_region(
        || "bind each RLC token (goal/term/unif)",
        |mut region| {
            // Goal: name + args == tokens
            region.constrain_equal(goal_name_cell.cell(), goal_token_cells[0].cell())?;
            for (i, a) in goal_name_arg_cells.iter().enumerate() {
                region.constrain_equal(a.cell(), goal_token_cells[i + 1].cell())?;
            }

            // Term: name + args == tokens
            region.constrain_equal(goal_term_name_cell.cell(), term_token_cells[0].cell())?;
            for (i, a) in goal_term_arg_cells.iter().enumerate() {
                region.constrain_equal(a.cell(), term_token_cells[i + 1].cell())?;
            }

            // Unif: name + args == tokens
            region.constrain_equal(unif_goal_name_cell.cell(), unif_token_cells[0].cell())?;
            for (i, a) in unif_goal_arg_cells.iter().enumerate() {
                region.constrain_equal(a.cell(), unif_token_cells[i + 1].cell())?;
            }

            Ok(())
        },
    )?;

    // --- 6️⃣ Visszaadjuk a goal_combined cellt (fő RLC output) ---
    Ok(goal_combined)
}




}
