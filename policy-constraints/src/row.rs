//! One linear constraint of a branch.

use crate::Fp;
use ff::Field;

/// What a row checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RowKind {
    /// Tag, name or arity of the parent goal or of a body goal.
    Signature,
    /// The number of body goals.
    ChildCount,
    /// A child slot that the branch does not use is empty.
    EmptyChildSlot,
    /// A variable-free subterm written in the clause, compared by digest.
    ExactSubterm,
    /// Tag, functor or arity of a constructor that contains variables.
    Shape,
    /// Two occurrences of the same clause variable have equal digests.
    SharedVariable,
    /// An arithmetic operand of a builtin is an integer.
    IntegerOperand,
    /// The two arguments of `=/2` are equal.
    Unification,
    /// An arithmetic step slot: its operands, its operation flags, or zero
    /// (see [`crate::arith`]); also `=:=` and the result of `is/2`.
    Arithmetic,
    /// Whether the node is an external fact.
    Fact,
    /// Whether the node is proved by a clause.
    Clause,
}

/// `Σ coefficient·w[column] + constant = 0` over a node assignment `w`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinearRow {
    pub kind: RowKind,
    /// Human-readable form, e.g. `payload(P) = applySupport`.
    pub description: String,
    pub terms: Vec<(usize, Fp)>,
    pub constant: Fp,
}

impl LinearRow {
    /// The residual of the row; zero when the row is satisfied.
    pub fn evaluate(&self, assignment: &[Fp]) -> Fp {
        self.terms
            .iter()
            .fold(self.constant, |sum, &(column, coefficient)| {
                sum + coefficient * assignment[column]
            })
    }

    pub fn is_satisfied_by(&self, assignment: &[Fp]) -> bool {
        self.evaluate(assignment) == Fp::ZERO
    }
}
