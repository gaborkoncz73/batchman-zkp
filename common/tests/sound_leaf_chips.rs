use common::{
    chips::fact_check::{
        external_fact_chip::{ExternalFactChip, ExternalFactConfig},
        sound_builtin_chip::{LegacyBuiltinCells, SoundBuiltinChip, SoundBuiltinConfig},
        u32_range_chip::{U32RangeChip, U32RangeConfig},
    },
    utils_2::common_helpers::{MAX_ARITY, MAX_FACTS_HASHES, MAX_PRED_LIST, to_fp_value},
};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    dev::MockProver,
    pasta::Fp,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error},
};

#[derive(Clone)]
struct RangeCircuit {
    value: Fp,
    enabled: Fp,
}

#[derive(Clone, Debug)]
struct RangeTestConfig {
    input: Column<Advice>,
    range: U32RangeConfig,
}

impl Circuit<Fp> for RangeCircuit {
    type Config = RangeTestConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        self.clone()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let input = meta.advice_column();
        meta.enable_equality(input);
        RangeTestConfig {
            input,
            range: U32RangeChip::configure(meta),
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let (value, enabled) = layouter.assign_region(
            || "range inputs",
            |mut region| {
                let value = region.assign_advice(
                    || "value",
                    config.input,
                    0,
                    || Value::known(self.value),
                )?;
                let enabled = region.assign_advice(
                    || "enabled",
                    config.input,
                    1,
                    || Value::known(self.enabled),
                )?;
                Ok((value, enabled))
            },
        )?;
        U32RangeChip::construct(config.range).assign(
            layouter.namespace(|| "u32 range"),
            &value,
            &enabled,
        )
    }
}

#[test]
fn u32_range_is_algebraic_and_conditional() {
    for circuit in [
        RangeCircuit {
            value: Fp::from(u32::MAX as u64),
            enabled: Fp::one(),
        },
        RangeCircuit {
            value: Fp::from(1_u64 << 32),
            enabled: Fp::zero(),
        },
    ] {
        MockProver::run(7, &circuit, vec![])
            .unwrap()
            .assert_satisfied();
    }
    for circuit in [
        RangeCircuit {
            value: Fp::from(1_u64 << 32),
            enabled: Fp::one(),
        },
        RangeCircuit {
            value: Fp::from(1),
            enabled: Fp::from(2),
        },
    ] {
        assert!(
            MockProver::run(7, &circuit, vec![])
                .unwrap()
                .verify()
                .is_err()
        );
    }
}

#[derive(Clone)]
struct ExternalFactCircuit {
    hash: Fp,
    branch: Fp,
}

#[derive(Clone, Debug)]
struct ExternalFactTestConfig {
    input: Column<Advice>,
    fact: ExternalFactConfig,
}

impl Circuit<Fp> for ExternalFactCircuit {
    type Config = ExternalFactTestConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        self.clone()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let input = meta.advice_column();
        let instance = meta.instance_column();
        meta.enable_equality(input);
        ExternalFactTestConfig {
            input,
            fact: ExternalFactChip::configure(meta, instance),
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let hash = layouter.assign_region(
            || "computed hash",
            |mut region| {
                region.assign_advice(|| "hash", config.input, 0, || Value::known(self.hash))
            },
        )?;
        ExternalFactChip::construct(config.fact).assign(
            layouter.namespace(|| "external membership"),
            &hash,
            Value::known(self.branch),
        )?;
        Ok(())
    }
}

fn fact_registry(member: Option<(usize, Fp)>) -> Vec<Fp> {
    let mut registry = (0..MAX_FACTS_HASHES)
        .map(|index| Fp::from(10_000 + index as u64))
        .collect::<Vec<_>>();
    if let Some((index, value)) = member {
        registry[index] = value;
    }
    registry
}

#[test]
fn external_fact_membership_checks_all_public_slots_and_branch_booleanity() {
    let hash = Fp::from(777);
    for index in [0, MAX_FACTS_HASHES / 2, MAX_FACTS_HASHES - 1] {
        let circuit = ExternalFactCircuit {
            hash,
            branch: Fp::one(),
        };
        MockProver::run(7, &circuit, vec![fact_registry(Some((index, hash)))])
            .unwrap()
            .assert_satisfied();
    }

    let selected_missing = ExternalFactCircuit {
        hash,
        branch: Fp::one(),
    };
    assert!(
        MockProver::run(7, &selected_missing, vec![fact_registry(None)])
            .unwrap()
            .verify()
            .is_err()
    );
    let inactive = ExternalFactCircuit {
        hash,
        branch: Fp::zero(),
    };
    MockProver::run(7, &inactive, vec![fact_registry(None)])
        .unwrap()
        .assert_satisfied();
    let non_boolean = ExternalFactCircuit {
        hash,
        branch: Fp::from(2),
    };
    assert!(
        MockProver::run(7, &non_boolean, vec![fact_registry(Some((0, hash)))])
            .unwrap()
            .verify()
            .is_err()
    );
}

#[derive(Clone)]
struct BuiltinCircuit {
    names: Vec<Fp>,
    arities: Vec<Fp>,
    args: Vec<Vec<Vec<Fp>>>,
    branch: Fp,
}

#[derive(Clone, Debug)]
struct BuiltinTestConfig {
    input: Column<Advice>,
    builtin: SoundBuiltinConfig,
}

impl Circuit<Fp> for BuiltinCircuit {
    type Config = BuiltinTestConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        self.clone()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let input = meta.advice_column();
        meta.enable_equality(input);
        BuiltinTestConfig {
            input,
            builtin: SoundBuiltinChip::configure(meta),
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let (names, arities, args) = layouter.assign_region(
            || "legacy builtin inputs",
            |mut region| {
                let mut row = 0;
                let mut names = Vec::new();
                let mut arities = Vec::new();
                for value in &self.names {
                    names.push(region.assign_advice(
                        || "name",
                        config.input,
                        row,
                        || Value::known(*value),
                    )?);
                    row += 1;
                }
                for value in &self.arities {
                    arities.push(region.assign_advice(
                        || "arity",
                        config.input,
                        row,
                        || Value::known(*value),
                    )?);
                    row += 1;
                }
                let mut args = Vec::new();
                for predicate in &self.args {
                    let mut argument_cells = Vec::new();
                    for argument in predicate {
                        let mut list_cells = Vec::new();
                        for value in argument {
                            list_cells.push(region.assign_advice(
                                || "argument slot",
                                config.input,
                                row,
                                || Value::known(*value),
                            )?);
                            row += 1;
                        }
                        argument_cells.push(list_cells);
                    }
                    args.push(argument_cells);
                }
                Ok((names, arities, args))
            },
        )?;
        SoundBuiltinChip::construct(config.builtin).assign(
            layouter.namespace(|| "sound builtin"),
            LegacyBuiltinCells {
                names: &names,
                arities: &arities,
                args: &args,
            },
            Value::known(self.branch),
        )?;
        Ok(())
    }
}

fn builtin(relation: &str, lhs: u64, rhs: u64, operations: &[(&str, u64)]) -> BuiltinCircuit {
    let mut names = vec![Fp::zero(); MAX_PRED_LIST];
    let mut arities = vec![Fp::zero(); MAX_PRED_LIST];
    let mut args = vec![vec![vec![Fp::zero(); MAX_PRED_LIST]; MAX_ARITY]; MAX_PRED_LIST];
    names[0] = to_fp_value(relation);
    arities[0] = Fp::from(2);
    args[0] = vec![vec![-Fp::one(); MAX_PRED_LIST]; MAX_ARITY];
    args[0][0][0] = Fp::from(lhs);
    args[0][1][0] = Fp::from(rhs);
    for (predicate, (operator, operand)) in operations.iter().enumerate() {
        let predicate = predicate + 1;
        names[predicate] = to_fp_value(operator);
        arities[predicate] = Fp::one();
        args[predicate] = vec![vec![-Fp::one(); MAX_PRED_LIST]; MAX_ARITY];
        args[predicate][0][0] = Fp::from(*operand);
    }
    BuiltinCircuit {
        names,
        arities,
        args,
        branch: Fp::one(),
    }
}

fn builtin_verifies(circuit: &BuiltinCircuit) -> bool {
    MockProver::run(10, circuit, vec![])
        .unwrap()
        .verify()
        .is_ok()
}

#[test]
fn builtin_equality_and_all_arithmetic_operations_are_constrained() {
    assert!(builtin_verifies(&builtin("=", 9, 9, &[])));
    assert!(!builtin_verifies(&builtin("=", 9, 8, &[])));
    let mut later_slot_mismatch = builtin("=", 9, 9, &[]);
    later_slot_mismatch.args[0][0][1] = Fp::from(1);
    later_slot_mismatch.args[0][1][1] = Fp::from(2);
    assert!(!builtin_verifies(&later_slot_mismatch));
    for circuit in [
        builtin("is", 7, 2, &[("+", 5)]),
        builtin("is", 3, 8, &[("-", 5)]),
        builtin("is", 10, 2, &[("*", 5)]),
        builtin("is", 2, 7, &[("div", 3)]),
        builtin("is", 20, 2, &[("+", 3), ("*", 4)]),
    ] {
        assert!(builtin_verifies(&circuit));
    }
    assert!(!builtin_verifies(&builtin("is", 9, 2, &[("*", 5)])));
    assert!(!builtin_verifies(&builtin("is", 0, 1, &[("div", 0)])));
}

#[test]
fn builtin_comparisons_use_unsigned_integer_order() {
    for circuit in [
        builtin("<", 1, 2, &[]),
        builtin("<=", 2, 2, &[]),
        builtin(">", 2, 1, &[]),
        builtin(">=", 2, 2, &[]),
    ] {
        assert!(builtin_verifies(&circuit));
    }
    for circuit in [
        builtin("<", 2, 1, &[]),
        builtin("<=", 2, 1, &[]),
        builtin(">", 1, 2, &[]),
        builtin(">=", 1, 2, &[]),
    ] {
        assert!(!builtin_verifies(&circuit));
    }
}

#[test]
fn builtin_rejects_overflow_underflow_unknown_opcodes_and_bad_branch_bits() {
    assert!(!builtin_verifies(&builtin(
        "is",
        0,
        u32::MAX as u64,
        &[("+", 1)],
    )));
    assert!(!builtin_verifies(&builtin("is", 0, 0, &[("-", 1)])));
    assert!(!builtin_verifies(&builtin(
        "is",
        0,
        1_u64 << 16,
        &[("*", 1_u64 << 16)],
    )));
    let mut unknown = builtin("is", 2, 2, &[]);
    unknown.names[1] = to_fp_value("unknown");
    unknown.arities[1] = Fp::one();
    assert!(!builtin_verifies(&unknown));
    let mut gap = builtin("is", 7, 2, &[("+", 5)]);
    gap.names.swap(1, 2);
    gap.arities.swap(1, 2);
    gap.args.swap(1, 2);
    assert!(!builtin_verifies(&gap));
    let mut non_boolean = builtin("=", 1, 1, &[]);
    non_boolean.branch = Fp::from(2);
    assert!(!builtin_verifies(&non_boolean));
    let mut inactive_unknown = unknown;
    inactive_unknown.branch = Fp::zero();
    assert!(builtin_verifies(&inactive_unknown));
}
