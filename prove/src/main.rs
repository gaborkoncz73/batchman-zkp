
use std::fs::File;
use anyhow::Result;
use ark_relations::lc;
use ark_snark::{CircuitSpecificSetupSNARK, SNARK};
use serde_json::Value;
use indexmap::IndexSet;
use ark_ff::{ BigInteger256};
use num_bigint::BigInt;
use num_traits::{ One};
use ark_bn254::Fr;
use ark_ff::{PrimeField, Zero};
use light_poseidon::{Poseidon, PoseidonHasher};
use light_poseidon::parameters::bn254_x5::get_poseidon_parameters;

use ark_bn254::{Bn254};
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef, LinearCombination, SynthesisError, Variable};
use ark_std::rand::thread_rng;
use ark_groth16::{prepare_verifying_key};

/// One line vector (unificational constraint)
#[derive(Clone, Debug)]
pub struct RowEntry {
    /// Weights of the variables ( [1, -1, 0, 0, const])
    pub coeffs: Vec<BigInt>,
    /// description of the unfication (pl. parent)
    pub desc: String,
}

#[derive(Clone, Debug)]
pub struct DotProductCircuit {
    pub rows: Vec<RowEntry>,
    pub witness: Vec<BigInt>,
}

impl DotProductCircuit {
    /// Create an empty system
    pub fn new() -> Self {
        Self {
            rows: vec![],
            witness: vec![],
        }
    }

    /// Add a new unification constraint (row)
    pub fn add_row(&mut self, coeffs: Vec<BigInt>, desc: &str) {
        self.rows.push(RowEntry {
            coeffs,
            desc: desc.to_string(),
        });
    }

    /// Set witness vector (Poseidon-hashed entities)
    pub fn set_witness(&mut self, witness: Vec<BigInt>) {
        self.witness = witness;
    }

    /// Check all constraints in plain arithmetic (off-chain check)
    pub fn verify_constraints(&self) -> bool {
        for (i, row) in self.rows.iter().enumerate() {
            let dot = self.dot_product(&row.coeffs, &self.witness);
            if !dot.is_zero() {
                println!("❌ Row {} failed: {}", i, row.desc);
                return false;
            }
        }
        println!("All constraints satisfied!");
        true
    }

    /// Compute dot product (Σ ai·bi)
    fn dot_product(&self, a: &[BigInt], b: &[BigInt]) -> BigInt {
        assert_eq!(a.len(), b.len(), "Mismatched vector sizes");
        let mut acc = BigInt::zero();
        for (ai, bi) in a.iter().zip(b.iter()) {
            acc += ai * bi;
        }
        acc
    }
}

/*impl ConstraintSynthesizer<Fr> for DotProductCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // === 1. Witness BigInt -> mezőváltozók ===
        let n = self.witness.len();
        let mut vars: Vec<Variable> = Vec::with_capacity(n);

        for (i, w) in self.witness.iter().enumerate() {
            let f = big_to_fr(w);
            let var = cs.new_witness_variable(|| Ok(f)).map_err(|e| {
                eprintln!("Hiba a {}. witness változónál: {:?}", i, e);
                e
            })?;
            vars.push(var);
        }

        // === 2. Minden sor: Σ(a_i * x_i) = 0 ===
        for (i, row) in self.rows.iter().enumerate() {
            // A sor és a witness hosszának egyeznie kell
            assert_eq!(
                row.coeffs.len(),
                n,
                "A sor hossza nem egyezik a witness hosszával: row {}",
                i
            );

            // Szorzatok összege: sum = Σ(a_i * x_i)
            let mut sum = LinearCombination::<Fr>::zero();

            for (j, coeff) in row.coeffs.iter().enumerate() {
                let a_f = big_to_fr(coeff);
                if !a_f.is_zero() {
                    sum = sum + (a_f, vars[j]);
                }
            }

            // === Pontosan ugyanaz, mint a dot_product == 0 feltétel ===
            // enforce: sum = 0   →   sum * 1 = 0
            cs.enforce_constraint(
                sum.clone(),
                LinearCombination::from(Variable::One),
                LinearCombination::zero(),
            )
            .map_err(|e| {
                eprintln!("Hiba a {}. sorban ('{}'): {:?}", i, row.desc, e);
                e
            })?;
        }

        Ok(())
    }
}*/

/// Convert BigInt → Fr (mod p)
/*fn big_to_fr(x: &BigInt) -> Fr {
    let mut bytes = x.to_signed_bytes_le();
    if bytes.is_empty() {
        bytes.push(0u8);
    }
    Fr::from_le_bytes_mod_order(&bytes)
}*/



fn main() -> Result<()> {
    // Reading the Proof Tree
    let file = File::open("proof_tree.json")?;
    let json: Value = serde_json::from_reader(file)?;
    let arr = json
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("expected top-level array"))?;

    // Getting the values of the X Y etc variables
    let mut entities: IndexSet<String> = IndexSet::new();
    collect_entities(arr, &mut entities);
    println!("Variables = {:?}", entities);

    let n = entities.len();

    // Collecting the unificational rows
    let mut rows: Vec<RowEntry> = Vec::new();
    collect_rows(arr,n, &mut rows);

    println!("Rows ({} db):", rows.len());
    for row in &rows{
        println!("{:?}", row);
    }

    // Witness with poseidon hashed values
    let w_vals = generate_w_vals(&entities);
    println!("w_vals = {:?}", w_vals);

    // === 5️⃣ ConstraintSystem előkészítése ===
    let mut cs_data = DotProductCircuit::new();
    for row in rows {
        cs_data.add_row(row.coeffs, &row.desc);
    }
    cs_data.set_witness(w_vals);

    println!("Satisfied: {:?}", cs_data.verify_constraints());

    // === 6️⃣ Arkworks R1CS létrehozása ===
    /*let cs_ref = ark_relations::r1cs::ConstraintSystem::<Fr>::new_ref();
    cs_data.clone().generate_constraints(cs_ref.clone())?;
    assert!(cs_ref.is_satisfied().unwrap(), "Constraints not satisfied!");

    println!("All constraints satisfied locally, generating ZKP...");

    let mut rng = thread_rng();
    let (pk, vk) = Groth16::<Bn254>::setup(cs_data.clone(), &mut rng)?;

    let proof = Groth16::<Bn254>::prove(&pk, cs_data.clone(), &mut rng)?;

    let cs_borrowed = cs_ref.borrow().unwrap();
    let mut public_inputs: Vec<Fr> = cs_borrowed.instance_assignment.clone();
    drop(cs_borrowed);
    if !public_inputs.is_empty() {
        public_inputs.remove(0); // az első elem a konstans 1
    }

    println!("Public inputs extracted = {:?}", public_inputs);

    let verify_input = common::VerifyInput {
        vk: vk,
        public_inputs,
        proof,
    };

    serde_json::to_writer(File::create("verify.json")?, &verify_input)?;
    println!("verify.json successfully exported!");*/

    Ok(())
}

// Proof tree process

fn collect_entities(nodes: &[Value], out: &mut IndexSet<String>) {
    for node in nodes {
        if let Some(term) = node.get("goal_term") {
            if let Some(args) = term.get("args").and_then(|a| a.as_array()) {
                for arg in args {
                    if let Some(s) = arg.as_str() {
                        out.insert(s.to_string());
                    }
                }
            }
        }
        if let Some(subtree) = node.get("subtree").and_then(|a| a.as_array()) {
            collect_entities(subtree, out);
        }
    }
}

/// Every goal_term(args) is one line (n+1 length: n variables + 1 constant)
fn collect_rows(
    nodes: &[serde_json::Value],
    n: usize,
    rows: &mut Vec<RowEntry>,
) {
    fn traverse(
        node: &serde_json::Value,
        var_order: &mut Vec<String>,
        n: usize,
        rows: &mut Vec<RowEntry>,
    ) {
        if let Some(term) = node.get("goal_term") {
            if let Some(args) = term.get("args").and_then(|a| a.as_array()) {
                // regisztráljuk a változókat az első előfordulás sorrendjében
                for arg in args {
                    if let Some(s) = arg.as_str() {
                        if !var_order.contains(&s.to_string()) {
                            var_order.push(s.to_string());
                        }
                    }
                }

                // new line inicialization n+1 elements (n variable + constant)
                let mut coeffs = vec![BigInt::zero(); n + 1];
                let mut const_sum = BigInt::zero();

                // every argument 1, plus the sum of the poseidon hashes
                for arg in args {
                    if let Some(s) = arg.as_str() {
                        if let Some(idx) = var_order.iter().position(|x| x == s) {
                            coeffs[idx] = BigInt::one();
                            const_sum += poseidon_entity(s);
                        }
                    }
                }

                // negative sum to the constant place
                coeffs[n] = -const_sum;

                // predicate descriptiion
                let pred = term
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                rows.push(RowEntry {
                    desc: pred,
                    coeffs,
                });
            }
        }

        // recursive traverse for the subtrees
        if let Some(subtree) = node.get("subtree").and_then(|a| a.as_array()) {
            for child in subtree {
                traverse(child, var_order, n, rows);
            }
        }
    }

    let mut var_order: Vec<String> = Vec::new();
    for node in nodes {
        traverse(node, &mut var_order, n, rows);
    }
}


// Poseidon mapping
fn poseidon_entity(name: &str) -> BigInt {
    let params = get_poseidon_parameters::<Fr>(2).expect("failed to get BN254 Poseidon parameters");
    let mut hasher = Poseidon::<Fr>::new(params);
    let n = Fr::from_le_bytes_mod_order(name.as_bytes());
    bigint_from_fr(&hasher.hash(&[n]).expect("Poseidon hash failed"))
}

fn generate_w_vals(index_map: &IndexSet<String>) -> Vec<BigInt> {
    let mut w = vec![BigInt::one(); index_map.len()+1];
    for (idx, name) in index_map.iter().enumerate() {
        w[idx] = poseidon_entity(name);
    }
    w
}

fn bigint_from_fr(fr: &Fr) -> BigInt {
    let bi: BigInteger256 = fr.into_bigint();

    // Convert little-endian limbs (4 × u64) into a BigInt
    let mut acc = BigInt::zero();
    let mut factor = BigInt::one();
    for limb in bi.0.iter() {
        acc += BigInt::from(*limb) * &factor;
        factor <<= 64; // multiply by 2^64 for next limb
    }
    acc
}
