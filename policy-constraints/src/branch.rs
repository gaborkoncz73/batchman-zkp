//! Branches: the alternative ways to prove one node.

use std::fmt::{self, Display, Formatter};

use ff::Field;
use halo2_proofs::pasta::Fp;
use policy_ir::PredicateIndicator;

use crate::position::Position;
use crate::row::LinearRow;

/// One alternative way to prove a node, with its linear rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    pub id: usize,
    pub kind: BranchKind,
    pub rows: Vec<LinearRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchKind {
    /// One alternative of a policy clause. A body with `;` has one
    /// alternative per way through the disjunctions, numbered left to right.
    Clause {
        predicate: PredicateIndicator,
        clause: usize,
        alternative: usize,
        children: usize,
    },
    /// A builtin call. Its rows fix the call's form; the arithmetic itself is
    /// checked separately.
    Builtin(BuiltinCheck),
    /// A goal of a predicate that the policy calls but does not define. Its
    /// rows fix the signature; registry membership is checked separately.
    ExternalFact { predicate: PredicateIndicator },
}

/// The form of a builtin call, with positions relative to the builtin goal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltinCheck {
    /// `A = B`: the two arguments have the same digest.
    Unify,
    /// `R is E`: the integer `R` (at `P.0`) is the value of `E` (at `P.1`).
    Evaluate {
        result: Position,
        expression: Expression,
    },
    /// `A < B` and the other arithmetic comparisons.
    Compare {
        relation: Comparison,
        left: Expression,
        right: Expression,
    },
}

/// An arithmetic expression tree whose leaves are integers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Integer {
        at: Position,
    },
    Apply {
        at: Position,
        operator: ArithmeticOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArithmeticOp {
    Add,
    Subtract,
    Multiply,
    /// `div`: integer division rounding towards negative infinity.
    FloorDivide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Comparison {
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Equal,
    NotEqual,
}

/// The rows of a branch combined with weights `s`: `Σ s_r·row_r(w)`.
///
/// This is the Batchman compressed topology vector `sᵀMᵢ`, split into the
/// coefficients of the cells and the coefficient of the constant 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressedBranch {
    pub coefficients: Vec<Fp>,
    pub constant: Fp,
}

impl Branch {
    pub fn is_satisfied_by(&self, assignment: &[Fp]) -> bool {
        self.rows.iter().all(|row| row.is_satisfied_by(assignment))
    }

    /// Combines the rows with one weight per row.
    pub fn compress(&self, width: usize, weights: &[Fp]) -> CompressedBranch {
        assert_eq!(weights.len(), self.rows.len(), "one weight per row");
        let mut compressed = CompressedBranch {
            coefficients: vec![Fp::ZERO; width],
            constant: Fp::ZERO,
        };
        for (row, weight) in self.rows.iter().zip(weights) {
            for &(column, coefficient) in &row.terms {
                compressed.coefficients[column] += *weight * coefficient;
            }
            compressed.constant += *weight * row.constant;
        }
        compressed
    }
}

impl CompressedBranch {
    /// `⟨coefficients, w⟩ + constant`; zero for an assignment that satisfies
    /// every row, and non-zero with overwhelming probability otherwise.
    pub fn evaluate(&self, assignment: &[Fp]) -> Fp {
        self.coefficients
            .iter()
            .zip(assignment)
            .fold(self.constant, |sum, (coefficient, value)| {
                sum + *coefficient * value
            })
    }
}

impl BuiltinCheck {
    pub fn functor(&self) -> &'static str {
        match self {
            Self::Unify => "=",
            Self::Evaluate { .. } => "is",
            Self::Compare { relation, .. } => relation.functor(),
        }
    }
}

impl ArithmeticOp {
    pub const ALL: [Self; 4] = [Self::Add, Self::Subtract, Self::Multiply, Self::FloorDivide];

    pub fn functor(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::FloorDivide => "div",
        }
    }
}

impl Comparison {
    pub const ALL: [Self; 6] = [
        Self::Less,
        Self::LessOrEqual,
        Self::Greater,
        Self::GreaterOrEqual,
        Self::Equal,
        Self::NotEqual,
    ];

    pub fn functor(self) -> &'static str {
        match self {
            Self::Less => "<",
            Self::LessOrEqual => "=<",
            Self::Greater => ">",
            Self::GreaterOrEqual => ">=",
            Self::Equal => "=:=",
            Self::NotEqual => "=\\=",
        }
    }
}

impl Display for Expression {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer { .. } => formatter.write_str("_"),
            Self::Apply {
                operator,
                left,
                right,
                ..
            } => write!(formatter, "({left} {} {right})", operator.functor()),
        }
    }
}

impl Display for BuiltinCheck {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unify => formatter.write_str("_ = _"),
            Self::Evaluate { expression, .. } => write!(formatter, "_ is {expression}"),
            Self::Compare {
                relation,
                left,
                right,
            } => write!(formatter, "{left} {} {right}", relation.functor()),
        }
    }
}

impl Display for BranchKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Clause {
                predicate,
                clause,
                alternative,
                ..
            } => write!(
                formatter,
                "clause {}/{} #{clause} alternative {alternative}",
                predicate.name, predicate.arity
            ),
            Self::Builtin(check) => write!(formatter, "builtin {check}"),
            Self::ExternalFact { predicate } => {
                write!(
                    formatter,
                    "external fact {}/{}",
                    predicate.name, predicate.arity
                )
            }
        }
    }
}
