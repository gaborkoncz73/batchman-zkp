use std::collections::HashMap;

use common::{data::{GoalEntry, ProofNode, TermFp, UnificationInputFp, Config}, utils_2::{common_helpers::{to_fp_value, MAX_ARITY}}};
use halo2_proofs::pasta::Fp;

// From the goal and hashmap it creates the Unification input
pub fn unification_input_from_goal_and_facts(g: &GoalEntry, facts: &HashMap<String, Fp>) -> UnificationInputFp {
    //Creating the goal_name
    let goal_name_termfp = encode_str_to_termfp(&g.goal, facts);
    
    //Creating the subtree goals term list
    let subtree_goals_term_fp: Vec<TermFp> = g.subtree
        .iter()
        .map(|a|encode_proofnode_to_termfp(a,facts)) 
        .collect();

    UnificationInputFp {
        goal_name: goal_name_termfp, 
        subtree_goals: subtree_goals_term_fp,
    }
}

// Converting a goal name into structured TermFp
fn encode_str_to_termfp(input: &str, facts: &HashMap<String, Fp>) -> TermFp {
    // Split the name and arguments
    let open = input.find('(').unwrap_or(input.len());
    let close = input.find(')').unwrap_or(input.len());

    let name_str = input[..open].trim();
    let args_str = if open < close {
        &input[open + 1..close]
    } else {
        ""
    };

    // Arguments into Fp list
    let mut args: Vec<Fp> = args_str
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| to_fp_value(s.trim()))
        .collect();

    // Padding until MAX_ARITY
    while args.len() < MAX_ARITY {
        args.push(Fp::zero());
    }

    // Name into Fp
    let name_fp = to_fp_value(name_str);

    // Get salt
    let salt = facts.get(input).copied().unwrap_or(Fp::zero());
    // Creating TermFp
    TermFp {
        name: name_fp,
        args,
        fact_hashes: salt,
    }
}

fn encode_proofnode_to_termfp(n: &ProofNode, facts: &HashMap<String, Fp> ) -> TermFp {
    match n {
        ProofNode::GoalNode(child) => encode_str_to_termfp(&child.goal, facts),
        _ => TermFp { name: to_fp_value("__TRUE__"), args: vec![Fp::zero(); MAX_ARITY], fact_hashes: Fp::zero()  },
    }
}

// Building the factmap to get the salts easier
pub fn build_fact_map(facts: &[Config]) -> HashMap<String, Fp> {
    let mut map = HashMap::new();

    for conf in facts {
        // Build the key string
        let key = if conf.args.is_empty() {
            conf.predicate.clone()
        } else {
            format!("{}({})", conf.predicate, conf.args.join(","))
        };

        // Convert salt to Fp
        let salt = to_fp_value(&conf.salt);

        map.insert(key, salt);
    }
    map
}

// CPU RLC counter
pub fn rlc_encode_cpu(tokens: &[Fp], alpha: Fp) -> Fp {
    let mut acc = Fp::zero();
    for &t in tokens {
        acc = acc * alpha + t;
    }
    acc
}