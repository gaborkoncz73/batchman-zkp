//! Circuits over a list of nodes: one per chip (3a, 3b), for testing and
//! measuring each check on its own, [`NodeCircuit`] with every per-node check
//! (3c, 4), and [`TreeCircuit`], which also links the nodes into a tree whose
//! roots are the trusted goals (5a).

use std::marker::PhantomData;

use ff::{Field, PrimeField};
use halo2_poseidon::poseidon::primitives::Spec;
use halo2_proofs::circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::plonk::{Circuit, Column, ConstraintSystem, Error, Instance};
use policy_constraints::{Cell, ConstraintSystem as Policy, Fp, Layout, Position, Slot};
use proof_witness::{Link, ProofWitness, Shard, ShardPublic, TrustedRoots};

use crate::arith::{ArithChip, ArithConfig};
use crate::boundary::{BoundaryChip, BoundaryConfig, Gate};
use crate::branch::{BranchChip, BranchConfig, BranchTable, Claim, NodeBranch};
use crate::fact::{FactChip, FactConfig};
use crate::link::{LinkChip, LinkConfig, Role, Side};
use crate::well_formed::{Shaped, WellFormedChip, WellFormedConfig};

/// The step 3a circuit: the well-formedness of a list of node assignments,
/// nothing else (no branch check, no links between the nodes).
#[derive(Debug)]
pub struct WellFormedCircuit<F: PrimeField, S> {
    layout: Layout,
    nodes: Vec<Value<Vec<F>>>,
    _spec: PhantomData<S>,
}

impl<F: PrimeField, S: Spec<F, 3, 2>> WellFormedCircuit<F, S> {
    /// One assignment per node, each as wide as `layout`.
    pub fn new(layout: Layout, nodes: Vec<Vec<F>>) -> Self {
        assert!(
            nodes.iter().all(|w| w.len() == layout.width()),
            "one value per cell"
        );
        Self {
            layout,
            nodes: nodes.into_iter().map(Value::known).collect(),
            _spec: PhantomData,
        }
    }
}

impl<F: PrimeField, S: Spec<F, 3, 2>> Circuit<F> for WellFormedCircuit<F, S> {
    type Config = WellFormedConfig<F>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            layout: self.layout.clone(),
            nodes: vec![Value::unknown(); self.nodes.len()],
            _spec: PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let w = meta.advice_column();
        WellFormedConfig::configure::<S>(meta, w)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = WellFormedChip::<F, S>::construct(config);
        chip.load_table(&mut layouter)?;
        for (index, node) in self.nodes.iter().enumerate() {
            let w: Vec<Value<F>> = (0..self.layout.width())
                .map(|cell| node.as_ref().map(|w| w[cell]))
                .collect();
            chip.assign_node(
                layouter.namespace(|| format!("node {index}")),
                &self.layout,
                &w,
            )?;
        }
        Ok(())
    }
}

/// The step 3b circuit: the branch check of a list of nodes, nothing else
/// (no well-formedness, no links between the nodes).
#[derive(Debug)]
pub struct BranchCircuit<F: PrimeField> {
    table: BranchTable<F>,
    nodes: Vec<Value<NodeBranch<F>>>,
}

impl<F: PrimeField> BranchCircuit<F> {
    pub fn new(table: BranchTable<F>, nodes: Vec<NodeBranch<F>>) -> Self {
        assert!(
            nodes
                .iter()
                .all(|node| node.assignment.len() == table.width()),
            "one value per cell"
        );
        Self {
            table,
            nodes: nodes.into_iter().map(Value::known).collect(),
        }
    }
}

impl<F: PrimeField> Circuit<F> for BranchCircuit<F> {
    type Config = BranchConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            table: self.table.clone(),
            nodes: vec![Value::unknown(); self.nodes.len()],
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        BranchConfig::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = BranchChip::construct(config);
        chip.load_table(&mut layouter, &self.table)?;
        for (index, node) in self.nodes.iter().enumerate() {
            chip.assign_node(
                layouter.namespace(|| format!("node {index}")),
                &self.table,
                node.as_ref(),
            )?;
        }
        Ok(())
    }
}

/// Every per-node check of `proof_witness::check` on a list of nodes:
/// well-formedness (3a), the secret branch (3b), the arithmetic of the step
/// slots (4a) and the published fact commitments (4b), with the chips' copies
/// of each `w` linked cell by cell. The public input is one instance column:
/// the fact registry's commitments. The nodes are not linked to each other;
/// [`TreeCircuit`] links them and checks the roots.
#[derive(Debug)]
pub struct NodeCircuit<F: PrimeField, S> {
    layout: Layout,
    table: BranchTable<F>,
    /// Per node: the assignment the well-formedness chip sees, and the
    /// assignment and claim the branch chip sees. Honest provers give both
    /// chips the same `w`.
    nodes: Vec<Value<(Vec<F>, NodeBranch<F>)>>,
    _spec: PhantomData<S>,
}

impl<F: PrimeField, S: Spec<F, 3, 2>> NodeCircuit<F, S> {
    /// The honest circuit: each chip sees the node's own assignment.
    pub fn new(layout: Layout, table: BranchTable<F>, nodes: Vec<NodeBranch<F>>) -> Self {
        let well_formed = nodes.iter().map(|node| node.assignment.clone()).collect();
        Self::with_assignments(layout, table, well_formed, nodes)
    }

    /// Lets the two chips see different assignments, as a cheating prover
    /// may; the links between the copies must catch it.
    pub fn with_assignments(
        layout: Layout,
        table: BranchTable<F>,
        well_formed: Vec<Vec<F>>,
        branch: Vec<NodeBranch<F>>,
    ) -> Self {
        assert_eq!(well_formed.len(), branch.len(), "one entry per node");
        assert_eq!(layout.width(), table.width(), "the table is the layout's");
        assert!(
            well_formed.iter().all(|w| w.len() == layout.width())
                && branch
                    .iter()
                    .all(|node| node.assignment.len() == layout.width()),
            "one value per cell"
        );
        Self {
            layout,
            table,
            nodes: well_formed
                .into_iter()
                .zip(branch)
                .map(Value::known)
                .collect(),
            _spec: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeConfig<F: PrimeField> {
    well_formed: WellFormedConfig<F>,
    branch: BranchConfig,
    arith: ArithConfig,
    fact: FactConfig<F>,
}

impl<F: PrimeField> NodeConfig<F> {
    /// Configures every per-node chip; the fact registry's commitments are
    /// the instance column `commitments`.
    fn configure<S: Spec<F, 3, 2>>(
        meta: &mut ConstraintSystem<F>,
        commitments: Column<Instance>,
    ) -> Self {
        let w = meta.advice_column();
        let well_formed = WellFormedConfig::configure::<S>(meta, w);
        let fact = FactConfig::configure(meta, well_formed.poseidon().clone(), commitments);
        Self {
            well_formed,
            branch: BranchConfig::configure(meta),
            arith: ArithConfig::configure(meta),
            fact,
        }
    }
}

/// The per-node chips, constructed once per synthesis.
struct NodeChips<F: PrimeField, S: Spec<F, 3, 2>> {
    well_formed: WellFormedChip<F, S>,
    branch: BranchChip,
    arith: ArithChip,
    fact: FactChip<F, S>,
}

impl<F: PrimeField, S: Spec<F, 3, 2>> NodeChips<F, S> {
    /// Constructs the chips and loads their tables.
    fn load(
        config: NodeConfig<F>,
        layouter: &mut impl Layouter<F>,
        table: &BranchTable<F>,
    ) -> Result<Self, Error> {
        let chips = Self {
            well_formed: WellFormedChip::construct(config.well_formed),
            branch: BranchChip::construct(config.branch),
            arith: ArithChip::construct(config.arith),
            fact: FactChip::construct(config.fact),
        };
        chips.well_formed.load_table(layouter)?;
        chips.branch.load_table(layouter, table)?;
        chips.arith.load_table(layouter)?;
        Ok(chips)
    }

    /// Every per-node check of node `index`: `w` as the well-formedness chip
    /// sees it, `node` as the branch chip sees it, the two linked cell by
    /// cell.
    fn assign_node(
        &self,
        layouter: &mut impl Layouter<F>,
        index: usize,
        layout: &Layout,
        table: &BranchTable<F>,
        w: &[Value<F>],
        node: Value<&NodeBranch<F>>,
    ) -> Result<Shaped<F>, Error> {
        let shaped = self.well_formed.assign_node(
            layouter.namespace(|| format!("node {index}: well-formedness")),
            layout,
            w,
        )?;
        self.arith.assign_node(
            layouter.namespace(|| format!("node {index}: arithmetic")),
            layout,
            &shaped.w,
        )?;
        self.fact.assign_node(
            layouter.namespace(|| format!("node {index}: fact")),
            layout,
            &shaped.w,
        )?;
        let checked = self.branch.assign_node(
            layouter.namespace(|| format!("node {index}: branch")),
            table,
            node,
        )?;
        layouter.assign_region(
            || format!("node {index}: one w"),
            |mut region| {
                for (left, right) in shaped.w.iter().zip(&checked) {
                    region.constrain_equal(left.cell(), right.cell())?;
                }
                Ok(())
            },
        )?;
        Ok(shaped)
    }
}

impl<F: PrimeField, S: Spec<F, 3, 2>> Circuit<F> for NodeCircuit<F, S> {
    type Config = NodeConfig<F>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            layout: self.layout.clone(),
            table: self.table.clone(),
            nodes: vec![Value::unknown(); self.nodes.len()],
            _spec: PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let commitments = meta.instance_column();
        NodeConfig::configure::<S>(meta, commitments)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chips = NodeChips::<F, S>::load(config, &mut layouter, &self.table)?;
        for (index, node) in self.nodes.iter().enumerate() {
            let w: Vec<Value<F>> = (0..self.layout.width())
                .map(|cell| node.as_ref().map(|(w, _)| w[cell]))
                .collect();
            chips.assign_node(
                &mut layouter,
                index,
                &self.layout,
                &self.table,
                &w,
                node.as_ref().map(|(_, node)| node),
            )?;
        }
        Ok(())
    }
}

/// One node of a [`TreeCircuit`]: its assignment and branch claim, and
/// whether it is part of the tree (inactive nodes are padding).
#[derive(Debug, Clone)]
pub struct TreeNode<F> {
    pub branch: NodeBranch<F>,
    pub active: bool,
}

/// A proof tree, or one shard of it, in one proof: every per-node check of
/// [`NodeCircuit`], the links between the nodes ([`crate::link`]), and the
/// links to other shards ([`crate::boundary`]).
///
/// The circuit has fixed numbers of node places (its capacity), roots,
/// incoming and outgoing link places, all public, as the verifying key
/// depends on them. The first `roots` places hold the root nodes: each is
/// active, proved by a clause (`is_clause = 1`), and its goal digest is the
/// matching entry of the trusted roots. The other places hold the other
/// nodes in any order, then padding; unused link places are inactive. A whole
/// tree in one proof (`K = 1`) has no link places; of a sharded tree, the
/// first shard has the roots and the others none.
///
/// Public input, by instance column: the fact registry's commitments, the
/// digests of the trusted roots, and the link commitments (the incoming
/// places', then the outgoing places'; 0 for an unused place).
#[derive(Debug)]
pub struct TreeCircuit<F: PrimeField, S> {
    layout: Layout,
    table: BranchTable<F>,
    roots: usize,
    nodes: Vec<Value<TreeNode<F>>>,
    incoming: Vec<Value<Gate<F>>>,
    outgoing: Vec<Value<Gate<F>>>,
    _spec: PhantomData<S>,
}

#[derive(Debug, Clone)]
pub struct TreeConfig<F: PrimeField> {
    node: NodeConfig<F>,
    link: LinkConfig,
    boundary: BoundaryConfig<F>,
    roots: Column<Instance>,
    links: Column<Instance>,
}

impl<F: PrimeField, S: Spec<F, 3, 2>> TreeCircuit<F, S> {
    /// The first `roots` of `nodes` are the roots, in the order of the
    /// trusted roots. No link places (see [`Self::with_links`]).
    pub fn new(
        layout: Layout,
        table: BranchTable<F>,
        roots: usize,
        nodes: Vec<TreeNode<F>>,
    ) -> Self {
        assert_eq!(layout.width(), table.width(), "the table is the layout's");
        assert!(roots <= nodes.len(), "every root is a node");
        assert!(
            nodes
                .iter()
                .all(|node| node.branch.assignment.len() == layout.width()),
            "one value per cell"
        );
        assert!(
            nodes[..roots].iter().all(|node| node.active),
            "roots are active"
        );
        Self {
            layout,
            table,
            roots,
            nodes: nodes.into_iter().map(Value::known).collect(),
            incoming: Vec::new(),
            outgoing: Vec::new(),
            _spec: PhantomData,
        }
    }

    /// The circuit's shape without a witness: `capacity` node places, the
    /// first `roots` of them roots, and the link places. Keys are generated
    /// from it.
    pub fn shape(
        layout: Layout,
        table: BranchTable<F>,
        roots: usize,
        capacity: usize,
        incoming: usize,
        outgoing: usize,
    ) -> Self {
        assert_eq!(layout.width(), table.width(), "the table is the layout's");
        assert!(roots <= capacity, "every root is a node");
        Self {
            layout,
            table,
            roots,
            nodes: vec![Value::unknown(); capacity],
            incoming: vec![Value::unknown(); incoming],
            outgoing: vec![Value::unknown(); outgoing],
            _spec: PhantomData,
        }
    }

    /// The same circuit with link places to other shards.
    pub fn with_links(mut self, incoming: Vec<Gate<F>>, outgoing: Vec<Gate<F>>) -> Self {
        let goal = self.layout.goal_columns().len();
        assert!(
            incoming
                .iter()
                .chain(&outgoing)
                .all(|gate| gate.goal.len() == goal),
            "one value per goal cell"
        );
        self.incoming = incoming.into_iter().map(Value::known).collect();
        self.outgoing = outgoing.into_iter().map(Value::known).collect();
        self
    }

    /// Number of node places.
    pub fn capacity(&self) -> usize {
        self.nodes.len()
    }
}

impl<S: Spec<Fp, 3, 2>> TreeCircuit<Fp, S> {
    /// Lays out an honest witness of a whole tree in `capacity` node places:
    /// the roots first, in the order of `trusted`, then the other nodes, then
    /// inactive copies of the last node.
    pub fn from_witness(
        system: &Policy,
        witness: &ProofWitness,
        trusted: &TrustedRoots,
        capacity: usize,
    ) -> Result<Self, String> {
        let nodes = place_nodes(system, witness, Some(trusted), capacity)?;
        Ok(Self::new(
            system.layout().clone(),
            BranchTable::from_policy(system),
            witness.roots.len(),
            nodes,
        ))
    }

    /// Lays out one shard of a plan: its nodes as [`Self::from_witness`] does
    /// (`trusted` is given for the first shard, which has the roots), and its
    /// links in `incoming` and `outgoing` places, the unused ones inactive.
    pub fn from_shard(
        system: &Policy,
        shard: &Shard,
        trusted: Option<&TrustedRoots>,
        capacity: usize,
        incoming: usize,
        outgoing: usize,
    ) -> Result<Self, String> {
        let nodes = place_nodes(system, &shard.witness, trusted, capacity)?;
        let padding = system.layout().goal_of(&shard.witness.nodes[0].assignment);
        let gates = |links: &[Link], places: usize, side: &str| -> Result<Vec<Gate<Fp>>, String> {
            if links.len() > places {
                return Err(format!(
                    "the shard has {} {side} links, the circuit has {places} places",
                    links.len()
                ));
            }
            let mut gates: Vec<Gate<Fp>> = links
                .iter()
                .map(|link| Gate {
                    goal: link.goal.clone(),
                    salt: link.salt,
                    active: true,
                })
                .collect();
            gates.resize_with(places, || Gate {
                goal: padding.clone(),
                salt: Fp::ZERO,
                active: false,
            });
            Ok(gates)
        };
        let incoming = gates(&shard.incoming, incoming, "incoming")?;
        let outgoing = gates(&shard.outgoing, outgoing, "outgoing")?;
        Ok(Self::new(
            system.layout().clone(),
            BranchTable::from_policy(system),
            shard.witness.roots.len(),
            nodes,
        )
        .with_links(incoming, outgoing))
    }
}

/// The nodes of `witness` in `capacity` places: the roots first, in the
/// order of `trusted` (no roots without it), then the others, then inactive
/// copies of the last node.
fn place_nodes(
    system: &Policy,
    witness: &ProofWitness,
    trusted: Option<&TrustedRoots>,
    capacity: usize,
) -> Result<Vec<TreeNode<Fp>>, String> {
    let digest = system
        .layout()
        .column(&Cell::Digest(Position::root(Slot::Parent)))
        .expect("the layout opens the parent goal");
    let node = |index: usize, active: bool| TreeNode {
        branch: NodeBranch {
            assignment: witness.nodes[index].assignment.clone(),
            claim: Claim::Branch(witness.nodes[index].branch),
        },
        active,
    };
    let mut nodes = Vec::with_capacity(capacity);
    for goal in trusted.into_iter().flat_map(TrustedRoots::goals) {
        let root = witness
            .roots
            .iter()
            .copied()
            .find(|&root| witness.nodes[root].assignment[digest] == goal.digest())
            .ok_or("a trusted root is not proved by any root node")?;
        nodes.push(node(root, true));
    }
    if nodes.len() != witness.roots.len() {
        return Err("the witness has roots that are not trusted".to_owned());
    }
    for index in (0..witness.nodes.len()).filter(|index| !witness.roots.contains(index)) {
        nodes.push(node(index, true));
    }
    if nodes.len() > capacity {
        return Err(format!(
            "the tree has {} nodes, the circuit has {capacity} places",
            nodes.len()
        ));
    }
    let last = witness
        .nodes
        .len()
        .checked_sub(1)
        .ok_or("the witness has no nodes")?;
    nodes.resize_with(capacity, || node(last, false));
    Ok(nodes)
}

/// The digests of the trusted roots: the second instance column of a
/// [`TreeCircuit`].
pub fn root_digests(trusted: &TrustedRoots) -> Vec<Fp> {
    trusted.goals().map(|goal| goal.digest()).collect()
}

/// The link commitments of a shard: the third instance column of its
/// [`TreeCircuit`] with `incoming` and `outgoing` places.
pub fn link_commitments(
    layout: &Layout,
    shard: &Shard,
    incoming: usize,
    outgoing: usize,
) -> Vec<Fp> {
    let public = shard.public(layout);
    let mut column = public.incoming;
    column.resize(incoming, Fp::ZERO);
    column.extend(public.outgoing);
    column.resize(incoming + outgoing, Fp::ZERO);
    column
}

/// What the verifier reads from the third instance column of a shard proof
/// with `incoming` incoming places: the commitments of the used places, for
/// [`proof_witness::check_links_between`].
pub fn shard_public(links: &[Fp], incoming: usize) -> ShardPublic {
    let used = |values: &[Fp]| {
        values
            .iter()
            .copied()
            .filter(|value| *value != Fp::ZERO)
            .collect()
    };
    let incoming = incoming.min(links.len());
    ShardPublic {
        incoming: used(&links[..incoming]),
        outgoing: used(&links[incoming..]),
    }
}

impl<F: PrimeField, S: Spec<F, 3, 2>> Circuit<F> for TreeCircuit<F, S> {
    type Config = TreeConfig<F>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            layout: self.layout.clone(),
            table: self.table.clone(),
            roots: self.roots,
            nodes: vec![Value::unknown(); self.nodes.len()],
            incoming: vec![Value::unknown(); self.incoming.len()],
            outgoing: vec![Value::unknown(); self.outgoing.len()],
            _spec: PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let commitments = meta.instance_column();
        let roots = meta.instance_column();
        let links = meta.instance_column();
        meta.enable_equality(roots);
        meta.enable_equality(links);
        let node = NodeConfig::configure::<S>(meta, commitments);
        let link = LinkConfig::configure(meta, node.well_formed.w());
        let boundary = BoundaryConfig::configure(meta, node.well_formed.poseidon().clone());
        TreeConfig {
            node,
            link,
            boundary,
            roots,
            links,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let w_column = config.node.well_formed.w();
        let chips = NodeChips::<F, S>::load(config.node, &mut layouter, &self.table)?;
        let link = LinkChip::construct(config.link);
        let boundary = BoundaryChip::<F, S>::construct(config.boundary);
        let layout = &self.layout;
        let goal_digest = layout
            .column(&Cell::Digest(Position::root(Slot::Parent)))
            .expect("the layout opens the parent goal");
        let is_clause = layout.is_clause();
        for (index, node) in self.nodes.iter().enumerate() {
            let w: Vec<Value<F>> = (0..layout.width())
                .map(|cell| node.as_ref().map(|node| node.branch.assignment[cell]))
                .collect();
            let shaped = chips.assign_node(
                &mut layouter,
                index,
                layout,
                &self.table,
                &w,
                node.as_ref().map(|node| &node.branch),
            )?;
            let role = if index < self.roots {
                Role::Root
            } else {
                Role::Other
            };
            link.assign_node(
                layouter.namespace(|| format!("node {index}: links")),
                layout,
                &shaped,
                &w,
                role,
                node.as_ref().map(|node| node.active),
            )?;
            if role == Role::Root {
                layouter.constrain_instance(shaped.w[goal_digest].cell(), config.roots, index)?;
                layouter.assign_region(
                    || format!("node {index}: a root is proved by a clause"),
                    |mut region| region.constrain_constant(shaped.w[is_clause].cell(), F::ONE),
                )?;
            }
        }

        // The goal's digest among the goal cells.
        let digest_index = layout
            .goal_columns()
            .iter()
            .position(|&column| column == goal_digest)
            .expect("the goal's digest is one of its cells");
        let places = self
            .incoming
            .iter()
            .map(|gate| (Side::Export, gate))
            .chain(self.outgoing.iter().map(|gate| (Side::Import, gate)));
        for (place, (side, gate)) in places.enumerate() {
            let values: Vec<Value<F>> = (0..layout.goal_columns().len())
                .map(|cell| gate.as_ref().map(|gate| gate.goal[cell]))
                .collect();
            let goal: Vec<AssignedCell<F, F>> = match side {
                // An incoming goal is checked as a parent goal.
                Side::Export => {
                    let mut w = vec![Value::known(F::ZERO); layout.width()];
                    for (&column, value) in layout.goal_columns().iter().zip(&values) {
                        w[column] = *value;
                    }
                    let cells = chips.well_formed.assign_goal(
                        layouter.namespace(|| format!("link {place}: goal")),
                        layout,
                        &w,
                    )?;
                    layout
                        .goal_columns()
                        .iter()
                        .map(|&column| cells[column].clone())
                        .collect()
                }
                // An outgoing goal is matched with a node's goal.
                Side::Import => layouter.assign_region(
                    || format!("link {place}: goal"),
                    |mut region| {
                        values
                            .iter()
                            .enumerate()
                            .map(|(row, value)| {
                                region.assign_advice(|| "goal", w_column, row, || *value)
                            })
                            .collect::<Result<Vec<_>, Error>>()
                    },
                )?,
            };
            let active = link.assign_gate(
                layouter.namespace(|| format!("link {place}: shuffle")),
                &goal,
                &values,
                side,
                gate.as_ref().map(|gate| gate.active),
            )?;
            let published = boundary.assign_link(
                layouter.namespace(|| format!("link {place}: commitment")),
                &goal[digest_index],
                gate.as_ref().map(|gate| gate.salt),
                &active,
            )?;
            layouter.constrain_instance(published.cell(), config.links, place)?;
        }
        Ok(())
    }
}
