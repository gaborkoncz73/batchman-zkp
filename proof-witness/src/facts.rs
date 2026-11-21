//! The registry of issued external facts (`issue/src/facts.yaml`).
//!
//! ```yaml
//! - goal: "rolling_treshold(low,0)"
//!   salt: "15414"
//! ```
//!
//! The salt is kept for the registry commitment (step 4); here a fact is only
//! parsed, checked and looked up.

use anyhow::{Context, Result, anyhow, bail};
use ff::PrimeField;
use halo2_proofs::pasta::Fp;
use num_bigint::BigUint;
use policy_constraints::GroundTerm;
use serde::Deserialize;

use crate::goal::ground_goal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact {
    /// The goal as written in the registry.
    pub text: String,
    pub goal: GroundTerm,
    pub salt: Fp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactRegistry {
    facts: Vec<Fact>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    goal: String,
    salt: String,
}

impl FactRegistry {
    pub fn from_yaml(source: &str, label: &str) -> Result<Self> {
        let entries: Vec<Entry> =
            serde_yaml::from_str(source).with_context(|| format!("{label}: not a fact list"))?;
        let mut facts: Vec<Fact> = Vec::with_capacity(entries.len());
        for (index, entry) in entries.into_iter().enumerate() {
            let label = format!("{label}[{index}]");
            let goal = ground_goal(&entry.goal, &format!("{label}.goal"))?;
            if let Some(previous) = facts.iter().position(|fact| fact.goal == goal) {
                bail!("{label}.goal: {} is already fact {previous}", entry.goal);
            }
            let salt = salt(&entry.salt).map_err(|message| anyhow!("{label}.salt: {message}"))?;
            facts.push(Fact {
                text: entry.goal,
                goal,
                salt,
            });
        }
        Ok(Self { facts })
    }

    pub fn facts(&self) -> &[Fact] {
        &self.facts
    }

    pub fn contains(&self, goal: &GroundTerm) -> bool {
        self.facts.iter().any(|fact| &fact.goal == goal)
    }
}

/// A salt is a field element in canonical decimal form.
fn salt(decimal: &str) -> Result<Fp, &'static str> {
    let integer = BigUint::parse_bytes(decimal.as_bytes(), 10)
        .ok_or("the salt is not an unsigned decimal integer")?;
    if integer.to_str_radix(10) != decimal {
        return Err("the salt is not in canonical decimal form (no sign, no leading zeros)");
    }
    let bytes = integer.to_bytes_le();
    let mut repr = [0_u8; 32];
    if bytes.len() > repr.len() {
        return Err("the salt is not smaller than the field modulus");
    }
    repr[..bytes.len()].copy_from_slice(&bytes);
    Option::from(Fp::from_repr(repr)).ok_or("the salt is not smaller than the field modulus")
}
