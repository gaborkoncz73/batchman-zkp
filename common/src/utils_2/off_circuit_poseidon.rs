/*use halo2_gadgets::poseidon::primitives::{
    Hash as PoseidonHash, P128Pow5T3, ConstantLength,
};
use halo2_proofs::pasta::Fp;

use crate::utils_2::common_helpers::to_fp_value;


#[inline]
fn poseidon_hash2_native(a: Fp, b: Fp) -> Fp {
    // This matches: Hash::<Fp, _, P128Pow5T3, ConstantLength<2>, 3, 2> in-circuit
    PoseidonHash::<Fp, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                .hash([a, b])
}

// Native version of the chip’s `hash_list` folding:
// acc := 0; for v in values { acc = Poseidon(acc, v) } ; return acc
#[inline]
pub fn poseidon_hash_list_native(values: &[Fp]) -> Fp {
    let mut acc = Fp::zero();
    for &v in values {
        acc = poseidon_hash2_native(acc, v);
    }
    acc
}

/// Public function: hash(name, args, salt) exactly like the chip.
///
/// Inputs:
/// - `name`: predicate/fact name (e.g. "parent")
/// - `args`: predicate args as strings (e.g. ["alice","bob"])
/// - `salt`: Fp salt (convert your BigUint→Fp off-chain the same way you do in-circuit)
///
/// Output:
/// - Fp hash identical to the chip’s Poseidon fold.
pub fn fact_hash_native_salted(name: &str, args: &[&str], salt: &str) -> Fp {
    let mut tokens: Vec<Fp> = Vec::with_capacity(1 + args.len() + 1);
    tokens.push(to_fp_value(name));
    for a in args {
        tokens.push(to_fp_value(a));
    }
    tokens.push(to_fp_value(salt));

    poseidon_hash_list_native(&tokens)
}

pub fn fact_hash_native_term(name: &Fp, args: &[Fp]) -> Fp {
    let mut tokens: Vec<Fp> = Vec::with_capacity(1 + args.len() + 1);
    tokens.push(*name);
    for a in args {
        tokens.push(*a);
    }
    poseidon_hash_list_native(&tokens)
}*/