//! The shapes a node of a ground term may have: the lookup table of the
//! well-formedness chip.
//!
//! | tag      | arity | payload zero | absent | child digests present |
//! |----------|-------|--------------|--------|-----------------------|
//! | absent   | 0     | yes          | yes    | none                  |
//! | atom     | 0     | no           | no     | none                  |
//! | integer  | 0     | no           | no     | none                  |
//! | nil      | 0     | yes          | no     | none                  |
//! | cons     | 2     | yes          | no     | 0, 1                  |
//! | compound | 1..=4 | no           | no     | 0 .. arity            |
//!
//! The rows come from [`TermTag::arities`], the same rule
//! [`policy_constraints::OpenedNode::check_structure`] applies.

use ff::PrimeField;
use policy_constraints::{MAX_ARITY, TermTag};

/// Width of a table row: tag, arity, payload zero, absent, one flag per child.
pub const SHAPE_WIDTH: usize = 4 + MAX_ARITY;

/// One allowed (tag, arity) pair and what it implies for the other cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shape {
    pub tag: TermTag,
    pub arity: usize,
    /// The payload must be zero (absent, nil and cons nodes).
    pub payload_zero: bool,
    /// The node is absent: its digest must be zero too.
    pub absent: bool,
    /// `present[k]`: child digest `k` exists (`k < arity`) and is non-zero;
    /// the other child digests are zero.
    pub present: [bool; MAX_ARITY],
}

impl Shape {
    /// Every allowed shape: the nine rows of the table.
    pub fn all() -> Vec<Shape> {
        TermTag::ALL
            .into_iter()
            .flat_map(|tag| {
                let (smallest, largest) = tag.arities();
                (smallest..=largest).map(move |arity| Shape {
                    tag,
                    arity,
                    payload_zero: matches!(tag, TermTag::Absent | TermTag::Nil | TermTag::Cons),
                    absent: tag == TermTag::Absent,
                    present: std::array::from_fn(|child| child < arity),
                })
            })
            .collect()
    }

    /// The shape of a node with these tag and arity cells, if it is allowed.
    pub fn of<F: PrimeField>(tag: F, arity: F) -> Option<Shape> {
        Self::all()
            .into_iter()
            .find(|shape| F::from(shape.tag as u64) == tag && F::from(shape.arity as u64) == arity)
    }

    /// The table row: tag, arity, payload zero, absent, present flags.
    pub fn row<F: PrimeField>(&self) -> [F; SHAPE_WIDTH] {
        let flag = |value: bool| F::from(u64::from(value));
        let mut row = [F::ZERO; SHAPE_WIDTH];
        row[0] = F::from(self.tag as u64);
        row[1] = F::from(self.arity as u64);
        row[2] = flag(self.payload_zero);
        row[3] = flag(self.absent);
        for (child, present) in self.present.iter().enumerate() {
            row[4 + child] = flag(*present);
        }
        row
    }
}
