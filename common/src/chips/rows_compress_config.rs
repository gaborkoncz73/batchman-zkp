use halo2_proofs::{
    circuit::{Layouter, AssignedCell, Value},
    plonk::{Column, Advice, ConstraintSystem, Error},
    pasta::Fp,
};

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

    pub fn assign_compressed_active(
        &self,
        mut layouter: impl Layouter<Fp>,
        rows: &Vec<Vec<Vec<AssignedCell<Fp,Fp>>>>, // [num_clauses][num_rows][dim]
        flags: &Vec<AssignedCell<Fp,Fp>>,          // [num_clauses]
        r: &AssignedCell<Fp,Fp>,
    ) -> Result<Vec<AssignedCell<Fp,Fp>>, Error> {

        let cfg = self.cfg.clone();
        let num_clauses = rows.len();
        let num_rows = rows[0].len();
        let dim = rows[0][0].len();

        // először előállítjuk a r^j hatványokat on-circuit
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

        // most kiszámoljuk a compressed_active[k]
        let compressed_active: Vec<AssignedCell<Fp, Fp>> =
            layouter.assign_region(|| "compute compressed_active", |mut region| {

                let mut out = Vec::new();

                for k in 0..dim {
                    // összegzés: sum_i b_i * (sum_j r^j * row_{i,j}[k])
                    let mut acc_val = Value::known(Fp::zero());

                    for i in 0..num_clauses {
                        let mut clause_val = Value::known(Fp::zero());

                        for j in 0..num_rows {
                            let val = &rows[i][j][k];
                            let pow = &powers[j];
                            clause_val = clause_val + val.value().zip(pow.value())
                                .map(|(v, p)| *v * *p);
                        }

                        let flag = &flags[i];
                        acc_val = acc_val + flag.value().zip(clause_val)
                            .map(|(f, c)| *f * c);
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
     pub fn assign_compressed_active_simple(
        &self,
        mut layouter: impl Layouter<Fp>,
        rows:   &Vec<Vec<Vec<AssignedCell<Fp,Fp>>>>, // [num_clauses][num_rows][dim]
        flags:  &Vec<AssignedCell<Fp,Fp>>,           // [num_clauses] one-hot
    ) -> Result<Vec<AssignedCell<Fp,Fp>>, Error> {
        let cfg = self.cfg.clone();

        assert!(!rows.is_empty());
        let num_clauses = rows.len();
        let num_rows    = rows[0].len();
        let dim         = rows[0][0].len();
        assert_eq!(flags.len(), num_clauses);

        // compressed[k] = Σ_i b_i * (Σ_j row_{i,j}[k])
        let compressed: Vec<AssignedCell<Fp,Fp>> = layouter.assign_region(
            || "compressed (no r)",
            |mut region| {
                let mut out = Vec::with_capacity(dim);

                for k in 0..dim {
                    let mut acc_val = Value::known(Fp::zero());

                    for i in 0..num_clauses {
                        let mut clause_sum = Value::known(Fp::zero());
                        for j in 0..num_rows {
                            clause_sum = clause_sum.zip(rows[i][j][k].value())
                                      .map(|(s, v)| s + *v);
                        }
                        acc_val = acc_val
                            .zip(flags[i].value())
                            .zip(clause_sum)
                            .map(|((a, b), c)| a + *b * c);
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
