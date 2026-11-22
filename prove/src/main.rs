//! The prover.
//!
//! ```text
//! prove plan            [options]               # proof tree → output/shard-<i>.witness.json
//! prove shard <file>... [options]               # shard witnesses → output/shard-<i>.proof.json
//! prove all             [options]               # both
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
//! --jobs N                             shards proved at the same time (default:
//!                                      all of them, at most one per thread)
//! --cache output/cache                 where the IPA parameters are kept between runs
//! --no-cache                           generate the parameters every time
//! ```
//!
//! `plan` splits the tree into shards of at most `capacity` nodes and gives
//! every link between them a fresh random salt. A witness file is secret:
//! it holds the shard's assignments and salts, and goes only to the machine
//! that proves the shard. A proof file is public.
//!
//! Before proving, `shard` and `all` generate the keys the shards need: one
//! for the shard with the roots and one for all the others. Both are made
//! from the same IPA parameters (they depend on `k` alone and are most of the
//! cost), which are read from the cache or generated once and stored; then
//! the two keys are generated at the same time, each on half of the threads.
//! Then `N` shards are proved at a time, each on `threads / N` threads; a
//! worker that is done takes the next shard. One proof uses many threads
//! poorly (about 5× on 32 threads), so several smaller proofs at a time
//! finish sooner on the same machine.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use ff::Field;
use node_circuit::{KeySource, ParamsCache, ShardKey, Sizes, circuit_k, prove_shard};
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
    jobs: Option<usize>,
    cache: Option<PathBuf>,
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
        jobs: None,
        cache: Some(PathBuf::from("output/cache")),
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
            "--cache" => options.cache = Some(PathBuf::from(value()?)),
            "--no-cache" => options.cache = None,
            "--jobs" => {
                let jobs: usize = value()?.parse().context("--jobs needs a positive number")?;
                if jobs == 0 {
                    bail!("--jobs needs a positive number");
                }
                options.jobs = Some(jobs);
            }
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
            let files: Vec<PathBuf> = options.files.iter().map(PathBuf::from).collect();
            prove_files(&options, &files)?;
        }
        "all" => {
            let files = write_plan(&options)?;
            prove_files(&options, &files)?;
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

/// The proving keys: for the shard with the roots, and for the others.
struct Keys {
    root: Option<ShardKey>,
    other: Option<ShardKey>,
}

impl Keys {
    /// Generates the keys that are needed, from parameters read from `cache`
    /// or generated once; both keys at the same time, each on half of the
    /// threads, when both are needed.
    fn generate(public: &Public, cache: &ParamsCache, root: bool, other: bool) -> Result<Self> {
        let started = Instant::now();
        let roots = public.roots.goals().count();
        let kinds: Vec<bool> = [(root, true), (other, false)]
            .into_iter()
            .filter_map(|(needed, is_root)| needed.then_some(is_root))
            .collect();
        let k_of = |is_root: bool| {
            circuit_k(
                &public.system,
                public.sizes,
                if is_root { roots } else { 0 },
            )
        };
        // One parameter set per k (the two kinds usually share it).
        let mut params = Vec::new();
        for &is_root in &kinds {
            let k = k_of(is_root);
            if !params.iter().any(|(known, _, _)| *known == k) {
                let (set, source) = cache.params(k)?;
                params.push((k, set, source));
            }
        }
        let parameters = started.elapsed();
        let params_for = |is_root: bool| {
            let k = k_of(is_root);
            params
                .iter()
                .find(|(known, _, _)| *known == k)
                .map(|(_, set, _)| set.clone())
                .expect("loaded above")
        };
        let generate = |is_root: bool| {
            ShardKey::with_params(
                params_for(is_root),
                &public.system,
                public.sizes,
                if is_root { roots } else { 0 },
            )
        };
        let keys = if kinds.len() == 2 {
            let half = (rayon::current_num_threads() / 2).max(1);
            std::thread::scope(|scope| -> Result<Self> {
                let spawn = |is_root: bool| {
                    scope.spawn(move || -> Result<ShardKey> {
                        rayon::ThreadPoolBuilder::new()
                            .num_threads(half)
                            .build()?
                            .install(|| generate(is_root))
                    })
                };
                let (root, other) = (spawn(true), spawn(false));
                let join = |handle: std::thread::ScopedJoinHandle<'_, Result<ShardKey>>| {
                    handle.join().expect("key generation panicked")
                };
                Ok(Self {
                    root: Some(join(root)?),
                    other: Some(join(other)?),
                })
            })?
        } else {
            Self {
                root: root.then(|| generate(true)).transpose()?,
                other: other.then(|| generate(false)).transpose()?,
            }
        };
        let sources: Vec<String> = params
            .iter()
            .map(|(k, _, source)| match source {
                KeySource::Cached => format!("k = {k} parameters from the cache"),
                KeySource::Generated => format!("k = {k} parameters generated"),
            })
            .collect();
        println!(
            "{} ({parameters:.2?}); {} key(s){} in {:.2?} in all",
            sources.join(", "),
            kinds.len(),
            if kinds.len() == 2 {
                " at the same time"
            } else {
                ""
            },
            started.elapsed()
        );
        Ok(keys)
    }

    fn get(&self, root: bool) -> &ShardKey {
        if root { &self.root } else { &self.other }
            .as_ref()
            .expect("generated for every shard")
    }
}

/// Reads the shard witness files, generates the keys they need, and proves
/// them `--jobs` at a time.
fn prove_files(options: &Options, files: &[PathBuf]) -> Result<()> {
    let public = Public::load(options)?;
    let shards = files
        .iter()
        .map(|path| {
            let label = path.display().to_string();
            let source =
                std::fs::read_to_string(path).with_context(|| format!("reading {label}"))?;
            let index = shard_index(path, ".witness.json").unwrap_or(0);
            Ok((index, Shard::from_json(&source, &label)?))
        })
        .collect::<Result<Vec<_>>>()?;
    let has_roots = |shard: &Shard| !shard.witness.roots.is_empty();
    let cache = options
        .cache
        .as_ref()
        .map_or_else(ParamsCache::none, ParamsCache::new);
    let keys = Keys::generate(
        &public,
        &cache,
        shards.iter().any(|(_, shard)| has_roots(shard)),
        shards.iter().any(|(_, shard)| !has_roots(shard)),
    )?;

    let threads = rayon::current_num_threads();
    let jobs = options
        .jobs
        .unwrap_or(shards.len())
        .clamp(1, shards.len().clamp(1, threads));
    let per_job = (threads / jobs).max(1);
    println!(
        "proving {} shard(s), {jobs} at a time on {per_job} thread(s) each",
        shards.len()
    );
    std::fs::create_dir_all(&options.out)?;
    let started = Instant::now();
    let next = AtomicUsize::new(0);
    std::thread::scope(|scope| -> Result<()> {
        let workers: Vec<_> = (0..jobs)
            .map(|_| {
                scope.spawn(|| -> Result<()> {
                    let pool = rayon::ThreadPoolBuilder::new()
                        .num_threads(per_job)
                        .build()?;
                    while let Some((index, shard)) = shards.get(next.fetch_add(1, Ordering::SeqCst))
                    {
                        let shard_started = Instant::now();
                        let proof = pool.install(|| {
                            prove_shard(
                                &public.system,
                                public.sizes,
                                keys.get(has_roots(shard)),
                                *index,
                                shard,
                                &public.roots,
                                &public.commitments,
                            )
                        })?;
                        let out = options.out.join(format!("shard-{index}.proof.json"));
                        std::fs::write(&out, proof.to_json()?)
                            .with_context(|| format!("writing {}", out.display()))?;
                        println!(
                            "shard {index}: {} nodes, proved in {:.2?}, {} bytes: {}",
                            shard.witness.nodes.len(),
                            shard_started.elapsed(),
                            proof.proof.len(),
                            out.display()
                        );
                    }
                    Ok(())
                })
            })
            .collect();
        for worker in workers {
            worker.join().expect("a prover thread panicked")?;
        }
        Ok(())
    })?;
    println!("all shards proved in {:.2?}", started.elapsed());
    Ok(())
}
