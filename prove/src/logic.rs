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

    // consistency setup
    let (g_text_name, g_text_arity) = parse_predicate_call(&goal.goal)
        .ok_or_else(|| anyhow!("goal parse error: '{}'", goal.goal))?;
    let (u_text_name, u_text_arity) = parse_predicate_call(&goal.goal_unification.goal)
        .ok_or_else(|| anyhow!("goal_unification.goal parse error: '{}'", goal.goal_unification.goal))?;

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
    let is_fact_leaf = rules
        .facts
        .iter()
        .any(|f| predicate_id(&f.name, f.arity, id_map) == goal_id);
    if is_fact_leaf {
        if goal.subtree.iter().any(|n| matches!(n, ProofNode::GoalNode(_)) || goal.goal != goal.goal_unification.goal) {
            return Err(anyhow!("fact '{}' subtree can't be empty or the two goals are not equal or it is not fact", goal.goal));
        }
        println!("{}fact leaf: {}", indent, goal.goal);
        return Ok(());
    }

    // Outer branches: (A) consistency proofs  ||  (B) body = subtree check + dot product proof
    let proofs_cons = proofs.clone();
    let proofs_b_join = proofs.clone();
    let pk_store_a = Arc::clone(pk_store);
    let pk_store_b = Arc::clone(pk_store);

    let (syntax_res, clauses_res): (Result<()>, Result<bool>) = rayon::join(
        // (A) consistency proofs
        || {
            let (r1, r2) = rayon::join(
                // Goal and Goal.term check
                || proofs::prove_consistency(
                    &g_text_name,
                    g_text_arity.clone(),
                    &goal.goal_term.name,
                    goal.goal_term.args.len(),
                    &proofs_cons,
                    &pk_store_a.clone(),
                ),
                //Goal and Goal.unification.goal check
                || proofs::prove_consistency(
                    &g_text_name,
                    g_text_arity,
                    &u_text_name,
                    u_text_arity,
                    &proofs_cons,
                    &pk_store_a,
                ),
            );
            r1.and(r2)
        },

        // (B) branch: (Goal.unification.body and Subtree consistency + dot-product proof that the predicate exists)
        || {
            // predicate candidates (name and arity)
            let pred_matches: Vec<&PredicateTemplate> = rules
                .predicates
                .iter()
                .filter(|p| predicate_id(&p.name, p.arity, id_map) == goal_id)
                .collect();
            if pred_matches.is_empty() {
                return Err(anyhow!(
                    "Predicate is not in the rules: {}",
                    goal.goal_term.name
                ));
            }

            // local universe + witness
            let universe = local_universe(rules, &goal.goal_term.name);
            let w_vec = witness_subtree_presence(goal, &universe);

            // előzetes hosszellenőrzés (body és subtree)
            let body_len = goal.goal_unification.body.len();
            let subtree_len = goal.subtree.len();
            if body_len != subtree_len {
                return Err(anyhow!(
                    "body/subtree elements differ ({} vs {}) at goal: {}",
                    body_len,
                    subtree_len,
                    goal.goal
                ));
            }

            // clones for the innes joins
            let proofs_pairwise = proofs_b_join.clone();
            let pk_store_b1 = Arc::clone(&pk_store_b);

            let proofs_dot = proofs_b_join.clone();
            let pk_store_b2 = Arc::clone(&pk_store_b);

            // Inner join: (B1) body[i] == subtree[i] || (B2) structural dot check 
            let (pair_res, found): (Result<()>, Result<bool>) = rayon::join(
                // (B1) body[i] == subtree[i] consistency check
                || {
                    // setup the body strings
                    let body_strs: Vec<&str> = goal
                        .goal_unification
                        .body
                        .par_iter()
                        .filter_map(|v| v.as_str())
                        .collect();

                    // setup the subtree goal nodes
                    let sub_goals: Vec<&GoalEntry> = goal
                        .subtree
                        .par_iter()
                        .filter_map(|n| {
                            if let ProofNode::GoalNode(g) = n {
                                Some(g)
                            } else {
                                None
                            }
                        })
                        .collect();

                    // zipping and paralel consistency proof generation
                    body_strs
                        .into_par_iter()
                        .zip(sub_goals.into_par_iter())
                        .try_for_each(|(b_str, g)| {
                            let (b_name, b_arity) = parse_predicate_call(b_str)
                                .ok_or_else(|| anyhow!("invalid predicate in body: {}", b_str))?;

                            let s_name = &g.goal_term.name;
                            let s_arity = g.goal_term.args.len();

                            proofs::prove_consistency(
                                &b_name,
                                b_arity,
                                s_name,
                                s_arity,
                                &proofs_pairwise,
                                &pk_store_b1,
                            )
                        })
                },

                // (B2) structural dot(c,w)=0 proof-search
                || {
                    let found = pred_matches.par_iter().any(|pred| {
                        pred.clauses.par_iter().any(|clause| {
                            let rows = rows_structural_global(clause, &universe);
                            let r = fs_coeffs(
                                &format!(
                                    "dotcheck:{}:{}:{}:{}",
                                    pred.name, goal.goal, goal.goal_term.name, depth
                                ),
                                rows.len(),
                            );
                            let c_vec = compress_rows(&rows, &r);

                            if c_vec.len() != w_vec.len() {
                                return false;
                            }

                            let c_pad = pad(c_vec.clone());
                            let w_pad = pad(w_vec.clone());

                            let dot_debug: Fp =
                                c_pad.iter().zip(&w_pad).map(|(a, b)| *a * *b).sum();
                            if !dot_debug.is_zero_vartime() {
                                return false;
                            }

                            if let Ok(proof) = common::prove_dot(&pk_store_b2, &c_pad, &w_pad) {
                                proofs_dot
                                    .dot_proofs
                                    .lock()
                                    .unwrap()
                                    .push((c_pad.clone(), proof));
                                println!("{}dot(c,w) = 0 (proof generated and stored)", indent);
                                return true;
                            }
                            false
                        })
                    });
                    Ok(found)
                },
            );

            // (B1) error handling
            pair_res?;
            // (B2) if there is no clause
            if !found? {
                return Err(anyhow!(
                    "'{}' does not fit any of the clauses",
                    goal.goal
                ));
            }

            Ok(true)
        },
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

pub fn pad(mut v: Vec<Fp>) -> Vec<Fp> {
    let const_col = v.pop().unwrap_or(Fp::one());
    while v.len() < MAX_DOT_DIM - 1 {
        v.push(Fp::zero());
    }
    v.push(const_col);
    v
}
