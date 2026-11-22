//! Branches under construction, with cells not yet mapped to columns.

use std::fmt::Display;

use crate::Fp;
use ff::Field;

use crate::arith::{Step, StepCell, StepOp};
use crate::branch::{Branch, BranchKind};
use crate::error::ModelError;
use crate::layout::{Cell, Layout};
use crate::position::{Position, Slot};
use crate::row::{LinearRow, RowKind};
use crate::term::{TermTag, symbol};

struct RowDraft {
    kind: RowKind,
    description: String,
    terms: Vec<(Cell, Fp)>,
    constant: Fp,
}

/// A step with its operands as cells: an integer payload or an earlier `z`.
struct StepDraft {
    op: StepOp,
    x: Cell,
    y: Cell,
}

pub(super) struct BranchDraft {
    kind: BranchKind,
    children: usize,
    rows: Vec<RowDraft>,
    steps: Vec<StepDraft>,
}

impl BranchDraft {
    pub(super) fn new(kind: BranchKind, children: usize) -> Self {
        Self {
            kind,
            children,
            rows: Vec::new(),
            steps: Vec::new(),
        }
    }

    /// Number of arithmetic steps the branch uses.
    pub(super) fn steps(&self) -> usize {
        self.steps.len()
    }

    /// Appends the step `x op y`; returns the cell of its result `z`.
    pub(super) fn push_step(&mut self, op: StepOp, x: Cell, y: Cell) -> Cell {
        let step = u8::try_from(self.steps.len()).expect("fewer than 256 steps");
        self.steps.push(StepDraft { op, x, y });
        Cell::Step {
            step,
            part: StepCell::Z,
        }
    }

    /// Rows of the `slots` step slots: the branch's steps copy their operands
    /// and set their flag, a comparison's `z` and all unused cells are zero.
    pub(super) fn step_rows(&mut self, slots: usize) {
        for index in 0..slots {
            let step = u8::try_from(index).expect("fewer than 256 steps");
            let cell = |part| Cell::Step { step, part };
            let used = self
                .steps
                .get(index)
                .map(|draft| (draft.op, draft.x.clone(), draft.y.clone()));
            match used {
                Some((op, x, y)) => {
                    self.require_equal(RowKind::Arithmetic, cell(StepCell::X), x, "operand");
                    self.require_equal(RowKind::Arithmetic, cell(StepCell::Y), y, "operand");
                    if !op.has_result() {
                        self.require(RowKind::Arithmetic, cell(StepCell::Z), Fp::ZERO, 0);
                    }
                    for flag in StepOp::ALL {
                        let set = u64::from(flag == op);
                        self.require(
                            RowKind::Arithmetic,
                            cell(StepCell::Flag(flag)),
                            Fp::from(set),
                            set,
                        );
                    }
                }
                None => {
                    for part in StepCell::all() {
                        self.require(RowKind::Arithmetic, cell(part), Fp::ZERO, 0);
                    }
                }
            }
        }
    }

    /// Row `is_fact = 1` for an external fact branch, `is_fact = 0` otherwise.
    pub(super) fn fact_row(&mut self) {
        let fact = u64::from(matches!(self.kind, BranchKind::ExternalFact { .. }));
        self.require(RowKind::Fact, Cell::IsFact, Fp::from(fact), fact);
    }

    /// Row `is_clause = 1` for a clause branch, `is_clause = 0` otherwise.
    pub(super) fn clause_row(&mut self) {
        let clause = u64::from(matches!(self.kind, BranchKind::Clause { .. }));
        self.require(RowKind::Clause, Cell::IsClause, Fp::from(clause), clause);
    }

    pub(super) fn kind(&self) -> &BranchKind {
        &self.kind
    }

    pub(super) fn children(&self) -> usize {
        self.children
    }

    /// Row `cell − value = 0`.
    pub(super) fn require(&mut self, kind: RowKind, cell: Cell, value: Fp, expected: impl Display) {
        self.rows.push(RowDraft {
            kind,
            description: format!("{cell} = {expected}"),
            terms: vec![(cell, Fp::ONE)],
            constant: -value,
        });
    }

    /// Row `left − right = 0`.
    pub(super) fn require_equal(
        &mut self,
        kind: RowKind,
        left: Cell,
        right: Cell,
        reason: impl Display,
    ) {
        self.rows.push(RowDraft {
            kind,
            description: format!("{left} = {right}   ({reason})"),
            terms: vec![(left, Fp::ONE), (right, -Fp::ONE)],
            constant: Fp::ZERO,
        });
    }

    /// The goal in `slot` is `name/arity`.
    pub(super) fn goal_signature(&mut self, slot: Slot, name: &str, arity: usize) {
        let root = Position::root(slot);
        let tag = if arity == 0 {
            TermTag::Atom
        } else {
            TermTag::Compound
        };
        self.require(
            RowKind::Signature,
            Cell::Tag(root.clone()),
            tag.field(),
            tag.name(),
        );
        self.require(
            RowKind::Signature,
            Cell::Payload(root.clone()),
            symbol(name),
            name,
        );
        self.require(
            RowKind::Signature,
            Cell::Arity(root),
            Fp::from(arity as u64),
            arity,
        );
    }

    /// The node at `at` is a constructor with the given tag, functor and arity.
    pub(super) fn node_shape(
        &mut self,
        kind: RowKind,
        at: &Position,
        tag: TermTag,
        functor: Option<&str>,
        arity: usize,
    ) {
        let (payload, payload_text) = functor.map_or((Fp::ZERO, "0"), |name| (symbol(name), name));
        self.require(kind, Cell::Tag(at.clone()), tag.field(), tag.name());
        self.require(kind, Cell::Payload(at.clone()), payload, payload_text);
        self.require(kind, Cell::Arity(at.clone()), Fp::from(arity as u64), arity);
    }

    pub(super) fn child_count(&mut self) {
        let children = self.children;
        self.require(
            RowKind::ChildCount,
            Cell::ChildCount,
            Fp::from(children as u64),
            children,
        );
    }

    /// Every child slot from `children` up to `child_slots` is empty.
    pub(super) fn empty_slots(&mut self, child_slots: usize) -> Result<(), ModelError> {
        for index in self.children..child_slots {
            let slot = u8::try_from(index)
                .map_err(|_| ModelError::new(format!("child slot {index} does not fit u8")))?;
            let root = Position::root(Slot::Child(slot));
            self.require(RowKind::EmptyChildSlot, Cell::Tag(root), Fp::ZERO, "absent");
        }
        Ok(())
    }

    pub(super) fn cells(&self) -> impl Iterator<Item = &Cell> {
        self.rows
            .iter()
            .flat_map(|row| row.terms.iter().map(|(cell, _)| cell))
    }

    pub(super) fn resolve(self, id: usize, layout: &Layout) -> Branch {
        let rows = self
            .rows
            .into_iter()
            .map(|row| LinearRow {
                kind: row.kind,
                description: row.description,
                terms: row
                    .terms
                    .into_iter()
                    .map(|(cell, coefficient)| {
                        let column = layout.column(&cell).unwrap_or_else(|| {
                            panic!("the layout covers every referenced cell, {cell} is missing")
                        });
                        (column, coefficient)
                    })
                    .collect(),
                constant: row.constant,
            })
            .collect();
        let column = |cell: &Cell| {
            layout
                .column(cell)
                .unwrap_or_else(|| panic!("the layout covers every operand, {cell} is missing"))
        };
        let steps = self
            .steps
            .iter()
            .map(|draft| Step {
                op: draft.op,
                x: column(&draft.x),
                y: column(&draft.y),
            })
            .collect();
        Branch {
            id,
            kind: self.kind,
            rows,
            steps,
        }
    }
}
