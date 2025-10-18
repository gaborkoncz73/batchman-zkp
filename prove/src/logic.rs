use anyhow::{anyhow, Result};
use std::{collections::BTreeMap, sync::Arc};
use rayon::prelude::*;
use halo2_proofs::pasta::Fp;
use crate::{proofs::{self, *}, utils::*};
use halo2_proofs::arithmetic::Field;
use common::data;


pub const MAX_DOT_DIM: usize = 7;

pub fn prove_node(
    goal: &data::GoalEntry,
    rules: &data::RuleTemplateFile,
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
        .filter_map(|n| if let data::ProofNode::GoalNode(g) = n { Some(g) } else { None })
        .try_for_each(|child| prove_node(child, rules, id_map, proofs, pk_store, depth + 1))?;

    println!("{}OK (ZK proofs collected): {}", indent, goal.goal);
    Ok(())
}

fn is_fact_leaf(rules: &data::RuleTemplateFile, goal_id: i64, goal: &data::GoalEntry, id_map: &BTreeMap<String, i64>, indent: &str) -> Result<bool>{
    let is_fact = rules
        .facts
        .iter()
        .any(|f| predicate_id(&f.name, f.arity, id_map) == goal_id);
    if is_fact {
        if goal.subtree.iter().any(|n| matches!(n, data::ProofNode::GoalNode(_))
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
    goal: &data::GoalEntry,
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
    goal: &data::GoalEntry,
    rules: &data::RuleTemplateFile,
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
    goal: &data::GoalEntry,
    rules: &'a data::RuleTemplateFile,
    id_map: &BTreeMap<String, i64>,
) -> Result<Vec<&'a data::PredicateTemplate>> {
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

fn validate_body_subtree_lengths(goal: &data::GoalEntry) -> Result<()> {
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
    goal: &data::GoalEntry,
    proofs: &ProofStore,
    pk_store: &Arc<common::ProvingKeyStore>,
) -> Result<()> {
    let body_strs: Vec<&str> = goal
        .goal_unification
        .body
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    let sub_goals: Vec<&data::GoalEntry> = goal
        .subtree
        .iter()
        .filter_map(|n| match n {
            data::ProofNode::GoalNode(g) => Some(g),
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

use rayon::join;
fn prove_dot_clause_match(
    pred_matches: &[&data::PredicateTemplate],
    goal: &data::GoalEntry,
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

            let variable_rules: Vec<Vec<Fp>> = rows_equality_global(clause);
            /*println!("Clause rows:");
            for r in &variable_rules {
                println!(
                    "{:?}",
                    r.iter()
                        .map(|x| if *x == Fp::ZERO { 0 } else if *x == Fp::ONE { 1 } else { -1 })
                        .collect::<Vec<_>>()
                );
            }*/

            // Get the actual goal variables
            let w_vec = flatten_goal_variables_fp(goal);
            //println!("Witness vector (Fp): {:?}", w_vec);




            let proofs1 = proofs.clone();
            let pk_store_1 = Arc::clone(pk_store);
            let proofs2 = proofs.clone();
            let pk_store_2 = Arc::clone(pk_store);


            // Run both proof groups in parallel
            let (main_ok, vars_ok) = join(
                // main structural dot proof
                || {
                    if let Ok((proof, instances)) = common::prove_dot(&pk_store_2, &c_pad, &w_pad, Fp::one()) {
                        proofs2.dot_proofs.lock().unwrap().push((instances.clone(), proof));
                        println!("{}dot(c,w) = 0 (proof generated and stored)", indent);
                        true
                    } else {
                        println!("{}dot(c,w) proof generation failed", indent);
                        false
                    }
                },
                // variable equality proofs (parallel inside)
                || {
                    variable_rules
                        .par_iter()
                        .enumerate()
                        .map(|(i, var_rule)| {
                            if var_rule.len() != w_vec.len() {
                                println!(
                                    "{}Variable rule {} length mismatch ({} vs {})",
                                    indent, i, var_rule.len(), w_vec.len()
                                );
                                return false;
                            }

                            // Pad both sides for the circuit
                            let var_pad = pad(var_rule.clone());
                            let w_pad2 = pad(w_vec.clone());

                            // Local dot check before proving
                            let dot_check: Fp = var_pad.iter().zip(&w_pad2).map(|(a, b)| *a * *b).sum();
                            /*println!(
                                "var_pad={:?}\nw_pad={:?}\nsum={:?}",
                                var_pad,
                                w_pad,
                                dot_check
                            );*/

                            if !dot_check.is_zero_vartime() {
                                println!("{}dot(rule{}, w)!=0", indent, i);
                                return false;
                            }
                            if let Ok((proof, instances)) = common::prove_dot(&pk_store_1, &var_pad, &w_pad2, Fp::zero()) {
                                 proofs1.dot_proofs.lock().unwrap().push((instances.clone(), proof));
                                println!("{}dot(rule{}, w)=0 proof stored", indent, i);
                                true
                            } else {
                                println!("{}dot(rule{}, w) proof generation failed", indent, i);
                                false
                          }
                        })
                        .all(|ok| ok)
                },
            );

            main_ok && vars_ok
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



/// Produce equality-rows from a ClauseTemplate.
/// Each row is a vector over flattened term positions,
/// encoding (a_i - a_j = 0) as (+1, -1) in the respective positions.
pub fn rows_equality_global(clause: &data::ClauseTemplate) -> Vec<Vec<Fp>> {
    let mut offsets = Vec::new();
    let mut current = 0usize;

    // node 0 = head (arity = head_arity)
    let head_arity = clause.children.first().map_or(2, |_| 2); // assume 2 for head
    offsets.push(0);

    // each child starts after all previous nodes' arities
    current += head_arity;
    for child in &clause.children {
        offsets.push(current);
        current += child.arity;
    }

    let total_positions = current;


    // collect all pairwise equalities into flattened indices
    let mut pairs = Vec::new();
    for eq in &clause.equalities {
        let left_index = offsets[eq.left.node] + eq.left.arg;
        let right_index = offsets[eq.right.node] + eq.right.arg;
        pairs.push((left_index, right_index));
    }

    // use union-find to connect equal variables with >2 appearances
    let mut uf = UnionFind::new(total_positions);
    for &(a, b) in &pairs {
        uf.union(a, b);
    }

    let mut groups: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for i in 0..total_positions {
        groups.entry(uf.find(i)).or_default().push(i);
    }

    // Generate equality rows ---
    let mut rows = Vec::new();
    for group in groups.values() {
    if group.len() > 1 {
        // connect all consecutive pairs within the group
        for w in group.windows(2) {
            let mut row = vec![Fp::ZERO; total_positions];
            row[w[0]] = Fp::ONE;
            row[w[1]] = -Fp::ONE;
             row.push(Fp::ZERO);
            rows.push(row);
        }
    }
}


    rows
}

/// Simple union-find (disjoint-set)
#[derive(Clone)]
struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
        }
    }
    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            let p = self.parent[x];
            self.parent[x] = self.find(p);
        }
        self.parent[x]
    }
    fn union(&mut self, a: usize, b: usize) {
        let pa = self.find(a);
        let pb = self.find(b);
        if pa != pb {
            self.parent[pa] = pb;
        }
    }
}


pub fn flatten_goal_variables(goal: &data::GoalEntry) -> Vec<String> {
    let mut vars = Vec::new();

    // Add head goal arguments
    vars.extend(goal.goal_term.args.clone());

    // Add each predicate call in the body
    for body_entry in &goal.goal_unification.body {
        if let Some(call_str) = body_entry.as_str() {
            // parse something like "parent(alice,bob)"
            if let Some((_, args)) = parse_predicate_call2(call_str) {
                vars.extend(args);
            }
        }
    }
    vars
}

pub fn flatten_goal_variables_fp(goal: &data::GoalEntry) -> Vec<Fp> {
    let vars = flatten_goal_variables(goal);
    let mut v_fp: Vec<Fp> = vars.iter().map(|s| str_to_fp(s)).collect();
    v_fp.push(Fp::ONE); // enforce non-triviality
    v_fp
}

/// helper to parse a predicate call string like "parent(alice,bob)"
fn parse_predicate_call2(s: &str) -> Option<(String, Vec<String>)> {
    let open = s.find('(')?;
    let close = s.find(')')?;
    let name = s[..open].trim().to_string();
    let args_str = &s[open + 1..close];
    let args = args_str
        .split(',')
        .map(|x| x.trim().to_string())
        .collect::<Vec<_>>();
    Some((name, args))
}