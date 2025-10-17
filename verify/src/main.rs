mod reader;

use rayon::prelude::*;
use reader::read_proofs;
use std::sync::Arc;
use common::*;
use halo2_proofs::{
    pasta::EqAffine,
    plonk::{verify_proof, SingleVerifier},
    transcript::{Blake2bRead, Challenge255},
};

pub const MAX_DOT_DIM: usize = 7;

fn main() -> anyhow::Result<()> {
    let pk_store = Arc::new(ProvingKeyStore::new(MAX_DOT_DIM, 5));

    // Load all proofs from files
    let dot_proofs = read_proofs("dot")?;
    let cons_proofs = read_proofs("cons")?;

    println!("Verifying {} dot proofs, {} consistency proofs", dot_proofs.len(), cons_proofs.len());

    // Batch verify
    let (dot_ok, cons_ok) = rayon::join(
        || {
            dot_proofs.par_iter().all(|(inputs, proof)| {
                let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof[..]);
                let strategy = SingleVerifier::new(&pk_store.params);
                verify_proof(
                    &pk_store.params,
                    &pk_store.dot_vk,
                    strategy,
                    &[&[&inputs[..]]],
                    &mut transcript,
                ).is_ok()
            })
        },
        || {
            cons_proofs.par_iter().all(|(_, proof)| {
                let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof[..]);
                let strategy = SingleVerifier::new(&pk_store.params);
                verify_proof(
                    &pk_store.params,
                    &pk_store.cons_vk,
                    strategy,
                    &[&[]],
                    &mut transcript,
                ).is_ok()
            })
        },
    );

    if dot_ok && cons_ok {
        println!("All proofs verified successfully!");
    } else {
        println!("At least one proof failed verification!");
    }
    println!("Total constraints: {}", 5 * dot_proofs.len() + 2 * cons_proofs.len());

    Ok(())
}