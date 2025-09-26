use std::fs::File;
use anyhow::Result;
use ark_bn254::{Bn254, Fr};
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef, SynthesisError,
};
use ark_r1cs_std::{alloc::AllocVar, fields::fp::FpVar, eq::EqGadget};
use ark_groth16::Groth16;
use ark_snark::{CircuitSpecificSetupSNARK, SNARK};
use rand::rngs::OsRng;
use common;

#[derive(Clone)]
struct AddCircuit {
    x: u64,
    y: u64,
    z: u64,
}

impl AddCircuit {
    fn new() -> Self {
        return AddCircuit {
            x: 0,
            y: 0,
            z: 0,
        };
    }
}

impl ConstraintSynthesizer<Fr> for AddCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // witness-ek (privát bemenetek)
        let x_var = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(self.x)))?;
        let y_var = FpVar::<Fr>::new_witness(cs.clone(), || Ok(Fr::from(self.y)))?;

        // public input (nyilvános bemenet)
        let z_var = FpVar::<Fr>::new_input(cs.clone(), || Ok(Fr::from(self.z)))?;

        // constraint: x + y == z
        let sum = &x_var + &y_var;
        sum.enforce_equal(&z_var)?;
        Ok(())
    }
    
}

fn main() -> Result<()>{
    let mut rng = OsRng;

    // Constraint System inicializálása
    let cs = ConstraintSystem::<Fr>::new_ref();
    let circuit = AddCircuit { x: 3, y: 4, z: 7 };

    circuit.clone().generate_constraints(cs.clone()).map_err(anyhow::Error::msg)?;
    assert!(cs.is_satisfied().map_err(anyhow::Error::msg)?);

    // Public inputok automatikus kinyerése (1 + valós értékek)
    let cs_borrowed = cs.borrow().ok_or_else(|| anyhow::anyhow!("missing value"))?;
    let mut public_inputs: Vec<Fr> = cs_borrowed.instance_assignment.clone();
    drop(cs_borrowed);
    if !public_inputs.is_empty() {
        public_inputs.remove(0);
    }

    println!("Public inputs extracted = {:?}", public_inputs);

    // Trusted Setup
    let (pk, vk) = Groth16::<Bn254>::setup(AddCircuit::new(), &mut rng).unwrap();

    // Proof generálás
    let proof = Groth16::<Bn254>::prove(&pk, circuit.clone(), &mut rng).map_err(anyhow::Error::msg)?;

    let verify_input = common::VerifyInput{
        vk: vk,
        public_inputs: public_inputs,
        proof: proof,
    };

    serde_json::to_writer(File::create("verify.json")?, &verify_input)?;

    
    Ok(())
}

