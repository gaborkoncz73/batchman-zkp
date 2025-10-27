
use halo2_proofs::{
    circuit::{AssignedCell, Layouter},
    pasta::Fp,
    plonk::Error,
};

use crate::{chips::finding_rule::body_subtree_chip::UnifCompareConfig, data::{PredicateTemplateFp, TermFp}, utils_2::common_helpers::{MAX_CANDIDATES, MAX_CHILDREN}};

pub fn bind_proof_and_candidates_sig_pairs(
    region_name: &str,
    layouter: &mut impl Layouter<Fp>,
    cfg: &UnifCompareConfig,
    goal_term: &TermFp,                    // Main term e.g. ancestor(alice,john)
    subtree_terms: &[TermFp],                // Subtree terms (e. g. parent, ancestor)
    rules: &[PredicateTemplateFp],         // Every rule predicates
) -> Result<
    (
        Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>,              // proof_pairs: (name, arity)
        Vec<Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>>,         // candidate_pairs_all: vec![(name, arity)]
    ),
    Error,
> {
    use halo2_proofs::circuit::Value;

    // arity counter
    let measure_arity = |args: &Vec<Fp>| -> u64 {
        args.iter().take_while(|&&a| a != Fp::zero()).count() as u64
    };

    layouter.assign_region(
        || region_name,
        |mut region| {
            let mut proof_pairs: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();
            let mut proof_row_offset = 0usize;

            // Goal name
            let goal_name_cell = region.assign_advice(
                || "proof.goal.name",
                cfg.proof_pairs,
                proof_row_offset,
                || Value::known(goal_term.name),
            )?;
            let goal_arity = Fp::from(measure_arity(&goal_term.args));
            let goal_arity_cell = region.assign_advice(
                || "proof.goal.arity",
                cfg.proof_pairs,
                proof_row_offset + 1,
                || Value::known(goal_arity),
            )?;
            proof_pairs.push((goal_name_cell, goal_arity_cell));
            proof_row_offset += 3;

            // Subtree terms
            for (i, term) in subtree_terms.iter().enumerate() {
                let name_cell = region.assign_advice(
                    || format!("proof.body[{i}].name"),
                    cfg.proof_pairs,
                    proof_row_offset ,
                    || Value::known(term.name),
                )?;
                let arity_fp = Fp::from(measure_arity(&term.args));
                let arity_cell = region.assign_advice(
                    || format!("proof.body[{i}].arity"),
                    cfg.proof_pairs,
                    proof_row_offset + 1,
                    || Value::known(arity_fp),
                )?;
                proof_pairs.push((name_cell, arity_cell));
                proof_row_offset += 2;
            }

            // padding proof until MAX_SIGS
            while proof_pairs.len() < MAX_CHILDREN {
                let n = region.assign_advice(
                    || format!("proof.pad.name{}", proof_pairs.len()),
                    cfg.proof_pairs,
                    proof_row_offset,
                    || Value::known(Fp::zero()),
                )?;
                let a = region.assign_advice(
                    || format!("proof.pad.arity{}", proof_pairs.len()),
                    cfg.proof_pairs,
                    proof_row_offset + 1,
                    || Value::known(Fp::zero()),
                )?;
                proof_pairs.push((n, a));
                proof_row_offset += 2;
            }

            // Rule candidates
            let mut candidate_pairs_all: Vec<Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>> = Vec::new();
            let mut candidate_row_offset = 0usize;

            for (p_i, pred) in rules.iter().enumerate() {
                for (c_i, cl) in pred.clauses.iter().enumerate() {

                    let mut v: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();

                    // pred name
                    let head_name = region.assign_advice(
                        || format!("head.name (p{p_i} c{c_i})"),
                        cfg.candidate_pairs,
                        candidate_row_offset,
                        || Value::known(pred.name),
                    )?;
                    let head_arity = region.assign_advice(
                        || format!("head.arity (p{p_i} c{c_i})"),
                        cfg.candidate_pairs,
                        candidate_row_offset + 1,
                        || Value::known(pred.arity),
                    )?;
                    v.push((head_name, head_arity));
                    candidate_row_offset += 2;

                    // CHILDREN (child.name, child.arity)
                    for (j, ch) in cl.children.iter().enumerate().take(MAX_CHILDREN) {
                        let child_name = region.assign_advice(
                            || format!("child[{j}].name"),
                            cfg.candidate_pairs,
                            candidate_row_offset,
                            || Value::known(ch.name),
                        )?;
                        let child_arity = region.assign_advice(
                            || format!("child[{j}].arity"),
                            cfg.candidate_pairs,
                            candidate_row_offset + 1,
                            || Value::known(ch.arity),
                        )?;
                        v.push((child_name, child_arity));
                        candidate_row_offset += 2;
                    }

                    // padding until MAX_SIGS
                    while v.len() < MAX_CHILDREN {
                        let pad_name = region.assign_advice(
                            || format!("pad.name"),
                            cfg.candidate_pairs,
                            candidate_row_offset,
                            || Value::known(Fp::zero()),
                        )?;
                        let pad_arity = region.assign_advice(
                            || format!("pad.arity"),
                            cfg.candidate_pairs,
                            candidate_row_offset + 1,
                            || Value::known(Fp::zero()),
                        )?;
                        v.push((pad_name, pad_arity));
                        candidate_row_offset += 2;
                    }
                    candidate_pairs_all.push(v);
                }
            }

            // padding until MAX_CANDIDATES
            while candidate_pairs_all.len() < MAX_CANDIDATES {
                let mut v: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();
                for _ in 0..MAX_CHILDREN + 1 {
                    let n = region.assign_advice(
                        || "cand.pad.name",
                        cfg.candidate_pairs,
                        candidate_row_offset,
                        || Value::known(Fp::zero()),
                    )?;
                    let a = region.assign_advice(
                        || "cand.pad.arity",
                        cfg.candidate_pairs,
                        candidate_row_offset + 1,
                        || Value::known(Fp::zero()),
                    )?;
                    v.push((n, a));
                    candidate_row_offset += 2;
                }
                candidate_pairs_all.push(v);
            }
            Ok((proof_pairs, candidate_pairs_all))
        },
    )
}