//! Step 5b: the links between shard proofs, the circuit form of
//! `proof_witness::shard`.
//!
//! A shard proof has a fixed number of link places on each side, part of its
//! verifying key. An active place takes part in the shard's shuffle
//! ([`crate::link`]) with a goal's cells and publishes the commitment
//! `link_commitment(digest, salt)` of the goal's digest and the link's secret
//! salt; an inactive place takes no part and publishes 0, which the verifier
//! skips:
//!
//! ```text
//! row 0    active    hash    published      published = active · hash
//! ```
//!
//! * An *incoming* place (the parent's shard) exports a goal whose cells the
//!   well-formedness chip checks as a parent goal, so its digest fixes them.
//! * An *outgoing* place (the child's shard) imports a goal; the shuffle
//!   matches its cells with a node's parent goal, which that node hashes.
//!
//! The verifier then matches the outgoing commitments of all shards with the
//! incoming ones ([`proof_witness::check_links_between`]). The hash is
//! Poseidon over `(LINK_DOMAIN, digest, salt, 0)`, four words, so the chip
//! adds no padding words (see [`crate::fact`]).

use std::marker::PhantomData;

use ff::PrimeField;
use halo2_poseidon::poseidon::primitives::{ConstantLength, Spec};
use halo2_poseidon::poseidon::{Hash, Pow5Chip, Pow5Config};
use halo2_proofs::circuit::{AssignedCell, Layouter, Value};
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector};
use halo2_proofs::poly::Rotation;
use policy_constraints::term::LINK_DOMAIN;

/// One link place of a shard: the goal's cells (in the order of
/// `Layout::goal_columns`), the link's salt, and whether the place is used.
#[derive(Debug, Clone)]
pub struct Gate<F> {
    pub goal: Vec<F>,
    pub salt: F,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct BoundaryConfig<F: PrimeField> {
    cells: [Column<Advice>; 3],
    q_link: Selector,
    poseidon: Pow5Config<F, 3, 2>,
}

impl<F: PrimeField> BoundaryConfig<F> {
    /// `poseidon` is the configuration of a Poseidon chip to hash with (the
    /// well-formedness chip's).
    pub fn configure(meta: &mut ConstraintSystem<F>, poseidon: Pow5Config<F, 3, 2>) -> Self {
        let cells: [Column<Advice>; 3] = std::array::from_fn(|_| meta.advice_column());
        for column in cells {
            meta.enable_equality(column);
        }
        let q_link = meta.selector();
        meta.create_gate("a link publishes its commitment if it is active", |meta| {
            let q = meta.query_selector(q_link);
            let active = meta.query_advice(cells[0], Rotation::cur());
            let hash = meta.query_advice(cells[1], Rotation::cur());
            let published = meta.query_advice(cells[2], Rotation::cur());
            let one = Expression::Constant(F::ONE);
            vec![
                q.clone() * (published - active.clone() * hash),
                q * active.clone() * (one - active),
            ]
        });
        Self {
            cells,
            q_link,
            poseidon,
        }
    }
}

#[derive(Debug)]
pub struct BoundaryChip<F: PrimeField, S: Spec<F, 3, 2>> {
    config: BoundaryConfig<F>,
    _spec: PhantomData<S>,
}

impl<F: PrimeField, S: Spec<F, 3, 2>> BoundaryChip<F, S> {
    pub fn construct(config: BoundaryConfig<F>) -> Self {
        Self {
            config,
            _spec: PhantomData,
        }
    }

    /// Hashes the goal's `digest` with `salt` and returns the published cell:
    /// the commitment if `active` is 1, else 0.
    pub fn assign_link(
        &self,
        mut layouter: impl Layouter<F>,
        digest: &AssignedCell<F, F>,
        salt: Value<F>,
        active: &AssignedCell<F, F>,
    ) -> Result<AssignedCell<F, F>, Error> {
        let config = &self.config;
        let (domain, salt, zero) = layouter.assign_region(
            || "link constants and salt",
            |mut region| {
                let domain = region.assign_advice_from_constant(
                    || "link domain",
                    config.cells[0],
                    0,
                    F::from(LINK_DOMAIN),
                )?;
                let zero =
                    region.assign_advice_from_constant(|| "zero", config.cells[1], 0, F::ZERO)?;
                let salt = region.assign_advice(|| "salt", config.cells[2], 0, || salt)?;
                Ok((domain, salt, zero))
            },
        )?;
        let hasher = Hash::<F, _, S, ConstantLength<4>, 3, 2>::init(
            Pow5Chip::construct(config.poseidon.clone()),
            layouter.namespace(|| "link hash init"),
        )?;
        let hash = hasher.hash(
            layouter.namespace(|| "link hash"),
            [domain, digest.clone(), salt, zero],
        )?;

        layouter.assign_region(
            || "link commitment",
            |mut region| {
                config.q_link.enable(&mut region, 0)?;
                let active = active.copy_advice(|| "active", &mut region, config.cells[0], 0)?;
                let hash = hash.copy_advice(|| "hash", &mut region, config.cells[1], 0)?;
                let published = active
                    .value()
                    .copied()
                    .zip(hash.value().copied())
                    .map(|(active, hash)| active * hash);
                region.assign_advice(|| "published", config.cells[2], 0, || published)
            },
        )
    }
}
