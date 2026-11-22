//! Addresses of subterms inside the goals of one proof-tree node.

use std::fmt::{self, Display, Formatter};

use crate::term::MAX_ARITY;

/// One goal of a node: the goal being proved (`P`) or a body goal (`C0`, `C1`, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Slot {
    Parent,
    Child(u8),
}

/// A subterm of a goal: the goal slot followed by child indices.
///
/// `P` is the parent goal itself, `P.1` its second argument, and `C0.1.0` the
/// first child of the second argument of the first body goal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    slot: Slot,
    path: Vec<u8>,
}

impl Position {
    pub fn root(slot: Slot) -> Self {
        Self {
            slot,
            path: Vec::new(),
        }
    }

    pub fn slot(&self) -> Slot {
        self.slot
    }

    pub fn path(&self) -> &[u8] {
        &self.path
    }

    /// The `index`-th child of this subterm.
    pub fn child(&self, index: usize) -> Self {
        assert!(index < MAX_ARITY, "child index {index} exceeds MAX_ARITY");
        let mut path = self.path.clone();
        path.push(index as u8);
        Self {
            slot: self.slot,
            path,
        }
    }

    /// The same path in another goal slot: `C0.1.0` in `P` is `P.1.0`.
    pub fn in_slot(&self, slot: Slot) -> Self {
        Self {
            slot,
            path: self.path.clone(),
        }
    }

    /// The strict prefixes of this position, from the goal root downwards.
    pub fn ancestors(&self) -> impl Iterator<Item = Position> + '_ {
        (0..self.path.len()).map(|length| Self {
            slot: self.slot,
            path: self.path[..length].to_vec(),
        })
    }
}

impl Display for Slot {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parent => formatter.write_str("P"),
            Self::Child(index) => write!(formatter, "C{index}"),
        }
    }
}

impl Display for Position {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.slot)?;
        for index in &self.path {
            write!(formatter, ".{index}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_print_and_list_their_ancestors() {
        let position = Position::root(Slot::Child(0)).child(1).child(0);
        assert_eq!(position.to_string(), "C0.1.0");
        let ancestors: Vec<String> = position.ancestors().map(|p| p.to_string()).collect();
        assert_eq!(ancestors, ["C0", "C0.1"]);
    }
}
