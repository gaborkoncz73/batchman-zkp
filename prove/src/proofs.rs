use std::sync::{Arc, Mutex};
use halo2_proofs::pasta::Fp;
use anyhow::Result;
use common::*;
use crate::utils::*;

pub type StoredDot = (Vec<Fp>, Vec<u8>);
pub type StoredCons = Vec<u8>;

#[derive(Clone)]
pub struct ProofStore {
    pub dot_proofs: Arc<Mutex<Vec<StoredDot>>>,
    pub cons_proofs: Arc<Mutex<Vec<StoredCons>>>,
}

pub fn prove_consistency(
    name_a: &str,
    arity_a: usize,
    name_b: &str,
    arity_b: usize,
    proofs: &ProofStore,
    pk_store: &Arc<ProvingKeyStore>,
) -> Result<()> {
    //println!("{:?} vs {:?}", name_a, name_b);
    let pub_name = str_to_fp(name_a);
    let pub_arity = Fp::from(arity_a as u64);
    let wit_name = str_to_fp(name_b);
    let wit_arity = Fp::from(arity_b as u64);

    let proof = common::prove_consistency(pk_store, pub_name, pub_arity, wit_name, wit_arity)?;
    proofs.cons_proofs.lock().unwrap().push(proof);
    Ok(())
}
