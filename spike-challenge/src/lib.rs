//! Step 2 spike: Batchman's branch compression in a Halo2 circuit, using a
//! challenge of the PSE fork.
//!
//! Every node of the proof tree has an assignment `w` and secretly picks one
//! branch `i` (a matrix `Mᵢ` of linear rows). The circuit proves, per node,
//!
//! ```text
//! ⟨cv, (w, 1)⟩ = 0      and      cv ∈ { cv₀, …, cv_{B-1} },   cvᵢ = Σₜ cᵗ · Mᵢ[t]
//! ```
//!
//! where `c` is a challenge drawn after `w` is committed. This is
//! `proof_witness::check`'s branch step (③c), and nothing else: no
//! well-formedness, no parent/child digest links, no roots.
//!
//! Layout (one row per node, one row per branch row):
//!
//! ```text
//!              w[0..W]   x[0..=W]       m[0..=W]   q_table q_horner q_last q_node
//!              phase 1   phase 2        fixed
//! table rows   –         Horner acc.    Mᵢ[t]      first   not last  last    –
//! node rows    w         cv (claimed)   –          –       –         –       1
//! ```
//!
//! The table is built in the circuit: `x` at the first row of a branch block is
//! `Mᵢ[0] + c·(Mᵢ[1] + c·(…))`, enforced row by row from the fixed `Mᵢ`. A node
//! row looks its `(1, x)` up among the `(q_table, x)` tuples; the leading `1`
//! keeps the all-zero tuples of the non-first table rows out of reach.

use ff::Field;
use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::halo2curves::pasta::Fp;
use halo2_proofs::plonk::{
    Advice, Challenge, Circuit, Column, ConstraintSystem, Error, Expression, FirstPhase, Fixed,
    SecondPhase, Selector,
};
use halo2_proofs::poly::Rotation;
use policy_constraints::ConstraintSystem as Policy;
use proof_witness::ProofWitness;

/// The branch matrices, dense: every row is `width` coefficients and a constant.
#[derive(Debug, Clone)]
pub struct BranchTable {
    width: usize,
    matrices: Vec<Vec<Vec<Fp>>>,
}

impl BranchTable {
    /// `matrices[i][t]` is row `t` of branch `i`: `width` coefficients, then
    /// the constant. A branch without rows becomes one zero row.
    pub fn new(width: usize, matrices: Vec<Vec<Vec<Fp>>>) -> Self {
        let matrices = matrices
            .into_iter()
            .map(|matrix| {
                assert!(matrix.iter().all(|row| row.len() == width + 1));
                if matrix.is_empty() { vec![vec![Fp::ZERO; width + 1]] } else { matrix }
            })
            .collect();
        Self { width, matrices }
    }

    pub fn from_policy(policy: &Policy) -> Self {
        let width = policy.layout().width();
        let matrices = policy
            .branches()
            .iter()
            .map(|branch| {
                branch
                    .rows
                    .iter()
                    .map(|row| {
                        let mut dense = vec![Fp::ZERO; width + 1];
                        for &(column, coefficient) in &row.terms {
                            dense[column] += coefficient;
                        }
                        dense[width] = row.constant;
                        dense
                    })
                    .collect()
            })
            .collect();
        Self::new(width, matrices)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn branches(&self) -> usize {
        self.matrices.len()
    }

    /// Rows the in-circuit table takes.
    pub fn rows(&self) -> usize {
        self.matrices.iter().map(Vec::len).sum()
    }

    /// `cvᵢ = Σₜ cᵗ · Mᵢ[t]`, the same weights as `proof_witness::check`.
    pub fn compress(&self, branch: usize, challenge: Fp) -> Vec<Fp> {
        horner(&self.matrices[branch], challenge).swap_remove(0)
    }
}

/// `acc[t] = M[t] + c · acc[t+1]`, `acc[last] = M[last]`.
fn horner(matrix: &[Vec<Fp>], challenge: Fp) -> Vec<Vec<Fp>> {
    let mut acc = matrix.to_vec();
    for t in (0..acc.len() - 1).rev() {
        for k in 0..acc[t].len() {
            let next = acc[t + 1][k];
            acc[t][k] += challenge * next;
        }
    }
    acc
}

/// What a node row claims as its compressed branch.
#[derive(Debug, Clone)]
pub enum Claim {
    /// The honest claim: `cv` of this branch, computed with the challenge.
    Branch(usize),
    /// A fixed vector, independent of the challenge (for cheating tests).
    Forged(Vec<Fp>),
}

#[derive(Debug, Clone)]
pub struct NodeInput {
    pub assignment: Vec<Fp>,
    pub claim: Claim,
}

#[derive(Debug, Clone)]
pub struct BranchCircuit {
    table: BranchTable,
    node_count: usize,
    nodes: Value<Vec<NodeInput>>,
}

impl BranchCircuit {
    pub fn new(table: BranchTable, nodes: Vec<NodeInput>) -> Self {
        assert!(nodes.iter().all(|node| node.assignment.len() == table.width));
        Self {
            node_count: nodes.len(),
            nodes: Value::known(nodes),
            table,
        }
    }

    pub fn from_witness(policy: &Policy, witness: &ProofWitness) -> Self {
        let nodes = witness
            .nodes
            .iter()
            .map(|node| NodeInput {
                assignment: node.assignment.clone(),
                claim: Claim::Branch(node.branch),
            })
            .collect();
        Self::new(BranchTable::from_policy(policy), nodes)
    }

    /// Rows used: the table, then one per node.
    pub fn rows(&self) -> usize {
        self.table.rows() + self.node_count
    }
}

#[derive(Debug, Clone)]
pub struct BranchConfig {
    w: Vec<Column<Advice>>,
    x: Vec<Column<Advice>>,
    m: Vec<Column<Fixed>>,
    q_table: Selector,
    q_horner: Selector,
    q_last: Selector,
    q_node: Selector,
    challenge: Challenge,
}

impl Circuit<Fp> for BranchCircuit {
    type Config = BranchConfig;
    type FloorPlanner = SimpleFloorPlanner;
    /// The width `W` of the node assignment.
    type Params = usize;

    fn without_witnesses(&self) -> Self {
        Self {
            table: self.table.clone(),
            node_count: self.node_count,
            nodes: Value::unknown(),
        }
    }

    fn params(&self) -> usize {
        self.table.width
    }

    fn configure(_: &mut ConstraintSystem<Fp>) -> BranchConfig {
        unreachable!("the width comes from configure_with_params")
    }

    fn configure_with_params(meta: &mut ConstraintSystem<Fp>, width: usize) -> BranchConfig {
        let w: Vec<_> = (0..width).map(|_| meta.advice_column_in(FirstPhase)).collect();
        let challenge = meta.challenge_usable_after(FirstPhase);
        let x: Vec<_> = (0..=width).map(|_| meta.advice_column_in(SecondPhase)).collect();
        let m: Vec<_> = (0..=width).map(|_| meta.fixed_column()).collect();
        let q_table = meta.complex_selector();
        let q_node = meta.complex_selector();
        let q_horner = meta.selector();
        let q_last = meta.selector();

        meta.create_gate("table: x = M + c · x_next", |meta| {
            let q = meta.query_selector(q_horner);
            let c = meta.query_challenge(challenge);
            (0..=width)
                .map(|k| {
                    let cur = meta.query_advice(x[k], Rotation::cur());
                    let next = meta.query_advice(x[k], Rotation::next());
                    let row = meta.query_fixed(m[k], Rotation::cur());
                    q.clone() * (cur - (row + c.clone() * next))
                })
                .collect::<Vec<_>>()
        });

        meta.create_gate("table: x = M on the last row of a branch", |meta| {
            let q = meta.query_selector(q_last);
            (0..=width)
                .map(|k| {
                    let cur = meta.query_advice(x[k], Rotation::cur());
                    let row = meta.query_fixed(m[k], Rotation::cur());
                    q.clone() * (cur - row)
                })
                .collect::<Vec<_>>()
        });

        meta.create_gate("node: ⟨cv, (w, 1)⟩ = 0", |meta| {
            let q = meta.query_selector(q_node);
            let constant = meta.query_advice(x[width], Rotation::cur());
            let dot = (0..width).fold(constant, |sum, k| {
                sum + meta.query_advice(x[k], Rotation::cur())
                    * meta.query_advice(w[k], Rotation::cur())
            });
            vec![q * dot]
        });

        meta.lookup_any("node: cv is a branch of the table", |meta| {
            let q_node = meta.query_selector(q_node);
            let q_table = meta.query_selector(q_table);
            let mut pairs: Vec<(Expression<Fp>, Expression<Fp>)> =
                vec![(q_node.clone(), q_table.clone())];
            for &column in &x {
                let value = meta.query_advice(column, Rotation::cur());
                pairs.push((q_node.clone() * value.clone(), q_table.clone() * value));
            }
            pairs
        });

        BranchConfig {
            w,
            x,
            m,
            q_table,
            q_horner,
            q_last,
            q_node,
            challenge,
        }
    }

    fn synthesize(&self, config: BranchConfig, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        // Unknown while the prover synthesizes phase 1, known in phase 2.
        let challenge = layouter.get_challenge(config.challenge);
        let width = self.table.width;

        layouter.assign_region(
            || "branch table",
            |mut region| {
                let mut offset = 0;
                for matrix in &self.table.matrices {
                    let acc = challenge.map(|c| horner(matrix, c));
                    config.q_table.enable(&mut region, offset)?;
                    for (t, row) in matrix.iter().enumerate() {
                        let selector = if t + 1 < matrix.len() { config.q_horner } else { config.q_last };
                        selector.enable(&mut region, offset + t)?;
                        for k in 0..=width {
                            region.assign_fixed(|| "M", config.m[k], offset + t, || Value::known(row[k]))?;
                            region.assign_advice(
                                || "acc",
                                config.x[k],
                                offset + t,
                                || acc.as_ref().map(|acc| acc[t][k]),
                            )?;
                        }
                    }
                    offset += matrix.len();
                }
                Ok(())
            },
        )?;

        layouter.assign_region(
            || "nodes",
            |mut region| {
                let claimed: Value<Vec<Vec<Fp>>> = self.nodes.as_ref().zip(challenge).map(|(nodes, c)| {
                    nodes
                        .iter()
                        .map(|node| match &node.claim {
                            Claim::Branch(branch) => self.table.compress(*branch, c),
                            Claim::Forged(vector) => vector.clone(),
                        })
                        .collect()
                });
                for n in 0..self.node_count {
                    config.q_node.enable(&mut region, n)?;
                    for k in 0..width {
                        region.assign_advice(
                            || "w",
                            config.w[k],
                            n,
                            || self.nodes.as_ref().map(|nodes| nodes[n].assignment[k]),
                        )?;
                    }
                    for k in 0..=width {
                        region.assign_advice(
                            || "cv",
                            config.x[k],
                            n,
                            || claimed.as_ref().map(|claimed| claimed[n][k]),
                        )?;
                    }
                }
                Ok(())
            },
        )
    }
}

/// The repository's policy and the witness of its proof tree (built and
/// checked by `proof-witness`), read from the repository root.
pub fn repository_inputs() -> anyhow::Result<(Policy, ProofWitness)> {
    use anyhow::Context;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let read = |path: &str| {
        std::fs::read_to_string(root.join(path)).with_context(|| format!("reading {path}"))
    };
    let policy_file: policy_ir::PolicyFile = serde_json::from_str(&read("input/rules.json")?)?;
    let policy = policy_constraints::compile(&policy_file)?;
    let tree = proof_witness::ProofTree::from_json(&read("input/proof_tree.json")?, "proof_tree.json")?;
    let facts = proof_witness::FactRegistry::from_yaml(&read("issue/src/facts.yaml")?, "facts.yaml")?;
    let roots =
        proof_witness::TrustedRoots::from_json(&read("input/trusted_roots.json")?, "trusted_roots.json")?;
    let witness = proof_witness::build(&policy, &tree, &facts, &roots)?;
    Ok((policy, witness))
}

/// The smallest `k` with room for `rows` plus the prover's blinding rows.
pub fn minimum_k(rows: usize) -> u32 {
    // Generous margin for the blinding rows of every phase.
    let needed = rows + 16;
    (needed.next_power_of_two().trailing_zeros()).max(4)
}
