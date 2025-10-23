
use halo2_proofs::{
    circuit::{AssignedCell, Layouter},
    pasta::Fp,
    plonk::Error,
};

use crate::{chips::body_subtree_chip::UnifCompareConfig, data::{PredicateTemplateFp, TermFp}, utils_2::common_helpers::{MAX_CANDIDATES, MAX_CHILDREN, MAX_SIGS}};



pub fn bind_proof_and_candidates_sig_pairs(
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
                cfg.body_name,
                row_offset,
                || Value::known(goal_term.name),
            )?;
            let goal_arity = Fp::from(measure_arity(&goal_term.args));
            let goal_arity_cell = region.assign_advice(
                || "proof.goal.arity",
                cfg.body_args[0],
                row_offset,
                || Value::known(goal_arity),
            )?;
            proof_pairs.push((goal_name_cell, goal_arity_cell));
            row_offset += 1;

            // =========================
            // 2️⃣ BODY TERMS (unif_body)
            // =========================
            for (i, term) in proof_terms.iter().enumerate() {
                let name_cell = region.assign_advice(
                    || format!("proof.body[{i}].name"),
                    cfg.body_name,
                    row_offset,
                    || Value::known(term.name),
                )?;
                let arity_fp = Fp::from(measure_arity(&term.args));
                let arity_cell = region.assign_advice(
                    || format!("proof.body[{i}].arity"),
                    cfg.body_args[0],
                    row_offset,
                    || Value::known(arity_fp),
                )?;
                proof_pairs.push((name_cell, arity_cell));
                row_offset += 1;
            }

            // padding proof oldal MAX_SIGS-ig
            while proof_pairs.len() < MAX_SIGS {
                let n = region.assign_advice(
                    || format!("proof.pad.name{}", proof_pairs.len()),
                    cfg.body_name,
                    row_offset,
                    || Value::known(Fp::zero()),
                )?;
                let a = region.assign_advice(
                    || format!("proof.pad.arity{}", proof_pairs.len()),
                    cfg.body_args[0],
                    row_offset,
                    || Value::known(Fp::zero()),
                )?;
                proof_pairs.push((n, a));
                row_offset += 1;
            }

            // =========================
            // 3️⃣ RULE CANDIDATES (összes predikátum a rules-ból)
            // =========================
            let mut candidate_pairs_all: Vec<Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>> = Vec::new();
            let mut base_row = 0usize;
            let mut added = 0usize;

            'outer: for (p_i, pred) in rules.iter().enumerate() {
                for (c_i, cl) in pred.clauses.iter().enumerate() {
                    if added == MAX_CANDIDATES { break 'outer; }

                    let mut v: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();

                    // HEAD (pred.name, pred.arity)
                    let head_name = region.assign_advice(
                        || format!("cand[{added}].head.name (p{p_i} c{c_i})"),
                        cfg.subtree_name,
                        base_row,
                        || Value::known(pred.name),
                    )?;
                    let head_arity = region.assign_advice(
                        || format!("cand[{added}].head.arity (p{p_i} c{c_i})"),
                        cfg.subtree_args[0],
                        base_row,
                        || Value::known(pred.arity),
                    )?;
                    v.push((head_name, head_arity));
                    base_row += 1;

                    // CHILDREN (child.name, child.arity)
                    for (j, ch) in cl.children.iter().enumerate().take(MAX_CHILDREN) {
                        let n = region.assign_advice(
                            || format!("cand[{added}].child[{j}].name"),
                            cfg.subtree_name,
                            base_row,
                            || Value::known(ch.name),
                        )?;
                        let a = region.assign_advice(
                            || format!("cand[{added}].child[{j}].arity"),
                            cfg.subtree_args[0],
                            base_row,
                            || Value::known(ch.arity),
                        )?;
                        v.push((n, a));
                        base_row += 1;
                    }

                    // padding MAX_SIGS-ig
                    while v.len() < MAX_SIGS {
                        let n = region.assign_advice(
                            || format!("cand[{added}].pad.name"),
                            cfg.subtree_name,
                            base_row,
                            || Value::known(Fp::zero()),
                        )?;
                        let a = region.assign_advice(
                            || format!("cand[{added}].pad.arity"),
                            cfg.subtree_args[0],
                            base_row,
                            || Value::known(Fp::zero()),
                        )?;
                        v.push((n, a));
                        base_row += 1;
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
                        cfg.subtree_name,
                        base_row,
                        || Value::known(Fp::zero()),
                    )?;
                    let a = region.assign_advice(
                        || "cand.pad.arity",
                        cfg.subtree_args[0],
                        base_row,
                        || Value::known(Fp::zero()),
                    )?;
                    v.push((n, a));
                    base_row += 1;
                }
                candidate_pairs_all.push(v);
            }

            Ok((proof_pairs, candidate_pairs_all))
        },
    )
}