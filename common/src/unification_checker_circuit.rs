use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, SimpleFloorPlanner},
    pasta::Fp,
    plonk::{Circuit, Column, ConstraintSystem, Error, Instance},
};
use crate::{
    chips::{
         fact_check::fact_hash_chip::{FactChip, FactConfig}, finding_rule::{body_subtree_chip::UnifCompareConfig, sig_check_chip::{SigCheckChip, SigCheckConfig}}, rlc_chip::RlcFixedChip, rules_check_chip::{RulesChip, RulesConfig}, value_check::{dot_chip::DotChip, rows_compress_config::{RowsCompressChip, RowsCompressConfig}, rule_rows_chip::{RuleRowsChip, RuleRowsConfig}}
    },
    data::{ClauseTemplateFp, PredicateTemplateFp, RuleTemplateFileFp, TermFp, TermSideFp, UnificationInputFp},
    utils_2::{common_helpers::{MAX_ARITY, MAX_CHILDREN, MAX_PRED_LIST, to_fp_value}, consistency_helpers::bind_goal_name_args_inputs, predicate_helpers::bind_proof_and_candidates_sig_pairs},
};
use halo2_proofs::circuit::Value;
pub const PER_TERM: usize  = MAX_ARITY * MAX_PRED_LIST;
pub const PER_NODE: usize  = MAX_PRED_LIST * PER_TERM;
pub const MAX_NODES: usize = 1 + MAX_CHILDREN;
pub const MAX_DOT_DIM: usize = MAX_NODES * PER_NODE + 1;

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
    let fact_hash_chip= FactChip::construct(cfg.fact_cfg.clone());

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
    )?;


    // Helper: determinisztikus flatten offsetek (head + children).
    // Ezt a segítségeddel már tudod (pl. ClauseTemplate-ből):
    let rows_chip = RuleRowsChip::construct(cfg.rule_rows_cfg.clone());
    let mut all_clause_rows: Vec<Vec<Vec<AssignedCell<Fp, Fp>>>> = Vec::new();

    for (p_i, pred) in self.rules.predicates.iter().enumerate() {
        for (c_i, clause) in pred.clauses.iter().enumerate() {

            let eqs_fp_4d = clause_equalities_4d_tuples_fp(clause);

            let rows_ij = rows_chip.assign_rule_rows_fp_4d(
                layouter.namespace(|| format!("rows pred{}_clause{}", p_i, c_i)),
                &eqs_fp_4d,
                MAX_DOT_DIM,
            )?;
            all_clause_rows.push(rows_ij); 
            // shape: [clause_idx][row_idx][k]
        }
    }
    let compress_chip = RowsCompressChip::construct(cfg.rows_compress_chip.clone());
    let compressed_vec: Vec<AssignedCell<Fp,Fp>> = compress_chip.assign_compressed_active_simple(
        layouter.namespace(|| "compress active clause rows (no r)"),
        &all_clause_rows,  // [clause][row][k]
        &b_flags,          // one-hot flags a SigCheck-ből
    )?;

    // 1) build w from actual input
    let w_fp = build_witness_w_fp(
        &self.unif.goal_name,
        &self.unif.subtree_goals,
    );

    // 2) assign w cells
    let w_cells = assign_w_cells(
        &mut layouter,
        &cfg.rows_compress_chip,
        &w_fp,
    )?;
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

pub fn clause_equalities_4d_tuples_fp(
    clause: &ClauseTemplateFp,
) -> Vec<(Fp,Fp,Fp,Fp,Fp,Fp,Fp,Fp)> {
    clause
        .equalities
        .iter()
        .filter_map(|eq| match (&eq.left, &eq.right) {
            (TermSideFp::Ref(l), TermSideFp::Ref(r)) => Some((
                l.children_node_list, // left node index
                l.predicate,         // left predicate index
                l.arg,               // left argument row index
                l.list_index,        // left list index

                r.children_node_list, // right node index
                r.predicate,         // right predicate index
                r.arg,               // right argument row index
                r.list_index,        // right list index
            )),
            _ => None, // ha bármelyik Value => nincs egyenlet sor
        })
        .collect()
}

pub fn build_witness_w_fp(
    goal_terms: &[TermFp],         // len ≤ MAX_PRED_LIST
    subtree_terms: &[Vec<TermFp>], // len ≤ MAX_CHILDREN, mindegyik len ≤ MAX_PRED_LIST
) -> Vec<Fp> {
    let mut w = vec![Fp::zero(); MAX_NODES * PER_NODE + 1]; // +1 homogén 1-nek a végére

    // node 0: goal
    for p in 0..MAX_PRED_LIST {
        let term_opt = goal_terms.get(p);
        let flat = term_opt.map(flatten_term_args).unwrap_or_else(|| vec![Fp::zero(); PER_TERM]);

        for a in 0..MAX_ARITY {
            for l in 0..MAX_PRED_LIST {
                let val = flat[a * MAX_PRED_LIST + l];
                if let Some(idx) = linear_idx_4d(0, p, a, l) {
                    w[idx] = val;
                }
            }
        }
    }

    // node 1..MAX_CHILDREN: subtree
    for n in 0..MAX_CHILDREN {
        let node_idx = 1 + n;
        let row_terms = subtree_terms.get(n); // Option<&Vec<TermFp>>

        for p in 0..MAX_PRED_LIST {
            let term_opt = row_terms.and_then(|row| row.get(p));
            let flat = term_opt.map(flatten_term_args).unwrap_or_else(|| vec![Fp::zero(); PER_TERM]);

            for a in 0..MAX_ARITY {
                for l in 0..MAX_PRED_LIST {
                    let val = flat[a * MAX_PRED_LIST + l];
                    if let Some(idx) = linear_idx_4d(node_idx, p, a, l) {
                        w[idx] = val;
                    }
                }
            }
        }
    }

    // homogén 1 a legvégén
    w[MAX_NODES * PER_NODE] = Fp::one();

    w
}


fn assign_w_cells(
    layouter: &mut impl Layouter<Fp>,
    cfg: &RowsCompressConfig,
    w: &[Fp],
) -> Result<Vec<AssignedCell<Fp, Fp>>, Error>
{
    layouter.assign_region(
        || "bind w vector",
        |mut region| {
            let mut out = Vec::with_capacity(w.len());
            for (i, val) in w.iter().enumerate() {
                let c = region.assign_advice(
                    || format!("w[{i}]"),
                    cfg.val,
                    i,
                    || Value::known(*val),
                )?;
                out.push(c);
            }
            Ok(out)
        },
    )
}

pub fn flatten_term_args(t: &TermFp) -> Vec<Fp> {
    let mut flat = Vec::with_capacity(MAX_ARITY * MAX_PRED_LIST);

    for arg_i in 0..MAX_ARITY {
        for list_i in 0..MAX_PRED_LIST {
            flat.push(
                t.args
                    .get(arg_i)
                    .and_then(|row| row.get(list_i))
                    .copied()
                    .unwrap_or(Fp::zero())  // padding safety
            );
        }
    }

    flat
}

#[inline]
fn linear_idx_4d(
    node_idx: usize,
    pred_idx: usize,
    arg_idx: usize,
    list_idx: usize,
) -> Option<usize> {
    if node_idx >= MAX_NODES { return None; }
    if pred_idx >= MAX_PRED_LIST { return None; }
    if arg_idx >= MAX_ARITY { return None; }
    if list_idx >= MAX_PRED_LIST { return None; }

    Some(
        node_idx * PER_NODE
        + pred_idx * PER_TERM
        + arg_idx * MAX_PRED_LIST
        + list_idx
    )
}

