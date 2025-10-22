/*use halo2_proofs::{
    circuit::{Chip, Layouter, Region, Value},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
    poly::Rotation,
    pasta::Fp,
};

// SUBCIRCUIT for consistency check
// Total constraints: 1 (independent from everything)
#[derive(Clone, Debug)]
pub struct ConsistencyConfig {
    pub adv_wit_1: Column<Advice>, // First witness
    pub adv_wit_2: Column<Advice>, // Second witness
    pub adv_wit_3: Column<Advice>, // Third witness (optinal)
    pub adv_flag: Column<Advice>,  // flag to decide whether its 2 or 3 pairs check
    pub fixed_q: Column<Fixed>,  // gate selector
}

// Chip definition
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
        let adv_wit_1 = meta.advice_column();
        let adv_wit_2 = meta.advice_column();
        let adv_wit_3 = meta.advice_column();
        let fixed_q = meta.fixed_column();
        let adv_flag = meta.advice_column();

        meta.enable_equality(adv_wit_1);
        meta.enable_equality(adv_wit_2);
        meta.enable_equality(adv_wit_3);
        meta.enable_equality(adv_flag);

        // Constraint: Ensures equality between 2 or 3 elements
        // Always enforces wit1 == wit2 (the first term).
        // If flag = 1, also enforces wit2 == wit3 (the second term).
        //  When flag = 0, the 3rd equality check is skipped (inactive).
        // This allows the same gate to verify either a 2-element or 3-element equality
        // depending on the flag value, saving constraints and keeping flexibility.
        meta.create_gate("2-or-3-element equality check", |meta| {
            let q = meta.query_fixed(fixed_q);
            let flag = meta.query_advice(adv_flag, Rotation::cur());

            let wit1  = meta.query_advice(adv_wit_1, Rotation::cur());
            let wit2  = meta.query_advice(adv_wit_2, Rotation::cur());
            let wit3 = meta.query_advice(adv_wit_3, Rotation::cur());

            vec![
                q * ((wit1.clone() - wit2.clone())*(wit1 - wit2.clone()) + flag * (wit2.clone() - wit3.clone())*(wit2 - wit3))
            ]
        });

        ConsistencyConfig { adv_wit_1, adv_wit_2, adv_wit_3, adv_flag, fixed_q }
    }

    // Assigns on a list of tuples (a_x, b_x) checks a_x == b_x
    pub fn assign_pairs2(
        &self,
        mut layouter: impl Layouter<Fp>,
        values_list: &[(Fp, Fp)],
    ) -> Result<(), Error> {
        let cfg = self.config();
        layouter.assign_region(
                ||"pair-2 equality region",
                |mut region: Region<'_, Fp>| {
                    for (i, &(a, b)) in values_list.iter().enumerate() {
                        region.assign_fixed(|| "q", cfg.fixed_q, i, || Value::known(Fp::one()))?;
                        region.assign_advice(|| "wit1", cfg.adv_wit_1, i, || Value::known(a))?;
                        region.assign_advice(|| "wit2", cfg.adv_wit_2, i, || Value::known(b))?;
                        region.assign_advice(|| "wit3", cfg.adv_wit_3, i, || Value::known(Fp::zero()))?;
                        region.assign_advice(|| "flag", cfg.adv_flag, i, || Value::known(Fp::zero()))?;
                    }
                
                Ok(())
            },
        )
    }   
    
    // Assigns one triple (a, b, c) checks a == b == c
    pub fn assign_pairs3(
        &self,
        mut layouter: impl Layouter<Fp>,
        values: (Fp, Fp, Fp),
    ) -> Result<(), Error> {
        let cfg = self.config();
        layouter.assign_region(
            || "pair3 equality region",
            |mut region: Region<'_, Fp>| {
                region.assign_fixed(|| "q", cfg.fixed_q, 0, || Value::known(Fp::one()))?;
                region.assign_advice(|| "wit1", cfg.adv_wit_1, 0, || Value::known(values.0))?;
                region.assign_advice(|| "wit2", cfg.adv_wit_2, 0, || Value::known(values.1))?;
                region.assign_advice(|| "wit3", cfg.adv_wit_3, 0, || Value::known(values.2))?;
                region.assign_advice(|| "flag", cfg.adv_flag, 0, || Value::known(Fp::one()))?;
                Ok(())
            },
        )
    }
}*/

use halo2_proofs::{
    circuit::{Chip, Layouter, Region, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
    poly::Rotation,
};

// ---------------------------
// Consistency chip: triple ellenőrzés
// ---------------------------
#[derive(Clone, Debug)]
pub struct ConsistencyConfig {
    pub goal: Column<Advice>,
    pub unif_goal: Column<Advice>,
    pub triple_l: Column<Advice>, // RLC(goal,args)
    pub triple_r: Column<Advice>, // RLC(unif_goal,body)
    pub q: Column<Fixed>,
}

pub struct ConsistencyChip {
    cfg: ConsistencyConfig,
}

impl Chip<Fp> for ConsistencyChip {
    type Config = ConsistencyConfig;
    type Loaded = ();
    fn config(&self) -> &Self::Config { &self.cfg }
    fn loaded(&self) -> &Self::Loaded { &() }
}

impl ConsistencyChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> ConsistencyConfig {
        let goal       = meta.advice_column();
        let unif_goal  = meta.advice_column();
        let triple_l   = meta.advice_column();
        let triple_r   = meta.advice_column();
        let q          = meta.fixed_column();

        meta.enable_equality(goal);
        meta.enable_equality(unif_goal);
        meta.enable_equality(triple_l);
        meta.enable_equality(triple_r);

        // Gate: enforce triple_l == triple_r, when q = 1
        meta.create_gate("Consistency triple equality", |meta| {
            let q    = meta.query_fixed(q);
            let lhs  = meta.query_advice(triple_l, Rotation::cur());
            let rhs  = meta.query_advice(triple_r, Rotation::cur());
            vec![ q * (lhs - rhs) ]
        });

        ConsistencyConfig { goal, unif_goal, triple_l, triple_r, q }
    }

    pub fn construct(cfg: ConsistencyConfig) -> Self { Self { cfg } }

    /// Assign goal, unif_goal, triple_l (RLC(goal,args)), triple_r (RLC(unif_goal,body))
    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        goal: Fp,
        unif_goal: Fp,
        triple_l: Fp,
        triple_r: Fp,
    ) -> Result<(), Error> {
        let cfg = self.config();

        layouter.assign_region(
            || "Consistency triple",
            |mut region: Region<'_, Fp>| {
                region.assign_fixed(|| "q", cfg.q, 0, || Value::known(Fp::one()))?;

                region.assign_advice(|| "goal", cfg.goal, 0, || Value::known(goal))?;
                region.assign_advice(|| "unif_goal", cfg.unif_goal, 0, || Value::known(unif_goal))?;
                region.assign_advice(|| "triple_l", cfg.triple_l, 0, || Value::known(triple_l))?;
                region.assign_advice(|| "triple_r", cfg.triple_r, 0, || Value::known(triple_r))?;
                Ok(())
            },
        )
    }
}
