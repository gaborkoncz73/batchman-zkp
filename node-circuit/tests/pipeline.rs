//! The prover and the verifier end to end (what the `prove` and `verify`
//! binaries run), with the real prover: a planned tree, its shard files and
//! proof files, and the verifier's rejections.

use ff::Field;
use node_circuit::{ShardKey, ShardProof, Sizes, prove_shard, verify_proofs};
use policy_constraints::{ConstraintSystem, Fp, compile};
use policy_parser::parse_policy;
use proof_witness::{FactRegistry, ProofTree, Shard, TrustedRoots, build, plan};

/// A chain `r → p → q(1)`: one node per shard, three shards, two links.
const POLICY: &str = "r :- p.\np :- q(1).\n";
const TREE: &str = r#"[{"goal": "r", "subtree": [{"goal": "p", "subtree": [{"goal": "q(1)", "subtree": [true]}]}]}]"#;
const FACTS: &str = "- goal: \"q(1)\"\n  salt: \"5\"\n";
const SIZES: Sizes = Sizes {
    capacity: 1,
    incoming: 1,
    outgoing: 1,
};

struct Run {
    system: ConstraintSystem,
    roots: TrustedRoots,
    published: Vec<Fp>,
    proofs: Vec<ShardProof>,
}

/// Plans, writes and reads every shard file, proves every shard, and writes
/// and reads every proof file.
fn run() -> Run {
    let system = compile(&parse_policy(POLICY, "policy.pl").unwrap()).unwrap();
    let roots = TrustedRoots::new(&["r"], "roots").unwrap();
    let facts = FactRegistry::from_yaml(FACTS, "facts").unwrap();
    let published = facts.commitments();
    let tree = ProofTree::from_json(TREE, "tree").unwrap();
    let witness = build(&system, &tree, &facts, &roots).unwrap();
    let shards = plan(&system, &witness, SIZES.capacity, || {
        Fp::random(rand_core::OsRng)
    })
    .unwrap();
    assert_eq!(shards.len(), 3);

    let root_key = ShardKey::generate(&system, SIZES, 1).unwrap();
    let other_key = ShardKey::generate(&system, SIZES, 0).unwrap();
    let proofs = shards
        .iter()
        .enumerate()
        .map(|(index, shard)| {
            let shard = Shard::from_json(&shard.to_json().unwrap(), "shard").unwrap();
            let key = if index == 0 { &root_key } else { &other_key };
            let proof =
                prove_shard(&system, SIZES, key, index, &shard, &roots, &published).unwrap();
            ShardProof::from_json(&proof.to_json().unwrap(), "proof").unwrap()
        })
        .collect();
    Run {
        system,
        roots,
        published,
        proofs,
    }
}

impl Run {
    fn verify(&self, proofs: &[ShardProof]) -> Result<(), String> {
        verify_proofs(&self.system, SIZES, &self.roots, &self.published, proofs)
            .map(|_| ())
            .map_err(|error| format!("{error:#}"))
    }
}

#[test]
fn proofs_verify_and_every_tampering_is_rejected() {
    let run = run();
    let verified =
        verify_proofs(&run.system, SIZES, &run.roots, &run.published, &run.proofs).unwrap();
    assert_eq!((verified.shards, verified.links), (3, 2));

    // The root shard is required, once.
    assert_eq!(
        run.verify(&run.proofs[1..]).unwrap_err(),
        "0 proofs claim the trusted roots; exactly one must"
    );
    let twice = [&run.proofs[..], &run.proofs[..1]].concat();
    assert_eq!(
        run.verify(&twice).unwrap_err(),
        "2 proofs claim the trusted roots; exactly one must"
    );
    // A proof missing: its links match nothing.
    assert!(
        run.verify(&run.proofs[..2])
            .unwrap_err()
            .contains("matches no")
    );
    // A changed link commitment, or hidden links: the proof fails.
    let mut forged = run.proofs.clone();
    let link = forged[1]
        .links
        .iter()
        .position(|link| *link != Fp::ZERO)
        .unwrap();
    forged[1].links[link] += Fp::ONE;
    assert_eq!(
        run.verify(&forged).unwrap_err(),
        "shard 1: the proof does not verify: ConstraintSystemFailure"
    );
    let mut hidden = run.proofs.clone();
    hidden[1].links = vec![Fp::ZERO; hidden[1].links.len()];
    assert!(
        run.verify(&hidden)
            .unwrap_err()
            .starts_with("shard 1: the proof does not verify")
    );
    // Another trusted root, or another fact list.
    let other_root = TrustedRoots::new(&["p"], "roots").unwrap();
    assert!(verify_proofs(&run.system, SIZES, &other_root, &run.published, &run.proofs).is_err());
    assert!(verify_proofs(&run.system, SIZES, &run.roots, &[Fp::ONE], &run.proofs).is_err());
    // Other sizes than the prover's.
    let sizes = Sizes {
        outgoing: 2,
        ..SIZES
    };
    assert!(verify_proofs(&run.system, sizes, &run.roots, &run.published, &run.proofs).is_err());
}

#[test]
fn malformed_files_are_rejected() {
    assert!(ShardProof::from_json("{}", "proof").is_err());
    let proof = ShardProof {
        shard: 0,
        root: true,
        links: vec![Fp::from(3)],
        proof: vec![0xab, 0x01],
    };
    let json = proof.to_json().unwrap();
    assert!(json.contains("\"ab01\""));
    assert_eq!(ShardProof::from_json(&json, "proof").unwrap(), proof);
    assert!(ShardProof::from_json(&json.replace("ab01", "ab0"), "proof").is_err());
    assert!(Sizes::from_json(r#"{"capacity": 0, "incoming": 0, "outgoing": 0}"#, "sizes").is_err());
}
