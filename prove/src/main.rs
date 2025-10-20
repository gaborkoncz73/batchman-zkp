use std::fs;
use std::sync::Arc;
use anyhow::Result;
use rand_core::OsRng;

use halo2_proofs::{
    dev::MockProver,
    pasta::{EqAffine, Fp},
    plonk::{
        create_proof, keygen_pk, keygen_vk, verify_proof, SingleVerifier,
        ProvingKey, VerifyingKey,
    },
    poly::commitment::Params,
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use rayon::prelude::*;

use common::{data, data::UnificationInput};
use common::unification_checker_circuit::UnificationCircuit;

fn main() -> Result<()> {
    // --- inputok ---
    let rules_text = fs::read_to_string("input/rules_template.json")?;
    let rules: data::RuleTemplateFile = serde_json::from_str(&rules_text)?;

    let proof_text = fs::read_to_string("input/proof_tree.json")?;
    let tree: Vec<data::ProofNode> = serde_json::from_str(&proof_text)?;

    // --- Params + shape + keygen ---
    let params: Params<EqAffine> = Params::new(5);
    let shape = UnificationCircuit {
        rules: rules.clone(),
        unif: UnificationInput {
            goal_name: String::new(),
            goal_term_args: vec![],
            goal_term_name: String::new(),
            unif_body: vec![],
            unif_goal: String::new(),
            substitution: vec![],
            subtree_goals: vec![],
        },
    };
    let vk: VerifyingKey<EqAffine> = keygen_vk(&params, &shape)?;
    let pk: ProvingKey<EqAffine>  = keygen_pk(&params, vk.clone(), &shape)?;

    // tartsd Arc-ban ha párhuzamosan használod
    let params = Arc::new(params);
    let vk = Arc::new(vk);
    let pk = Arc::new(pk);

    // --- rekurzív bizonyítás/ellenőrzés minden node-ra ---
    tree.par_iter()
        .try_for_each(|node| prove_tree(&rules, node, &params, &vk, &pk))?;

    println!("✅ All unification goals proved and verified successfully!");
    Ok(())
}

// Pontos típusok az aláírásban (nincs '_'):
fn prove_tree(
    rules: &data::RuleTemplateFile,
    node: &data::ProofNode,
    params: &Arc<Params<EqAffine>>,
    vk: &Arc<VerifyingKey<EqAffine>>,
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

        // opcionális sanity check
        let prover = MockProver::run(5, &circuit, vec![])?;
        prover.assert_satisfied();

        // proof készítés — FIGYELEM: pk.as_ref(), params.as_ref()
        let mut transcript = Blake2bWrite::<Vec<u8>, _, Challenge255<_>>::init(vec![]);
        let instances: Vec<&[&[Fp]]> = vec![&[]];
        create_proof(
            params.as_ref(),
            pk.as_ref(),
            &[circuit],
            &instances,
            OsRng,
            &mut transcript,
        )?;
        let proof = transcript.finalize();

        // verify — FIGYELEM: vk.as_ref(), params.as_ref()
        let mut vr_transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
        let strategy = SingleVerifier::new(params.as_ref());
        verify_proof(
            params.as_ref(),
            vk.as_ref(),
            strategy,
            &instances,
            &mut vr_transcript,
        )?;

        println!("✅ Proved & verified goal: {}", g.goal);

        // rekurzió
        g.subtree.par_iter()
            .try_for_each(|sub| prove_tree(rules, sub, params, vk, pk))?;
    }
    Ok(())
}
