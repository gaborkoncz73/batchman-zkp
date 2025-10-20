use halo2_proofs::{arithmetic::Field, pasta::Fp};

use crate::{data::{self, ClauseTemplate}, utils_2::common_helpers::{str_to_fp,fs_coeffs,compress_rows,pad}};

pub fn get_w_and_v_vec(
    clause: &ClauseTemplate,
    goal_term_args: &[String],
    unif_body: &[String],
    seed: &str,
    m: usize,
) -> Result<(Vec<Fp>, Vec<Fp>), halo2_proofs::plonk::Error>
{
    // Determine real head arity dynamically (it was already proven that they have the same length)
    let head_arity = goal_term_args.len();

    // Build equality rows with the real arity
    let variable_rules = rows_equality_global(&clause, head_arity);

    // Flatten goal and body arguments into w_vec
    let w_vec = flatten_goal_variables_fp(goal_term_args, &unif_body);

    // Compress the equality rows
    let r = fs_coeffs(seed, m);
    let compressed_rows = compress_rows(&variable_rules, &r);

    // Pad both vectors to the same dimension
    let padded_w_vec = pad(w_vec);
    let padded_compressed_rows = pad(compressed_rows);

    Ok((padded_w_vec, padded_compressed_rows))
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

/// Produce equality-rows from a ClauseTemplate
/// Each row is a vector over flattened term positions
/// encoding (a_i - a_j = 0) as (+1, -1) in the respective positions
pub fn rows_equality_global(
    clause: &data::ClauseTemplate,
    head_arity: usize,
) -> Vec<Vec<Fp>> 
{
    let mut offsets = Vec::new();
    let mut current = 0usize;

    // Head starts at 0
    offsets.push(0);

    // Children start after the head’s arguments
    current += head_arity;
    for child in &clause.children {
        offsets.push(current);
        current += child.arity;
    }

    let total_positions = current;

    // Collect all pairwise equalities into flattened indices
    let mut pairs = Vec::new();
    for eq in &clause.equalities {
        let left_index = offsets[eq.left.node] + eq.left.arg;
        let right_index = offsets[eq.right.node] + eq.right.arg;
        pairs.push((left_index, right_index));
    }

    // Use union-find to merge equal variables
    let mut uf = UnionFind::new(total_positions);
    for &(a, b) in &pairs {
        uf.union(a, b);
    }

    let mut groups: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for i in 0..total_positions {
        groups.entry(uf.find(i)).or_default().push(i);
    }

    // Generate equality rows
    let mut rows = Vec::new();
    for group in groups.values() {
        if group.len() > 1 {
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

// Convert the flattended String vector into Fp vector
pub fn flatten_goal_variables_fp(
    goal_term_args: &[String],
    goal_unification_body: &[String]
) -> Vec<Fp>
{
    let vars = flatten_goal_variables(goal_term_args,goal_unification_body);
    let mut v_fp: Vec<Fp> = vars.iter().map(|s| str_to_fp(s)).collect();
    v_fp.push(Fp::ONE); // enforce non-triviality
    v_fp
}

// Flatten the variables: "goal":"ancestor(alice,john)" and "body": ["parent(alice,bob)", "ancestor(bob,john)" -> alice,john,alice,bob,bob,john
pub fn flatten_goal_variables(
    goal_term_args: &[String],
    goal_unification_body: &[String]
) -> Vec<String>
{
    let mut vars = Vec::new();

    // Add head goal arguments
    vars.extend_from_slice(goal_term_args);

    // Add each predicate call in the body
    for body_entry in goal_unification_body.iter() {
        if let Some((_, args)) = parse_predicate_call_for_variables(body_entry) {
            vars.extend(args);
        }
    }
    vars
}

// helper to parse a predicate call string like "parent(alice,bob)" -> alice,bob
fn parse_predicate_call_for_variables(
    s: &str
) -> Option<(String, Vec<String>)>
{
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