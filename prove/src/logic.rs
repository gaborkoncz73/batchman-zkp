use anyhow::{anyhow, Result};
use std::{collections::BTreeMap, sync::Arc};
use rayon::prelude::*;
use halo2_proofs::pasta::Fp;
use crate::{data::*, proofs::{self, *}, utils::*};
use halo2_proofs::arithmetic::Field;

pub const MAX_DOT_DIM: usize = 7;

pub fn prove_node(
    goal: &GoalEntry,
    rules: &RuleTemplateFile,
    id_map: &BTreeMap<String, i64>,
    proofs: &ProofStore,
    pk_store: &Arc<common::ProvingKeyStore>,
    depth: usize,
) -> Result<()> {
    let indent = "  ".repeat(depth);
    println!("{}ZK-Batchman check: {}", indent, goal.goal);

    // predicate ID
    let goal_id = predicate_id(&goal.goal_term.name, goal.goal_term.args.len(), id_map);
    if goal_id == 0 {
        return Err(anyhow!(
            "Unknown predicate/arity: {}/{}",
            goal.goal_term.name,
            goal.goal_term.args.len()
        ));
    }
    // is it a fact and fact check
    if is_fact_leaf(rules, goal_id, goal, id_map, &indent)? {
        return Ok(());
    }

    // Outer branches: (A) consistency proofs  ||  (B) body = subtree check + dot product proof
    let proofs_cons = proofs.clone();
    let proofs_b_join = proofs.clone();
    let pk_store_a = Arc::clone(pk_store);
    let pk_store_b = Arc::clone(pk_store);

    let (syntax_res, clauses_res): (Result<()>, Result<bool>) = rayon::join(
        // (A) consistency proofs
        || prove_syntax_consistency(&goal, &proofs_cons, &pk_store_a),

        // (B) branch: (Goal.unification.body and Subtree consistency + dot-product proof that the predicate exists)
        || prove_structural_clause_match(goal, rules, id_map, &proofs_b_join, &pk_store_b, depth, &indent),
    );

    // (A) syntax branch result
    syntax_res?;
    // (B) proof branch result
    if !clauses_res? {
        return Err(anyhow!(
            "'{}' does not fit any of the clauses",
            goal.goal
        ));
    }

    // paralel recursion on the children
    goal.subtree
        .par_iter()
        .filter_map(|n| if let ProofNode::GoalNode(g) = n { Some(g) } else { None })
        .try_for_each(|child| prove_node(child, rules, id_map, proofs, pk_store, depth + 1))?;

    println!("{}OK (ZK proofs collected): {}", indent, goal.goal);
    Ok(())
}

fn is_fact_leaf(rules: &RuleTemplateFile, goal_id: i64, goal: &GoalEntry, id_map: &BTreeMap<String, i64>, indent: &str) -> Result<bool>{
    let is_fact = rules
        .facts
        .iter()
        .any(|f| predicate_id(&f.name, f.arity, id_map) == goal_id);
    if is_fact {
        if goal.subtree.iter().any(|n| matches!(n, ProofNode::GoalNode(_))
            || goal.goal != goal.goal_unification.goal)
        {
            return Err(anyhow!(
                "fact '{}' subtree can't be empty or the two goals are not equal or it is not fact",
                goal.goal
            ));
        }
        println!("{}fact leaf: {}", indent, goal.goal);
        return Ok(true);
    }
    return Ok(false);
}


fn prove_syntax_consistency(
    goal: &GoalEntry,
    proofs: &ProofStore,
    pk_store: &Arc<common::ProvingKeyStore>,
) -> Result<()> {
    // Parse predicate names and arities inside the helper — they're only used here
    let (g_text_name, g_text_arity) = parse_predicate_call(&goal.goal)
        .ok_or_else(|| anyhow!("goal parse error: '{}'", goal.goal))?;
    let (u_text_name, u_text_arity) = parse_predicate_call(&goal.goal_unification.goal)
        .ok_or_else(|| anyhow!("goal_unification.goal parse error: '{}'", goal.goal_unification.goal))?;

    // Run both consistency proofs in parallel
    let (r1, r2) = rayon::join(
        // Goal vs Goal.term
        || proofs::prove_consistency(
            &g_text_name,
            g_text_arity.clone(),
            &goal.goal_term.name,
            goal.goal_term.args.len(),
            proofs,
            pk_store,
        ),
        // Goal vs Goal.unification.goal
        || proofs::prove_consistency(
            &g_text_name,
            g_text_arity,
            &u_text_name,
            u_text_arity,
            proofs,
            pk_store,
        ),
    );

    // Both must succeed
    r1.and(r2)
}

fn prove_structural_clause_match(
    goal: &GoalEntry,
    rules: &RuleTemplateFile,
    id_map: &BTreeMap<String, i64>,
    proofs: &ProofStore,
    pk_store: &Arc<common::ProvingKeyStore>,
    depth: usize,
    indent: &str,
) -> Result<bool> {
    // Predicate candidates
    let pred_matches = find_predicate_matches(goal, rules, id_map)?;
    let universe = local_universe(rules, &goal.goal_term.name);
    let w_vec = witness_subtree_presence(goal, &universe);

    // Body/subtree consistency checks
    validate_body_subtree_lengths(goal)?;

    // Run both inner joins in parallel
    let proofs_pairwise = proofs.clone();
    let pk_store_1 = Arc::clone(pk_store);
    let proofs_dot = proofs.clone();
    let pk_store_2 = Arc::clone(pk_store);

    let (pair_res, found) = rayon::join(
        || prove_body_subtree_consistency(goal, &proofs_pairwise, &pk_store_1),
        || prove_dot_clause_match(&pred_matches, goal, &universe, &w_vec, &proofs_dot, &pk_store_2, depth, indent),
    );

    pair_res?;
    Ok(found?)
}

fn find_predicate_matches<'a>(
    goal: &GoalEntry,
    rules: &'a RuleTemplateFile,
    id_map: &BTreeMap<String, i64>,
) -> Result<Vec<&'a PredicateTemplate>> {
    let goal_id = predicate_id(&goal.goal_term.name, goal.goal_term.args.len(), id_map);
    let matches: Vec<_> = rules
        .predicates
        .iter()
        .filter(|p| predicate_id(&p.name, p.arity, id_map) == goal_id)
        .collect();

    if matches.is_empty() {
        return Err(anyhow!("Predicate not found in rules: {}", goal.goal_term.name));
    }
    Ok(matches)
}

fn validate_body_subtree_lengths(goal: &GoalEntry) -> Result<()> {
    let b = goal.goal_unification.body.len();
    let s = goal.subtree.len();
    if b != s {
        return Err(anyhow!(
            "Body/subtree elements differ ({} vs {}) at goal: {}",
            b, s, goal.goal
        ));
    }
    Ok(())
}

fn prove_body_subtree_consistency(
    goal: &GoalEntry,
    proofs: &ProofStore,
    pk_store: &Arc<common::ProvingKeyStore>,
) -> Result<()> {
    let body_strs: Vec<&str> = goal
        .goal_unification
        .body
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    let sub_goals: Vec<&GoalEntry> = goal
        .subtree
        .iter()
        .filter_map(|n| match n {
            ProofNode::GoalNode(g) => Some(g),
            _ => None,
        })
        .collect();

    body_strs
        .into_par_iter()
        .zip(sub_goals.into_par_iter())
        .try_for_each(|(b_str, g)| {
            let (b_name, b_arity) = parse_predicate_call(b_str)
                .ok_or_else(|| anyhow!("invalid predicate in body: {}", b_str))?;
            let s_name = &g.goal_term.name;
            let s_arity = g.goal_term.args.len();

            proofs::prove_consistency(&b_name, b_arity, s_name, s_arity, proofs, pk_store)
        })
}

fn prove_dot_clause_match(
    pred_matches: &[&PredicateTemplate],
    goal: &GoalEntry,
    universe: &Vec<String>,
    w_vec: &Vec<Fp>,
    proofs: &ProofStore,
    pk_store: &Arc<common::ProvingKeyStore>,
    depth: usize,
    indent: &str,
) -> Result<bool> {
    let found = pred_matches.par_iter().any(|pred| {
        pred.clauses.par_iter().any(|clause| {
            let rows = rows_structural_global(clause, universe);
            let r = fs_coeffs(
                &format!("dotcheck:{}:{}:{}:{}", pred.name, goal.goal, goal.goal_term.name, depth),
                rows.len(),
            );
            let c_vec = compress_rows(&rows, &r);

            if c_vec.len() != w_vec.len() {
                return false;
            }

            let c_pad = pad(c_vec.clone());
            let w_pad = pad(w_vec.clone());

            let dot_debug: Fp = c_pad.iter().zip(&w_pad).map(|(a, b)| *a * *b).sum();
            if !dot_debug.is_zero_vartime() {
                return false;
            }

            if let Ok(proof) = common::prove_dot(pk_store, &c_pad, &w_pad) {
                proofs.dot_proofs.lock().unwrap().push((c_pad.clone(), proof));
                println!("{}dot(c,w) = 0 (proof generated and stored)", indent);
                return true;
            }
            false
        })
    });
    Ok(found)
}


pub fn pad(mut v: Vec<Fp>) -> Vec<Fp> {
    let const_col = v.pop().unwrap_or(Fp::one());
    while v.len() < MAX_DOT_DIM - 1 {
        v.push(Fp::zero());
    }
    v.push(const_col);
    v
}