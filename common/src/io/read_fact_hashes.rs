use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use halo2_proofs::pasta::Fp;
use halo2curves::ff::PrimeField;
use num_bigint::BigUint;

use crate::{circuit_dialect::CircuitLimits, utils_2::common_helpers::MAX_FACTS_HASHES};

pub fn read_fact_hashes(path: &Path) -> Result<Vec<Fp>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read fact hashes from {}", path.display()))?;
    parse_fact_hashes(&content)
        .with_context(|| format!("invalid fact hashes in {}", path.display()))
}

pub fn parse_fact_hashes(content: &str) -> Result<Vec<Fp>> {
    let hash_strings: Vec<String> = serde_json::from_str(content)?;
    hash_strings
        .into_iter()
        .enumerate()
        .map(|(index, decimal)| parse_fact_hash(index, &decimal))
        .collect()
}

pub fn validate_fact_hashes_for_current_circuit(
    hashes: &[Fp],
    limits: &CircuitLimits,
) -> Result<()> {
    if limits.max_external_facts > MAX_FACTS_HASHES {
        bail!(
            "circuit limits allow {} external facts, but the current circuit has only {MAX_FACTS_HASHES} public fact-hash rows",
            limits.max_external_facts
        );
    }
    if hashes.len() != MAX_FACTS_HASHES {
        bail!(
            "fact hash input must contain exactly {MAX_FACTS_HASHES} field elements, got {}",
            hashes.len()
        );
    }
    Ok(())
}

fn parse_fact_hash(index: usize, decimal: &str) -> Result<Fp> {
    let Some(integer) = BigUint::parse_bytes(decimal.as_bytes(), 10) else {
        bail!("fact_hashes[{index}] is not an unsigned decimal integer");
    };
    if integer.to_str_radix(10) != decimal {
        bail!("fact_hashes[{index}] is not in canonical decimal form");
    }
    let integer_bytes = integer.to_bytes_le();
    if integer_bytes.len() > 32 {
        bail!("fact_hashes[{index}] does not fit the field representation");
    }
    let mut field_bytes = [0_u8; 32];
    field_bytes[..integer_bytes.len()].copy_from_slice(&integer_bytes);
    Option::<Fp>::from(Fp::from_repr(field_bytes))
        .ok_or_else(|| anyhow::anyhow!("fact_hashes[{index}] is not a canonical field element"))
}
