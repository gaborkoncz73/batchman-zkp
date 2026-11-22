//! The shared layout of a node assignment: which cell holds what.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Formatter};

use crate::Fp;
use ff::Field;

use crate::arith::{self, StepCell};
use crate::error::ModelError;
use crate::position::{Position, Slot};
use crate::term::{GroundTerm, MAX_ARITY, TermTag, hash_node};

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
    /// A cell of arithmetic step slot `step` (see [`crate::arith`]).
    Step { step: u8, part: StepCell },
    /// 1 if the node is an external fact, 0 otherwise.
    IsFact,
    /// The secret salt of the node's fact commitment (zero for other nodes).
    FactSalt,
    /// 1 if the node is proved by a clause of the policy, 0 otherwise.
    IsClause,
}

impl Cell {
    /// The position the cell describes; `None` for [`Cell::ChildCount`].
    pub fn position(&self) -> Option<&Position> {
        match self {
            Self::ChildCount
            | Self::Step { .. }
            | Self::IsFact
            | Self::FactSalt
            | Self::IsClause => None,
            Self::Digest(at) | Self::Tag(at) | Self::Payload(at) | Self::Arity(at) => Some(at),
        }
    }

    /// The same cell with its position moved to `slot`.
    fn in_slot(&self, slot: Slot) -> Self {
        match self {
            Self::ChildCount
            | Self::Step { .. }
            | Self::IsFact
            | Self::FactSalt
            | Self::IsClause => self.clone(),
            Self::Digest(at) => Self::Digest(at.in_slot(slot)),
            Self::Tag(at) => Self::Tag(at.in_slot(slot)),
            Self::Payload(at) => Self::Payload(at.in_slot(slot)),
            Self::Arity(at) => Self::Arity(at.in_slot(slot)),
        }
    }
}

impl Display for Cell {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChildCount => formatter.write_str("children"),
            Self::Digest(at) => write!(formatter, "digest({at})"),
            Self::Tag(at) => write!(formatter, "tag({at})"),
            Self::Payload(at) => write!(formatter, "payload({at})"),
            Self::Arity(at) => write!(formatter, "arity({at})"),
            Self::Step { step, part } => write!(formatter, "{part}[{step}]"),
            Self::IsFact => formatter.write_str("is_fact"),
            Self::FactSalt => formatter.write_str("fact_salt"),
            Self::IsClause => formatter.write_str("is_clause"),
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

impl OpenedNode {
    /// Checks that the cells of this node, read from `assignment`, have the
    /// shape of one node of a ground term (see [`crate::term`]):
    ///
    /// * the tag is a [`TermTag`];
    /// * an absent node has all its cells zero;
    /// * a present node has an arity its tag allows, payload zero if it is nil
    ///   or cons, and exactly `arity` non-zero child digests followed by zeros.
    ///
    /// The digest is not checked here; see [`OpenedNode::check_digest`].
    pub fn check_structure(&self, assignment: &[Fp]) -> Result<(), String> {
        let digest = assignment[self.digest];
        let tag = assignment[self.tag];
        let payload = assignment[self.payload];
        let arity = assignment[self.arity];
        let children = self.children.map(|column| assignment[column]);

        let tag = TermTag::from_field(tag).ok_or("the tag is not a term tag")?;
        if tag == TermTag::Absent {
            let cells = [digest, payload, arity].into_iter().chain(children);
            if cells.into_iter().any(|value| value != Fp::ZERO) {
                return Err("the node is absent, but not all of its cells are zero".to_owned());
            }
            return Ok(());
        }

        let (smallest, largest) = tag.arities();
        let count = (smallest..=largest)
            .find(|&count| Fp::from(count as u64) == arity)
            .ok_or_else(|| {
                format!(
                    "a {} node must have arity {smallest}..={largest}",
                    tag.name()
                )
            })?;
        if matches!(tag, TermTag::Nil | TermTag::Cons) && payload != Fp::ZERO {
            return Err(format!("a {} node must have payload zero", tag.name()));
        }
        for (index, child) in children.iter().enumerate() {
            if index < count && *child == Fp::ZERO {
                return Err(format!(
                    "child {index} is missing, but the arity is {count}"
                ));
            }
            if index >= count && *child != Fp::ZERO {
                return Err(format!(
                    "child {index} is present, but the arity is {count}"
                ));
            }
        }
        Ok(())
    }

    /// Checks that a present node's digest is the hash of its other cells. An
    /// absent node is skipped: [`OpenedNode::check_structure`] makes it zero.
    pub fn check_digest(&self, assignment: &[Fp]) -> Result<(), String> {
        let tag = assignment[self.tag];
        if tag == TermTag::Absent.field() {
            return Ok(());
        }
        let children = self.children.map(|column| assignment[column]);
        let hash = hash_node(
            tag,
            assignment[self.payload],
            assignment[self.arity],
            &children,
        );
        if assignment[self.digest] != hash {
            return Err("the digest is not the hash of the other cells".to_owned());
        }
        Ok(())
    }
}

/// A cell of a child slot and the cell of the child node it must equal: the
/// body goal in slot `k` of a node is the parent goal of its `k`-th child, so
/// `payload(C0.1)` of the node equals `payload(P.1)` of child 0. The values
/// are column indices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotLink {
    /// The cell in the node, under a child slot.
    pub cell: Cell,
    /// Its column in the node's assignment.
    pub in_parent: usize,
    /// The column of the same cell under `P` in the child's assignment.
    pub in_child: usize,
}

/// Column layout shared by every branch.
///
/// The opened positions are closed under prefixes and always include the
/// parent goal and every child slot, so each referenced digest is linked by a
/// chain of opened nodes to the digest of a whole goal.
///
/// The layout is symmetric: every goal slot (`P`, `C0`, `C1`, …) opens the
/// same paths, so a child slot has exactly the cells of a parent goal, in the
/// same order ([`Layout::goal_columns`], [`Layout::slot_links`]). The cells of
/// a present slot equal the parent-side cells of the node that proves that
/// body goal, which hashes them; a node hashes only its parent-side positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    child_slots: usize,
    cells: Vec<Cell>,
    columns: BTreeMap<Cell, usize>,
    opened: Vec<OpenedNode>,
    goal: Vec<usize>,
    links: Vec<Vec<SlotLink>>,
    steps: usize,
    facts: bool,
}

impl Layout {
    /// Builds the smallest symmetric layout that contains `referenced` cells,
    /// followed by `steps` arithmetic step slots, the fact cells if `facts`,
    /// and `is_clause`.
    pub(crate) fn covering<'a>(
        child_slots: usize,
        steps: usize,
        facts: bool,
        //A referenced cellák, amiket a layoutnak tartalmaznia kell. Ezek lehetnek ChildCount, Digest, Tag, Payload vagy Arity cellák.
        //Ezek oonnan jönnek, hogy a branch-ekben milyen cellákra van szükség a constraint-ekhez.
        referenced: impl IntoIterator<Item = &'a Cell>,
    ) -> Result<Self, ModelError> {
        // AZ indexedik child slot-ot adja vissza, ha az index kisebb, mint child_slots, különben hibát dob.
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
                Cell::ChildCount
                | Cell::Step { .. }
                | Cell::IsFact
                | Cell::FactSalt
                | Cell::IsClause => {}
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
        // Symmetry: every path opened in some goal slot is opened in all of
        // them, so a slot's cells and a parent goal's cells are the same list.
        // The set stays prefix-closed, and the digests of the opened positions'
        // children come below.
        let slots: Vec<Slot> = std::iter::once(Ok(Slot::Parent))
            .chain((0..child_slots).map(slot))
            .collect::<Result<_, _>>()?;
        let mirrored: Vec<Position> = opened
            .iter()
            .flat_map(|position| slots.iter().map(|&slot| position.in_slot(slot)))
            .collect();
        opened.extend(mirrored);
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
        for step in 0..steps {
            let step = u8::try_from(step)
                .map_err(|_| ModelError::new(format!("{steps} arithmetic steps are too many")))?;
            cells.extend(StepCell::all().map(|part| Cell::Step { step, part }));
        }
        if facts {
            cells.extend([Cell::IsFact, Cell::FactSalt]);
        }
        cells.push(Cell::IsClause);
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
        let in_slot = |slot: Slot| {
            cells
                .iter()
                .filter(move |cell| cell.position().is_some_and(|at| at.slot() == slot))
        };
        let goal: Vec<usize> = in_slot(Slot::Parent).map(|cell| columns[cell]).collect();
        let links: Vec<Vec<SlotLink>> = slots[1..]
            .iter()
            .map(|&slot| {
                in_slot(slot)
                    .map(|cell| SlotLink {
                        cell: cell.clone(),
                        in_parent: columns[cell],
                        in_child: columns[&cell.in_slot(Slot::Parent)],
                    })
                    .collect()
            })
            .collect();
        debug_assert!(links.iter().all(|links| {
            links
                .iter()
                .map(|link| link.in_child)
                .eq(goal.iter().copied())
        }));

        Ok(Self {
            child_slots,
            cells,
            columns,
            opened,
            goal,
            links,
            steps,
            facts,
        })
    }

    /// Number of arithmetic step slots.
    pub fn steps(&self) -> usize {
        self.steps
    }

    /// The columns of the `is_fact` and `fact_salt` cells, if the policy has
    /// external facts.
    pub fn fact_cells(&self) -> Option<(usize, usize)> {
        self.facts
            .then(|| (self.columns[&Cell::IsFact], self.columns[&Cell::FactSalt]))
    }

    /// The column of `is_clause`.
    pub fn is_clause(&self) -> usize {
        self.columns[&Cell::IsClause]
    }

    /// The columns of the parent goal's cells (every cell under `P`), in
    /// layout order. A node exports this list; the cells of child slot `k` are
    /// the same list under `C<k>` ([`Layout::slot_links`]).
    pub fn goal_columns(&self) -> &[usize] {
        &self.goal
    }

    /// The parent goal's cells of `assignment`, in the order of
    /// [`Layout::goal_columns`].
    pub fn goal_of(&self, assignment: &[Fp]) -> Vec<Fp> {
        self.goal.iter().map(|&column| assignment[column]).collect()
    }

    /// The goal digest among `goal`'s cells (a list of
    /// [`Layout::goal_columns`] values).
    pub fn goal_digest(&self, goal: &[Fp]) -> Fp {
        let digest = self.columns[&Cell::Digest(Position::root(Slot::Parent))];
        let index = self
            .goal
            .iter()
            .position(|&column| column == digest)
            .expect("the goal's digest is one of its cells");
        goal[index]
    }

    /// Checks a list of goal cells (in the order of
    /// [`Layout::goal_columns`]) as a node checks its parent goal: every
    /// opened position is well-formed and hashes to its digest. The goal
    /// digest then fixes every cell.
    pub fn check_goal(&self, goal: &[Fp]) -> Result<(), ModelError> {
        if goal.len() != self.goal.len() {
            return Err(ModelError::new(format!(
                "the goal has {} cells, the layout has {}",
                goal.len(),
                self.goal.len()
            )));
        }
        let mut assignment = vec![Fp::ZERO; self.width()];
        for (&column, &value) in self.goal.iter().zip(goal) {
            assignment[column] = value;
        }
        for node in self
            .opened
            .iter()
            .filter(|node| node.position.slot() == Slot::Parent)
        {
            node.check_structure(&assignment)
                .and_then(|()| node.check_digest(&assignment))
                .map_err(|reason| {
                    ModelError::new(format!("subterm {}: {reason}", node.position))
                })?;
        }
        Ok(())
    }

    /// The column of a cell of step slot `step`.
    pub fn step_column(&self, step: usize, part: StepCell) -> usize {
        let step = u8::try_from(step).expect("fewer than 256 steps");
        self.columns[&Cell::Step { step, part }]
    }

    /// The reference check of the arithmetic step slots of `assignment`
    /// ([`arith::check_step`]).
    pub fn check_arithmetic(&self, assignment: &[Fp]) -> Result<(), ModelError> {
        for step in 0..self.steps {
            arith::check_step(|part| assignment[self.step_column(step, part)])
                .map_err(|reason| ModelError::new(format!("step {step}: {reason}")))?;
        }
        Ok(())
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

    /// The links of child slot `slot`: every cell under `C<slot>` with the
    /// column of the same cell under `P` (see [`SlotLink`]), in the order of
    /// [`Layout::goal_columns`].
    pub fn slot_links(&self, slot: usize) -> &[SlotLink] {
        &self.links[slot]
    }

    /// Checks the structure of every opened node of `assignment`
    /// ([`OpenedNode::check_structure`]) and the digests of the parent-side
    /// ones ([`OpenedNode::check_digest`]). This is the branch-independent
    /// part of a node check.
    ///
    /// Child-side digests are not hashed here: the cells of a present child
    /// slot are linked to the child node's parent-side cells, which the child
    /// hashes ([`Layout::slot_links`]), and an absent slot is all zero.
    pub fn check_well_formed(&self, assignment: &[Fp]) -> Result<(), ModelError> {
        if assignment.len() != self.width() {
            return Err(ModelError::new(format!(
                "the assignment has {} cells, the layout has {}",
                assignment.len(),
                self.width()
            )));
        }
        for node in &self.opened {
            let checked = node.check_structure(assignment).and_then(|()| {
                if node.position.slot() == Slot::Parent {
                    node.check_digest(assignment)
                } else {
                    Ok(())
                }
            });
            checked.map_err(|reason| {
                ModelError::new(format!("subterm {}: {reason}", node.position))
            })?;
        }
        Ok(())
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
        //Megadunk egy pozíciót és ha azon a helyen van GroundTerm, akkor visszaadja, ha nincs, akkor None-t ad vissza.
        let subterm = |at: &Position| -> Option<&GroundTerm> {
            let goal = match at.slot() {
                Slot::Parent => parent,
                Slot::Child(index) => children.get(usize::from(index))?,
            };
            goal.at(at.path())
        };
        //Visszaadja a cellák értékeit, ha van GroundTerm, ha nincs, akkor Fp::ZERO-t ad vissza.
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
                // Filled per branch: ConstraintSystem::complete.
                Cell::Step { .. } | Cell::IsFact | Cell::FactSalt | Cell::IsClause => Fp::ZERO,
            })
            .collect())
    }
}
