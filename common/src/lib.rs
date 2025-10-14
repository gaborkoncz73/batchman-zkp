/*use std::{fs::File, io::BufReader};

use ark_bn254::{Bn254, Fr};
use ark_groth16::{ Groth16, VerifyingKey};
use ark_snark::SNARK;
use serde::{ de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};
use base64::{engine::general_purpose, Engine as _};
use serde_json::from_reader;
use anyhow::Result;

#[derive(CanonicalDeserialize, CanonicalSerialize)]
pub struct VerifyInput {
    pub vk: VerifyingKey<Bn254>,
    pub public_inputs: Vec<Fr>,
    pub proof: <Groth16<Bn254> as SNARK<Fr>>::Proof,
}

impl Serialize for VerifyInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::Error as _;
        let mut v = Vec::new();
        self.serialize_uncompressed(&mut v)
            .map_err(|e| S::Error::custom(e.to_string()))?;
        serializer.serialize_str(&general_purpose::STANDARD.encode(&v))
    }
}

impl<'de> Deserialize<'de> for VerifyInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error as _;
        let v = String::deserialize(deserializer)?;
        let decoded = general_purpose::STANDARD
            .decode(v)
            .map_err(|e| D::Error::custom(e.to_string()))?;
        Self::deserialize_uncompressed(&decoded[..]).map_err(|e|D::Error::custom(e.to_string()))
    }
}

pub fn load_input<T: DeserializeOwned>(path: &str) -> Result<T> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(from_reader(reader)?)
}*/





use std::sync::Arc;

use ark_groth16::{prepare_verifying_key, Groth16, PreparedVerifyingKey, ProvingKey};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_r1cs_std::{
    alloc::AllocVar,
    eq::EqGadget,
    fields::{fp::FpVar, FieldVar},
};
use ark_bn254::{Bn254, Fr};
use ark_snark::CircuitSpecificSetupSNARK;
use ark_ff::Zero;
// ---------------- ZK Circuit-ek ----------------

// 1) Dot-product: Σ c_i * w_i == 0
pub struct DotCircuit {
    pub c_vec: Vec<Fr>, // public
    pub w_vec: Vec<Fr>, // witness
}

impl ConstraintSynthesizer<Fr> for DotCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let c_var = Vec::<FpVar<Fr>>::new_input(cs.clone(), || Ok(self.c_vec))?;
        let w_var = Vec::<FpVar<Fr>>::new_witness(cs.clone(), || Ok(self.w_vec))?;
        let mut acc = FpVar::<Fr>::zero();
        for (a, b) in c_var.iter().zip(w_var.iter()) {
            acc += a * b;
        }
        acc.enforce_equal(&FpVar::<Fr>::zero())?;

        // 2) konstans oszlop = 1 (utolsó elem)
        let one = FpVar::<Fr>::one();
        let last = w_var.last().ok_or(SynthesisError::AssignmentMissing)?;
        last.enforce_equal(&one)?;

        for wi in w_var.iter().take(w_var.len()-1) { // az utolsó a konstans
        (wi.clone() * (wi.clone() - FpVar::<Fr>::one()))
            .enforce_equal(&FpVar::<Fr>::zero())?;
        }
        Ok(())
    }
}

// 2) Szigorú szintaxis-egyezés (név + aritás) külön KÉT kényszerrel, nem összegezve
// A cél: bizonyítani, hogy (name_a == name_b) ÉS (arity_a == arity_b)
// Itt a *_a mezők public, a *_b mezők witness (de lehetne fordítva is).
pub struct ConsistencyCircuit {
    pub pub_name: Fr,
    pub wit_name: Fr,
    pub pub_arity: Fr,
    pub wit_arity: Fr,
}

impl ConstraintSynthesizer<Fr> for ConsistencyCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let pub_name  = FpVar::<Fr>::new_input(cs.clone(),  || Ok(self.pub_name))?;
        let pub_arity = FpVar::<Fr>::new_input(cs.clone(),  || Ok(self.pub_arity))?;
        let wit_name  = FpVar::<Fr>::new_witness(cs.clone(),|| Ok(self.wit_name))?;
        let wit_arity = FpVar::<Fr>::new_witness(cs.clone(),|| Ok(self.wit_arity))?;

        // 1) név egyezés (külön constraint)
        (pub_name - wit_name).enforce_equal(&FpVar::<Fr>::zero())?;

        // 2) aritás egyezés (külön constraint)
        (pub_arity - wit_arity).enforce_equal(&FpVar::<Fr>::zero())?;
        Ok(())
    }
}

pub struct ProvingKeyStore {
    pub consistency_pk: Arc<ProvingKey<Bn254>>,
    pub consistency_pvk: Arc<PreparedVerifyingKey<Bn254>>,
    pub dot_pk: Arc<ProvingKey<Bn254>>,
    pub dot_pvk: Arc<PreparedVerifyingKey<Bn254>>,
}

impl ProvingKeyStore {
    /// Létrehozza az összes proving/verifying kulcsot 1×
    pub fn new() -> Self {
        let mut rng = ark_std::rand::thread_rng();

        // 🔹 ConsistencyCircuit setup
        let cons_circuit = ConsistencyCircuit {
            pub_name:  Fr::zero(),
            wit_name:  Fr::zero(),
            pub_arity: Fr::zero(),
            wit_arity: Fr::zero(),
        };
        let (cons_pk, cons_vk) = Groth16::<Bn254>::setup(cons_circuit, &mut rng)
            .expect("failed setup for consistency circuit");
        let cons_pvk = prepare_verifying_key(&cons_vk);

        // 🔹 DotCircuit setup (ha a max hossz ismert, pl. 128)
        let dot_circuit = DotCircuit {
            c_vec: vec![Fr::zero(); 128],
            w_vec: vec![Fr::zero(); 128],
        };
        let (dot_pk, dot_vk) = Groth16::<Bn254>::setup(dot_circuit, &mut rng)
            .expect("failed setup for dot circuit");
        let dot_pvk = prepare_verifying_key(&dot_vk);

        Self {
            consistency_pk: Arc::new(cons_pk),
            consistency_pvk: Arc::new(cons_pvk),
            dot_pk: Arc::new(dot_pk),
            dot_pvk: Arc::new(dot_pvk),
        }
    }
}