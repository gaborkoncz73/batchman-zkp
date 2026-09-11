//! Canonical commitment protocol for one authenticated external fact.
//!
//! A fact registry authorizes a canonical typed goal, not a legacy scalar
//! projection of its arguments.  The salt remains an opaque field element;
//! parsing or deriving it belongs to the registry input layer.

use halo2_gadgets::poseidon::primitives::{ConstantLength, Hash as PoseidonHash, P128Pow5T3};
use halo2_proofs::pasta::Fp;
use halo2curves::ff::PrimeField;
use num_bigint::BigUint;

use crate::{typed_term::hash_ground_goal, witness_ir::GroundCall};

/// Version of the typed external-fact hash preimage.
pub const TYPED_FACT_HASH_SCHEMA_VERSION: u32 = 1;

/// Versioned domain separator for typed external-fact commitments.
pub const TYPED_FACT_DOMAIN_TAG: u64 = 0x4241_5454_4641_4301;

/// `[domain, schema_version, typed_goal_digest, salt]`.
pub const TYPED_FACT_HASH_INPUTS: usize = 4;

/// One registry entry after parser and circuit-dialect validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedFact {
    pub goal: GroundCall,
    pub salt: Fp,
}

impl TypedFact {
    pub fn commitment(&self) -> Result<Fp, crate::typed_term::TypedTermError> {
        Ok(typed_fact_hash(hash_ground_goal(&self.goal)?, self.salt))
    }
}

/// Commits one canonical typed goal digest and its private registry salt.
pub fn typed_fact_hash(goal_digest: Fp, salt: Fp) -> Fp {
    PoseidonHash::<Fp, P128Pow5T3, ConstantLength<TYPED_FACT_HASH_INPUTS>, 3, 2>::init().hash([
        Fp::from(TYPED_FACT_DOMAIN_TAG),
        Fp::from(TYPED_FACT_HASH_SCHEMA_VERSION as u64),
        goal_digest,
        salt,
    ])
}

/// Parses a canonical unsigned decimal field element used as a fact salt.
pub fn parse_typed_fact_salt(decimal: &str) -> Result<Fp, String> {
    let Some(integer) = BigUint::parse_bytes(decimal.as_bytes(), 10) else {
        return Err("salt is not an unsigned decimal integer".to_owned());
    };
    if integer.to_str_radix(10) != decimal {
        return Err("salt is not in canonical unsigned decimal form".to_owned());
    }
    let bytes = integer.to_bytes_le();
    if bytes.len() > 32 {
        return Err("salt does not fit the field representation".to_owned());
    }
    let mut repr = [0_u8; 32];
    repr[..bytes.len()].copy_from_slice(&bytes);
    Option::<Fp>::from(Fp::from_repr(repr))
        .ok_or_else(|| "salt is not a canonical field element".to_owned())
}
