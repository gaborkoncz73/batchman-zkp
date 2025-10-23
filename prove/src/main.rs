use std::fs;
use std::sync::Arc;
use anyhow::Result;
use common::data::{GoalEntry, ProofNode, RuleTemplateFileFp, TermFp};
use rand_core::OsRng;

use halo2_proofs::{
    pasta::{EqAffine, Fp},
    plonk::{keygen_pk, keygen_vk, ProvingKey, VerifyingKey},
    poly::commitment::Params,
    transcript::{Blake2bWrite, Challenge255},
};
use rayon::prelude::*;

use common::{data, data::UnificationInputFp};
use common::unification_checker_circuit::UnificationCircuit;
use common::utils_2::common_helpers::{cpu_rulehash_first_2, hash_rule_template_poseidon, to_fp_value, MAX_ARITY};

mod writer;
use writer::{init_output_dir, write_proof};

fn main() -> Result<()> {
    // --- inputok ---
    let rules_text = fs::read_to_string("input/rules_template.json")?;
    let rules_text2 = fs::read_to_string("input/rules_template.json")?;
    let rules: data::RuleTemplateFile = serde_json::from_str(&rules_text)?;
    let rules2: data::RuleTemplateFile = serde_json::from_str(&rules_text2)?;



    let expected = cpu_rulehash_first_2(&rules);


    let proof_text = fs::read_to_string("input/proof_tree.json")?;
    let tree: Vec<data::ProofNode> = serde_json::from_str(&proof_text)?;

    let result2 = hash_rule_template_poseidon(&rules);
    let result3 = hash_rule_template_poseidon(&rules2);
    println!("Poseidon hash (identical to circuit) = {:?}", result2);
    println!("Poseidon hash (identical to circuit) = {:?}", result3);

    println!(
        "Loaded {} predicates, {} facts, {} proof nodes.",
        rules.predicates.len(),
        rules.facts.len(),
        tree.len()
    );
    let rules_fp = RuleTemplateFileFp::from(&rules);
    // --- Params + keygen ---
    let params: Params<EqAffine> = Params::new(8);
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
    tree.iter()
    .try_for_each(|node| {
        if let Err(e) = prove_tree(&rules2, node, &params,  &pk, &expected) {
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
    res: &Fp,
) -> Result<()> {
    if let data::ProofNode::GoalNode(g) = node {
        /*let unif_input = UnificationInput {
            goal_name: g.goal.clone(),
            goal_term_args: g.goal_term.args.clone(),
            goal_term_name: g.goal_term.name.clone(),
            unif_body: g.goal_unification.body.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect(),
            unif_goal: g.goal_unification.goal.clone(),
            substitution: g.substitution.clone(),
            subtree_goals: g.subtree.iter()
                .filter_map(|n| match n {
                    data::ProofNode::GoalNode(child) => Some(child.goal.clone()),
                    _ => None,
                })
                .collect(),
        };

        let circuit = UnificationCircuit { rules: rules.clone(), unif: unif_input };*/
        // 🔹 Közvetlen konverzió a ProofNode → Fp inputra
        let rules_fp = RuleTemplateFileFp::from(rules);

        let unif_input_fp = unification_input_from_goal(g);

        // 🔹 Circuit Fp bemenettel
        let circuit = UnificationCircuit {
            rules: rules_fp,
            unif: unif_input_fp,
        };
        let instances: Vec<&[&[Fp]]> = vec![&[]];
        // --- proof készítés ---
        let mut transcript = Blake2bWrite::<Vec<u8>, _, Challenge255<_>>::init(vec![]);
        //let instance = vec![]; // 1 publikus input mező
        //let instances = vec![instance.as_slice()]; // &[&[Fp]] szintű lista
        halo2_proofs::plonk::create_proof(
            params.as_ref(),
            pk.as_ref(),
            &[circuit],
            &instances,
            OsRng,
            &mut transcript,
        )?;
        let proof = transcript.finalize();

        write_proof("unif", &proof, None)?;

        // rekurzió
        g.subtree.iter()
            .try_for_each(|sub| prove_tree(rules, sub, params, pk, res))?;
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

fn unification_input_from_goal(g: &GoalEntry) -> UnificationInputFp {
    //Creating the goal_name
    let goal_name_termfp = encode_str_to_termfp(&g.goal);
    
    //Creating the goal_term parts
    let goal_term_name_fp = to_fp_value(&g.goal_term.name);

    let mut goal_term_args_fp: Vec<Fp> = g.goal_term.args.iter().map(|s| to_fp_value(s)).collect();
    goal_term_args_fp.resize(MAX_ARITY, Fp::zero());

    //Creating the goal_unify_name
    let goal_unif_name_termfp = encode_str_to_termfp(&g.goal_unification.goal);


    // body/subtree maradhat
    let unif_body_fp: Vec<TermFp> = g.goal_unification
        .body
        .iter()
        .map(encode_json_val_to_termfp) // NINCS filter_map!
        .collect();

    let subtree_goals_fp: Vec<TermFp> = g.subtree
        .iter()
        .map(encode_proofnode_to_termfp) // NINCS filter_map!
        .collect();

    UnificationInputFp {
        goal_name: goal_name_termfp,       // ⬅ már PADDELT RLC(name,args)
        goal_term_name: goal_term_name_fp,
        goal_term_args: goal_term_args_fp,      // ⬅ PADDELT args, hogy a chip ugyanígy lássa
        unif_body: unif_body_fp,
        unif_goal: goal_unif_name_termfp,       // ⬅ ugyanaz az érték
        substitution: g.substitution.iter().map(|s| to_fp_value(s)).collect(),
        subtree_goals: subtree_goals_fp,
    }
}


pub fn encode_predicate_to_fp_vec(input: &str) -> Vec<Fp> {
    // 1️⃣ Szétszedjük a bemenetet: pl. "ancestor(a,b,c)" -> "ancestor", ["a", "b", "c"]
    let open = input.find('(').unwrap_or(input.len());
    let close = input.find(')').unwrap_or(input.len());
    let name = &input[..open].trim();

    let args_str = if open < close {
        &input[open + 1..close]
    } else {
        ""
    };

    let args: Vec<&str> = args_str
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim())
        .collect();

    // 2️⃣ Token lista: [predicate_name] + args
    let mut tokens: Vec<Fp> = Vec::with_capacity(MAX_ARITY);
    tokens.push(to_fp_value(name));
    for arg in &args {
        tokens.push(to_fp_value(arg));
    }

    // 3️⃣ Padding nullákkal MAX_ARITY-ig
    while tokens.len() < MAX_ARITY + 1{
        tokens.push(Fp::zero());
    }

    tokens
}



/// ✅ Stringből (pl. "ancestor(a,b,c)") készít egy TermFp struktúrát.
/// Ha kevesebb argumentum van, 0-val feltölti MAX_ARITY-ig.
pub fn encode_str_to_termfp(input: &str) -> TermFp {
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

    // 4️⃣ TermFp létrehozása
    TermFp {
        name: to_fp_value(name_str),
        args,
    }
}

fn encode_json_val_to_termfp(v: &serde_json::Value) -> TermFp {
    if let Some(s) = v.as_str() {
        encode_str_to_termfp(s) // a meglévő, string → TermFp
    } else if v == &serde_json::Value::Bool(true) {
        TermFp { name: to_fp_value("__TRUE__"), args: vec![Fp::zero(); MAX_ARITY] }
    } else {
        TermFp { name: to_fp_value("__INVALID__"), args: vec![Fp::zero(); MAX_ARITY] }
    }
}

fn encode_proofnode_to_termfp(n: &ProofNode) -> TermFp {
    match n {
        ProofNode::GoalNode(child) => encode_str_to_termfp(&child.goal),
        // ha van külön bool/leaf variánsod, azt kezeld itt:
        // ProofNode::Bool(true) | ProofNode::LeafTrue => ...
        // A példád alapján a JSON-ban "true" szerepel, ezért:
        _ => TermFp { name: to_fp_value("__TRUE__"), args: vec![Fp::zero(); MAX_ARITY] },
    }
}