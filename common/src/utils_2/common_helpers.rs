use std::usize;

use halo2_proofs::pasta::Fp;

//for the MNB example
pub const MAX_PRED_LIST:usize = 4;
pub const MAX_CLAUSES_PER_PREDICATE: usize = 4;
pub const MAX_PREDICATES_OVERALL: usize = 10;
pub const MAX_ARITY: usize = 4;
pub const MAX_FACTS_HASHES: usize = 33;
pub const MAX_EQUALITIES: usize = 6;
pub const MAX_RULE_COMPONENTS: usize = 30;
pub const MAX_CHILDREN: usize = 10;

//for tests
/*pub const MAX_PRED_LIST:usize = 3;
pub const MAX_CLAUSES_PER_PREDICATE: usize = 2;

pub const MAX_PREDICATES_OVERALL: usize = 1;
pub const MAX_ARITY: usize = 3;
pub const MAX_FACTS_HASHES: usize = 70;
pub const MAX_EQUALITIES: usize = 6;

pub const MAX_CHILDREN: usize = 15;
pub const MAX_RULE_COMPONENTS: usize = 30;*/


pub const MAX_CANDIDATES: usize = MAX_PREDICATES_OVERALL * MAX_CLAUSES_PER_PREDICATE;



pub const MAX_SIG_TOKENS: usize = MAX_PRED_LIST *(1 + MAX_ARITY) * (1 + MAX_CHILDREN);


pub fn to_fp_value(s: &str) -> Fp {
    let s = s.trim().trim_matches('\'');

    // ✅ 1) Próbáljuk számként értelmezni
    if let Ok(v) = s.parse::<u64>() {
        // Hatékony és pontos konverzió az Fp mezőbe
        let fp_val = Fp::from(v);
        return fp_val;
    }

    // ✅ 2) Egyébként HASH → mezőelem (jelenlegi logika)
    let hash = blake3::hash(s.as_bytes());
    let bytes = hash.as_bytes();

    let limbs: [u64; 4] = [
        u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
        u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
        u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
        u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
    ];

    Fp::from_raw(limbs)
}