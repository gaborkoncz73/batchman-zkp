//! How many rows a circuit uses, and the smallest `k` that fits it.
//!
//! The floor planner is run against an [`Assignment`] that only records the
//! highest row any cell, selector or constant lands on.

use ff::Field;
use halo2_proofs::circuit::Value;
use halo2_proofs::plonk::{
    Advice, Any, Assigned, Assignment, Challenge, Circuit, Column, ConstraintSystem, Error, Fixed,
    FloorPlanner, Instance, Selector,
};

#[derive(Default)]
struct RowCounter {
    rows: usize,
}

impl RowCounter {
    fn touch(&mut self, row: usize) {
        self.rows = self.rows.max(row + 1);
    }
}

impl<F: Field> Assignment<F> for RowCounter {
    fn enter_region<NR: Into<String>, N: FnOnce() -> NR>(&mut self, _: N) {}

    fn annotate_column<A: FnOnce() -> AR, AR: Into<String>>(&mut self, _: A, _: Column<Any>) {}

    fn exit_region(&mut self) {}

    fn enable_selector<A: FnOnce() -> AR, AR: Into<String>>(
        &mut self,
        _: A,
        _: &Selector,
        row: usize,
    ) -> Result<(), Error> {
        self.touch(row);
        Ok(())
    }

    fn query_instance(&self, _: Column<Instance>, _: usize) -> Result<Value<F>, Error> {
        Ok(Value::unknown())
    }

    fn assign_advice<V, VR, A, AR>(
        &mut self,
        _: A,
        _: Column<Advice>,
        row: usize,
        _: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.touch(row);
        Ok(())
    }

    fn assign_fixed<V, VR, A, AR>(
        &mut self,
        _: A,
        _: Column<Fixed>,
        row: usize,
        _: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.touch(row);
        Ok(())
    }

    fn copy(&mut self, _: Column<Any>, _: usize, _: Column<Any>, _: usize) -> Result<(), Error> {
        Ok(())
    }

    /// Lookup tables are padded to the last usable row; that is not a use.
    fn fill_from_row(
        &mut self,
        _: Column<Fixed>,
        _: usize,
        _: Value<Assigned<F>>,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn get_challenge(&self, _: Challenge) -> Value<F> {
        Value::unknown()
    }

    fn push_namespace<NR: Into<String>, N: FnOnce() -> NR>(&mut self, _: N) {}

    fn pop_namespace(&mut self, _: Option<String>) {}
}

/// The number of rows `circuit` assigns (cells, selectors and constants).
pub fn rows_used<F: Field, C: Circuit<F>>(circuit: &C) -> usize {
    let mut meta = ConstraintSystem::default();
    let config = C::configure(&mut meta);
    let mut counter = RowCounter::default();
    C::FloorPlanner::synthesize(&mut counter, circuit, config, meta.constants().clone())
        .expect("the circuit synthesizes");
    counter.rows
}

/// The smallest `k` whose `2^k` rows hold the used rows and the prover's
/// blinding rows.
pub fn minimum_k<F: Field, C: Circuit<F>>(circuit: &C) -> u32 {
    let mut meta = ConstraintSystem::<F>::default();
    C::configure(&mut meta);
    let needed = (rows_used(circuit) + meta.blinding_factors() + 1).max(meta.minimum_rows());
    needed.next_power_of_two().trailing_zeros()
}
