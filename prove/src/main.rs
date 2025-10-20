use std::fs;
use std::sync::Arc;
use anyhow::Result;
use rand_core::OsRng;

use halo2_proofs::{
    pasta::{EqAffine, Fp},
    plonk::{keygen_pk, keygen_vk, ProvingKey, VerifyingKey},
    poly::commitment::Params,
    transcript::{Blake2bWrite, Challenge255},
};
use rayon::prelude::*;

use common::{data, data::UnificationInput};
use common::unification_checker_circuit::UnificationCircuit;

mod writer;
use writer::{init_output_dir, write_proof};

fn main() -> Result<()> {
    // --- inputok ---
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

    // --- Params + keygen ---
    let params: Params<EqAffine> = Params::new(5);
    let shape = UnificationCircuit {
        rules: rules.clone(),
        unif: UnificationInput::default(),
    };
    let vk: VerifyingKey<EqAffine> = keygen_vk(&params, &shape)?;
    let pk: ProvingKey<EqAffine> = keygen_pk(&params, vk.clone(), &shape)?;

    let params = Arc::new(params);
    let pk = Arc::new(pk);

    init_output_dir()?;

    // --- minden node-ra proof készítés ---
    tree.par_iter()
    .try_for_each(|node| {
        if let Err(e) = prove_tree(&rules, node, &params,  &pk) {
            eprintln!("❌ Error on node: {e:?}");
            return Err(e);
        }
        Ok(())
    })?;

    println!("All unification goals proved successfully and saved!");
    Ok(())
}

fn prove_tree(
    rules: &data::RuleTemplateFile,
    node: &data::ProofNode,
    params: &Arc<Params<EqAffine>>,
    pk: &Arc<ProvingKey<EqAffine>>,
) -> Result<()> {
    if let data::ProofNode::GoalNode(g) = node {
        let unif_input = UnificationInput {
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

        let circuit = UnificationCircuit { rules: rules.clone(), unif: unif_input };

        // --- proof készítés ---
        let mut transcript = Blake2bWrite::<Vec<u8>, _, Challenge255<_>>::init(vec![]);
        let instances: Vec<&[&[Fp]]> = vec![&[]];
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
        g.subtree.par_iter()
            .try_for_each(|sub| prove_tree(rules, sub, params, pk))?;
    }
    Ok(())
}
