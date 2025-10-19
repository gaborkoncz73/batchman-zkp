use blake2::{Blake2s256, Digest};
use halo2_proofs::pasta::Fp;
use halo2_proofs::pasta::group::ff::FromUniformBytes;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::hash::{Hash, Hasher};
use crate::data::*;

pub fn build_predicate_id_map(rules: &RuleTemplateFile) -> BTreeMap<String, i64> {
    let mut map = BTreeMap::<String, i64>::new();
    let mut counter = 1i64;
    for p in &rules.predicates {
        map.insert(format!("{}/{}", p.name, p.arity), counter);
        counter += 1;
    }
    for f in &rules.facts {
        let key = format!("{}/{}", f.name, f.arity);
        if !map.contains_key(&key) {
            map.insert(key, counter);
            counter += 1;
        }
    }
    map
}

pub fn predicate_id(name: &str, arity: usize, id_map: &BTreeMap<String, i64>) -> i64 {
    let key = format!("{}/{}", name, arity);
    *id_map.get(&key).unwrap_or(&0)
}

pub fn parse_predicate_call(s: &str) -> Option<(String, usize)> {
    let pos = s.find('(')?;
    let name = s[..pos].to_string();
    let inside = &s[pos + 1..s.len() - 1];
    let argc = if inside.trim().is_empty() {
        0
    } else {
        inside.split(',').count()
    };
    Some((name, argc))
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

pub fn local_universe(rules: &RuleTemplateFile, pred_name: &str) -> Vec<String> {
    let mut set = BTreeSet::<String>::new();
    for p in &rules.predicates {
        if p.name == pred_name {
            for cl in &p.clauses {
                for ch in &cl.children {
                    set.insert(ch.name.clone());
                }
            }
        }
    }
    for f in &rules.facts {
        set.insert(f.name.clone());
    }
    set.into_iter().collect()
}

pub fn witness_subtree_presence(goal: &crate::data::GoalEntry, universe: &[String]) -> Vec<Fp> {
    let present: HashSet<String> = goal
        .subtree
        .iter()
        .filter_map(|n| if let crate::data::ProofNode::GoalNode(g) = n { Some(g.goal_term.name.clone()) } else { None })
        .collect();
    let mut w: Vec<Fp> = universe
        .iter()
        .map(|u| if present.contains(u) { Fp::one() } else { Fp::zero() })
        .collect();
    w.push(Fp::one());
    w
}

pub fn rows_structural_global(clause: &crate::data::ClauseTemplate, universe: &[String]) -> Vec<Vec<Fp>> {
    let expected: HashSet<String> = clause.children.iter().map(|c| c.name.clone()).collect();
    let neg_one = -Fp::one();
    let cols = universe.len() + 1;
    let mut rows = Vec::with_capacity(universe.len());
    for (j, u) in universe.iter().enumerate() {
        let mut v = vec![Fp::zero(); cols];
        v[j] = Fp::one();
        if expected.contains(u) {
            v[cols - 1] = neg_one;
        }
        rows.push(v);
    }
    rows
}

pub fn str_to_fp(s: &str) -> Fp {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    let v = h.finish();
    Fp::from(v)
}