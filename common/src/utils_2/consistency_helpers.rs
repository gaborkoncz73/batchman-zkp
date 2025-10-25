use halo2_proofs::{
    circuit::{Layouter, AssignedCell, Value},
    pasta::Fp,
    plonk::Error,
};

use crate::chips::{fact_check::fact_hash_chip::FactConfig};
use crate::data::UnificationInputFp;

/// Segédfüggvény a goal, unif_goal és term mezők bekötéséhez.
/// Ez lesz hívva a fő circuit synthesize-ban.
pub fn bind_goal_name_args_inputs(
    region_name: &str,
    layouter: &mut impl Layouter<Fp>,
    cfg: &FactConfig,
    unif: &UnificationInputFp,
) -> Result<
    (
        AssignedCell<Fp, Fp>,                     // goal_name_cell
        Vec<AssignedCell<Fp, Fp>>,                // goal_name_arg_cells
    ),
    Error,
> {
    layouter.assign_region(
        || region_name,
        |mut region| {
            // Goal name
            let goal_name_cell: AssignedCell<Fp, Fp> = region.assign_advice(
                || "goal_name",
                cfg.name,
                0,
                || Value::known(unif.goal_name.name),
            )?;
            // Goal args
            let mut goal_name_arg_cells = Vec::new();
            for (i, arg) in unif.goal_name.args.iter().enumerate() {
                let c = region.assign_advice(
                    || format!("goal_name_arg_{i}"),
                    cfg.args,
                    i,
                    || Value::known(*arg),
                )?;
                goal_name_arg_cells.push(c);
            }

            Ok((
                goal_name_cell,
                goal_name_arg_cells,
            ))
        },
    )
}