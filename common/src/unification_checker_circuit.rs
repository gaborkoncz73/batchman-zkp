use std::sync::Mutex;
use halo2_proofs::{
    circuit::{Chip, Layouter, SimpleFloorPlanner},
    pasta::Fp,
    plonk::{Circuit, ConstraintSystem, Error, Instance, Column},
};
use crate::{
    chips::{
        consistency_chip::ConsistencyConfig, poseidon_hash::{HashEqChip, HashEqConfig}, rlc_chip::RlcFixedChip, rlc_goal_check_chip::{RlcGoalCheckChip, RlcGoalCheckConfig}, ConsistencyChip, DotChip
    },
    data::{RuleTemplateFile, RuleTemplateFileFp, UnificationInputFp},
    utils_2::{
        common_helpers::to_fp_value, consistency_helpers::{build_consistency_pairs, creating_the_triple}, value_helpers::get_w_and_v_vec
    },
};
use once_cell::sync::Lazy;

pub const MAX_DOT_DIM: usize = 10;

// Global constraint counter
pub static TOTAL_CONSTRAINTS: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));

// Adds to the global constraint count
pub fn add_constraints(n: u64) {
    let mut counter = TOTAL_CONSTRAINTS.lock().unwrap();
    *counter += n;
}

// Reads the current total constraint count
pub fn get_constraints() -> u64 {
    let counter = TOTAL_CONSTRAINTS.lock().unwrap();
    *counter
}

// Circuit definition
#[derive(Debug, Clone)]
pub struct UnificationCircuit {
    pub rules: RuleTemplateFileFp,
    pub unif: UnificationInputFp,
}

#[derive(Clone, Debug)]
pub struct UnifConfig {
    pub cons_cfg: <ConsistencyChip as Chip<Fp>>::Config,
    pub dot_cfg: <DotChip as Chip<Fp>>::Config,
    pub hash_cfg: HashEqConfig,
    pub rlc_cfg: <RlcFixedChip as Chip<Fp>>::Config,
    pub goal_check_cfg: RlcGoalCheckConfig,
    //pub instance: Column<Instance>,
}

impl Circuit<Fp> for UnificationCircuit {
    type Config = UnifConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            rules: RuleTemplateFileFp {
                predicates: vec![],
                facts: vec![],
            },
            unif: UnificationInputFp {
                goal_name: Fp::zero(),
                goal_term_args: vec![],
                goal_term_name: Fp::zero(),
                unif_body: vec![],
                unif_goal: Fp::zero(),
                substitution: vec![],
                subtree_goals: vec![],
            },
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let alpha = {
            // pl. használd a már meglévő to_fp_value()-t
            to_fp_value("rlc_alpha_v1")
        };
        let cons_cfg = ConsistencyChip::configure(meta);
        let dot_cfg = DotChip::configure(meta);
        let hash_cfg = HashEqChip::configure(meta);
        let goal_check_cfg = RlcGoalCheckChip::configure(meta, alpha);

        
        let rlc_cfg = RlcFixedChip::configure(meta, alpha);

        //let instance = meta.instance_column();
        //meta.enable_equality(instance);

        UnifConfig { cons_cfg, dot_cfg, hash_cfg, rlc_cfg, goal_check_cfg/*, instance*/ }
    }

    fn synthesize(
        &self,
        cfg: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        use halo2_proofs::circuit::Value;

        /*// ✅ Flatten the rules into Fp values
        let flat = crate::utils_2::common_helpers::flatten_rule_template_to_fp(&self.rules);
        let leaves: Vec<Value<Fp>> = flat.iter().map(|&x| Value::known(x)).collect();

        // ✅ Expected hash from public instance
        let expected_cell = layouter.assign_region(
            || "load expected hash (instance)",
            |mut region| {
                region.assign_advice_from_instance(
                    || "expected hash",
                    cfg.instance,
                    0, // public input row = 0
                    cfg.hash_cfg.expected_col,
                    0,
                )
            },
        )?;

        // ✅ Compute Poseidon tree hash for all flattened Fp values
        let root_cell = HashEqChip::tree_hash_all(
            &cfg.hash_cfg,
            layouter.namespace(|| "rulehash full tree"),
            &leaves,
        )?;

        // ✅ Enforce equality: hash(root) == expected (public input)
        layouter.assign_region(
            || "enforce tree root == expected",
            |mut region| {
                region.constrain_equal(root_cell.cell(), expected_cell.cell())?;
                Ok(())
            },
        )?;*/

        // ✅ Continue with other checks
    let rlc_chip  = RlcFixedChip::construct(cfg.rlc_cfg.clone());

    let (goal_cell, unif_goal_cell, term_name_cell, term_arg_cells) = layouter.assign_region(
        || "Bind parent inputs",
        |mut region| {
            let goal_cell = region.assign_advice(
                || "goal_name_from_parent",
                cfg.goal_check_cfg.goal_name,
                0,
                || Value::known(self.unif.goal_name),
            )?;

            let unif_goal_cell = region.assign_advice(
                || "unif_goal_from_parent",
                cfg.goal_check_cfg.unif_goal,
                1,
                || Value::known(self.unif.unif_goal),
            )?;

            let term_name_cell = region.assign_advice(
                || "goal_term_name",
                cfg.goal_check_cfg.goal_name,
                2,
                || Value::known(self.unif.goal_term_name),
            )?;

            let mut term_arg_cells = Vec::new();
            for (i, arg) in self.unif.goal_term_args.iter().enumerate() {
                let cell = region.assign_advice(
                    || format!("goal_term_arg_{}", i),
                    cfg.goal_check_cfg.goal_name, // akár külön column is lehet
                    3 + i,
                    || Value::known(*arg),
                )?;
                term_arg_cells.push(cell);
            }

            Ok((goal_cell, unif_goal_cell, term_name_cell, term_arg_cells))
        },
    )?;


    let goal_check = RlcGoalCheckChip::construct(cfg.goal_check_cfg.clone());
    let combined_cell = goal_check.assign(
    layouter.namespace(|| "GoalCheck"),
    &goal_cell,
    &unif_goal_cell,
    &term_name_cell,
    &term_arg_cells,
)?;
    


        // (Consistency check)
        /*let triple = creating_the_triple(
            &self.unif.goal_name,
            &self.unif.goal_term_name,
            &self.unif.goal_term_args,
            &self.unif.unif_goal,
        )?;

        add_constraints(1);
        cons_chip.assign_pairs3(
            layouter.namespace(|| "goal_vs_unif_goal_vs_combined_term_goal"),
            triple,
        )?;

        let all_pairs: Vec<(Fp, Fp)> = build_consistency_pairs(
            &self.unif.goal_name,
            &self.unif.goal_term_args,
            &self.unif.unif_body,
            &self.unif.subtree_goals,
        )?;

        add_constraints(all_pairs.len() as u64);
        cons_chip.assign_pairs2(
            layouter.namespace(|| "goal_term_and_body_subtree_consistency"),
            &all_pairs,
        )?;

        if self.unif.subtree_goals.is_empty() {
            println!("Total constraints so far: {} (fact)", get_constraints());
            return Ok(());
        }

        let (matching_clause, structure_vec, witness_vec) =
            get_matching_structure_and_vectors(&self.unif, &self.rules)?;

        add_constraints(1);
        cons_chip.assign_pairs2(
            layouter.namespace(|| format!("pred_clause_dotcheck")),
            &[(structure_vec, witness_vec)],
        )?;

        let (padded_w_vec, padded_compressed_rows) = get_w_and_v_vec(
            &matching_clause,
            &self.unif.goal_term_args,
            &self.unif.unif_body,
            "seed",
            MAX_DOT_DIM,
        )?;

        add_constraints((MAX_DOT_DIM * 2 + 1) as u64);
        dot_chip.assign_dot_check(
            layouter.namespace(|| format!("variable_dot_check:")),
            &padded_w_vec,
            &padded_compressed_rows,
            Fp::zero(),
        )?;

        println!("Total constraints so far: {} (predicate)", get_constraints());*/
        Ok(())
    }
}
