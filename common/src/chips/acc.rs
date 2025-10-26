/*use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, SimpleFloorPlanner, Value},
    pasta::Fp,
    plonk::{Circuit, Column, ConstraintSystem, Error, Instance},
};
use snark_verifier::{
    loader::{Halo2Loader, Loader}, pcs::AccumulationScheme, util::arithmetic::CurveAffine, verifier::{plonk::{PlonkProof, PlonkSuccinctVerifier}, SnarkVerifier}
};

use std::marker::PhantomData;

/// Recursive verifier chip
#[derive(Clone, Debug)]
pub struct RecursiveVerifierChip<C, L, AS>
where
    C: CurveAffine,
    L: Loader<C>,
    AS: AccumulationScheme<C, L>,
{
    pub acc_cell: AssignedCell<Fp, Fp>, // scalar cell in your Fp circuit
    phantom: PhantomData<(C, L, AS)>,
}

#[derive(Clone, Debug)]
pub struct RecursiveVerifierConfig {
    pub acc_column: Column<Instance>, // or Advice if you prefer
}

impl<C, L, AS> Chip<Fp> for RecursiveVerifierChip<C, L, AS>
where
    C: CurveAffine,
    L: Loader<C>,
    AS: AccumulationScheme<C, L>,
{
    type Config = RecursiveVerifierConfig;
    type Loaded = AssignedCell<Fp, Fp>;

    fn config(&self) -> &Self::Config {
        unimplemented!()
    }

    fn loaded(&self) -> &Self::Loaded {
        &self.acc_cell
    }
}

impl<C, L, AS> RecursiveVerifierChip<C, L, AS>
where
    C: CurveAffine,
    L: Loader<C>,
    AS: AccumulationScheme<C, L>,
{
    pub fn construct(cfg: RecursiveVerifierConfig, acc_cell: AssignedCell<Fp, Fp>) -> Self {
        Self {
            acc_cell,
            phantom: PhantomData,
        }
    }

    /// Assign a new accumulator value
    pub fn assign_accumulator(
        &self,
        mut layouter: impl Layouter<Fp>,
        value: Value<Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        layouter.assign_region(
            || "assign accumulator",
            |mut region| {
                let cell = region.assign_advice(self.acc_cell.column(), 0, value)?;
                Ok(cell)
            },
        )
    }

    /// Verify a pre-existing Halo2 proof inside this circuit
    pub fn verify_proof(
        &self,
        layouter: impl Layouter<Fp>,
        vk: &<AS as AccumulationScheme<C, L>>::VerifyingKey,
        protocol: &snark_verifier::verifier::plonk::PlonkProtocol<C, L>,
        instances: &[Vec<L::LoadedScalar>],
        proof: &PlonkProof<C, L, AS>,
    ) -> Result<(), Error> {
        // Create a loader for curve points in-circuit
        let loader = Halo2Loader::<C, YourEccChip>::new(layouter);

        // Call the succinct verifier
        let accumulators =
            PlonkSuccinctVerifier::<AS>::verify(vk, protocol, instances, proof)
                .map_err(|_| Error::Synthesis)?;

        // You can now assign these accumulator scalars in your circuit
        for acc in accumulators {
            let _assigned_acc = self.assign_accumulator(loader.layouter(), acc)?;
        }

        Ok(())
    }
}

// Example usage in a circuit
#[derive(Debug, Clone)]
pub struct RecursiveCircuit<C, L, AS>
where
    C: CurveAffine,
    L: Loader<C>,
    AS: AccumulationScheme<C, L>,
{
    pub recursive_input: Fp,
    phantom: PhantomData<(C, L, AS)>,
}

impl<C, L, AS> Circuit<Fp> for RecursiveCircuit<C, L, AS>
where
    C: CurveAffine,
    L: Loader<C>,
    AS: AccumulationScheme<C, L>,
{
    type Config = RecursiveVerifierConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            recursive_input: Fp::zero(),
            phantom: PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let acc_column = meta.instance_column();
        meta.enable_equality(acc_column);
        RecursiveVerifierConfig { acc_column }
    }

    fn synthesize(
        &self,
        cfg: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let acc_cell = layouter.assign_region(
            || "assign input accumulator",
            |mut region| {
                region.assign_advice(cfg.acc_column, 0, Value::known(self.recursive_input))
            },
        )?;

        let chip = RecursiveVerifierChip::<C, L, AS>::construct(cfg, acc_cell);

        // Later in circuit: call `chip.verify_proof(...)` for each proof to verify recursively

        Ok(())
    }
}
*/