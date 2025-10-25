use std::fs;
use std::sync::Arc;
use anyhow::Result;
use common::{chips::fact_hash_chip::FactConfig, data::{FactTemplate, GoalEntry, ProofNode, RuleTemplateFileFp, TermFp}, utils_2::common_helpers::MAX_FACTS_HASHES};
use halo2curves::bls12381::Fr;
use num_bigint::BigUint;
use rand_core::OsRng;

use halo2_proofs::{
    pasta::{EqAffine, Fp},
    plonk::{keygen_pk, keygen_vk, ProvingKey, VerifyingKey},
    poly::commitment::Params,
    transcript::{Blake2bWrite, Challenge255},
};

use halo2_gadgets::poseidon::primitives::{
    Hash as PoseidonHash, P128Pow5T3, ConstantLength,
};

use rayon::prelude::*;

use common::{data, data::UnificationInputFp};
use common::unification_checker_circuit::UnificationCircuit;
use common::utils_2::common_helpers::{cpu_rulehash_first_2, hash_rule_template_poseidon, to_fp_value, MAX_ARITY};

mod writer;
use serde::{ser::SerializeTuple, Deserialize};
use writer::{init_output_dir, write_proof};

//Config struct to read the yaml
#[derive(Debug, Deserialize)]
struct Config {
    predicate: String,
    args: Vec<String>,
    salt: String,
}


fn main() -> Result<()> {
    // --- inputok ---
    let config_file = "input/facts.yaml";

    let file_content = fs::read_to_string(config_file)
        .expect("Failed to read the YAML file.");

    let configs: Vec<Config> = serde_yaml::from_str(&file_content)
        .expect("Wring YAML format");

    println!("{:?}", configs);
    let rules_text = fs::read_to_string("input/rules_template.json")?;
    let rules: data::RuleTemplateFile = serde_json::from_str(&rules_text)?;

    let proof_text = fs::read_to_string("input/proof_tree.json")?;
    let tree: Vec<data::ProofNode> = serde_json::from_str(&proof_text)?;

    println!(
        "Loaded {} predicates, {} facts, {} proof nodes.",
        rules.predicates.len(),
        rules.facts.len(),
        tree.len()
    );
    let rules_fp = RuleTemplateFileFp::from(&rules);
    // --- Params + keygen ---

    let params: Params<EqAffine> = Params::new(7);
    let shape = UnificationCircuit {
        rules: rules_fp,
        unif: UnificationInputFp::default(),
    };
    let vk: VerifyingKey<EqAffine> = keygen_vk(&params, &shape)?;
    let pk: ProvingKey<EqAffine> = keygen_pk(&params, vk.clone(), &shape)?;

    let params = Arc::new(params);
    let pk = Arc::new(pk);

    init_output_dir()?;

    // --- minden node-ra proof készítés ---
    println!("aa");
    tree.iter()
    .try_for_each(|node| {
        if let Err(e) = prove_tree(&rules, node, &params,  &pk, &configs) {
            eprintln!("Error on node: {e:?}");
            return Err(e);
        }
        Ok(())
    })?;

    println!("All unification goals proof saved!");
    Ok(())
}

fn prove_tree(
    rules: &data::RuleTemplateFile,
    node: &data::ProofNode,
    params: &Arc<Params<EqAffine>>,
    pk: &Arc<ProvingKey<EqAffine>>,
    facts: &Vec<Config>,
) -> Result<()> {
    if let data::ProofNode::GoalNode(g) = node {
        
        let rules_fp = RuleTemplateFileFp::from(rules);

        let unif_input_fp = unification_input_from_goal(g, facts);

        let public_hashes: Vec<Fp> = facts
            .iter()
            .map(|f| {
                let args_ref: Vec<&str> = f.args.iter().map(|s| s.as_str()).collect();
                fact_hash_native_salted(&f.predicate, &args_ref, &f.salt)
            })
            // kiegészítjük 0-kkal, ha kevesebb a fact, mint MAX_FACTS_HASHES
            .chain(std::iter::repeat(Fp::zero()))
            .take(MAX_FACTS_HASHES)
            .collect();
        // 🔹 Circuit Fp bemenettel
        let circuit = UnificationCircuit {
            rules: rules_fp,
            unif: unif_input_fp,
        };
        // --- proof készítés ---
        let mut transcript = Blake2bWrite::<Vec<u8>, _, Challenge255<_>>::init(vec![]);

        /*let col0: &[Fp] = &public_hashes;
        let instance_cols: [&[Fp]; 1] = [col0];
        let instances: [&[&[Fp]]; 1] = [&instance_cols];

        let instance: Vec<Fp> = vec![]; // 1 publikus input mező
        let instances: Vec<&[Fp]> = vec![instance.as_slice()]; // &[&[Fp]] szintű lista
        let instances: Vec<&[&[Fp]]> = vec![&[]];
        let instance_columns: [&[Fp]; 1] = [&public_hashes];
        let instances: Vec<&[&[Fp]]> = vec![&instance_columns];
        let instances: Vec<&[Fp]> = vec![ &public_hashes[..] ];

        let public_hashes_slice: &[Fp] = &public_hashes;*/


// 2️⃣ Ez már stabil, mert a public_hashes_slice él addig, amíg a függvény véget nem ér
//let instance_cols: [&[Fp]; 1] = [public_hashes_slice];
//let instances: Vec<&[&[Fp]]> = vec![&instance_cols];
let public_hashes_slice: &[Fp] = &public_hashes;        // len == num_public_hashes
let instances: &[&[&[Fp]]] = &[&[public_hashes_slice]]; 
        halo2_proofs::plonk::create_proof(
            params.as_ref(),
            pk.as_ref(),
            &[circuit],
            &instances,
            OsRng,
            &mut transcript,
        )?;
        let proof = transcript.finalize();

        write_proof("unif", &proof, &instances)?;

        // rekurzió
        g.subtree.par_iter()
            .try_for_each(|sub| prove_tree(rules, sub, params, pk, facts))?;
    }
    Ok(())
}



pub fn rlc_encode_cpu(tokens: &[Fp], alpha: Fp) -> Fp {
    let mut acc = Fp::zero();
    for &t in tokens {
        acc = acc * alpha + t;
    }
    acc
}

fn unification_input_from_goal(g: &GoalEntry, facts: &Vec<Config>) -> UnificationInputFp {
    //Creating the goal_name
    let goal_name_termfp = encode_str_to_termfp(&g.goal, facts);
    
    //Creating the subtree goals term list
    let subtree_goals_fp: Vec<TermFp> = g.subtree
        .iter()
        .map(|a|encode_proofnode_to_termfp(a,facts)) 
        .collect();

    UnificationInputFp {
        goal_name: goal_name_termfp, 
        subtree_goals: subtree_goals_fp,
    }
}



/// ✅ Stringből (pl. "ancestor(a,b,c)") készít egy TermFp struktúrát.
/// Ha kevesebb argumentum van, 0-val feltölti MAX_ARITY-ig.
pub fn encode_str_to_termfp(input: &str, facts: &Vec<Config>) -> TermFp {
    // 1️⃣ Szétválasztjuk a név és az argumentumokat
    let open = input.find('(').unwrap_or(input.len());
    let close = input.find(')').unwrap_or(input.len());

    let name_str = input[..open].trim();
    let args_str = if open < close {
        &input[open + 1..close]
    } else {
        ""
    };

    // 2️⃣ Argumentumok listája
    let mut args: Vec<Fp> = args_str
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| to_fp_value(s.trim()))
        .collect();

    // 3️⃣ Padding nullákkal MAX_ARITY-ig
    while args.len() < MAX_ARITY {
        args.push(Fp::zero());
    }
    let name_fp = to_fp_value(name_str);
    let hashed = fact_hash_native_term(&name_fp, &args);


        let salt = facts
            .iter()
            .find_map(|conf| {
                let name = to_fp_value(&conf.predicate);
                let args: Vec<Fp> = conf.args.iter().map(|s| to_fp_value(s)).collect();
                let example_hashed = fact_hash_native_term(&name, &args);
                if hashed == example_hashed {
                    Some(to_fp_value(&conf.salt))
                } else {
                    None
                }
            })
            .unwrap_or(Fp::zero());
    // 4️⃣ TermFp létrehozása
    TermFp {
        name: name_fp,
        args,
        fact_hashes: salt,
    }
}

fn encode_proofnode_to_termfp(n: &ProofNode, facts: &Vec<Config>) -> TermFp {
    match n {
        ProofNode::GoalNode(child) => encode_str_to_termfp(&child.goal, facts),
        // ha van külön bool/leaf variánsod, azt kezeld itt:
        // ProofNode::Bool(true) | ProofNode::LeafTrue => ...
        // A példád alapján a JSON-ban "true" szerepel, ezért:
        _ => TermFp { name: to_fp_value("__TRUE__"), args: vec![Fp::zero(); MAX_ARITY], fact_hashes: Fp::zero()  },
    }
}



#[inline]
fn poseidon_hash2_native(a: Fp, b: Fp) -> Fp {
    // This matches: Hash::<Fp, _, P128Pow5T3, ConstantLength<2>, 3, 2> in-circuit
    PoseidonHash::<Fp, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                .hash([a, b])
}

// Native version of the chip’s `hash_list` folding:
// acc := 0; for v in values { acc = Poseidon(acc, v) } ; return acc
#[inline]
fn poseidon_hash_list_native(values: &[Fp]) -> Fp {
    let mut acc = Fp::zero();
    for &v in values {
        acc = poseidon_hash2_native(acc, v);
    }
    acc
}

/// Public function: hash(name, args, salt) exactly like the chip.
///
/// Inputs:
/// - `name`: predicate/fact name (e.g. "parent")
/// - `args`: predicate args as strings (e.g. ["alice","bob"])
/// - `salt`: Fp salt (convert your BigUint→Fp off-chain the same way you do in-circuit)
///
/// Output:
/// - Fp hash identical to the chip’s Poseidon fold.
pub fn fact_hash_native_salted(name: &str, args: &[&str], salt: &str) -> Fp {
    let mut tokens: Vec<Fp> = Vec::with_capacity(1 + args.len() + 1);
    tokens.push(to_fp_value(name));
    for a in args {
        tokens.push(to_fp_value(a));
    }
    tokens.push(to_fp_value(salt));

    poseidon_hash_list_native(&tokens)
}

pub fn fact_hash_native_term(name: &Fp, args: &[Fp]) -> Fp {
    let mut tokens: Vec<Fp> = Vec::with_capacity(1 + args.len() + 1);
    tokens.push(*name);
    for a in args {
        tokens.push(*a);
    }
    poseidon_hash_list_native(&tokens)
}