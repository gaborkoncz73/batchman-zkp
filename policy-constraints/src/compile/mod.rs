//! Policy → layout and branches.

mod body;
mod builtin;
mod clause;
mod draft;
mod show;

use std::collections::BTreeSet;

use policy_ir::{PolicyFile, PredicateIndicator};

use self::body::Callee;
use self::draft::BranchDraft;
use crate::branch::{Branch, BranchKind, BuiltinCheck};
use crate::error::ModelError;
use crate::layout::Layout;
use crate::position::Slot;
use crate::term::MAX_ARITY;

/// The constraint model of a policy: the shared layout and every branch.
///
/// Branches are ordered as clause alternatives (in policy order), builtin
/// call forms (in order of first use), then external fact predicates (sorted).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintSystem {
    layout: Layout,
    branches: Vec<Branch>,
}

impl ConstraintSystem {
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    pub fn branches(&self) -> &[Branch] {
        &self.branches
    }

    /// The branches of the clauses of `name/arity`.
    pub fn clause_branches<'a>(
        &'a self,
        name: &'a str,
        arity: usize,
    ) -> impl Iterator<Item = &'a Branch> + 'a {
        self.branches.iter().filter(move |branch| {
            matches!(&branch.kind, BranchKind::Clause { predicate, .. }
                if predicate.name == name && predicate.arity == arity)
        })
    }

    pub fn builtin_checks(&self) -> impl Iterator<Item = &BuiltinCheck> {
        self.branches
            .iter()
            .filter_map(|branch| match &branch.kind {
                BranchKind::Builtin(check) => Some(check),
                _ => None,
            })
    }

    pub fn external_predicates(&self) -> impl Iterator<Item = &PredicateIndicator> {
        self.branches
            .iter()
            .filter_map(|branch| match &branch.kind {
                BranchKind::ExternalFact { predicate } => Some(predicate),
                _ => None,
            })
    }
}

/// Builds the constraint model of a policy.
/// eredménye a ConstraintSystem, ami tartalmazza a layout-ot és a branch-eket
pub fn compile(policy: &PolicyFile) -> Result<ConstraintSystem, ModelError> {
    policy.validate().map_err(ModelError::new)?;

    let defined: BTreeSet<PredicateIndicator> = policy
        .predicates
        .iter()
        .map(|predicate| predicate.indicator.clone())
        .collect();
    for predicate in &defined {
        if body::is_reserved(&predicate.name)
            || builtin::is_builtin(&predicate.name, predicate.arity)
        {
            return Err(ModelError::new(format!(
                "the policy redefines the system predicate {}/{}",
                predicate.name, predicate.arity
            )));
        }
    }

    let mut drafts = Vec::new();
    let mut builtin_checks: Vec<BuiltinCheck> = Vec::new();
    let mut externals = BTreeSet::new();
    for predicate in &policy.predicates {
        let indicator = &predicate.indicator;
        for (ordinal, clause) in predicate.clauses.iter().enumerate() {
            let context = format!("{}/{} clause {ordinal}", indicator.name, indicator.arity);
            for (alternative, goals) in body::alternatives(&clause.body).into_iter().enumerate() {
                for term in &goals {
                    let goal = body::goal(term).map_err(|error| error.context(&context))?;
                    match body::classify(&goal, &defined)
                        .map_err(|error| error.context(&context))?
                    {
                        Callee::Policy => {}
                        Callee::Builtin => {
                            let check = builtin::builtin_check(&goal)
                                .map_err(|error| error.context(&context))?;
                            if !builtin_checks.contains(&check) {
                                builtin_checks.push(check);
                            }
                        }
                        Callee::External => {
                            externals.insert(goal.indicator());
                        }
                    }
                }
                let draft = clause::clause_branch(indicator, ordinal, clause, alternative, &goals)
                    .map_err(|error| error.context(&context))?;
                drafts.push(draft);
            }
        }
    }
    drafts.extend(builtin_checks.into_iter().map(builtin::builtin_branch));
    for predicate in externals {
        drafts.push(external_fact_branch(predicate)?);
    }

    let child_slots = drafts.iter().map(BranchDraft::children).max().unwrap_or(0);
    for draft in &mut drafts {
        draft.empty_slots(child_slots)?;
    }
    let layout = Layout::covering(child_slots, drafts.iter().flat_map(BranchDraft::cells))?;
    let branches = drafts
        .into_iter()
        .enumerate()
        .map(|(id, draft)| draft.resolve(id, &layout))
        .collect();
    Ok(ConstraintSystem { layout, branches })
}

fn external_fact_branch(predicate: PredicateIndicator) -> Result<BranchDraft, ModelError> {
    if predicate.arity > MAX_ARITY {
        return Err(ModelError::new(format!(
            "external predicate {}/{} exceeds the maximum arity {MAX_ARITY}",
            predicate.name, predicate.arity
        )));
    }
    let (name, arity) = (predicate.name.clone(), predicate.arity);
    let mut draft = BranchDraft::new(BranchKind::ExternalFact { predicate }, 0);
    draft.goal_signature(Slot::Parent, &name, arity);
    draft.child_count();
    Ok(draft)
}
