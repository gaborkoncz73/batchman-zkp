//! Regression tests for the production typed matrix/dot-product argument.
//!
//! `MockProver` checks lookup tuple components individually. The real backend
//! compresses the same tuple with transcript `theta` after advice commitment.

use common::chips::typed_variable_dot_chip::{
    TypedVariableCandidate, TypedVariableDotChip, TypedVariableDotConfig, TypedVariableEquality,
};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    dev::MockProver,
    pasta::Fp,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error},
};

#[derive(Clone, Debug)]
struct DotCase {
    flag: Fp,
    equalities: Vec<(Fp, Fp)>,
}

#[derive(Clone, Debug)]
struct TestCircuit {
    candidates: Vec<DotCase>,
}

#[derive(Clone, Debug)]
struct TestConfig {
    input: Column<Advice>,
    dot: TypedVariableDotConfig,
}

impl Circuit<Fp> for TestCircuit {
    type Config = TestConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            candidates: self
                .candidates
                .iter()
                .map(|case| DotCase {
                    flag: Fp::zero(),
                    equalities: vec![(Fp::zero(), Fp::zero()); case.equalities.len()],
                })
                .collect(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let input = meta.advice_column();
        let flag = meta.advice_column();
        let left = meta.advice_column();
        let right = meta.advice_column();
        for column in [input, flag, left, right] {
            meta.enable_equality(column);
        }
        let dot = TypedVariableDotChip::configure_with_columns(meta, flag, left, right);
        TestConfig { input, dot }
    }

    fn synthesize(&self, cfg: Self::Config, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        let chip = TypedVariableDotChip::construct(cfg.dot);
        chip.load_zero_table(layouter.namespace(|| "zero table"))?;
        let candidates = layouter.assign_region(
            || "authenticated typed occurrence cells",
            |mut region| {
                let mut row = 0;
                let mut assigned = Vec::with_capacity(self.candidates.len());
                for (candidate_index, candidate) in self.candidates.iter().enumerate() {
                    let flag = region.assign_advice(
                        || format!("constrained candidate flag[{candidate_index}]"),
                        cfg.input,
                        row,
                        || Value::known(candidate.flag),
                    )?;
                    row += 1;
                    let mut equalities = Vec::with_capacity(candidate.equalities.len());
                    for (equality_index, (left, right)) in
                        candidate.equalities.iter().copied().enumerate()
                    {
                        let left = region.assign_advice(
                            || format!("typed left digest[{candidate_index}][{equality_index}]"),
                            cfg.input,
                            row,
                            || Value::known(left),
                        )?;
                        row += 1;
                        let right = region.assign_advice(
                            || format!("typed right digest[{candidate_index}][{equality_index}]"),
                            cfg.input,
                            row,
                            || Value::known(right),
                        )?;
                        row += 1;
                        equalities.push(TypedVariableEquality::new(left, right));
                    }
                    assigned.push(TypedVariableCandidate { flag, equalities });
                }
                Ok(assigned)
            },
        )?;
        chip.assign_candidates(layouter.namespace(|| "matrix dot products"), &candidates)
    }
}

fn verify(candidates: Vec<DotCase>) -> Result<(), Vec<halo2_proofs::dev::VerifyFailure>> {
    MockProver::run(8, &TestCircuit { candidates }, vec![])
        .unwrap()
        .verify()
}

#[test]
fn selected_equal_occurrences_pass() {
    assert_eq!(
        verify(vec![DotCase {
            flag: Fp::one(),
            equalities: vec![(Fp::from(7), Fp::from(7)), (Fp::from(9), Fp::from(9))],
        }]),
        Ok(())
    );
}

#[test]
fn inactive_candidate_does_not_constrain_occurrences() {
    assert_eq!(
        verify(vec![DotCase {
            flag: Fp::zero(),
            equalities: vec![(Fp::from(7), Fp::from(100))],
        }]),
        Ok(())
    );
}

#[test]
fn one_selected_mismatch_is_rejected() {
    assert!(
        verify(vec![DotCase {
            flag: Fp::one(),
            equalities: vec![(Fp::from(7), Fp::from(8))],
        }])
        .is_err()
    );
}

#[test]
fn opposite_residuals_cannot_cancel_as_an_unweighted_dot_product() {
    assert!(
        verify(vec![DotCase {
            flag: Fp::one(),
            equalities: vec![(Fp::from(8), Fp::from(7)), (Fp::from(7), Fp::from(8)),],
        }])
        .is_err()
    );
}

#[test]
fn selected_and_inactive_candidates_are_independent() {
    assert_eq!(
        verify(vec![
            DotCase {
                flag: Fp::zero(),
                equalities: vec![(Fp::from(1), Fp::from(2))],
            },
            DotCase {
                flag: Fp::one(),
                equalities: vec![(Fp::from(42), Fp::from(42))],
            },
        ]),
        Ok(())
    );
}
