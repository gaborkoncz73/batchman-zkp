use common::{
    chips::fact_check::poseidon_hash::{PoseidonHashChip, PoseidonHashConfig},
    typed_fact::typed_fact_hash,
};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    dev::MockProver,
    pasta::Fp,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance},
};
use halo2curves::ff::Field;

const TEST_K: u32 = 8;

#[derive(Clone)]
struct TypedFactHashCircuit {
    goal_digest: Fp,
    salt: Fp,
}

#[derive(Clone, Debug)]
struct TypedFactHashConfig {
    input: Column<Advice>,
    instance: Column<Instance>,
    poseidon: PoseidonHashConfig,
}

impl Circuit<Fp> for TypedFactHashCircuit {
    type Config = TypedFactHashConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            goal_digest: Fp::ZERO,
            salt: Fp::ZERO,
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let input = meta.advice_column();
        let instance = meta.instance_column();
        meta.enable_equality(input);
        meta.enable_equality(instance);
        let poseidon = PoseidonHashChip::configure(meta);
        TypedFactHashConfig {
            input,
            instance,
            poseidon,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let [goal_digest, salt] = layouter.assign_region(
            || "typed fact witness",
            |mut region| {
                Ok([
                    region.assign_advice(
                        || "typed goal digest",
                        config.input,
                        0,
                        || Value::known(self.goal_digest),
                    )?,
                    region.assign_advice(
                        || "fact salt",
                        config.input,
                        1,
                        || Value::known(self.salt),
                    )?,
                ])
            },
        )?;
        let hash = PoseidonHashChip::construct(config.poseidon).hash_typed_fact(
            layouter.namespace(|| "typed fact hash"),
            &goal_digest,
            &salt,
        )?;
        layouter.constrain_instance(hash.cell(), config.instance, 0)
    }
}

fn verifies(goal_digest: Fp, salt: Fp, expected: Fp) -> bool {
    let circuit = TypedFactHashCircuit { goal_digest, salt };
    MockProver::run(TEST_K, &circuit, vec![vec![expected]])
        .unwrap()
        .verify()
        .is_ok()
}

#[test]
fn native_and_circuit_typed_fact_hashes_match() {
    for (goal_digest, salt) in [
        (Fp::from(7), Fp::from(11)),
        (Fp::from(123_456), Fp::from(987_654)),
        (Fp::ZERO, Fp::ONE),
    ] {
        assert!(verifies(
            goal_digest,
            salt,
            typed_fact_hash(goal_digest, salt)
        ));
    }
}

#[test]
fn typed_fact_hash_binds_goal_digest_salt_and_public_result() {
    let goal_digest = Fp::from(41);
    let salt = Fp::from(73);
    let expected = typed_fact_hash(goal_digest, salt);

    assert!(verifies(goal_digest, salt, expected));
    assert!(!verifies(goal_digest + Fp::ONE, salt, expected));
    assert!(!verifies(goal_digest, salt + Fp::ONE, expected));
    assert!(!verifies(goal_digest, salt, expected + Fp::ONE));
}
