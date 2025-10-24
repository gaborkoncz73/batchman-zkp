use halo2_proofs::{
    circuit::{Layouter, AssignedCell, Value},
    plonk::{Advice, Column, ConstraintSystem, Error},
    pasta::Fp,
};
use halo2curves::ff::PrimeField;

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

    pub fn assign_rule_rows_fp(
        &self,
        mut layouter: impl Layouter<Fp>,
        clause_eqs_fp: &[(Fp, Fp, Fp, Fp)], // (ln, la, rn, ra)
        offsets_fp: &[Fp],                   // node -> kezdőindex
        _head_arity_fp: Fp,                  // (nem kell külön)
        max_dim: usize,
    ) -> Result<Vec<Vec<AssignedCell<Fp, Fp>>>, Error> {
        let cfg = self.cfg.clone();
        let offsets: Vec<usize> = offsets_fp.iter().map(fp_to_usize).collect();

        // Fordítsuk globális indexpárokra. None => nullsor.
        let clause_eqs_global: Vec<Option<(usize,usize)>> = clause_eqs_fp.iter().map(|(ln_fp, la_fp, rn_fp, ra_fp)| {
            let ln = fp_to_usize(ln_fp);
            let la = fp_to_usize(la_fp);
            let rn = fp_to_usize(rn_fp);
            let ra = fp_to_usize(ra_fp);

            // node out of range → nullsor
            if ln >= offsets.len() || rn >= offsets.len() {
                return None;
            }
            let l_idx = offsets[ln].saturating_add(la);
            let r_idx = offsets[rn].saturating_add(ra);

            // out of range → nullsor
            if l_idx >= max_dim || r_idx >= max_dim {
                return None;
            }
            // azonos pozíció (paddelt (0,0)==(0,0) is ide esik) → nullsor
            if l_idx == r_idx {
                return None;
            }
            Some((l_idx, r_idx))
        }).collect();

        let rows = layouter.assign_region(
            || "assign clause equality rows (safe)",
            |mut region| {
                let mut all_rows = Vec::with_capacity(clause_eqs_global.len());
                let mut row_offset = 0;

                for (row_idx, maybe_pair) in clause_eqs_global.iter().enumerate() {
                    let mut assigned_row = Vec::with_capacity(max_dim);

                    for k in 0..max_dim {
                        let val = match maybe_pair {
                            Some((l_idx, r_idx)) if k == *l_idx => Value::known(Fp::one()),
                            Some((l_idx, r_idx)) if k == *r_idx => Value::known(-Fp::one()),
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

                    // Szép debug kiírás
                    match maybe_pair {
                        Some((l, r)) => {
                            let dbg: Vec<i8> = (0..max_dim).map(|k|
                                if k == *l { 1 } else if k == *r { -1 } else { 0 }
                            ).collect();
                            //println!("row[{row_idx}] (+1@{l}, -1@{r}): {:?}", dbg);
                        }
                        None => {
                            //println!("row[{row_idx}] (null-row)");
                        }
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
