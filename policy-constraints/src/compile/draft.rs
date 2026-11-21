//! Branches under construction, with cells not yet mapped to columns.

use std::fmt::Display;

use ff::Field;
use halo2_proofs::pasta::Fp;

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

pub(super) struct BranchDraft {
    kind: BranchKind,
    children: usize,
    rows: Vec<RowDraft>,
}

impl BranchDraft {
    pub(super) fn new(kind: BranchKind, children: usize) -> Self {
        Self {
            kind,
            children,
            rows: Vec::new(),
        }
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
        Branch {
            id,
            kind: self.kind,
            rows,
        }
    }
}
