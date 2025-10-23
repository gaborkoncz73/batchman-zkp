use std::hash::{Hash, Hasher};
use blake2::{Blake2s256, Digest};
use halo2_proofs::pasta::group::ff::FromUniformBytes;
use halo2_proofs::pasta::Fp;
use halo2curves::ff::PrimeField;
use std::fs;
use crate::{data::RuleTemplateFile, unification_checker_circuit::MAX_DOT_DIM};

// MAX Term args len
pub const MAX_ARITY: usize = 2;

// MAX predicates in a unification
pub const MAX_PAIRS: usize = 2;

pub const MAX_SUBTREE_LEN: usize = 2;

pub const MAX_CHILDREN: usize = 3;
pub const MAX_CANDIDATES: usize = 3;
pub const MAX_SIGS: usize = 1 + MAX_CHILDREN;
pub const MAX_SIG_TOKENS: usize = 1 + 2 * (1 + MAX_CHILDREN);
pub const MAX_EQUALITIES: usize = 4;
pub const MAX_CLAUSES: usize = 2;
pub const MAX_PREDICATES: usize = 2;
pub const MAX_FACTS: usize = 1;

pub fn str_to_fp(
    s: &str
) -> Fp
{
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    let v = h.finish();
    Fp::from(v)
}

pub fn to_fp_value(s: &str) -> Fp {
    // 1️⃣ hash → 32 byte
    let hash = blake3::hash(s.as_bytes());
    let bytes = hash.as_bytes();

    // 2️⃣ 4 darab 64 bites blokk, kis endian konverzióval
    let limbs: [u64; 4] = [
        u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
        u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
        u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
        u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
    ];

    // 3️⃣ Halo2 Fp::from_raw vár egy [u64; 4]-et (kis endian)
    Fp::from_raw(limbs)
}

pub fn pad(
    mut v: Vec<Fp>
) -> Vec<Fp>
{
    let const_col = v.pop().unwrap_or(Fp::one());
    while v.len() < MAX_DOT_DIM - 1 {
        v.push(Fp::zero());
    }
    v.push(const_col);
    v
}

pub fn fs_coeffs(
    seed: &str,
    m: usize
) -> Vec<Fp>
{
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

pub fn compress_rows(
    rows: &[Vec<Fp>],
    r: &[Fp]
) -> Vec<Fp>
{
    if rows.is_empty() {
        // Return empty vector if there are no rows
        return Vec::new();
    }
    let m = rows[0].len();
    let mut c = vec![Fp::zero(); m];
    for (ri, row) in r.iter().zip(rows.iter()) {
        for j in 0..m {
            c[j] += *ri * row[j];
        }
    }
    c
}

pub fn parse_predicate_call(
    s: &str
) -> Option<(String, usize)>
{
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

use halo2_gadgets::poseidon::primitives::{
    ConstantLength, Hash as PoseidonHash, P128Pow5T3,
};

pub fn poseidon_hash_cpu(inputs: &[Fp]) -> Fp {
    // Poseidon hash (P128Pow5T3, WIDTH=3, RATE=2)
    // `ConstantLength` szerint fix hosszú bemenetet vár, így konvertáljuk.
    // A generikus hosszt a bemenet hosszából vesszük.
    match inputs.len() {
        1 => PoseidonHash::<Fp, P128Pow5T3, ConstantLength<1>, 3, 2>::init()
            .hash(inputs.try_into().unwrap()),
        2 => PoseidonHash::<Fp, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
            .hash(inputs.try_into().unwrap()),
        3 => PoseidonHash::<Fp, P128Pow5T3, ConstantLength<3>, 3, 2>::init()
            .hash(inputs.try_into().unwrap()),
        4 => PoseidonHash::<Fp, P128Pow5T3, ConstantLength<4>, 3, 2>::init()
            .hash(inputs.try_into().unwrap()),
        _ => panic!("Unsupported input length for CPU Poseidon hash"),
    }
}

pub fn flatten_rule_template_to_fp(rules: &RuleTemplateFile) -> Vec<Fp> {
    /*let json = fs::read_to_string(path).expect("cannot read file");
    let rules: RuleTemplateFile = serde_json::from_str(&json).expect("bad JSON");*/

    let mut result = Vec::new();

    for pred in &rules.predicates {
        result.push(str_to_fp(&format!("pred:{}", pred.name)));
        result.push(Fp::from(pred.arity as u64));

        for clause in &pred.clauses {
            for child in &clause.children {
                result.push(str_to_fp(&child.name));
                result.push(Fp::from(child.arity as u64));
            }
            for eq in &clause.equalities {
                result.push(Fp::from(eq.left.node as u64));
                result.push(Fp::from(eq.left.arg as u64));
                result.push(Fp::from(eq.right.node as u64));
                result.push(Fp::from(eq.right.arg as u64));
            }
        }
    }

    for fact in &rules.facts {
        result.push(str_to_fp(&fact.name));
        result.push(Fp::from(fact.arity as u64));
    }

    result
}

// --- hash the flattened template ---
pub fn hash_rule_template_poseidon(rules: &RuleTemplateFile) -> Fp {
    let flat = flatten_rule_template_to_fp(rules);

    // if too long, fold pairwise (Poseidon tree)
    let mut layer = flat;
    while layer.len() > 1 {
        let mut next = Vec::new();
        for chunk in layer.chunks(2) {
            let h = if chunk.len() == 2 {
                poseidon_hash_cpu(&[chunk[0], chunk[1]])
            } else {
                chunk[0]
            };
            next.push(h);
        }
        layer = next;
    }
    layer[0]
}

pub fn pad_to_const<const L_MAX: usize>(items: &[Fp]) -> [Fp; L_MAX] {
    println!("items: {:?}", items.len());
    println!("L: {:?}", L_MAX);
    //assert!(items.len() + 2 >= L_MAX, "items too long for L_MAX");
    let mut out = [Fp::zero(); L_MAX];
    // 1) prefix: actual length
    out[0] = Fp::from(items.len() as u64);
    // 2) copy items
    for (i, v) in items.iter().enumerate() {
        if(i != (items.len()-1)){    
            out[1 + i] = *v;
        }
            
    }
    // 3) sentinel at the end
    out[L_MAX - 1] = Fp::one();
    out
}

/// One-shot Poseidon over exactly L_MAX elements (matches the circuit)
pub fn poseidon_hash_cpu_const<const L_MAX: usize>(msg: &[Fp]) -> Fp {
    let arr: [Fp; L_MAX] = pad_to_const::<L_MAX>(msg);
    PoseidonHash::<Fp, P128Pow5T3, ConstantLength<L_MAX>, 3, 2>::init().hash(arr)
}













use num_bigint::BigUint;
use sha2::{ Sha256};

pub fn str_to_fp2(s: &str) -> Fp {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let digest = hasher.finalize(); // 32 bájt

    // Fp::from_uniform_bytes() 64 bájtot vár, ezért kitöltjük nullákkal.
    let mut wide_bytes = [0u8; 64];
    wide_bytes[..32].copy_from_slice(&digest);

    Fp::from_uniform_bytes(&wide_bytes)
}

// ---------------------------
// 2) A te flattenelőddel kompatibilis
//    (használj itt is str_to_fp-et)
// ---------------------------

pub fn flatten_rule_template_to_fp2(rules: &RuleTemplateFile) -> Vec<Fp> {
    let mut out = Vec::new();

    for pred in &rules.predicates {
        out.push(str_to_fp(&format!("pred:{}", pred.name)));
        out.push(Fp::from(pred.arity as u64));

        for clause in &pred.clauses {
            for child in &clause.children {
                out.push(str_to_fp(&child.name));
                out.push(Fp::from(child.arity as u64));
            }
            for eq in &clause.equalities {
                out.push(Fp::from(eq.left.node as u64));
                out.push(Fp::from(eq.left.arg as u64));
                out.push(Fp::from(eq.right.node as u64));
                out.push(Fp::from(eq.right.arg as u64));
            }
        }
    }

    for fact in &rules.facts {
        out.push(str_to_fp(&fact.name));
        out.push(Fp::from(fact.arity as u64));
    }

    out
}

// ---------------------------
// 3) EXACT replika: az első L elemet
//    hash-eljük EGYBEN, egyszer,
//    ConstantLength<L>-lel (mint a circuit)
// ---------------------------
pub fn cpu_rulehash_first_L<const L: usize>(rules: &RuleTemplateFile) -> Fp {
    let flat = flatten_rule_template_to_fp(rules);
    assert!(flat.len() >= L, "flattened rules must have at least L elements");
    let slice = &flat[..L];

    // Egyetlen Poseidon hívás, nem fa, nem folding:
    match L {
        1 => PoseidonHash::<Fp, P128Pow5T3, ConstantLength<1>, 3, 2>::init()
                .hash([slice[0]]),
        2 => PoseidonHash::<Fp, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                .hash([slice[0], slice[1]]),
        3 => PoseidonHash::<Fp, P128Pow5T3, ConstantLength<3>, 3, 2>::init()
                .hash([slice[0], slice[1], slice[2]]),
        4 => PoseidonHash::<Fp, P128Pow5T3, ConstantLength<4>, 3, 2>::init()
                .hash([slice[0], slice[1], slice[2], slice[3]]),
                
        // ha más L is előfordul, bővítsd a match-et:
        _ => panic!("add a match arm for L = {}", L),
    }
}

// ---------------------------
// 4) Kényelmi alias a te esetedre (L=2)
// ---------------------------
pub fn cpu_rulehash_first_2(rules: &RuleTemplateFile) -> Fp {
    cpu_rulehash_first_L::<2>(rules)
}


pub fn poseidon_tree_hash_cpu(leaves: &[Fp]) -> Fp {
    if leaves.is_empty() {
        // ugyanaz az üres-fa konvenció, mint a circuitben
        return PoseidonHash::<Fp, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
            .hash([Fp::zero(), Fp::zero()]);
    }

    let mut layer = leaves.to_vec();
    while layer.len() > 1 {
        let mut next = Vec::with_capacity((layer.len()+1)/2);
        let mut i = 0;
        while i < layer.len() {
            if i + 1 < layer.len() {
                next.push(
                    PoseidonHash::<Fp, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                        .hash([layer[i], layer[i+1]])
                );
                i += 2;
            } else {
                // páratlan: ugyanaz a padoló logika, mint a circuitben!
                next.push(
                    PoseidonHash::<Fp, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                        .hash([layer[i], Fp::zero()])
                );
                i += 1;
            }
        }
        layer = next;
    }
    layer[0]
}