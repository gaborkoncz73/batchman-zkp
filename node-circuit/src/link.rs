//! Step 5a: the links between the nodes, the circuit form of
//! `proof_witness::check`'s matching of body goals with nodes.
//!
//! Every goal slot of a node (`P`, `C0`, `C1`, …) has the same list of cells
//! ([`Layout::goal_columns`], [`Layout::slot_links`]). The chip copies each
//! list into a segment of rows and compresses it with the challenge `c`
//! (drawn after the cells are committed) by Horner's rule, so the last row of
//! a segment holds
//!
//! ```text
//! h = v₀·c^(G−1) + v₁·c^(G−2) + … + v_(G−1)      (G cells per goal)
//! ```
//!
//! Two lists of cells give the same `h` with probability at most `G/p`
//! unless they are equal. Per node, segment 0 is the parent goal (the node
//! *exports* it) and segment `k + 1` is child slot `k` (the node *imports*
//! it). A shuffle argument proves that the pairs
//!
//! ```text
//! (export, export · h)   over all rows      and      (import, import · h)   over all rows
//! ```
//!
//! are the same multiset, where
//!
//! * `export = active` on the parent segment of a non-root node, and
//! * `import = active · (1 − absent)` on a slot segment, `absent` being the
//!   slot root's flag from the well-formedness chip ([`crate::well_formed`]).
//!
//! Rows that are neither give `(0, 0)` on both sides; since both sides range
//! over the same rows, the zeros balance whenever the rest does, and the flag
//! in the pair keeps a zero `h` apart from them. So every present body goal
//! of an active node is the goal of exactly one active non-root node, and
//! every active non-root node proves exactly one body goal: the active nodes
//! reachable from the roots form a tree (a cycle can balance itself, but no
//! root reaches it, as every node has one parent and the roots have none).
//!
//! A root node (placed first, see [`crate::TreeCircuit`]) exports nothing and
//! is active by a constant. Inactive nodes are padding: honest copies of some
//! node, checked like every node, that export and import nothing.
//!
//! ```text
//!          phase   segment rows                       last row of a segment
//! v        1       a copy of the cell                 –
//! h        2       Horner in c                        the compressed goal
//! flag     1       –                                  export or import
//! active   1       –                                  the node's activity
//! absent   1       –                                  the slot's absent flag
//! ```

use ff::PrimeField;
use halo2_proofs::circuit::{AssignedCell, Layouter, Value};
use halo2_proofs::plonk::{
    Advice, Challenge, Column, ConstraintSystem, Error, Expression, FirstPhase, SecondPhase,
    Selector,
};
use halo2_proofs::poly::Rotation;
use policy_constraints::Layout;

use crate::well_formed::Shaped;

#[derive(Debug, Clone)]
pub struct LinkConfig {
    v: Column<Advice>,
    h: Column<Advice>,
    flag: Column<Advice>,
    active: Column<Advice>,
    absent: Column<Advice>,
    /// The first row of a segment.
    q_start: Selector,
    /// The other rows of a segment.
    q_step: Selector,
    /// The last row of the parent segment of a non-root node.
    q_export: Selector,
    /// The last row of a child slot segment.
    q_import: Selector,
    c: Challenge,
}

impl LinkConfig {
    /// Configures the chip; `v` is an equality-enabled first-phase column the
    /// chip may share with other chips' regions.
    pub fn configure<F: PrimeField>(meta: &mut ConstraintSystem<F>, v: Column<Advice>) -> Self {
        meta.enable_equality(v);
        let flag = meta.advice_column_in(FirstPhase);
        let active = meta.advice_column_in(FirstPhase);
        let absent = meta.advice_column_in(FirstPhase);
        meta.enable_equality(active);
        meta.enable_equality(absent);
        let c = meta.challenge_usable_after(FirstPhase);
        let h = meta.advice_column_in(SecondPhase);

        let q_start = meta.selector();
        let q_step = meta.selector();
        let q_export = meta.complex_selector();
        let q_import = meta.complex_selector();

        meta.create_gate("link: a segment starts with its first cell", |meta| {
            let q = meta.query_selector(q_start);
            let h = meta.query_advice(h, Rotation::cur());
            let v = meta.query_advice(v, Rotation::cur());
            vec![q * (h - v)]
        });
        meta.create_gate("link: Horner step", |meta| {
            let q = meta.query_selector(q_step);
            let c = meta.query_challenge(c);
            let h_cur = meta.query_advice(h, Rotation::cur());
            let h_prev = meta.query_advice(h, Rotation::prev());
            let v = meta.query_advice(v, Rotation::cur());
            vec![q * (h_cur - c * h_prev - v)]
        });
        meta.create_gate("link: a node exports its goal if it is active", |meta| {
            let q = meta.query_selector(q_export);
            let one = Expression::Constant(F::ONE);
            let flag = meta.query_advice(flag, Rotation::cur());
            let active = meta.query_advice(active, Rotation::cur());
            vec![
                q.clone() * (flag - active.clone()),
                q * active.clone() * (one - active),
            ]
        });
        meta.create_gate(
            "link: an active node imports every present body goal",
            |meta| {
                let q = meta.query_selector(q_import);
                let one = Expression::Constant(F::ONE);
                let flag = meta.query_advice(flag, Rotation::cur());
                let active = meta.query_advice(active, Rotation::cur());
                let absent = meta.query_advice(absent, Rotation::cur());
                vec![
                    q.clone() * (flag - active.clone() * (one.clone() - absent)),
                    q * active.clone() * (one - active),
                ]
            },
        );

        meta.shuffle("link: the body goals are the nodes' goals", |meta| {
            let export = meta.query_selector(q_export) * meta.query_advice(flag, Rotation::cur());
            let import = meta.query_selector(q_import) * meta.query_advice(flag, Rotation::cur());
            let h = meta.query_advice(h, Rotation::cur());
            vec![
                (export.clone(), import.clone()),
                (export * h.clone(), import * h),
            ]
        });

        Self {
            v,
            h,
            flag,
            active,
            absent,
            q_start,
            q_step,
            q_export,
            q_import,
            c,
        }
    }
}

/// Which side of the shuffle a link to another shard takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// An incoming link stands in for a node of another shard: it exports
    /// the goal.
    Export,
    /// An outgoing link stands in for a body goal of another shard: it
    /// imports the goal.
    Import,
}

/// Whether a node is a root, active, or padding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// One of the first nodes, whose goal is a trusted root: active by a
    /// constant, exports nothing.
    Root,
    /// Any other node; `Value` holds whether it is active (private).
    Other,
}

#[derive(Debug, Clone)]
pub struct LinkChip {
    config: LinkConfig,
}

impl LinkChip {
    pub fn construct(config: LinkConfig) -> Self {
        Self { config }
    }

    /// The segments of one node: its exported goal and its imported body
    /// goals, with their flags. `w` are the values of `shaped.w`: the
    /// compressed goals are computed from them, as the PSE prover computes
    /// only the cells of the current phase, so a first-phase cell has no
    /// value while the second phase is assigned. `active` is ignored for a
    /// root.
    pub fn assign_node<F: PrimeField>(
        &self,
        mut layouter: impl Layouter<F>,
        layout: &Layout,
        shaped: &Shaped<F>,
        w: &[Value<F>],
        role: Role,
        active: Value<bool>,
    ) -> Result<(), Error> {
        let config = &self.config;
        let c = layouter.get_challenge(config.c);
        let goal = layout.goal_columns();
        let mut segments: Vec<Vec<usize>> = vec![goal.to_vec()];
        for slot in 0..layout.child_slots() {
            segments.push(
                layout
                    .slot_links(slot)
                    .iter()
                    .map(|link| link.in_parent)
                    .collect(),
            );
        }
        let length = goal.len();
        let active = active.map(|active| F::from(u64::from(active)));

        layouter.assign_region(
            || "link",
            |mut region| {
                let mut first_active: Option<AssignedCell<F, F>> = None;
                for (segment, columns) in segments.iter().enumerate() {
                    let start = segment * length;
                    let mut h = Value::known(F::ZERO);
                    for (offset, &column) in columns.iter().enumerate() {
                        let row = start + offset;
                        shaped.w[column].copy_advice(|| "v", &mut region, config.v, row)?;
                        h = if offset == 0 {
                            config.q_start.enable(&mut region, row)?;
                            w[column]
                        } else {
                            config.q_step.enable(&mut region, row)?;
                            h * c + w[column]
                        };
                        region.assign_advice(|| "h", config.h, row, || h)?;
                    }

                    let last = start + length - 1;
                    let active_cell = match (role, &first_active) {
                        (Role::Root, None) => region.assign_advice_from_constant(
                            || "a root is active",
                            config.active,
                            last,
                            F::ONE,
                        )?,
                        (Role::Other, None) => {
                            region.assign_advice(|| "active", config.active, last, || active)?
                        }
                        (_, Some(first)) => {
                            first.copy_advice(|| "active", &mut region, config.active, last)?
                        }
                    };
                    let flag = if segment == 0 {
                        if role == Role::Other {
                            config.q_export.enable(&mut region, last)?;
                        }
                        active_cell.value().copied()
                    } else {
                        config.q_import.enable(&mut region, last)?;
                        let absent = shaped.slot_absent(layout, segment - 1).copy_advice(
                            || "absent",
                            &mut region,
                            config.absent,
                            last,
                        )?;
                        active_cell
                            .value()
                            .copied()
                            .zip(absent.value().copied())
                            .map(|(active, absent)| active * (F::ONE - absent))
                    };
                    region.assign_advice(|| "flag", config.flag, last, || flag)?;
                    first_active.get_or_insert(active_cell);
                }
                Ok(())
            },
        )
    }

    /// One link to another shard: a single segment over the goal's cells
    /// (`goal`, in the order of [`Layout::goal_columns`], with their values
    /// `values`), exported or imported when `active`. Returns the `active`
    /// cell.
    pub fn assign_gate<F: PrimeField>(
        &self,
        mut layouter: impl Layouter<F>,
        goal: &[AssignedCell<F, F>],
        values: &[Value<F>],
        side: Side,
        active: Value<bool>,
    ) -> Result<AssignedCell<F, F>, Error> {
        let config = &self.config;
        let c = layouter.get_challenge(config.c);
        let active = active.map(|active| F::from(u64::from(active)));
        layouter.assign_region(
            || "link gate",
            |mut region| {
                let mut h = Value::known(F::ZERO);
                for (row, (cell, value)) in goal.iter().zip(values).enumerate() {
                    cell.copy_advice(|| "v", &mut region, config.v, row)?;
                    h = if row == 0 {
                        config.q_start.enable(&mut region, row)?;
                        *value
                    } else {
                        config.q_step.enable(&mut region, row)?;
                        h * c + *value
                    };
                    region.assign_advice(|| "h", config.h, row, || h)?;
                }
                let last = goal.len() - 1;
                let active_cell =
                    region.assign_advice(|| "active", config.active, last, || active)?;
                region.assign_advice(|| "flag", config.flag, last, || active)?;
                match side {
                    Side::Export => config.q_export.enable(&mut region, last)?,
                    Side::Import => {
                        config.q_import.enable(&mut region, last)?;
                        region.assign_advice_from_constant(
                            || "a link is no absent slot",
                            config.absent,
                            last,
                            F::ZERO,
                        )?;
                    }
                }
                Ok(active_cell)
            },
        )
    }
}
