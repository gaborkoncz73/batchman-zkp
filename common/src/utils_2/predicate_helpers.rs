use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, Error},
};

use crate::{
    data::TermFp, unification_checker_circuit::MAX_NODES, utils_2::common_helpers::MAX_PRED_LIST,
};

/// Assign only the legacy proof signature projection still consumed by the
/// builtin and canonical leaf compatibility checks.
///
/// Policy candidate signatures are deliberately absent here: selected-clause
/// authorization is performed by `TypedRuleChip` over the canonical typed
/// policy plan, so assigning the legacy candidates would add unused rows and
/// an unused equality-enabled advice column to the active circuit.
pub fn bind_proof_sig_pairs(
    region_name: &str,
    layouter: &mut impl Layouter<Fp>,
    proof_pairs: Column<Advice>,
    goal_terms: &[TermFp],         // ✅ Vec<TermFp>
    subtree_terms: &[Vec<TermFp>], // ✅ Vec<Vec<TermFp>>
) -> Result<Vec<Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>>, Error> {
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
            if goal_terms.len() > MAX_PRED_LIST
                || subtree_terms.len() + 1 > MAX_NODES
                || subtree_terms.iter().any(|row| row.len() > MAX_PRED_LIST)
            {
                return Err(Error::Synthesis);
            }
            let mut final_proof_pairs: Vec<Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>> =
                Vec::new();
            let mut proof_row_offset = 0usize;

            // ── GOAL sor (a goal_terms predikátumlistája EGY sorban) ─────────────────
            {
                let mut row: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();

                for (gi, goal_term) in goal_terms.iter().enumerate() {
                    let n = region.assign_advice(
                        || format!("proof.goal[{gi}].name"),
                        proof_pairs,
                        proof_row_offset,
                        || Value::known(goal_term.name),
                    )?;
                    let a = region.assign_advice(
                        || format!("proof.goal[{gi}].arity"),
                        proof_pairs,
                        proof_row_offset + 1,
                        || Value::known(measure_arity(&goal_term.args)),
                    )?;
                    //println!("N: \n {:?} \nA?:? {:?}", n, a);
                    row.push((n, a));

                    proof_row_offset += 2;
                }

                // padding a sor végén
                while row.len() < MAX_PRED_LIST {
                    let pn = region.assign_advice(
                        || "proof.goal.pad.name",
                        proof_pairs,
                        proof_row_offset,
                        || Value::known(Fp::zero()),
                    )?;
                    let pa = region.assign_advice(
                        || "proof.goal.pad.arity",
                        proof_pairs,
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
                        proof_pairs,
                        proof_row_offset,
                        || Value::known(term.name),
                    )?;
                    let a = region.assign_advice(
                        || format!("proof.sub[{si}][{ti}].arity"),
                        proof_pairs,
                        proof_row_offset + 1,
                        || Value::known(measure_arity(&term.args)),
                    )?;
                    row.push((n, a));
                    proof_row_offset += 2;
                }

                // padding a sor végén
                while row.len() < MAX_PRED_LIST {
                    let pn = region.assign_advice(
                        || "proof.sub.pad.name",
                        proof_pairs,
                        proof_row_offset,
                        || Value::known(Fp::zero()),
                    )?;
                    let pa = region.assign_advice(
                        || "proof.sub.pad.arity",
                        proof_pairs,
                        proof_row_offset + 1,
                        || Value::known(Fp::zero()),
                    )?;
                    row.push((pn, pa));
                    proof_row_offset += 2;
                }

                final_proof_pairs.push(row);
            }

            // Parent plus MAX_CHILDREN rows have one fixed circuit shape.
            while final_proof_pairs.len() < MAX_NODES {
                let mut row: Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)> = Vec::new();
                for _ in 0..MAX_PRED_LIST {
                    let pn = region.assign_advice(
                        || "proof.total.pad.name",
                        proof_pairs,
                        proof_row_offset,
                        || Value::known(Fp::zero()),
                    )?;
                    let pa = region.assign_advice(
                        || "proof.total.pad.arity",
                        proof_pairs,
                        proof_row_offset + 1,
                        || Value::known(Fp::zero()),
                    )?;
                    row.push((pn, pa));
                    proof_row_offset += 2;
                }
                final_proof_pairs.push(row);
            }

            Ok(final_proof_pairs)
        },
    )
}
