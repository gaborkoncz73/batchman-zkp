use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;

use ark_bn254::{Bn254, Fr};
use ark_ff::{Field, One, Zero};
use ark_groth16::Groth16;
use ark_r1cs_std::{
    alloc::AllocVar,
    eq::EqGadget,
    fields::{fp::FpVar, FieldVar},
};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_snark::{CircuitSpecificSetupSNARK, SNARK};
use rayon::prelude::*; // ⚡ párhuzamos feldolgozás

// ---------------- JSON struktúrák ----------------

#[derive(Debug, Deserialize)]
struct RuleTemplateFile {
    predicates: Vec<PredicateTemplate>,
    facts: Vec<FactTemplate>,
}

#[derive(Debug, Deserialize)]
struct PredicateTemplate {
    name: String,
    arity: usize,
    clauses: Vec<ClauseTemplate>,
}

#[derive(Debug, Deserialize)]
struct ClauseTemplate {
    children: Vec<ChildSig>,
    equalities: Vec<String>, // most nem használjuk; később vihetjük ZK-ba
}

#[derive(Debug, Deserialize)]
struct ChildSig {
    name: String,
    arity: usize,
}

#[derive(Debug, Deserialize)]
struct FactTemplate {
    name: String,
    arity: usize,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ProofNode {
    GoalNode(GoalEntry),
    True(bool),
}

#[derive(Debug, Deserialize)]
struct GoalEntry {
    goal: String,
    goal_term: Term,
    goal_unification: Unification,
    substitution: Vec<String>,
    subtree: Vec<ProofNode>,
}

#[derive(Debug, Deserialize)]
struct Term {
    name: String,
    args: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Unification {
    goal: String,
    body: Vec<serde_json::Value>,
}

// ---------------- Dinamikus predikátum ID-k ----------------
// Célja: a predikátumokat és tényeket egy determinista numerikus azonosítóhoz rendelje
// Így lehet gyors ellenőrzést csinálni, predikátumnevek és aritások között
fn build_predicate_id_map(rules: &RuleTemplateFile) -> BTreeMap<String, i64> {
    let mut map = BTreeMap::<String, i64>::new();
    let mut counter = 1i64;
    // Predikátum
    for p in &rules.predicates {
        map.insert(format!("{}/{}", p.name, p.arity), counter);
        counter += 1;
    }
    // Factek, mert nem mindegyik szerepel explicit a szabálylistában
    for f in &rules.facts {
        let key = format!("{}/{}", f.name, f.arity);
        if !map.contains_key(&key) {
            map.insert(key, counter);
            counter += 1;
        }
    }
    map
}
// Visszaadja a keresertt predikátum ID-jét
fn predicate_id(name: &str, arity: usize, id_map: &BTreeMap<String, i64>) -> i64 {
    let key = format!("{}/{}", name, arity);
    *id_map.get(&key).unwrap_or(&0)
}

// ---------------- Determinisztikus „random” súlyok Batchman-hez ----------------

fn fs_coeffs(seed: &str, m: usize) -> Vec<Fr> {
    use std::hash::{Hash, Hasher};
    let mut out = Vec::with_capacity(m);
    for i in 0..m {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        seed.hash(&mut h);
        i.hash(&mut h);
        let val = (h.finish() % 97) as u64 + 1;
        out.push(Fr::from(val));
    }
    out
}

fn compress_rows(rows: &[Vec<Fr>], r: &[Fr]) -> Vec<Fr> {
    let m = rows[0].len();
    let mut c = vec![Fr::zero(); m];
    for (ri, row) in r.iter().zip(rows.iter()) {
        for j in 0..m {
            c[j] += *ri * row[j];
        }
    }
    c
}

// ---------------- Kisegítő: string → mezőelem (hash nélkül, std hasherrel) ----------------

fn str_to_fr(s: &str) -> Fr {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    let v = h.finish();
    Fr::from(v)
}

// ---------------- ZK Circuit-ek ----------------

// 1) Dot-product: Σ c_i * w_i == 0
struct DotCircuit {
    c_vec: Vec<Fr>, // public
    w_vec: Vec<Fr>, // witness
}

impl ConstraintSynthesizer<Fr> for DotCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let c_var = Vec::<FpVar<Fr>>::new_input(cs.clone(), || Ok(self.c_vec))?;
        let w_var = Vec::<FpVar<Fr>>::new_witness(cs.clone(), || Ok(self.w_vec))?;
        let mut acc = FpVar::<Fr>::zero();
        for (a, b) in c_var.iter().zip(w_var.iter()) {
            acc += a * b;
        }
        acc.enforce_equal(&FpVar::<Fr>::zero())?;
        Ok(())
    }
}

// 2) Szigorú szintaxis-egyezés (név + aritás) külön KÉT kényszerrel, nem összegezve
// A cél: bizonyítani, hogy (name_a == name_b) ÉS (arity_a == arity_b)
// Itt a *_a mezők public, a *_b mezők witness (de lehetne fordítva is).
struct ConsistencyCircuit {
    pub_name: Fr,
    wit_name: Fr,
    pub_arity: Fr,
    wit_arity: Fr,
}

impl ConstraintSynthesizer<Fr> for ConsistencyCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let pub_name  = FpVar::<Fr>::new_input(cs.clone(),  || Ok(self.pub_name))?;
        let pub_arity = FpVar::<Fr>::new_input(cs.clone(),  || Ok(self.pub_arity))?;
        let wit_name  = FpVar::<Fr>::new_witness(cs.clone(),|| Ok(self.wit_name))?;
        let wit_arity = FpVar::<Fr>::new_witness(cs.clone(),|| Ok(self.wit_arity))?;

        // 1) név egyezés (külön constraint)
        (pub_name - wit_name).enforce_equal(&FpVar::<Fr>::zero())?;

        // 2) aritás egyezés (külön constraint)
        (pub_arity - wit_arity).enforce_equal(&FpVar::<Fr>::zero())?;
        Ok(())
    }
}

// ---------------- Univerzum + witness + sorok ----------------

fn local_universe(rules: &RuleTemplateFile, pred_name: &str) -> Vec<String> {
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

fn witness_subtree_presence(goal: &GoalEntry, universe: &[String]) -> Vec<Fr> {
    let present: HashSet<String> = goal
        .subtree
        .iter()
        .filter_map(|n| if let ProofNode::GoalNode(g) = n { Some(g.goal_term.name.clone()) } else { None })
        .collect();
    let mut w: Vec<Fr> = universe
        .iter()
        .map(|u| if present.contains(u) { Fr::one() } else { Fr::zero() })
        .collect();
    w.push(Fr::one()); // konstans oszlop
    w
}

fn rows_structural_global(clause: &ClauseTemplate, universe: &[String]) -> Vec<Vec<Fr>> {
    let expected: HashSet<String> = clause.children.iter().map(|c| c.name.clone()).collect();
    let neg_one = -Fr::one();
    let cols = universe.len() + 1;
    let mut rows = Vec::with_capacity(universe.len());
    for (j, u) in universe.iter().enumerate() {
        let mut v = vec![Fr::zero(); cols];
        v[j] = Fr::one();
        if expected.contains(u) {
            v[cols - 1] = neg_one;
        }
        rows.push(v);
    }
    rows
}

fn rows_body_global(goal: &GoalEntry, universe: &[String]) -> Vec<Vec<Fr>> {
    let body_preds: HashSet<String> = goal
        .goal_unification
        .body
        .iter()
        .filter_map(|v| v.as_str())
        .filter(|s| *s != "true")
        .filter_map(|s| s.split('(').next().map(|p| p.to_string()))
        .collect();
    let neg_one = -Fr::one();
    let cols = universe.len() + 1;
    let mut rows = Vec::with_capacity(universe.len());
    for (j, u) in universe.iter().enumerate() {
        let mut v = vec![Fr::zero(); cols];
        v[j] = Fr::one();
        if body_preds.contains(u) {
            v[cols - 1] = neg_one;
        }
        rows.push(v);
    }
    rows
}

// ---------------- Segéd: goal-string parse ----------------

// Visszatér tupleben a névvel + aritásszámmal
fn parse_predicate_call(s: &str) -> Option<(String, usize)> {
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

// ---------------- ZK bizonyítási lépések ----------------

fn prove_consistency(name_a: &str, arity_a: usize, name_b: &str, arity_b: usize) -> Result<()> {
    // Public: A, Witness: B  (vagy fordítva is lehet)
    let circuit_for_setup = ConsistencyCircuit {
        pub_name:  Fr::zero(),
        wit_name:  Fr::zero(),
        pub_arity: Fr::zero(),
        wit_arity: Fr::zero(),
    };

    let mut rng = ark_std::rand::thread_rng();
    let (pk, vk) = Groth16::<Bn254>::setup(circuit_for_setup, &mut rng)?;

    let pub_name  = str_to_fr(name_a);
    let pub_arity = Fr::from(arity_a as u64);
    let wit_name  = str_to_fr(name_b);
    let wit_arity = Fr::from(arity_b as u64);

    let circuit = ConsistencyCircuit {
        pub_name,
        wit_name,
        pub_arity,
        wit_arity,
    };

    let proof = Groth16::<Bn254>::prove(&pk, circuit, &mut rng)?;
    let ok = Groth16::<Bn254>::verify(&vk, &[pub_name, pub_arity], &proof)?;
    if !ok {
        return Err(anyhow!("ZK consistency verify failed (name/arity mismatch)"));
    }
    Ok(())
}

// ---------------- Fő ZK ellenőrzés (node-onként) ----------------

fn prove_node(
    goal: &GoalEntry,
    rules: &RuleTemplateFile,
    id_map: &BTreeMap<String, i64>,
    //csak kiíráshoz
    depth: usize,
) -> Result<()> {
    let indent = "  ".repeat(depth);
    println!("{}ZK-Batchman check: {}", indent, goal.goal);

    // 0) Szigorú szintaxis ZK-ban:
    // goal.goal  vs  goal_term
    let (g_text_name, g_text_arity) = parse_predicate_call(&goal.goal)
        .ok_or_else(|| anyhow!("goal parse hiba: '{}'", goal.goal))?;
    prove_consistency(
        &g_text_name,
        g_text_arity,
        &goal.goal_term.name,
        goal.goal_term.args.len(),
    )?;

    // goal_unification.goal  vs  goal_term
    let (u_text_name, u_text_arity) = parse_predicate_call(&goal.goal_unification.goal)
        .ok_or_else(|| anyhow!("goal_unification.goal parse hiba: '{}'", goal.goal_unification.goal))?;
    prove_consistency(
        &u_text_name,
        u_text_arity,
        &goal.goal_term.name,
        goal.goal_term.args.len(),
    )?;

    // 1) Ismeretlen predikátum/aritás off-chain gyors check
    let goal_id = predicate_id(&goal.goal_term.name, goal.goal_term.args.len(), id_map);
    if goal_id == 0 {
        return Err(anyhow!(
            "Ismeretlen predikátum/aritás: {}/{}",
            goal.goal_term.name,
            goal.goal_term.args.len()
        ));
    }

    // 2) Fact leaf?
    if let Some(_fact) = rules.facts.iter().find(|f| {
        predicate_id(&f.name, f.arity, id_map) == goal_id
    }) {
        if goal.subtree.iter().any(|n| matches!(n, ProofNode::GoalNode(_))) {
            return Err(anyhow!("fact '{}' subtree-je üres kell hogy legyen", goal.goal));
        }
        println!("{}fact leaf: {}", indent, goal.goal);
        return Ok(());
    }

    // 3) Predikátum jelöltek (név/aritás)
    let pred_matches: Vec<&PredicateTemplate> = rules
        .predicates
        .iter()
        .filter(|p| predicate_id(&p.name, p.arity, id_map) == goal_id)
        .collect();
    if pred_matches.is_empty() {
        return Err(anyhow!("Predikátum nincs a szabályokban: {}", goal.goal_term.name));
    }

    

    // 4) Lokális univerzum és witness
    let universe = local_universe(rules, &goal.goal_term.name);

    println!("universum: {:?}", universe);

    let w_vec = witness_subtree_presence(goal, &universe);

    // 5) Klóz-ellenőrzés (párhuzamos): minden klózra külön dot(c,w) == 0 (NINCS összevonás)
    let any_ok = pred_matches.par_iter().any(|pred| {
        pred.clauses.par_iter().any(|clause| {
            // rows = structural ∪ body
            let mut rows = rows_structural_global(clause, &universe);
            rows.extend(rows_body_global(goal, &universe));
            // seed pl. a predikátum neve + goal + mélység alapján
            let r = fs_coeffs(&format!("{}:{}:{}", pred.name, goal.goal, depth), rows.len());
            let c_vec = compress_rows(&rows, &r);

            if c_vec.len() != w_vec.len() {
                return false;
            }

            // gyors előszűrés, ha nem 0 nem is lesz proof felesleges legenerálni
            
            let dot_debug: Fr = c_vec.iter().zip(&w_vec).map(|(a, b)| *a * *b).sum();
            if !dot_debug.is_zero() {
                return false;
            }

            // ZK: dot == 0 külön bizonyítás
            let mut rng = ark_std::rand::thread_rng();
            let circuit_for_setup = DotCircuit {
                c_vec: vec![Fr::zero(); c_vec.len()],
                w_vec: vec![Fr::zero(); w_vec.len()],
            };
            if let Ok((pk, vk)) = Groth16::<Bn254>::setup(circuit_for_setup, &mut rng) {
                let circuit = DotCircuit {
                    c_vec: c_vec.clone(),
                    w_vec: w_vec.clone(),
                };
                if let Ok(proof) = Groth16::<Bn254>::prove(&pk, circuit, &mut rng) {
                    if let Ok(true) = Groth16::<Bn254>::verify(&vk, &c_vec, &proof) {
                        println!("{}dot(c,w) = 0 (illeszkedik)", indent);
                        return true;
                    }
                }
            }
            false
        })
    });

    if !any_ok {
        return Err(anyhow!(
            "'{}' nem illeszkedik egyik klózra sem (külön ZK dot-proofokkal)",
            goal.goal
        ));
    }

    // 6) Rekurzió a gyerekekre (párhuzamosítható is, de a pk/vk setup per-proof nagy; itt soros)
    for sub in &goal.subtree {
        if let ProofNode::GoalNode(child) = sub {
            prove_node(child, rules, id_map, depth + 1)?;
        }
    }

    println!("{}OK (ZK Batchman + külön szintaxis-ZK): {}", indent, goal.goal);
    Ok(())
}

// ---------------- main ----------------

fn main() -> Result<()> {
    let rules_text = fs::read_to_string("rules_template.json")?;
    let rules: RuleTemplateFile = serde_json::from_str(&rules_text)?;
    let proof_text = fs::read_to_string("proof_tree.json")?;
    let tree: Vec<ProofNode> = serde_json::from_str(&proof_text)?;

    

    let id_map = build_predicate_id_map(&rules);

    println!("A: {:?}", id_map);

    println!(
        "Betöltve {} szabály, {} fact és {} proof node.",
        rules.predicates.len(),
        rules.facts.len(),
        tree.len()
    );

    // gyökerek feldolgozása párhuzamosan
    tree.par_iter()
        .filter_map(|n| if let ProofNode::GoalNode(g) = n { Some(g) } else { None })
        .map(|g| prove_node(g, &rules, &id_map, 0))
        .collect::<Result<Vec<_>>>()?;

    println!("Az egész proof tree ZK-ban (külön dot-proofok + szintaxis-ZK) verifikálva!");
    Ok(())
}
