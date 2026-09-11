use std::usize;

use halo2_proofs::pasta::Fp;

//for the MNB example
pub const MAX_PRED_LIST: usize = 4;
pub const MAX_CLAUSES_PER_PREDICATE: usize = 4;
pub const MAX_PREDICATES_OVERALL: usize = 10;
pub const MAX_ARITY: usize = 4;
pub const MAX_FACTS_HASHES: usize = 33;
pub const MAX_EQUALITIES: usize = 11;
pub const MAX_RULE_COMPONENTS: usize = 30;
pub const MAX_CHILDREN: usize = 10;

//These have to be parameterized for the given policy or for the tests
/*pub const MAX_PRED_LIST:usize = 1;
pub const MAX_CLAUSES_PER_PREDICATE: usize = 2;

pub const MAX_PREDICATES_OVERALL: usize = 1;
pub const MAX_ARITY: usize = 2;
pub const MAX_FACTS_HASHES: usize = 59;
pub const MAX_EQUALITIES: usize = 6;

pub const MAX_CHILDREN: usize = 2;
pub const MAX_RULE_COMPONENTS: usize = 30;*/

pub const MAX_CANDIDATES: usize = MAX_PREDICATES_OVERALL * MAX_CLAUSES_PER_PREDICATE;

pub const MAX_SIG_TOKENS: usize = MAX_PRED_LIST * (1 + MAX_ARITY) * (1 + MAX_CHILDREN);

/// Encodes a Prolog atom in a domain that is disjoint from the direct integer
/// encoding used by arithmetic gadgets.
///
/// The BLAKE3-to-field reduction is retained for compatibility with the
/// existing policy/fact fixtures.  What matters here is that an atom whose
/// spelling is numeric (for example `'1'`) is never interpreted as integer 1.
pub fn to_fp_atom(atom: &str) -> Fp {
    let hash = blake3::hash(atom.as_bytes());
    let bytes = hash.as_bytes();

    let limbs: [u64; 4] = [
        u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
        u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
        u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
        u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
    ];
    Fp::from_raw(limbs)
}

/// Compatibility codec for legacy scalar slots.
///
/// Canonical unsigned decimal text denotes an integer. Quoted text denotes an
/// atom even when the text between the quotes is numeric. Every other token is
/// an atom. Callers that already know the semantic type should prefer
/// [`to_fp_atom`] or `Fp::from(integer)` directly.
pub fn to_fp_value(s: &str) -> Fp {
    let s = s.trim();
    if s.len() >= 2 && s.starts_with('\'') && s.ends_with('\'') {
        return to_fp_atom(&s[1..s.len() - 1]);
    }
    if let Ok(value) = s.parse::<u64>() {
        if value.to_string() == s {
            return Fp::from(value);
        }
    }
    to_fp_atom(s)
}
