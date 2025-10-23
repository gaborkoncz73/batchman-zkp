use std::sync::Mutex;
use halo2_proofs::{
    circuit::{ Chip, Layouter, SimpleFloorPlanner},
    pasta::Fp,
    plonk::{Circuit, ConstraintSystem, Error},
};
use crate::{
    chips::{
         body_subtree_chip::{BodySubtreeChip, UnifCompareConfig}, poseidon_hash::{HashEqChip, HashEqConfig}, rlc_chip::RlcFixedChip, rlc_goal_check_chip::{RlcGoalCheckChip, RlcGoalCheckConfig}, rolc_compare_chip::RlcCompareChip, sig_check_chip::{SigCheckChip, SigCheckConfig}, ConsistencyChip, DotChip
    },
    data::{FactTemplateFp, PredicateTemplateFp, RuleTemplateFileFp, TermFp, UnificationInputFp},
    utils_2::{common_helpers::{to_fp_value,}, consistency_helpers::bind_goal_name_args_inputs, predicate_helpers::bind_proof_and_candidates_sig_pairs},
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
    pub unif_cmp_cfg: UnifCompareConfig,
    pub sig_check_cfg: SigCheckConfig,
}

impl Circuit<Fp> for UnificationCircuit {
    type Config = UnifConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
                rules: RuleTemplateFileFp {
                predicates: std::array::from_fn(|_| PredicateTemplateFp::default()),
                facts: std::array::from_fn(|_| FactTemplateFp::default()),
            },
            unif: UnificationInputFp {
                goal_name: TermFp::default(),
                goal_term_args: vec![],
                goal_term_name: Fp::zero(),
                unif_body: vec![],
                unif_goal: TermFp::default(),
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
        let unif_cmp_cfg = UnifCompareConfig::configure(meta);
        let sig_check_cfg = SigCheckChip::configure(meta,alpha);
        //let instance = meta.instance_column();
        //meta.enable_equality(instance);


        UnifConfig { cons_cfg, dot_cfg, hash_cfg, rlc_cfg, goal_check_cfg,unif_cmp_cfg, sig_check_cfg/*, instance*/ }
    }

    fn synthesize(
        &self,
        cfg: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        use halo2_proofs::circuit::Value;

        // ✅ Flatten the rules into Fp values
        /*let flat = crate::utils_2::common_helpers::flatten_rule_template_to_fp(&self.rules);
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

    // Consistency check for Goal name + args == Term name + args == Unif goal name + args
    let (
        goal_name_cell,
        goal_name_arg_cells,
        term_name_cell,
        term_arg_cells,
        unif_goal_name_cell,
        unif_goal_arg_cells,
    ) = bind_goal_name_args_inputs(
        "Bind parent inputs",
        &mut layouter,
        &cfg.goal_check_cfg,
        &self.unif,
    )?;

    let goal_check = RlcGoalCheckChip::construct(cfg.goal_check_cfg.clone());
    goal_check.assign(
        layouter.namespace(|| "GoalCheck"),
        &goal_name_cell,
        &goal_name_arg_cells,
        &term_name_cell,
        &term_arg_cells,
        &unif_goal_name_cell,
        &unif_goal_arg_cells,
    )?;

    // Consistency check for Subtree and Body 
    let body_chip = BodySubtreeChip::construct(cfg.unif_cmp_cfg.clone());
    let (body_pairs, subtree_pairs) = body_chip.assign(
        layouter.namespace(|| "Body-Subtree"),
        &self.unif.unif_body,
        &self.unif.subtree_goals,
    )?;

    let rlc_cmp = RlcCompareChip::construct(cfg.rlc_cfg.clone());
    rlc_cmp.assign_pairwise(
        layouter.namespace(|| "RLC compare"),
        &body_pairs,
        &subtree_pairs,
    )?;



    //Check if the used unification is a valid rule
    let (proof_pairs, candidate_pairs_all) = bind_proof_and_candidates_sig_pairs(
        "Bind proof + candidates (name,arity)",
        &mut layouter,
        &cfg.unif_cmp_cfg,
        &self.unif.goal_name,   // goal TermFp
        &self.unif.unif_body,   // body TermFp-k
        &self.rules.predicates, // predikátumok
    )?;

    // is_fact = (body üres?) — itt egyszerűen tanúként bevisszük:

    // Rules-only check (fact ág kommentelve)
    let sig_chip = SigCheckChip::construct(cfg.sig_check_cfg.clone());
    let is_fact_cell = layouter.assign_region(
        || "is_fact",
        |mut region| {
            // a 3. proof-pár második komponense (arity)
            let fact_val = proof_pairs.get(1)
                .map(|(_name, arity)| {
                    arity.value().map(|v| if *v == Fp::zero() { Fp::one() } else { Fp::zero() })
                })
                .unwrap_or(Value::known(Fp::zero()));

            region.assign_advice(
                || "is_fact",
                cfg.sig_check_cfg.sig_arity,
                0,
                || fact_val,
            )
        },
    )?;
    
    sig_chip.assign(
        layouter.namespace(|| "Sig membership (rules or fact placeholder)"),
        &proof_pairs,
        &candidate_pairs_all, // this can be &[] for fact
        &is_fact_cell,
    )?;

        /*if self.unif.subtree_goals.is_empty() {
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





