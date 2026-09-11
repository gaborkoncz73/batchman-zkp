use halo2_gadgets::poseidon::{
    Hash, Pow5Chip, Pow5Config,
    primitives::{ConstantLength, P128Pow5T3},
};
use halo2_proofs::{
    circuit::{AssignedCell, Layouter},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error},
};
use halo2curves::ff::Field;

use crate::typed_fact::{
    TYPED_FACT_DOMAIN_TAG, TYPED_FACT_HASH_INPUTS, TYPED_FACT_HASH_SCHEMA_VERSION,
};

/// ─────────────────────────────
/// CONFIG
/// ─────────────────────────────
#[derive(Clone, Debug)]
pub struct PoseidonHashConfig {
    /// Poseidon chip konfiguráció (WIDTH=3, RATE=2)
    pub poseidon: Pow5Config<Fp, 3, 2>,
    /// oszlop a hash input értékeinek
    pub input_col: Column<Advice>,
    /// oszlop az elvárt hash-nek (ha összevetjük)
    pub expected_col: Column<Advice>,
}

/// ─────────────────────────────
/// CHIP
/// ─────────────────────────────
#[derive(Clone, Debug)]
pub struct PoseidonHashChip {
    pub cfg: PoseidonHashConfig,
}

impl PoseidonHashChip {
    /// Poseidon gadget konfigurálása
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> PoseidonHashConfig {
        let state = [0, 1, 2].map(|_| meta.advice_column());
        let partial_sbox = meta.advice_column();
        let rc_a = [0, 1, 2].map(|_| meta.fixed_column());
        let rc_b = [0, 1, 2].map(|_| meta.fixed_column());

        let input_col = meta.advice_column();
        let expected_col = meta.advice_column();
        //let instance = meta.instance_column();

        for col in state
            .iter()
            .chain([&partial_sbox, &input_col, &expected_col])
        {
            meta.enable_equality(*col);
        }
        //meta.enable_equality(instance);

        let constant = meta.fixed_column();
        meta.enable_constant(constant);

        let poseidon =
            Pow5Chip::<Fp, 3, 2>::configure::<P128Pow5T3>(meta, state, partial_sbox, rc_a, rc_b);

        PoseidonHashConfig {
            poseidon,
            input_col,
            expected_col,
        }
    }

    pub fn construct(cfg: PoseidonHashConfig) -> Self {
        Self { cfg }
    }

    /// Hash két elemről: H([a,b])
    pub fn hash2(
        &self,
        mut layouter: impl Layouter<Fp>,
        pair: [AssignedCell<Fp, Fp>; 2],
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        self.hash_fixed(layouter.namespace(|| "poseidon hash2"), pair)
    }

    /// Fixed-length Poseidon sponge over the supplied cells themselves.
    ///
    /// `Hash::hash` copy-constrains every absorbed message word into the
    /// Poseidon state.  In particular, this must not be replaced by assigning
    /// fresh cells from `cell.value()`, which would leave commitments
    /// disconnected from the circuit witness.
    pub fn hash_fixed<const L: usize>(
        &self,
        mut layouter: impl Layouter<Fp>,
        inputs: [AssignedCell<Fp, Fp>; L],
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let chip = Pow5Chip::<Fp, 3, 2>::construct(self.cfg.poseidon.clone());
        let hasher = Hash::<Fp, Pow5Chip<Fp, 3, 2>, P128Pow5T3, ConstantLength<L>, 3, 2>::init(
            chip,
            layouter.namespace(|| "poseidon init"),
        )?;
        hasher.hash(layouter.namespace(|| "poseidon hash"), inputs)
    }

    /// Hashes the exact typed external-fact preimage
    /// `[domain, schema_version, typed_goal_digest, salt]`.
    ///
    /// Both caller-supplied cells are absorbed through copy constraints.  The
    /// protocol constants are fixed cells, so neither can become prover data.
    pub fn hash_typed_fact(
        &self,
        mut layouter: impl Layouter<Fp>,
        typed_goal_digest: &AssignedCell<Fp, Fp>,
        salt: &AssignedCell<Fp, Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let constants = layouter.assign_region(
            || "typed fact hash protocol constants",
            |mut region| {
                let domain = region.assign_advice_from_constant(
                    || "typed fact domain",
                    self.cfg.input_col,
                    0,
                    Fp::from(TYPED_FACT_DOMAIN_TAG),
                )?;
                let schema = region.assign_advice_from_constant(
                    || "typed fact schema version",
                    self.cfg.input_col,
                    1,
                    Fp::from(TYPED_FACT_HASH_SCHEMA_VERSION as u64),
                )?;
                Ok([domain, schema])
            },
        )?;

        self.hash_fixed::<TYPED_FACT_HASH_INPUTS>(
            layouter.namespace(|| "typed external fact Poseidon"),
            [
                constants[0].clone(),
                constants[1].clone(),
                typed_goal_digest.clone(),
                salt.clone(),
            ],
        )
    }

    // Hash tetszőleges hosszú listáról (Vec<Value<Fp>>)
    pub fn hash_list(
        &self,
        mut layouter: impl Layouter<Fp>,
        vals: &[AssignedCell<Fp, Fp>],
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        // Start with zero accumulator (assigned)
        let mut acc = layouter.assign_region(
            || "initial acc",
            |mut region| {
                region.assign_advice_from_constant(|| "init acc", self.cfg.input_col, 0, Fp::ZERO)
            },
        )?;

        // Each iteration takes the accumulator + next value, hashes them
        for (i, val) in vals.iter().enumerate() {
            let pair = [acc.clone(), val.clone()];
            acc = self.hash2(layouter.namespace(|| format!("hash step {i}")), pair)?;
        }

        // Return the final accumulated hash
        Ok(acc)
    }
    pub fn hash_nested_pairs(
        &self,
        mut layouter: impl Layouter<Fp>,
        proof_pairs_nested: &[Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>],
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        // acc = 0
        let mut acc = layouter.assign_region(
            || "initial acc",
            |mut region| {
                region.assign_advice_from_constant(|| "acc = 0", self.cfg.input_col, 0, Fp::ZERO)
            },
        )?;

        for (i, inner) in proof_pairs_nested.iter().enumerate() {
            for (j, (a, b)) in inner.iter().enumerate() {
                // acc = H([acc, a])
                acc = self.hash2(
                    layouter.namespace(|| format!("hash pair[{i}][{j}] step a")),
                    [acc.clone(), a.clone()],
                )?;

                // acc = H([acc, b])
                acc = self.hash2(
                    layouter.namespace(|| format!("hash pair[{i}][{j}] step b")),
                    [acc.clone(), b.clone()],
                )?;
            }
        }

        Ok(acc)
    }

    // Hash → publikusan megadott hash-érték ellenőrzése
    /*pub fn verify_hash_commitment(
        &self,
        mut layouter: impl Layouter<Fp>,
        vals: &[Value<Fp>],
        instance_index: usize,
    ) -> Result<(), Error> {
        let computed_hash =
            self.hash_list(layouter.namespace(|| "hash inputs"), vals)?;

        // betöltjük a publikus instance hash-t
        let expected = layouter.assign_region(
            || "expected hash (instance)",
            |mut region| {
                region.assign_advice_from_instance(
                    || "expected hash",
                    self.cfg.instance,
                    instance_index,
                    self.cfg.expected_col,
                    0,
                )
            },
        )?;

        // összevetjük a kettőt
        layouter.assign_region(
            || "check hash eq",
            |mut region| region.constrain_equal(computed_hash.cell(), expected.cell()),
        )

        Ok(())
    }*/
}
