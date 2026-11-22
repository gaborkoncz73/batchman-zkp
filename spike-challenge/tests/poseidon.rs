//! The PSE Poseidon gadget hashes exactly like `policy_constraints::term`
//! (zcash's halo2_gadgets), natively and in a circuit of the PSE fork.

use ff::Field;
use halo2_poseidon::poseidon::primitives::{self, ConstantLength, P128Pow5T3};
use halo2_poseidon::poseidon::{Hash, Pow5Chip, Pow5Config};
use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::dev::MockProver;
use halo2_proofs::halo2curves::pasta::Fp;
use halo2_proofs::halo2curves::pasta::EqAffine;
use halo2_proofs::plonk::{Advice, Circuit, Column, ConstraintSystem, Error, keygen_vk};
use halo2_proofs::poly::commitment::ParamsProver;
use halo2_proofs::poly::ipa::commitment::ParamsIPA;
use policy_constraints::term::hash_node;
use rand_core::OsRng;

/// `NODE_DOMAIN` of policy-constraints' term.rs.
const NODE_DOMAIN: u64 = 0x424d_4e4f_4445_0001;

fn message(tag: Fp, payload: Fp, arity: Fp, children: [Fp; 4]) -> [Fp; 8] {
    [Fp::from(NODE_DOMAIN), tag, payload, arity, children[0], children[1], children[2], children[3]]
}

#[test]
fn the_pse_gadget_computes_the_same_digests() {
    for _ in 0..100 {
        let [tag, payload, arity] = [(); 3].map(|_| Fp::random(OsRng));
        let children = [(); 4].map(|_| Fp::random(OsRng));
        let pse = primitives::Hash::<Fp, P128Pow5T3, ConstantLength<8>, 3, 2>::init()
            .hash(message(tag, payload, arity, children));
        assert_eq!(pse, hash_node(tag, payload, arity, &children));
    }
}

#[derive(Clone)]
struct NodeHash {
    message: Value<[Fp; 8]>,
    expected: Value<Fp>,
}

impl Circuit<Fp> for NodeHash {
    /// The chip's config and its state columns (the chip keeps those private).
    type Config = (Pow5Config<Fp, 3, 2>, [Column<Advice>; 3]);
    type FloorPlanner = SimpleFloorPlanner;
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self { message: Value::unknown(), expected: Value::unknown() }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let state = [(); 3].map(|_| meta.advice_column());
        let partial_sbox = meta.advice_column();
        let rc_a = [(); 3].map(|_| meta.fixed_column());
        let rc_b = [(); 3].map(|_| meta.fixed_column());
        meta.enable_constant(rc_b[0]);
        (Pow5Chip::configure::<P128Pow5T3>(meta, state, partial_sbox, rc_a, rc_b), state)
    }

    fn synthesize(&self, (config, state): Self::Config, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        let words = layouter.assign_region(
            || "message",
            |mut region| {
                let mut words = Vec::new();
                for i in 0..8 {
                    // The message sits in rows 0..3 of the state columns.
                    words.push(region.assign_advice(
                        || "word",
                        state[i % 3],
                        i / 3,
                        || self.message.map(|message| message[i]),
                    )?);
                }
                Ok(words.try_into().unwrap())
            },
        )?;
        let hasher = Hash::<_, _, P128Pow5T3, ConstantLength<8>, 3, 2>::init(
            Pow5Chip::construct(config),
            layouter.namespace(|| "init"),
        )?;
        let digest = hasher.hash(layouter.namespace(|| "hash"), words)?;
        layouter.assign_region(
            || "expected",
            |mut region| {
                let expected = region.assign_advice(|| "expected", state[0], 0, || self.expected)?;
                region.constrain_equal(digest.cell(), expected.cell())
            },
        )
    }
}

#[test]
fn the_pse_chip_proves_a_node_digest() {
    let children = [(); 4].map(|_| Fp::random(OsRng));
    let (tag, payload, arity) = (Fp::from(5), Fp::random(OsRng), Fp::from(4));
    let digest = hash_node(tag, payload, arity, &children);
    let message = message(tag, payload, arity, children);

    let honest = NodeHash { message: Value::known(message), expected: Value::known(digest) };
    assert_eq!(MockProver::run(9, &honest, vec![]).unwrap().verify(), Ok(()));

    let forged = NodeHash { message: Value::known(message), expected: Value::known(digest + Fp::ONE) };
    assert!(MockProver::run(9, &forged, vec![]).unwrap().verify().is_err());

    // The smallest k whose key generation fits one node hash.
    let k = (5..=9)
        .find(|&k| {
            let params: ParamsIPA<EqAffine> = ParamsIPA::new(k);
            keygen_vk(&params, &honest).is_ok()
        })
        .unwrap();
    println!("one node hash fits in k = {k} (≤ {} rows)", 1 << k);
}
