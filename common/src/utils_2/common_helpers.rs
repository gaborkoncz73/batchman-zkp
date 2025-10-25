use halo2_proofs::pasta::Fp;

// MAX Term args len
pub const MAX_ARITY: usize = 2;

// MAX predicates in a unification
pub const MAX_PAIRS: usize = 2;

pub const MAX_SUBTREE_LEN: usize = 2;

pub const MAX_FACTS_HASHES: usize = 5;

pub const MAX_CHILDREN: usize = 3;
pub const MAX_CANDIDATES: usize = 3;
pub const MAX_SIGS: usize = 1 + MAX_CHILDREN;
pub const MAX_SIG_TOKENS: usize = 1 + 2 * (1 + MAX_CHILDREN);
pub const MAX_EQUALITIES: usize = 4;
pub const MAX_CLAUSES: usize = 2;
pub const MAX_PREDICATES: usize = 2;
pub const MAX_FACTS: usize = 5;

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