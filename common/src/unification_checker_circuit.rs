use std::sync::Mutex;
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, SimpleFloorPlanner},
    pasta::Fp,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error},
};
use crate::{
    chips::{
         body_subtree_chip::{BodySubtreeChip, UnifCompareConfig}, poseidon_hash::{HashEqChip, HashEqConfig}, rlc_chip::RlcFixedChip, rlc_goal_check_chip::{RlcGoalCheckChip, RlcGoalCheckConfig}, rolc_compare_chip::RlcCompareChip, sig_check_chip::{SigCheckChip, SigCheckConfig}, ConsistencyChip, DotChip
    },
    data::{ClauseTemplateFp, FactTemplateFp, PredicateTemplateFp, RuleTemplateFileFp, TermFp, UnificationInputFp},
    utils_2::{common_helpers::{str_to_fp, to_fp_value, MAX_ARITY, MAX_CANDIDATES, MAX_CHILDREN, MAX_CLAUSES, MAX_PAIRS, MAX_PREDICATES, MAX_SIGS}, consistency_helpers::bind_goal_name_args_inputs},
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


/*let (body_pairs, subtree_pairs) = bind_body_and_subtree_as_cells_padded(
        "Bind body & subtree (padded)",
        &mut layouter,
        &cfg.unif_cmp_cfg,                 // term_name + term_args oszlopok
        &self.unif.unif_body,              // Vec<TermFp>
        &self.unif.subtree_goals,          // Vec<TermFp>
    )?;  // -> Vec<(name_cell, Vec<arg_cells>)> mindkét oldalra, hossza MAX_PAIRS

    // === 3) RLC-vel páronkénti equality (teljes huzalozás!) ===
    let rlc_chip = RlcFixedChip::construct(cfg.rlc_cfg.clone());

    rlc_chip.cmp_term_lists_pairwise_with_rlc_cells(
        layouter.namespace(|| "Compare body vs subtree (wired RLC)"),
        &body_pairs,
        &subtree_pairs,
    )?;
let (proof_pairs, candidate_pairs_all) = bind_proof_and_candidates_sig_pairs(
    "Bind proof + candidates (name,arity)",
    &mut layouter,
    &cfg.unif_cmp_cfg,
    &self.unif.goal_name,   // goal TermFp
    &self.unif.unif_body,   // body TermFp-k
    &self.rules.predicates, // predikátumok
)?;*/

// is_fact = (body üres?) — itt egyszerűen tanúként bevisszük:

// Rules-only check (fact ág kommentelve)
/*let sig_chip = SigCheckChip::construct(cfg.sig_check_cfg.clone());
let is_fact_cell = layouter.assign_region(
    || "is_fact",
    |mut region| {
        // a 3. proof-pár második komponense (arity)
        let fact_val = proof_pairs.get(2)
            .map(|(_name, arity)| {
                arity.value().map(|v| if *v == Fp::zero() { Fp::one() } else { Fp::zero() })
            })
            .unwrap_or(Value::known(Fp::zero())); // ha nincs 3. elem, akkor false (0)

        region.assign_advice(
            || "is_fact",
            cfg.sig_check_cfg.sig_arity,
            0,
            || fact_val,
        )
    },
)?;

// candidate_pairs_all: rules flattened to (name, arity) SIG sequences.
// If there are no suitable candidates for facts, just pass an empty vec: &[].
println!("fact: {:?}", is_fact_cell.value());
sig_chip.check_membership_rules_or_fact_placeholder(
    layouter.namespace(|| "Sig membership (rules or fact placeholder)"),
    &proof_pairs,
    &candidate_pairs_all, // this can be &[] for fact
    &is_fact_cell,
)?;


// 4) OR tagság-ellenőrzés: proof_sigs ∈ {candidate_sigs}




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

        println!("Total constraints so far: {} (predicate)", get_constraints());*/ */
        Ok(())
    }
}




/*fn bind_proof_and_candidates_sig_pairs(
    region_name: &str,
    layouter: &mut impl Layouter<Fp>,
    cfg: &UnifCompareConfig,
    goal_term: &TermFp,                    // ⬅️ a fő célterm pl. ancestor(alice,john)
    proof_terms: &[TermFp],                // ⬅️ unification body termek (pl. parent, ancestor)
    rules: &[PredicateTemplateFp],         // ⬅️ az összes rules predikátum
) -> Result<
    (
        Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>,              // proof_pairs: (name, arity)
        Vec<Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>>,         // candidate_pairs_all: vec![(name, arity)]
    ),
    Error,
> {
    use halo2_proofs::circuit::Value;

    // kis helper: aritás számolása padelt args-ból (első 0-ig)
    let measure_arity = |args: &Vec<Fp>| -> u64 {
        args.iter().take_while(|&&a| a != Fp::zero()).count() as u64
    };

    layouter.assign_region(
        || region_name,
        |mut region| {
            let mut proof_pairs: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();
            let mut row_offset = 0usize;

            // =========================
            // 1️⃣ GOAL TERM (head)
            // =========================
            let goal_name_cell = region.assign_advice(
                || "proof.goal.name",
                cfg.term_name,
                row_offset,
                || Value::known(goal_term.name),
            )?;
            let goal_arity = Fp::from(measure_arity(&goal_term.args));
            let goal_arity_cell = region.assign_advice(
                || "proof.goal.arity",
                cfg.term_args[0],
                row_offset + 1,
                || Value::known(goal_arity),
            )?;
            proof_pairs.push((goal_name_cell, goal_arity_cell));
            row_offset += 2;

            // =========================
            // 2️⃣ BODY TERMS (unif_body)
            // =========================
            for (i, term) in proof_terms.iter().enumerate() {
                let name_cell = region.assign_advice(
                    || format!("proof.body[{i}].name"),
                    cfg.term_name,
                    row_offset,
                    || Value::known(term.name),
                )?;
                let arity_fp = Fp::from(measure_arity(&term.args));
                let arity_cell = region.assign_advice(
                    || format!("proof.body[{i}].arity"),
                    cfg.term_args[0],
                    row_offset + 1,
                    || Value::known(arity_fp),
                )?;
                proof_pairs.push((name_cell, arity_cell));
                row_offset += 2;
            }

            // padding proof oldal MAX_SIGS-ig
            while proof_pairs.len() < MAX_SIGS {
                let n = region.assign_advice(
                    || format!("proof.pad.name{}", proof_pairs.len()),
                    cfg.term_name,
                    row_offset,
                    || Value::known(Fp::zero()),
                )?;
                let a = region.assign_advice(
                    || format!("proof.pad.arity{}", proof_pairs.len()),
                    cfg.term_args[0],
                    row_offset + 1,
                    || Value::known(Fp::zero()),
                )?;
                proof_pairs.push((n, a));
                row_offset += 2;
            }

            // =========================
            // 3️⃣ RULE CANDIDATES (összes predikátum a rules-ból)
            // =========================
            let mut candidate_pairs_all: Vec<Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>> = Vec::new();
            let mut base_row = row_offset + 8;
            let mut added = 0usize;

            'outer: for (p_i, pred) in rules.iter().enumerate() {
                for (c_i, cl) in pred.clauses.iter().enumerate() {
                    if added == MAX_CANDIDATES { break 'outer; }

                    let mut v: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();

                    // HEAD (pred.name, pred.arity)
                    let head_name = region.assign_advice(
                        || format!("cand[{added}].head.name (p{p_i} c{c_i})"),
                        cfg.term_name,
                        base_row,
                        || Value::known(pred.name),
                    )?;
                    let head_arity = region.assign_advice(
                        || format!("cand[{added}].head.arity (p{p_i} c{c_i})"),
                        cfg.term_args[0],
                        base_row + 1,
                        || Value::known(pred.arity),
                    )?;
                    v.push((head_name, head_arity));
                    base_row += 2;

                    // CHILDREN (child.name, child.arity)
                    for (j, ch) in cl.children.iter().enumerate().take(MAX_CHILDREN) {
                        let n = region.assign_advice(
                            || format!("cand[{added}].child[{j}].name"),
                            cfg.term_name,
                            base_row,
                            || Value::known(ch.name),
                        )?;
                        let a = region.assign_advice(
                            || format!("cand[{added}].child[{j}].arity"),
                            cfg.term_args[0],
                            base_row + 1,
                            || Value::known(ch.arity),
                        )?;
                        v.push((n, a));
                        base_row += 2;
                    }

                    // padding MAX_SIGS-ig
                    while v.len() < MAX_SIGS {
                        let n = region.assign_advice(
                            || format!("cand[{added}].pad.name"),
                            cfg.term_name,
                            base_row,
                            || Value::known(Fp::zero()),
                        )?;
                        let a = region.assign_advice(
                            || format!("cand[{added}].pad.arity"),
                            cfg.term_args[0],
                            base_row + 1,
                            || Value::known(Fp::zero()),
                        )?;
                        v.push((n, a));
                        base_row += 2;
                    }

                    candidate_pairs_all.push(v);
                    added += 1;
                }
            }

            // padding MAX_CANDIDATES-ig
            while candidate_pairs_all.len() < MAX_CANDIDATES {
                let mut v: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();
                for _ in 0..MAX_SIGS {
                    let n = region.assign_advice(
                        || "cand.pad.name",
                        cfg.term_name,
                        base_row,
                        || Value::known(Fp::zero()),
                    )?;
                    let a = region.assign_advice(
                        || "cand.pad.arity",
                        cfg.term_args[0],
                        base_row + 1,
                        || Value::known(Fp::zero()),
                    )?;
                    v.push((n, a));
                    base_row += 2;
                }
                candidate_pairs_all.push(v);
            }

            Ok((proof_pairs, candidate_pairs_all))
        },
    )
}*/
