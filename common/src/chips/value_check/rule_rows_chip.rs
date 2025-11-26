use halo2_proofs::{
    circuit::{Layouter, AssignedCell, Value},
    plonk::{Advice, Column, ConstraintSystem, Error},
    pasta::Fp,
};
use halo2curves::ff::PrimeField;

use crate::{unification_checker_circuit::{MAX_NODES, PER_NODE, PER_TERM}, utils_2::common_helpers::{MAX_ARITY, MAX_PRED_LIST}};

#[derive(Clone, Debug)]
pub struct RuleRowsConfig {
    pub val: Column<Advice>,
}

#[derive(Clone, Debug)]
pub struct RuleRowsChip {
    pub cfg: RuleRowsConfig,
}

impl RuleRowsChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> RuleRowsConfig {
        let val = meta.advice_column();
        meta.enable_equality(val);
        RuleRowsConfig { val }
    }

    pub fn construct(cfg: RuleRowsConfig) -> Self { Self { cfg } }

     pub fn assign_rule_rows_fp_4d(
        &self,
        mut layouter: impl Layouter<Fp>,
        clause_eqs_4d: &[(Fp,Fp,Fp,Fp,Fp,Fp,Fp,Fp)], // (n,p,a,l, n',p',a',l')
        max_dim: usize, // == MAX_DOT_DIM
    ) -> Result<Vec<Vec<AssignedCell<Fp,Fp>>>, Error> {
        let cfg = self.cfg.clone();

        let clause_eqs_global: Vec<Option<(usize,usize)>> = clause_eqs_4d.iter().map(|t| {
            let (ln, lp, la, ll, rn, rp, ra, rl) = t;

            // Fp -> usize
            let ln = fp_to_usize(ln);
            let lp = fp_to_usize(lp);
            let la = fp_to_usize(la);
            let ll = fp_to_usize(ll);

            let rn = fp_to_usize(rn);
            let rp = fp_to_usize(rp);
            let ra = fp_to_usize(ra);
            let rl = fp_to_usize(rl);

            let l_idx = linear_idx_4d(ln, lp, la, ll)?;
            let r_idx = linear_idx_4d(rn, rp, ra, rl)?;

            if l_idx >= max_dim || r_idx >= max_dim { return None; }
            if l_idx == r_idx { return None; } // (0,0)==(0,0) padding equality → nincs sor

            Some((l_idx, r_idx))
        }).collect();

        let rows = layouter.assign_region(
            || "assign clause equality rows (4D)",
            |mut region| {
                let mut all_rows = Vec::with_capacity(clause_eqs_global.len());
                let mut row_offset = 0;

                for (row_idx, maybe_pair) in clause_eqs_global.iter().enumerate() {
                    let mut assigned_row = Vec::with_capacity(max_dim);

                    for k in 0..max_dim {
                        let val = match maybe_pair {
                            Some((l_idx, _)) if k == *l_idx => Value::known(Fp::one()),
                            Some((_, r_idx)) if k == *r_idx => Value::known(-Fp::one()),
                            _ => Value::known(Fp::zero()),
                        };
                        let cell = region.assign_advice(
                            || format!("row[{row_idx}][{k}]"),
                            cfg.val,
                            row_offset,
                            || val,
                        )?;
                        assigned_row.push(cell);
                        row_offset += 1;
                    }

                    all_rows.push(assigned_row);
                }
                
                Ok(all_rows)
            },
        )?;
        Ok(rows)
    }
}

// Fp -> usize (alsó 8 byte little-endian)
fn fp_to_usize(x: &Fp) -> usize {
    let bytes = x.to_repr();
    let mut lo = [0u8; 8];
    lo.copy_from_slice(&bytes[0..8]);
    u64::from_le_bytes(lo) as usize
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