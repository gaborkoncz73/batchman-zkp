use halo2_proofs::{
    circuit::{Chip, Layouter, Region, Value},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
    poly::Rotation,
    pasta::Fp,
};

// --------------------------
// Config: két soros private equality gate
// --------------------------
#[derive(Clone, Debug)]
pub struct ConsistencyConfig {
    pub adv_pub: Column<Advice>, // row0 = pub_name, row1 = pub_arity
    pub adv_wit: Column<Advice>, // row0 = wit_name, row1 = wit_arity
    pub fixed_q: Column<Fixed>,  // gate selector
}

// --------------------------
// Chip definition
// --------------------------
pub struct ConsistencyChip {
    config: ConsistencyConfig,
}

impl Chip<Fp> for ConsistencyChip {
    type Config = ConsistencyConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl ConsistencyChip {
    pub fn construct(config: ConsistencyConfig) -> Self {
        Self { config }
    }

    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> ConsistencyConfig {
        let adv_pub = meta.advice_column();
        let adv_wit = meta.advice_column();
        let fixed_q = meta.fixed_column();

        meta.enable_equality(adv_pub);
        meta.enable_equality(adv_wit);

        meta.create_gate("2-row private equality", |meta| {
            let q = meta.query_fixed(fixed_q);

            let pub_name  = meta.query_advice(adv_pub, Rotation::cur());
            let wit_name  = meta.query_advice(adv_wit, Rotation::cur());
            let pub_arity = meta.query_advice(adv_pub, Rotation::next());
            let wit_arity = meta.query_advice(adv_wit, Rotation::next());

            vec![
                q.clone() * (wit_name - pub_name),
                q * (wit_arity - pub_arity),
            ]
        });

        ConsistencyConfig { adv_pub, adv_wit, fixed_q }
    }

    // Assign a single pair (goal vs unification) into 2 rows
    pub fn assign_pairs( &self, mut layouter: impl Layouter<Fp>, pairs: Vec<(Fp, Fp, Fp, Fp)>, ) -> Result<(), Error> {
        let cfg = self.config();
        layouter.assign_region( || "consistency region (multi-pair)", |mut region| {
            for (i, (goal_name, goal_arity, unif_name, unif_arity)) in pairs.iter().enumerate() { // each pair uses 2 rows
                let row_name = i * 2; let row_arity = row_name + 1; // enable gate selectorfor both rows
                region.assign_fixed( || format!("q_name_{}", i), cfg.fixed_q, row_name, || Value::known(Fp::one()), )?;
                region.assign_fixed( || format!("q_arity_{}", i), cfg.fixed_q, row_arity, || Value::known(Fp::one()), )?;// row 0 = names
                region.assign_advice( || format!("goal_name_{}", i), cfg.adv_pub, row_name, || Value::known(*goal_name), )?;
                region.assign_advice( || format!("unif_name_{}", i), cfg.adv_wit, row_name, || Value::known(*unif_name), )?; // row 1 = arities
                region.assign_advice( || format!("goal_arity_{}", i), cfg.adv_pub, row_arity, || Value::known(*goal_arity), )?;
                region.assign_advice( || format!("unif_arity_{}", i), cfg.adv_wit, row_arity, || Value::known(*unif_arity), )?; 
            }
    
            Ok(()) }, ) }
    pub fn assign_pair2(
        &self,
        mut layouter: impl Layouter<Fp>,
        values: (Fp, Fp),
    ) -> Result<(), Error> {
        let cfg = self.config();
        layouter.assign_region(
            || "simple equality",
            |mut region| {
                region.assign_fixed(|| "q", cfg.fixed_q, 0, || Value::known(Fp::one()))?;
                region.assign_advice(|| "pub", cfg.adv_pub, 0, || Value::known(values.0))?;
                region.assign_advice(|| "wit", cfg.adv_wit, 0, || Value::known(values.1))?;
                Ok(())
            },
        )
    }
}
