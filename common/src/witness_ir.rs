//! Lossless, parser-derived input for one local proof step.
//!
//! This layer is intentionally independent from the legacy `TermFp` matrix.
//! A ground Prolog term has exactly one representation here, regardless of
//! whether it occurs as a parent goal, a child goal, or an external fact.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::circuit_dialect::BuiltinKind;
use crate::policy_ir::{PredicateIndicator, Term};

pub const LOCAL_STEP_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GroundTerm {
    Atom {
        value: String,
    },
    Integer {
        value: u64,
    },
    Compound {
        functor: String,
        args: Vec<GroundTerm>,
    },
    Nil,
    Cons {
        head: Box<GroundTerm>,
        tail: Box<GroundTerm>,
    },
}

impl GroundTerm {
    pub fn from_policy_term(term: &Term) -> Result<Self, WitnessIrError> {
        match term {
            Term::Variable { id } => Err(WitnessIrError::new(format!(
                "ground witness term contains variable ID {id}"
            ))),
            Term::Atom { value } => Ok(Self::Atom {
                value: value.clone(),
            }),
            Term::Integer { decimal } => {
                let value = decimal.parse::<u64>().map_err(|_| {
                    WitnessIrError::new(format!("integer {decimal:?} does not fit u64"))
                })?;
                if value.to_string() != *decimal {
                    return Err(WitnessIrError::new(format!(
                        "integer {decimal:?} is not in canonical decimal form"
                    )));
                }
                Ok(Self::Integer { value })
            }
            Term::Compound { functor, args } => Ok(Self::Compound {
                functor: functor.clone(),
                args: args
                    .iter()
                    .map(Self::from_policy_term)
                    .collect::<Result<_, _>>()?,
            }),
            Term::List { items, tail } => {
                let tail = tail
                    .as_deref()
                    .map(Self::from_policy_term)
                    .transpose()?
                    .unwrap_or(Self::Nil);
                items.iter().rev().try_fold(tail, |tail, item| {
                    Ok(Self::Cons {
                        head: Box::new(Self::from_policy_term(item)?),
                        tail: Box::new(tail),
                    })
                })
            }
            Term::Float { .. } => Err(WitnessIrError::new(
                "float literals are outside the circuit witness dialect",
            )),
            Term::String { .. } => Err(WitnessIrError::new(
                "double-quoted strings are outside the circuit witness dialect",
            )),
            Term::BackQuoted { .. } => Err(WitnessIrError::new(
                "back-quoted strings are outside the circuit witness dialect",
            )),
        }
    }

    fn validate(&self) -> Result<(), WitnessIrError> {
        match self {
            Self::Atom { .. } | Self::Integer { .. } | Self::Nil => Ok(()),
            Self::Compound { functor, args } => {
                if functor.is_empty() {
                    return Err(WitnessIrError::new("compound functor must not be empty"));
                }
                for argument in args {
                    argument.validate()?;
                }
                Ok(())
            }
            Self::Cons { head, tail } => {
                head.validate()?;
                tail.validate()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GroundCall {
    pub predicate: PredicateIndicator,
    pub args: Vec<GroundTerm>,
}

impl GroundCall {
    pub fn from_goal_term(term: &Term) -> Result<Self, WitnessIrError> {
        let call = match term {
            Term::Atom { value } => Ok(Self {
                predicate: PredicateIndicator {
                    name: value.clone(),
                    arity: 0,
                },
                args: Vec::new(),
            }),
            Term::Compound { functor, args } => Ok(Self {
                predicate: PredicateIndicator {
                    name: functor.clone(),
                    arity: args.len(),
                },
                args: args
                    .iter()
                    .map(GroundTerm::from_policy_term)
                    .collect::<Result<_, _>>()?,
            }),
            other => Err(WitnessIrError::new(format!(
                "local-step goal must be callable, got {other:?}"
            ))),
        }?;
        call.validate()?;
        Ok(call)
    }

    pub fn validate(&self) -> Result<(), WitnessIrError> {
        if self.predicate.name.is_empty() {
            return Err(WitnessIrError::new(
                "ground goal predicate name must not be empty",
            ));
        }
        if self.predicate.arity != self.args.len() {
            return Err(WitnessIrError::new(format!(
                "ground goal {}/{} contains {} arguments",
                self.predicate.name,
                self.predicate.arity,
                self.args.len()
            )));
        }
        for argument in &self.args {
            argument.validate()?;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for GroundCall {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawGroundCall {
            predicate: PredicateIndicator,
            args: Vec<GroundTerm>,
        }

        let raw = RawGroundCall::deserialize(deserializer)?;
        let call = Self {
            predicate: raw.predicate,
            args: raw.args,
        };
        call.validate().map_err(de::Error::custom)?;
        Ok(call)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GoalResolution {
    Policy { predicate: PredicateIndicator },
    ExternalFact { predicate: PredicateIndicator },
    Builtin { builtin: BuiltinKind },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedGoal {
    pub goal: GroundCall,
    pub resolution: GoalResolution,
}

impl ResolvedGoal {
    pub fn new(goal: GroundCall, resolution: GoalResolution) -> Result<Self, WitnessIrError> {
        validate_resolution(&goal, &resolution)?;
        Ok(Self { goal, resolution })
    }

    pub fn validate(&self) -> Result<(), WitnessIrError> {
        validate_resolution(&self.goal, &self.resolution)
    }
}

impl<'de> Deserialize<'de> for ResolvedGoal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawResolvedGoal {
            goal: GroundCall,
            resolution: GoalResolution,
        }

        let raw = RawResolvedGoal::deserialize(deserializer)?;
        Self::new(raw.goal, raw.resolution).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StepTransition {
    Rule { children: Vec<ResolvedGoal> },
    ExternalFact,
    Builtin { builtin: BuiltinKind },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalStepWitness {
    pub schema_version: u32,
    pub goal: GroundCall,
    pub resolution: GoalResolution,
    pub transition: StepTransition,
}

impl LocalStepWitness {
    pub fn new(
        goal: GroundCall,
        resolution: GoalResolution,
        transition: StepTransition,
    ) -> Result<Self, WitnessIrError> {
        let witness = Self {
            schema_version: LOCAL_STEP_SCHEMA_VERSION,
            goal,
            resolution,
            transition,
        };
        witness.validate()?;
        Ok(witness)
    }

    pub fn validate(&self) -> Result<(), WitnessIrError> {
        if self.schema_version != LOCAL_STEP_SCHEMA_VERSION {
            return Err(WitnessIrError::new(format!(
                "unsupported local-step schema version {}; expected {LOCAL_STEP_SCHEMA_VERSION}",
                self.schema_version
            )));
        }
        validate_resolution(&self.goal, &self.resolution)?;

        match (&self.resolution, &self.transition) {
            (GoalResolution::Policy { .. }, StepTransition::Rule { children }) => {
                for child in children {
                    child.validate()?;
                }
                Ok(())
            }
            (GoalResolution::ExternalFact { .. }, StepTransition::ExternalFact) => Ok(()),
            (
                GoalResolution::Builtin { builtin: left },
                StepTransition::Builtin { builtin: right },
            ) if left == right => Ok(()),
            _ => Err(WitnessIrError::new(
                "goal resolution and local-step transition do not agree",
            )),
        }
    }

    pub fn to_canonical_json(&self) -> Result<String, serde_json::Error> {
        let mut json = serde_json::to_string_pretty(self)?;
        json.push('\n');
        Ok(json)
    }
}

impl<'de> Deserialize<'de> for LocalStepWitness {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawLocalStepWitness {
            schema_version: u32,
            goal: GroundCall,
            resolution: GoalResolution,
            transition: StepTransition,
        }

        let raw = RawLocalStepWitness::deserialize(deserializer)?;
        let witness = Self {
            schema_version: raw.schema_version,
            goal: raw.goal,
            resolution: raw.resolution,
            transition: raw.transition,
        };
        witness.validate().map_err(de::Error::custom)?;
        Ok(witness)
    }
}

fn validate_resolution(
    goal: &GroundCall,
    resolution: &GoalResolution,
) -> Result<(), WitnessIrError> {
    goal.validate()?;
    match resolution {
        GoalResolution::Policy { predicate } | GoalResolution::ExternalFact { predicate } => {
            if predicate != &goal.predicate {
                return Err(WitnessIrError::new(
                    "resolved predicate does not match the ground goal predicate",
                ));
            }
            if builtin_for_indicator(predicate).is_some() {
                return Err(WitnessIrError::new(
                    "builtin predicate cannot be labelled as a policy or external goal",
                ));
            }
        }
        GoalResolution::Builtin { builtin } => {
            if goal.predicate.name != builtin.functor() || goal.predicate.arity != builtin.arity() {
                return Err(WitnessIrError::new(format!(
                    "builtin {builtin:?} does not match ground goal {}/{}",
                    goal.predicate.name, goal.predicate.arity
                )));
            }
        }
    }
    Ok(())
}

fn builtin_for_indicator(indicator: &PredicateIndicator) -> Option<BuiltinKind> {
    if indicator.arity != 2 {
        return None;
    }
    match indicator.name.as_str() {
        "=" => Some(BuiltinKind::Unify),
        "is" => Some(BuiltinKind::Is),
        "<" => Some(BuiltinKind::LessThan),
        "=<" => Some(BuiltinKind::LessThanOrEqual),
        ">" => Some(BuiltinKind::GreaterThan),
        ">=" => Some(BuiltinKind::GreaterThanOrEqual),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitnessIrError {
    message: String,
}

impl WitnessIrError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for WitnessIrError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for WitnessIrError {}
