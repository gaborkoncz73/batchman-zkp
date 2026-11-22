//! Step 3a: the well-formedness chip, the circuit form of
//! [`Layout::check_well_formed`].
//!
//! A node's assignment `w` lies in one advice column, one cell per row. Every
//! opened position of the layout gets a block of two rows, its cells copied
//! out of `w`:
//!
//! ```text
//!          block[0]  [1]      [2]    [3]     [4..8]          [8]   [9]
//! row 0    tag       payload  arity  digest  child digests   hash  –
//! row 1    payload0  absent   present[0..4]  inverse[0..4]
//! ```
//!
//! * A lookup puts `(tag, arity, payload0, absent, present)` in the table of
//!   [`Shape`]s: the tag is a tag, the arity fits it, and the flags are the
//!   ones this (tag, arity) implies.
//! * Gates make the cells obey the flags: `payload · payload0 = 0`,
//!   `digest · absent = 0`, child digest `k` is zero where `present[k] = 0`,
//!   and non-zero (it has an inverse) where `present[k] = 1`.
//! * Parent-side positions are hashed with Poseidon over
//!   `(NODE_DOMAIN, tag, payload, arity, children)`, and a gate makes
//!   `digest = (1 − absent) · hash`. Child-side positions are not hashed: their
//!   cells are matched with the parent-side cells of the node that proves the
//!   body goal ([`crate::link`]), which hashes them.

use std::marker::PhantomData;

use ff::PrimeField;
use halo2_poseidon::poseidon::primitives::{ConstantLength, Spec};
use halo2_poseidon::poseidon::{Hash, Pow5Chip, Pow5Config};
use halo2_proofs::circuit::{AssignedCell, Layouter, Value};
use halo2_proofs::plonk::{
    Advice, Column, ConstraintSystem, Error, Expression, Selector, TableColumn,
};
use halo2_proofs::poly::Rotation;
use policy_constraints::term::NODE_DOMAIN;
use policy_constraints::{Layout, MAX_ARITY, Slot};

use crate::shape::{SHAPE_WIDTH, Shape};

const BLOCK_COLUMNS: usize = 10;
// Row 0 of a block.
const TAG: usize = 0;
const PAYLOAD: usize = 1;
const ARITY: usize = 2;
const DIGEST: usize = 3;
const CHILD: usize = 4;
const HASH: usize = CHILD + MAX_ARITY;
// Row 1 of a block.
const PAYLOAD_ZERO: usize = 0;
const ABSENT: usize = 1;
const PRESENT: usize = 2;
const INVERSE: usize = PRESENT + MAX_ARITY;

/// Width of the Poseidon state (rate 2), as in [`policy_constraints::term`].
const WIDTH: usize = 3;
const RATE: usize = 2;
/// Words hashed per node: the domain, tag, payload, arity and the children.
const HASHED: usize = 4 + MAX_ARITY;

#[derive(Debug, Clone)]
pub struct WellFormedConfig<F: PrimeField> {
    w: Column<Advice>,
    block: [Column<Advice>; BLOCK_COLUMNS],
    /// First row of every block: the lookup and the structure gate.
    q_shape: Selector,
    /// First row of a parent-side block: the digest gate.
    q_hash: Selector,
    /// `on`, then the [`Shape::row`] columns.
    table: [TableColumn; 1 + SHAPE_WIDTH],
    poseidon: Pow5Config<F, WIDTH, RATE>,
}

impl<F: PrimeField> WellFormedConfig<F> {
    /// Configures the chip around the column `w` that holds the assignments.
    pub fn configure<S: Spec<F, WIDTH, RATE>>(
        meta: &mut ConstraintSystem<F>,
        w: Column<Advice>,
    ) -> Self {
        meta.enable_equality(w);
        let block: [Column<Advice>; BLOCK_COLUMNS] = std::array::from_fn(|_| meta.advice_column());
        // The cells of row 0 are copies (of w, or of the hash output).
        for column in &block[..=HASH] {
            meta.enable_equality(*column);
        }
        let q_shape = meta.complex_selector();
        let q_hash = meta.selector();
        let table = std::array::from_fn(|_| meta.lookup_table_column());

        let state = std::array::from_fn(|_| meta.advice_column());
        let partial_sbox = meta.advice_column();
        let rc_a = std::array::from_fn(|_| meta.fixed_column());
        let rc_b = std::array::from_fn(|_| meta.fixed_column());
        // Also holds NODE_DOMAIN, copied into every hash.
        meta.enable_constant(rc_b[0]);
        let poseidon = Pow5Chip::configure::<S>(meta, state, partial_sbox, rc_a, rc_b);

        meta.create_gate("subterm cells follow the shape flags", |meta| {
            let q = meta.query_selector(q_shape);
            let one = Expression::Constant(F::ONE);
            let payload = meta.query_advice(block[PAYLOAD], Rotation::cur());
            let digest = meta.query_advice(block[DIGEST], Rotation::cur());
            let payload_zero = meta.query_advice(block[PAYLOAD_ZERO], Rotation::next());
            let absent = meta.query_advice(block[ABSENT], Rotation::next());
            let mut constraints = vec![
                q.clone() * payload * payload_zero,
                q.clone() * digest * absent,
            ];
            for child in 0..MAX_ARITY {
                let digest = meta.query_advice(block[CHILD + child], Rotation::cur());
                let present = meta.query_advice(block[PRESENT + child], Rotation::next());
                let inverse = meta.query_advice(block[INVERSE + child], Rotation::next());
                // Missing children are zero, present ones are not.
                constraints.push(q.clone() * digest.clone() * (one.clone() - present.clone()));
                constraints.push(q.clone() * present * (digest * inverse - one.clone()));
            }
            constraints
        });

        meta.create_gate("parent-side digest is the hash", |meta| {
            let q = meta.query_selector(q_hash);
            let one = Expression::Constant(F::ONE);
            let digest = meta.query_advice(block[DIGEST], Rotation::cur());
            let hash = meta.query_advice(block[HASH], Rotation::cur());
            let absent = meta.query_advice(block[ABSENT], Rotation::next());
            vec![q * (digest - (one - absent) * hash)]
        });

        meta.lookup("subterm shape", |meta| {
            let q = meta.query_selector(q_shape);
            let mut inputs = vec![
                meta.query_advice(block[TAG], Rotation::cur()),
                meta.query_advice(block[ARITY], Rotation::cur()),
                meta.query_advice(block[PAYLOAD_ZERO], Rotation::next()),
                meta.query_advice(block[ABSENT], Rotation::next()),
            ];
            for child in 0..MAX_ARITY {
                inputs.push(meta.query_advice(block[PRESENT + child], Rotation::next()));
            }
            // A disabled row looks up the all-zero row, whose `on` is 0; an
            // enabled one can only match a real shape, whose `on` is 1.
            let mut pairs = vec![(q.clone(), table[0])];
            for (input, column) in inputs.into_iter().zip(&table[1..]) {
                pairs.push((q.clone() * input, *column));
            }
            pairs
        });

        Self {
            w,
            block,
            q_shape,
            q_hash,
            table,
            poseidon,
        }
    }

    /// The column that holds the node assignments.
    pub fn w(&self) -> Column<Advice> {
        self.w
    }

    /// The Poseidon chip's configuration, for other chips that hash on the
    /// same columns.
    pub fn poseidon(&self) -> &Pow5Config<F, WIDTH, RATE> {
        &self.poseidon
    }
}

/// The cells of one node the other chips use.
#[derive(Debug, Clone)]
pub struct Shaped<F: PrimeField> {
    /// The cells of `w`, one per cell of the layout.
    pub w: Vec<AssignedCell<F, F>>,
    /// The `absent` flag of every opened position, in the order of
    /// [`Layout::opened_nodes`]: 1 if the position is absent, else 0.
    pub absent: Vec<AssignedCell<F, F>>,
}

impl<F: PrimeField> Shaped<F> {
    /// The `absent` flag of the root of child slot `slot`.
    pub fn slot_absent(&self, layout: &Layout, slot: usize) -> &AssignedCell<F, F> {
        let root = policy_constraints::Position::root(Slot::Child(
            u8::try_from(slot).expect("fewer than 256 slots"),
        ));
        let index = layout
            .opened_nodes()
            .iter()
            .position(|node| node.position == root)
            .expect("the layout opens every child slot");
        &self.absent[index]
    }
}

#[derive(Debug)]
pub struct WellFormedChip<F: PrimeField, S: Spec<F, WIDTH, RATE>> {
    config: WellFormedConfig<F>,
    _spec: PhantomData<S>,
}

impl<F: PrimeField, S: Spec<F, WIDTH, RATE>> WellFormedChip<F, S> {
    pub fn construct(config: WellFormedConfig<F>) -> Self {
        Self {
            config,
            _spec: PhantomData,
        }
    }

    /// Loads the table of shapes. Once per circuit.
    pub fn load_table(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error> {
        let table = self.config.table;
        layouter.assign_table(
            || "subterm shapes",
            |mut assigner| {
                // Row 0 is the all-zero row of disabled positions; the layouter
                // also pads the table with copies of it.
                for column in table {
                    assigner.assign_cell(|| "off", column, 0, || Value::known(F::ZERO))?;
                }
                for (index, shape) in Shape::all().iter().enumerate() {
                    let row = index + 1;
                    assigner.assign_cell(|| "on", table[0], row, || Value::known(F::ONE))?;
                    for (column, value) in table[1..].iter().zip(shape.row::<F>()) {
                        assigner.assign_cell(|| "shape", *column, row, || Value::known(value))?;
                    }
                }
                Ok(())
            },
        )
    }

    /// Assigns one node's `w` and checks the well-formedness of every opened
    /// position of `layout`. Returns the cells of `w` and the `absent` flags
    /// for the other checks.
    pub fn assign_node(
        &self,
        layouter: impl Layouter<F>,
        layout: &Layout,
        w: &[Value<F>],
    ) -> Result<Shaped<F>, Error> {
        self.assign(layouter, layout, w, false)
    }

    /// Assigns a goal alone and checks it as a node checks its parent goal:
    /// every parent-side position is well-formed and hashes to its digest.
    /// `w` is as wide as `layout`, with the goal in its parent-side cells
    /// (the other cells are not checked). Returns the cells of `w`.
    pub fn assign_goal(
        &self,
        layouter: impl Layouter<F>,
        layout: &Layout,
        w: &[Value<F>],
    ) -> Result<Vec<AssignedCell<F, F>>, Error> {
        Ok(self.assign(layouter, layout, w, true)?.w)
    }

    /// [`Self::assign_node`], or with `goal_only` only the parent-side
    /// positions (the `absent` flags of the others are then missing).
    fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        layout: &Layout,
        w: &[Value<F>],
        goal_only: bool,
    ) -> Result<Shaped<F>, Error> {
        let config = &self.config;
        assert_eq!(w.len(), layout.width(), "one value per cell of the layout");

        let (cells, domain) = layouter.assign_region(
            || "w",
            |mut region| {
                let cells = w
                    .iter()
                    .enumerate()
                    .map(|(row, value)| region.assign_advice(|| "w", config.w, row, || *value))
                    .collect::<Result<Vec<_>, Error>>()?;
                let domain = region.assign_advice_from_constant(
                    || "node domain",
                    config.w,
                    w.len(),
                    F::from(NODE_DOMAIN),
                )?;
                Ok((cells, domain))
            },
        )?;

        let mut absent = Vec::with_capacity(layout.opened_nodes().len());
        for node in layout.opened_nodes() {
            if goal_only && node.position.slot() != Slot::Parent {
                continue;
            }
            let hash = if node.position.slot() == Slot::Parent {
                let mut message = vec![domain.clone()];
                message.extend(
                    [node.tag, node.payload, node.arity].map(|column| cells[column].clone()),
                );
                message.extend(node.children.map(|column| cells[column].clone()));
                let message: [AssignedCell<F, F>; HASHED] = message
                    .try_into()
                    .expect("domain, tag, payload, arity and the children");
                let chip = Pow5Chip::construct(config.poseidon.clone());
                let hasher = Hash::<F, _, S, ConstantLength<HASHED>, WIDTH, RATE>::init(
                    chip,
                    layouter.namespace(|| "hash init"),
                )?;
                Some(hasher.hash(
                    layouter.namespace(|| format!("hash {}", node.position)),
                    message,
                )?)
            } else {
                None
            };

            let node_absent = layouter.assign_region(
                || format!("subterm {}", node.position),
                |mut region| {
                    config.q_shape.enable(&mut region, 0)?;
                    let copies = [
                        (TAG, node.tag),
                        (PAYLOAD, node.payload),
                        (ARITY, node.arity),
                        (DIGEST, node.digest),
                    ]
                    .into_iter()
                    .chain((0..MAX_ARITY).map(|child| (CHILD + child, node.children[child])));
                    for (column, cell) in copies {
                        cells[cell].copy_advice(|| "cell", &mut region, config.block[column], 0)?;
                    }
                    if let Some(hash) = &hash {
                        config.q_hash.enable(&mut region, 0)?;
                        hash.copy_advice(|| "hash", &mut region, config.block[HASH], 0)?;
                    }

                    // The flags of the node's shape; zeros if it has none, so
                    // the lookup fails.
                    let shape: Value<Option<Shape>> = cells[node.tag]
                        .value()
                        .copied()
                        .zip(cells[node.arity].value().copied())
                        .map(|(tag, arity)| Shape::of(tag, arity));
                    let flag = |pick: &dyn Fn(&Shape) -> bool| {
                        shape
                            .as_ref()
                            .map(|shape| F::from(u64::from(shape.as_ref().is_some_and(pick))))
                    };
                    region.assign_advice(
                        || "payload zero",
                        config.block[PAYLOAD_ZERO],
                        1,
                        || flag(&|shape| shape.payload_zero),
                    )?;
                    let absent = region.assign_advice(
                        || "absent",
                        config.block[ABSENT],
                        1,
                        || flag(&|shape| shape.absent),
                    )?;
                    for child in 0..MAX_ARITY {
                        region.assign_advice(
                            || "present",
                            config.block[PRESENT + child],
                            1,
                            || flag(&|shape| shape.present[child]),
                        )?;
                        let inverse = cells[node.children[child]]
                            .value()
                            .map(|digest| digest.invert().unwrap_or(F::ZERO));
                        region.assign_advice(
                            || "inverse",
                            config.block[INVERSE + child],
                            1,
                            || inverse,
                        )?;
                    }
                    Ok(absent)
                },
            )?;
            absent.push(node_absent);
        }
        Ok(Shaped { w: cells, absent })
    }
}
