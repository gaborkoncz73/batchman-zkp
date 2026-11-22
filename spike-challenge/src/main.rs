//! Proves and verifies the branch circuit on the repository's proof tree with
//! the real prover (IPA over the Pasta curves, as in the main workspace), and
//! prints sizes and timings.
//!
//! ```text
//! cargo run --release --manifest-path spike-challenge/Cargo.toml
//! ```

use std::time::Instant;

use anyhow::Result;
use ff::Field;
use halo2_proofs::halo2curves::pasta::{EqAffine, Fp};
use halo2_proofs::plonk::{create_proof, keygen_pk, keygen_vk, verify_proof};
use halo2_proofs::poly::VerificationStrategy;
use halo2_proofs::poly::commitment::ParamsProver;
use halo2_proofs::poly::ipa::commitment::{IPACommitmentScheme, ParamsIPA};
use halo2_proofs::poly::ipa::multiopen::{ProverIPA, VerifierIPA};
use halo2_proofs::poly::ipa::strategy::SingleStrategy;
use halo2_proofs::transcript::{
    Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
};
use policy_constraints::Cell;
use rand_core::OsRng;
use spike_challenge::{BranchCircuit, BranchTable, Claim, NodeInput, minimum_k, repository_inputs};

fn main() -> Result<()> {
    let (policy, witness) = repository_inputs()?;
    let table = BranchTable::from_policy(&policy);
    let honest = BranchCircuit::from_witness(&policy, &witness);
    let k = minimum_k(honest.rows());
    println!(
        "{} branches, {} table rows, width {} (+1 constant), {} nodes → {} rows, k = {k}",
        table.branches(),
        table.rows(),
        table.width(),
        witness.nodes.len(),
        honest.rows()
    );

    let started = Instant::now();
    let params: ParamsIPA<EqAffine> = ParamsIPA::new(k);
    let vk = keygen_vk(&params, &honest)?;
    let pk = keygen_pk(&params, vk, &honest)?;
    println!("setup + keygen   {:>8.2?}", started.elapsed());

    let prove = |circuit: BranchCircuit| -> Result<Vec<u8>> {
        let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
        create_proof::<IPACommitmentScheme<EqAffine>, ProverIPA<EqAffine>, _, _, _, _>(
            &params,
            &pk,
            &[circuit],
            &[&[]],
            OsRng,
            &mut transcript,
        )?;
        Ok(transcript.finalize())
    };
    let verify = |proof: &[u8]| -> bool {
        let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(proof);
        verify_proof::<IPACommitmentScheme<EqAffine>, VerifierIPA<EqAffine>, _, _, _>(
            &params,
            pk.get_vk(),
            SingleStrategy::new(&params),
            &[&[]],
            &mut transcript,
        )
        .is_ok()
    };

    let started = Instant::now();
    let proof = prove(honest)?;
    println!("prove            {:>8.2?}", started.elapsed());
    println!("proof size       {:>8} bytes", proof.len());

    let started = Instant::now();
    let accepted = verify(&proof);
    println!("verify           {:>8.2?}  accepted = {accepted}", started.elapsed());

    // A cheating prover: node 2 claims one more child than it has.
    let child_count = policy.layout().column(&Cell::ChildCount).expect("always opened");
    let mut nodes: Vec<NodeInput> = witness
        .nodes
        .iter()
        .map(|node| NodeInput { assignment: node.assignment.clone(), claim: Claim::Branch(node.branch) })
        .collect();
    nodes[2].assignment[child_count] += Fp::ONE;
    let cheating = prove(BranchCircuit::new(table, nodes))?;
    println!("cheating proof   accepted = {}", verify(&cheating));
    Ok(())
}
