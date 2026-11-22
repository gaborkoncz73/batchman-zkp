//! The verifier.
//!
//! ```text
//! verify [proof file]... [options]
//! ```
//!
//! Checks shard proofs (by default every `output/shard-<i>.proof.json`)
//! against inputs the verifier controls:
//!
//! ```text
//! --policy prolog/policy.pl                 the public policy
//! --roots input/trusted_roots.json          the goals the verifier wants proved
//! --commitments registry/commitments.json   the fact issuer's published commitments
//! --config config/circuit.json              the circuit sizes
//! --cache output/cache                      the verifier's own IPA parameters between runs
//! --no-cache                                generate the parameters every time
//! ```
//!
//! The verifying keys are generated here from the policy and the sizes, never
//! taken from the prover. Exactly one proof must have the trusted roots, every
//! proof must verify, and the link commitments of the proofs must match.

use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use node_circuit::{ParamsCache, ShardProof, Sizes, VerifierKeys, verify_proofs_with};
use policy_constraints::compile;
use policy_parser::parse_policy;
use proof_witness::{TrustedRoots, parse_commitments};

fn main() -> Result<()> {
    let mut policy = "prolog/policy.pl".to_owned();
    let mut roots = "input/trusted_roots.json".to_owned();
    let mut commitments = "registry/commitments.json".to_owned();
    let mut config = "config/circuit.json".to_owned();
    let mut cache = Some(PathBuf::from("output/cache"));
    let mut files: Vec<PathBuf> = Vec::new();
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        let mut value = || {
            arguments
                .next()
                .with_context(|| format!("{argument} needs a value"))
        };
        match argument.as_str() {
            "--policy" => policy = value()?,
            "--roots" => roots = value()?,
            "--commitments" => commitments = value()?,
            "--config" => config = value()?,
            "--cache" => cache = Some(PathBuf::from(value()?)),
            "--no-cache" => cache = None,
            flag if flag.starts_with("--") => bail!("unknown option {flag}"),
            _ => files.push(PathBuf::from(argument)),
        }
    }
    if files.is_empty() {
        for entry in std::fs::read_dir("output").context("reading output")? {
            let path = entry?.path();
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if name.starts_with("shard-") && name.ends_with(".proof.json") {
                files.push(path);
            }
        }
        files.sort();
        if files.is_empty() {
            bail!("no output/shard-*.proof.json; run `prove all` first or name the proof files");
        }
    }

    let read =
        |path: &str| std::fs::read_to_string(path).with_context(|| format!("reading {path}"));
    let system = compile(&parse_policy(&read(&policy)?, &policy)?)
        .with_context(|| format!("compiling {policy}"))?;
    let sizes = Sizes::from_json(&read(&config)?, &config)?;
    let trusted = TrustedRoots::from_json(&read(&roots)?, &roots)?;
    let published = parse_commitments(&read(&commitments)?, &commitments)?;
    let proofs = files
        .iter()
        .map(|path| {
            let label = path.display().to_string();
            ShardProof::from_json(
                &std::fs::read_to_string(path).with_context(|| format!("reading {label}"))?,
                &label,
            )
        })
        .collect::<Result<Vec<_>>>()?;

    let started = Instant::now();
    let cache = cache.map_or_else(ParamsCache::none, ParamsCache::new);
    let mut keys = VerifierKeys::new(sizes, &trusted).with_cache(cache);
    let verified = verify_proofs_with(&system, &mut keys, &trusted, &published, &proofs)?;
    println!(
        "verified: {} shard proofs, {} links between them, the {} trusted root(s) hold ({:.2?}, key generation included)",
        verified.shards,
        verified.links,
        trusted.goals().count(),
        started.elapsed()
    );
    Ok(())
}
