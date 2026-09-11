//! Circuit implementation of the canonical typed-term commitment protocol.
//!
//! The chip intentionally has no dependency on the legacy `TermFp` matrix. It
//! hashes one fixed-width node or goal preimage and verifies authenticated term
//! paths by re-hashing the target and every ancestor. A later integration layer
//! can connect the returned cells to rule-selection and projection cells.

use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};
use halo2curves::ff::Field;

use crate::{
    chips::fact_check::poseidon_hash::{PoseidonHashChip, PoseidonHashConfig},
    typed_term::{
        GOAL_DOMAIN_TAG, GOAL_HASH_INPUTS, MAX_TYPED_COMPOUND_ARITY, TERM_NODE_DOMAIN_TAG,
        TERM_NODE_HASH_INPUTS, TYPED_TERM_HASH_SCHEMA_VERSION, TermPath, TermPathStep, TermTag,
        TypedNodePreimage, TypedTermError, TypedTermOpening, hash_ground_term, symbol_to_field,
    },
    witness_ir::GroundCall,
};

const NODE_FIELD_COUNT: usize = 7;
const NODE_KIND_COUNT: usize = 5;
const COMPOUND_ARITY_COUNT: usize = MAX_TYPED_COMPOUND_ARITY;
const NODE_REGION_ROWS: usize = NODE_FIELD_COUNT + NODE_KIND_COUNT + COMPOUND_ARITY_COUNT;

const GOAL_FIELD_COUNT: usize = 6;
const GOAL_ARITY_COUNT: usize = MAX_TYPED_COMPOUND_ARITY + 1;
const GOAL_REGION_ROWS: usize = GOAL_FIELD_COUNT + GOAL_ARITY_COUNT;

#[derive(Clone, Debug)]
pub struct TypedTermConfig {
    pub poseidon: PoseidonHashConfig,
    pub value: Column<Advice>,
    q_node: Selector,
    q_goal: Selector,
    q_gated_root: Selector,
}

#[derive(Clone, Debug)]
pub struct TypedTermChip {
    config: TypedTermConfig,
}

#[derive(Clone, Debug)]
pub struct AssignedTypedNode {
    pub tag: AssignedCell<Fp, Fp>,
    pub payload: AssignedCell<Fp, Fp>,
    pub arity: AssignedCell<Fp, Fp>,
    pub children: [AssignedCell<Fp, Fp>; MAX_TYPED_COMPOUND_ARITY],
    pub kind_bits: [AssignedCell<Fp, Fp>; NODE_KIND_COUNT],
    pub compound_arity_bits: [AssignedCell<Fp, Fp>; COMPOUND_ARITY_COUNT],
}

#[derive(Clone, Debug)]
pub struct AssignedHashedNode {
    pub preimage: AssignedTypedNode,
    pub digest: AssignedCell<Fp, Fp>,
}

#[derive(Clone, Debug)]
pub struct TypedGoalPreimage {
    pub predicate: Fp,
    pub arity: u8,
    pub arguments: [Fp; MAX_TYPED_COMPOUND_ARITY],
}

impl TypedGoalPreimage {
    pub fn from_ground_goal(goal: &GroundCall) -> Result<Self, TypedTermError> {
        goal.validate()
            .map_err(|error| TypedTermError::new(error.to_string()))?;
        if goal.args.len() > MAX_TYPED_COMPOUND_ARITY {
            return Err(TypedTermError::new(format!(
                "goal {}/{} exceeds typed-term maximum arity {MAX_TYPED_COMPOUND_ARITY}",
                goal.predicate.name, goal.predicate.arity
            )));
        }

        let mut arguments = [Fp::ZERO; MAX_TYPED_COMPOUND_ARITY];
        for (index, argument) in goal.args.iter().enumerate() {
            arguments[index] = hash_ground_term(argument)?;
        }
        Ok(Self {
            predicate: symbol_to_field(&goal.predicate.name),
            arity: goal.predicate.arity as u8,
            arguments,
        })
    }
}

#[derive(Clone, Debug)]
pub struct AssignedTypedGoal {
    pub predicate: AssignedCell<Fp, Fp>,
    pub arity: AssignedCell<Fp, Fp>,
    pub arguments: [AssignedCell<Fp, Fp>; MAX_TYPED_COMPOUND_ARITY],
    pub arity_bits: [AssignedCell<Fp, Fp>; GOAL_ARITY_COUNT],
    pub digest: AssignedCell<Fp, Fp>,
}

#[derive(Clone, Debug)]
pub struct AssignedTypedOpening {
    pub claimed_root: AssignedCell<Fp, Fp>,
    pub computed_root: AssignedCell<Fp, Fp>,
    pub target: AssignedHashedNode,
}

impl TypedTermChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> TypedTermConfig {
        let poseidon = PoseidonHashChip::configure(meta);
        Self::configure_with_poseidon(meta, poseidon)
    }

    /// Configures typed-term canonicality around an already configured
    /// Poseidon gadget. The main circuit uses this form so typed goals,
    /// commitments and fact hashes share one permutation configuration rather
    /// than adding a duplicate set of advice/fixed columns.
    pub fn configure_with_poseidon(
        meta: &mut ConstraintSystem<Fp>,
        poseidon: PoseidonHashConfig,
    ) -> TypedTermConfig {
        let value = meta.advice_column();
        meta.enable_equality(value);
        let q_node = meta.selector();
        let q_goal = meta.selector();
        let q_gated_root = meta.selector();

        meta.create_gate("canonical typed-term node", |meta| {
            let q = meta.query_selector(q_node);
            let mut at = |rotation| meta.query_advice(value, Rotation(rotation));

            let tag = at(0);
            let payload = at(1);
            let arity = at(2);
            let children = [at(3), at(4), at(5), at(6)];
            let kinds = [at(7), at(8), at(9), at(10), at(11)];
            let arities = [at(12), at(13), at(14), at(15)];
            let one = Expression::Constant(Fp::ONE);

            let mut constraints = Vec::new();
            for bit in kinds.iter().chain(arities.iter()) {
                constraints.push(bit.clone() * (bit.clone() - one.clone()));
            }

            let kind_sum = kinds
                .iter()
                .cloned()
                .fold(Expression::Constant(Fp::ZERO), |sum, bit| sum + bit);
            constraints.push(kind_sum - one.clone());

            let expected_tag = kinds.iter().enumerate().fold(
                Expression::Constant(Fp::ZERO),
                |sum, (index, bit)| {
                    sum + Expression::Constant(Fp::from((index + 1) as u64)) * bit.clone()
                },
            );
            constraints.push(tag - expected_tag);

            let compound_arity_sum = arities
                .iter()
                .cloned()
                .fold(Expression::Constant(Fp::ZERO), |sum, bit| sum + bit);
            constraints.push(compound_arity_sum - kinds[4].clone());

            let expected_arity = arities.iter().enumerate().fold(
                Expression::Constant(Fp::from(2)) * kinds[3].clone(),
                |sum, (index, bit)| {
                    sum + Expression::Constant(Fp::from((index + 1) as u64)) * bit.clone()
                },
            );
            constraints.push(arity - expected_arity);

            constraints.push((kinds[2].clone() + kinds[3].clone()) * payload);

            let scalar_or_nil = kinds[0].clone() + kinds[1].clone() + kinds[2].clone();
            for child in &children {
                constraints.push(scalar_or_nil.clone() * child.clone());
            }
            constraints.push(kinds[3].clone() * children[2].clone());
            constraints.push(kinds[3].clone() * children[3].clone());

            constraints.push(arities[0].clone() * children[1].clone());
            constraints.push((arities[0].clone() + arities[1].clone()) * children[2].clone());
            constraints.push(
                (arities[0].clone() + arities[1].clone() + arities[2].clone())
                    * children[3].clone(),
            );

            constraints
                .into_iter()
                .map(|constraint| q.clone() * constraint)
                .collect::<Vec<_>>()
        });

        meta.create_gate("canonical typed goal", |meta| {
            let q = meta.query_selector(q_goal);
            let mut at = |rotation| meta.query_advice(value, Rotation(rotation));

            let arity = at(1);
            let arguments = [at(2), at(3), at(4), at(5)];
            let arity_bits = [at(6), at(7), at(8), at(9), at(10)];
            let one = Expression::Constant(Fp::ONE);

            let mut constraints = Vec::new();
            for bit in &arity_bits {
                constraints.push(bit.clone() * (bit.clone() - one.clone()));
            }
            constraints.push(
                arity_bits
                    .iter()
                    .cloned()
                    .fold(Expression::Constant(Fp::ZERO), |sum, bit| sum + bit)
                    - one,
            );
            constraints.push(
                arity
                    - arity_bits.iter().enumerate().fold(
                        Expression::Constant(Fp::ZERO),
                        |sum, (index, bit)| {
                            sum + Expression::Constant(Fp::from(index as u64)) * bit.clone()
                        },
                    ),
            );

            constraints.push(arity_bits[0].clone() * arguments[0].clone());
            constraints
                .push((arity_bits[0].clone() + arity_bits[1].clone()) * arguments[1].clone());
            constraints.push(
                (arity_bits[0].clone() + arity_bits[1].clone() + arity_bits[2].clone())
                    * arguments[2].clone(),
            );
            constraints.push(
                (arity_bits[0].clone()
                    + arity_bits[1].clone()
                    + arity_bits[2].clone()
                    + arity_bits[3].clone())
                    * arguments[3].clone(),
            );

            constraints
                .into_iter()
                .map(|constraint| q.clone() * constraint)
                .collect::<Vec<_>>()
        });

        // The caller supplies a candidate-selection flag that is constrained
        // by the rule-selection chip.  Zero deliberately disables only this
        // external root link; the opening's hashes, path and child links are
        // always verified by `verify_opening_detached`.
        meta.create_gate("candidate-gated typed opening root", |meta| {
            let q = meta.query_selector(q_gated_root);
            let flag = meta.query_advice(value, Rotation::cur());
            let computed_root = meta.query_advice(value, Rotation::next());
            let expected_root = meta.query_advice(value, Rotation(2));
            vec![q * flag * (computed_root - expected_root)]
        });

        TypedTermConfig {
            poseidon,
            value,
            q_node,
            q_goal,
            q_gated_root,
        }
    }

    pub fn construct(config: TypedTermConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &TypedTermConfig {
        &self.config
    }

    /// Assigns a fixed-shape canonical node preimage. Variant and arity are
    /// advice values constrained by one-hot bits, so the synthesis shape does
    /// not depend on the witness term kind.
    pub fn assign_node(
        &self,
        mut layouter: impl Layouter<Fp>,
        node: &TypedNodePreimage,
    ) -> Result<AssignedTypedNode, Error> {
        let (fields, kind_bits, arity_bits) = node_witness(node);
        let assigned = layouter.assign_region(
            || "assign canonical typed node",
            |mut region| {
                self.config.q_node.enable(&mut region, 0)?;
                let values = fields
                    .into_iter()
                    .chain(kind_bits)
                    .chain(arity_bits)
                    .collect::<Vec<_>>();
                debug_assert_eq!(values.len(), NODE_REGION_ROWS);
                values
                    .into_iter()
                    .enumerate()
                    .map(|(row, value)| {
                        region.assign_advice(
                            || format!("typed node value[{row}]"),
                            self.config.value,
                            row,
                            || Value::known(value),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()
            },
        )?;

        Ok(AssignedTypedNode {
            tag: assigned[0].clone(),
            payload: assigned[1].clone(),
            arity: assigned[2].clone(),
            children: assigned[3..7]
                .to_vec()
                .try_into()
                .map_err(|_| Error::Synthesis)?,
            kind_bits: assigned[7..12]
                .to_vec()
                .try_into()
                .map_err(|_| Error::Synthesis)?,
            compound_arity_bits: assigned[12..16]
                .to_vec()
                .try_into()
                .map_err(|_| Error::Synthesis)?,
        })
    }

    /// Hashes the exact assigned preimage cells using the native protocol's
    /// fixed nine-word Poseidon message.
    pub fn hash_node(
        &self,
        mut layouter: impl Layouter<Fp>,
        node: &AssignedTypedNode,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let constants = self.assign_protocol_constants(
            layouter.namespace(|| "typed node protocol constants"),
            TERM_NODE_DOMAIN_TAG,
        )?;
        let inputs = [
            constants[0].clone(),
            constants[1].clone(),
            node.tag.clone(),
            node.payload.clone(),
            node.arity.clone(),
            node.children[0].clone(),
            node.children[1].clone(),
            node.children[2].clone(),
            node.children[3].clone(),
        ];
        PoseidonHashChip::construct(self.config.poseidon.clone())
            .hash_fixed::<TERM_NODE_HASH_INPUTS>(
                layouter.namespace(|| "typed node Poseidon"),
                inputs,
            )
    }

    pub fn assign_and_hash_node(
        &self,
        mut layouter: impl Layouter<Fp>,
        node: &TypedNodePreimage,
    ) -> Result<AssignedHashedNode, Error> {
        let preimage = self.assign_node(layouter.namespace(|| "typed node preimage"), node)?;
        let digest = self.hash_node(layouter.namespace(|| "typed node hash"), &preimage)?;
        Ok(AssignedHashedNode { preimage, digest })
    }

    pub fn constrain_node_hash(
        &self,
        mut layouter: impl Layouter<Fp>,
        node: &TypedNodePreimage,
        expected: &AssignedCell<Fp, Fp>,
    ) -> Result<AssignedHashedNode, Error> {
        let assigned =
            self.assign_and_hash_node(layouter.namespace(|| "checked typed node"), node)?;
        constrain_equal(
            layouter.namespace(|| "bind typed node hash"),
            &assigned.digest,
            expected,
        )?;
        Ok(assigned)
    }

    pub fn assign_goal(
        &self,
        mut layouter: impl Layouter<Fp>,
        goal: &TypedGoalPreimage,
    ) -> Result<AssignedTypedGoal, Error> {
        let arity = usize::from(goal.arity);
        let mut arity_bits = [Fp::ZERO; GOAL_ARITY_COUNT];
        if arity < GOAL_ARITY_COUNT {
            arity_bits[arity] = Fp::ONE;
        }

        let assigned = layouter.assign_region(
            || "assign canonical typed goal",
            |mut region| {
                self.config.q_goal.enable(&mut region, 0)?;
                let values = std::iter::once(goal.predicate)
                    .chain(std::iter::once(Fp::from(arity as u64)))
                    .chain(goal.arguments)
                    .chain(arity_bits)
                    .collect::<Vec<_>>();
                debug_assert_eq!(values.len(), GOAL_REGION_ROWS);
                values
                    .into_iter()
                    .enumerate()
                    .map(|(row, value)| {
                        region.assign_advice(
                            || format!("typed goal value[{row}]"),
                            self.config.value,
                            row,
                            || Value::known(value),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()
            },
        )?;

        let predicate = assigned[0].clone();
        let arity = assigned[1].clone();
        let arguments: [AssignedCell<Fp, Fp>; MAX_TYPED_COMPOUND_ARITY] = assigned[2..6]
            .to_vec()
            .try_into()
            .map_err(|_| Error::Synthesis)?;
        let arity_bits = assigned[6..11]
            .to_vec()
            .try_into()
            .map_err(|_| Error::Synthesis)?;

        let constants = self.assign_protocol_constants(
            layouter.namespace(|| "typed goal protocol constants"),
            GOAL_DOMAIN_TAG,
        )?;
        let inputs = [
            constants[0].clone(),
            constants[1].clone(),
            predicate.clone(),
            arity.clone(),
            arguments[0].clone(),
            arguments[1].clone(),
            arguments[2].clone(),
            arguments[3].clone(),
        ];
        let digest = PoseidonHashChip::construct(self.config.poseidon.clone())
            .hash_fixed::<GOAL_HASH_INPUTS>(layouter.namespace(|| "typed goal Poseidon"), inputs)?;

        Ok(AssignedTypedGoal {
            predicate,
            arity,
            arguments,
            arity_bits,
            digest,
        })
    }

    pub fn assign_ground_goal(
        &self,
        layouter: impl Layouter<Fp>,
        goal: &GroundCall,
    ) -> Result<AssignedTypedGoal, Error> {
        let preimage = TypedGoalPreimage::from_ground_goal(goal).map_err(|_| Error::Synthesis)?;
        self.assign_goal(layouter, &preimage)
    }

    /// Algebraically verifies a native root-to-target opening against a
    /// circuit-fixed expected path without connecting it to an external root.
    ///
    /// The expected path controls the synthesis shape. Both path copies carried
    /// by the opening are assigned as advice and constrained to those constants.
    /// Each ancestor is hashed once, the selected child is copy-constrained to
    /// the previously computed digest, and the final digest is constrained to
    /// the root claimed inside the opening. Consequently an unselected
    /// candidate may use a deterministic dummy opening, but it may not use a
    /// malformed opening.
    pub fn verify_opening_detached(
        &self,
        mut layouter: impl Layouter<Fp>,
        opening: &TypedTermOpening,
        expected_path: &TermPath,
    ) -> Result<AssignedTypedOpening, Error> {
        if opening.path.steps.len() != expected_path.steps.len()
            || opening.ancestors.len() != expected_path.steps.len()
            || expected_path.steps.iter().any(invalid_path_step)
        {
            return Err(Error::Synthesis);
        }

        let claimed_root = layouter.assign_region(
            || "assign typed opening claimed root",
            |mut region| {
                region.assign_advice(
                    || "claimed opening root",
                    self.config.value,
                    0,
                    || Value::known(opening.root),
                )
            },
        )?;

        let target = self.assign_and_hash_node(
            layouter.namespace(|| "typed opening target"),
            &opening.target,
        )?;
        let mut current = target.digest.clone();

        for level in (0..expected_path.steps.len()).rev() {
            let expected_step = &expected_path.steps[level];
            let ancestor = &opening.ancestors[level];
            self.bind_path_step(
                layouter.namespace(|| format!("typed opening path step {level}")),
                &opening.path.steps[level],
                &ancestor.step,
                expected_step,
            )?;

            let parent = self.assign_and_hash_node(
                layouter.namespace(|| format!("typed opening ancestor {level}")),
                &ancestor.parent,
            )?;
            let selected_child = self.constrain_parent_shape_for_step(
                layouter.namespace(|| format!("typed opening ancestor shape {level}")),
                &parent.preimage,
                expected_step,
            )?;
            constrain_equal(
                layouter.namespace(|| format!("typed opening child link {level}")),
                &selected_child,
                &current,
            )?;
            current = parent.digest;
        }

        constrain_equal(
            layouter.namespace(|| "typed opening computed root"),
            &current,
            &claimed_root,
        )?;

        Ok(AssignedTypedOpening {
            claimed_root,
            computed_root: current,
            target,
        })
    }

    /// Connects an already verified opening to one expected goal-argument root
    /// exactly when `candidate_flag` is non-zero.
    ///
    /// In production `candidate_flag` must be one of the boolean, at-most-one
    /// cells returned by the rule-selection chip. This method intentionally
    /// does not duplicate that booleanity constraint. It copy-constrains all
    /// three inputs before applying
    /// `candidate_flag * (computed_root - expected_root) = 0`.
    pub fn bind_opening_root_if(
        &self,
        mut layouter: impl Layouter<Fp>,
        candidate_flag: &AssignedCell<Fp, Fp>,
        opening: &AssignedTypedOpening,
        expected_root: &AssignedCell<Fp, Fp>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "candidate-gated typed opening root",
            |mut region| {
                self.config.q_gated_root.enable(&mut region, 0)?;
                candidate_flag.copy_advice(
                    || "candidate flag",
                    &mut region,
                    self.config.value,
                    0,
                )?;
                opening.computed_root.copy_advice(
                    || "verified opening root",
                    &mut region,
                    self.config.value,
                    1,
                )?;
                expected_root.copy_advice(
                    || "expected goal argument root",
                    &mut region,
                    self.config.value,
                    2,
                )?;
                Ok(())
            },
        )
    }

    /// Backwards-compatible wrapper that verifies the opening and binds its
    /// root unconditionally to `expected_root`.
    pub fn verify_opening(
        &self,
        mut layouter: impl Layouter<Fp>,
        opening: &TypedTermOpening,
        expected_path: &TermPath,
        expected_root: &AssignedCell<Fp, Fp>,
    ) -> Result<AssignedTypedOpening, Error> {
        let assigned = self.verify_opening_detached(
            layouter.namespace(|| "verify detached typed opening"),
            opening,
            expected_path,
        )?;
        constrain_equal(
            layouter.namespace(|| "bind typed opening expected root"),
            &assigned.computed_root,
            expected_root,
        )?;
        Ok(assigned)
    }

    fn assign_protocol_constants(
        &self,
        mut layouter: impl Layouter<Fp>,
        domain: u64,
    ) -> Result<[AssignedCell<Fp, Fp>; 2], Error> {
        layouter.assign_region(
            || "typed hash protocol constants",
            |mut region| {
                let domain = region.assign_advice_from_constant(
                    || "typed hash domain",
                    self.config.poseidon.input_col,
                    0,
                    Fp::from(domain),
                )?;
                let version = region.assign_advice_from_constant(
                    || "typed hash version",
                    self.config.poseidon.input_col,
                    1,
                    Fp::from(TYPED_TERM_HASH_SCHEMA_VERSION as u64),
                )?;
                Ok([domain, version])
            },
        )
    }

    fn bind_path_step(
        &self,
        mut layouter: impl Layouter<Fp>,
        opening_step: &TermPathStep,
        ancestor_step: &TermPathStep,
        expected_step: &TermPathStep,
    ) -> Result<(), Error> {
        let expected = path_step_code(expected_step).ok_or(Error::Synthesis)?;
        let opening = path_step_code(opening_step).ok_or(Error::Synthesis)?;
        let ancestor = path_step_code(ancestor_step).ok_or(Error::Synthesis)?;
        layouter.assign_region(
            || "bind typed opening path code",
            |mut region| {
                let opening_cell = region.assign_advice(
                    || "opening path code",
                    self.config.value,
                    0,
                    || Value::known(opening),
                )?;
                let ancestor_cell = region.assign_advice(
                    || "ancestor path code",
                    self.config.value,
                    1,
                    || Value::known(ancestor),
                )?;
                region.constrain_constant(opening_cell.cell(), expected)?;
                region.constrain_constant(ancestor_cell.cell(), expected)
            },
        )
    }

    fn constrain_parent_shape_for_step(
        &self,
        mut layouter: impl Layouter<Fp>,
        parent: &AssignedTypedNode,
        step: &TermPathStep,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let (expected_tag, child_index, forbidden_arities) = match step {
            TermPathStep::Head => (TermTag::Cons, 0, 0),
            TermPathStep::Tail => (TermTag::Cons, 1, 0),
            TermPathStep::Argument { index } if usize::from(*index) < MAX_TYPED_COMPOUND_ARITY => {
                (TermTag::Compound, usize::from(*index), usize::from(*index))
            }
            TermPathStep::Argument { .. } => return Err(Error::Synthesis),
        };

        layouter.assign_region(
            || "constrain typed opening ancestor kind",
            |mut region| {
                region.constrain_constant(parent.tag.cell(), Fp::from(expected_tag.as_u64()))?;
                for bit in parent.compound_arity_bits.iter().take(forbidden_arities) {
                    region.constrain_constant(bit.cell(), Fp::ZERO)?;
                }
                Ok(())
            },
        )?;
        Ok(parent.children[child_index].clone())
    }
}

fn node_witness(
    node: &TypedNodePreimage,
) -> (
    [Fp; NODE_FIELD_COUNT],
    [Fp; NODE_KIND_COUNT],
    [Fp; COMPOUND_ARITY_COUNT],
) {
    let mut fields = [Fp::ZERO; NODE_FIELD_COUNT];
    let mut kind_bits = [Fp::ZERO; NODE_KIND_COUNT];
    let mut arity_bits = [Fp::ZERO; COMPOUND_ARITY_COUNT];

    fields[0] = Fp::from(node.tag().as_u64());
    kind_bits[node.tag().as_u64() as usize - 1] = Fp::ONE;
    match node {
        TypedNodePreimage::Atom { symbol } => fields[1] = *symbol,
        TypedNodePreimage::Integer { value } => fields[1] = Fp::from(*value),
        TypedNodePreimage::Nil => {}
        TypedNodePreimage::Cons { head, tail } => {
            fields[2] = Fp::from(2);
            fields[3] = *head;
            fields[4] = *tail;
        }
        TypedNodePreimage::Compound {
            functor,
            arity,
            children,
        } => {
            fields[1] = *functor;
            fields[2] = Fp::from(u64::from(*arity));
            fields[3..].copy_from_slice(children);
            let arity = usize::from(*arity);
            if (1..=MAX_TYPED_COMPOUND_ARITY).contains(&arity) {
                arity_bits[arity - 1] = Fp::ONE;
            }
        }
    }
    (fields, kind_bits, arity_bits)
}

fn path_step_code(step: &TermPathStep) -> Option<Fp> {
    match step {
        TermPathStep::Head => Some(Fp::from(1)),
        TermPathStep::Tail => Some(Fp::from(2)),
        TermPathStep::Argument { index } if usize::from(*index) < MAX_TYPED_COMPOUND_ARITY => {
            Some(Fp::from(3 + u64::from(*index)))
        }
        TermPathStep::Argument { .. } => None,
    }
}

fn invalid_path_step(step: &TermPathStep) -> bool {
    path_step_code(step).is_none()
}

fn constrain_equal(
    mut layouter: impl Layouter<Fp>,
    left: &AssignedCell<Fp, Fp>,
    right: &AssignedCell<Fp, Fp>,
) -> Result<(), Error> {
    layouter.assign_region(
        || "copy-constrain typed term cells",
        |mut region| region.constrain_equal(left.cell(), right.cell()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        policy_ir::PredicateIndicator,
        typed_term::{build_dummy_term_opening, build_term_opening, hash_ground_goal},
        witness_ir::GroundTerm,
    };
    use halo2_proofs::{
        circuit::SimpleFloorPlanner,
        dev::MockProver,
        plonk::{Circuit, Column, Instance},
    };

    const TEST_K: u32 = 12;

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

    fn goal() -> GroundCall {
        GroundCall {
            predicate: PredicateIndicator {
                name: "p".to_owned(),
                arity: 2,
            },
            args: vec![atom("1"), integer(1)],
        }
    }

    #[derive(Clone)]
    struct HashCircuit {
        node: TypedNodePreimage,
        goal: GroundCall,
    }

    #[derive(Clone, Debug)]
    struct HashConfig {
        typed: TypedTermConfig,
        instance: Column<Instance>,
    }

    impl Circuit<Fp> for HashCircuit {
        type Config = HashConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            self.clone()
        }

        fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
            let typed = TypedTermChip::configure(meta);
            let instance = meta.instance_column();
            meta.enable_equality(instance);
            HashConfig { typed, instance }
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<Fp>,
        ) -> Result<(), Error> {
            let chip = TypedTermChip::construct(config.typed);
            let node = chip
                .assign_and_hash_node(layouter.namespace(|| "test canonical node"), &self.node)?;
            layouter.constrain_instance(node.digest.cell(), config.instance, 0)?;

            let goal =
                chip.assign_ground_goal(layouter.namespace(|| "test canonical goal"), &self.goal)?;
            layouter.constrain_instance(goal.digest.cell(), config.instance, 1)
        }
    }

    #[derive(Clone)]
    struct OpeningCircuit {
        opening: TypedTermOpening,
        expected_path: TermPath,
    }

    #[derive(Clone, Debug)]
    struct OpeningConfig {
        typed: TypedTermConfig,
        instance: Column<Instance>,
    }

    impl Circuit<Fp> for OpeningCircuit {
        type Config = OpeningConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            self.clone()
        }

        fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
            let typed = TypedTermChip::configure(meta);
            let instance = meta.instance_column();
            meta.enable_equality(instance);
            OpeningConfig { typed, instance }
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<Fp>,
        ) -> Result<(), Error> {
            let chip = TypedTermChip::construct(config.typed.clone());
            let expected_root = layouter.assign_region(
                || "load expected typed root",
                |mut region| {
                    region.assign_advice_from_instance(
                        || "expected typed root",
                        config.instance,
                        0,
                        config.typed.value,
                        0,
                    )
                },
            )?;
            let verified = chip.verify_opening(
                layouter.namespace(|| "test typed opening"),
                &self.opening,
                &self.expected_path,
                &expected_root,
            )?;
            layouter.constrain_instance(verified.target.digest.cell(), config.instance, 1)
        }
    }

    #[derive(Clone)]
    struct GatedOpeningCircuit {
        opening: TypedTermOpening,
        expected_path: TermPath,
    }

    impl Circuit<Fp> for GatedOpeningCircuit {
        type Config = OpeningConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            self.clone()
        }

        fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
            let typed = TypedTermChip::configure(meta);
            let instance = meta.instance_column();
            meta.enable_equality(instance);
            OpeningConfig { typed, instance }
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<Fp>,
        ) -> Result<(), Error> {
            let chip = TypedTermChip::construct(config.typed.clone());
            let (expected_root, candidate_flag) = layouter.assign_region(
                || "load gated opening public inputs",
                |mut region| {
                    let expected_root = region.assign_advice_from_instance(
                        || "expected typed root",
                        config.instance,
                        0,
                        config.typed.value,
                        0,
                    )?;
                    let candidate_flag = region.assign_advice_from_instance(
                        || "candidate flag",
                        config.instance,
                        1,
                        config.typed.value,
                        1,
                    )?;
                    Ok((expected_root, candidate_flag))
                },
            )?;
            let verified = chip.verify_opening_detached(
                layouter.namespace(|| "test detached typed opening"),
                &self.opening,
                &self.expected_path,
            )?;
            chip.bind_opening_root_if(
                layouter.namespace(|| "test candidate-gated typed root"),
                &candidate_flag,
                &verified,
                &expected_root,
            )
        }
    }

    fn opening_fixture() -> (TypedTermOpening, TermPath) {
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
        (build_term_opening(&term, path.clone()).unwrap(), path)
    }

    #[test]
    fn native_and_chip_node_and_goal_hashes_match() {
        let node_term = compound("f", vec![atom("a"), integer(7)]);
        let node = TypedNodePreimage::from_ground_term(&node_term).unwrap();
        let goal = goal();
        let instances = vec![node.hash().unwrap(), hash_ground_goal(&goal).unwrap()];
        let circuit = HashCircuit { node, goal };

        MockProver::run(TEST_K, &circuit, vec![instances])
            .unwrap()
            .assert_satisfied();
    }

    #[test]
    fn opening_chip_accepts_native_opening_and_rejects_tampering() {
        let (opening, expected_path) = opening_fixture();
        let target = opening.target.hash().unwrap();
        let instances = vec![opening.root, target];

        MockProver::run(
            TEST_K,
            &OpeningCircuit {
                opening: opening.clone(),
                expected_path: expected_path.clone(),
            },
            vec![instances.clone()],
        )
        .unwrap()
        .assert_satisfied();

        let mut tampered_target = opening.clone();
        tampered_target.target = TypedNodePreimage::from_ground_term(&atom("c")).unwrap();

        let mut tampered_path = opening.clone();
        tampered_path.path.steps[2] = TermPathStep::Tail;

        let mut tampered_sibling = opening.clone();
        let TypedNodePreimage::Cons { tail, .. } = &mut tampered_sibling.ancestors[2].parent else {
            panic!("the final opening ancestor must be a cons node");
        };
        *tail += Fp::ONE;

        let mut tampered_root = opening;
        tampered_root.root += Fp::ONE;

        for bad_opening in [
            tampered_target,
            tampered_path,
            tampered_sibling,
            tampered_root,
        ] {
            let prover = MockProver::run(
                TEST_K,
                &OpeningCircuit {
                    opening: bad_opening,
                    expected_path: expected_path.clone(),
                },
                vec![instances.clone()],
            )
            .unwrap();
            assert!(prover.verify().is_err());
        }
    }

    #[test]
    fn candidate_gate_disables_only_the_external_root_link() {
        let (honest, expected_path) = opening_fixture();
        let dummy = build_dummy_term_opening(&expected_path).unwrap();
        assert_ne!(dummy.root, honest.root);

        // An unselected candidate may carry its deterministic, internally
        // valid filler even though that filler is unrelated to the goal root.
        MockProver::run(
            TEST_K,
            &GatedOpeningCircuit {
                opening: dummy.clone(),
                expected_path: expected_path.clone(),
            },
            vec![vec![honest.root, Fp::ZERO]],
        )
        .unwrap()
        .assert_satisfied();

        // Selecting the same candidate activates the goal-root link.
        let selected_dummy = MockProver::run(
            TEST_K,
            &GatedOpeningCircuit {
                opening: dummy.clone(),
                expected_path: expected_path.clone(),
            },
            vec![vec![honest.root, Fp::ONE]],
        )
        .unwrap();
        assert!(selected_dummy.verify().is_err());

        MockProver::run(
            TEST_K,
            &GatedOpeningCircuit {
                opening: honest.clone(),
                expected_path: expected_path.clone(),
            },
            vec![vec![honest.root, Fp::ONE]],
        )
        .unwrap()
        .assert_satisfied();

        // Disabling the root link must not disable the opening's own hash
        // chain, declared root, path, or sibling authentication.
        let mut malformed = dummy;
        malformed.root += Fp::ONE;
        let disabled_malformed = MockProver::run(
            TEST_K,
            &GatedOpeningCircuit {
                opening: malformed,
                expected_path,
            },
            vec![vec![honest.root, Fp::ZERO]],
        )
        .unwrap();
        assert!(disabled_malformed.verify().is_err());
    }
}
