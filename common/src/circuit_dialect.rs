//! The explicit semantic boundary between the lossless Prolog IR and the
//! local-step circuit.
//!
//! The ANTLR frontend intentionally understands more syntax than the circuit.
//! Nothing may cross into witness generation merely because it was parseable:
//! it must also pass the allow-list and resource limits in this module.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::policy_ir::{PolicyFile, PredicateIndicator, Term};

pub const CIRCUIT_DIALECT: &str = "batchman_local_step_circuit_v1";

pub const SUPPORTED_CONTROL_FUNCTORS: &[&str] = &[",", ";"];
pub const SUPPORTED_RELATION_BUILTINS: &[&str] = &["=", "is", "<", "=<", ">", ">="];
pub const SUPPORTED_ARITHMETIC_OPERATORS: &[&str] = &["+", "-", "*", "div"];

const UNSUPPORTED_CONTROL_FUNCTORS: &[&str] = &["->", "\\+", ":"];
const UNSUPPORTED_RELATION_BUILTINS: &[&str] = &[
    "\\=", "==", "\\==", "@<", "@=<", "@>", "@>=", "=..", "=:=", "=\\=", "<=",
];
const OTHER_ARITHMETIC_OPERATORS: &[&str] = &[
    "/", "//", "mod", "rem", "**", "^", "<<", ">>", "/\\", "\\/", "\\",
];

/// Explicit protocol/input bounds. They are data, not hidden case-study
/// constants; prover and verifier must load the same profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CircuitLimits {
    pub integer_bits: usize,
    pub max_predicate_arity: usize,
    pub max_body_goals_per_branch: usize,
    pub max_term_nodes: usize,
    pub max_term_depth: usize,
    pub max_list_items: usize,
    pub max_arithmetic_operations: usize,
    pub max_proof_roots: usize,
    pub max_proof_nodes: usize,
    pub max_proof_depth: usize,
    pub max_children_per_node: usize,
    pub max_goal_bytes: usize,
    pub max_external_facts: usize,
}

impl CircuitLimits {
    pub fn validate(&self) -> Result<(), CircuitValidationError> {
        let positive = [
            ("max_predicate_arity", self.max_predicate_arity),
            ("max_body_goals_per_branch", self.max_body_goals_per_branch),
            ("max_term_nodes", self.max_term_nodes),
            ("max_term_depth", self.max_term_depth),
            ("max_list_items", self.max_list_items),
            ("max_arithmetic_operations", self.max_arithmetic_operations),
            ("max_proof_roots", self.max_proof_roots),
            ("max_proof_nodes", self.max_proof_nodes),
            ("max_proof_depth", self.max_proof_depth),
            ("max_children_per_node", self.max_children_per_node),
            ("max_goal_bytes", self.max_goal_bytes),
            ("max_external_facts", self.max_external_facts),
        ];
        for (field, value) in positive {
            if value == 0 {
                return Err(CircuitValidationError::new(
                    format!("circuit_limits.{field}"),
                    "limit must be greater than zero",
                ));
            }
        }
        if !(1..=64).contains(&self.integer_bits) {
            return Err(CircuitValidationError::new(
                "circuit_limits.integer_bits",
                "integer_bits must be between 1 and 64",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CircuitValidationError {
    pub path: String,
    pub message: String,
}

impl CircuitValidationError {
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl Display for CircuitValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}

impl Error for CircuitValidationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuiltinKind {
    Unify,
    Is,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl BuiltinKind {
    pub const fn functor(self) -> &'static str {
        match self {
            Self::Unify => "=",
            Self::Is => "is",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "=<",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
        }
    }

    pub const fn arity(self) -> usize {
        2
    }

    /// Token expected by the unchanged circuit-v1 builtin gadget. Prolog uses
    /// `=<`, while that legacy gadget historically used `<=`.
    pub const fn legacy_circuit_functor(self) -> &'static str {
        match self {
            Self::LessThanOrEqual => "<=",
            _ => self.functor(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitGoalKind {
    Predicate(PredicateIndicator),
    Builtin(BuiltinKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyAnalysis {
    pub defined_predicates: BTreeSet<PredicateIndicator>,
    pub required_external_predicates: BTreeSet<PredicateIndicator>,
    pub used_builtins: BTreeSet<BuiltinKind>,
    pub used_arithmetic_operators: BTreeSet<String>,
    pub predicate_count: usize,
    pub source_clause_count: usize,
    pub effective_candidate_count: usize,
    pub max_arity: usize,
    pub max_body_goals_per_branch: usize,
    pub max_variables_per_clause: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VariableMode {
    Allow,
    Forbid,
}

/// Validation of a policy against the legacy circuit dialect.
pub trait CircuitPolicyValidation {
    fn validate_for_circuit(
        &self,
        limits: &CircuitLimits,
    ) -> Result<PolicyAnalysis, CircuitValidationError>;
}

impl CircuitPolicyValidation for PolicyFile {
    /// Validates this policy without lowering or mutating it.
    fn validate_for_circuit(
        &self,
        limits: &CircuitLimits,
    ) -> Result<PolicyAnalysis, CircuitValidationError> {
        limits.validate()?;
        self.validate()
            .map_err(|message| CircuitValidationError::new("policy", message))?;
        if self.predicates.is_empty() {
            return Err(CircuitValidationError::new(
                "predicates",
                "a circuit policy must define at least one predicate",
            ));
        }

        for (predicate_index, predicate) in self.predicates.iter().enumerate() {
            let predicate_path = format!("predicates[{predicate_index}]");
            validate_indicator(&predicate.indicator, &format!("{predicate_path}.indicator"))?;
            if predicate.indicator.arity > limits.max_predicate_arity {
                return Err(CircuitValidationError::new(
                    format!("{predicate_path}.indicator.arity"),
                    format!(
                        "arity {} exceeds configured maximum {}",
                        predicate.indicator.arity, limits.max_predicate_arity
                    ),
                ));
            }
            if is_reserved_goal_functor(&predicate.indicator.name) {
                return Err(CircuitValidationError::new(
                    format!("{predicate_path}.indicator.name"),
                    format!(
                        "{}/{} is reserved by {CIRCUIT_DIALECT}",
                        predicate.indicator.name, predicate.indicator.arity
                    ),
                ));
            }
            if predicate.clauses.is_empty() {
                return Err(CircuitValidationError::new(
                    format!("{predicate_path}.clauses"),
                    "a circuit predicate must contain at least one clause",
                ));
            }

            for (clause_index, clause) in predicate.clauses.iter().enumerate() {
                let clause_path = format!("{predicate_path}.clauses[{clause_index}]");
                for (argument_index, argument) in clause.head.iter().enumerate() {
                    let path = format!("{clause_path}.head[{argument_index}]");
                    validate_term_shape(argument, limits, &path)?;
                    validate_data_term(argument, VariableMode::Allow, limits, &path)?;
                }

                let branch_goals = max_goals_in_branch(&clause.body);
                if branch_goals > limits.max_body_goals_per_branch {
                    return Err(CircuitValidationError::new(
                        format!("{clause_path}.body"),
                        format!(
                            "body branch has {branch_goals} goals, exceeding configured maximum {}",
                            limits.max_body_goals_per_branch
                        ),
                    ));
                }
                validate_policy_body(&clause.body, limits, &format!("{clause_path}.body"))?;
            }
        }

        Ok(analyze_policy(self))
    }
}

/// Validates and classifies one parsed proof-tree goal.
///
/// A proof node represents one local step, so control trees are rejected here.
/// Variables are rejected recursively: proof-tree goals must be ground.
pub fn validate_ground_goal_for_circuit(
    term: &Term,
    limits: &CircuitLimits,
) -> Result<CircuitGoalKind, CircuitValidationError> {
    limits.validate()?;
    validate_term_shape(term, limits, "goal")?;
    validate_single_goal(term, VariableMode::Forbid, limits, "goal")
}

fn validate_indicator(
    indicator: &PredicateIndicator,
    path: &str,
) -> Result<(), CircuitValidationError> {
    if indicator.name.is_empty() {
        return Err(CircuitValidationError::new(
            format!("{path}.name"),
            "predicate name must not be empty",
        ));
    }
    Ok(())
}

fn validate_policy_body(
    term: &Term,
    limits: &CircuitLimits,
    path: &str,
) -> Result<(), CircuitValidationError> {
    match term {
        Term::Compound { functor, args } if matches!(functor.as_str(), "," | ";") => {
            validate_arity(functor, args, 2, path)?;
            validate_policy_body(&args[0], limits, &format!("{path}.args[0]"))?;
            validate_policy_body(&args[1], limits, &format!("{path}.args[1]"))
        }
        Term::Atom { value } if value == "true" => Ok(()),
        _ => {
            validate_term_shape(term, limits, path)?;
            validate_single_goal(term, VariableMode::Allow, limits, path).map(|_| ())
        }
    }
}

fn validate_single_goal(
    term: &Term,
    variables: VariableMode,
    limits: &CircuitLimits,
    path: &str,
) -> Result<CircuitGoalKind, CircuitValidationError> {
    match term {
        Term::Atom { value } if value == "!" => Err(CircuitValidationError::new(
            path,
            "cut is not supported by the circuit",
        )),
        Term::Atom { value } if matches!(value.as_str(), "true" | "false") => {
            Err(CircuitValidationError::new(
                path,
                format!("{value}/0 is not supported as an executable circuit goal"),
            ))
        }
        Term::Atom { value } => {
            validate_callable_name(value, 0, path)?;
            Ok(CircuitGoalKind::Predicate(PredicateIndicator {
                name: value.clone(),
                arity: 0,
            }))
        }
        Term::Compound { functor, args } if matches!(functor.as_str(), "," | ";") => {
            Err(CircuitValidationError::new(
                path,
                format!(
                    "a proof node must contain one goal; {functor}/{} belongs to a policy body",
                    args.len()
                ),
            ))
        }
        Term::Compound { functor, args }
            if UNSUPPORTED_CONTROL_FUNCTORS.contains(&functor.as_str()) =>
        {
            Err(CircuitValidationError::new(
                path,
                format!("control operator {functor}/{} is not supported", args.len()),
            ))
        }
        Term::Compound { functor, args } if functor == "is" => {
            validate_arity(functor, args, 2, path)?;
            match (&args[0], variables) {
                (Term::Variable { .. }, VariableMode::Allow) | (Term::Integer { .. }, _) => {
                    validate_arithmetic_term(
                        &args[0],
                        variables,
                        limits,
                        &format!("{path}.args[0]"),
                    )?;
                }
                (Term::Variable { .. }, VariableMode::Forbid) => {
                    return Err(CircuitValidationError::new(
                        format!("{path}.args[0]"),
                        "proof-tree goals must be ground",
                    ));
                }
                _ => {
                    return Err(CircuitValidationError::new(
                        format!("{path}.args[0]"),
                        "left side of is/2 must be an integer or a policy variable",
                    ));
                }
            }
            validate_arithmetic_term(&args[1], variables, limits, &format!("{path}.args[1]"))?;
            Ok(CircuitGoalKind::Builtin(BuiltinKind::Is))
        }
        Term::Compound { functor, args } if functor == "=" => {
            validate_arity(functor, args, 2, path)?;
            validate_data_term(&args[0], variables, limits, &format!("{path}.args[0]"))?;
            validate_data_term(&args[1], variables, limits, &format!("{path}.args[1]"))?;
            Ok(CircuitGoalKind::Builtin(BuiltinKind::Unify))
        }
        Term::Compound { functor, args } if matches!(functor.as_str(), "<" | "=<" | ">" | ">=") => {
            validate_arity(functor, args, 2, path)?;
            validate_arithmetic_term(&args[0], variables, limits, &format!("{path}.args[0]"))?;
            validate_arithmetic_term(&args[1], variables, limits, &format!("{path}.args[1]"))?;
            let kind = match functor.as_str() {
                "<" => BuiltinKind::LessThan,
                "=<" => BuiltinKind::LessThanOrEqual,
                ">" => BuiltinKind::GreaterThan,
                ">=" => BuiltinKind::GreaterThanOrEqual,
                _ => unreachable!(),
            };
            Ok(CircuitGoalKind::Builtin(kind))
        }
        Term::Compound { functor, args }
            if UNSUPPORTED_RELATION_BUILTINS.contains(&functor.as_str()) =>
        {
            Err(CircuitValidationError::new(
                path,
                format!("relation builtin {functor}/{} is not supported", args.len()),
            ))
        }
        Term::Compound { functor, args }
            if SUPPORTED_ARITHMETIC_OPERATORS.contains(&functor.as_str())
                || OTHER_ARITHMETIC_OPERATORS.contains(&functor.as_str()) =>
        {
            Err(CircuitValidationError::new(
                path,
                format!(
                    "arithmetic expression {functor}/{} cannot be a standalone goal",
                    args.len()
                ),
            ))
        }
        Term::Compound { functor, args } => {
            validate_callable_name(functor, args.len(), path)?;
            if args.is_empty() {
                return Err(CircuitValidationError::new(
                    path,
                    "zero-arity call must use the canonical atom representation",
                ));
            }
            if args.len() > limits.max_predicate_arity {
                return Err(CircuitValidationError::new(
                    path,
                    format!(
                        "arity {} exceeds configured maximum {}",
                        args.len(),
                        limits.max_predicate_arity
                    ),
                ));
            }
            for (index, argument) in args.iter().enumerate() {
                validate_data_term(
                    argument,
                    variables,
                    limits,
                    &format!("{path}.args[{index}]"),
                )?;
            }
            Ok(CircuitGoalKind::Predicate(PredicateIndicator {
                name: functor.clone(),
                arity: args.len(),
            }))
        }
        other => Err(CircuitValidationError::new(
            path,
            format!(
                "a proof node must be a callable goal, got {}",
                term_kind(other)
            ),
        )),
    }
}

fn validate_callable_name(
    functor: &str,
    arity: usize,
    path: &str,
) -> Result<(), CircuitValidationError> {
    if functor.is_empty() {
        return Err(CircuitValidationError::new(
            path,
            "callable functor must not be empty",
        ));
    }
    if is_reserved_goal_functor(functor) {
        return Err(CircuitValidationError::new(
            path,
            format!("reserved goal functor {functor}/{arity} has an unsupported shape"),
        ));
    }
    Ok(())
}

fn validate_data_term(
    term: &Term,
    variables: VariableMode,
    limits: &CircuitLimits,
    path: &str,
) -> Result<(), CircuitValidationError> {
    match term {
        Term::Variable { .. } if variables == VariableMode::Allow => Ok(()),
        Term::Variable { .. } => Err(CircuitValidationError::new(
            path,
            "proof-tree goals must be ground",
        )),
        Term::Atom { .. } => Ok(()),
        Term::Integer { decimal } => validate_integer(decimal, limits, path),
        Term::Compound { functor, args } => {
            if functor.is_empty() {
                return Err(CircuitValidationError::new(
                    path,
                    "compound functor must not be empty",
                ));
            }
            if args.is_empty() {
                return Err(CircuitValidationError::new(
                    path,
                    "zero-arity compound must use the canonical atom representation",
                ));
            }
            if matches!(functor.as_str(), "{}" | "->" | "\\+" | ":" | ";") {
                return Err(CircuitValidationError::new(
                    path,
                    format!("data constructor {functor}/{} is not supported", args.len()),
                ));
            }
            if args.len() > limits.max_predicate_arity {
                return Err(CircuitValidationError::new(
                    path,
                    format!(
                        "compound arity {} exceeds configured maximum {}",
                        args.len(),
                        limits.max_predicate_arity
                    ),
                ));
            }
            for (index, argument) in args.iter().enumerate() {
                validate_data_term(
                    argument,
                    variables,
                    limits,
                    &format!("{path}.args[{index}]"),
                )?;
            }
            Ok(())
        }
        Term::List { items, tail } => {
            if items.is_empty() && tail.is_some() {
                return Err(CircuitValidationError::new(
                    path,
                    "list with an empty item prefix and an explicit tail is not canonical",
                ));
            }
            for (index, item) in items.iter().enumerate() {
                validate_data_term(item, variables, limits, &format!("{path}.items[{index}]"))?;
            }
            if let Some(tail) = tail {
                validate_data_term(tail, variables, limits, &format!("{path}.tail"))?;
            }
            Ok(())
        }
    }
}

fn validate_arithmetic_term(
    term: &Term,
    variables: VariableMode,
    limits: &CircuitLimits,
    path: &str,
) -> Result<(), CircuitValidationError> {
    match term {
        Term::Variable { .. } if variables == VariableMode::Allow => Ok(()),
        Term::Variable { .. } => Err(CircuitValidationError::new(
            path,
            "proof-tree goals must be ground",
        )),
        Term::Integer { decimal } => validate_integer(decimal, limits, path),
        Term::Compound { functor, args }
            if SUPPORTED_ARITHMETIC_OPERATORS.contains(&functor.as_str()) =>
        {
            validate_arity(functor, args, 2, path)?;
            validate_arithmetic_term(&args[0], variables, limits, &format!("{path}.args[0]"))?;
            validate_arithmetic_term(&args[1], variables, limits, &format!("{path}.args[1]"))?;
            if functor == "div" && matches!(&args[1], Term::Integer { decimal } if decimal == "0") {
                return Err(CircuitValidationError::new(
                    format!("{path}.args[1]"),
                    "division by the literal zero is not allowed",
                ));
            }
            if arithmetic_operation_count(term) > limits.max_arithmetic_operations {
                return Err(CircuitValidationError::new(
                    path,
                    format!(
                        "arithmetic expression exceeds configured maximum of {} operations",
                        limits.max_arithmetic_operations
                    ),
                ));
            }
            Ok(())
        }
        Term::Compound { functor, args }
            if OTHER_ARITHMETIC_OPERATORS.contains(&functor.as_str()) =>
        {
            Err(CircuitValidationError::new(
                path,
                format!(
                    "arithmetic operator {functor}/{} is not supported",
                    args.len()
                ),
            ))
        }
        other => Err(CircuitValidationError::new(
            path,
            format!(
                "arithmetic expressions support only variables, unsigned {}-bit integers and +, -, *, div; got {}",
                limits.integer_bits,
                term_kind(other)
            ),
        )),
    }
}

fn validate_integer(
    decimal: &str,
    limits: &CircuitLimits,
    path: &str,
) -> Result<(), CircuitValidationError> {
    let value = decimal.parse::<u64>().map_err(|_| {
        CircuitValidationError::new(
            path,
            format!(
                "integer must be a non-negative {}-bit decimal value",
                limits.integer_bits
            ),
        )
    })?;
    if value.to_string() != decimal {
        return Err(CircuitValidationError::new(
            path,
            "integer decimal representation must be canonical",
        ));
    }
    if limits.integer_bits < 64 && value >= (1_u64 << limits.integer_bits) {
        return Err(CircuitValidationError::new(
            path,
            format!(
                "integer is outside the configured unsigned {}-bit domain",
                limits.integer_bits
            ),
        ));
    }
    Ok(())
}

fn validate_term_shape(
    term: &Term,
    limits: &CircuitLimits,
    path: &str,
) -> Result<(), CircuitValidationError> {
    fn visit(
        term: &Term,
        depth: usize,
        nodes: &mut usize,
        limits: &CircuitLimits,
        path: &str,
    ) -> Result<(), CircuitValidationError> {
        *nodes += 1;
        if *nodes > limits.max_term_nodes {
            return Err(CircuitValidationError::new(
                path,
                format!(
                    "term exceeds configured maximum of {} nodes",
                    limits.max_term_nodes
                ),
            ));
        }
        if depth > limits.max_term_depth {
            return Err(CircuitValidationError::new(
                path,
                format!(
                    "term exceeds configured maximum depth {}",
                    limits.max_term_depth
                ),
            ));
        }
        match term {
            Term::Compound { args, .. } => {
                for argument in args {
                    visit(argument, depth + 1, nodes, limits, path)?;
                }
            }
            Term::List { items, tail } => {
                if items.len() > limits.max_list_items {
                    return Err(CircuitValidationError::new(
                        path,
                        format!(
                            "list has {} items, exceeding configured maximum {}",
                            items.len(),
                            limits.max_list_items
                        ),
                    ));
                }
                for item in items {
                    visit(item, depth + 1, nodes, limits, path)?;
                }
                if let Some(tail) = tail {
                    visit(tail, depth + 1, nodes, limits, path)?;
                }
            }
            Term::Variable { .. } | Term::Atom { .. } | Term::Integer { .. } => {}
        }
        Ok(())
    }

    let mut nodes = 0;
    visit(term, 1, &mut nodes, limits, path)
}

fn max_goals_in_branch(term: &Term) -> usize {
    match term {
        Term::Compound { functor, args } if functor == ";" && args.len() == 2 => {
            max_goals_in_branch(&args[0]).max(max_goals_in_branch(&args[1]))
        }
        Term::Compound { functor, args } if functor == "," && args.len() == 2 => {
            max_goals_in_branch(&args[0]) + max_goals_in_branch(&args[1])
        }
        Term::Atom { value } if value == "true" => 0,
        _ => 1,
    }
}

fn arithmetic_operation_count(term: &Term) -> usize {
    match term {
        Term::Compound { functor, args }
            if SUPPORTED_ARITHMETIC_OPERATORS.contains(&functor.as_str()) =>
        {
            1 + args.iter().map(arithmetic_operation_count).sum::<usize>()
        }
        _ => 0,
    }
}

fn analyze_policy(policy: &PolicyFile) -> PolicyAnalysis {
    let defined_predicates: BTreeSet<_> = policy
        .predicates
        .iter()
        .map(|predicate| predicate.indicator.clone())
        .collect();

    let mut required_external_predicates = BTreeSet::new();
    let mut used_builtins = BTreeSet::new();
    let mut used_arithmetic_operators = BTreeSet::new();
    let mut source_clause_count = 0;
    let mut effective_candidate_count = 0;
    let mut max_arity = policy
        .predicates
        .iter()
        .map(|predicate| predicate.indicator.arity)
        .max()
        .unwrap_or_default();
    let mut max_body_goals_per_branch = 0;
    let mut max_variables_per_clause = 0;

    for predicate in &policy.predicates {
        source_clause_count += predicate.clauses.len();
        for clause in &predicate.clauses {
            effective_candidate_count += disjunction_branch_count(&clause.body);
            max_body_goals_per_branch =
                max_body_goals_per_branch.max(max_goals_in_branch(&clause.body));
            max_variables_per_clause = max_variables_per_clause.max(clause.variables.len());
            collect_body_analysis(
                &clause.body,
                &defined_predicates,
                &mut required_external_predicates,
                &mut used_builtins,
                &mut used_arithmetic_operators,
                &mut max_arity,
            );
        }
    }

    PolicyAnalysis {
        predicate_count: defined_predicates.len(),
        defined_predicates,
        required_external_predicates,
        used_builtins,
        used_arithmetic_operators,
        source_clause_count,
        effective_candidate_count,
        max_arity,
        max_body_goals_per_branch,
        max_variables_per_clause,
    }
}

fn collect_body_analysis(
    term: &Term,
    defined: &BTreeSet<PredicateIndicator>,
    external: &mut BTreeSet<PredicateIndicator>,
    builtins: &mut BTreeSet<BuiltinKind>,
    arithmetic: &mut BTreeSet<String>,
    max_arity: &mut usize,
) {
    match term {
        Term::Compound { functor, args } if matches!(functor.as_str(), "," | ";") => {
            for argument in args {
                collect_body_analysis(argument, defined, external, builtins, arithmetic, max_arity);
            }
        }
        Term::Atom { value } if value == "true" => {}
        Term::Atom { value } => {
            let indicator = PredicateIndicator {
                name: value.clone(),
                arity: 0,
            };
            if !defined.contains(&indicator) {
                external.insert(indicator);
            }
        }
        Term::Compound { functor, args } if functor == "=" => {
            builtins.insert(BuiltinKind::Unify);
        }
        Term::Compound { functor, args } if functor == "is" => {
            builtins.insert(BuiltinKind::Is);
            collect_arithmetic_operators(&args[1], arithmetic);
        }
        Term::Compound { functor, args } if matches!(functor.as_str(), "<" | "=<" | ">" | ">=") => {
            builtins.insert(match functor.as_str() {
                "<" => BuiltinKind::LessThan,
                "=<" => BuiltinKind::LessThanOrEqual,
                ">" => BuiltinKind::GreaterThan,
                ">=" => BuiltinKind::GreaterThanOrEqual,
                _ => unreachable!(),
            });
            for argument in args {
                collect_arithmetic_operators(argument, arithmetic);
            }
        }
        Term::Compound { functor, args } => {
            *max_arity = (*max_arity).max(args.len());
            let indicator = PredicateIndicator {
                name: functor.clone(),
                arity: args.len(),
            };
            if !defined.contains(&indicator) {
                external.insert(indicator);
            }
        }
        Term::Variable { .. } | Term::Integer { .. } | Term::List { .. } => {}
    }
}

fn collect_arithmetic_operators(term: &Term, operators: &mut BTreeSet<String>) {
    if let Term::Compound { functor, args } = term {
        if SUPPORTED_ARITHMETIC_OPERATORS.contains(&functor.as_str()) {
            operators.insert(functor.clone());
        }
        for argument in args {
            collect_arithmetic_operators(argument, operators);
        }
    }
}

fn disjunction_branch_count(term: &Term) -> usize {
    match term {
        Term::Compound { functor, args } if functor == ";" && args.len() == 2 => {
            disjunction_branch_count(&args[0]).saturating_add(disjunction_branch_count(&args[1]))
        }
        Term::Compound { functor, args } if functor == "," && args.len() == 2 => {
            disjunction_branch_count(&args[0]).saturating_mul(disjunction_branch_count(&args[1]))
        }
        _ => 1,
    }
}

fn validate_arity(
    functor: &str,
    args: &[Term],
    expected: usize,
    path: &str,
) -> Result<(), CircuitValidationError> {
    if args.len() != expected {
        return Err(CircuitValidationError::new(
            path,
            format!("{functor} must have arity {expected}, got {}", args.len()),
        ));
    }
    Ok(())
}

fn is_reserved_goal_functor(functor: &str) -> bool {
    matches!(functor, "true" | "false" | "!")
        || SUPPORTED_CONTROL_FUNCTORS.contains(&functor)
        || UNSUPPORTED_CONTROL_FUNCTORS.contains(&functor)
        || SUPPORTED_RELATION_BUILTINS.contains(&functor)
        || UNSUPPORTED_RELATION_BUILTINS.contains(&functor)
        || SUPPORTED_ARITHMETIC_OPERATORS.contains(&functor)
        || OTHER_ARITHMETIC_OPERATORS.contains(&functor)
}

fn term_kind(term: &Term) -> &'static str {
    match term {
        Term::Variable { .. } => "variable",
        Term::Atom { .. } => "atom",
        Term::Integer { .. } => "integer",
        Term::Compound { .. } => "compound",
        Term::List { .. } => "list",
    }
}
