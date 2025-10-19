use std::{fs, sync::{Arc, Mutex}};
use anyhow::Result;
use rayon::prelude::*;
use halo2_proofs::{dev::MockProver, pasta::Fp};

use common::{unif::{UnificationCircuit, UnificationInput}, *};


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
        .filter_map(|n| {
            if let data::ProofNode::GoalNode(g) = n {
                Some(g)
            } else {
                None
            }
        })
        .try_for_each(|goal_entry| -> Result<()> {
            // Convert the GoalEntry into a flat UnificationInput for circuit
            let unif_input = UnificationInput {
                goal_name: goal_entry.goal.clone(),
                goal_term_args: goal_entry.goal_term.args.clone(),
                goal_term_name: goal_entry.goal_term.name.clone(),
                unif_body: goal_entry
                    .goal_unification
                    .body
                    .iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect(),
                unif_goal: goal_entry.goal_unification.goal.clone(),
                substitution: goal_entry.substitution.clone(),
                subtree_goals: goal_entry
                    .subtree
                    .iter()
                    .filter_map(|n| match n {
                        data::ProofNode::GoalNode(g) => Some(g.goal.clone()),
                        _ => None,
                    })
                    .collect(),
            };

            // Construct the circuit
            let circuit = UnificationCircuit {
                rules: rules.clone(),
                unif: unif_input,
            };

            // Run a mock prover for debugging
            let prover = MockProver::run(10, &circuit, vec![])?;
            prover.assert_satisfied();
            println!("UnificationCircuit verified for goal: {}", goal_entry.goal);

            Ok(())
        })?;

    println!("All unification goals verified successfully!");
    Ok(())
}
