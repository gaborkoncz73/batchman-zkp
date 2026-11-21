//! Helpers shared by the integration tests.

#![allow(dead_code)]

use ff::Field;
use halo2_proofs::pasta::Fp;
use policy_constraints::{Branch, ConstraintSystem, GroundTerm, compile};
use policy_parser::{parse_goal, parse_policy};
use rand_core::OsRng;

pub fn system(source: &str) -> ConstraintSystem {
    compile(&parse_policy(source, "test.pl").expect("policy parses")).expect("policy compiles")
}

pub fn compile_error(source: &str) -> String {
    compile(&parse_policy(source, "test.pl").expect("policy parses"))
        .expect_err("policy is rejected")
        .to_string()
}

pub fn ground(goal: &str) -> GroundTerm {
    GroundTerm::from_term(&parse_goal(goal, "goal").expect("goal parses")).expect("goal is ground")
}

/// The assignment of a node proving `parent` from `children`.
pub fn assignment(system: &ConstraintSystem, parent: &str, children: &[&str]) -> Vec<Fp> {
    let children: Vec<GroundTerm> = children.iter().map(|goal| ground(goal)).collect();
    system
        .layout()
        .assign(&ground(parent), &children)
        .expect("node fits the layout")
}

/// Labels of the branches the assignment satisfies.
pub fn satisfied(system: &ConstraintSystem, assignment: &[Fp]) -> Vec<String> {
    system
        .branches()
        .iter()
        .filter(|branch| branch.is_satisfied_by(assignment))
        .map(|branch| branch.kind.to_string())
        .collect()
}

/// Descriptions of the rows of `branch` that the assignment violates.
pub fn violated(branch: &Branch, assignment: &[Fp]) -> Vec<String> {
    branch
        .rows
        .iter()
        .filter(|row| !row.is_satisfied_by(assignment))
        .map(|row| row.description.clone())
        .collect()
}

/// Evaluates the branch compressed with fresh random weights.
pub fn compressed_residual(system: &ConstraintSystem, branch: &Branch, assignment: &[Fp]) -> Fp {
    let weights: Vec<Fp> = branch.rows.iter().map(|_| Fp::random(OsRng)).collect();
    branch
        .compress(system.layout().width(), &weights)
        .evaluate(assignment)
}
