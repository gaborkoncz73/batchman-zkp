use anyhow::{anyhow, Result};
use halo2_proofs::pasta::group::ff::FromUniformBytes;
use halo2_proofs::plonk::SingleVerifier;
use rand::thread_rng;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::sync::{Arc, Mutex};

/*use ark_bn254::{Bn254, Fr};
use ark_ff::{One, PrimeField, Zero};
use ark_groth16::{prepare_verifying_key, Groth16, PreparedVerifyingKey, Proof, ProvingKey};
use ark_r1cs_std::{
    alloc::AllocVar,
    eq::EqGadget,
    fields::{fp::FpVar, FieldVar},
};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_snark::{CircuitSpecificSetupSNARK, SNARK};*/
use rayon::prelude::*; // párhuzamos feldolgozás

use common;

use halo2_proofs::pasta::{EqAffine, Fp};

pub const MAX_DOT_DIM: usize = 7;
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
use blake2::{Blake2s256, Digest};


fn fs_coeffs(seed: &str, m: usize) -> Vec<Fp> {
    let mut out = Vec::with_capacity(m);
    for i in 0..m {
        let mut h = Blake2s256::new();
        h.update(seed.as_bytes());
        h.update(i.to_le_bytes());
        let hash32 = h.finalize();            // 32 bytes

        let mut wide = [0u8; 64];             // zero-extend to 64 bytes
        wide[..32].copy_from_slice(&hash32);

        let fp = <Fp as FromUniformBytes<64>>::from_uniform_bytes(&wide);
        out.push(fp);
    }
    out
}


fn compress_rows(rows: &[Vec<Fp>], r: &[Fp]) -> Vec<Fp> {
    let m = rows[0].len();
    let mut c = vec![Fp::zero(); m];
    for (ri, row) in r.iter().zip(rows.iter()) {
        for j in 0..m {
            c[j] += *ri * row[j];
        }
    }
    c
}

// string → mezőelem (hash nélkül, std hasherrel)

fn str_to_fp(s: &str) -> Fp {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    let v = h.finish();
    Fp::from(v)
}



// Univerzum + witness + sorok

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

fn witness_subtree_presence(goal: &GoalEntry, universe: &[String]) -> Vec<Fp> {
    let present: HashSet<String> = goal
        .subtree
        .iter()
        .filter_map(|n| if let ProofNode::GoalNode(g) = n { Some(g.goal_term.name.clone()) } else { None })
        .collect();
    let mut w: Vec<Fp> = universe
        .iter()
        .map(|u| if present.contains(u) { Fp::one() } else { Fp::zero() })
        .collect();
    w.push(Fp::one()); // konstans oszlop
    w
}

fn rows_structural_global(clause: &ClauseTemplate, universe: &[String]) -> Vec<Vec<Fp>> {
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


fn prove_consistency(
    name_a: &str,
    arity_a: usize,
    name_b: &str,
    arity_b: usize,
    proofs: &ProofStore,
    pk_store: &Arc<common::ProvingKeyStore>,
) -> Result<()> {
    // Convert inputs to field elements (Fp instead of Fr)
    let pub_name  = str_to_fp(name_a);
    let pub_arity = Fp::from(arity_a as u64);
    let wit_name  = str_to_fp(name_b);
    let wit_arity = Fp::from(arity_b as u64);

    // Generate proof using Halo2/PLONK helper
    let proof = common::prove_consistency(
        pk_store,
        pub_name,
        pub_arity,
        wit_name,
        wit_arity,
    )?;

    // Store the proof in the shared vector
    proofs
        .cons_proofs
        .lock()
        .unwrap()
        .push((proof));

    Ok(())
}

use halo2_proofs::arithmetic::Field;
// ---------------- Fő ZK ellenőrzés (node-onként) ----------------

// A proofs gyűjtő: minden proofhoz mentjük a c_vec (public input) és proof párost
fn prove_node(
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
        .ok_or_else(|| anyhow!("goal parse hiba: '{}'", goal.goal))?;
    let (u_text_name, u_text_arity) = parse_predicate_call(&goal.goal_unification.goal)
        .ok_or_else(|| anyhow!("goal_unification.goal parse hiba: '{}'", goal.goal_unification.goal))?;

    // predikátum ID és fact detection
    let goal_id = predicate_id(&goal.goal_term.name, goal.goal_term.args.len(), id_map);
    if goal_id == 0 {
        return Err(anyhow!(
            "Ismeretlen predikátum/aritás: {}/{}",
            goal.goal_term.name,
            goal.goal_term.args.len()
        ));
    }
    //Tény-e, majd itt jön a fact check
    let is_fact_leaf = rules
        .facts
        .iter()
        .any(|f| predicate_id(&f.name, f.arity, id_map) == goal_id);
    if is_fact_leaf {
        if goal.subtree.iter().any(|n| matches!(n, ProofNode::GoalNode(_)) || goal.goal != goal.goal_unification.goal) {
            return Err(anyhow!("fact '{}' subtree-je nem lehet nem üres vagy nem egyenlő a két goal", goal.goal));
        }
        println!("{}fact leaf: {}", indent, goal.goal);
        return Ok(());
    }

    // külső párhuzamos ágak: (A) consistency proofs  ||  (B) body = subtree check + dot product proof
    let proofs_cons = proofs.clone();
    let proofs_b_join = proofs.clone();
    let pk_store_a = Arc::clone(pk_store);
    let pk_store_b = Arc::clone(pk_store);

    let (syntax_res, clauses_res): (Result<()>, Result<bool>) = rayon::join(
        // (A) consistency proofs: goal és goal_unification egyezése
        || {
            let (r1, r2) = rayon::join(
                || prove_consistency(
                    &g_text_name,
                    g_text_arity.clone(),
                    &goal.goal_term.name,
                    goal.goal_term.args.len(),
                    &proofs_cons,
                    &pk_store_a,
                ),
                || prove_consistency(
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

        // (B) fő ág: fact vagy belső join (pairwise consistency + dot-product proof)
        || {
            // predikátum jelöltek (név/aritás)
            let pred_matches: Vec<&PredicateTemplate> = rules
                .predicates
                .iter()
                .filter(|p| predicate_id(&p.name, p.arity, id_map) == goal_id)
                .collect();
            if pred_matches.is_empty() {
                return Err(anyhow!(
                    "Predikátum nincs a szabályokban: {}",
                    goal.goal_term.name
                ));
            }

            // lokális univerzum + witness
            let universe = local_universe(rules, &goal.goal_term.name);
            let w_vec = witness_subtree_presence(goal, &universe);

            // előzetes hosszellenőrzés (body és subtree)
            let body_len = goal.goal_unification.body.len();
            let subtree_len = goal.subtree.len();
            if body_len != subtree_len {
                return Err(anyhow!(
                    "body/subtree elemszám eltér ({} vs {}) a goalnál: {}",
                    body_len,
                    subtree_len,
                    goal.goal
                ));
            }

            // klónok a belső joinhoz
            let proofs_pairwise = proofs_b_join.clone();
            let pk_cons = Arc::clone(&pk_store_b.cons_pk);
            let proofs_dot = proofs_b_join.clone();
            let pk_dot = Arc::clone(&pk_store_b.dot_pk);

            // BELSŐ JOIN: (B1) body[i] == subtree[i] || (B2) structural dot check 
            let (pair_res, found): (Result<()>, Result<bool>) = rayon::join(
                // (B1) body[i] == subtree[i] consistency check
                || {
                    // előkészítjük a body stringeket
                    let body_strs: Vec<&str> = goal
                        .goal_unification
                        .body
                        .par_iter()
                        .filter_map(|v| v.as_str())
                        .collect();

                    // előkészítjük a subtree goal node-okat
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

                    // zip-elés és párhuzamos consistency proof generálás
                    body_strs
                        .into_par_iter()
                        .zip(sub_goals.into_par_iter())
                        .try_for_each(|(b_str, g)| {
                            let (b_name, b_arity) = parse_predicate_call(b_str)
                                .ok_or_else(|| anyhow!("invalid predicate in body: {}", b_str))?;

                            let s_name = &g.goal_term.name;
                            let s_arity = g.goal_term.args.len();

                            prove_consistency(
                                &b_name,
                                b_arity,
                                s_name,
                                s_arity,
                                &proofs_pairwise,
                                &pk_store_a,
                            )
                        })
                },

                // (B2) structural dot(c,w)=0 proof-keresés
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

                            // proof generálás
                            let mut rng = ark_std::rand::thread_rng();
                            let circuit = common::DotCircuit {
                                c_vec: c_pad.clone(),
                                w_vec: w_pad.clone(),
                            };

                            if let Ok(proof) = common::prove_dot(&pk_store_b, &c_pad, &w_pad) {
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

            // (B1) hibakezelés
            pair_res?;
            // (B2) ha nincs egyező klóz
            if !found? {
                return Err(anyhow!(
                    "'{}' nem illeszkedik egyik klózra sem",
                    goal.goal
                ));
            }

            Ok(true)
        },
    );

    // (A) syntax ág eredmény
    syntax_res?;
    // (B) proof ág eredmény
    if !clauses_res? {
        return Err(anyhow!(
            "'{}' nem illeszkedik egyik klózra sem",
            goal.goal
        ));
    }

    // rekurzió a gyerekekre párhuzamosan
    goal.subtree
        .par_iter()
        .filter_map(|n| if let ProofNode::GoalNode(g) = n { Some(g) } else { None })
        .try_for_each(|child| prove_node(child, rules, id_map, proofs, pk_store, depth + 1))?;

    println!("{}OK (ZK proofok gyűjtve): {}", indent, goal.goal);
    Ok(())
}




type StoredDot = (Vec<Fp>, Vec<u8>);
type StoredCons = Vec<u8>;

#[derive(Clone)]
struct ProofStore {
    dot_proofs: Arc<Mutex<Vec<StoredDot>>>,
    cons_proofs: Arc<Mutex<Vec<StoredCons>>>,
}
fn main() -> Result<()> {
    let rules_text = fs::read_to_string("rules_template.json")?;
    let rules: RuleTemplateFile = serde_json::from_str(&rules_text)?;
    let proof_text = fs::read_to_string("proof_tree.json")?;
    let tree: Vec<ProofNode> = serde_json::from_str(&proof_text)?;

    let id_map = build_predicate_id_map(&rules);

    let proofs = ProofStore {
        dot_proofs: Arc::new(Mutex::new(Vec::new())),
        cons_proofs: Arc::new(Mutex::new(Vec::new())),
    };

    // ✅ create SRS + proving keys
    let pk_store = Arc::new(common::ProvingKeyStore::new(MAX_DOT_DIM,4));

    println!(
        "Betöltve {} szabály, {} fact és {} proof node.",
        rules.predicates.len(),
        rules.facts.len(),
        tree.len()
    );

    tree.par_iter()
        .filter_map(|n| if let ProofNode::GoalNode(g) = n { Some(g) } else { None })
        .map(|g| prove_node(g, &rules, &id_map, &proofs, &pk_store, 0))
        .collect::<Result<Vec<_>>>()?;

    let dot_proofs = proofs.dot_proofs.lock().unwrap().clone();
    let cons_proofs = proofs.cons_proofs.lock().unwrap().clone();
    if !dot_proofs.is_empty() {
    let (c_inputs, proof_bytes) = &dot_proofs[0];
    assert!(
        common::verify_dot(&pk_store, proof_bytes, c_inputs)?,
        "❌ single verify_dot sanity check failed!"
            );
            println!("✅ single verify_dot sanity check passed!");
        }

        if !cons_proofs.is_empty() {
            let proof_bytes = &cons_proofs[0];
            assert!(
                common::verify_consistency(&pk_store, proof_bytes)?,
                "❌ single verify_consistency sanity check failed!"
            );
            println!("✅ single verify_consistency sanity check passed!");
        }
    println!("Batch verify...");

    use halo2_proofs::{
        plonk::{verify_proof},
        transcript::{Blake2bRead, Challenge255},
    };

    let (dot_ok, cons_ok) = rayon::join(
        || {
            dot_proofs.par_iter().all(|(inputs, proof)| {
                let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof[..]);
                let strategy = SingleVerifier::new(&pk_store.params);
                verify_proof(
                    &pk_store.params,
                    &pk_store.dot_vk,
                    strategy,
                    &[ &[ &inputs[..] ] ], // ✅ public inputok megadása
                    &mut transcript,
                ).is_ok()
            })
        },
        || {
            cons_proofs.par_iter().all(|proof| {
                let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof[..]);
                let strategy = SingleVerifier::new(&pk_store.params);
                verify_proof(
                    &pk_store.params,
                    &pk_store.cons_vk,
                    strategy,
                    &[&[]], // ✅ public inputok megadása
                    &mut transcript,
                ).is_ok()
            })
        },
    );


    if dot_ok && cons_ok {
        println!("✅ All proofs verified successfully!");
    } else {
        println!("❌ At least one proof failed verification!");
    }
    Ok(())
}


fn pad(mut v: Vec<Fp>) -> Vec<Fp> {
    let const_col = v.pop().unwrap_or(Fp::one()); // a végén most a konstans van
    while v.len() < MAX_DOT_DIM-1 {
        v.push(Fp::zero());
    }
    v.push(const_col); // visszarakjuk a konstans oszlopot a legvégére
    v
}
