//! Prints the constraint model of a compiled policy.
//!
//! ```text
//! cargo run -p policy-constraints -- [input/rules.json] [--rows]
//! ```

use anyhow::{Context, Result};
use policy_constraints::{BranchKind, compile};
use policy_ir::PolicyFile;

fn main() -> Result<()> {
    let mut path = "input/rules.json".to_owned();
    let mut show_rows = false;
    for argument in std::env::args().skip(1) {
        if argument == "--rows" {
            show_rows = true;
        } else {
            path = argument;
        }
    }

    let source = std::fs::read_to_string(&path).with_context(|| format!("reading {path}"))?;
    let policy: PolicyFile =
        serde_json::from_str(&source).with_context(|| format!("parsing {path}"))?;
    let system = compile(&policy).with_context(|| format!("compiling {path}"))?;

    let layout = system.layout();
    println!(
        "layout: {} cells, {} opened positions, {} child slots",
        layout.width(),
        layout.opened_nodes().len(),
        layout.child_slots()
    );
    for branch in system.branches() {
        let children = match &branch.kind {
            BranchKind::Clause { children, .. } => *children,
            _ => 0,
        };
        println!(
            "branch {:>2}: {} ({children} children, {} rows)",
            branch.id,
            branch.kind,
            branch.rows.len()
        );
        if show_rows {
            for row in &branch.rows {
                println!("    {:<16} {}", format!("{:?}", row.kind), row.description);
            }
        }
    }
    Ok(())
}
