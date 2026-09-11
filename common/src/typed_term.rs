//! Canonical, type-preserving commitments for ground Prolog terms and goals.
//!
//! This module is deliberately circuit-independent. It defines the native
//! Poseidon protocol that a later circuit chip must implement exactly, plus
//! authenticated path openings for exposing only the subterms a selected rule
//! actually uses.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use halo2_gadgets::poseidon::primitives::{ConstantLength, Hash as PoseidonHash, P128Pow5T3};
use halo2_proofs::pasta::Fp;
use halo2curves::ff::{Field, FromUniformBytes};
use serde::{Deserialize, Serialize};

use crate::witness_ir::{GroundCall, GroundTerm};

pub const TYPED_TERM_HASH_SCHEMA_VERSION: u32 = 1;
pub const MAX_TYPED_COMPOUND_ARITY: usize = 4;

pub const TERM_NODE_HASH_INPUTS: usize = 9;
pub const GOAL_HASH_INPUTS: usize = 8;

// These values are protocol constants. The explicit tags are absorbed even
// though ConstantLength also separates the 9-word node and 8-word goal hash.
pub const TERM_NODE_DOMAIN_TAG: u64 = 0x4241_5454_4552_4d01;
pub const GOAL_DOMAIN_TAG: u64 = 0x4241_5454_474f_4101;

const SYMBOL_HASH_DOMAIN: &[u8] = b"batchman/typed-term/v1/symbol";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u64)]
pub enum TermTag {
    Atom = 1,
    Integer = 2,
    Nil = 3,
    Cons = 4,
    Compound = 5,
}

impl TermTag {
    pub const fn as_u64(self) -> u64 {
        self as u64
    }
}

/// One canonical preimage in the typed term Merkle tree.
///
/// Compound children are padded with zero to a fixed arity-four shape. A
/// zero-arity compound is rejected because the canonical Prolog
/// representation for it is `Atom`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedNodePreimage {
    Atom {
        symbol: Fp,
    },
    Integer {
        value: u64,
    },
    Nil,
    Cons {
        head: Fp,
        tail: Fp,
    },
    Compound {
        functor: Fp,
        arity: u8,
        children: [Fp; MAX_TYPED_COMPOUND_ARITY],
    },
}

impl TypedNodePreimage {
    pub fn from_ground_term(term: &GroundTerm) -> Result<Self, TypedTermError> {
        match term {
            GroundTerm::Atom { value } => Ok(Self::Atom {
                symbol: symbol_to_field(value),
            }),
            GroundTerm::Integer { value } => Ok(Self::Integer { value: *value }),
            GroundTerm::Nil => Ok(Self::Nil),
            GroundTerm::Cons { head, tail } => Ok(Self::Cons {
                head: hash_ground_term(head)?,
                tail: hash_ground_term(tail)?,
            }),
            GroundTerm::Compound { functor, args } => {
                if args.is_empty() {
                    return Err(TypedTermError::new(
                        "zero-arity compound must use the canonical atom representation",
                    ));
                }
                if args.len() > MAX_TYPED_COMPOUND_ARITY {
                    return Err(TypedTermError::new(format!(
                        "compound {functor}/{} exceeds typed-term maximum arity {MAX_TYPED_COMPOUND_ARITY}",
                        args.len()
                    )));
                }

                let mut children = [Fp::ZERO; MAX_TYPED_COMPOUND_ARITY];
                for (index, argument) in args.iter().enumerate() {
                    children[index] = hash_ground_term(argument)?;
                }
                Ok(Self::Compound {
                    functor: symbol_to_field(functor),
                    arity: args.len() as u8,
                    children,
                })
            }
        }
    }

    pub const fn tag(&self) -> TermTag {
        match self {
            Self::Atom { .. } => TermTag::Atom,
            Self::Integer { .. } => TermTag::Integer,
            Self::Nil => TermTag::Nil,
            Self::Cons { .. } => TermTag::Cons,
            Self::Compound { .. } => TermTag::Compound,
        }
    }

    pub fn hash(&self) -> Result<Fp, TypedTermError> {
        let mut inputs = [Fp::ZERO; TERM_NODE_HASH_INPUTS];
        inputs[0] = Fp::from(TERM_NODE_DOMAIN_TAG);
        inputs[1] = Fp::from(TYPED_TERM_HASH_SCHEMA_VERSION as u64);
        inputs[2] = Fp::from(self.tag().as_u64());

        match self {
            Self::Atom { symbol } => inputs[3] = *symbol,
            Self::Integer { value } => inputs[3] = Fp::from(*value),
            Self::Nil => {}
            Self::Cons { head, tail } => {
                inputs[4] = Fp::from(2);
                inputs[5] = *head;
                inputs[6] = *tail;
            }
            Self::Compound {
                functor,
                arity,
                children,
            } => {
                let arity = usize::from(*arity);
                if !(1..=MAX_TYPED_COMPOUND_ARITY).contains(&arity) {
                    return Err(TypedTermError::new(format!(
                        "compound preimage arity {arity} is outside 1..={MAX_TYPED_COMPOUND_ARITY}"
                    )));
                }
                if children[arity..].iter().any(|child| *child != Fp::ZERO) {
                    return Err(TypedTermError::new(
                        "compound preimage has non-zero children beyond its arity",
                    ));
                }
                inputs[3] = *functor;
                inputs[4] = Fp::from(arity as u64);
                inputs[5..].copy_from_slice(children);
            }
        }

        Ok(poseidon_hash_node(inputs))
    }

    fn child_at(&self, step: &TermPathStep) -> Result<Fp, TypedTermError> {
        match (self, step) {
            (Self::Cons { head, .. }, TermPathStep::Head) => Ok(*head),
            (Self::Cons { tail, .. }, TermPathStep::Tail) => Ok(*tail),
            (
                Self::Compound {
                    arity, children, ..
                },
                TermPathStep::Argument { index },
            ) if usize::from(*index) < usize::from(*arity) => Ok(children[usize::from(*index)]),
            (Self::Compound { arity, .. }, TermPathStep::Argument { index }) => {
                Err(TypedTermError::new(format!(
                    "compound path argument {index} is outside arity {arity}"
                )))
            }
            (node, step) => Err(TypedTermError::new(format!(
                "path step {step:?} cannot descend through {:?}",
                node.tag()
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TermPathStep {
    Head,
    Tail,
    Argument { index: u8 },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TermPath {
    pub steps: Vec<TermPathStep>,
}

impl TermPath {
    pub fn root() -> Self {
        Self::default()
    }

    pub fn new(steps: Vec<TermPathStep>) -> Self {
        Self { steps }
    }

    pub fn pushed(&self, step: TermPathStep) -> Self {
        let mut steps = self.steps.clone();
        steps.push(step);
        Self { steps }
    }
}

/// One authenticated parent on a root-to-target path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpeningAncestor {
    pub step: TermPathStep,
    pub parent: TypedNodePreimage,
}

/// Native witness proving that `target` occurs at `path` below `root`.
///
/// Ancestors are stored from root to target. `validate` folds them in reverse
/// and requires every selected child digest and the final root to match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedTermOpening {
    pub root: Fp,
    pub path: TermPath,
    pub target: TypedNodePreimage,
    pub ancestors: Vec<OpeningAncestor>,
}

impl TypedTermOpening {
    /// Validates the complete opening and returns the authenticated target
    /// digest.
    pub fn validate(&self) -> Result<Fp, TypedTermError> {
        if self.path.steps.len() != self.ancestors.len() {
            return Err(TypedTermError::new(format!(
                "opening has {} path steps but {} ancestors",
                self.path.steps.len(),
                self.ancestors.len()
            )));
        }

        let target_digest = self.target.hash()?;
        let mut current = target_digest;
        for level in (0..self.ancestors.len()).rev() {
            let ancestor = &self.ancestors[level];
            if ancestor.step != self.path.steps[level] {
                return Err(TypedTermError::new(format!(
                    "opening ancestor {level} does not match its declared path step"
                )));
            }
            let selected_child = ancestor.parent.child_at(&ancestor.step)?;
            if selected_child != current {
                return Err(TypedTermError::new(format!(
                    "opening ancestor {level} does not authenticate the selected child"
                )));
            }
            current = ancestor.parent.hash()?;
        }

        if current != self.root {
            return Err(TypedTermError::new(
                "opening recomputed root does not match the claimed root",
            ));
        }
        Ok(target_digest)
    }
}

/// Builds a canonical authenticated opening for one ground term path.
pub fn build_term_opening(
    term: &GroundTerm,
    path: TermPath,
) -> Result<TypedTermOpening, TypedTermError> {
    let root = hash_ground_term(term)?;
    let mut current = term;
    let mut ancestors = Vec::with_capacity(path.steps.len());

    for (depth, step) in path.steps.iter().enumerate() {
        let parent = TypedNodePreimage::from_ground_term(current)?;
        let next = match (current, step) {
            (GroundTerm::Cons { head, .. }, TermPathStep::Head) => head.as_ref(),
            (GroundTerm::Cons { tail, .. }, TermPathStep::Tail) => tail.as_ref(),
            (GroundTerm::Compound { args, .. }, TermPathStep::Argument { index }) => {
                args.get(usize::from(*index)).ok_or_else(|| {
                    TypedTermError::new(format!(
                        "path step {depth} selects argument {index}, but compound arity is {}",
                        args.len()
                    ))
                })?
            }
            (node, step) => {
                return Err(TypedTermError::new(format!(
                    "path step {depth} ({step:?}) cannot descend through {node:?}"
                )));
            }
        };
        ancestors.push(OpeningAncestor {
            step: step.clone(),
            parent,
        });
        current = next;
    }

    let opening = TypedTermOpening {
        root,
        path,
        target: TypedNodePreimage::from_ground_term(current)?,
        ancestors,
    };
    opening.validate()?;
    Ok(opening)
}

/// Builds a deterministic, internally valid opening with exactly `path`.
///
/// This is the canonical filler for policy-fixed opening slots belonging to
/// unselected rule candidates.  The returned root is deliberately unrelated
/// to any proof goal: a circuit must connect it to a goal root through the
/// selected candidate flag.  Every ancestor is nevertheless a real canonical
/// node, so disabling that external root link never disables opening
/// authentication itself.
pub fn build_dummy_term_opening(path: &TermPath) -> Result<TypedTermOpening, TypedTermError> {
    const DUMMY_COMPOUND_FUNCTOR: &str = "$batchman_typed_opening_dummy";

    let mut root = GroundTerm::Nil;
    for step in path.steps.iter().rev() {
        root = match step {
            TermPathStep::Head => GroundTerm::Cons {
                head: Box::new(root),
                tail: Box::new(GroundTerm::Nil),
            },
            TermPathStep::Tail => GroundTerm::Cons {
                head: Box::new(GroundTerm::Nil),
                tail: Box::new(root),
            },
            TermPathStep::Argument { index } if usize::from(*index) < MAX_TYPED_COMPOUND_ARITY => {
                let mut args = vec![GroundTerm::Nil; usize::from(*index) + 1];
                args[usize::from(*index)] = root;
                GroundTerm::Compound {
                    functor: DUMMY_COMPOUND_FUNCTOR.to_owned(),
                    args,
                }
            }
            TermPathStep::Argument { index } => {
                return Err(TypedTermError::new(format!(
                    "dummy opening argument {index} exceeds typed-term maximum index {}",
                    MAX_TYPED_COMPOUND_ARITY - 1
                )));
            }
        };
    }

    build_term_opening(&root, path.clone())
}

pub fn hash_ground_term(term: &GroundTerm) -> Result<Fp, TypedTermError> {
    TypedNodePreimage::from_ground_term(term)?.hash()
}

/// Hashes one canonical ground call from its typed argument roots.
pub fn hash_ground_goal(goal: &GroundCall) -> Result<Fp, TypedTermError> {
    goal.validate()
        .map_err(|error| TypedTermError::new(error.to_string()))?;
    if goal.args.len() > MAX_TYPED_COMPOUND_ARITY {
        return Err(TypedTermError::new(format!(
            "goal {}/{} exceeds typed-term maximum arity {MAX_TYPED_COMPOUND_ARITY}",
            goal.predicate.name, goal.predicate.arity
        )));
    }

    let mut inputs = [Fp::ZERO; GOAL_HASH_INPUTS];
    inputs[0] = Fp::from(GOAL_DOMAIN_TAG);
    inputs[1] = Fp::from(TYPED_TERM_HASH_SCHEMA_VERSION as u64);
    inputs[2] = symbol_to_field(&goal.predicate.name);
    inputs[3] = Fp::from(goal.predicate.arity as u64);
    for (index, argument) in goal.args.iter().enumerate() {
        inputs[4 + index] = hash_ground_term(argument)?;
    }
    Ok(poseidon_hash_goal(inputs))
}

/// Domain-separated symbol-to-field mapping. Unlike the legacy codec, this
/// never parses digit-only atoms as integers.
pub fn symbol_to_field(symbol: &str) -> Fp {
    let mut hasher = blake3::Hasher::new();
    hasher.update(SYMBOL_HASH_DOMAIN);
    hasher.update(&(symbol.len() as u64).to_le_bytes());
    hasher.update(symbol.as_bytes());

    let mut uniform = [0_u8; 64];
    hasher.finalize_xof().fill(&mut uniform);
    Fp::from_uniform_bytes(&uniform)
}

fn poseidon_hash_node(inputs: [Fp; TERM_NODE_HASH_INPUTS]) -> Fp {
    PoseidonHash::<Fp, P128Pow5T3, ConstantLength<TERM_NODE_HASH_INPUTS>, 3, 2>::init().hash(inputs)
}

fn poseidon_hash_goal(inputs: [Fp; GOAL_HASH_INPUTS]) -> Fp {
    PoseidonHash::<Fp, P128Pow5T3, ConstantLength<GOAL_HASH_INPUTS>, 3, 2>::init().hash(inputs)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedTermError {
    message: String,
}

impl TypedTermError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for TypedTermError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for TypedTermError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy_ir::PredicateIndicator;

    fn atom(value: &str) -> GroundTerm {
        GroundTerm::Atom {
            value: value.to_owned(),
        }
    }

    fn integer(value: u64) -> GroundTerm {
        GroundTerm::Integer { value }
    }

    fn cons(head: GroundTerm, tail: GroundTerm) -> GroundTerm {
        GroundTerm::Cons {
            head: Box::new(head),
            tail: Box::new(tail),
        }
    }

    fn compound(functor: &str, args: Vec<GroundTerm>) -> GroundTerm {
        GroundTerm::Compound {
            functor: functor.to_owned(),
            args,
        }
    }

    #[test]
    fn digit_atom_and_integer_have_distinct_typed_hashes() {
        assert_ne!(
            hash_ground_term(&atom("1")).unwrap(),
            hash_ground_term(&integer(1)).unwrap()
        );
    }

    #[test]
    fn nil_and_bracket_atom_have_distinct_typed_hashes() {
        assert_ne!(
            hash_ground_term(&GroundTerm::Nil).unwrap(),
            hash_ground_term(&atom("[]")).unwrap()
        );
    }

    #[test]
    fn list_and_compound_order_are_bound() {
        let list_ab = cons(atom("a"), cons(atom("b"), GroundTerm::Nil));
        let list_ba = cons(atom("b"), cons(atom("a"), GroundTerm::Nil));
        assert_ne!(
            hash_ground_term(&list_ab).unwrap(),
            hash_ground_term(&list_ba).unwrap()
        );

        let compound_ab = compound("f", vec![atom("a"), atom("b")]);
        let compound_ba = compound("f", vec![atom("b"), atom("a")]);
        assert_ne!(
            hash_ground_term(&compound_ab).unwrap(),
            hash_ground_term(&compound_ba).unwrap()
        );
    }

    #[test]
    fn goal_argument_order_is_bound() {
        let goal = |args| GroundCall {
            predicate: PredicateIndicator {
                name: "p".to_owned(),
                arity: 2,
            },
            args,
        };
        assert_ne!(
            hash_ground_goal(&goal(vec![atom("a"), integer(1)])).unwrap(),
            hash_ground_goal(&goal(vec![integer(1), atom("a")])).unwrap()
        );
    }

    #[test]
    fn opening_authenticates_path_and_rejects_tampering() {
        let term = compound(
            "f",
            vec![
                cons(atom("a"), cons(atom("b"), GroundTerm::Nil)),
                compound("g", vec![integer(1), integer(2)]),
            ],
        );
        let path = TermPath::new(vec![
            TermPathStep::Argument { index: 0 },
            TermPathStep::Tail,
            TermPathStep::Head,
        ]);
        let opening = build_term_opening(&term, path).unwrap();
        assert_eq!(
            opening.validate().unwrap(),
            hash_ground_term(&atom("b")).unwrap()
        );

        let mut wrong_target = opening.clone();
        wrong_target.target = TypedNodePreimage::from_ground_term(&atom("c")).unwrap();
        assert!(wrong_target.validate().is_err());

        let mut wrong_path = opening.clone();
        wrong_path.path.steps[2] = TermPathStep::Tail;
        assert!(wrong_path.validate().is_err());

        let mut wrong_sibling = opening.clone();
        let TypedNodePreimage::Cons { tail, .. } = &mut wrong_sibling.ancestors[2].parent else {
            panic!("the final opening ancestor must be a cons node");
        };
        *tail += Fp::ONE;
        assert!(wrong_sibling.validate().is_err());

        let mut wrong_root = opening;
        wrong_root.root += Fp::ONE;
        assert!(wrong_root.validate().is_err());
    }

    #[test]
    fn dummy_opening_is_deterministic_valid_and_uses_the_exact_path() {
        let path = TermPath::new(vec![
            TermPathStep::Argument { index: 2 },
            TermPathStep::Tail,
            TermPathStep::Head,
        ]);
        let first = build_dummy_term_opening(&path).unwrap();
        let second = build_dummy_term_opening(&path).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.path, path);
        assert_eq!(first.ancestors.len(), 3);
        first.validate().unwrap();

        let invalid = TermPath::new(vec![TermPathStep::Argument {
            index: MAX_TYPED_COMPOUND_ARITY as u8,
        }]);
        assert!(build_dummy_term_opening(&invalid).is_err());
    }
}
