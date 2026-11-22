//! Shard proofs (5b): every shard of a plan is a tree circuit with link
//! places; the verifier matches the shards' link commitments. Each cheat is
//! caught where proof-witness' reference says: inside a shard proof, or by
//! the verifier across the shards.

use ff::Field;
use halo2_poseidon::poseidon::primitives::P128Pow5T3;
use halo2_proofs::dev::MockProver;
use halo2_proofs::halo2curves::pasta::EqAffine;
use halo2_proofs::plonk::{create_proof, keygen_pk, keygen_vk, verify_proof};
use halo2_proofs::poly::VerificationStrategy;
use halo2_proofs::poly::commitment::ParamsProver;
use halo2_proofs::poly::ipa::commitment::{IPACommitmentScheme, ParamsIPA};
use halo2_proofs::poly::ipa::multiopen::{ProverIPA, VerifierIPA};
use halo2_proofs::poly::ipa::strategy::SingleStrategy;
use halo2_proofs::transcript::{
    Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
};
use node_circuit::{TreeCircuit, link_commitments, minimum_k, root_digests, shard_public};
use policy_constraints::term::symbol;
use policy_constraints::{Cell, ConstraintSystem, Fp, Position, Slot, compile};
use policy_parser::parse_policy;
use proof_witness::{
    FactRegistry, ProofTree, Shard, TrustedRoots, build, check_links_between, check_sharded,
    parse_commitments, plan,
};
use rand_core::OsRng;

/// A chain `r → p → q(1)`: with one node per shard, three shards and two
/// links.
const CHAIN: &str = "r :- p.\np :- q(1).\n";
const CHAIN_TREE: &str = r#"[{"goal": "r", "subtree": [{"goal": "p", "subtree": [{"goal": "q(1)", "subtree": [true]}]}]}]"#;
const CHAIN_FACTS: &str = "- goal: \"q(1)\"\n  salt: \"5\"\n";

const SHUFFLE: &str = "link: the body goals are the nodes' goals";

/// A planned tree and the sizes of its shard circuits.
struct Planned {
    system: ConstraintSystem,
    shards: Vec<Shard>,
    roots: TrustedRoots,
    published: Vec<Fp>,
    capacity: usize,
    incoming: usize,
    outgoing: usize,
}

fn planned(
    policy: &str,
    tree: &str,
    facts: &FactRegistry,
    published: Vec<Fp>,
    capacity: usize,
) -> Planned {
    let system = compile(&parse_policy(policy, "policy.pl").unwrap()).unwrap();
    let tree = ProofTree::from_json(tree, "tree").unwrap();
    let roots = TrustedRoots::new(&[tree.nodes()[0].text.as_str()], "roots").unwrap();
    let witness = build(&system, &tree, facts, &roots).unwrap();
    let mut next = 0_u64;
    let shards = plan(&system, &witness, capacity, || {
        next += 1;
        Fp::from(100 + next)
    })
    .unwrap();
    // One circuit shape for every shard: the largest link counts.
    let incoming = shards
        .iter()
        .map(|shard| shard.incoming.len())
        .max()
        .unwrap();
    let outgoing = shards
        .iter()
        .map(|shard| shard.outgoing.len())
        .max()
        .unwrap();
    Planned {
        system,
        shards,
        roots,
        published,
        capacity,
        incoming,
        outgoing,
    }
}

fn chain() -> Planned {
    let facts = FactRegistry::from_yaml(CHAIN_FACTS, "facts").unwrap();
    let published = facts.commitments();
    planned(CHAIN, CHAIN_TREE, &facts, published, 1)
}

impl Planned {
    fn reference(&self) -> Result<(), String> {
        check_sharded(
            &self.system,
            &self.shards,
            &self.roots,
            &self.published,
            symbol("challenge"),
        )
        .map_err(|error| format!("{error:#}"))
    }

    fn circuit(&self, index: usize) -> TreeCircuit<Fp, P128Pow5T3> {
        let trusted = (index == 0).then_some(&self.roots);
        TreeCircuit::from_shard(
            &self.system,
            &self.shards[index],
            trusted,
            self.capacity,
            self.incoming,
            self.outgoing,
        )
        .unwrap()
    }

    /// The shard's public input; the trusted roots only for the first.
    fn instances(&self, index: usize) -> Vec<Vec<Fp>> {
        let roots = if index == 0 {
            root_digests(&self.roots)
        } else {
            Vec::new()
        };
        let links = link_commitments(
            self.system.layout(),
            &self.shards[index],
            self.incoming,
            self.outgoing,
        );
        vec![self.published.clone(), roots, links]
    }

    /// MockProver's failures for shard `index` against `instances`.
    fn mock(&self, index: usize, instances: Vec<Vec<Fp>>) -> Vec<String> {
        let circuit = self.circuit(index);
        match MockProver::run(minimum_k(&circuit), &circuit, instances)
            .unwrap()
            .verify()
        {
            Ok(()) => Vec::new(),
            Err(failures) => failures.iter().map(ToString::to_string).collect(),
        }
    }

    /// The verifier's matching of the published link commitments.
    fn across(&self) -> Result<(), String> {
        let publics: Vec<_> = (0..self.shards.len())
            .map(|index| shard_public(&self.instances(index)[2], self.incoming))
            .collect();
        check_links_between(&publics).map_err(|error| format!("{error:#}"))
    }

    /// The digest column of the body goal in slot 0 of the first node.
    fn slot_digest(&self) -> usize {
        self.system
            .layout()
            .column(&Cell::Digest(Position::root(Slot::Child(0))))
            .unwrap()
    }

    fn goal_digest_index(&self) -> usize {
        let layout = self.system.layout();
        let digest = layout
            .column(&Cell::Digest(Position::root(Slot::Parent)))
            .unwrap();
        layout
            .goal_columns()
            .iter()
            .position(|&column| column == digest)
            .unwrap()
    }
}

fn any(failures: &[String], expected: &str) {
    assert!(
        failures.iter().any(|failure| failure.contains(expected)),
        "no {expected:?} in {failures:#?}"
    );
}

#[test]
fn honest_shards_pass_and_their_links_match() {
    let chain = chain();
    assert_eq!(chain.shards.len(), 3);
    chain.reference().unwrap();
    for index in 0..3 {
        assert_eq!(
            chain.mock(index, chain.instances(index)),
            Vec::<String>::new()
        );
    }
    chain.across().unwrap();
}

#[test]
fn a_wrong_salt_passes_its_shard_but_not_the_verifier() {
    // The child's shard publishes its link under another salt: its proof is
    // valid, and only the matching across the shards fails.
    let mut chain = chain();
    chain.shards[2].outgoing[0].salt += Fp::ONE;
    let reference = chain.reference().unwrap_err();
    assert_eq!(
        reference,
        "shard 2: outgoing link 0 matches no incoming link"
    );
    assert_eq!(chain.mock(2, chain.instances(2)), Vec::<String>::new());
    assert_eq!(chain.across().unwrap_err(), reference);
}

#[test]
fn an_unused_link_place_publishes_zero() {
    let mut chain = chain();
    let honest = chain.instances(0);
    chain.shards[0].incoming.clear();
    // The place is inactive now: it publishes 0, not the commitment …
    any(
        &chain.mock(0, honest),
        "not satisfied by cell (Column('Instance', 2 - ), outside any region, on row 0)",
    );
    // … and r's body goal is proved by nothing.
    any(&chain.mock(0, chain.instances(0)), SHUFFLE);
    assert!(
        chain
            .reference()
            .unwrap_err()
            .contains("shard 0: node 0: no node proves the body goal in child slot 0")
    );
}

#[test]
fn an_incoming_goal_must_hash_to_its_digest() {
    // r's body goal p and the incoming link both carry a forged digest: they
    // match each other, but the link's cells do not hash to it.
    let mut chain = chain();
    let digest = chain.goal_digest_index();
    let column = chain.slot_digest();
    chain.shards[0].incoming[0].goal[digest] += Fp::ONE;
    chain.shards[0].witness.nodes[0].assignment[column] += Fp::ONE;
    assert_eq!(
        chain.reference().unwrap_err(),
        "shard 0: incoming link 0: subterm P: the digest is not the hash of the other cells"
    );
    any(
        &chain.mock(0, chain.instances(0)),
        "parent-side digest is the hash",
    );
}

#[test]
fn an_outgoing_goal_must_be_a_node_goal() {
    let mut chain = chain();
    let digest = chain.goal_digest_index();
    chain.shards[1].outgoing[0].goal[digest] += Fp::ONE;
    assert_eq!(
        chain.reference().unwrap_err(),
        "shard 1: outgoing link 0: no node proves its goal"
    );
    any(&chain.mock(1, chain.instances(1)), SHUFFLE);
}

#[test]
fn the_energy_tree_in_two_shards() {
    const POLICY: &str = include_str!("../../prolog/policy.pl");
    const TREE: &str = include_str!("../../input/proof_tree.json");
    const FACTS: &str = include_str!("../../registry/facts.yaml");
    const COMMITMENTS: &str = include_str!("../../registry/commitments.json");
    let facts = FactRegistry::from_yaml(FACTS, "facts.yaml").unwrap();
    let published = parse_commitments(COMMITMENTS, "commitments.json").unwrap();
    let energy = planned(POLICY, TREE, &facts, published, 25);
    assert_eq!(energy.shards.len(), 2);
    assert_eq!((energy.incoming, energy.outgoing), (1, 1));
    energy.reference().unwrap();
    for index in 0..2 {
        assert_eq!(
            energy.mock(index, energy.instances(index)),
            Vec::<String>::new()
        );
    }
    energy.across().unwrap();
}

#[test]
fn shards_are_proved_and_verified_with_the_real_prover() {
    // Each shard on its own; the first has its own key (it has the roots).
    let chain = chain();
    for index in 0..chain.shards.len() {
        let circuit = chain.circuit(index);
        let instances = chain.instances(index);
        let columns: Vec<&[Fp]> = instances.iter().map(Vec::as_slice).collect();
        let params: ParamsIPA<EqAffine> = ParamsIPA::new(minimum_k(&circuit));
        let vk = keygen_vk(&params, &circuit).unwrap();
        let pk = keygen_pk(&params, vk, &circuit).unwrap();
        let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
        create_proof::<IPACommitmentScheme<EqAffine>, ProverIPA<EqAffine>, _, _, _, _>(
            &params,
            &pk,
            &[circuit],
            &[&columns],
            OsRng,
            &mut transcript,
        )
        .unwrap();
        let proof = transcript.finalize();
        let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof[..]);
        verify_proof::<IPACommitmentScheme<EqAffine>, VerifierIPA<EqAffine>, _, _, _>(
            &params,
            pk.get_vk(),
            SingleStrategy::new(&params),
            &[&columns],
            &mut transcript,
        )
        .unwrap();
    }
    chain.across().unwrap();
}
