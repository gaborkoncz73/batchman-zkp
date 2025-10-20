use std::sync::Mutex;
use halo2_proofs::{
    circuit::{Chip, Layouter, SimpleFloorPlanner}, pasta::Fp, plonk::{Circuit, ConstraintSystem, Error}
};
use crate::{chips::{ConsistencyChip, DotChip}, data::UnificationInput, utils_2::{consistency_helpers::{build_consistency_pairs, creating_the_triple}, predicate_helpers::get_matching_structure_and_vectors, value_helpers::get_w_and_v_vec}};
use crate::data::RuleTemplateFile;
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
    pub rules: RuleTemplateFile,
    pub unif: UnificationInput,
}

#[derive(Clone, Debug)]
pub struct UnifConfig {
    pub cons_cfg: <ConsistencyChip as Chip<Fp>>::Config,
    pub dot_cfg: <DotChip as Chip<Fp>>::Config,
}

impl Circuit<Fp> for UnificationCircuit {
    type Config = UnifConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            rules: RuleTemplateFile { predicates: vec![], facts: vec![] },
            unif: UnificationInput {
                goal_name: String::new(),
                goal_term_args: vec![],
                goal_term_name: String::new(),
                unif_body: vec![],
                unif_goal: String::new(),
                substitution: vec![],
                subtree_goals: vec![],
            },
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let cons_cfg = ConsistencyChip::configure(meta);
        let dot_cfg = DotChip::configure(meta);
        UnifConfig { cons_cfg  , dot_cfg }
    }

    fn synthesize(
        &self,
        cfg: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let cons_chip = ConsistencyChip::construct(cfg.cons_cfg);
        let dot_chip = DotChip::construct(cfg.dot_cfg);

        // (Consistency check)
        let triple = creating_the_triple(
            &self.unif.goal_name,
            &self.unif.goal_term_name,
            &self.unif.goal_term_args,
            &self.unif.unif_goal)?;

        // Add 1 to the constraint because a cons_chip costs 1 constraint/region
        add_constraints(1);
        cons_chip.assign_pairs3(
            layouter.namespace(|| "goal_vs_unif_goal_vs_combined_term_goal"),
            triple,
        )?;
        
        let all_pairs: Vec<(Fp, Fp)> = build_consistency_pairs(
            &self.unif.goal_name,
            &self.unif.goal_term_args,
            &self.unif.unif_body,
            &self.unif.subtree_goals)?;

        // check of 1 pair costs 1 constraint so the amount of pairs determine the new constraints
        add_constraints(all_pairs.len() as u64);
        cons_chip.assign_pairs2(
            layouter.namespace(|| "goal_term_and_body_subtree_consistency"),
            &all_pairs,
        )?;

        // FACT CHECK COMES HERE
        if self.unif.subtree_goals.is_empty(){
            println!("Total constraints so far: {} (fact)", get_constraints());
            return Ok(());
        }

        // Finding and proving the existence of the matching predicate
        let (matching_clause, structure_vec, witness_vec) = get_matching_structure_and_vectors(
            &self.unif,
            &self.rules
        )?;
            
        add_constraints(1);
        cons_chip.assign_pairs2(
            layouter.namespace(|| format!("pred_clause_dotcheck")),
            &[(structure_vec,witness_vec)],
        )?;

        // Constructing the equations for the variable check
        let (padded_w_vec,padded_compressed_rows) = get_w_and_v_vec(
            &matching_clause,
            &self.unif.goal_term_args,
            &self.unif.unif_body,
            "seed",
            MAX_DOT_DIM
        )?;
        
        add_constraints((MAX_DOT_DIM*2+1) as u64);
        dot_chip.assign_dot_check(
            layouter.namespace(|| format!("variable_dot_check:")),
            &padded_w_vec, 
            &padded_compressed_rows,
            Fp::zero(),
        )?;

        println!("Total constraints so far: {} (predicate)", get_constraints());
        Ok(())
    }
}
