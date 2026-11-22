//! Sharding: a proof tree split into parts that are proved independently,
//! each by its own proof, possibly on another machine.
//!
//! An edge of the tree whose parent and child lie in different shards is a
//! *link*. The shard of the parent gets an *incoming* link: it stands in for
//! the child, offering the child's goal to the body goal that needs it. The
//! shard of the child gets an *outgoing* link: it stands in for the parent,
//! using up the child's goal. Within a shard, the links take part in the
//! matching of body goals with goals like nodes and body goals do
//! ([`crate::check`]).
//!
//! Both shards publish the same commitment for a link,
//! [`link_commitment`]`(digest, salt)` of the goal's digest and a secret salt
//! of the link. The incoming side checks the goal's cells as a node checks its
//! own goal ([`Layout::check_goal`]), so the digest fixes every cell; the
//! outgoing side's cells are matched with a node's goal, which that node
//! hashes. The verifier checks that the outgoing commitments of all shards
//! are exactly the incoming ones ([`check_links_between`]).
//!
//! Together, the matchings within the shards and the matching of the
//! commitments give every non-root node exactly one parent, as in one proof,
//! so the nodes reachable from the roots form a tree. All roots lie in the
//! first shard.
//!
//! [`link_commitment`]: policy_constraints::term::link_commitment
//! [`Layout::check_goal`]: policy_constraints::Layout::check_goal

use std::collections::BTreeMap;

use anyhow::{Context, Result, anyhow, bail};
use ff::PrimeField;
use policy_constraints::term::link_commitment;
use policy_constraints::{ConstraintSystem, Fp, Layout};
use serde::{Deserialize, Serialize};

use crate::check::{check_part, check_roots};
use crate::facts::{decimal, field_element};
use crate::goal::ground_goal;
use crate::roots::TrustedRoots;
use crate::witness::{NodeWitness, ProofWitness};

/// One end of a link: the goal's cells (in the order of
/// [`Layout::goal_columns`]) and the link's salt.
///
/// [`Layout::goal_columns`]: policy_constraints::Layout::goal_columns
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub goal: Vec<Fp>,
    pub salt: Fp,
}

impl Link {
    /// The public commitment of the link.
    pub fn commitment(&self, layout: &Layout) -> Fp {
        link_commitment(layout.goal_digest(&self.goal), self.salt)
    }
}

/// What the prover of one shard needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shard {
    /// The shard's nodes, roots first (only the first shard has roots). The
    /// tree shape is not kept: the check does not read it.
    pub witness: ProofWitness,
    /// Goals of body goals here that nodes of other shards prove.
    pub incoming: Vec<Link>,
    /// Goals of nodes here that body goals of other shards use.
    pub outgoing: Vec<Link>,
    /// Bookkeeping: `witness.nodes[i]` is node `origin[i]` of the whole tree.
    pub origin: Vec<usize>,
}

impl Shard {
    /// The commitments the shard's proof publishes.
    pub fn public(&self, layout: &Layout) -> ShardPublic {
        let commit = |links: &[Link]| links.iter().map(|link| link.commitment(layout)).collect();
        ShardPublic {
            incoming: commit(&self.incoming),
            outgoing: commit(&self.outgoing),
        }
    }
}

/// The JSON form of a [`Shard`]: what the machine that proves the shard
/// needs. It holds the shard's private witness, so it stays with the prover.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShardFile {
    schema_version: u32,
    /// Number of roots: the first nodes.
    roots: usize,
    nodes: Vec<NodeFile>,
    incoming: Vec<LinkFile>,
    outgoing: Vec<LinkFile>,
    origin: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NodeFile {
    goal: String,
    branch: usize,
    assignment: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LinkFile {
    goal: Vec<String>,
    salt: String,
}

const SHARD_SCHEMA: u32 = 1;

impl Shard {
    /// The shard as JSON (see [`Shard::from_json`]).
    pub fn to_json(&self) -> Result<String> {
        let cells = |values: &[Fp]| values.iter().map(|value| decimal(*value)).collect();
        let links = |links: &[Link]| {
            links
                .iter()
                .map(|link| LinkFile {
                    goal: cells(&link.goal),
                    salt: decimal(link.salt),
                })
                .collect()
        };
        if self.witness.roots != (0..self.witness.roots.len()).collect::<Vec<_>>() {
            bail!("the roots of a shard are its first nodes");
        }
        let file = ShardFile {
            schema_version: SHARD_SCHEMA,
            roots: self.witness.roots.len(),
            nodes: self
                .witness
                .nodes
                .iter()
                .map(|node| NodeFile {
                    goal: node.text.clone(),
                    branch: node.branch,
                    assignment: cells(&node.assignment),
                })
                .collect(),
            incoming: links(&self.incoming),
            outgoing: links(&self.outgoing),
            origin: self.origin.clone(),
        };
        Ok(serde_json::to_string(&file)?)
    }

    /// Reads a shard written by [`Shard::to_json`].
    pub fn from_json(source: &str, label: &str) -> Result<Self> {
        let file: ShardFile =
            serde_json::from_str(source).with_context(|| format!("{label}: not a shard file"))?;
        if file.schema_version != SHARD_SCHEMA {
            bail!(
                "{label}: schema version {} is not {SHARD_SCHEMA}",
                file.schema_version
            );
        }
        if file.roots > file.nodes.len() || file.origin.len() != file.nodes.len() {
            bail!("{label}: the roots or the origins do not fit the nodes");
        }
        let cells = |texts: &[String], what: &str| -> Result<Vec<Fp>> {
            texts
                .iter()
                .map(|text| {
                    field_element(text).map_err(|message| anyhow!("{label}: {what}: {message}"))
                })
                .collect()
        };
        let links = |links: &[LinkFile], side: &str| -> Result<Vec<Link>> {
            links
                .iter()
                .enumerate()
                .map(|(index, link)| {
                    let what = format!("{side} link {index}");
                    Ok(Link {
                        goal: cells(&link.goal, &what)?,
                        salt: field_element(&link.salt)
                            .map_err(|message| anyhow!("{label}: {what}: {message}"))?,
                    })
                })
                .collect()
        };
        let nodes = file
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                Ok(NodeWitness {
                    text: node.goal.clone(),
                    goal: ground_goal(&node.goal, &format!("{label}: node {index}"))?,
                    parent: None,
                    children: Vec::new(),
                    branch: node.branch,
                    assignment: cells(&node.assignment, &format!("node {index}"))?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            witness: ProofWitness {
                nodes,
                roots: (0..file.roots).collect(),
            },
            incoming: links(&file.incoming, "incoming")?,
            outgoing: links(&file.outgoing, "outgoing")?,
            origin: file.origin,
        })
    }
}

/// The link commitments one shard proof publishes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShardPublic {
    pub incoming: Vec<Fp>,
    pub outgoing: Vec<Fp>,
}

/// Splits `witness` into shards of at most `capacity` nodes each, with a
/// fresh `salt()` per link.
///
/// Whole subtrees are kept together where they fit: going up from the
/// leaves, a node's subtree is cut below it (its largest remaining child
/// subtrees first) until it fits in `capacity`. The pieces are then packed
/// into shards, the pieces of the roots first, then the others from the
/// largest down, each into the first shard with room. An edge is a link when
/// its ends end up in different shards.
pub fn plan(
    system: &ConstraintSystem,
    witness: &ProofWitness,
    capacity: usize,
    mut salt: impl FnMut() -> Fp,
) -> Result<Vec<Shard>> {
    if capacity == 0 {
        bail!("a shard must hold at least one node");
    }
    let nodes = &witness.nodes;
    // Nodes are in pre-order: children come after their parent.
    let mut remaining = vec![1_usize; nodes.len()];
    let mut tops: Vec<usize> = witness.roots.clone();
    for index in (0..nodes.len()).rev() {
        let mut children: Vec<usize> = nodes[index].children.clone();
        children.sort_by_key(|&child| std::cmp::Reverse(remaining[child]));
        let mut size = 1 + children
            .iter()
            .map(|&child| remaining[child])
            .sum::<usize>();
        for child in children {
            if size <= capacity {
                break;
            }
            size -= remaining[child];
            tops.push(child);
        }
        remaining[index] = size;
    }

    // Pack the pieces: the roots' first, into the first shard.
    let mut loads: Vec<usize> = vec![0];
    let mut shard_of_top: BTreeMap<usize, usize> = BTreeMap::new();
    for &root in &witness.roots {
        loads[0] += remaining[root];
        shard_of_top.insert(root, 0);
    }
    if loads[0] > capacity {
        bail!(
            "the roots' pieces have {} nodes, more than a shard of {capacity}",
            loads[0]
        );
    }
    let mut others: Vec<usize> = tops
        .iter()
        .copied()
        .filter(|top| !witness.roots.contains(top))
        .collect();
    others.sort_by_key(|&top| (std::cmp::Reverse(remaining[top]), top));
    for top in others {
        let shard = match loads
            .iter()
            .position(|&load| load + remaining[top] <= capacity)
        {
            Some(shard) => shard,
            None => {
                loads.push(0);
                loads.len() - 1
            }
        };
        loads[shard] += remaining[top];
        shard_of_top.insert(top, shard);
    }
    let mut shard_of = vec![usize::MAX; nodes.len()];
    for index in 0..nodes.len() {
        shard_of[index] = match shard_of_top.get(&index) {
            Some(&shard) => shard,
            None => {
                let parent = nodes[index]
                    .parent
                    .context("a node without a parent is a root")?;
                shard_of[parent]
            }
        };
    }

    let layout = system.layout();
    let mut shards: Vec<Shard> = (0..loads.len())
        .map(|_| Shard {
            witness: ProofWitness {
                nodes: Vec::new(),
                roots: Vec::new(),
            },
            incoming: Vec::new(),
            outgoing: Vec::new(),
            origin: Vec::new(),
        })
        .collect();
    let order = witness
        .roots
        .iter()
        .copied()
        .chain((0..nodes.len()).filter(|index| !witness.roots.contains(index)));
    for index in order {
        let shard = &mut shards[shard_of[index]];
        if witness.roots.contains(&index) {
            shard.witness.roots.push(shard.witness.nodes.len());
        }
        shard.witness.nodes.push(NodeWitness {
            parent: None,
            children: Vec::new(),
            ..nodes[index].clone()
        });
        shard.origin.push(index);
    }
    for (child, node) in nodes.iter().enumerate() {
        let Some(parent) = node.parent else { continue };
        if shard_of[parent] != shard_of[child] {
            let link = Link {
                goal: layout.goal_of(&node.assignment),
                salt: salt(),
            };
            shards[shard_of[parent]].incoming.push(link.clone());
            shards[shard_of[child]].outgoing.push(link);
        }
    }
    Ok(shards)
}

/// Checks one shard as its proof will: every node, the matching of body
/// goals with goals including the links, and, for the first shard (`trusted`
/// given), the roots. The other shards have no roots. Returns the shard's
/// public commitments.
pub fn check_shard(
    system: &ConstraintSystem,
    shard: &Shard,
    trusted: Option<&TrustedRoots>,
    commitments: &[Fp],
    challenge: Fp,
) -> Result<ShardPublic> {
    if trusted.is_none() && !shard.witness.roots.is_empty() {
        bail!("only the first shard has roots");
    }
    let goals =
        |links: &[Link]| -> Vec<Vec<Fp>> { links.iter().map(|link| link.goal.clone()).collect() };
    check_part(
        system,
        &shard.witness,
        &goals(&shard.incoming),
        &goals(&shard.outgoing),
        commitments,
        challenge,
    )?;
    if let Some(trusted) = trusted {
        check_roots(system, &shard.witness, trusted)?;
    }
    Ok(shard.public(system.layout()))
}

/// The verifier's check across the shards: every outgoing commitment is one
/// incoming commitment and the other way around, as multisets.
pub fn check_links_between(publics: &[ShardPublic]) -> Result<()> {
    let key = |value: &Fp| value.to_repr();
    let mut unused: BTreeMap<_, usize> = BTreeMap::new();
    for public in publics {
        for commitment in &public.incoming {
            *unused.entry(key(commitment)).or_default() += 1;
        }
    }
    for (shard, public) in publics.iter().enumerate() {
        for (index, commitment) in public.outgoing.iter().enumerate() {
            match unused.get_mut(&key(commitment)) {
                Some(count) if *count > 0 => *count -= 1,
                _ => bail!("shard {shard}: outgoing link {index} matches no incoming link"),
            }
        }
    }
    for (shard, public) in publics.iter().enumerate() {
        for (index, commitment) in public.incoming.iter().enumerate() {
            if unused[&key(commitment)] > 0 {
                bail!("shard {shard}: incoming link {index} matches no outgoing link");
            }
        }
    }
    Ok(())
}

/// Checks every shard (the first one with the roots) and the links between
/// them.
pub fn check_sharded(
    system: &ConstraintSystem,
    shards: &[Shard],
    trusted: &TrustedRoots,
    commitments: &[Fp],
    challenge: Fp,
) -> Result<()> {
    if shards.is_empty() {
        bail!("there are no shards");
    }
    let publics = shards
        .iter()
        .enumerate()
        .map(|(index, shard)| {
            let roots = (index == 0).then_some(trusted);
            check_shard(system, shard, roots, commitments, challenge)
                .with_context(|| format!("shard {index}"))
        })
        .collect::<Result<Vec<_>>>()?;
    check_links_between(&publics)
}
