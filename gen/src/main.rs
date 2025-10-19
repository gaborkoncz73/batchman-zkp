use std::{fs, sync::{Arc, Mutex}};
use anyhow::Result;
use rayon::prelude::*;
use halo2_proofs::{dev::MockProver, pasta::Fp};

use common::{unification_checker_circuit::{UnificationCircuit, UnificationInput}, *};


fn main() -> Result<()> {
    // Load JSON input files
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

    // Iterate over all goal nodes and prove unification
    tree.par_iter()
        .try_for_each(|node| prove_tree(&rules, node))?;


    println!("All unification goals verified successfully!");
    Ok(())
}


fn prove_tree(rules: &data::RuleTemplateFile, node: &data::ProofNode) -> Result<()> {
    if let data::ProofNode::GoalNode(g) = node {
        // -- 1️⃣ Circuit input előállítása --
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

        // -- 2️⃣ Circuit futtatás --
        let circuit = UnificationCircuit {
            rules: rules.clone(),
            unif: unif_input,
        };

        let prover = MockProver::run(5, &circuit, vec![])?;
        prover.assert_satisfied();
        println!("✅ Verified goal: {}", g.goal);

        // -- 3️⃣ Rekurzív bejárás a subtree-kre --
        g.subtree.par_iter()
            .try_for_each(|sub| prove_tree(rules, sub))?;
            }
    Ok(())
}