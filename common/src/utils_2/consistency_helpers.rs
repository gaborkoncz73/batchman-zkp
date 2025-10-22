use halo2_proofs::{
    circuit::{Layouter, AssignedCell, Value},
    pasta::Fp,
    plonk::Error,
};

use crate::chips::rlc_goal_check_chip::RlcGoalCheckConfig;
use crate::data::UnificationInputFp;

/// Segédfüggvény a goal, unif_goal és term mezők bekötéséhez.
/// Ez lesz hívva a fő circuit synthesize-ban.
pub fn bind_goal_name_args_inputs(
    region_name: &str,
    layouter: &mut impl Layouter<Fp>,
    cfg: &RlcGoalCheckConfig,
    unif: &UnificationInputFp,
) -> Result<
    (
        AssignedCell<Fp, Fp>,                     // goal_name_cell
        Vec<AssignedCell<Fp, Fp>>,                // goal_name_arg_cells
        AssignedCell<Fp, Fp>,                     // unif_goal_name_cell
        Vec<AssignedCell<Fp, Fp>>,                // unif_goal_arg_cells
        AssignedCell<Fp, Fp>,                     // term_name_cell
        Vec<AssignedCell<Fp, Fp>>,                // term_arg_cells
    ),
    Error,
> {
    layouter.assign_region(
        || region_name,
        |mut region| {
            // Goal name
            let goal_name_cell = region.assign_advice(
                || "goal_name",
                cfg.goal_name,
                0,
                || Value::known(unif.goal_name.name),
            )?;
            //println!("goal: {:?} term: {:?} unif: {:?}", unif.goal_name.args.len(), unif.goal_term_args.len(), unif.unif_goal.args.len());
            // Goal args
            let mut goal_name_arg_cells = Vec::new();
            for (i, arg) in unif.goal_name.args.iter().enumerate() {
                let c = region.assign_advice(
                    || format!("goal_name_arg_{i}"),
                    cfg.goal_name,
                    1 + i,
                    || Value::known(*arg),
                )?;
                goal_name_arg_cells.push(c);
            }

            // Goal term name
            let term_name_cell = region.assign_advice(
                || "term_name",
                cfg.term_goal,
                0,
                || Value::known(unif.goal_term_name)
            )?;

            // Goal term args 
            let mut term_arg_cells=Vec::new();
            for(i,arg) in unif.goal_term_args.iter().enumerate() {
                let t = region.assign_advice(
                    || format!("goal_term_arg_{i}"),
                    cfg.term_goal,
                    1 + i,
                    || Value::known(*arg)
                )?;
                term_arg_cells.push(t);
            }

            // Unif goal name
            let unif_goal_name_cell = region.assign_advice(
                || "unif_goal_name",
                cfg.unif_goal,
                0,
                || Value::known(unif.unif_goal.name),
            )?;

            // Unif goal args
            let mut unif_goal_arg_cells = Vec::new();
            for (i, arg) in unif.unif_goal.args.iter().enumerate() {
                let c = region.assign_advice(
                    || format!("unif_goal_arg_{i}"),
                    cfg.unif_goal,
                    1 + i,
                    || Value::known(*arg),
                )?;
                unif_goal_arg_cells.push(c);
            }

            Ok((
                goal_name_cell,
                goal_name_arg_cells,
                term_name_cell,
                term_arg_cells,
                unif_goal_name_cell,
                unif_goal_arg_cells,
            ))
        },
    )
}