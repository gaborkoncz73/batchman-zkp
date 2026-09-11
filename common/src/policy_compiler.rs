use std::collections::BTreeMap;

use crate::circuit_dialect::{BuiltinKind, CircuitLimits, PolicyAnalysis};
use crate::data::{
    ChildSig, ClauseTemplate, Equality, PredicateTemplate, RuleTemplateFile, RuleTemplateFileFp,
    TermRefComplex, TermSide,
};
use crate::policy_ir::{Call, Clause, PolicyFile, Term};
use crate::typed_policy::{
    TypedCandidateId, TypedClausePlan, TypedEquality, TypedGoalSignature, TypedGoalSlot,
    TypedPatternCheck, TypedPolicyPlan, TypedPredicatePlan, TypedTermRef,
};
use crate::typed_term::{TermPathStep, TermTag, hash_ground_term, symbol_to_field};
use crate::utils_2::common_helpers::{
    MAX_ARITY, MAX_CHILDREN, MAX_CLAUSES_PER_PREDICATE, MAX_EQUALITIES, MAX_PRED_LIST,
    MAX_PREDICATES_OVERALL,
};
use anyhow::{Result, bail};
use halo2_proofs::pasta::Fp;
use halo2curves::ff::Field;

use crate::witness_ir::GroundTerm;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Location {
    children_node_list: usize,
    predicate: usize,
    arg: usize,
    list_index: usize,
}

#[derive(Default)]
struct Occurrences {
    variables: BTreeMap<u32, Vec<Location>>,
    literals: Vec<(Location, String)>,
}

#[derive(Default)]
struct TypedOccurrences {
    variables: BTreeMap<u32, Vec<TypedTermRef>>,
    patterns: Vec<TypedPatternCheck>,
}

/// Deterministic, in-memory compilation result consumed by the current
/// circuit.  `PolicyFile` remains the source of truth; no serialized
/// circuit-v1 sidecar is involved in this API.
#[derive(Debug, Clone)]
pub struct CompiledPolicy {
    analysis: PolicyAnalysis,
    layout: RuleTemplateFile,
    rules: RuleTemplateFileFp,
    typed_plan: TypedPolicyPlan,
}

impl CompiledPolicy {
    pub fn analysis(&self) -> &PolicyAnalysis {
        &self.analysis
    }

    /// Deterministic compatibility layout, exposed for diagnostics and golden
    /// tests. Production callers should consume `rules_fp` instead.
    pub fn layout(&self) -> &RuleTemplateFile {
        &self.layout
    }

    pub fn rules_fp(&self) -> &RuleTemplateFileFp {
        &self.rules
    }

    pub fn into_rules_fp(self) -> RuleTemplateFileFp {
        self.rules
    }

    pub fn typed_plan(&self) -> &TypedPolicyPlan {
        &self.typed_plan
    }

    pub fn into_typed_plan(self) -> TypedPolicyPlan {
        self.typed_plan
    }

    /// Transfers both circuit-facing policy views without allowing their
    /// deterministic candidate order to be reconstructed independently.
    pub fn into_circuit_parts(self) -> (RuleTemplateFileFp, TypedPolicyPlan) {
        (self.rules, self.typed_plan)
    }
}

/// Validates the semantic policy against the configured circuit dialect and
/// then projects it into the bounded representation used by the current
/// circuit.
///
/// This projection is intentionally explicit and fallible.  Constructs that
/// the parser and semantic IR preserve but the current circuit cannot encode
/// are rejected instead of being truncated or silently reinterpreted.
pub fn compile_policy_for_current_circuit(
    policy: &PolicyFile,
    limits: &CircuitLimits,
) -> Result<CompiledPolicy> {
    let analysis = policy
        .validate_for_circuit(limits)
        .map_err(anyhow::Error::new)?;
    let (layout, typed_plan) = lower_validated_policy(policy)?;
    let rules = RuleTemplateFileFp::from(&layout);
    Ok(CompiledPolicy {
        analysis,
        layout,
        rules,
        typed_plan,
    })
}

impl Occurrences {
    fn variable(&mut self, id: u32, location: Location) {
        self.variables.entry(id).or_default().push(location);
    }

    fn literal(&mut self, location: Location, value: String) {
        self.literals.push((location, value));
    }

    fn atom(&mut self, location: Location, value: &str) {
        // `TermSide::Value` is a legacy string carrier. Preserve the one
        // otherwise ambiguous case explicitly so RuleTemplateFileFp can keep
        // atom `1` distinct from integer 1.
        if value.parse::<u64>().is_ok() {
            self.literal(location, format!("'{value}'"));
        } else {
            self.literal(location, value.to_owned());
        }
    }

    fn into_equalities(self) -> Vec<Equality> {
        let mut equalities = Vec::new();
        for occurrences in self.variables.into_values() {
            if let Some(first) = occurrences.first().copied() {
                for other in occurrences.into_iter().skip(1) {
                    equalities.push(Equality {
                        left: TermSide::Ref(first.into()),
                        right: TermSide::Ref(other.into()),
                    });
                }
            }
        }
        equalities.extend(self.literals.into_iter().map(|(location, value)| Equality {
            left: TermSide::Ref(location.into()),
            right: TermSide::Value(value),
        }));
        equalities
    }
}

impl From<Location> for TermRefComplex {
    fn from(location: Location) -> Self {
        Self {
            children_node_list: location.children_node_list,
            predicate: location.predicate,
            arg: location.arg,
            list_index: location.list_index,
        }
    }
}

fn lower_validated_policy(policy: &PolicyFile) -> Result<(RuleTemplateFile, TypedPolicyPlan)> {
    validate_directives_for_circuit_v1(policy)?;
    let mut predicates = Vec::with_capacity(policy.predicates.len());
    let mut typed_predicates = Vec::with_capacity(policy.predicates.len());

    for predicate in &policy.predicates {
        let candidate_count = predicate.clauses.iter().fold(0_usize, |total, clause| {
            total.saturating_add(body_branch_count(&clause.body))
        });
        if candidate_count > MAX_CLAUSES_PER_PREDICATE {
            bail!(
                "circuit-v1 supports at most {MAX_CLAUSES_PER_PREDICATE} clauses per predicate, but {}/{} normalizes to {candidate_count}",
                predicate.indicator.name,
                predicate.indicator.arity
            );
        }

        let mut clauses = Vec::with_capacity(candidate_count);
        let mut typed_candidates = Vec::with_capacity(candidate_count);
        for clause in &predicate.clauses {
            let branches = body_branches(&clause.body);
            for (branch_ordinal, branch) in branches.into_iter().enumerate() {
                // Both views are derived in this single loop. Their candidate
                // order therefore cannot drift through an independent branch
                // normalization pass.
                clauses.push(lower_clause(clause, branch.clone())?);
                typed_candidates.push(lower_typed_clause(clause, branch_ordinal, branch)?);
            }
        }

        let arity = u8::try_from(predicate.indicator.arity).map_err(|_| {
            anyhow::anyhow!(
                "typed predicate arity {} does not fit u8 for {}/{}",
                predicate.indicator.arity,
                predicate.indicator.name,
                predicate.indicator.arity
            )
        })?;
        predicates.push(PredicateTemplate {
            name: predicate.indicator.name.clone(),
            arity: predicate.indicator.arity,
            clauses,
        });
        typed_predicates.push(TypedPredicatePlan {
            name: symbol_to_field(&predicate.indicator.name),
            arity,
            candidates: typed_candidates,
        });
    }

    let rules = RuleTemplateFile { predicates };
    validate_circuit_v1(&rules)?;
    let typed_plan = TypedPolicyPlan {
        predicates: typed_predicates,
    };
    typed_plan.validate().map_err(anyhow::Error::msg)?;
    Ok((rules, typed_plan))
}

pub fn to_circuit_v1_json(rules: &RuleTemplateFile) -> Result<String> {
    let mut json = serde_json::to_string_pretty(rules)?;
    json.push('\n');
    Ok(json)
}

fn validate_directives_for_circuit_v1(policy: &PolicyFile) -> Result<()> {
    for directive in &policy.directives {
        if !matches!(
            directive,
            Term::Compound { functor, args } if functor == "module" && args.len() == 2
        ) {
            bail!("directive {directive:?} cannot be represented by circuit-v1");
        }
    }
    Ok(())
}

fn lower_clause(source_clause: &Clause, goals: Vec<&Term>) -> Result<ClauseTemplate> {
    let mut occurrences = Occurrences::default();
    collect_call_arguments(&source_clause.head, 0, 0, &mut occurrences)?;

    let mut children = Vec::new();
    for (row, goal) in goals.into_iter().enumerate() {
        children.push(lower_goal(goal, row + 1, &mut occurrences)?);
    }

    Ok(ClauseTemplate {
        children,
        equalities: occurrences.into_equalities(),
    })
}

fn lower_typed_clause(
    source_clause: &Clause,
    branch_ordinal: usize,
    goals: Vec<&Term>,
) -> Result<TypedClausePlan> {
    let mut occurrences = TypedOccurrences::default();
    for (argument, term) in source_clause.head.args.iter().enumerate() {
        let at = TypedTermRef::root(TypedGoalSlot::Parent, argument).map_err(anyhow::Error::msg)?;
        collect_typed_pattern(term, at, &mut occurrences)?;
    }

    let mut children = Vec::with_capacity(goals.len());
    for (child_index, goal) in goals.into_iter().enumerate() {
        children.push(lower_typed_goal(goal, child_index, &mut occurrences)?);
    }

    Ok(TypedClausePlan {
        id: TypedCandidateId {
            source_clause_ordinal: source_clause.ordinal,
            branch_ordinal,
        },
        children,
        equalities: occurrences.into_equalities(),
        patterns: occurrences.patterns,
    })
}

fn lower_typed_goal(
    goal: &Term,
    child_index: usize,
    occurrences: &mut TypedOccurrences,
) -> Result<TypedGoalSignature> {
    let child_index = u8::try_from(child_index)
        .map_err(|_| anyhow::anyhow!("typed child index {child_index} does not fit u8"))?;
    match goal {
        Term::Atom { value } if value == "!" => {
            bail!("cut cannot be represented by the typed policy plan")
        }
        Term::Atom { value } if value == "true" => {
            bail!("true must be removed before typed branch lowering")
        }
        Term::Atom { value } => Ok(TypedGoalSignature {
            name: symbol_to_field(value),
            arity: 0,
        }),
        Term::Compound { functor, args } => {
            if matches!(functor.as_str(), "," | ";" | "->" | "\\+" | ":") {
                bail!("control term {functor}/{} was not normalized", args.len());
            }
            let arity = u8::try_from(args.len())
                .map_err(|_| anyhow::anyhow!("typed child arity {} does not fit u8", args.len()))?;
            for (argument, term) in args.iter().enumerate() {
                let at = TypedTermRef::root(TypedGoalSlot::Child { index: child_index }, argument)
                    .map_err(anyhow::Error::msg)?;
                collect_typed_pattern(term, at, occurrences)?;
            }
            Ok(TypedGoalSignature {
                name: symbol_to_field(functor),
                arity,
            })
        }
        other => bail!("term cannot be used as a typed policy goal: {other:?}"),
    }
}

fn collect_typed_pattern(
    term: &Term,
    at: TypedTermRef,
    occurrences: &mut TypedOccurrences,
) -> Result<()> {
    if !contains_variable(term) {
        occurrences.patterns.push(TypedPatternCheck::Exact {
            at,
            digest: ground_pattern_digest(term)?,
        });
        return Ok(());
    }

    match term {
        Term::Variable { id } => {
            occurrences.variables.entry(*id).or_default().push(at);
            Ok(())
        }
        Term::Compound { functor, args } => {
            let arity = u8::try_from(args.len()).map_err(|_| {
                anyhow::anyhow!("typed compound arity {} does not fit u8", args.len())
            })?;
            occurrences.patterns.push(TypedPatternCheck::Shape {
                at: at.clone(),
                tag: TermTag::Compound,
                payload: symbol_to_field(functor),
                arity,
            });
            for (index, argument) in args.iter().enumerate() {
                let index = u8::try_from(index)
                    .map_err(|_| anyhow::anyhow!("typed compound index {index} does not fit u8"))?;
                collect_typed_pattern(
                    argument,
                    at.pushed(TermPathStep::Argument { index }),
                    occurrences,
                )?;
            }
            Ok(())
        }
        Term::List { items, tail } => {
            if items.is_empty() {
                bail!("variable-bearing list with an empty item prefix is not canonical");
            }
            collect_typed_list_pattern(items, tail.as_deref(), at, occurrences)
        }
        // All non-variable leaves and variable-free constructors return in the
        // maximal-subtree fast path above.
        Term::Atom { .. }
        | Term::Integer { .. }
        | Term::Float { .. }
        | Term::String { .. }
        | Term::BackQuoted { .. } => unreachable!("leaf cannot contain a variable"),
    }
}

fn collect_typed_list_pattern(
    items: &[Term],
    tail: Option<&Term>,
    at: TypedTermRef,
    occurrences: &mut TypedOccurrences,
) -> Result<()> {
    let Some((head, rest)) = items.split_first() else {
        return if let Some(tail) = tail {
            collect_typed_pattern(tail, at, occurrences)
        } else {
            occurrences.patterns.push(TypedPatternCheck::Exact {
                at,
                digest: ground_pattern_digest(&Term::List {
                    items: Vec::new(),
                    tail: None,
                })?,
            });
            Ok(())
        };
    };

    let suffix_contains_variable =
        items.iter().any(contains_variable) || tail.is_some_and(contains_variable);
    if !suffix_contains_variable {
        occurrences.patterns.push(TypedPatternCheck::Exact {
            at,
            digest: ground_pattern_digest(&Term::List {
                items: items.to_vec(),
                tail: tail.cloned().map(Box::new),
            })?,
        });
        return Ok(());
    }

    occurrences.patterns.push(TypedPatternCheck::Shape {
        at: at.clone(),
        tag: TermTag::Cons,
        payload: Fp::ZERO,
        arity: 2,
    });
    collect_typed_pattern(head, at.pushed(TermPathStep::Head), occurrences)?;
    collect_typed_list_pattern(rest, tail, at.pushed(TermPathStep::Tail), occurrences)
}

fn ground_pattern_digest(term: &Term) -> Result<Fp> {
    let ground = GroundTerm::from_policy_term(term).map_err(anyhow::Error::new)?;
    hash_ground_term(&ground).map_err(anyhow::Error::new)
}

fn contains_variable(term: &Term) -> bool {
    match term {
        Term::Variable { .. } => true,
        Term::Compound { args, .. } => args.iter().any(contains_variable),
        Term::List { items, tail } => {
            items.iter().any(contains_variable) || tail.as_deref().is_some_and(contains_variable)
        }
        Term::Atom { .. }
        | Term::Integer { .. }
        | Term::Float { .. }
        | Term::String { .. }
        | Term::BackQuoted { .. } => false,
    }
}

impl TypedOccurrences {
    fn into_equalities(&self) -> Vec<TypedEquality> {
        let mut equalities = Vec::new();
        for occurrences in self.variables.values() {
            if let Some(first) = occurrences.first() {
                equalities.extend(
                    occurrences
                        .iter()
                        .skip(1)
                        .cloned()
                        .map(|right| TypedEquality {
                            left: first.clone(),
                            right,
                        }),
                );
            }
        }
        equalities
    }
}

fn lower_goal(goal: &Term, row: usize, occurrences: &mut Occurrences) -> Result<Vec<ChildSig>> {
    match goal {
        Term::Atom { value } if value == "!" => {
            bail!("cut cannot be represented by circuit-v1")
        }
        Term::Atom { value } => Ok(vec![ChildSig {
            name: value.clone(),
            arity: 0,
        }]),
        Term::Compound { functor, args } if functor == "is" => {
            if args.len() != 2 {
                bail!("is must have arity 2");
            }
            let (initial, steps) = accumulator_expression(&args[1])?;
            let mut children = vec![ChildSig {
                name: "is".to_owned(),
                arity: 2,
            }];
            collect_slot(&args[0], Location::new(row, 0, 0, 0), occurrences)?;
            collect_slot(initial, Location::new(row, 0, 1, 0), occurrences)?;
            for (index, (operator, operand)) in steps.into_iter().enumerate() {
                let predicate = index + 1;
                children.push(ChildSig {
                    name: operator.to_owned(),
                    arity: 1,
                });
                collect_slot(operand, Location::new(row, predicate, 0, 0), occurrences)?;
            }
            Ok(children)
        }
        Term::Compound { functor, args } => {
            if matches!(functor.as_str(), "," | ";" | "->" | "\\+" | ":") {
                bail!("control term {functor}/{} was not normalized", args.len());
            }
            let children = vec![ChildSig {
                name: if functor == "=<" {
                    BuiltinKind::LessThanOrEqual
                        .legacy_circuit_functor()
                        .to_owned()
                } else {
                    functor.clone()
                },
                arity: args.len(),
            }];
            for (argument, term) in args.iter().enumerate() {
                collect_argument(term, row, 0, argument, occurrences)?;
            }
            Ok(children)
        }
        other => bail!("term cannot be used as a circuit-v1 goal: {other:?}"),
    }
}

fn collect_call_arguments(
    call: &Call,
    row: usize,
    predicate: usize,
    occurrences: &mut Occurrences,
) -> Result<()> {
    for (argument, term) in call.args.iter().enumerate() {
        collect_argument(term, row, predicate, argument, occurrences)?;
    }
    Ok(())
}

fn collect_argument(
    term: &Term,
    row: usize,
    predicate: usize,
    argument: usize,
    occurrences: &mut Occurrences,
) -> Result<()> {
    match term {
        Term::List { items, tail } => {
            if items.is_empty() && tail.is_none() {
                occurrences.literal(Location::new(row, predicate, argument, 0), "[]".to_owned());
                return Ok(());
            }

            let mut list_index = 0;
            for item in items {
                let fields = comma_fields(item);
                for field in fields {
                    collect_slot(
                        field,
                        Location::new(row, predicate, argument, list_index),
                        occurrences,
                    )?;
                    list_index += 1;
                }
            }
            if let Some(tail) = tail {
                collect_slot(
                    tail,
                    Location::new(row, predicate, argument, list_index),
                    occurrences,
                )?;
            } else {
                occurrences.literal(
                    Location::new(row, predicate, argument, list_index),
                    "[]".to_owned(),
                );
            }
            Ok(())
        }
        _ => collect_slot(
            term,
            Location::new(row, predicate, argument, 0),
            occurrences,
        ),
    }
}

fn collect_slot(term: &Term, location: Location, occurrences: &mut Occurrences) -> Result<()> {
    match term {
        Term::Variable { id } => {
            occurrences.variable(*id, location);
            Ok(())
        }
        Term::Atom { value } => {
            occurrences.atom(location, value);
            Ok(())
        }
        Term::Integer { decimal } => {
            occurrences.literal(location, decimal.clone());
            Ok(())
        }
        Term::Float { lexeme } => {
            occurrences.literal(location, lexeme.clone());
            Ok(())
        }
        Term::List { items, tail } if items.is_empty() && tail.is_none() => {
            occurrences.literal(location, "[]".to_owned());
            Ok(())
        }
        Term::String { .. }
        | Term::BackQuoted { .. }
        | Term::Compound { .. }
        | Term::List { .. } => bail!(
            "nested term {term:?} cannot be represented in the circuit-v1 scalar slot at {location:?}"
        ),
    }
}

fn accumulator_expression(term: &Term) -> Result<(&Term, Vec<(&str, &Term)>)> {
    fn visit<'a>(term: &'a Term, steps: &mut Vec<(&'a str, &'a Term)>) -> Result<&'a Term> {
        match term {
            Term::Compound { functor, args }
                if is_arithmetic_operator(functor) && args.len() == 2 =>
            {
                let initial = visit(&args[0], steps)?;
                if matches!(&args[1], Term::Compound { functor, .. } if is_arithmetic_operator(functor))
                {
                    bail!(
                        "right-nested expression {term:?} cannot be represented by circuit-v1's accumulator bytecode"
                    );
                }
                steps.push((functor, &args[1]));
                Ok(initial)
            }
            Term::Compound { functor, args }
                if is_arithmetic_operator(functor) && args.len() == 1 =>
            {
                bail!("unary arithmetic operator {functor}/1 is not supported by circuit-v1")
            }
            _ => Ok(term),
        }
    }

    let mut steps = Vec::new();
    let initial = visit(term, &mut steps)?;
    Ok((initial, steps))
}

fn is_arithmetic_operator(functor: &str) -> bool {
    matches!(
        functor,
        "+" | "-" | "*" | "/" | "//" | "div" | "mod" | "rem" | "**" | "^" | "<<" | ">>"
    )
}

fn body_branches(term: &Term) -> Vec<Vec<&Term>> {
    match term {
        Term::Compound { functor, args } if functor == ";" && args.len() == 2 => {
            let mut branches = body_branches(&args[0]);
            branches.extend(body_branches(&args[1]));
            branches
        }
        Term::Compound { functor, args } if functor == "," && args.len() == 2 => {
            let left = body_branches(&args[0]);
            let right = body_branches(&args[1]);
            left.into_iter()
                .flat_map(|left_branch| {
                    right.iter().cloned().map(move |right_branch| {
                        let mut branch = left_branch.clone();
                        branch.extend(right_branch);
                        branch
                    })
                })
                .collect()
        }
        Term::Atom { value } if value == "true" => vec![Vec::new()],
        goal => vec![vec![goal]],
    }
}

fn body_branch_count(term: &Term) -> usize {
    match term {
        Term::Compound { functor, args } if functor == ";" && args.len() == 2 => {
            body_branch_count(&args[0]).saturating_add(body_branch_count(&args[1]))
        }
        Term::Compound { functor, args } if functor == "," && args.len() == 2 => {
            body_branch_count(&args[0]).saturating_mul(body_branch_count(&args[1]))
        }
        _ => 1,
    }
}

fn conjunction_goals(term: &Term) -> Vec<&Term> {
    match term {
        Term::Compound { functor, args } if functor == "," && args.len() == 2 => {
            let mut goals = conjunction_goals(&args[0]);
            goals.extend(conjunction_goals(&args[1]));
            goals
        }
        _ => vec![term],
    }
}

fn comma_fields(term: &Term) -> Vec<&Term> {
    conjunction_goals(term)
}

fn validate_circuit_v1(rules: &RuleTemplateFile) -> Result<()> {
    if rules.predicates.len() > MAX_PREDICATES_OVERALL {
        bail!(
            "circuit-v1 supports at most {MAX_PREDICATES_OVERALL} predicates, got {}",
            rules.predicates.len()
        );
    }

    for predicate in &rules.predicates {
        if predicate.arity > MAX_ARITY {
            bail!(
                "circuit-v1 supports arity at most {MAX_ARITY}, but {}/{} exceeds it",
                predicate.name,
                predicate.arity
            );
        }
        if predicate.clauses.len() > MAX_CLAUSES_PER_PREDICATE {
            bail!(
                "circuit-v1 supports at most {MAX_CLAUSES_PER_PREDICATE} clauses per predicate, but {}/{} lowers to {}",
                predicate.name,
                predicate.arity,
                predicate.clauses.len()
            );
        }

        for (clause_index, clause) in predicate.clauses.iter().enumerate() {
            if clause.children.len() > MAX_CHILDREN {
                bail!(
                    "circuit-v1 supports at most {MAX_CHILDREN} body goals, but {}/{} clause {} has {}",
                    predicate.name,
                    predicate.arity,
                    clause_index,
                    clause.children.len()
                );
            }
            if clause.equalities.len() > MAX_EQUALITIES {
                bail!(
                    "circuit-v1 supports at most {MAX_EQUALITIES} equalities, but {}/{} clause {} has {}",
                    predicate.name,
                    predicate.arity,
                    clause_index,
                    clause.equalities.len()
                );
            }

            for (row_index, row) in clause.children.iter().enumerate() {
                if row.is_empty() {
                    bail!(
                        "circuit-v1 body row {} of {}/{} clause {} is empty",
                        row_index + 1,
                        predicate.name,
                        predicate.arity,
                        clause_index
                    );
                }
                if row.len() > MAX_PRED_LIST {
                    bail!(
                        "circuit-v1 supports at most {MAX_PRED_LIST} predicates in a body row, but row {} of {}/{} clause {} has {}",
                        row_index + 1,
                        predicate.name,
                        predicate.arity,
                        clause_index,
                        row.len()
                    );
                }
                for child in row {
                    if child.arity > MAX_ARITY {
                        bail!(
                            "circuit-v1 supports arity at most {MAX_ARITY}, but child {}/{} in {}/{} clause {} exceeds it",
                            child.name,
                            child.arity,
                            predicate.name,
                            predicate.arity,
                            clause_index
                        );
                    }
                }
            }

            for equality in &clause.equalities {
                validate_side(&equality.left, predicate, clause, clause_index)?;
                validate_side(&equality.right, predicate, clause, clause_index)?;
            }
        }
    }

    Ok(())
}

fn validate_side(
    side: &TermSide,
    predicate: &PredicateTemplate,
    clause: &ClauseTemplate,
    clause_index: usize,
) -> Result<()> {
    let TermSide::Ref(reference) = side else {
        return Ok(());
    };

    if reference.list_index >= MAX_PRED_LIST {
        bail!(
            "circuit-v1 list index {} is outside 0..{MAX_PRED_LIST} in {}/{} clause {}",
            reference.list_index,
            predicate.name,
            predicate.arity,
            clause_index
        );
    }

    if reference.children_node_list == 0 {
        if reference.predicate != 0 {
            bail!(
                "circuit-v1 head references must use predicate index 0 in {}/{} clause {}",
                predicate.name,
                predicate.arity,
                clause_index
            );
        }
        if reference.arg >= predicate.arity {
            bail!(
                "circuit-v1 head argument {} is outside arity {} in {}/{} clause {}",
                reference.arg,
                predicate.arity,
                predicate.name,
                predicate.arity,
                clause_index
            );
        }
        return Ok(());
    }

    let row_index = reference.children_node_list - 1;
    let row = clause.children.get(row_index).ok_or_else(|| {
        anyhow::anyhow!(
            "circuit-v1 reference points to missing body row {} in {}/{} clause {}",
            reference.children_node_list,
            predicate.name,
            predicate.arity,
            clause_index
        )
    })?;
    let child = row.get(reference.predicate).ok_or_else(|| {
        anyhow::anyhow!(
            "circuit-v1 reference points to missing predicate {} in body row {} of {}/{} clause {}",
            reference.predicate,
            reference.children_node_list,
            predicate.name,
            predicate.arity,
            clause_index
        )
    })?;
    if reference.arg >= child.arity {
        bail!(
            "circuit-v1 argument {} is outside child arity {} for {}/{} in {}/{} clause {}",
            reference.arg,
            child.arity,
            child.name,
            child.arity,
            predicate.name,
            predicate.arity,
            clause_index
        );
    }

    Ok(())
}

impl Location {
    const fn new(
        children_node_list: usize,
        predicate: usize,
        arg: usize,
        list_index: usize,
    ) -> Self {
        Self {
            children_node_list,
            predicate,
            arg,
            list_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::policy_ir::Term;

    use super::accumulator_expression;

    #[test]
    fn lowers_left_spine_without_losing_operands() {
        let expression = Term::compound(
            "div",
            vec![
                Term::compound(
                    "*",
                    vec![Term::Variable { id: 0 }, Term::Variable { id: 1 }],
                ),
                Term::Integer {
                    decimal: "100".to_owned(),
                },
            ],
        );
        let (initial, steps) = accumulator_expression(&expression).unwrap();
        assert_eq!(initial, &Term::Variable { id: 0 });
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].0, "*");
        assert_eq!(steps[0].1, &Term::Variable { id: 1 });
        assert_eq!(steps[1].0, "div");
    }
}
