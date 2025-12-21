use halo2_proofs::{
    circuit::{Layouter, AssignedCell, Value},
    plonk::{Column, Advice, ConstraintSystem, Error},
    pasta::Fp,
};

#[derive(Clone, Debug)]
pub struct RowsCompressConfig {
    pub val: Column<Advice>,   // nem használjuk itt, de meghagyjuk kompat miatt
    pub acc: Column<Advice>,   // ide írjuk a c_i[k]-et
}

#[derive(Clone, Debug)]
pub struct RowsCompressChip {
    pub cfg: RowsCompressConfig,
}

impl RowsCompressChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> RowsCompressConfig {
        let val = meta.advice_column();
        let acc = meta.advice_column();
        meta.enable_equality(val);
        meta.enable_equality(acc);
        RowsCompressConfig { val, acc }
    }

    pub fn construct(cfg: RowsCompressConfig) -> Self { Self { cfg } }

    /// Minden klózhoz külön compressed vektort állítunk elő:
    /// c_i[k] = sum_{j} row_{i,j}[k]
    ///
    /// rows: [num_clauses][num_rows][dim]
    pub fn assign_compressed_all(
    &self,
    mut layouter: impl Layouter<Fp>,
    rows: &Vec<Vec<Vec<AssignedCell<Fp,Fp>>>>, // [actual_clauses][rows][dim]
    _flags: &Vec<AssignedCell<Fp,Fp>>,          // [MAX_CANDIDATES]
    max_candidates: usize,
    dim: usize,
) -> Result<Vec<Vec<AssignedCell<Fp,Fp>>>, Error> {

    let cfg = self.cfg.clone();
    let actual = rows.len();

    layouter.assign_region(
        || "all compressed c-vectors",
        |mut region| {
            let mut c_all: Vec<Vec<AssignedCell<Fp,Fp>>> = Vec::new();
            let mut offset = 0;

            for i in 0..max_candidates {

                let mut c_i: Vec<AssignedCell<Fp,Fp>> = Vec::with_capacity(dim);

                for k in 0..dim {
                    let mut sum_val = Value::known(Fp::zero());

                    // ha van row ehhez a clause-hoz
                    if i < actual {
                        for row in &rows[i] {
                            let val = row[k].value();
                            sum_val = sum_val.zip(val).map(|(a,v)| a + *v);
                        }
                    }

                    let c_cell = region.assign_advice(
                        || format!("c[{i}][{k}]"),
                        cfg.acc,
                        offset,
                        || sum_val,
                    )?;

                    c_i.push(c_cell);
                    offset += 1;
                }

                c_all.push(c_i);
            }

            Ok(c_all)
        },
    )
}
}
