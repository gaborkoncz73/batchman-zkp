use halo2_gadgets::utilities::FieldValue;
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Region, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression},
};
use halo2curves::ff::Field;

#[derive(Clone, Debug)]
pub struct DotExistsConfig {
    pub adv_w:    Column<Advice>,
    pub adv_c:    Column<Advice>,
    pub adv_dot:  Column<Advice>,

    pub adv_b:    Column<Advice>,
    pub adv_z:    Column<Advice>,
    pub adv_fact: Column<Advice>,

    pub adv_prod: Column<Advice>,
    pub adv_final: Column<Advice>,    // (1 - fact) * prod  MUST be 0
}

#[derive(Clone, Debug)]
pub struct DotExistsChip {
    pub cfg: DotExistsConfig,
}

impl Chip<Fp> for DotExistsChip {
    type Config = DotExistsConfig;
    type Loaded = ();
    fn config(&self) -> &Self::Config { &self.cfg }
    fn loaded(&self) -> &Self::Loaded { &() }
}

impl DotExistsChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> DotExistsConfig {
        let adv_w     = meta.advice_column();
        let adv_c     = meta.advice_column();
        let adv_dot   = meta.advice_column();

        let adv_b     = meta.advice_column();
        let adv_z     = meta.advice_column();
        let adv_fact  = meta.advice_column();
        
        let adv_prod  = meta.advice_column();
        let adv_final = meta.advice_column();

        for col in [adv_w, adv_c, adv_dot, adv_b, adv_z, adv_fact, adv_prod, adv_final] {
            meta.enable_equality(col);
        }

        // ⚠️ NINCS create_gate !!!
        // EZ EGY GATE-MENTES KIALAKÍTÁS

        DotExistsConfig {
            adv_w, adv_c, adv_dot, adv_b, adv_z, adv_fact, adv_prod, adv_final
        }
    }

    pub fn assign_exists_dot_zero(
        &self,
        mut layouter: impl Layouter<Fp>,
        w_vec: &[AssignedCell<Fp,Fp>],
        c_list: &Vec<Vec<AssignedCell<Fp,Fp>>>,
        b_flags: &Vec<AssignedCell<Fp,Fp>>,
        fact_cell: &AssignedCell<Fp,Fp>,
    ) -> Result<(), Error> {

        let cfg = self.cfg.clone();
        let dim = w_vec.len();

        layouter.assign_region(
            || "dot(z*b) with final enforcement",
            |mut region| {
                let mut row = 0;
                let mut prod_val = Value::known(Fp::one());   

                for (i, c_i) in c_list.iter().enumerate() {

                    // compute dot_i
                    let mut acc = Value::known(Fp::zero());
                    for k in 0..dim {
                        acc = acc.zip(w_vec[k].value()).zip(c_i[k].value())
                            .map(|((acc, w), c)| acc + (*w * *c));

                        region.assign_advice(|| "w", cfg.adv_w, row, || w_vec[k].value().copied())?;
                        region.assign_advice(|| "c", cfg.adv_c, row, || c_i[k].value().copied())?;
                        row += 1;
                    }

                    let dot_cell = region.assign_advice(
                        || format!("dot[{i}]"),
                        cfg.adv_dot,
                        row,
                        || acc
                    )?;

                    let b_local = region.assign_advice(
                        || format!("b[{i}]"),
                        cfg.adv_b,
                        row,
                        || b_flags[i].value().copied()
                    )?;
                    region.constrain_equal(b_local.cell(), b_flags[i].cell())?;

                    // z[i] = 1 if dot_i == 0
                    let z_val = dot_cell.value().map(|d|
                        if bool::from(d.is_zero()) {Fp::one()} else {Fp::zero()}
                    );
                    let z_cell = region.assign_advice(
                        || format!("z[{i}]"),
                        cfg.adv_z,
                        row,
                        || z_val
                    )?;

                    // multiply into product: prod *= (1 - z*b)
                    let zb_val = z_cell.value().zip(b_local.value())
                        .map(|(z,b)| *z * *b);

                    prod_val = prod_val.zip(zb_val)
                        .map(|(p,zb)| p * (Fp::one() - zb));

                    // also forward fact for this row
                    let fact_local = region.assign_advice(
                        || "fact",
                        cfg.adv_fact,
                        row,
                        || fact_cell.value().copied()
                    )?;
                    region.constrain_equal(fact_local.cell(), fact_cell.cell())?;

                    row += 1;
                }

                // store final product
                let prod_cell = region.assign_advice(
                    || "prod_final",
                    cfg.adv_prod,
                    row,
                    || prod_val
                )?;

                // compute final_check = (1 - fact) * prod
                let final_check_val = prod_val.zip(fact_cell.value())
                    .map(|(p,f)| (Fp::one() - *f) * p);

                let final_cell = region.assign_advice(
                    || "final_check",
                    cfg.adv_final,
                    row,
                    || final_check_val
                )?;
                //println!("FINAL: {:?}", final_cell.value());
                // ENFORCEMENT HERE:
                region.constrain_constant(final_cell.cell(), Fp::zero())?;

                Ok(())
            }
        )?;

        Ok(())
    }
}
