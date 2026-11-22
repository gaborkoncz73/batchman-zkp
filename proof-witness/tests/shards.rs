//! Sharding the repository's proof tree: the planner keeps subtrees together,
//! every honest plan passes, and every kind of tampering with the links is
//! caught where the shard proofs or the verifier will catch it.

use ff::Field;
use policy_constraints::term::symbol;
use policy_constraints::{ConstraintSystem, Fp, compile};
use policy_parser::parse_policy;
use proof_witness::{
    FactRegistry, ProofTree, ProofWitness, Shard, ShardPublic, TrustedRoots, build, check,
    check_links_between, check_sharded, parse_commitments, plan,
};

const POLICY: &str = include_str!("../../prolog/policy.pl");
const TREE: &str = include_str!("../../input/proof_tree.json");
const FACTS: &str = include_str!("../../registry/facts.yaml");
const COMMITMENTS: &str = include_str!("../../registry/commitments.json");
const ROOTS: &str = include_str!("../../input/trusted_roots.json");

struct Energy {
    system: ConstraintSystem,
    witness: ProofWitness,
    roots: TrustedRoots,
    commitments: Vec<Fp>,
}

fn energy() -> Energy {
    let system = compile(&parse_policy(POLICY, "policy.pl").unwrap()).unwrap();
    let roots = TrustedRoots::from_json(ROOTS, "trusted_roots").unwrap();
    let witness = build(
        &system,
        &ProofTree::from_json(TREE, "proof_tree").unwrap(),
        &FactRegistry::from_yaml(FACTS, "facts").unwrap(),
        &roots,
    )
    .unwrap();
    Energy {
        system,
        witness,
        roots,
        commitments: parse_commitments(COMMITMENTS, "commitments").unwrap(),
    }
}

fn challenge() -> Fp {
    symbol("test challenge")
}

/// Deterministic, distinct salts.
fn salts() -> impl FnMut() -> Fp {
    let mut next = 0_u64;
    move || {
        next += 1;
        Fp::from(1000 + next)
    }
}

impl Energy {
    fn plan(&self, capacity: usize) -> Vec<Shard> {
        plan(&self.system, &self.witness, capacity, salts()).unwrap()
    }

    fn check(&self, shards: &[Shard]) -> anyhow::Result<()> {
        check_sharded(
            &self.system,
            shards,
            &self.roots,
            &self.commitments,
            challenge(),
        )
    }

    fn error(&self, shards: &[Shard]) -> String {
        format!("{:#}", self.check(shards).unwrap_err())
    }

    /// The shard and incoming link of the first link of the plan, and the
    /// shard and outgoing link of its other end.
    fn first_link(&self, shards: &[Shard]) -> ((usize, usize), (usize, usize)) {
        let (parent_shard, _) = shards
            .iter()
            .enumerate()
            .find(|(_, shard)| !shard.incoming.is_empty())
            .expect("the plan has a link");
        let commitment = shards[parent_shard].incoming[0].commitment(self.system.layout());
        let child = shards
            .iter()
            .enumerate()
            .find_map(|(shard, candidate)| {
                candidate
                    .outgoing
                    .iter()
                    .position(|link| link.commitment(self.system.layout()) == commitment)
                    .map(|index| (shard, index))
            })
            .expect("both ends");
        ((parent_shard, 0), child)
    }
}

#[test]
fn honest_plans_pass_and_keep_subtrees_together() {
    let energy = energy();
    let nodes = energy.witness.nodes.len();
    for (capacity, expected) in [
        (50, Some((1, 0))),
        (25, Some((2, 1))),
        (13, None),
        (7, None),
        (5, None),
    ] {
        let shards = energy.plan(capacity);
        energy.check(&shards).unwrap();

        let mut seen: Vec<usize> = shards
            .iter()
            .flat_map(|shard| shard.origin.clone())
            .collect();
        seen.sort_unstable();
        assert_eq!(seen, (0..nodes).collect::<Vec<_>>(), "every node once");
        assert!(
            shards
                .iter()
                .all(|shard| shard.witness.nodes.len() <= capacity)
        );
        assert_eq!(shards[0].witness.roots, [0]);
        assert!(
            shards[1..]
                .iter()
                .all(|shard| shard.witness.roots.is_empty())
        );

        let links: usize = shards.iter().map(|shard| shard.incoming.len()).sum();
        let outgoing: usize = shards.iter().map(|shard| shard.outgoing.len()).sum();
        assert_eq!(links, outgoing);
        // Subtrees stay together: about one link per extra shard.
        assert!(
            links < 2 * shards.len(),
            "{capacity}: {links} links, {} shards",
            shards.len()
        );
        if let Some(expected) = expected {
            assert_eq!((shards.len(), links), expected, "capacity {capacity}");
        }
    }
}

#[test]
fn one_shard_is_the_whole_tree() {
    let energy = energy();
    let shards = energy.plan(energy.witness.nodes.len());
    assert_eq!(shards.len(), 1);
    assert!(shards[0].incoming.is_empty() && shards[0].outgoing.is_empty());
    check(
        &energy.system,
        &shards[0].witness,
        &energy.roots,
        &energy.commitments,
        challenge(),
    )
    .unwrap();
}

#[test]
fn the_two_ends_of_a_link_must_use_the_same_salt() {
    let energy = energy();
    let mut shards = energy.plan(13);
    let (_, (shard, index)) = energy.first_link(&shards);
    shards[shard].outgoing[index].salt += Fp::ONE;
    assert_eq!(
        energy.error(&shards),
        format!("shard {shard}: outgoing link {index} matches no incoming link")
    );
}

#[test]
fn an_incoming_link_must_offer_the_goal_its_body_goal_needs() {
    // The incoming link offers another node's (well-formed) goal: the body
    // goal waiting for it is left without a match.
    let energy = energy();
    let mut shards = energy.plan(13);
    let ((shard, index), _) = energy.first_link(&shards);
    let other = energy
        .system
        .layout()
        .goal_of(&energy.witness.nodes[1].assignment);
    assert_ne!(shards[shard].incoming[index].goal, other);
    shards[shard].incoming[index].goal = other;
    let error = energy.error(&shards);
    assert!(
        error.starts_with(&format!("shard {shard}: node "))
            && error.contains("no node proves the body goal in child slot"),
        "{error}"
    );
}

#[test]
fn an_incoming_link_hashes_its_goal() {
    // A forged cell in both the incoming link and the body goal it serves:
    // the matching in the shard holds, but the link's goal is not one whose
    // digest fixes its cells, so its commitment could stand for another goal.
    let energy = energy();
    let mut shards = energy.plan(13);
    let ((shard, index), _) = energy.first_link(&shards);
    let layout = energy.system.layout();
    let goal = shards[shard].incoming[index].goal.clone();
    // The body goal that waits for the link, as (node, slot).
    let (node, slot) = shards[shard]
        .witness
        .nodes
        .iter()
        .enumerate()
        .find_map(|(node, witness)| {
            (0..layout.child_slots())
                .find(|&slot| {
                    layout
                        .slot_links(slot)
                        .iter()
                        .map(|link| witness.assignment[link.in_parent])
                        .eq(goal.iter().copied())
                })
                .map(|slot| (node, slot))
        })
        .expect("a body goal waits for the link");
    // A goal cell the node's own checks do not read: the node stays
    // well-formed (its slots are not hashed) and its branch holds.
    let parent = &shards[shard].witness.nodes[node];
    let branch = &energy.system.branches()[parent.branch];
    let cell = (0..goal.len())
        .find(|&cell| {
            let mut w = parent.assignment.clone();
            w[layout.slot_links(slot)[cell].in_parent] += Fp::ONE;
            layout.check_well_formed(&w).is_ok() && branch.is_satisfied_by(&w)
        })
        .expect("some cell of the body goal is free in the node");
    shards[shard].incoming[index].goal[cell] += Fp::ONE;
    let column = layout.slot_links(slot)[cell].in_parent;
    shards[shard].witness.nodes[node].assignment[column] += Fp::ONE;

    let error = energy.error(&shards);
    assert!(
        error.starts_with(&format!("shard {shard}: incoming link {index}: subterm P")),
        "{error}"
    );
}

#[test]
fn a_link_needs_both_ends() {
    // Without its outgoing end, the child's goal is used by no body goal.
    let energy = energy();
    let mut shards = energy.plan(13);
    let ((parent, incoming), (child, outgoing)) = energy.first_link(&shards);
    let mut cut = shards.clone();
    cut[child].outgoing.remove(outgoing);
    let error = energy.error(&cut);
    assert!(
        error.starts_with(&format!("shard {child}: node "))
            && error.contains("proves a goal that no body goal uses"),
        "{error}"
    );
    // Without either end, the parent's body goal is proved by no node.
    shards[child].outgoing.remove(outgoing);
    shards[parent].incoming.remove(incoming);
    let error = energy.error(&shards);
    assert!(
        error.starts_with(&format!("shard {parent}: node "))
            && error.contains("no node proves the body goal"),
        "{error}"
    );
}

#[test]
fn a_link_commitment_counts_once() {
    // A second incoming copy of a link, published by another shard, has no
    // second outgoing end.
    let publics = [
        ShardPublic {
            incoming: vec![Fp::from(7)],
            outgoing: vec![],
        },
        ShardPublic {
            incoming: vec![Fp::from(7)],
            outgoing: vec![Fp::from(7)],
        },
    ];
    // Either copy is the extra one; the first is named.
    assert_eq!(
        format!("{:#}", check_links_between(&publics).unwrap_err()),
        "shard 0: incoming link 0 matches no outgoing link"
    );
}

#[test]
fn only_the_first_shard_has_roots() {
    let energy = energy();
    let mut shards = energy.plan(13);
    shards[1].witness.roots = vec![0];
    assert_eq!(
        energy.error(&shards),
        "shard 1: only the first shard has roots"
    );
}

#[test]
fn a_shard_file_round_trips() {
    let energy = energy();
    for shard in energy.plan(13) {
        let json = shard.to_json().unwrap();
        assert_eq!(Shard::from_json(&json, "shard").unwrap(), shard);
    }
    assert!(Shard::from_json("{}", "shard").is_err());
}
