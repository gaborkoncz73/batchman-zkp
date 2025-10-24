use std::sync::Mutex;
use halo2_proofs::{
    circuit::{ AssignedCell, Chip, Layouter, Region, SimpleFloorPlanner},
    pasta::Fp,
    plonk::{Circuit, Column, ConstraintSystem, Error, Instance},
};
use crate::{
    chips::{
         body_subtree_chip::{BodySubtreeChip, UnifCompareConfig}, fact_hash_chip::{self, FactChip, FactConfig}, rlc_chip::RlcFixedChip, rlc_goal_check_chip::{RlcGoalCheckChip, RlcGoalCheckConfig}, rolc_compare_chip::RlcCompareChip, rows_compress_config::{RowsCompressChip, RowsCompressConfig}, rule_rows_chip::{RuleRowsChip, RuleRowsConfig}, sig_check_chip::{SigCheckChip, SigCheckConfig}, ConsistencyChip, DotChip
    },
    data::{ClauseTemplateFp, FactTemplateFp, PredicateTemplateFp, RuleTemplateFileFp, TermFp, UnificationInputFp},
    utils_2::{common_helpers::to_fp_value, consistency_helpers::bind_goal_name_args_inputs, predicate_helpers::bind_proof_and_candidates_sig_pairs},
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
    pub num_public_hashes: usize,
}

#[derive(Clone, Debug)]
pub struct UnifConfig {
    pub cons_cfg: <ConsistencyChip as Chip<Fp>>::Config,
    pub dot_cfg: <DotChip as Chip<Fp>>::Config,
    //pub hash_cfg: PoseidonHashConfig,
    pub rlc_cfg: <RlcFixedChip as Chip<Fp>>::Config,
    pub goal_check_cfg: RlcGoalCheckConfig,
    pub unif_cmp_cfg: UnifCompareConfig,
    pub sig_check_cfg: SigCheckConfig,
    pub rows_compress_chip :RowsCompressConfig,
    pub rule_rows_cfg: RuleRowsConfig,
    pub fact_cfg: FactConfig,

   pub instance_hashes: Column<Instance>,
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
            num_public_hashes: 3,
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let alpha = {
            // pl. használd a már meglévő to_fp_value()-t
            to_fp_value("rlc_alpha_v1")
        };
        let cons_cfg = ConsistencyChip::configure(meta);
        let dot_cfg = DotChip::configure(meta);
        //let hash_cfg = PoseidonHashChip::configure(meta);
        let goal_check_cfg = RlcGoalCheckChip::configure(meta, alpha);
        
        let rlc_cfg = RlcFixedChip::configure(meta, alpha);
        let unif_cmp_cfg: UnifCompareConfig = UnifCompareConfig::configure(meta);
        let sig_check_cfg = SigCheckChip::configure(meta,alpha);
        let rows_compress_chip = RowsCompressChip::configure(meta);
        let rule_rows_cfg: RuleRowsConfig = RuleRowsChip::configure(meta);



        // ⬇️ publikusan megadott hashek oszlopa
        let instance_hashes = meta.instance_column();
       
        meta.enable_equality(instance_hashes);

        let fact_cfg = FactChip::configure(meta, instance_hashes);


        UnifConfig { cons_cfg, dot_cfg, /*hash_cfg*/ rlc_cfg, goal_check_cfg,unif_cmp_cfg, sig_check_cfg, rows_compress_chip, rule_rows_cfg, fact_cfg, instance_hashes }
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

        let is_fact_local_for_fact_check = layouter.assign_region(
        || "copy is_fact for FactChip",
        |mut region| {
            let local = region.assign_advice(
                || "is_fact local copy",
                cfg.fact_cfg.is_fact,  // bármelyik advice col jó
                0,
                || is_fact_cell.value().copied(),
            )?;
            region.constrain_equal(local.cell(), is_fact_cell.cell())?;
            Ok(local)
        },
    )?;
    
    let b_flags = sig_chip.assign(
        layouter.namespace(|| "Sig membership (rules or fact placeholder)"),
        &proof_pairs,
        &candidate_pairs_all, // this can be &[] for fact
        &is_fact_cell,
    )?;
    
    //println!("goal: , {:?}:{:?}", self.unif.goal_name.name, self.unif.goal_name.args);
    //println!("b: {:?}\n\n", b_flags);

    // Helper: determinisztikus flatten offsetek (head + children).
    // Ezt a segítségeddel már tudod (pl. ClauseTemplate-ből):
    let rows_chip = RuleRowsChip::construct(cfg.rule_rows_cfg.clone());
    let target_len = b_flags.len();
    let mut built = 0usize;
    let mut all_clause_rows = Vec::with_capacity(target_len);

    'outer: for (p_i, pred) in self.rules.predicates.iter().enumerate() {
    for (c_i, clause) in pred.clauses.iter().enumerate() {
        if built == target_len { break 'outer; }

        let eqs_fp     = clause_equalities_as_index_tuples_fp(clause);
        let offsets_fp = offsets_for_clause_fp(pred, clause);

        let rows_ij = rows_chip.assign_rule_rows_fp(
            layouter.namespace(|| format!("rows for pred{}_clause{}", p_i, c_i)),
            &eqs_fp,
            &offsets_fp,
            pred.arity,
            MAX_DOT_DIM,
        )?;
        

        all_clause_rows.push(rows_ij);
        built += 1;
        }
    }

        let compress_chip = RowsCompressChip::construct(cfg.rows_compress_chip.clone());
        let compressed_vec: Vec<AssignedCell<Fp,Fp>> = compress_chip.assign_compressed_active_simple(
            layouter.namespace(|| "compress active clause rows (no r)"),
            &all_clause_rows,  // [clause][row][k]
            &b_flags,          // one-hot flags a SigCheck-ből
        )?;

        let w_cells: Vec<AssignedCell<Fp,Fp>> = {
        let mut w_fp: Vec<Fp> = Vec::new();

        // head goal args (Fp-k, mert UnificationInputFp)
        w_fp.extend_from_slice(&self.unif.goal_term_args);

        // body args (minden TermFp args)
        for t in &self.unif.unif_body {
            w_fp.extend_from_slice(&t.args);
        }

        // homogenitási 1
        w_fp.push(Fp::one());

        // pad MAX_DOT_DIM-re
        if w_fp.len() < MAX_DOT_DIM {
            w_fp.resize(MAX_DOT_DIM, Fp::zero());
        } else if w_fp.len() > MAX_DOT_DIM {
            // ha túlcsordulna, itt vágd/hibázd – tetszés szerint
            w_fp.truncate(MAX_DOT_DIM);
        }

        layouter.assign_region(
            || "bind w vec",
            |mut region| {
                let mut out = Vec::with_capacity(MAX_DOT_DIM);
                for (i, val) in w_fp.iter().copied().enumerate() {
                    let c = region.assign_advice(
                        || format!("w[{i}]"),
                        cfg.rows_compress_chip.val,
                        i,
                        || Value::known(val),
                    )?;
                    out.push(c);
                }
                Ok(out)
            },
        )?
    };
    let flag_cell = layouter.assign_region(
        || "dot flag assign",
        |mut region| {
            region.assign_advice(
                || "flag constant (numeric mode)",
                cfg.dot_cfg.adv_flag,
                0,
                || Value::known(Fp::zero()),  // boolean enforcement kikapcsolva
            )
        },
    )?;
    // 4) Dot check: <w, compressed> == 0
    let dot_chip = DotChip::construct(cfg.dot_cfg.clone());
    dot_chip.assign_dot_check(
        layouter.namespace(|| "dot(w, compressed) == 0"),
        &w_cells,
        &compressed_vec,
        &flag_cell,
        &is_fact_cell, // 👈 bekötve ide
    )?;

    let fact_hash_chip= FactChip::construct(cfg.fact_cfg.clone());

    let goal_name_salt_cell = layouter.assign_region(
    || "assign goal_name_salt",
    |mut region| {
        region.assign_advice(
            || "goal salt",
            cfg.fact_cfg.salt, // any advice column, e.g., from FactConfig or your own column
            0,
            || Value::known(self.unif.goal_name.fact_hashes), // or however your salt is stored
        )
    },
)?;

        fact_hash_chip.assign(
        layouter.namespace(|| "Fact membership"),
        &goal_name_cell,
        &goal_name_arg_cells,
        &goal_name_salt_cell,
        &is_fact_local_for_fact_check,
    )?;


        Ok(())
    }
}


pub fn clause_equalities_as_index_tuples_fp(
    clause: &ClauseTemplateFp,
) -> Vec<(Fp, Fp, Fp, Fp)> {
    clause.equalities.iter().map(|eq| {
        (
            eq.left.node,  // Fp
            eq.left.arg,   // Fp
            eq.right.node, // Fp
            eq.right.arg,  // Fp
        )
    }).collect()
}

pub fn offsets_for_clause_fp(pred: &PredicateTemplateFp, clause: &ClauseTemplateFp) -> Vec<Fp> {
    let mut offsets = Vec::new();
    let mut cur = Fp::zero();

    // Head offset (0)
    offsets.push(cur);

    // head arity (a predikátumé)
    cur += pred.arity;

    // gyermek node-ok
    for ch in &clause.children {
        offsets.push(cur);
        cur += ch.arity;
    }

    offsets
}