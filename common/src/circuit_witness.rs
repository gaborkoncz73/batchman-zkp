//! Fixed-width typed statement carried beside the temporary legacy matrix.
//!
//! The canonical ground calls are retained as opening witnesses; the field
//! view is derived from those calls and is never reconstructed from the lossy
//! legacy matrix.

use halo2_gadgets::poseidon::primitives::{ConstantLength, Hash as PoseidonHash, P128Pow5T3};
use halo2_proofs::pasta::Fp;
use halo2curves::ff::Field;

use crate::{
    policy_ir::PredicateIndicator,
    typed_term::{
        GOAL_DOMAIN_TAG, GOAL_HASH_INPUTS, MAX_TYPED_COMPOUND_ARITY,
        TYPED_TERM_HASH_SCHEMA_VERSION, hash_ground_term, symbol_to_field,
    },
    witness_ir::{GroundCall, LocalStepWitness, StepTransition},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedGoalFp {
    pub name: Fp,
    pub arity: Fp,
    pub argument_roots: [Fp; MAX_TYPED_COMPOUND_ARITY],
}

impl Default for TypedGoalFp {
    fn default() -> Self {
        Self {
            name: Fp::ZERO,
            arity: Fp::ZERO,
            argument_roots: [Fp::ZERO; MAX_TYPED_COMPOUND_ARITY],
        }
    }
}

impl TypedGoalFp {
    pub fn from_ground_call(call: &GroundCall) -> Result<Self, crate::typed_term::TypedTermError> {
        call.validate()
            .map_err(|error| crate::typed_term::TypedTermError::new(error.to_string()))?;
        if call.args.len() > MAX_TYPED_COMPOUND_ARITY {
            return Err(crate::typed_term::TypedTermError::new(format!(
                "goal {}/{} exceeds typed maximum arity {MAX_TYPED_COMPOUND_ARITY}",
                call.predicate.name, call.predicate.arity
            )));
        }
        let mut argument_roots = [Fp::ZERO; MAX_TYPED_COMPOUND_ARITY];
        for (index, argument) in call.args.iter().enumerate() {
            argument_roots[index] = hash_ground_term(argument)?;
        }
        Ok(Self {
            name: symbol_to_field(&call.predicate.name),
            arity: Fp::from(call.predicate.arity as u64),
            argument_roots,
        })
    }

    pub fn hash(&self) -> Fp {
        let mut inputs = [Fp::ZERO; GOAL_HASH_INPUTS];
        inputs[0] = Fp::from(GOAL_DOMAIN_TAG);
        inputs[1] = Fp::from(TYPED_TERM_HASH_SCHEMA_VERSION as u64);
        inputs[2] = self.name;
        inputs[3] = self.arity;
        inputs[4..].copy_from_slice(&self.argument_roots);
        PoseidonHash::<Fp, P128Pow5T3, ConstantLength<GOAL_HASH_INPUTS>, 3, 2>::init().hash(inputs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedLocalStepFp {
    pub goal: TypedGoalFp,
    pub children: Vec<TypedGoalFp>,
    /// Canonical parser-derived parent goal used to build authenticated term
    /// openings inside the circuit.
    pub ground_goal: GroundCall,
    /// Canonical parser-derived semantic children.  The input-only `[true]`
    /// marker is deliberately absent.
    pub ground_children: Vec<GroundCall>,
}

impl Default for TypedLocalStepFp {
    fn default() -> Self {
        let ground_goal = GroundCall::default();
        let goal = TypedGoalFp::from_ground_call(&ground_goal)
            .expect("the fixed key-generation goal must be a valid typed goal");
        Self {
            goal,
            children: Vec::new(),
            ground_goal,
            ground_children: Vec::new(),
        }
    }
}

impl TypedLocalStepFp {
    pub fn from_semantic_step(
        step: &LocalStepWitness,
    ) -> Result<Self, crate::typed_term::TypedTermError> {
        let goal = TypedGoalFp::from_ground_call(&step.goal)?;
        let children = match &step.transition {
            StepTransition::Rule { children } => children
                .iter()
                .map(|child| TypedGoalFp::from_ground_call(&child.goal))
                .collect::<Result<Vec<_>, _>>()?,
            StepTransition::ExternalFact | StepTransition::Builtin { .. } => Vec::new(),
        };
        let ground_children = match &step.transition {
            StepTransition::Rule { children } => {
                children.iter().map(|child| child.goal.clone()).collect()
            }
            StepTransition::ExternalFact | StepTransition::Builtin { .. } => Vec::new(),
        };
        Ok(Self {
            goal,
            children,
            ground_goal: step.goal.clone(),
            ground_children,
        })
    }
}

impl Default for GroundCall {
    fn default() -> Self {
        Self {
            predicate: PredicateIndicator {
                // A valid canonical zero-arity goal is needed during key
                // generation.  It is only a shape witness and is never a
                // production statement.
                name: "$batchman_shape".to_owned(),
                arity: 0,
            },
            args: Vec::new(),
        }
    }
}
