use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    pasta::{EqAffine, Fp},
    plonk::{keygen_pk, keygen_vk, create_proof, verify_proof, Advice, Circuit, Column, ConstraintSystem, Error, Fixed, SingleVerifier}, poly::commitment::Params, transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use common::data::*; use rand_core::OsRng;
// your unchanged data.rs
use serde_json;
use std::{fs, sync::Arc};
use anyhow::Result;
// --------------------------------------------------
// 1️⃣ ConsistencyChip (same as before)
// --------------------------------------------------
#[derive(Clone, Debug)]
pub struct ConsistencyConfig {
    adv_pub: Column<Advice>,
    adv_wit: Column<Advice>,
    fixed_q: Column<Fixed>,
}

#[derive(Clone, Debug)]
pub struct ConsistencyChip {
    cfg: ConsistencyConfig,
}

impl ConsistencyChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> ConsistencyConfig {
        let adv_pub = meta.advice_column();
        let adv_wit = meta.advice_column();
        let fixed_q = meta.fixed_column();
        meta.enable_equality(adv_pub);
        meta.enable_equality(adv_wit);

        meta.create_gate("eq(name,arity)", |meta| {
            let q = meta.query_fixed(fixed_q);
            let pn = meta.query_advice(adv_pub, halo2_proofs::poly::Rotation::cur());
            let wn = meta.query_advice(adv_wit, halo2_proofs::poly::Rotation::cur());
            let pa = meta.query_advice(adv_pub, halo2_proofs::poly::Rotation::next());
            let wa = meta.query_advice(adv_wit, halo2_proofs::poly::Rotation::next());
            vec![q.clone() * (pn - wn), q * (pa - wa)]
        });
        ConsistencyConfig { adv_pub, adv_wit, fixed_q }
    }

    pub fn construct(cfg: ConsistencyConfig) -> Self {
        Self { cfg }
    }

    pub fn assign_pair(
        &self,
        mut layouter: impl Layouter<Fp>,
        (pn, pa, wn, wa): (Fp, Fp, Fp, Fp),
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "pair",
            |mut region| {
                region.assign_fixed(|| "q0", self.cfg.fixed_q, 0, || Value::known(Fp::one()))?;
                region.assign_fixed(|| "q1", self.cfg.fixed_q, 1, || Value::known(Fp::one()))?;
                region.assign_advice(|| "pn", self.cfg.adv_pub, 0, || Value::known(pn))?;
                region.assign_advice(|| "wn", self.cfg.adv_wit, 0, || Value::known(wn))?;
                region.assign_advice(|| "pa", self.cfg.adv_pub, 1, || Value::known(pa))?;
                region.assign_advice(|| "wa", self.cfg.adv_wit, 1, || Value::known(wa))?;
                Ok(())
            },
        )
    }
}

// --------------------------------------------------
// 2️⃣ BatchCircuit with full witness
// --------------------------------------------------
#[derive(Clone, Debug)]
pub struct BatchCircuit {
    pub rules: RuleTemplateFile,
    pub proof_tree: Vec<ProofNode>,
}

#[derive(Clone, Debug)]
pub struct BatchConfig {
    pub cons_cfg: ConsistencyConfig,
}

impl Circuit<Fp> for BatchCircuit {
    type Config = BatchConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        // Halo2 requirement: empty copy
        BatchCircuit { rules: RuleTemplateFile { predicates: vec![], facts: vec![] }, proof_tree: vec![] }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let cons_cfg = ConsistencyChip::configure(meta);
        BatchConfig { cons_cfg }
    }

    fn synthesize(
        &self,
        cfg: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let chip = ConsistencyChip::construct(cfg.cons_cfg);

        // 🔸 INSIDE CIRCUIT: build all consistency pairs
        for (i, node) in self.proof_tree.iter().enumerate() {
            if let ProofNode::GoalNode(g) = node {
                let pn = str_to_fp(&g.goal_term.name);
                let pa = Fp::from(g.goal_term.args.len() as u64);
                if let Some((wn_str, wa_len)) = parse_predicate_call(&g.goal_unification.goal) {
                    let wn = str_to_fp(&wn_str);
                    let wa = Fp::from(wa_len as u64);
                    chip.assign_pair(
                        layouter.namespace(|| format!("pair_{}", i)),
                        (pn, pa, wn, wa),
                    )?;
                }
            }
        }

        Ok(())
    }
}

// --------------------------------------------------
// 3️⃣ Helpers
// --------------------------------------------------
fn str_to_fp(s: &str) -> Fp {
    use halo2_proofs::arithmetic::Field;
    let mut acc = 0u64;
    for b in s.bytes() { acc = acc.wrapping_add(b as u64); }
    Fp::from(acc)
}

fn parse_predicate_call(s: &str) -> Option<(String, usize)> {
    let open = s.find('(')?;
    let close = s.find(')')?;
    let name = s[..open].trim().to_string();
    let args_str = &s[open + 1..close];
    let args = args_str.split(',').filter(|a| !a.trim().is_empty()).count();
    Some((name, args))
}

// --------------------------------------------------
// 4️⃣ Runner
// --------------------------------------------------
fn main() -> Result<()> {
    // Load JSON data (exactly as generated by Prolog)
    let rules: RuleTemplateFile = serde_json::from_str(&fs::read_to_string("input/rules_template.json")?)?;
    let tree: Vec<ProofNode> = serde_json::from_str(&fs::read_to_string("input/proof_tree.json")?)?;

    // Build circuit instance
    let circuit = BatchCircuit { rules, proof_tree: tree };

    // Setup SRS parameters (2^k rows)
    let k: u32 = 5;
    let params: Params<EqAffine> = Params::new(k);
    let params = Arc::new(params);

    // Key generation
    let vk = keygen_vk(&params, &circuit).expect("keygen_vk failed");
    let pk = keygen_pk(&params, vk.clone(), &circuit).expect("keygen_pk failed");

    // Prove (no public inputs yet)
    let mut proof_transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof(
        &params,
        &pk,
        &[circuit],
        &[&[]], // no public instances
        OsRng,
        &mut proof_transcript,
    )?;
    let proof: Vec<u8> = proof_transcript.finalize();

    // Verify
    let mut verify_transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
    let strategy = SingleVerifier::new(&params);
    verify_proof(
        &params,
        &vk,
        strategy,
        &[&[]], // must match instance structure above
        &mut verify_transcript,
    )
    .expect("verification failed");

    println!("✅ Real Halo2 proof created and verified successfully!");
    Ok(())
}
