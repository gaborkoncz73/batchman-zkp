//! The prover.
//!
//! ```text
//! prove plan            [options]               # proof tree → output/shard-<i>.witness.json
//! prove shard <file>... [options]               # a shard witness → output/shard-<i>.proof.json
//! prove all             [options]               # both, one shard after the other
//! ```
//!
//! Options and their defaults:
//!
//! ```text
//! --policy prolog/policy.pl            the public policy
//! --tree input/proof_tree.json         the proof tree (plan, all)
//! --facts registry/facts.yaml          the issued facts with their salts (plan, all; secret)
//! --roots input/trusted_roots.json     the trusted roots (public)
//! --commitments registry/commitments.json   the published fact commitments (public)
//! --config config/circuit.json         the circuit sizes the verifier uses (public)
//! --out output                         where the files go
//! ```
//!
//! `plan` splits the tree into shards of at most `capacity` nodes and gives
//! every link between them a fresh random salt. A witness file is secret:
//! it holds the shard's assignments and salts, and goes only to the machine
//! that proves the shard. A proof file is public.

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use ff::Field;
use node_circuit::{ShardKey, Sizes, prove_shard};
use policy_constraints::{ConstraintSystem, Fp, compile};
use policy_parser::parse_policy;
use proof_witness::{
    FactRegistry, ProofTree, Shard, TrustedRoots, build, check_shard, parse_commitments, plan,
};
use rand_core::OsRng;

struct Options {
    policy: String,
    tree: String,
    facts: String,
    roots: String,
    commitments: String,
    config: String,
    out: PathBuf,
    files: Vec<String>,
}

fn main() -> Result<()> {
    let mut arguments = std::env::args().skip(1);
    let command = arguments.next().unwrap_or_default();
    let mut options = Options {
        policy: "prolog/policy.pl".to_owned(),
        tree: "input/proof_tree.json".to_owned(),
        facts: "registry/facts.yaml".to_owned(),
        roots: "input/trusted_roots.json".to_owned(),
        commitments: "registry/commitments.json".to_owned(),
        config: "config/circuit.json".to_owned(),
        out: PathBuf::from("output"),
        files: Vec::new(),
    };
    while let Some(argument) = arguments.next() {
        let mut value = || {
            arguments
                .next()
                .with_context(|| format!("{argument} needs a value"))
        };
        match argument.as_str() {
            "--policy" => options.policy = value()?,
            "--tree" => options.tree = value()?,
            "--facts" => options.facts = value()?,
            "--roots" => options.roots = value()?,
            "--commitments" => options.commitments = value()?,
            "--config" => options.config = value()?,
            "--out" => options.out = PathBuf::from(value()?),
            flag if flag.starts_with("--") => bail!("unknown option {flag}"),
            _ => options.files.push(argument),
        }
    }
    match command.as_str() {
        "plan" => {
            write_plan(&options)?;
        }
        "shard" => {
            if options.files.is_empty() {
                bail!("prove shard needs at least one shard witness file");
            }
            let public = Public::load(&options)?;
            let mut keys = Keys::default();
            for file in &options.files {
                prove_file(&options, &public, &mut keys, Path::new(file))?;
            }
        }
        "all" => {
            let files = write_plan(&options)?;
            let public = Public::load(&options)?;
            let mut keys = Keys::default();
            for file in &files {
                prove_file(&options, &public, &mut keys, file)?;
            }
        }
        _ => bail!("usage: prove plan | prove shard <shard witness>... | prove all  [options]"),
    }
    Ok(())
}

fn read(path: &str) -> Result<String> {
    std::fs::read_to_string(path).with_context(|| format!("reading {path}"))
}

fn policy(options: &Options) -> Result<ConstraintSystem> {
    let policy = parse_policy(&read(&options.policy)?, &options.policy)?;
    compile(&policy).with_context(|| format!("compiling {}", options.policy))
}

fn sizes(options: &Options) -> Result<Sizes> {
    Sizes::from_json(&read(&options.config)?, &options.config)
}

/// Splits the tree into shards, checks each like its proof will, and writes
/// the shard witness files (after removing those of an earlier plan).
fn write_plan(options: &Options) -> Result<Vec<PathBuf>> {
    let system = policy(options)?;
    let sizes = sizes(options)?;
    let roots = TrustedRoots::from_json(&read(&options.roots)?, &options.roots)?;
    let commitments = parse_commitments(&read(&options.commitments)?, &options.commitments)?;
    let tree = ProofTree::from_json(&read(&options.tree)?, &options.tree)?;
    let facts = FactRegistry::from_yaml(&read(&options.facts)?, &options.facts)?;
    let witness = build(&system, &tree, &facts, &roots)?;
    let shards = plan(&system, &witness, sizes.capacity, || Fp::random(OsRng))?;
    for (index, shard) in shards.iter().enumerate() {
        if shard.incoming.len() > sizes.incoming || shard.outgoing.len() > sizes.outgoing {
            bail!(
                "shard {index} has {} incoming and {} outgoing links; {} allows {} and {}",
                shard.incoming.len(),
                shard.outgoing.len(),
                options.config,
                sizes.incoming,
                sizes.outgoing
            );
        }
        let trusted = (index == 0).then_some(&roots);
        check_shard(&system, shard, trusted, &commitments, Fp::random(OsRng))
            .with_context(|| format!("shard {index}"))?;
    }

    std::fs::create_dir_all(&options.out)
        .with_context(|| format!("creating {}", options.out.display()))?;
    for entry in std::fs::read_dir(&options.out)? {
        let path = entry?.path();
        if shard_index(&path, ".witness.json").is_some()
            || shard_index(&path, ".proof.json").is_some()
        {
            std::fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
        }
    }
    let mut files = Vec::with_capacity(shards.len());
    for (index, shard) in shards.iter().enumerate() {
        let path = options.out.join(format!("shard-{index}.witness.json"));
        std::fs::write(&path, shard.to_json()?)
            .with_context(|| format!("writing {}", path.display()))?;
        files.push(path);
    }
    let links: usize = shards.iter().map(|shard| shard.incoming.len()).sum();
    println!(
        "{} nodes in {} shards of at most {} nodes, {links} links: {}/shard-*.witness.json (secret)",
        witness.nodes.len(),
        shards.len(),
        sizes.capacity,
        options.out.display()
    );
    Ok(files)
}

/// `n` of a file named `shard-<n><suffix>`.
fn shard_index(path: &Path, suffix: &str) -> Option<usize> {
    path.file_name()?
        .to_str()?
        .strip_prefix("shard-")?
        .strip_suffix(suffix)?
        .parse()
        .ok()
}

/// The public inputs and sizes every shard proof uses.
struct Public {
    system: ConstraintSystem,
    sizes: Sizes,
    roots: TrustedRoots,
    commitments: Vec<Fp>,
}

impl Public {
    fn load(options: &Options) -> Result<Self> {
        Ok(Self {
            system: policy(options)?,
            sizes: sizes(options)?,
            roots: TrustedRoots::from_json(&read(&options.roots)?, &options.roots)?,
            commitments: parse_commitments(&read(&options.commitments)?, &options.commitments)?,
        })
    }
}

/// The proving keys generated so far: for the shard with the roots, and
/// for the others.
#[derive(Default)]
struct Keys {
    root: Option<ShardKey>,
    other: Option<ShardKey>,
}

fn prove_file(options: &Options, public: &Public, keys: &mut Keys, path: &Path) -> Result<()> {
    let label = path.display().to_string();
    let index = shard_index(path, ".witness.json").unwrap_or(0);
    let shard = Shard::from_json(
        &std::fs::read_to_string(path).with_context(|| format!("reading {label}"))?,
        &label,
    )?;
    let root = !shard.witness.roots.is_empty();
    let slot = if root {
        &mut keys.root
    } else {
        &mut keys.other
    };
    if slot.is_none() {
        let started = Instant::now();
        let roots = if root {
            public.roots.goals().count()
        } else {
            0
        };
        let key = ShardKey::generate(&public.system, public.sizes, roots)?;
        println!(
            "key for {} shards: k = {}, {:.2?}",
            if root { "root" } else { "other" },
            key.k(),
            started.elapsed()
        );
        *slot = Some(key);
    }
    let key = slot.as_ref().expect("generated above");
    let started = Instant::now();
    let proof = prove_shard(
        &public.system,
        public.sizes,
        key,
        index,
        &shard,
        &public.roots,
        &public.commitments,
    )?;
    let out = options.out.join(format!("shard-{index}.proof.json"));
    std::fs::create_dir_all(&options.out)?;
    std::fs::write(&out, proof.to_json()?).with_context(|| format!("writing {}", out.display()))?;
    println!(
        "shard {index}: {} nodes, proved in {:.2?}, {} bytes: {}",
        shard.witness.nodes.len(),
        started.elapsed(),
        proof.proof.len(),
        out.display()
    );
    Ok(())
}
