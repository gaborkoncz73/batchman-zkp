//! Arithmetic steps: how builtin calls are checked without revealing which
//! builtin a node is.
//!
//! The expression of a builtin call compiles into a short program of steps,
//! each `z = x op y` (`+ − * div`) or a comparison of `x` and `y` (`< =< =\=`;
//! `>` and `>=` swap the operands, `=:=` is a plain row). Every node
//! assignment has the same number of step slots, each with the cells `x`, `y`,
//! `z` and one flag per operation. A branch's rows bind them: a builtin branch
//! copies its operands into `x` and `y` and sets the flag of each step's
//! operation; every other branch sets all step cells to zero. One uniform
//! check per node — [`check_step`] here, the arithmetic chip in the circuit —
//! then enforces whatever the flags say.
//!
//! Integers are 64-bit, as in [`crate::ArithmeticOp::apply`]: every `x`, `y`
//! and `z` must decode to an `i64`, so a result that overflows has no
//! assignment at all.

use std::fmt::{self, Display, Formatter};

use ff::{Field, PrimeField};

use crate::Fp;
use crate::branch::{ArithmeticOp, Comparison};

/// The operation of a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StepOp {
    Add,
    Subtract,
    Multiply,
    /// Integer division rounding towards negative infinity (`div`).
    FloorDivide,
    Less,
    LessOrEqual,
    NotEqual,
}

impl StepOp {
    pub const ALL: [Self; 7] = [
        Self::Add,
        Self::Subtract,
        Self::Multiply,
        Self::FloorDivide,
        Self::Less,
        Self::LessOrEqual,
        Self::NotEqual,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Subtract => "sub",
            Self::Multiply => "mul",
            Self::FloorDivide => "div",
            Self::Less => "lt",
            Self::LessOrEqual => "le",
            Self::NotEqual => "ne",
        }
    }

    /// The arithmetic operation of a step with a result.
    pub fn arithmetic(self) -> Option<ArithmeticOp> {
        match self {
            Self::Add => Some(ArithmeticOp::Add),
            Self::Subtract => Some(ArithmeticOp::Subtract),
            Self::Multiply => Some(ArithmeticOp::Multiply),
            Self::FloorDivide => Some(ArithmeticOp::FloorDivide),
            Self::Less | Self::LessOrEqual | Self::NotEqual => None,
        }
    }

    /// Whether the step has a result `z` (the comparisons do not).
    pub fn has_result(self) -> bool {
        matches!(
            self,
            Self::Add | Self::Subtract | Self::Multiply | Self::FloorDivide
        )
    }
}

impl From<ArithmeticOp> for StepOp {
    fn from(operator: ArithmeticOp) -> Self {
        match operator {
            ArithmeticOp::Add => Self::Add,
            ArithmeticOp::Subtract => Self::Subtract,
            ArithmeticOp::Multiply => Self::Multiply,
            ArithmeticOp::FloorDivide => Self::FloorDivide,
        }
    }
}

/// A cell of a step slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StepCell {
    X,
    Y,
    Z,
    /// 1 if the step does this operation, 0 otherwise.
    Flag(StepOp),
}

impl StepCell {
    /// Every cell of a step slot, in layout order.
    pub fn all() -> impl Iterator<Item = StepCell> {
        [Self::X, Self::Y, Self::Z]
            .into_iter()
            .chain(StepOp::ALL.map(Self::Flag))
    }
}

impl Display for StepCell {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::X => formatter.write_str("x"),
            Self::Y => formatter.write_str("y"),
            Self::Z => formatter.write_str("z"),
            Self::Flag(op) => formatter.write_str(op.name()),
        }
    }
}

/// One step of a builtin branch: the operation and the columns its operands
/// are copied from (an integer payload, or the `z` of an earlier step).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    pub op: StepOp,
    pub x: usize,
    pub y: usize,
}

/// How a comparison is done by the steps: `>` and `>=` swap the operands of
/// `<` and `=<`; `=:=` is a row, not a step.
pub fn comparison_step(relation: Comparison) -> Option<(StepOp, bool)> {
    match relation {
        Comparison::Less => Some((StepOp::Less, false)),
        Comparison::Greater => Some((StepOp::Less, true)),
        Comparison::LessOrEqual => Some((StepOp::LessOrEqual, false)),
        Comparison::GreaterOrEqual => Some((StepOp::LessOrEqual, true)),
        Comparison::NotEqual => Some((StepOp::NotEqual, false)),
        Comparison::Equal => None,
    }
}

/// The `i64` a field element encodes ([`crate::term::integer`]), if any.
pub fn decode(value: Fp) -> Option<i64> {
    let small = |value: Fp| -> Option<u64> {
        let repr = value.to_repr();
        let bytes = repr.as_ref();
        if bytes[8..].iter().any(|byte| *byte != 0) {
            return None;
        }
        Some(u64::from_le_bytes(
            bytes[..8].try_into().expect("eight bytes"),
        ))
    };
    if let Some(magnitude) = small(value) {
        return i64::try_from(magnitude).ok();
    }
    let magnitude = small(-value)?;
    i64::try_from(-i128::from(magnitude)).ok()
}

/// Whether `op` holds for `x`, `y` and `z` (integers, not field elements).
pub fn holds(op: StepOp, x: i64, y: i64, z: i64) -> bool {
    let (x, y, z) = (i128::from(x), i128::from(y), i128::from(z));
    match op {
        StepOp::Add => z == x + y,
        StepOp::Subtract => z == x - y,
        StepOp::Multiply => z == x * y,
        // Floor division: the remainder x − z·y has the sign of y.
        StepOp::FloorDivide => {
            let remainder = x - z * y;
            if y > 0 {
                (0..y).contains(&remainder)
            } else {
                y < 0 && (y + 1..=0).contains(&remainder)
            }
        }
        StepOp::Less => x < y,
        StepOp::LessOrEqual => x <= y,
        StepOp::NotEqual => x != y,
    }
}

/// The reference check of one step slot, from its cells: `x`, `y` and `z`
/// are 64-bit integers, and every operation whose flag is set holds.
pub fn check_step(cells: impl Fn(StepCell) -> Fp) -> Result<(), String> {
    let decoded = |part: StepCell| {
        decode(cells(part)).ok_or_else(|| format!("{part} is not a 64-bit integer"))
    };
    let (x, y, z) = (
        decoded(StepCell::X)?,
        decoded(StepCell::Y)?,
        decoded(StepCell::Z)?,
    );
    for op in StepOp::ALL {
        if cells(StepCell::Flag(op)) != Fp::ZERO && !holds(op, x, y, z) {
            return Err(if op.has_result() {
                format!("{} does not hold for x = {x}, y = {y}, z = {z}", op.name())
            } else {
                format!("{} does not hold for x = {x}, y = {y}", op.name())
            });
        }
    }
    Ok(())
}
