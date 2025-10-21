use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    plonk::{Advice, Column, ConstraintSystem, Error, Instance},
    pasta::Fp,
};
use halo2_gadgets::poseidon::{
    primitives::{P128Pow5T3, ConstantLength},
    Pow5Chip, Pow5Config, Hash,
};

#[derive(Clone, Debug)]
pub struct HashEqConfig {
    // Poseidon chip configuration (WIDTH=3, RATE=2)
    pub poseidon: Pow5Config<Fp, 3, 2>,
    pub expected_col: Column<Advice>,
    pub input_col: Column<Advice>,
}

pub struct HashEqChip;

impl HashEqChip {
    /// 🧱 Poseidon chip konfiguráció létrehozása
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> HashEqConfig {
        let state = [0, 1, 2].map(|_| meta.advice_column());
        let partial_sbox = meta.advice_column();
        let rc_a = [0, 1, 2].map(|_| meta.fixed_column());
        let rc_b = [0, 1, 2].map(|_| meta.fixed_column());

        let expected_col = meta.advice_column();
        let input_col = meta.advice_column();

        for col in state.iter().chain([&partial_sbox, &expected_col, &input_col]) {
            meta.enable_equality(*col);
        }

        let constant = meta.fixed_column();
        meta.enable_constant(constant);

        let poseidon =
            Pow5Chip::<Fp, 3, 2>::configure::<P128Pow5T3>(meta, state, partial_sbox, rc_a, rc_b);

        HashEqConfig {
            poseidon,
            expected_col,
            input_col,
        }
    }

    /// 🌳 Poseidon fa-hash a teljes (dinamikus hosszú) listára.
    pub fn tree_hash_all(
        config: &HashEqConfig,
        mut layouter: impl Layouter<Fp>,
        leaves: &[Value<Fp>],
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        use std::convert::TryInto;

        if leaves.is_empty() {
            let zero = Value::known(Fp::zero());
            return Self::hash2(config, layouter, [zero, zero]);
        }

        let mut layer: Vec<AssignedCell<Fp, Fp>> = layouter.assign_region(
            || "load leaves",
            |mut region| {
                let mut v = Vec::with_capacity(leaves.len());
                for (i, val) in leaves.iter().enumerate() {
                    let cell = region.assign_advice(
                        || format!("leaf[{i}]"),
                        config.input_col,
                        i,
                        || *val,
                    )?;
                    v.push(cell);
                }
                Ok(v)
            },
        )?;

        while layer.len() > 1 {
            let mut next: Vec<AssignedCell<Fp, Fp>> = Vec::with_capacity((layer.len() + 1) / 2);
            let mut i = 0usize;
            while i < layer.len() {
                if i + 1 < layer.len() {
                    let a = &layer[i];
                    let b = &layer[i + 1];
                    let h = Self::hash2(
                        config,
                        layouter.namespace(|| format!("H({i},{})", i + 1)),
                        [a.value().copied(), b.value().copied()],
                    )?;
                    next.push(h);
                    i += 2;
                } else {
                    let a = &layer[i];
                    let h = Self::hash2(
                        config,
                        layouter.namespace(|| format!("H({i},pad)")),
                        [a.value().copied(), Value::known(Fp::zero())],
                    )?;
                    next.push(h);
                    i += 1;
                }
            }
            layer = next;
        }

        Ok(layer.remove(0))
    }

    fn hash2(
        config: &HashEqConfig,
        mut layouter: impl Layouter<Fp>,
        pair: [Value<Fp>; 2],
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let inputs: [AssignedCell<Fp, Fp>; 2] = layouter.assign_region(
            || "hash2 inputs",
            |mut region| {
                let a = region.assign_advice(|| "a", config.input_col, 0, || pair[0])?;
                let b = region.assign_advice(|| "b", config.input_col, 1, || pair[1])?;
                Ok([a, b])
            },
        )?;

        let chip = Pow5Chip::<Fp, 3, 2>::construct(config.poseidon.clone());
        let hasher = Hash::<Fp, Pow5Chip<Fp, 3, 2>, P128Pow5T3, ConstantLength<2>, 3, 2>::init(
            chip,
            layouter.namespace(|| "poseidon init"),
        )?;

        hasher.hash(layouter.namespace(|| "poseidon hash2"), inputs)
    }

    pub fn tree_hash_and_constrain_instance(
        config: &HashEqConfig,
        mut layouter: impl Layouter<Fp>,
        leaves: &[Value<Fp>],
        instance: Column<Instance>,
        row: usize,
    ) -> Result<(), Error> {
        let root = Self::tree_hash_all(config, layouter.namespace(|| "tree"), leaves)?;
        layouter.constrain_instance(root.cell(), instance, row)
    }
}
