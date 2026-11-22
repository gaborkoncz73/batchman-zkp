//! Step 4b: the fact chip, the circuit form of the fact check of
//! `proof_witness::check`.
//!
//! The fact registry publishes one commitment per issued fact,
//! `Poseidon(FACT_DOMAIN, digest(goal), salt, 0)` with a secret random salt
//! ([`policy_constraints::term::fact_commitment`]). The list is the circuit's
//! public input: an instance column, one commitment per row, zero below.
//!
//! Every node hashes its own goal digest with its `fact_salt` cell, on the
//! Poseidon columns of the well-formedness chip, and looks up
//! `is_fact · hash` in the instance column. For a node that is no fact the
//! branch's rows make `is_fact = 0`, and 0 is in the column (below the list);
//! for a fact node `is_fact = 1`, so its commitment must be a published one.
//! Every node does the same, so the proof does not show which nodes are facts.
//!
//! ```text
//!          [0]       [1]
//! row 0    is_fact   hash      ← lookup  is_fact · hash  ∈  commitments
//! ```
//!
//! The domain and the zero word of the hash are constants in a region of
//! their own. Four words are a whole number of rate-2 blocks, so the Poseidon
//! chip adds no padding words: it loads those through fixed cells, whose
//! values the PSE prover does not compute.

use std::marker::PhantomData;

use ff::PrimeField;
use halo2_poseidon::poseidon::primitives::{ConstantLength, Spec};
use halo2_poseidon::poseidon::{Hash, Pow5Chip, Pow5Config};
use halo2_proofs::circuit::{AssignedCell, Layouter};
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Error, Instance, Selector};
use halo2_proofs::poly::Rotation;
use policy_constraints::term::FACT_DOMAIN;
use policy_constraints::{Cell, Layout, Position, Slot};

#[derive(Debug, Clone)]
pub struct FactConfig<F: PrimeField> {
    cells: [Column<Advice>; 2],
    commitments: Column<Instance>,
    q_fact: Selector,
    poseidon: Pow5Config<F, 3, 2>,
}

impl<F: PrimeField> FactConfig<F> {
    /// `poseidon` is the configuration of a Poseidon chip to hash with (the
    /// well-formedness chip's); `commitments` is the public commitment list.
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        poseidon: Pow5Config<F, 3, 2>,
        commitments: Column<Instance>,
    ) -> Self {
        let cells: [Column<Advice>; 2] = std::array::from_fn(|_| meta.advice_column());
        for column in cells {
            meta.enable_equality(column);
        }
        let q_fact = meta.complex_selector();
        meta.lookup_any("a fact's commitment is published", |meta| {
            let q = meta.query_selector(q_fact);
            let is_fact = meta.query_advice(cells[0], Rotation::cur());
            let hash = meta.query_advice(cells[1], Rotation::cur());
            let published = meta.query_instance(commitments, Rotation::cur());
            vec![(q * is_fact * hash, published)]
        });
        Self {
            cells,
            commitments,
            q_fact,
            poseidon,
        }
    }

    /// The public commitment list.
    pub fn commitments(&self) -> Column<Instance> {
        self.commitments
    }
}

#[derive(Debug)]
pub struct FactChip<F: PrimeField, S: Spec<F, 3, 2>> {
    config: FactConfig<F>,
    _spec: PhantomData<S>,
}

impl<F: PrimeField, S: Spec<F, 3, 2>> FactChip<F, S> {
    pub fn construct(config: FactConfig<F>) -> Self {
        Self {
            config,
            _spec: PhantomData,
        }
    }

    /// Hashes the node's goal digest with its salt and looks the commitment
    /// up if the node is a fact. Nothing to do if the policy has no external
    /// facts. `w` are the node's cells in `layout` order.
    pub fn assign_node(
        &self,
        mut layouter: impl Layouter<F>,
        layout: &Layout,
        w: &[AssignedCell<F, F>],
    ) -> Result<(), Error> {
        let Some((is_fact, salt)) = layout.fact_cells() else {
            return Ok(());
        };
        let config = &self.config;
        let digest = layout
            .column(&Cell::Digest(Position::root(Slot::Parent)))
            .expect("the parent goal is always opened");

        let (domain, zero) = layouter.assign_region(
            || "fact constants",
            |mut region| {
                let domain = region.assign_advice_from_constant(
                    || "fact domain",
                    config.cells[0],
                    0,
                    F::from(FACT_DOMAIN),
                )?;
                let zero =
                    region.assign_advice_from_constant(|| "zero", config.cells[1], 0, F::ZERO)?;
                Ok((domain, zero))
            },
        )?;
        let hasher = Hash::<F, _, S, ConstantLength<4>, 3, 2>::init(
            Pow5Chip::construct(config.poseidon.clone()),
            layouter.namespace(|| "fact hash init"),
        )?;
        let hash = hasher.hash(
            layouter.namespace(|| "fact hash"),
            [domain, w[digest].clone(), w[salt].clone(), zero],
        )?;

        layouter.assign_region(
            || "fact",
            |mut region| {
                config.q_fact.enable(&mut region, 0)?;
                w[is_fact].copy_advice(|| "is_fact", &mut region, config.cells[0], 0)?;
                hash.copy_advice(|| "hash", &mut region, config.cells[1], 0)?;
                Ok(())
            },
        )
    }
}
