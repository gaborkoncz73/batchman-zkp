//! Node assignments and branch choices.

use anyhow::{Result, bail};
use ff::Field;
use policy_constraints::Fp;
use policy_constraints::term::symbol;
use policy_constraints::{Branch, BranchKind, ConstraintSystem, GroundTerm, ModelError, RowKind};
use rayon::prelude::*;

use crate::facts::FactRegistry;
use crate::roots::TrustedRoots;
use crate::tree::ProofTree;

/// What the prover knows about one node of the proof tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeWitness {
    /// The goal as written in the proof tree.
    pub text: String,
    pub goal: GroundTerm,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    /// Index into [`ConstraintSystem::branches`].
    pub branch: usize,
    /// The node assignment `w`.
    pub assignment: Vec<Fp>,
}

/// The witness of a whole proof tree, nodes in pre-order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofWitness {
    pub nodes: Vec<NodeWitness>,
    pub roots: Vec<usize>,
}

/// Assigns every node of `tree` and chooses the branch that proves it.
pub fn build(
    system: &ConstraintSystem,
    tree: &ProofTree,
    facts: &FactRegistry,
    trusted: &TrustedRoots,
) -> Result<ProofWitness> {
    check_roots(tree, trusted)?;

    let nodes = tree
        .nodes()
        .par_iter()
        .enumerate()
        .map(|(index, node)| {
            let children: Vec<GroundTerm> = node
                .children
                .iter()
                .map(|&child| tree.nodes()[child].goal.clone())
                .collect();
            let goals = system
                .layout()
                .assign(&node.goal, &children)
                .map_err(|error| anyhow::anyhow!("node {index} {}: {error}", node.text))?;
            let (branch, assignment) = choose_branch(system, facts, &node.goal, &goals)
                .map_err(|reason| anyhow::anyhow!("node {index} {}: {reason}", node.text))?;
            Ok(NodeWitness {
                text: node.text.clone(),
                goal: node.goal.clone(),
                parent: node.parent,
                children: node.children.clone(),
                branch,
                assignment,
            })
        })
        // Collecting the results first makes the reported error the one of
        // the first failing node, whatever order the threads finish in.
        .collect::<Vec<Result<NodeWitness>>>()
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

    for &root in tree.roots() {
        let node = &nodes[root];
        let kind = &system.branches()[node.branch].kind;
        if !matches!(kind, BranchKind::Clause { .. }) {
            bail!(
                "trusted root {} is proved by {kind}, not by a clause of the policy",
                node.text
            );
        }
    }

    Ok(ProofWitness {
        nodes,
        roots: tree.roots().to_vec(),
    })
}

/// The roots of the tree are exactly the trusted goals.
fn check_roots(tree: &ProofTree, trusted: &TrustedRoots) -> Result<()> {
    let root_goals: Vec<&GroundTerm> = tree
        .roots()
        .iter()
        .map(|&root| &tree.nodes()[root].goal)
        .collect();
    for &root in tree.roots() {
        let node = &tree.nodes()[root];
        if !trusted.contains(&node.goal) {
            bail!(
                "the proof tree proves {}, which is not a trusted root",
                node.text
            );
        }
    }
    for (index, goal) in root_goals.iter().enumerate() {
        if root_goals[..index].contains(goal) {
            bail!(
                "the proof tree proves {} twice",
                tree.nodes()[tree.roots()[index]].text
            );
        }
    }
    if let Some(missing) = trusted.goals().position(|goal| !root_goals.contains(&goal)) {
        bail!("trusted root {missing} is not proved by the proof tree");
    }
    Ok(())
}

/// The first branch that proves the node, with the node's assignment
/// completed for it (`goals` holds only the goal cells), or why none does.
fn choose_branch(
    system: &ConstraintSystem,
    facts: &FactRegistry,
    goal: &GroundTerm,
    goals: &[Fp],
) -> Result<(usize, Vec<Fp>), String> {
    let salt = facts.salt(goal).unwrap_or(Fp::ZERO);
    let mut refusals = Vec::new();
    for branch in system.branches() {
        let assignment = match completed(system, branch, goals, salt) {
            Ok(assignment) => assignment,
            Err(error) => {
                // The steps cannot be computed (division by zero, overflow):
                // worth reporting if the call has this branch's form.
                if has_form(branch, goals) {
                    refusals.push(format!("{}: {error}", branch.kind));
                }
                continue;
            }
        };
        if !branch.is_satisfied_by(&assignment) {
            // A call of this form whose result is not the computed one.
            if matches!(branch.kind, BranchKind::Builtin(_)) && has_form(branch, goals) {
                refusals.push(format!("{}: the builtin is false", branch.kind));
            }
            continue;
        }
        match &branch.kind {
            BranchKind::Clause { .. } => return Ok((branch.id, assignment)),
            BranchKind::Builtin(check) => match check.holds(goal) {
                Ok(true) => return Ok((branch.id, assignment)),
                Ok(false) => refusals.push(format!("{}: the builtin is false", branch.kind)),
                Err(error) => refusals.push(format!("{}: {error}", branch.kind)),
            },
            BranchKind::ExternalFact { .. } => {
                if facts.contains(goal) {
                    return Ok((branch.id, assignment));
                }
                refusals.push(format!("{}: not in the fact registry", branch.kind));
            }
        }
    }
    if refusals.is_empty() {
        Err(format!(
            "no branch matches; {}",
            closest(system, goal, goals)
        ))
    } else {
        Err(refusals.join("; "))
    }
}

/// `goals` completed for `branch` (its arithmetic steps and fact cells).
fn completed(
    system: &ConstraintSystem,
    branch: &Branch,
    goals: &[Fp],
    salt: Fp,
) -> Result<Vec<Fp>, ModelError> {
    let mut assignment = goals.to_vec();
    system.complete(branch.id, &mut assignment, salt)?;
    Ok(assignment)
}

/// The branch's rows other than the arithmetic and fact ones hold on `goals`.
fn has_form(branch: &Branch, goals: &[Fp]) -> bool {
    branch
        .rows
        .iter()
        .filter(|row| !matches!(row.kind, RowKind::Arithmetic | RowKind::Fact))
        .all(|row| row.is_satisfied_by(goals))
}

/// Among the branches for the goal's predicate, the one with the fewest
/// violated rows, for the error message.
fn closest(system: &ConstraintSystem, goal: &GroundTerm, goals: &[Fp]) -> String {
    let violated = |branch: &Branch| -> Vec<String> {
        let assignment =
            completed(system, branch, goals, Fp::ZERO).unwrap_or_else(|_| goals.to_vec());
        branch
            .rows
            .iter()
            .filter(|row| !row.is_satisfied_by(&assignment))
            .map(|row| row.description.clone())
            .collect()
    };
    system
        .branches()
        .iter()
        .filter(|branch| {
            let (name, arity) = predicate(&branch.kind);
            symbol(name) == goal.payload() && arity == goal.arity()
        })
        .map(|branch| (branch, violated(branch)))
        .min_by_key(|(_, rows)| rows.len())
        .map_or_else(
            || "the policy has no branch for this predicate".to_owned(),
            |(branch, rows)| format!("closest is {}, violating {}", branch.kind, rows.join(", ")),
        )
}

/// The name and arity of the goals a branch proves.
fn predicate(kind: &BranchKind) -> (&str, usize) {
    match kind {
        BranchKind::Clause { predicate, .. } | BranchKind::ExternalFact { predicate } => {
            (&predicate.name, predicate.arity)
        }
        BranchKind::Builtin(check) => (check.functor(), 2),
    }
}
