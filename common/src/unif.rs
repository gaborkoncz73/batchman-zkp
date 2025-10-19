use std::fmt::format;

use blake2::{Blake2s256, Digest};
use halo2_proofs::{
    circuit::{Chip, Layouter, SimpleFloorPlanner, Value}, pasta::{group::ff::FromUniformBytes, Fp}, plonk::{Circuit, ConstraintSystem, Error}
};
use halo2_proofs::arithmetic::Field;
use serde::{Deserialize, Serialize};

use crate::{chips::{ConsistencyChip, DotChip}, data::{self, ClauseTemplate}, utils::*};
use crate::data::RuleTemplateFile;
pub const MAX_DOT_DIM: usize = 7;
// ------------------------------------------------------
// 🔹 Flat input struktúra (nem tartalmaz rekurzív subtree-t)
// ------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnificationInput {
    pub goal_name: String,
    pub goal_term_args: Vec<String>,
    pub goal_term_name: String,
    pub unif_body: Vec<String>,          // pl. ["parent(alice,bob)", "ancestor(bob,john)"]
    pub unif_goal: String,
    pub substitution: Vec<String>,       // pl. ["X=bob", "Y=john"]
    pub subtree_goals: Vec<String>,      // pl. ["parent(alice,bob)", "ancestor(bob,john)"]
}

// ------------------------------------------------------
// Circuit definíció
// ------------------------------------------------------
#[derive(Debug, Clone)]
pub struct UnificationCircuit {
    pub rules: RuleTemplateFile,
    pub unif: UnificationInput,
}

#[derive(Clone, Debug)]
pub struct UnifConfig {
    pub cons_cfg: <ConsistencyChip as Chip<Fp>>::Config,
    pub dot_cfg: <DotChip as Chip<Fp>>::Config,
}

impl Circuit<Fp> for UnificationCircuit {
    type Config = UnifConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            rules: RuleTemplateFile { predicates: vec![], facts: vec![] },
            unif: UnificationInput {
                goal_name: String::new(),
                goal_term_args: vec![],
                goal_term_name: String::new(),
                unif_body: vec![],
                unif_goal: String::new(),
                substitution: vec![],
                subtree_goals: vec![],
            },
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let cons_cfg = ConsistencyChip::configure(meta);
        let dot_cfg = DotChip::configure(meta);
        UnifConfig { cons_cfg  , dot_cfg }
    }

    fn synthesize(
        &self,
        cfg: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let cons_chip = ConsistencyChip::construct(cfg.cons_cfg);
        let dot_chip = DotChip::construct(cfg.dot_cfg);

        // ------------------------------------------------------
        // (Consistency check)
        // ------------------------------------------------------
        let (unif_goal_name, unif_goal_arity) =
            parse_predicate_call(&self.unif.unif_goal).unwrap_or((String::new(), 0));

        let (goal_name, goal_arity) =
            parse_predicate_call(&self.unif.goal_name).unwrap_or((String::new(), 0));

        //Goal name and arity
        let goal_name_fp = str_to_fp(&goal_name);
        let goal_arity_fp = Fp::from(goal_arity as u64);

        //Goal term name and arity
        let goal_term_name_fp = str_to_fp(&self.unif.goal_term_name);
        let goal_term_arity_fp = Fp::from(self.unif.goal_term_args.len() as u64);

        //Unification goal name and goal arity
        let unif_goal_name_fp = str_to_fp(&unif_goal_name);
        let unif_goal_arity_fp = Fp::from(unif_goal_arity as u64);

        cons_chip.assign_pairs(
            layouter.namespace(|| "goal_vs_unif_goal"),
            vec![(goal_name_fp, goal_arity_fp, unif_goal_name_fp, unif_goal_arity_fp),
            (goal_name_fp, goal_arity_fp, goal_term_name_fp, goal_term_arity_fp)],

        )?;
        for (i, (a,b)) in extract_args(&self.unif.goal_name)
            .into_iter()
            .zip(extract_args(&self.unif.unif_goal))
            .enumerate()
        {
            cons_chip.assign_pair2(layouter.namespace(||format!("name_args_vs_body_args:{}",i)),
            (str_to_fp(&a),str_to_fp(&b)))?;
        }

        for (i, (a,b)) in extract_args(&self.unif.goal_name)
            .into_iter()
            .zip(&self.unif.goal_term_args)
            .enumerate()
        {
            cons_chip.assign_pair2(layouter.namespace(||format!("name_args_vs_body_args:{}",i)),
            (str_to_fp(&a),str_to_fp(&b)))?;
        }

        for (i, (body_str, subtree_str)) in self
            .unif
            .unif_body
            .iter()
            .zip(self.unif.subtree_goals.iter())
            .enumerate()
        {
            // Convert both strings to field elements
            let body_fp = str_to_fp(body_str);
            let subtree_fp = str_to_fp(subtree_str);

            // Call your 2-value consistency check circuit
            cons_chip.assign_pair2(
                layouter.namespace(|| format!("body_vs_subtree_{}", i)),
                (body_fp, subtree_fp),
            )?;
        }
        // Predicate check 


        let universe = local_universe(&self.rules, &self.unif.goal_term_name);
        let w_vec = witness_subtree_presence_goal(&self.unif, &universe);

        let mut variable_rules: Vec<Vec<Fp>> = Vec::new();

        // Iterate all predicates in the ruleset
        for (p_i, pred) in self.rules.predicates.iter().enumerate() {
            // Only check matching predicate name/arity
            if pred.name != self.unif.goal_term_name || pred.arity != self.unif.goal_term_args.len() {
                continue;
            }

            // Iterate through all clauses of this predicate
            for (c_i, clause) in pred.clauses.iter().enumerate() {
                // Structural row vectors (same as CPU-side)
                let rows = rows_structural_global(clause, &universe);
                let r = fs_coeffs(
                    &format!("dotcheck:{}:{}:{}", pred.name, self.unif.goal_name, c_i),
                    rows.len(),
                );
                let c_vec = compress_rows(&rows, &r);

                // Flatten and pad vectors to MAX_DOT_DIM
                if c_vec.len() != w_vec.len() {
                    println!("Clause {} skipped: len mismatch ({} vs {})", c_i, c_vec.len(), w_vec.len());
                    continue;
                }
                let c_pad = pad(c_vec.clone());
                let w_pad = pad(w_vec.clone());

                let dot_debug: Fp = c_pad.iter().zip(&w_pad).map(|(a, b)| *a * *b).sum();
                if !dot_debug.is_zero_vartime() {
                    continue;
                }

                // Perform Halo2 proof of dot(c, w) = 0
                dot_chip.assign_dot_check(
                    layouter.namespace(|| format!("pred{}_clause{}_dotcheck", p_i, c_i)),
                    &w_pad,
                    &c_pad,
                Fp::one(),
                )?;
                variable_rules = rows_equality_global(clause);
            }
        }
        let w_vec = flatten_goal_variables_fp(&self.unif.goal_term_args, &self.unif.unif_body);
        let r = fs_coeffs("seed", 7);
        let compressed_rows = compress_rows(&variable_rules, &r);
        dot_chip.assign_dot_check(
            layouter.namespace(|| format!("variable_dot_check:")),
            &w_vec,
     &compressed_rows,
Fp::zero(),)?;
        Ok(())
    }
}



fn local_universe(rules: &RuleTemplateFile, goal_name: &str) -> Vec<String> {
    // egyszerű verzió: összes konstans + predikátum neve
    let mut uni = vec![goal_name.to_string()];
    for p in &rules.predicates {
        uni.push(p.name.clone());
    }
    uni.sort();
    uni.dedup();
    uni
}

fn witness_subtree_presence_goal(unif: &UnificationInput, universe: &Vec<String>) -> Vec<Fp> {
    // flatten body+goal struktúra a univerzumra vetítve
    let mut vec = Vec::new();
    for u in universe {
        let present = unif.unif_body.iter().any(|b| b.contains(u));
        vec.push(if present { Fp::one() } else { Fp::zero() });
    }
    vec.push(Fp::one()); // offset / const col
    vec
}


pub fn flatten_goal_variables(goal_term_args: &Vec<String>, goal_unification_body: &Vec<String>) -> Vec<String> {
    let mut vars = Vec::new();

    // Add head goal arguments
    vars.extend(goal_term_args.clone());

    // Add each predicate call in the body
    for body_entry in goal_unification_body.iter() {
        let call_str = body_entry ;
            // parse something like "parent(alice,bob)"
            if let Some((_, args)) = parse_predicate_call2(call_str) {
                vars.extend(args);
            }
        
    }
    vars
}

pub fn flatten_goal_variables_fp(goal_term_args: &Vec<String>, goal_unification_body: &Vec<String>) -> Vec<Fp> {
    let vars = flatten_goal_variables(goal_term_args,goal_unification_body);
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

pub fn pad(mut v: Vec<Fp>) -> Vec<Fp> {
    let const_col = v.pop().unwrap_or(Fp::one());
    while v.len() < MAX_DOT_DIM - 1 {
        v.push(Fp::zero());
    }
    v.push(const_col);
    v
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


pub fn fs_coeffs(seed: &str, m: usize) -> Vec<Fp> {
    let mut out = Vec::with_capacity(m);
    for i in 0..m {
        let mut h = Blake2s256::new();
        h.update(seed.as_bytes());
        h.update(i.to_le_bytes());
        let hash32 = h.finalize();

        let mut wide = [0u8; 64];
        wide[..32].copy_from_slice(&hash32);

        let fp = <Fp as FromUniformBytes<64>>::from_uniform_bytes(&wide);
        out.push(fp);
    }
    out
}

pub fn compress_rows(rows: &[Vec<Fp>], r: &[Fp]) -> Vec<Fp> {
    let m = rows[0].len();
    let mut c = vec![Fp::zero(); m];
    for (ri, row) in r.iter().zip(rows.iter()) {
        for j in 0..m {
            c[j] += *ri * row[j];
        }
    }
    c
}

fn extract_args(goal_str: &str) -> Vec<String> {
    if let Some(start) = goal_str.find('(') {
        if let Some(end) = goal_str.find(')') {
            return goal_str[start + 1..end]  // zárójelek közti rész
                .split(',')                   // vesszővel elválasztott argumentumok
                .map(|s| s.trim().to_string()) // szóközöket levágjuk
                .filter(|s| !s.is_empty())     // üreseket eldobjuk
                .collect();
        }
    }
    vec![] // ha nem található zárójel
}