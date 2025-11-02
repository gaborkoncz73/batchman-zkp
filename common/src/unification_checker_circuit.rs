use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, SimpleFloorPlanner},
    pasta::Fp,
    plonk::{Circuit, Column, ConstraintSystem, Error, Instance},
};
use crate::{
    chips::{
         fact_check::fact_hash_chip::{FactChip, FactConfig}, finding_rule::{body_subtree_chip::UnifCompareConfig, sig_check_chip::{SigCheckChip, SigCheckConfig}}, rlc_chip::RlcFixedChip, rules_check_chip::{RulesChip, RulesConfig}, value_check::{dot_chip::DotChip, rows_compress_config::{RowsCompressChip, RowsCompressConfig}, rule_rows_chip::{RuleRowsChip, RuleRowsConfig}}
    },
    data::{ClauseTemplateFp, PredicateTemplateFp, RuleTemplateFileFp, TermFp, UnificationInputFp},
    utils_2::{common_helpers::to_fp_value, consistency_helpers::{bind_goal_name_args_inputs/*, bind_rules*/}, predicate_helpers::bind_proof_and_candidates_sig_pairs},
};
use halo2_proofs::circuit::Value;
pub const MAX_DOT_DIM: usize = 10;

// Circuit definition
#[derive(Debug, Clone)]
pub struct UnificationCircuit {
    pub rules: RuleTemplateFileFp,
    pub unif: UnificationInputFp,
}

#[derive(Clone, Debug)]
pub struct UnifConfig {
    pub dot_cfg: <DotChip as Chip<Fp>>::Config,
    //pub hash_cfg: PoseidonHashConfig,
    pub rlc_cfg: <RlcFixedChip as Chip<Fp>>::Config,
    pub unif_cmp_cfg: UnifCompareConfig,
    pub sig_check_cfg: SigCheckConfig,
    pub rows_compress_chip :RowsCompressConfig,
    pub rule_rows_cfg: RuleRowsConfig,
    pub fact_cfg: FactConfig,
    //pub rules_check_cfg: RulesConfig,

    pub public_facts_hashes: Column<Instance>,
    //pub public_rules_hash: Column<Instance>,
}

impl Circuit<Fp> for UnificationCircuit {
    type Config = UnifConfig;
    type FloorPlanner = SimpleFloorPlanner;

     fn without_witnesses(&self) -> Self {
        Self {
            rules: RuleTemplateFileFp {
                predicates: Vec::new(), // empty but valid
            },
            unif: UnificationInputFp {
                goal_name: vec![TermFp::default()],
                subtree_goals: Vec::new(), // empty tree
            },
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let alpha = {
            to_fp_value("rlc_alpha_v1")
        };
        let dot_cfg = DotChip::configure(meta);
        
        let rlc_cfg = RlcFixedChip::configure(meta, alpha);
        let unif_cmp_cfg: UnifCompareConfig = UnifCompareConfig::configure(meta);
        let sig_check_cfg = SigCheckChip::configure(meta,alpha);
        let rows_compress_chip = RowsCompressChip::configure(meta);
        let rule_rows_cfg: RuleRowsConfig = RuleRowsChip::configure(meta);

        let public_facts_hashes = meta.instance_column();
        //let public_rules_hash = meta.instance_column();
       
        meta.enable_equality(public_facts_hashes);
        //meta.enable_equality(public_rules_hash);

        let fact_cfg = FactChip::configure(meta, public_facts_hashes);
        //let rules_check_cfg = RulesChip::configure(meta, public_rules_hash);

        UnifConfig {dot_cfg, rlc_cfg,unif_cmp_cfg, sig_check_cfg, rows_compress_chip, rule_rows_cfg, fact_cfg, public_facts_hashes/*, rules_check_cfg,  public_rules_hash*/ }
    }

    fn synthesize(
        &self,
        cfg: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error>
    {

    // Consistency check for Goal name + args == Term name + args == Unif goal name + args
    let (
        goal_name_cell,
        goal_name_arg_cells,
    ) = bind_goal_name_args_inputs(
        "Bind parent inputs",
        &mut layouter,
        &cfg.fact_cfg,
        &self.unif,
    )?;

    //Check if the used unification is a valid rule
    let (proof_pairs, candidate_pairs_all) = bind_proof_and_candidates_sig_pairs(
        "Bind proof + candidates (name,arity)",
        &mut layouter,
        &cfg.unif_cmp_cfg,
        &self.unif.goal_name,   // goal TermFp
        &self.unif.subtree_goals,   // subtree TermFp vec
        &self.rules.predicates, // predikátumok
    )?;
    
    // Rules-only check (fact ág kommentelve)
    let is_fact_cell = layouter.assign_region(
        || "is_fact",
        |mut region| {

            // 3. component, name, arity, ? than its fact
            let fact_val = proof_pairs
                .get(1)                      // első predikátum lista (goal head)
                .and_then(|row| row.get(0))  // annak első eleme
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
 //   println!()
    let sig_chip = SigCheckChip::construct(cfg.sig_check_cfg.clone());
    let b_flags = sig_chip.assign(
        layouter.namespace(|| "Sig membership (rules or fact placeholder)"),
        &proof_pairs,
        &candidate_pairs_all,
        &is_fact_cell,
    )?;
    /*let fact_hash_chip= FactChip::construct(cfg.fact_cfg.clone());

    let mut flag_copy_cells: Vec<AssignedCell<Fp, Fp>> = Vec::new();

    for (i, bflag) in b_flags.iter().enumerate() {
        let copied = layouter.assign_region(
            || format!("flag_copy_{i}"),
            |mut region| {
                region.assign_advice(
                    || "flag_copy",
                    cfg.fact_cfg.salt,
                    i+1,
                    || bflag.value().copied()
                )
            },
        )?;
        flag_copy_cells.push(copied.clone());
    }


    let goal_name_salt_cell = layouter.assign_region(
        || "assign goal_name_salt",
        |mut region| {
            region.assign_advice(
                || "goal salt",
                cfg.fact_cfg.salt, // any advice column, e.g., from FactConfig or your own column
                0,
                || Value::known(self.unif.goal_name.get(0).unwrap().fact_hashes), // or however your salt is stored
            )
        },
    )?;
    fact_hash_chip.assign(
        layouter.namespace(|| "Fact membership"),
        &goal_name_cell,
        &goal_name_arg_cells,
        &goal_name_salt_cell,
        &is_fact_local_for_fact_check,
        &flag_copy_cells,
    )?;*/


    // Helper: determinisztikus flatten offsetek (head + children).
    // Ezt a segítségeddel már tudod (pl. ClauseTemplate-ből):
    /*let rows_chip = RuleRowsChip::construct(cfg.rule_rows_cfg.clone());
    let mut all_clause_rows = Vec::new();
    for (p_i, pred) in self.rules.predicates.iter().enumerate() {
        for (c_i, clause) in pred.clauses.iter().enumerate() {

            let eqs_fp     = clause_equalities_as_index_tuples_fp(clause);
            let offsets_fp = offsets_for_clause_fp(pred, clause);

            let rows_ij = rows_chip.assign_rule_rows_fp(
                layouter.namespace(|| format!("rows for pred{}_clause{}", p_i, c_i)),
                &eqs_fp,
                &offsets_fp,
                MAX_DOT_DIM,
            )?;
            

            all_clause_rows.push(rows_ij);
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
        w_fp.extend_from_slice(&self.unif.goal_name.args);

        // body args (minden TermFp args)
        for t in &self.unif.subtree_goals {
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
        &is_fact_cell,
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
*/
            
    Ok(())
    }
}


/*pub fn clause_equalities_as_index_tuples_fp(
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
}*/