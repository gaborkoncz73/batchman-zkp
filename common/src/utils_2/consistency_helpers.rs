/*use halo2_proofs::{
    circuit::{Layouter, AssignedCell, Value},
    pasta::Fp,
    plonk::Error,
};

use crate::{chips::{fact_check::fact_hash_chip::FactConfig, rules_check_chip::RulesConfig}, data::RuleTemplateFileFp};
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

pub fn bind_rules(
    region_name: &str,
    layouter: &mut impl Layouter<Fp>,
    cfg: &RulesConfig,
    rules: &RuleTemplateFileFp,
) -> Result<Vec<AssignedCell<Fp, Fp>>,Error>
{
    layouter.assign_region(
        || region_name,
        |mut region| {
            let mut flattened_rules = Vec::new();
            let mut row: usize = 0 ;
            // Flattening predicates

            // Iterate over predicates
            for pred in rules.predicates.iter() {
                let pred_name_cell = region.assign_advice(
                    || "rule_name",
                    cfg.rules, 
                    row,
                    || Value::known(pred.name),
                )?;
                flattened_rules.push(pred_name_cell);

                let pred_arity_cell = region.assign_advice(
                    || "rule_arity",
                    cfg.rules, 
                    row+1,
                    || Value::known(pred.arity),
                )?;
                flattened_rules.push(pred_arity_cell);
                row += 2;

                // Iterate over clauses
                for clause in pred.clauses.iter() {

                    // Iterate over children
                    for child in clause.children.iter() {
                        let pred_ch_name_cell = region.assign_advice(
                            || "ch_name",
                            cfg.rules, 
                            row,
                            || Value::known(child.name),
                        )?;
                        flattened_rules.push(pred_ch_name_cell);

                        let pred_ch_arity_cell = region.assign_advice(
                            || "ch_arity",
                            cfg.rules, 
                            row+1,
                            || Value::known(child.arity),
                        )?;
                        flattened_rules.push(pred_ch_arity_cell);
                        row += 2;
                    }

                    // Iterate over equalities
                    for eq in clause.equalities.iter() {
                        let left_node_cell = region.assign_advice(
                            || "left_node",
                            cfg.rules, 
                            row,
                            || Value::known(eq.left.node),
                        )?;
                        flattened_rules.push(left_node_cell);

                        let left_arg_cell = region.assign_advice(
                            || "left_arg",
                            cfg.rules, 
                            row+1,
                            || Value::known(eq.left.arg),
                        )?;
                        flattened_rules.push(left_arg_cell);

                        let right_node_cell = region.assign_advice(
                            || "right_node",
                            cfg.rules, 
                            row+2,
                            || Value::known(eq.right.node),
                        )?;
                        flattened_rules.push(right_node_cell);

                        let right_arg_cell = region.assign_advice(
                            || "right_arg",
                            cfg.rules, 
                            row+3,
                            || Value::known(eq.right.arg),
                        )?;
                        flattened_rules.push(right_arg_cell);
                        row += 4;
                    }
                }
            } 

            // Iterate over facts
            for fact in rules.facts.iter() {
                let fact_name_cell = region.assign_advice(
                    || "fact_name",
                    cfg.rules, 
                    row,
                    || Value::known(fact.name),
                )?;
                flattened_rules.push(fact_name_cell);

                let fact_arity_cell = region.assign_advice(
                    || "fact_arity",
                    cfg.rules, 
                    row+1,
                    || Value::known(fact.arity),
                )?;
                flattened_rules.push(fact_arity_cell);
                row += 2;
            }

            Ok(flattened_rules)
        },
    )
}*/