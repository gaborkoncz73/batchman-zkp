//! The shared layout of a node assignment: which cell holds what.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Formatter};

use ff::Field;
use halo2_proofs::pasta::Fp;

use crate::error::ModelError;
use crate::position::{Position, Slot};
use crate::term::{GroundTerm, MAX_ARITY};

/// The meaning of one cell of a node assignment.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Cell {
    /// Number of body goals of the node.
    ChildCount,
    /// Digest of the subterm at the position (zero if absent).
    Digest(Position),
    /// Tag of the subterm (see [`crate::TermTag`]).
    Tag(Position),
    /// Functor symbol or integer value of the subterm.
    Payload(Position),
    /// Number of children of the subterm.
    Arity(Position),
}

impl Display for Cell {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChildCount => formatter.write_str("children"),
            Self::Digest(at) => write!(formatter, "digest({at})"),
            Self::Tag(at) => write!(formatter, "tag({at})"),
            Self::Payload(at) => write!(formatter, "payload({at})"),
            Self::Arity(at) => write!(formatter, "arity({at})"),
        }
    }
}

/// A position whose node is opened: its tag, payload, arity and the digests of
/// all its `MAX_ARITY` child positions are cells, so the node's digest can be
/// recomputed from them. The values are column indices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenedNode {
    pub position: Position,
    pub digest: usize,
    pub tag: usize,
    pub payload: usize,
    pub arity: usize,
    pub children: [usize; MAX_ARITY],
}

/// Column layout shared by every branch.
///
/// The opened positions are closed under prefixes and always include the
/// parent goal and every child slot, so each referenced digest is linked by a
/// chain of opened nodes to the digest of a whole goal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    child_slots: usize,
    cells: Vec<Cell>,
    columns: BTreeMap<Cell, usize>,
    opened: Vec<OpenedNode>,
}

impl Layout {
    /// Builds the smallest layout that contains `referenced` cells.
    pub(crate) fn covering<'a>(
        child_slots: usize,
        referenced: impl IntoIterator<Item = &'a Cell>,
    ) -> Result<Self, ModelError> {
        let slot = |index: usize| {
            u8::try_from(index)
                .map(Slot::Child)
                .map_err(|_| ModelError::new(format!("{child_slots} body goals are too many")))
        };
        let mut opened = BTreeSet::from([Position::root(Slot::Parent)]);
        for index in 0..child_slots {
            opened.insert(Position::root(slot(index)?));
        }
        let mut digests = BTreeSet::new();
        for cell in referenced {
            match cell {
                Cell::ChildCount => {}
                Cell::Digest(at) => {
                    opened.extend(at.ancestors());
                    digests.insert(at.clone());
                }
                Cell::Tag(at) | Cell::Payload(at) | Cell::Arity(at) => {
                    opened.extend(at.ancestors());
                    opened.insert(at.clone());
                }
            }
        }
        for position in &opened {
            digests.insert(position.clone());
            digests.extend((0..MAX_ARITY).map(|index| position.child(index)));
        }

        let mut cells = vec![Cell::ChildCount];
        for position in &digests {
            cells.push(Cell::Digest(position.clone()));
            if opened.contains(position) {
                cells.push(Cell::Tag(position.clone()));
                cells.push(Cell::Payload(position.clone()));
                cells.push(Cell::Arity(position.clone()));
            }
        }
        let columns: BTreeMap<Cell, usize> = cells
            .iter()
            .enumerate()
            .map(|(column, cell)| (cell.clone(), column))
            .collect();
        let opened = opened
            .into_iter()
            .map(|position| OpenedNode {
                digest: columns[&Cell::Digest(position.clone())],
                tag: columns[&Cell::Tag(position.clone())],
                payload: columns[&Cell::Payload(position.clone())],
                arity: columns[&Cell::Arity(position.clone())],
                children: std::array::from_fn(|index| {
                    columns[&Cell::Digest(position.child(index))]
                }),
                position,
            })
            .collect();

        Ok(Self {
            child_slots,
            cells,
            columns,
            opened,
        })
    }

    /// Number of cells in a node assignment.
    pub fn width(&self) -> usize {
        self.cells.len()
    }

    /// Maximum number of body goals of any branch.
    pub fn child_slots(&self) -> usize {
        self.child_slots
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn column(&self, cell: &Cell) -> Option<usize> {
        self.columns.get(cell).copied()
    }

    pub fn opened_nodes(&self) -> &[OpenedNode] {
        &self.opened
    }

    /// Fills the cells of a node whose goal is `parent` and whose body goals
    /// are `children`. Positions that do not exist in the goals are zero.
    pub fn assign(
        &self,
        parent: &GroundTerm,
        children: &[GroundTerm],
    ) -> Result<Vec<Fp>, ModelError> {
        if children.len() > self.child_slots {
            return Err(ModelError::new(format!(
                "the node has {} body goals, the layout has {} slots",
                children.len(),
                self.child_slots
            )));
        }
        let subterm = |at: &Position| -> Option<&GroundTerm> {
            let goal = match at.slot() {
                Slot::Parent => parent,
                Slot::Child(index) => children.get(usize::from(index))?,
            };
            goal.at(at.path())
        };
        Ok(self
            .cells
            .iter()
            .map(|cell| match cell {
                Cell::ChildCount => Fp::from(children.len() as u64),
                Cell::Digest(at) => subterm(at).map_or(Fp::ZERO, GroundTerm::digest),
                Cell::Tag(at) => subterm(at).map_or(Fp::ZERO, |node| node.tag().field()),
                Cell::Payload(at) => subterm(at).map_or(Fp::ZERO, GroundTerm::payload),
                Cell::Arity(at) => {
                    subterm(at).map_or(Fp::ZERO, |node| Fp::from(node.arity() as u64))
                }
            })
            .collect())
    }
}
