//! Measures the node circuits on the repository's proof tree, with the real
//! prover (IPA over Pasta): rows, k, key generation, proving, proof size and
//! verifying. Each per-node check on its own and all of them together run on
//! the first 1, 10 and all nodes; the tree circuit (5a) proves the whole tree
//! in 50, 64 and 128 node places; and the tree is split into shards (5b) of
//! at most 25, 13 and 7 nodes, each shard proved on its own.
//!
//! ```text
//! cargo run --release -p node-circuit --example measure
//! ```

use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use ff::Field;
use halo2_poseidon::poseidon::primitives::P128Pow5T3;
use halo2_proofs::dev::MockProver;
use halo2_proofs::halo2curves::pasta::EqAffine;
use halo2_proofs::plonk::{Circuit, create_proof, keygen_pk, keygen_vk, verify_proof};
use halo2_proofs::poly::VerificationStrategy;
use halo2_proofs::poly::commitment::ParamsProver;
use halo2_proofs::poly::ipa::commitment::{IPACommitmentScheme, ParamsIPA};
use halo2_proofs::poly::ipa::multiopen::{ProverIPA, VerifierIPA};
use halo2_proofs::poly::ipa::strategy::SingleStrategy;
use halo2_proofs::transcript::{
    Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
};
use node_circuit::{
    BranchCircuit, BranchTable, Claim, NodeBranch, NodeCircuit, TreeCircuit, WellFormedCircuit,
    link_commitments, minimum_k, root_digests, rows_used, shard_public,
};
use policy_constraints::{Fp, Slot, compile};
use proof_witness::{
    FactRegistry, ProofTree, TrustedRoots, build, check_links_between, parse_commitments, plan,
};
use rand_core::OsRng;

fn main() -> Result<()> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let read = |path: &str| {
        std::fs::read_to_string(root.join(path)).with_context(|| format!("reading {path}"))
    };
    let policy = compile(&serde_json::from_str(&read("input/rules.json")?)?)?;
    let tree = ProofTree::from_json(&read("input/proof_tree.json")?, "proof_tree.json")?;
    let facts = FactRegistry::from_yaml(&read("registry/facts.yaml")?, "facts.yaml")?;
    let roots = TrustedRoots::from_json(&read("input/trusted_roots.json")?, "trusted_roots.json")?;
    let commitments = parse_commitments(&read("registry/commitments.json")?, "commitments.json")?;
    let witness = build(&policy, &tree, &facts, &roots)?;
    let layout = policy.layout();
    let table = BranchTable::from_policy(&policy);
    let hashed = layout
        .opened_nodes()
        .iter()
        .filter(|node| node.position.slot() == Slot::Parent)
        .count();
    let counts = [1, 10, witness.nodes.len()];

    println!(
        "3a well-formedness: width {}, {} opened positions per node, {hashed} hashed",
        layout.width(),
        layout.opened_nodes().len()
    );
    header();
    for count in counts {
        let nodes = witness.nodes[..count]
            .iter()
            .map(|node| node.assignment.clone())
            .collect();
        measure(
            count,
            WellFormedCircuit::<Fp, P128Pow5T3>::new(layout.clone(), nodes),
            &[],
        )?;
    }

    println!(
        "\n3b branch check: {} branches, {} non-zero entries, powers c^0..c^{} and γ^0..γ^{}",
        table.branches(),
        table.entries(),
        policy
            .branches()
            .iter()
            .map(|b| b.rows.len())
            .max()
            .unwrap_or(1)
            - 1,
        table.width()
    );
    header();
    for count in counts {
        let nodes = witness.nodes[..count]
            .iter()
            .map(|node| NodeBranch {
                assignment: node.assignment.clone(),
                claim: Claim::Branch(node.branch),
            })
            .collect();
        measure(count, BranchCircuit::new(table.clone(), nodes), &[])?;
    }

    println!("\nall per-node checks (3a + 3b + 4a + 4b) on one w per node");
    header();
    for count in counts {
        let nodes = witness.nodes[..count]
            .iter()
            .map(|node| NodeBranch {
                assignment: node.assignment.clone(),
                claim: Claim::Branch(node.branch),
            })
            .collect();
        let circuit = NodeCircuit::<Fp, P128Pow5T3>::new(layout.clone(), table.clone(), nodes);
        measure(count, circuit, std::slice::from_ref(&commitments))?;
    }

    println!(
        "\nthe whole tree in one proof (5a): {} nodes in node places",
        witness.nodes.len()
    );
    println!("places  rows   k    keygen     prove     proof    verify");
    let instances = [commitments.clone(), root_digests(&roots), Vec::new()];
    for capacity in [witness.nodes.len(), 64, 128] {
        let circuit =
            TreeCircuit::<Fp, P128Pow5T3>::from_witness(&policy, &witness, &roots, capacity)
                .map_err(anyhow::Error::msg)?;
        measure(capacity, circuit, &instances)?;
    }

    println!("\nthe tree in shards (5b), one proof per shard, proved one after the other here");
    for capacity in [25, 13, 7] {
        let shards = plan(&policy, &witness, capacity, || Fp::random(OsRng))?;
        let incoming = shards
            .iter()
            .map(|shard| shard.incoming.len())
            .max()
            .unwrap_or(0);
        let outgoing = shards
            .iter()
            .map(|shard| shard.outgoing.len())
            .max()
            .unwrap_or(0);
        let links: usize = shards.iter().map(|shard| shard.incoming.len()).sum();
        println!(
            "\n{} shards of at most {capacity} nodes, {links} links, {incoming} incoming and {outgoing} outgoing places per shard",
            shards.len()
        );
        println!("shard   rows   k    keygen     prove     proof    verify");
        let mut measured = Vec::new();
        let mut publics = Vec::new();
        for (index, shard) in shards.iter().enumerate() {
            let trusted = (index == 0).then_some(&roots);
            let circuit = TreeCircuit::<Fp, P128Pow5T3>::from_shard(
                &policy, shard, trusted, capacity, incoming, outgoing,
            )
            .map_err(anyhow::Error::msg)?;
            let links = link_commitments(layout, shard, incoming, outgoing);
            publics.push(shard_public(&links, incoming));
            let root_column = if index == 0 {
                root_digests(&roots)
            } else {
                Vec::new()
            };
            let instances = [commitments.clone(), root_column, links];
            measured.push(measure(index, circuit, &instances)?);
        }
        check_links_between(&publics)?;
        let slowest = measured.iter().map(|m| m.prove).max().unwrap_or_default();
        let total: Duration = measured.iter().map(|m| m.prove).sum();
        let bytes: usize = measured.iter().map(|m| m.proof).sum();
        let verify: Duration = measured.iter().map(|m| m.verify).sum();
        println!(
            "  slowest shard {slowest:.2?}, all shards {total:.2?}, proofs {bytes} B, verify all {verify:.2?}"
        );
    }
    Ok(())
}

/// What one proof cost.
struct Measured {
    prove: Duration,
    proof: usize,
    verify: Duration,
}

fn header() {
    println!("nodes   rows   k    keygen     prove     proof    verify");
}

/// Mock-checks, then proves and verifies `circuit` with the real prover.
/// `instances` are the circuit's public input columns.
fn measure<C: Circuit<Fp>>(count: usize, circuit: C, instances: &[Vec<Fp>]) -> Result<Measured> {
    let columns: Vec<&[Fp]> = instances.iter().map(Vec::as_slice).collect();
    let rows = rows_used(&circuit);
    let k = minimum_k(&circuit);
    let mock = MockProver::run(k, &circuit, instances.to_vec())
        .unwrap()
        .verify()
        .is_ok();

    let (pk, params, keygen) = timed(|| -> Result<_> {
        let params: ParamsIPA<EqAffine> = ParamsIPA::new(k);
        let vk = keygen_vk(&params, &circuit)?;
        Ok((keygen_pk(&params, vk, &circuit)?, params))
    })
    .map(|((pk, params), elapsed)| (pk, params, elapsed))?;

    let (proof, prove) = timed(|| -> Result<Vec<u8>> {
        let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
        create_proof::<IPACommitmentScheme<EqAffine>, ProverIPA<EqAffine>, _, _, _, _>(
            &params,
            &pk,
            &[circuit],
            &[&columns],
            OsRng,
            &mut transcript,
        )?;
        Ok(transcript.finalize())
    })?;

    let (accepted, verify) = timed(|| -> Result<bool> {
        let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof[..]);
        Ok(
            verify_proof::<IPACommitmentScheme<EqAffine>, VerifierIPA<EqAffine>, _, _, _>(
                &params,
                pk.get_vk(),
                SingleStrategy::new(&params),
                &[&columns],
                &mut transcript,
            )
            .is_ok(),
        )
    })?;
    assert!(mock && accepted, "the honest tree must pass");

    println!(
        "{count:>5} {rows:>6} {k:>3} {keygen:>9.2?} {prove:>9.2?} {:>7} B {verify:>8.2?}",
        proof.len()
    );
    Ok(Measured {
        prove,
        proof: proof.len(),
        verify,
    })
}

fn timed<T>(run: impl FnOnce() -> Result<T>) -> Result<(T, Duration)> {
    let started = Instant::now();
    let value = run()?;
    Ok((value, started.elapsed()))
}
