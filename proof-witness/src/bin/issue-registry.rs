//! The fact issuer: gives every fact of a registry a fresh random salt and
//! publishes the commitments.
//!
//! ```text
//! cargo run -p proof-witness --bin issue-registry -- <facts.yaml> <registry.yaml> <commitments.json>
//! ```
//!
//! `<facts.yaml>` lists the facts (its salts are ignored). `<registry.yaml>`
//! gets the same facts with new salts; it is the prover's secret.
//! `<commitments.json>` gets one commitment per fact; it is public.

use anyhow::{Context, Result, bail};
use ff::Field;
use policy_constraints::Fp;
use proof_witness::{FactRegistry, commitments_to_json};
use rand_core::OsRng;

fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [facts, registry, commitments] = arguments.as_slice() else {
        bail!("usage: issue-registry <facts.yaml> <registry.yaml> <commitments.json>");
    };
    let source = std::fs::read_to_string(facts).with_context(|| format!("reading {facts}"))?;
    let issued = FactRegistry::from_yaml(&source, facts)?.resalted(|| Fp::random(OsRng));
    std::fs::write(registry, issued.to_yaml()?).with_context(|| format!("writing {registry}"))?;
    std::fs::write(
        commitments,
        commitments_to_json(&issued.commitments())? + "\n",
    )
    .with_context(|| format!("writing {commitments}"))?;
    println!(
        "{} facts issued: {registry} (secret), {commitments} (public)",
        issued.facts().len()
    );
    Ok(())
}
