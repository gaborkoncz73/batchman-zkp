/*use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    plonk::{Advice, Column, ConstraintSystem, Error, Instance},
    pasta::Fp,
};
use halo2_gadgets::poseidon::{
    primitives::{P128Pow5T3, ConstantLength},
    Pow5Chip, Pow5Config, Hash,
};
use halo2curves::ff::Field;

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
        let instance = meta.instance_column();

        for col in state.iter().chain([&partial_sbox, &input_col, &expected_col]) {
            meta.enable_equality(*col);
        }
        meta.enable_equality(instance);

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
        pair: [Value<Fp>; 2],
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let inputs: [AssignedCell<Fp, Fp>; 2] = layouter.assign_region(
            || "hash2 inputs",
            |mut region| {
                let a = region.assign_advice(|| "a", self.cfg.input_col, 0, || pair[0])?;
                let b = region.assign_advice(|| "b", self.cfg.input_col, 1, || pair[1])?;
                Ok([a, b])
            },
        )?;

        let chip = Pow5Chip::<Fp, 3, 2>::construct(self.cfg.poseidon.clone());
        let hasher = Hash::<Fp, Pow5Chip<Fp, 3, 2>, P128Pow5T3, ConstantLength<2>, 3, 2>::init(
            chip,
            layouter.namespace(|| "poseidon init"),
        )?;

        hasher.hash(layouter.namespace(|| "poseidon hash2"), inputs)
    }

    /// Hash tetszőleges hosszú listáról (Vec<Value<Fp>>)
    pub fn hash_list(
        &self,
        mut layouter: impl Layouter<Fp>,
        vals: &[Value<Fp>],
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let mut acc = Value::known(Fp::ZERO);

        // rate=2 → chunkonként hash-elünk
        for (i, chunk) in vals.chunks(2).enumerate() {
            let pair = [
                chunk.get(0).cloned().unwrap_or(Value::known(Fp::ZERO)),
                chunk.get(1).cloned().unwrap_or(Value::known(Fp::ZERO)),
            ];
            let res = self.hash2(layouter.namespace(|| format!("hash chunk {i}")), pair)?;
            acc = res.value().cloned();
        }

        // végső hash a maradékból
        self.hash2(layouter.namespace(|| "final poseidon hash"), [acc, Value::known(Fp::ZERO)])
    }

    /// Hash → publikusan megadott hash-érték ellenőrzése
    pub fn verify_hash_commitment(
        &self,
        mut layouter: impl Layouter<Fp>,
        vals: &[Value<Fp>],
        instance_index: usize,
    ) -> Result<(), Error> {
        let computed_hash =
            self.hash_list(layouter.namespace(|| "hash inputs"), vals)?;

        // betöltjük a publikus instance hash-t
        /*let expected = layouter.assign_region(
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
        )?;*/

        // összevetjük a kettőt
        /*layouter.assign_region(
            || "check hash eq",
            |mut region| region.constrain_equal(computed_hash.cell(), expected.cell()),
        )*/
    
        Ok(())
    }
}
*/