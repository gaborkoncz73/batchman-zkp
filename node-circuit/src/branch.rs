//! Step 3b: the branch check, the circuit form of `proof_witness::check`'s
//! compressed-branch step, with the branch chosen in secret.
//!
//! Every branch `i` is a matrix `Mᵢ` of linear rows over `(w, 1)`. With the
//! challenge `c` (drawn after `w` is committed) a branch compresses to the
//! vector `cvᵢ = Σₜ cᵗ · Mᵢ[t]`, and with the challenge `γ` (drawn after the
//! claimed `cv` is committed) a vector compresses to the number
//! `Σⱼ γʲ · cv[j]`. Per node the circuit proves
//!
//! ```text
//! ⟨cv, (w, 1)⟩ = 0      and      Σⱼ γʲ·cv[j]  ∈  { Tᵢ },   Tᵢ = Σ_{(t,j)} Mᵢ[t][j] · cᵗ · γʲ
//! ```
//!
//! so `cv` is one of the `cvᵢ` (else it matches a `Tᵢ` with probability at
//! most `B·(W+1)/p`), and `w` satisfies every row of that branch (else the
//! dot product is zero with probability at most `rows/p`).
//!
//! Three phases: `w` → `c` → `cv`, `acc`, powers of `c` → `γ` → `h`, powers of
//! `γ`, the table `Tᵢ`. The columns are shared by role:
//!
//! ```text
//!          phase   node rows (j = 0..=W)   table rows (one per entry)   power rows
//! w        1       w[j]                    –                            –
//! x2       2       cv[j]                   cᵗ (looked up)               c⁰, c¹, …
//! acc      2       Σ_{j'<j} cv·w           –                            –
//! x3       3       h[j] (Horner in γ)      γʲ (looked up)               γ⁰, γ¹, …
//! sum      3       –                       running Tᵢ                   –
//! index    fixed   –                       t                            exponent
//! cell     fixed   –                       j                            –
//! coeff.   fixed   –                       Mᵢ[t][j]                     –
//! ```

use ff::{Field, PrimeField};
use halo2_proofs::circuit::{AssignedCell, Layouter, Value};
use halo2_proofs::plonk::{
    Advice, Challenge, Column, ConstraintSystem, Error, Expression, FirstPhase, Fixed, SecondPhase,
    Selector, ThirdPhase, VirtualCells,
};
use halo2_proofs::poly::Rotation;
use policy_constraints::{ConstraintSystem as Policy, Fp};

/// One non-zero entry `Mᵢ[row][cell]` of a branch matrix; `cell == width` is
/// the constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry<F> {
    pub row: usize,
    pub cell: usize,
    pub coefficient: F,
}

/// The branches as lists of their non-zero entries.
#[derive(Debug, Clone)]
pub struct BranchTable<F> {
    width: usize,
    branches: Vec<Vec<Entry<F>>>,
    max_row: usize,
}

impl<F: PrimeField> BranchTable<F> {
    /// `branches[i]` are the entries of branch `i` over `width` cells. A
    /// branch without entries becomes one zero entry (it holds for every `w`).
    pub fn new(width: usize, branches: Vec<Vec<Entry<F>>>) -> Self {
        let branches: Vec<Vec<Entry<F>>> = branches
            .into_iter()
            .map(|entries| {
                assert!(entries.iter().all(|entry| entry.cell <= width));
                if entries.is_empty() {
                    vec![Entry {
                        row: 0,
                        cell: 0,
                        coefficient: F::ZERO,
                    }]
                } else {
                    entries
                }
            })
            .collect();
        let max_row = branches
            .iter()
            .flatten()
            .map(|entry| entry.row)
            .max()
            .unwrap_or(0);
        Self {
            width,
            branches,
            max_row,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn branches(&self) -> usize {
        self.branches.len()
    }

    /// Rows of the in-circuit table: one per entry.
    pub fn entries(&self) -> usize {
        self.branches.iter().map(Vec::len).sum()
    }

    /// `cvᵢ = Σₜ cᵗ · Mᵢ[t]`, `width + 1` long.
    pub fn compress(&self, branch: usize, c: F) -> Vec<F> {
        let mut cv = vec![F::ZERO; self.width + 1];
        for entry in &self.branches[branch] {
            cv[entry.cell] += entry.coefficient * c.pow_vartime([entry.row as u64]);
        }
        cv
    }

    /// `Tᵢ = Σⱼ γʲ · cvᵢ[j]`.
    pub fn fold(&self, branch: usize, c: F, gamma: F) -> F {
        self.branches[branch].iter().fold(F::ZERO, |sum, entry| {
            sum + entry.coefficient
                * c.pow_vartime([entry.row as u64])
                * gamma.pow_vartime([entry.cell as u64])
        })
    }
}

impl BranchTable<Fp> {
    /// The branches of a compiled policy.
    pub fn from_policy(policy: &Policy) -> Self {
        let width = policy.layout().width();
        let branches = policy
            .branches()
            .iter()
            .map(|branch| {
                let mut entries = Vec::new();
                for (row, linear) in branch.rows.iter().enumerate() {
                    for &(cell, coefficient) in &linear.terms {
                        entries.push(Entry {
                            row,
                            cell,
                            coefficient,
                        });
                    }
                    if linear.constant != Fp::ZERO {
                        entries.push(Entry {
                            row,
                            cell: width,
                            coefficient: linear.constant,
                        });
                    }
                }
                entries
            })
            .collect();
        Self::new(width, branches)
    }
}

/// What a node claims as its compressed branch.
#[derive(Debug, Clone)]
pub enum Claim<F> {
    /// The honest claim: `cvᵢ` of this branch, computed with the challenge.
    Branch(usize),
    /// A fixed vector, independent of the challenge (for cheating tests).
    Forged(Vec<F>),
}

/// The prover's input for one node: its assignment and its claim.
#[derive(Debug, Clone)]
pub struct NodeBranch<F> {
    pub assignment: Vec<F>,
    pub claim: Claim<F>,
}

#[derive(Debug, Clone)]
pub struct BranchConfig {
    w: Column<Advice>,
    x2: Column<Advice>,
    acc: Column<Advice>,
    x3: Column<Advice>,
    sum: Column<Advice>,
    index: Column<Fixed>,
    cell: Column<Fixed>,
    coefficient: Column<Fixed>,
    // Node rows.
    q_first: Selector,
    q_dot: Selector,
    q_last: Selector,
    // Table rows.
    q_entry: Selector,
    q_entry_first: Selector,
    q_entry_next: Selector,
    q_branch_end: Selector,
    // Power rows.
    q_c_power: Selector,
    q_c_power_first: Selector,
    q_c_power_next: Selector,
    q_gamma_power: Selector,
    q_gamma_power_first: Selector,
    q_gamma_power_next: Selector,
    c: Challenge,
    gamma: Challenge,
}

impl BranchConfig {
    pub fn configure<F: PrimeField>(meta: &mut ConstraintSystem<F>) -> Self {
        let w = meta.advice_column_in(FirstPhase);
        meta.enable_equality(w);
        let c = meta.challenge_usable_after(FirstPhase);
        let x2 = meta.advice_column_in(SecondPhase);
        let acc = meta.advice_column_in(SecondPhase);
        let gamma = meta.challenge_usable_after(SecondPhase);
        let x3 = meta.advice_column_in(ThirdPhase);
        let sum = meta.advice_column_in(ThirdPhase);
        let index = meta.fixed_column();
        let cell = meta.fixed_column();
        let coefficient = meta.fixed_column();

        let q_first = meta.complex_selector();
        let q_dot = meta.selector();
        let q_last = meta.selector();
        let q_entry = meta.complex_selector();
        let q_entry_first = meta.selector();
        let q_entry_next = meta.selector();
        let q_branch_end = meta.complex_selector();
        let q_c_power = meta.complex_selector();
        let q_c_power_first = meta.selector();
        let q_c_power_next = meta.selector();
        let q_gamma_power = meta.complex_selector();
        let q_gamma_power_first = meta.selector();
        let q_gamma_power_next = meta.selector();

        let advice = |meta: &mut VirtualCells<'_, F>, column: Column<Advice>, at: Rotation| {
            meta.query_advice(column, at)
        };

        meta.create_gate("node: the dot product starts at zero", |meta| {
            let q = meta.query_selector(q_first);
            vec![q * advice(meta, acc, Rotation::cur())]
        });

        meta.create_gate("node: dot product and Horner steps", |meta| {
            let q = meta.query_selector(q_dot);
            let gamma = meta.query_challenge(gamma);
            let cv = advice(meta, x2, Rotation::cur());
            let value = advice(meta, w, Rotation::cur());
            let acc_cur = advice(meta, acc, Rotation::cur());
            let acc_next = advice(meta, acc, Rotation::next());
            let h_cur = advice(meta, x3, Rotation::cur());
            let h_next = advice(meta, x3, Rotation::next());
            vec![
                q.clone() * (acc_next - acc_cur - cv.clone() * value),
                q * (h_cur - cv - gamma * h_next),
            ]
        });

        meta.create_gate("node: ⟨cv, (w, 1)⟩ = 0", |meta| {
            let q = meta.query_selector(q_last);
            let constant = advice(meta, x2, Rotation::cur());
            let acc = advice(meta, acc, Rotation::cur());
            let h = advice(meta, x3, Rotation::cur());
            vec![q.clone() * (acc + constant.clone()), q * (h - constant)]
        });

        // One gate per selector: MockProver wants every cell a gate queries
        // to be assigned wherever the gate is on.
        let term = |meta: &mut VirtualCells<'_, F>| {
            meta.query_fixed(coefficient, Rotation::cur())
                * advice(meta, x2, Rotation::cur())
                * advice(meta, x3, Rotation::cur())
        };
        meta.create_gate(
            "table: the first entry of a branch starts the sum",
            |meta| {
                let q = meta.query_selector(q_entry_first);
                vec![q * (advice(meta, sum, Rotation::cur()) - term(meta))]
            },
        );
        meta.create_gate("table: every further entry adds to the sum", |meta| {
            let q = meta.query_selector(q_entry_next);
            let sum_cur = advice(meta, sum, Rotation::cur());
            let sum_prev = advice(meta, sum, Rotation::prev());
            vec![q * (sum_cur - sum_prev - term(meta))]
        });

        let powers = [
            (x2, c, q_c_power_first, q_c_power_next, "c"),
            (x3, gamma, q_gamma_power_first, q_gamma_power_next, "γ"),
        ];
        for (column, base, q_first_power, q_next_power, name) in powers {
            meta.create_gate(format!("powers of {name}: the first is 1"), |meta| {
                let q = meta.query_selector(q_first_power);
                vec![q * (advice(meta, column, Rotation::cur()) - Expression::Constant(F::ONE))]
            });
            meta.create_gate(
                format!("powers of {name}: each is the previous times {name}"),
                |meta| {
                    let q = meta.query_selector(q_next_power);
                    let base = meta.query_challenge(base);
                    let cur = advice(meta, column, Rotation::cur());
                    let prev = advice(meta, column, Rotation::prev());
                    vec![q * (cur - prev * base)]
                },
            );
        }

        meta.lookup_any("node: the compressed claim is a branch", |meta| {
            let q = meta.query_selector(q_first);
            let end = meta.query_selector(q_branch_end);
            vec![
                (q.clone(), end.clone()),
                (
                    q * advice(meta, x3, Rotation::cur()),
                    end * advice(meta, sum, Rotation::cur()),
                ),
            ]
        });

        meta.lookup_any("table: cᵗ is a power of c", |meta| {
            let q = meta.query_selector(q_entry);
            let power = meta.query_selector(q_c_power);
            let row = meta.query_fixed(index, Rotation::cur());
            let value = advice(meta, x2, Rotation::cur());
            vec![
                (q.clone(), power.clone()),
                (q.clone() * row.clone(), power.clone() * row),
                (q * value.clone(), power * value),
            ]
        });

        meta.lookup_any("table: γʲ is a power of γ", |meta| {
            let q = meta.query_selector(q_entry);
            let power = meta.query_selector(q_gamma_power);
            let entry_cell = meta.query_fixed(cell, Rotation::cur());
            let exponent = meta.query_fixed(index, Rotation::cur());
            let value = advice(meta, x3, Rotation::cur());
            vec![
                (q.clone(), power.clone()),
                (q.clone() * entry_cell, power.clone() * exponent),
                (q * value.clone(), power * value),
            ]
        });

        Self {
            w,
            x2,
            acc,
            x3,
            sum,
            index,
            cell,
            coefficient,
            q_first,
            q_dot,
            q_last,
            q_entry,
            q_entry_first,
            q_entry_next,
            q_branch_end,
            q_c_power,
            q_c_power_first,
            q_c_power_next,
            q_gamma_power,
            q_gamma_power_first,
            q_gamma_power_next,
            c,
            gamma,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BranchChip {
    config: BranchConfig,
}

impl BranchChip {
    pub fn construct(config: BranchConfig) -> Self {
        Self { config }
    }

    /// The powers of `c` and `γ`, and the table of the `Tᵢ`. Once per circuit.
    pub fn load_table<F: PrimeField>(
        &self,
        layouter: &mut impl Layouter<F>,
        table: &BranchTable<F>,
    ) -> Result<(), Error> {
        let config = &self.config;
        let c = layouter.get_challenge(config.c);
        let gamma = layouter.get_challenge(config.gamma);

        layouter.assign_region(
            || "powers of c and γ",
            |mut region| {
                let powers = [
                    (
                        table.max_row,
                        c,
                        config.x2,
                        config.q_c_power,
                        config.q_c_power_first,
                        config.q_c_power_next,
                    ),
                    (
                        table.width,
                        gamma,
                        config.x3,
                        config.q_gamma_power,
                        config.q_gamma_power_first,
                        config.q_gamma_power_next,
                    ),
                ];
                let mut offset = 0;
                for (largest, base, column, q_power, q_first, q_next) in powers {
                    for exponent in 0..=largest {
                        let row = offset + exponent;
                        q_power.enable(&mut region, row)?;
                        if exponent == 0 {
                            q_first.enable(&mut region, row)?;
                        } else {
                            q_next.enable(&mut region, row)?;
                        }
                        region.assign_fixed(
                            || "exponent",
                            config.index,
                            row,
                            || Value::known(F::from(exponent as u64)),
                        )?;
                        let power = base.map(|base| base.pow_vartime([exponent as u64]));
                        region.assign_advice(|| "power", column, row, || power)?;
                    }
                    offset += largest + 1;
                }
                Ok(())
            },
        )?;

        layouter.assign_region(
            || "branch table",
            |mut region| {
                let mut row = 0;
                for entries in &table.branches {
                    let mut sum = Value::known(F::ZERO);
                    for (position, entry) in entries.iter().enumerate() {
                        config.q_entry.enable(&mut region, row)?;
                        if position == 0 {
                            config.q_entry_first.enable(&mut region, row)?;
                        } else {
                            config.q_entry_next.enable(&mut region, row)?;
                        }
                        if position + 1 == entries.len() {
                            config.q_branch_end.enable(&mut region, row)?;
                        }
                        let fixed = [
                            (config.index, entry.row as u64),
                            (config.cell, entry.cell as u64),
                        ];
                        for (column, value) in fixed {
                            region.assign_fixed(
                                || "entry",
                                column,
                                row,
                                || Value::known(F::from(value)),
                            )?;
                        }
                        region.assign_fixed(
                            || "coefficient",
                            config.coefficient,
                            row,
                            || Value::known(entry.coefficient),
                        )?;
                        let c_power = c.map(|c| c.pow_vartime([entry.row as u64]));
                        let gamma_power = gamma.map(|gamma| gamma.pow_vartime([entry.cell as u64]));
                        sum = sum + c_power * gamma_power * Value::known(entry.coefficient);
                        region.assign_advice(|| "cᵗ", config.x2, row, || c_power)?;
                        region.assign_advice(|| "γʲ", config.x3, row, || gamma_power)?;
                        region.assign_advice(|| "sum", config.sum, row, || sum)?;
                        row += 1;
                    }
                }
                Ok(())
            },
        )
    }

    /// One node: `w`, the claimed `cv`, the dot product and the Horner fold.
    /// Returns the cells of `w` (to be linked to the other chips' copies).
    pub fn assign_node<F: PrimeField>(
        &self,
        mut layouter: impl Layouter<F>,
        table: &BranchTable<F>,
        node: Value<&NodeBranch<F>>,
    ) -> Result<Vec<AssignedCell<F, F>>, Error> {
        let config = &self.config;
        let width = table.width;
        let c = layouter.get_challenge(config.c);
        let gamma = layouter.get_challenge(config.gamma);

        let cv: Value<Vec<F>> = node.zip(c).map(|(node, c)| match &node.claim {
            Claim::Branch(branch) => table.compress(*branch, c),
            Claim::Forged(vector) => vector.clone(),
        });
        let acc: Value<Vec<F>> = node.zip(cv.as_ref()).map(|(node, cv)| {
            let mut acc = vec![F::ZERO];
            for j in 0..width {
                acc.push(acc[j] + cv[j] * node.assignment[j]);
            }
            acc
        });
        let h: Value<Vec<F>> = cv.as_ref().zip(gamma).map(|(cv, gamma)| {
            let mut h = cv.clone();
            for j in (0..width).rev() {
                h[j] = cv[j] + gamma * h[j + 1];
            }
            h
        });

        layouter.assign_region(
            || "node branch",
            |mut region| {
                config.q_first.enable(&mut region, 0)?;
                config.q_last.enable(&mut region, width)?;
                let mut cells = Vec::with_capacity(width);
                for j in 0..=width {
                    if j < width {
                        config.q_dot.enable(&mut region, j)?;
                        let value = node.map(|node| node.assignment[j]);
                        cells.push(region.assign_advice(|| "w", config.w, j, || value)?);
                    }
                    region.assign_advice(|| "cv", config.x2, j, || cv.as_ref().map(|cv| cv[j]))?;
                    region.assign_advice(
                        || "acc",
                        config.acc,
                        j,
                        || acc.as_ref().map(|acc| acc[j]),
                    )?;
                    region.assign_advice(|| "h", config.x3, j, || h.as_ref().map(|h| h[j]))?;
                }
                Ok(cells)
            },
        )
    }
}
