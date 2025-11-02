use halo2_proofs::{
    circuit::{Layouter, AssignedCell, Value},
    plonk::{Column, Advice, ConstraintSystem, Error},
    pasta::Fp,
};

use crate::utils_2::common_helpers::MAX_CANDIDATES;

#[derive(Clone, Debug)]
pub struct RowsCompressConfig {
    pub val: Column<Advice>,     // input row elem (row_{i,j}[k])
    pub pow: Column<Advice>,     // r^j sorozat
    pub acc: Column<Advice>,     // compressed accumulator
    pub flag: Column<Advice>,    // b_i (one-hot)
}

#[derive(Clone, Debug)]
pub struct RowsCompressChip {
    pub cfg: RowsCompressConfig,
}

impl RowsCompressChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> RowsCompressConfig {
        let val = meta.advice_column();
        let pow = meta.advice_column();
        let acc = meta.advice_column();
        let flag = meta.advice_column();

        meta.enable_equality(val);
        meta.enable_equality(pow);
        meta.enable_equality(acc);
        meta.enable_equality(flag);

        RowsCompressConfig { val, pow, acc, flag }
    }

    pub fn construct(cfg: RowsCompressConfig) -> Self {
        Self { cfg }
    }

    /// compressed_active[k] = sum_i b_i * (sum_j r^j * row_{i,j}[k])
    pub fn assign_compressed_active(
        &self,
        mut layouter: impl Layouter<Fp>,
        rows: &Vec<Vec<Vec<AssignedCell<Fp,Fp>>>>, // [actual_clauses][num_rows][dim]
        flags: &Vec<AssignedCell<Fp,Fp>>,          // [MAX_CANDIDATES] one-hot
        r: &AssignedCell<Fp,Fp>,
    ) -> Result<Vec<AssignedCell<Fp,Fp>>, Error> {
        let cfg = self.cfg.clone();

        assert!(!rows.is_empty(), "rows must not be empty");
        let actual_clauses = rows.len();
        let num_rows       = rows[0].len();
        let dim            = rows[0][0].len();

        // opcionális, ha szeretnéd védeni a layoutot:
        assert_eq!(flags.len(), MAX_CANDIDATES, "flags must be MAX_CANDIDATES long");

        // r^j előállítása (j = 0..num_rows-1)
        let powers: Vec<AssignedCell<Fp, Fp>> = layouter.assign_region(
            || "compute powers of r",
            |mut region| {
                let mut acc = region.assign_advice(|| "r^0", cfg.pow, 0, || Value::known(Fp::one()))?;
                let mut powers = vec![acc.clone()];
                for j in 1..num_rows {
                    let next = region.assign_advice(
                        || format!("r^{j}"),
                        cfg.pow,
                        j,
                        || acc.value().zip(r.value()).map(|(a, r)| *a * *r)
                    )?;
                    acc = next.clone();
                    powers.push(next);
                }
                Ok(powers)
            },
        )?;

        // compressed_active kiszámítása úgy, hogy a flags hossza diktál,
        // de a rows hozzájárulás 0, ha i >= actual_clauses.
        let compressed_active: Vec<AssignedCell<Fp, Fp>> =
            layouter.assign_region(|| "compute compressed_active", |mut region| {
                let mut out = Vec::with_capacity(dim);

                for k in 0..dim {
                    let mut acc_val = Value::known(Fp::zero());

                    // i a flags (MAX_CANDIDATES) szerint lépeget
                    for i in 0..flags.len() {
                        // clause_sum = 0, ha i kívül esik a rows-on
                        let mut clause_sum = Value::known(Fp::zero());

                        if i < actual_clauses {
                            for j in 0..num_rows {
                                let v   = rows[i][j][k].value();
                                let p_j = powers[j].value();
                                clause_sum = clause_sum.zip(v).zip(p_j).map(|((s, v), p)| s + *v * *p);
                            }
                        }
                        // acc += b_i * clause_sum
                        acc_val = acc_val
                            .zip(flags[i].value())
                            .zip(clause_sum)
                            .map(|((a, b), c)| a + *b * c);
                    }

                    let cell = region.assign_advice(
                        || format!("compressed_active[{k}]"),
                        cfg.acc,
                        k,
                        || acc_val,
                    )?;
                    out.push(cell);
                }

                Ok(out)
            })?;

        Ok(compressed_active)
    }

    /// compressed[k] = sum_i b_i * (sum_j row_{i,j}[k])  (r nélkül)
   pub fn assign_compressed_active_simple(
    &self,
    mut layouter: impl Layouter<Fp>,
    rows:   &Vec<Vec<Vec<AssignedCell<Fp,Fp>>>>, // [num_clauses'][num_rows][dim]
    flags:  &Vec<AssignedCell<Fp,Fp>>,           // [MAX_CANDIDATES] one-hot
) -> Result<Vec<AssignedCell<Fp,Fp>>, Error> {
    let cfg = self.cfg.clone();

    assert!(!rows.is_empty(), "rows must not be empty");

    let actual_clauses = rows.len();   // valódi klózok száma
    let flag_count     = flags.len();  // MAX_CANDIDATES
    let dim            = rows[0][0].len();

    let compressed: Vec<AssignedCell<Fp,Fp>> =
        layouter.assign_region(
            || "compressed (safe, no r)",
            |mut region| {
                let mut out = Vec::with_capacity(dim);

                for k in 0..dim {
                    let mut acc_val = Value::known(Fp::zero());

                    // i megy a flags szerint (mert ezekből pontosan egy 1)
                    for i in 0..flag_count {
                        let mut clause_sum = Value::known(Fp::zero());

                        // Csak akkor adj hozzá rows-t, ha van ilyen klóz
                        if i < actual_clauses {
                            let rows_i   = &rows[i];
                            let num_rows = rows_i.len();

                            for j in 0..num_rows {
                                // A dim mismatch itt is kezelve (ha rövidebb sor lenne)
                                if k < rows_i[j].len() {
                                    clause_sum = clause_sum
                                        .zip(rows_i[j][k].value())
                                        .map(|(s,v)| s + *v);
                                }
                            }
                        }
                        // Ha nincs rows[i], clause_sum == 0 → nem zavar

                        // b_i * clause_sum
                        acc_val = acc_val
                            .zip(flags[i].value())
                            .zip(clause_sum)
                            .map(|((a,b),c)| a + *b * c);
                    }

                    let cell = region.assign_advice(
                        || format!("compressed[{k}]"),
                        cfg.acc,
                        k,
                        || acc_val,
                    )?;
                    out.push(cell);
                }
                Ok(out)
            }
        )?;

    Ok(compressed)
}

}
