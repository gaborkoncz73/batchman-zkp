//! The registry of issued external facts (`registry/facts.yaml`), private to
//! the prover, and the list of their commitments (`registry/commitments.json`),
//! public.
//!
//! ```yaml
//! - goal: "rolling_treshold(low,0)"
//!   salt: "1543…"      # a random field element, in decimal
//! ```
//!
//! The commitment of a fact is [`fact_commitment`] of its goal digest and its
//! salt. The verifier sees only the commitments: with random salts they do not
//! reveal which facts they stand for.

use anyhow::{Context, Result, anyhow, bail};
use ff::PrimeField;
use num_bigint::BigUint;
use policy_constraints::Fp;
use policy_constraints::GroundTerm;
use policy_constraints::term::fact_commitment;
use serde::{Deserialize, Serialize};

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

#[derive(Deserialize, Serialize)]
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
            let salt =
                field_element(&entry.salt).map_err(|message| anyhow!("{label}.salt: {message}"))?;
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

    /// The salt of the fact `goal`, if it is in the registry.
    pub fn salt(&self, goal: &GroundTerm) -> Option<Fp> {
        self.facts
            .iter()
            .find(|fact| &fact.goal == goal)
            .map(|fact| fact.salt)
    }

    /// The public commitments, in registry order.
    pub fn commitments(&self) -> Vec<Fp> {
        self.facts
            .iter()
            .map(|fact| fact_commitment(fact.goal.digest(), fact.salt))
            .collect()
    }

    /// The registry as YAML, salts in canonical decimal.
    pub fn to_yaml(&self) -> Result<String> {
        let entries: Vec<Entry> = self
            .facts
            .iter()
            .map(|fact| Entry {
                goal: fact.text.clone(),
                salt: decimal(fact.salt),
            })
            .collect();
        Ok(serde_yaml::to_string(&entries)?)
    }

    /// A registry of the same goals with fresh salts from `salt`.
    pub fn resalted(&self, mut salt: impl FnMut() -> Fp) -> Self {
        let facts = self
            .facts
            .iter()
            .map(|fact| Fact {
                salt: salt(),
                ..fact.clone()
            })
            .collect();
        Self { facts }
    }
}

/// Reads the public commitment list: a JSON array of canonical decimals.
pub fn parse_commitments(source: &str, label: &str) -> Result<Vec<Fp>> {
    let texts: Vec<String> =
        serde_json::from_str(source).with_context(|| format!("{label}: not a commitment list"))?;
    texts
        .iter()
        .enumerate()
        .map(|(index, text)| {
            field_element(text).map_err(|message| anyhow!("{label}[{index}]: {message}"))
        })
        .collect()
}

/// The commitment list as JSON.
pub fn commitments_to_json(commitments: &[Fp]) -> Result<String> {
    let texts: Vec<String> = commitments.iter().map(|value| decimal(*value)).collect();
    Ok(serde_json::to_string_pretty(&texts)?)
}

/// A field element in canonical decimal form.
pub fn decimal(value: Fp) -> String {
    BigUint::from_bytes_le(value.to_repr().as_ref()).to_str_radix(10)
}

/// Parses a field element in canonical decimal form (salts, commitments and
/// the cells of witness files).
pub fn field_element(decimal: &str) -> Result<Fp, &'static str> {
    let integer =
        BigUint::parse_bytes(decimal.as_bytes(), 10).ok_or("not an unsigned decimal integer")?;
    if integer.to_str_radix(10) != decimal {
        return Err("not in canonical decimal form (no sign, no leading zeros)");
    }
    let bytes = integer.to_bytes_le();
    let mut repr = [0_u8; 32];
    if bytes.len() > repr.len() {
        return Err("not smaller than the field modulus");
    }
    repr[..bytes.len()].copy_from_slice(&bytes);
    Option::from(Fp::from_repr(repr)).ok_or("not smaller than the field modulus")
}
