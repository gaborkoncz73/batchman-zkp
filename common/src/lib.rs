pub mod data;
pub mod unification_checker_circuit;
mod utils;
mod chips;
mod utils_2;
use crate::chips::{ConsistencyChip, DotChip};

use std::sync::Arc;

use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    pasta::{EqAffine, Fp},
    plonk::{
        Advice, Circuit, Column, ConstraintSystem, Constraints, Error, Expression, Fixed, Instance,
        ProvingKey, VerifyingKey, SingleVerifier,
        create_proof, keygen_pk, keygen_vk, verify_proof,
    },
    poly::commitment::Params,
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use rand_core::OsRng;
use halo2_proofs::poly::Rotation;

// DOT PRODUCT CIRCUIT

// Configuration for the dot-product circuit
#[derive(Clone, Debug)]
pub struct DotConfig {
    adv_w: Column<Advice>,     // w[i] witness inputs
    adv_acc: Column<Advice>,   // running sum
    fixed_q: Column<Fixed>,    // selector
    fixed_last: Column<Fixed>, // last-row flag
    fixed_first: Column<Fixed>,// first-row flag
    #[allow(dead_code)]
    instance_c: Column<Instance>, // c[i] public inputs
    instance_flag: Column<Instance>,
}

// Circuit enforcing dot(c, w) = 0 with boolean w constraints
#[derive(Clone, Debug)]
pub struct DotCircuit {
    pub c_vec: Vec<Fp>, // public (Instance)
    pub w_vec: Vec<Fp>, // witness (Advice)
}

impl Circuit<Fp> for DotCircuit {
    type Config = DotConfig;
    type FloorPlanner = SimpleFloorPlanner;

    // Returns a copy of the circuit with zeroed witnesses
    fn without_witnesses(&self) -> Self {
        Self {
            c_vec: vec![Fp::from(0); self.c_vec.len()],
            w_vec: vec![Fp::from(0); self.w_vec.len()],
        }
    }

    // Defines columns and gates for the dot-product constraints
    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let adv_w      = meta.advice_column();
        let adv_acc    = meta.advice_column();
        let fixed_q    = meta.fixed_column();
        let fixed_last = meta.fixed_column();
        let fixed_first= meta.fixed_column();
        let instance_c   = meta.instance_column();
        let instance_flag = meta.instance_column();

        meta.enable_equality(adv_w);
        meta.enable_equality(adv_acc);
        meta.enable_equality(instance_c);
        meta.enable_equality(instance_flag);

        // First row: acc_0 = w_0 * c_0
        meta.create_gate("first row acc0 = w0*c0", |meta| {
            let q_first = meta.query_fixed(fixed_first);
            let w0   = meta.query_advice(adv_w,   Rotation::cur());
            let c0   = meta.query_instance(instance_c, Rotation::cur());
            let acc0 = meta.query_advice(adv_acc, Rotation::cur());
            Constraints::with_selector(q_first, [ acc0 - w0 * c0 ])
        });

        // Running sum for rows i>0: acc_i = acc_i-1 + w_i * c_i
        // selector = q * (1 - first)
        meta.create_gate("running sum", |meta| {
            let q        = meta.query_fixed(fixed_q);
            let is_first = meta.query_fixed(fixed_first);
            let sel      = q * (Expression::Constant(Fp::one()) - is_first);

            let wi   = meta.query_advice(adv_w,   Rotation::cur());
            let ci   = meta.query_instance(instance_c, Rotation::cur());
            let acci = meta.query_advice(adv_acc, Rotation::cur());
            let accp = meta.query_advice(adv_acc, Rotation::prev()); // no next!

            Constraints::with_selector(sel, [ acci - accp - wi * ci ])
        });

        // Boolean for non-last rows: w*(w-1)=0
        meta.create_gate("boolean non-last (optional)", |meta| {
            let q        = meta.query_fixed(fixed_q);
            let is_last  = meta.query_fixed(fixed_last);
            let sel      = q * (Expression::Constant(Fp::one()) - is_last);

            let w = meta.query_advice(adv_w, Rotation::cur());
            let enforce  = meta.query_instance(instance_flag, Rotation::cur());
            Constraints::with_selector(sel, [ enforce * w.clone() * (w - Expression::Constant(Fp::one())) ])
        });

        // Last row: w_last = 1  AND  acc_last = 0
        meta.create_gate("last row constraints", |meta| {
            let q       = meta.query_fixed(fixed_q);
            let is_last = meta.query_fixed(fixed_last);
            let sel     = q * is_last;

            let w_last   = meta.query_advice(adv_w,   Rotation::cur());
            let acc_last = meta.query_advice(adv_acc, Rotation::cur());

            Constraints::with_selector(sel, [
                w_last - Expression::Constant(Fp::one()),
                acc_last, // == 0
            ])
        });

        DotConfig { adv_w, adv_acc, fixed_q, fixed_last, fixed_first, instance_c, instance_flag }
    }

    // Assigns witness and fixed values for each circuit row
    fn synthesize(
        &self,
        cfg: Self::Config,
        mut layouter: impl Layouter<Fp>
    ) -> Result<(), Error> {
        let n = self.w_vec.len();
        assert_eq!(self.c_vec.len(), n, "c_vec and w_vec len mismatch");
        assert!(n >= 1, "need at least one row");

        layouter.assign_region(
        || "dot region (prev-based)",
        |mut region| {
            for i in 0..n {
                // q = 1 on all rows
                region.assign_fixed(|| "q", cfg.fixed_q, i, || Value::known(Fp::one()))?;

                // FIRST flag
                region.assign_fixed(|| "first", cfg.fixed_first, i, ||   // use fixed_first
                    Value::known(if i == 0 { Fp::one() } else { Fp::zero() })
                )?;

                // LAST flag
                region.assign_fixed(|| "last", cfg.fixed_last, i, || {
                    Value::known(if i + 1 == n { Fp::one() } else { Fp::zero() })
                })?;

                // w[i]
                region.assign_advice(|| "w", cfg.adv_w, i, || Value::known(self.w_vec[i]))?;
            }

                // Accumulation values (explicitly compute them)
                // acc_0 = w_0*c_0
                let mut acc = self.w_vec[0] * self.c_vec[0];
                region.assign_advice(|| "acc[0]", cfg.adv_acc, 0, || Value::known(acc))?;

                for i in 1..n {
                    acc += self.w_vec[i] * self.c_vec[i];
                    region.assign_advice(|| format!("acc[{i}]"), cfg.adv_acc, i, || Value::known(acc))?;
                }

                Ok(())
            }
        )?;

        Ok(())
    }
}

// CONSISTENCY CIRCUIT

// Config for private name/arity equality check
#[derive(Clone, Debug)]
pub struct ConsistencyConfig {
    adv_pub: Column<Advice>, // row0 = pub_name, row1 = pub_arity
    adv_wit: Column<Advice>, // row0 = wit_name, row1 = wit_arity
    fixed_q: Column<Fixed>, // gate selector
}

// Circuit enforcing pub_name==wit_name and pub_arity==wit_arity
#[derive(Clone, Debug)]
pub struct ConsistencyCircuit {
    pub_name:  Fp,
    pub_arity: Fp,
    wit_name:  Fp,
    wit_arity: Fp,
}

impl Circuit<Fp> for ConsistencyCircuit {
    type Config = ConsistencyConfig;
    type FloorPlanner = SimpleFloorPlanner;

    // Returns a copy with zeroed private values
    fn without_witnesses(&self) -> Self {
        Self {
            pub_name: Fp::zero(),
            pub_arity: Fp::zero(),
            wit_name: Fp::zero(),
            wit_arity: Fp::zero(),
        }
    }

    // Defines equality constraints between public and witness values
    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let adv_pub = meta.advice_column();
        let adv_wit = meta.advice_column();
        let fixed_q = meta.fixed_column();

        meta.enable_equality(adv_pub);
        meta.enable_equality(adv_wit);

        // Two-row equality: (pub_name==wit_name, pub_arity==wit_arity)
        meta.create_gate("private equality on 2 rows", |meta| {
            let q = meta.query_fixed(fixed_q);

            let pub_name  = meta.query_advice(adv_pub, Rotation::cur());
            let wit_name  = meta.query_advice(adv_wit, Rotation::cur());
            let pub_arity = meta.query_advice(adv_pub, Rotation::next());
            let wit_arity = meta.query_advice(adv_wit, Rotation::next());

            Constraints::with_selector(q, [
                wit_name - pub_name,
                wit_arity - pub_arity,
            ])
        });

        ConsistencyConfig { adv_pub, adv_wit, fixed_q }
    }

    // Assigns advice and selector values for both rows
    fn synthesize(
        &self,
        cfg: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "consistency region (all private)",
            |mut region| {
                // enable gate for both rows
                region.assign_fixed(|| "q0", cfg.fixed_q, 0, || Value::known(Fp::one()))?;
                region.assign_fixed(|| "q1", cfg.fixed_q, 1, || Value::known(Fp::one()))?;

                // row 0 = names
                region.assign_advice(|| "pub_name", cfg.adv_pub, 0, || Value::known(self.pub_name))?;
                region.assign_advice(|| "wit_name", cfg.adv_wit, 0, || Value::known(self.wit_name))?;

                // row 1 = arities
                region.assign_advice(|| "pub_arity", cfg.adv_pub, 1, || Value::known(self.pub_arity))?;
                region.assign_advice(|| "wit_arity", cfg.adv_wit, 1, || Value::known(self.wit_arity))?;
                Ok(())
            },
        )
    }
}


// Keys/Parameters

// Holds proving and verifying keys for all circuits
pub struct ProvingKeyStore {
    pub params: Arc<Params<EqAffine>>,
    pub dot_vk: Arc<VerifyingKey<EqAffine>>,
    pub dot_pk: Arc<ProvingKey<EqAffine>>,
    pub cons_vk: Arc<VerifyingKey<EqAffine>>,
    pub cons_pk: Arc<ProvingKey<EqAffine>>,
    pub max_dim: usize,
}

impl ProvingKeyStore {
    // Generates parameter set and keys for both circuits
    pub fn new(max_dim: usize, k: u32) -> Self {
        let params = Arc::new(Params::<EqAffine>::new(k));

        // empty templates for key generation
        let empty_dot = DotCircuit {
            c_vec: vec![Fp::zero(); max_dim],
            w_vec: vec![Fp::zero(); max_dim],
        };
        let empty_cons = ConsistencyCircuit {
            pub_name: Fp::zero(), pub_arity: Fp::zero(),
            wit_name: Fp::zero(), wit_arity: Fp::zero()
        };

        // generate keys for both circuits
        let dot_vk  = keygen_vk(&params, &empty_dot).expect("vk gen failed (dot)");
        let dot_pk  = keygen_pk(&params, dot_vk.clone(), &empty_dot).expect("pk gen failed (dot)");
        let cons_vk = keygen_vk(&params, &empty_cons).expect("vk gen failed (consistency)");
        let cons_pk = keygen_pk(&params, cons_vk.clone(), &empty_cons).expect("pk gen failed (consistency)");

        Self {
            params,
            dot_vk: Arc::new(dot_vk),
            dot_pk: Arc::new(dot_pk),
            cons_vk: Arc::new(cons_vk),
            cons_pk: Arc::new(cons_pk),
            max_dim,
        }
    }
}

// Helper: prove/verify

// Generates a proof for the Dot circuit: sum(c[i]*w[i]) = 0
pub fn prove_dot(
    pks: &ProvingKeyStore,
    c_vec: &[Fp],
    w_vec: &[Fp],
    enforce_flag: Fp,
) -> anyhow::Result<(Vec<u8>, Vec<Vec<Fp>>)> {
    anyhow::ensure!(c_vec.len() == w_vec.len(), "c_vec/w_vec mismatch");
    anyhow::ensure!(c_vec.len() <= pks.max_dim, "padding needed!");

    let n = c_vec.len();
    let flag_vec: Vec<Fp> = std::iter::repeat(enforce_flag).take(n).collect();

    let circuit = DotCircuit { c_vec: c_vec.to_vec(), w_vec: w_vec.to_vec() };
    let instances: Vec<Vec<Fp>> = vec![c_vec.to_vec(), flag_vec];
    let inst_refs: Vec<&[Fp]> = instances.iter().map(|v| v.as_slice()).collect();
    let inst_cols: Vec<&[&[Fp]]> = vec![&inst_refs];

    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof(&pks.params, &pks.dot_pk, &[circuit], &inst_cols, OsRng, &mut transcript)?;

    Ok((transcript.finalize(), instances))
}

// Verifies a Dot circuit proof against public inputs
pub fn verify_dot(
    pks: &ProvingKeyStore,
    proof: &[u8],
    c_vec: &[Fp],
    enforce_flag: Fp,   // ⬅️ verifier also decides the mode!
) -> anyhow::Result<bool> {
    anyhow::ensure!(c_vec.len() <= pks.max_dim, "padding needed!");
    let n = c_vec.len();
    let flag_vec: Vec<Fp> = std::iter::repeat(enforce_flag).take(n).collect();

    let instances: Vec<Vec<Fp>> = vec![ c_vec.to_vec(), flag_vec ];
    let inst_refs: Vec<&[Fp]> = instances.iter().map(|v| v.as_slice()).collect();
    let inst_cols: Vec<&[&[Fp]]> = vec![ &inst_refs ];

    let strategy = SingleVerifier::new(&pks.params);
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(proof);
    let ok = verify_proof(
        &pks.params,
        &pks.dot_vk,
        strategy,
        &inst_cols,
        &mut transcript,
    ).is_ok();

    Ok(ok)
}


// Generates a proof for the Consistency circuit
// Ensures (pub_name, pub_arity) == (wit_name, wit_arity)
pub fn prove_consistency(
    pks: &ProvingKeyStore,
    pub_name: Fp,
    pub_arity: Fp,
    wit_name: Fp,
    wit_arity: Fp,
) -> anyhow::Result<Vec<u8>> {
    let circuit = ConsistencyCircuit {
        pub_name,
        pub_arity,
        wit_name,
        wit_arity,
    };

    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof(
        &pks.params,
        &pks.cons_pk,
        &[circuit],
        &[&[]],  // no public inputs
        OsRng,
        &mut transcript,
    )?;
    Ok(transcript.finalize())
}

// Verifies a Consistency circuit proof (private-only)
pub fn verify_consistency(pks: &ProvingKeyStore, proof: &[u8]) -> anyhow::Result<bool> {
    let strategy = SingleVerifier::new(&pks.params);
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(proof);

    let ok = verify_proof(
        &pks.params,
        &pks.cons_vk,
        strategy,
        &[&[]],
        &mut transcript,
    ).is_ok();

    Ok(ok)
}