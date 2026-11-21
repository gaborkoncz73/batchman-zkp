//! Builds the witness of the repository's proof tree and prints a summary.
//!
//! ```text
//! cargo run -p proof-witness -- [--policy input/rules.json] [--tree input/proof_tree.json]
//!                               [--facts issue/src/facts.yaml] [--roots input/trusted_roots.json]
//!                               [--nodes]
//! ```

use std::collections::BTreeMap;

use anyhow::{Context, Result, bail};
use ff::Field;
use halo2_proofs::pasta::Fp;
use policy_constraints::compile;
use policy_ir::PolicyFile;
use proof_witness::{FactRegistry, ProofTree, TrustedRoots, build, check};
use rand_core::OsRng;

fn main() -> Result<()> {
    let mut policy = "input/rules.json".to_owned();
    let mut tree = "input/proof_tree.json".to_owned();
    let mut facts = "issue/src/facts.yaml".to_owned();
    let mut roots = "input/trusted_roots.json".to_owned();
    let mut show_nodes = false;
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        let target = match argument.as_str() {
            "--nodes" => {
                show_nodes = true;
                continue;
            }
            "--policy" => &mut policy,
            "--tree" => &mut tree,
            "--facts" => &mut facts,
            "--roots" => &mut roots,
            other => bail!("unknown argument {other}"),
        };
        *target = arguments
            .next()
            .with_context(|| format!("{argument} needs a path"))?;
    }
    let read =
        |path: &str| std::fs::read_to_string(path).with_context(|| format!("reading {path}"));

    let policy_file: PolicyFile =
        serde_json::from_str(&read(&policy)?).with_context(|| format!("parsing {policy}"))?;
    let system = compile(&policy_file).with_context(|| format!("compiling {policy}"))?;
    let tree = ProofTree::from_json(&read(&tree)?, &tree)?;
    let facts = FactRegistry::from_yaml(&read(&facts)?, &facts)?;
    let roots = TrustedRoots::from_json(&read(&roots)?, &roots)?;
    let witness = build(&system, &tree, &facts, &roots)?;
    check(&system, &witness, &roots, Fp::random(OsRng)).context("reference check")?;

    println!(
        "{} nodes, {} roots, {} cells per node, {} facts in the registry",
        witness.nodes.len(),
        witness.roots.len(),
        system.layout().width(),
        facts.facts().len()
    );
    println!("reference check passed");
    let mut uses: BTreeMap<usize, usize> = BTreeMap::new();
    for node in &witness.nodes {
        *uses.entry(node.branch).or_default() += 1;
    }
    println!(
        "branches used: {} of {}",
        uses.len(),
        system.branches().len()
    );
    for (branch, count) in &uses {
        println!(
            "  {count:>3} × branch {branch:>2}: {}",
            system.branches()[*branch].kind
        );
    }

    if show_nodes {
        println!();
        for (index, node) in witness.nodes.iter().enumerate() {
            let depth =
                std::iter::successors(node.parent, |&parent| witness.nodes[parent].parent).count();
            println!(
                "{index:>3} {}{}   [branch {}]",
                "  ".repeat(depth),
                node.text,
                node.branch
            );
        }
    }
    Ok(())
}
