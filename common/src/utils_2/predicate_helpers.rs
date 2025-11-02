
use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    pasta::Fp,
    plonk::Error,
};

use crate::{
    chips::finding_rule::body_subtree_chip::UnifCompareConfig,
    data::{PredicateTemplateFp, TermFp},
    utils_2::common_helpers::{MAX_CANDIDATES, MAX_CHILDREN},
};


pub fn bind_proof_and_candidates_sig_pairs(
    region_name: &str,
    layouter: &mut impl Layouter<Fp>,
    cfg: &UnifCompareConfig,
    goal_terms: &[TermFp],         // ✅ Vec<TermFp>
    subtree_terms: &[Vec<TermFp>], // ✅ Vec<Vec<TermFp>>
    rules: &[PredicateTemplateFp],
) -> Result<
    (
        Vec<Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>>,               // final_proof_pairs [rows][(name,arity)]
        Vec<Vec<Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>>>,          // candidate_pairs_all [candidate][rows][(name,arity)]
    ),
    Error,
> {
    //println!("gaty: {:?}", goal_terms);
    // aritás mérése: a 2D mátrixot sorfolytonosan bejárjuk és az első zéróig számolunk
    let measure_arity = |matrix: &Vec<Vec<Fp>>| -> Fp {
        let count = matrix
            .iter()
            .take_while(|row| row.get(0).map(|v| *v != Fp::one().neg()).unwrap_or(false))
            .count() as u64;
        Fp::from(count)
    };

    layouter.assign_region(
        || region_name,
        |mut region| {
            let mut final_proof_pairs: Vec<Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>> = Vec::new();
            let mut proof_row_offset = 0usize;

            // ── GOAL sor (a goal_terms predikátumlistája EGY sorban) ─────────────────
            {
                let mut row: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();

                for (gi, goal_term) in goal_terms.iter().enumerate() {
                    let n = region.assign_advice(
                        || format!("proof.goal[{gi}].name"),
                        cfg.proof_pairs,
                        proof_row_offset,
                        || Value::known(goal_term.name),
                    )?;
                    let a = region.assign_advice(
                        || format!("proof.goal[{gi}].arity"),
                        cfg.proof_pairs,
                        proof_row_offset + 1,
                        || Value::known(measure_arity(&goal_term.args)),
                    )?;
                    //println!("N: \n {:?} \nA?:? {:?}", n, a);
                    row.push((n, a));
                    
                    proof_row_offset += 2;
                }

                // padding a sor végén
                while row.len() < MAX_CHILDREN {
                    let pn = region.assign_advice(
                        || "proof.goal.pad.name",
                        cfg.proof_pairs,
                        proof_row_offset,
                        || Value::known(Fp::zero()),
                    )?;
                    let pa = region.assign_advice(
                        || "proof.goal.pad.arity",
                        cfg.proof_pairs,
                        proof_row_offset + 1,
                        || Value::known(Fp::zero()),
                    )?;
                    row.push((pn, pa));
                    proof_row_offset += 2;
                }

                final_proof_pairs.push(row);
            }

            // ── SUBTREE sorok (minden subtree külön SOR) ────────────────────────────
            for (si, subtree) in subtree_terms.iter().enumerate() {
                let mut row: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();

                for (ti, term) in subtree.iter().enumerate() {
                    let n = region.assign_advice(
                        || format!("proof.sub[{si}][{ti}].name"),
                        cfg.proof_pairs,
                        proof_row_offset,
                        || Value::known(term.name),
                    )?;
                    let a = region.assign_advice(
                        || format!("proof.sub[{si}][{ti}].arity"),
                        cfg.proof_pairs,
                        proof_row_offset + 1,
                        || Value::known(measure_arity(&term.args)),
                    )?;
                    row.push((n, a));
                    proof_row_offset += 2;
                }

                // padding a sor végén
                while row.len() < MAX_CHILDREN {
                    let pn = region.assign_advice(
                        || "proof.sub.pad.name",
                        cfg.proof_pairs,
                        proof_row_offset,
                        || Value::known(Fp::zero()),
                    )?;
                    let pa = region.assign_advice(
                        || "proof.sub.pad.arity",
                        cfg.proof_pairs,
                        proof_row_offset + 1,
                        || Value::known(Fp::zero()),
                    )?;
                    row.push((pn, pa));
                    proof_row_offset += 2;
                }

                final_proof_pairs.push(row);
            }

            // ── (opcionális) teljes proof rows padding MAX_CHILDREN-ig ──────────────
            while final_proof_pairs.len() < MAX_CHILDREN {
                let mut row: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();
                for _ in 0..MAX_CHILDREN {
                    let pn = region.assign_advice(
                        || "proof.total.pad.name",
                        cfg.proof_pairs,
                        proof_row_offset,
                        || Value::known(Fp::zero()),
                    )?;
                    let pa = region.assign_advice(
                        || "proof.total.pad.arity",
                        cfg.proof_pairs,
                        proof_row_offset + 1,
                        || Value::known(Fp::zero()),
                    )?;
                    row.push((pn, pa));
                    proof_row_offset += 2;
                }
                final_proof_pairs.push(row);
                
            }

            // ── CANDIDATES: 3D szerkezet [candidate][row][(name,arity)] ────────────
            let mut candidate_pairs_all: Vec<Vec<Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>>> = Vec::new();
            let mut candidate_row_offset = 0usize;

            for pred in rules {
                for cl in &pred.clauses {
                    let mut rows: Vec<Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>> = Vec::new();

                    // HEAD sor
                    {
                        let mut head_row: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();

                        let hn = region.assign_advice(
                            || "cand.head.name",
                            cfg.candidate_pairs,
                            candidate_row_offset,
                            || Value::known(pred.name),
                        )?;
                        let ha = region.assign_advice(
                            || "cand.head.arity",
                            cfg.candidate_pairs,
                            candidate_row_offset + 1,
                            || Value::known(pred.arity),
                        )?;
                        head_row.push((hn, ha));
                        candidate_row_offset += 2;

                        while head_row.len() < MAX_CHILDREN {
                            let pn = region.assign_advice(
                                || "cand.head.pad.name",
                                cfg.candidate_pairs,
                                candidate_row_offset,
                                || Value::known(Fp::zero()),
                            )?;
                            let pa = region.assign_advice(
                                || "cand.head.pad.arity",
                                cfg.candidate_pairs,
                                candidate_row_offset + 1,
                                || Value::known(Fp::zero()),
                            )?;
                            head_row.push((pn, pa));
                            candidate_row_offset += 2;
                        }

                        rows.push(head_row);
                    }

                    // BODY sorok: a clause.children egy 2D lista → minden belső lista egy SOR
                    // BODY sorok: minden belső lista egy SOR
                    for (row_i, row_children) in cl.children.iter().enumerate() {
                        let mut r: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();

                        // kitöltés
                        for (col_i, ch) in row_children.iter().enumerate() {
                            let n = region.assign_advice(
                                || format!("cand.child[{row_i}][{col_i}].name"),
                                cfg.candidate_pairs,
                                candidate_row_offset,
                                || Value::known(ch.name),
                            )?;
                            let a = region.assign_advice(
                                || format!("cand.child[{row_i}][{col_i}].arity"),
                                cfg.candidate_pairs,
                                candidate_row_offset + 1,
                                || Value::known(ch.arity),
                            )?;
                            r.push((n, a));
                            candidate_row_offset += 2;
                        }

                        // OSZLOPOK (párok) paddingje a SOR VÉGÉN — UGYANÚGY, mint a proof-nál
                        while r.len() < MAX_CHILDREN {
                            let pn = region.assign_advice(
                                || "cand.child.pad.name",
                                cfg.candidate_pairs,
                                candidate_row_offset,
                                || Value::known(Fp::zero()),
                            )?;
                            let pa = region.assign_advice(
                                || "cand.child.pad.arity",
                                cfg.candidate_pairs,
                                candidate_row_offset + 1,
                                || Value::known(Fp::zero()),
                            )?;
                            r.push((pn, pa));
                            candidate_row_offset += 2;
                        }

                        // ✅ most toljuk be a sort
                        rows.push(r);
                    }

                    // ✅ MIUTÁN minden BODY sor bekerült: SOROK SZÁMÁNAK paddingje
                    while rows.len() < MAX_CHILDREN {
                        let mut empty_row: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();
                        for _ in 0..MAX_CHILDREN {
                            let pn = region.assign_advice(
                                || "cand.rows.pad.name",
                                cfg.candidate_pairs,
                                candidate_row_offset,
                                || Value::known(Fp::zero()),
                            )?;
                            let pa = region.assign_advice(
                                || "cand.rows.pad.arity",
                                cfg.candidate_pairs,
                                candidate_row_offset + 1,
                                || Value::known(Fp::zero()),
                            )?;
                            empty_row.push((pn, pa));
                            candidate_row_offset += 2;
                        }
                        rows.push(empty_row);
                    }


                    candidate_pairs_all.push(rows);
                }
            }

            // kandidátusok paddingje MAX_CANDIDATES-ig
            while candidate_pairs_all.len() < MAX_CANDIDATES {
                // üres jelölt: MAX_CHILDREN sor, soronként MAX_CHILDREN pár (mind 0)
                let mut empty_rule: Vec<Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>> = Vec::new();
                for _ in 0..MAX_CHILDREN {
                    let mut row: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();
                    for _ in 0..MAX_CHILDREN {
                        let pn = region.assign_advice(
                            || "cand.full.pad.name",
                            cfg.candidate_pairs,
                            candidate_row_offset,
                            || Value::known(Fp::zero()),
                        )?;
                        let pa = region.assign_advice(
                            || "cand.full.pad.arity",
                            cfg.candidate_pairs,
                            candidate_row_offset + 1,
                            || Value::known(Fp::zero()),
                        )?;
                        row.push((pn, pa));
                        candidate_row_offset += 2;
                    }
                    empty_rule.push(row);
                }
                candidate_pairs_all.push(empty_rule);
            }
            
            Ok((final_proof_pairs, candidate_pairs_all))
        },
    )
}
