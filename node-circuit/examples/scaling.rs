//! The measurements of the thesis (step 6c), with the real prover, written
//! as CSV files to `measurements/`:
//!
//! * `threads.csv`: one proof of the energy tree on 1 … 32 threads;
//! * `sharding.csv`, experiment `k_curve`: one synthetic tree split into
//!   shards that fit `2^k` rows, for decreasing `k` (so growing `K`);
//! * `sharding.csv`, experiment `size`: synthetic trees of growing size, in
//!   shards of a fixed `k` and, while it fits, in one proof;
//! * `grid.csv`: the energy tree in shards that fit `2^k` rows for several
//!   `k`, each plan proved with every combination of threads per shard and
//!   shards proved at the same time (at most all threads in use).
//!
//! Every sharded run proves the shards twice: one after the other with all
//! threads (`prove_total_s`, `prove_max_s`: the slowest shard, which is the
//! wall-clock time on `K` such machines), and all at once on this machine,
//! each in a pool of `threads / K` threads (`parallel_wall_s`: an emulation of
//! `K` smaller machines that share the memory bandwidth). Key generation and
//! verification are timed apart; `verify_s` excludes the verifier's key
//! generation.
//!
//! ```text
//! cargo run --release -p node-circuit --example scaling -- [threads|k-curve|size|grid|all] [--reps N]
//! ```
//!
//! The synthetic policy is a binary recursion whose leaves use an issued
//! fact:
//!
//! ```prolog
//! t(0, X) :- base(X).
//! t(N, node(L, R)) :- N > 0, M is N - 1, t(M, L), t(M, R).
//! ```
//!
//! The proof of `t(d, T)` has `5·2^d − 3` nodes: the `t` nodes, their
//! builtins `N > 0` and `M is N − 1`, and the `base(x)` facts.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use ff::Field;
use node_circuit::pipeline::shape;
use node_circuit::{
    ShardKey, ShardProof, Sizes, VerifierKeys, minimum_k, prove_shard, rows_used,
    verify_proofs_with,
};
use policy_constraints::{ConstraintSystem, Fp, compile};
use policy_parser::parse_policy;
use proof_witness::{
    FactRegistry, ProofTree, ProofWitness, Shard, TrustedRoots, build, parse_commitments, plan,
};
use rand_core::OsRng;

const SYNTHETIC: &str =
    "t(0, X) :- base(X).\nt(N, node(L, R)) :- N > 0, M is N - 1, t(M, L), t(M, R).\n";

fn main() -> Result<()> {
    let mut experiment = "all".to_owned();
    let mut reps = 2_usize;
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--reps" => {
                reps = arguments
                    .next()
                    .context("--reps needs a number")?
                    .parse()
                    .context("--reps needs a number")?;
            }
            "threads" | "k-curve" | "size" | "grid" | "all" => experiment = argument,
            other => bail!("unknown argument {other}"),
        }
    }
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../measurements");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("machine.txt"), machine())?;
    let threads = rayon::current_num_threads();

    if matches!(experiment.as_str(), "threads" | "all") {
        thread_scaling(&out, reps)?;
    }
    if matches!(experiment.as_str(), "grid" | "all") {
        grid(&out, reps, threads)?;
    }
    let mut rows = Vec::new();
    if matches!(experiment.as_str(), "k-curve" | "all") {
        let tree = Tree::synthetic(6)?;
        for k in (14..=19).rev() {
            let (sizes, shards) = fit(&tree, k)?;
            for rep in 0..reps {
                let run = run(&tree, sizes, &shards, threads)?;
                rows.push(row("k_curve", &tree, sizes, rep, &run));
            }
        }
    }
    if matches!(experiment.as_str(), "size" | "all") {
        for depth in 3..=7 {
            let tree = Tree::synthetic(depth)?;
            let mut configs = vec![fit(&tree, 15)?];
            // One proof, while it fits 2^19 rows.
            let whole = Sizes {
                capacity: tree.witness.nodes.len(),
                incoming: 0,
                outgoing: 0,
            };
            if minimum_k(&shape(&tree.system, whole, 1)) <= 19 {
                configs.push((whole, tree.plan(whole.capacity)?));
            }
            for (sizes, shards) in configs {
                for rep in 0..reps {
                    let run = run(&tree, sizes, &shards, threads)?;
                    rows.push(row("size", &tree, sizes, rep, &run));
                }
            }
        }
    }
    if !rows.is_empty() {
        let header = "experiment,tree,nodes,k,capacity,incoming,outgoing,shards,links,repetition,\
                      keygen_s,prove_total_s,prove_max_s,parallel_threads,parallel_wall_s,\
                      proof_bytes,verify_s\n";
        let path = out.join("sharding.csv");
        std::fs::write(&path, header.to_owned() + &rows.concat())?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

/// A proof tree with everything a prover and a verifier need.
struct Tree {
    name: String,
    system: ConstraintSystem,
    witness: ProofWitness,
    roots: TrustedRoots,
    commitments: Vec<Fp>,
}

impl Tree {
    fn energy() -> Result<Self> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let read = |path: &str| {
            std::fs::read_to_string(root.join(path)).with_context(|| format!("reading {path}"))
        };
        let system = compile(&parse_policy(&read("prolog/policy.pl")?, "policy.pl")?)?;
        let roots = TrustedRoots::from_json(&read("input/trusted_roots.json")?, "trusted_roots")?;
        let facts = FactRegistry::from_yaml(&read("registry/facts.yaml")?, "facts.yaml")?;
        let tree = ProofTree::from_json(&read("input/proof_tree.json")?, "proof_tree.json")?;
        let witness = build(&system, &tree, &facts, &roots)?;
        let commitments = parse_commitments(&read("registry/commitments.json")?, "commitments")?;
        Ok(Self {
            name: "energy".to_owned(),
            system,
            witness,
            roots,
            commitments,
        })
    }

    /// The proof of `t(depth, T)` in the synthetic policy.
    fn synthetic(depth: usize) -> Result<Self> {
        fn term(depth: usize) -> String {
            if depth == 0 {
                "x".to_owned()
            } else {
                let child = term(depth - 1);
                format!("node({child},{child})")
            }
        }
        fn node(depth: usize) -> String {
            let leaf = |goal: String| format!(r#"{{"goal":"{goal}","subtree":[true]}}"#);
            if depth == 0 {
                return format!(
                    r#"{{"goal":"t(0,x)","subtree":[{}]}}"#,
                    leaf("base(x)".to_owned())
                );
            }
            let child = node(depth - 1);
            format!(
                r#"{{"goal":"t({depth},{})","subtree":[{},{},{child},{child}]}}"#,
                term(depth),
                leaf(format!("{depth}>0")),
                leaf(format!("{} is {depth}-1", depth - 1)),
            )
        }
        let system = compile(&parse_policy(SYNTHETIC, "synthetic.pl")?)?;
        let facts = FactRegistry::from_yaml("- goal: \"base(x)\"\n  salt: \"1\"\n", "facts")?
            .resalted(|| Fp::random(OsRng));
        let roots = TrustedRoots::new(&[format!("t({depth},{})", term(depth))], "roots")?;
        let tree = ProofTree::from_json(&format!("[{}]", node(depth)), "synthetic tree")?;
        let witness = build(&system, &tree, &facts, &roots)?;
        assert_eq!(witness.nodes.len(), 5 * (1 << depth) - 3);
        Ok(Self {
            name: format!("synthetic-d{depth}"),
            system,
            witness,
            roots,
            commitments: facts.commitments(),
        })
    }

    fn plan(&self, capacity: usize) -> Result<Vec<Shard>> {
        plan(&self.system, &self.witness, capacity, || Fp::random(OsRng))
    }
}

/// The largest shard capacity whose shards, with the link places the plan
/// needs, fit in `2^k` rows; one shard if the whole tree fits.
fn fit(tree: &Tree, k: u32) -> Result<(Sizes, Vec<Shard>)> {
    let nodes = tree.witness.nodes.len();
    // Rows per node place: the difference of two circuits large enough that
    // their node rows, not the branch table, set their height.
    let rows = |capacity| {
        rows_used(&shape(
            &tree.system,
            Sizes {
                capacity,
                incoming: 0,
                outgoing: 0,
            },
            1,
        ))
    };
    let per_node = ((rows(16) - rows(8)) / 8).max(1);
    // Start above the largest capacity that can fit, then go down.
    let mut capacity = ((1_usize << k) / per_node + 1).clamp(1, nodes);
    loop {
        let shards = tree.plan(capacity)?;
        let sizes = Sizes {
            capacity,
            incoming: shards
                .iter()
                .map(|shard| shard.incoming.len())
                .max()
                .unwrap_or(0),
            outgoing: shards
                .iter()
                .map(|shard| shard.outgoing.len())
                .max()
                .unwrap_or(0),
        };
        if minimum_k(&shape(&tree.system, sizes, 1)) <= k {
            println!(
                "{} at k = {k}: {} shards of at most {capacity} nodes ({} + {} link places)",
                tree.name,
                shards.len(),
                sizes.incoming,
                sizes.outgoing
            );
            return Ok((sizes, shards));
        }
        if capacity == 1 {
            bail!("{}: not even one node fits 2^{k} rows", tree.name);
        }
        capacity -= 1;
    }
}

/// What one run of a plan cost.
struct Run {
    k: u32,
    shards: usize,
    links: usize,
    keygen: Duration,
    prove_total: Duration,
    prove_max: Duration,
    parallel_threads: usize,
    parallel_wall: Duration,
    proof_bytes: usize,
    verify: Duration,
}

/// Generates the keys, proves every shard one after the other and then all
/// at once, and verifies.
fn run(tree: &Tree, sizes: Sizes, shards: &[Shard], threads: usize) -> Result<Run> {
    let roots = tree.roots.goals().count();
    let started = Instant::now();
    let root_key = ShardKey::generate(&tree.system, sizes, roots)?;
    let other_key = if shards.len() > 1 {
        Some(ShardKey::generate(&tree.system, sizes, 0)?)
    } else {
        None
    };
    let keygen = started.elapsed();
    let key = |index: usize| {
        if index == 0 {
            &root_key
        } else {
            other_key.as_ref().expect("generated for K > 1")
        }
    };
    let prove = |index: usize, shard: &Shard| {
        prove_shard(
            &tree.system,
            sizes,
            key(index),
            index,
            shard,
            &tree.roots,
            &tree.commitments,
        )
    };

    // One after the other, each with every thread.
    let mut proofs: Vec<ShardProof> = Vec::with_capacity(shards.len());
    let mut times = Vec::with_capacity(shards.len());
    for (index, shard) in shards.iter().enumerate() {
        let started = Instant::now();
        proofs.push(prove(index, shard)?);
        times.push(started.elapsed());
    }

    // All at once, each in a pool of its own; in waves if there are more
    // shards than threads.
    let per_shard = (threads / shards.len()).max(1);
    let wave = (threads / per_shard).max(1);
    let started = Instant::now();
    for chunk in shards.iter().enumerate().collect::<Vec<_>>().chunks(wave) {
        std::thread::scope(|scope| -> Result<()> {
            let handles: Vec<_> = chunk
                .iter()
                .map(|&(index, shard)| {
                    scope.spawn(move || -> Result<ShardProof> {
                        let pool = rayon::ThreadPoolBuilder::new()
                            .num_threads(per_shard)
                            .build()?;
                        pool.install(|| prove(index, shard))
                    })
                })
                .collect();
            for handle in handles {
                handle.join().expect("a prover thread panicked")?;
            }
            Ok(())
        })?;
    }
    let parallel_wall = started.elapsed();

    let mut keys = VerifierKeys::generate(&tree.system, sizes, &tree.roots, shards.len() > 1)?;
    let started = Instant::now();
    let verified = verify_proofs_with(
        &tree.system,
        &mut keys,
        &tree.roots,
        &tree.commitments,
        &proofs,
    )?;
    let verify = started.elapsed();

    let run = Run {
        k: root_key.k(),
        shards: shards.len(),
        links: verified.links,
        keygen,
        prove_total: times.iter().sum(),
        prove_max: times.iter().copied().max().unwrap_or_default(),
        parallel_threads: per_shard,
        parallel_wall,
        proof_bytes: proofs.iter().map(|proof| proof.proof.len()).sum(),
        verify,
    };
    println!(
        "  {}: K = {}, k = {}, keys {:.1?}, one after the other {:.2?} (slowest {:.2?}), \
         at once on {} threads each {:.2?}, {} B, verify {:.0?}",
        tree.name,
        run.shards,
        run.k,
        run.keygen,
        run.prove_total,
        run.prove_max,
        run.parallel_threads,
        run.parallel_wall,
        run.proof_bytes,
        run.verify
    );
    Ok(run)
}

fn row(experiment: &str, tree: &Tree, sizes: Sizes, repetition: usize, run: &Run) -> String {
    let seconds = |duration: Duration| format!("{:.3}", duration.as_secs_f64());
    format!(
        "{experiment},{},{},{},{},{},{},{},{},{repetition},{},{},{},{},{},{},{}\n",
        tree.name,
        tree.witness.nodes.len(),
        run.k,
        sizes.capacity,
        sizes.incoming,
        sizes.outgoing,
        run.shards,
        run.links,
        seconds(run.keygen),
        seconds(run.prove_total),
        seconds(run.prove_max),
        run.parallel_threads,
        seconds(run.parallel_wall),
        run.proof_bytes,
        seconds(run.verify),
    )
}

/// Proves every shard with `concurrent` workers, each with a pool of
/// `threads` threads; a worker that is done takes the next shard. Returns
/// the wall-clock time until the last shard is proved.
fn prove_concurrently(
    tree: &Tree,
    sizes: Sizes,
    shards: &[Shard],
    keys: (&ShardKey, Option<&ShardKey>),
    concurrent: usize,
    threads: usize,
) -> Result<Duration> {
    let next = AtomicUsize::new(0);
    let started = Instant::now();
    std::thread::scope(|scope| -> Result<()> {
        let workers: Vec<_> = (0..concurrent)
            .map(|_| {
                scope.spawn(|| -> Result<()> {
                    let pool = rayon::ThreadPoolBuilder::new()
                        .num_threads(threads)
                        .build()?;
                    loop {
                        let index = next.fetch_add(1, Ordering::SeqCst);
                        let Some(shard) = shards.get(index) else {
                            return Ok(());
                        };
                        let key = if index == 0 {
                            keys.0
                        } else {
                            keys.1.expect("generated for K > 1")
                        };
                        pool.install(|| {
                            prove_shard(
                                &tree.system,
                                sizes,
                                key,
                                index,
                                shard,
                                &tree.roots,
                                &tree.commitments,
                            )
                        })?;
                    }
                })
            })
            .collect();
        for worker in workers {
            worker.join().expect("a prover thread panicked")?;
        }
        Ok(())
    })?;
    Ok(started.elapsed())
}

/// The energy tree in shards that fit `2^k` rows, `k = 16 … 13`, each plan
/// proved with `t` threads per shard and `c` shards at a time, for
/// `t = 1, 2, 4, …` and `c = 1, 2, 4, …, K` with `c · t` at most `threads`.
fn grid(out: &Path, reps: usize, threads: usize) -> Result<()> {
    let tree = Tree::energy()?;
    let roots = tree.roots.goals().count();
    let mut csv = String::from(
        "tree,nodes,k,capacity,incoming,outgoing,shards,links,threads_per_shard,\
         concurrent_shards,threads_used,repetition,wall_s\n",
    );
    let mut seen = Vec::new();
    for k in (13..=16).rev() {
        let (sizes, shards) = match fit(&tree, k) {
            Ok(plan) => plan,
            Err(error) => {
                println!("energy at k = {k}: {error}");
                continue;
            }
        };
        if seen.contains(&shards.len()) {
            continue;
        }
        seen.push(shards.len());
        let root_key = ShardKey::generate(&tree.system, sizes, roots)?;
        let other_key = if shards.len() > 1 {
            Some(ShardKey::generate(&tree.system, sizes, 0)?)
        } else {
            None
        };
        let links: usize = shards.iter().map(|shard| shard.incoming.len()).sum();
        let mut concurrencies: Vec<usize> = std::iter::successors(Some(1), |c| Some(c * 2))
            .take_while(|&c| c < shards.len())
            .collect();
        concurrencies.push(shards.len());
        for &concurrent in &concurrencies {
            let mut per_shard = 1;
            while concurrent * per_shard <= threads {
                for repetition in 0..reps {
                    let wall = prove_concurrently(
                        &tree,
                        sizes,
                        &shards,
                        (&root_key, other_key.as_ref()),
                        concurrent,
                        per_shard,
                    )?;
                    println!(
                        "energy k = {}: K = {}, {concurrent} at a time × {per_shard} threads: {wall:.2?}",
                        root_key.k(),
                        shards.len()
                    );
                    writeln!(
                        csv,
                        "energy,{},{},{},{},{},{},{links},{per_shard},{concurrent},{},{repetition},{:.3}",
                        tree.witness.nodes.len(),
                        root_key.k(),
                        sizes.capacity,
                        sizes.incoming,
                        sizes.outgoing,
                        shards.len(),
                        concurrent * per_shard,
                        wall.as_secs_f64()
                    )?;
                }
                per_shard *= 2;
            }
        }
    }
    let path = out.join("grid.csv");
    std::fs::write(&path, csv)?;
    println!("wrote {}", path.display());
    Ok(())
}

/// One proof of the energy tree on 1, 2, 4, … threads.
fn thread_scaling(out: &Path, reps: usize) -> Result<()> {
    let tree = Tree::energy()?;
    let sizes = Sizes {
        capacity: 54,
        incoming: 0,
        outgoing: 0,
    };
    let shards = tree.plan(sizes.capacity)?;
    let key = ShardKey::generate(&tree.system, sizes, tree.roots.goals().count())?;
    let mut csv = String::from("tree,nodes,k,threads,repetition,prove_s\n");
    let mut threads = 1;
    while threads <= rayon::current_num_threads() {
        for repetition in 0..reps {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()?;
            let started = Instant::now();
            pool.install(|| {
                prove_shard(
                    &tree.system,
                    sizes,
                    &key,
                    0,
                    &shards[0],
                    &tree.roots,
                    &tree.commitments,
                )
            })?;
            let elapsed = started.elapsed();
            println!("energy on {threads} threads: {elapsed:.2?}");
            writeln!(
                csv,
                "energy,{},{},{threads},{repetition},{:.3}",
                tree.witness.nodes.len(),
                key.k(),
                elapsed.as_secs_f64()
            )?;
        }
        threads *= 2;
    }
    let path: PathBuf = out.join("threads.csv");
    std::fs::write(&path, csv)?;
    println!("wrote {}", path.display());
    Ok(())
}

/// The machine the measurements ran on.
fn machine() -> String {
    let read = |path: &str| std::fs::read_to_string(path).unwrap_or_default();
    let cpuinfo = read("/proc/cpuinfo");
    let model = cpuinfo
        .lines()
        .find(|line| line.starts_with("model name"))
        .and_then(|line| line.split(':').nth(1))
        .unwrap_or("unknown")
        .trim()
        .to_owned();
    let memory = read("/proc/meminfo")
        .lines()
        .next()
        .unwrap_or("")
        .to_owned();
    let rustc = std::process::Command::new("rustc")
        .arg("-V")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_default();
    format!(
        "cpu: {model}\nlogical cpus: {}\n{memory}\nrayon threads: {}\n{rustc}\nprofile: release\n",
        std::thread::available_parallelism().map_or(0, usize::from),
        rayon::current_num_threads(),
    )
}
