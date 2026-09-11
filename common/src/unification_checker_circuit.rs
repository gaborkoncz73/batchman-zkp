use crate::{
    chips::{
        fact_check::{
            external_fact_chip::{ExternalFactChip, ExternalFactConfig},
            poseidon_hash::{PoseidonHashChip, PoseidonHashConfig},
            sound_builtin_chip::{LegacyBuiltinCells, SoundBuiltinChip, SoundBuiltinConfig},
        },
        gated_constant_chip::GatedConstantConfig,
        resolution_chip::ResolutionConfig,
        typed_builtin_chip::{TypedBuiltinChip, TypedBuiltinConfig, TypedBuiltinLegacyCells},
        typed_rule_chip::{TypedRuleChip, TypedRuleConfig, TypedRuleInputs},
        typed_term_chip::{AssignedTypedGoal, TypedGoalPreimage, TypedTermChip, TypedTermConfig},
    },
    commitments::{
        CHILDREN_COMMITMENT_DOMAIN_TAG, CHILDREN_COMMITMENT_INPUTS,
        ORDERED_CHILDREN_COMMITMENT_INSTANCE_ROW, PARENT_GOAL_COMMITMENT_INSTANCE_ROW,
        POLICY_COMMITMENT_INSTANCE_ROW, RESOLUTION_TAG_INSTANCE_ROW, circuit_policy_commitment,
    },
    data::{
        ClauseTemplateFp, LocalStepKind, RuleTemplateFileFp, TermFp, TermSideFp, UnificationInputFp,
    },
    typed_policy::TypedPolicyPlan,
    typed_term::MAX_TYPED_COMPOUND_ARITY,
    utils_2::{
        common_helpers::{MAX_ARITY, MAX_CHILDREN, MAX_PRED_LIST},
        predicate_helpers::bind_proof_sig_pairs,
    },
};
use halo2_proofs::circuit::Value;
use halo2_proofs::{
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner},
    pasta::Fp,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance},
};
pub const PER_TERM: usize = MAX_ARITY * MAX_PRED_LIST;
pub const PER_NODE: usize = MAX_PRED_LIST * PER_TERM;
pub const MAX_NODES: usize = 1 + MAX_CHILDREN;
pub const MAX_DOT_DIM: usize = MAX_NODES * PER_NODE + 1;

// Circuit definition
#[derive(Debug, Clone)]
pub struct UnificationCircuit {
    pub rules: RuleTemplateFileFp,
    /// Type-preserving clause plan produced in the same compiler pass and in
    /// exactly the same candidate order as `rules`.
    pub typed_policy: TypedPolicyPlan,
    pub unif: UnificationInputFp,
}

#[derive(Clone, Debug)]
pub struct UnifConfig {
    pub proof_pairs: Column<Advice>,
    pub witness: Column<Advice>,
    pub poseidon_cfg: PoseidonHashConfig,
    pub typed_term_cfg: TypedTermConfig,
    pub typed_builtin_cfg: TypedBuiltinConfig,
    pub typed_rule_cfg: TypedRuleConfig,
    pub external_fact_cfg: ExternalFactConfig,
    pub builtin_cfg: SoundBuiltinConfig,
    pub resolution_cfg: ResolutionConfig,
    pub canonical_cfg: GatedConstantConfig,
    pub public_inputs: Column<Instance>,
}

impl Circuit<Fp> for UnificationCircuit {
    type Config = UnifConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            // The current workflow deliberately uses a policy-specific key.
            // Rules are therefore circuit parameters, not private witnesses.
            rules: self.rules.clone(),
            typed_policy: self.typed_policy.clone(),
            unif: UnificationInputFp::default(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let proof_pairs = meta.advice_column();
        meta.enable_equality(proof_pairs);
        let witness = meta.advice_column();
        meta.enable_equality(witness);
        let public_inputs = meta.instance_column();
        meta.enable_equality(public_inputs);
        let poseidon_cfg = PoseidonHashChip::configure(meta);
        let typed_term_cfg = TypedTermChip::configure_with_poseidon(meta, poseidon_cfg.clone());
        let typed_builtin_cfg = TypedBuiltinChip::configure(meta, typed_term_cfg.value);
        let typed_rule_cfg = TypedRuleChip::configure(meta);
        let external_fact_cfg = ExternalFactChip::configure(meta, public_inputs);
        let builtin_cfg = SoundBuiltinChip::configure(meta);
        let resolution_cfg = ResolutionConfig::configure(meta);
        let canonical_cfg = GatedConstantConfig::configure(meta);

        UnifConfig {
            proof_pairs,
            witness,
            poseidon_cfg,
            typed_term_cfg,
            typed_builtin_cfg,
            typed_rule_cfg,
            external_fact_cfg,
            builtin_cfg,
            resolution_cfg,
            canonical_cfg,
            public_inputs,
        }
    }

    fn synthesize(&self, cfg: Self::Config, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        let proof_pairs = bind_proof_sig_pairs(
            "Bind proof signatures (name,arity)",
            &mut layouter,
            cfg.proof_pairs,
            &self.unif.goal_name,     // goal TermFp
            &self.unif.subtree_goals, // subtree TermFp vec
        )?;

        // The temporary legacy vector remains the input of builtin and leaf
        // compatibility gadgets during this migration. Public parent/child identity
        // is assigned separately from the canonical typed ground calls below.
        let w_fp = build_witness_w_fp(&self.unif.goal_name, &self.unif.subtree_goals);
        let w_cells = assign_w_cells(&mut layouter, cfg.witness, &w_fp)?;

        let assigned_typed = assign_typed_local_step_commitments(
            &mut layouter,
            &cfg.typed_term_cfg,
            &cfg.poseidon_cfg,
            &self.unif.typed,
        )?;
        layouter.constrain_instance(
            assigned_typed.parent.digest.cell(),
            cfg.public_inputs,
            PARENT_GOAL_COMMITMENT_INSTANCE_ROW,
        )?;
        layouter.constrain_instance(
            assigned_typed.ordered_children_commitment.cell(),
            cfg.public_inputs,
            ORDERED_CHILDREN_COMMITMENT_INSTANCE_ROW,
        )?;

        // Policy is a public, policy-specific circuit parameter in the current
        // key-generation workflow.  Assigning from a constant binds this value to
        // the VK/PK fixed columns; it is not a prover-chosen advice value.
        let policy_commitment = layouter.assign_region(
            || "bind policy commitment",
            |mut region| {
                region.assign_advice_from_constant(
                    || "policy commitment",
                    cfg.poseidon_cfg.input_col,
                    0,
                    circuit_policy_commitment(&self.rules, &self.typed_policy),
                )
            },
        )?;
        layouter.constrain_instance(
            policy_commitment.cell(),
            cfg.public_inputs,
            POLICY_COMMITMENT_INSTANCE_ROW,
        )?;

        let (goal_name_cell, goal_name_arg_cells) =
            parent_cells_from_bound_witness(&proof_pairs, &w_cells)?;

        // Candidate selection and every authoritative rule check now operate
        // on the canonical typed statement. The native first-match result is
        // only an assignment hint: boolean flags, their at-most-one fold, all
        // signatures, paths, patterns and equalities are constrained below.
        let typed_rule_result = TypedRuleChip::construct(cfg.typed_rule_cfg.clone())
            .select_and_assign(
                layouter.namespace(|| "canonical typed selected policy clause"),
                &TypedTermChip::construct(cfg.typed_term_cfg.clone()),
                &self.rules,
                &self.typed_policy,
                TypedRuleInputs {
                    parent: &assigned_typed.parent,
                    children: &assigned_typed.children,
                    semantic_child_count: &assigned_typed.semantic_child_count,
                    ground_parent: &self.unif.typed.ground_goal,
                    ground_children: &self.unif.typed.ground_children,
                },
                matches!(self.unif.resolution, LocalStepKind::Rule),
            )?;

        let goal_name_salt_cell = layouter.assign_region(
            || "assign goal_name_salt",
            |mut region| {
                region.assign_advice(
                    || "goal salt",
                    cfg.witness,
                    0,
                    || {
                        Value::known(
                            self.unif
                                .goal_name
                                .first()
                                .map(|term| term.fact_hashes)
                                .unwrap_or(Fp::zero()),
                        )
                    },
                )
            },
        )?;

        // Fact authorization is over the canonical typed goal digest, not the
        // context-dependent legacy matrix projection.
        let fact_hash = PoseidonHashChip::construct(cfg.poseidon_cfg.clone()).hash_typed_fact(
            layouter.namespace(|| "Poseidon(typed goal digest, fact salt)"),
            &assigned_typed.parent.digest,
            &goal_name_salt_cell,
        )?;
        let external_branch = ExternalFactChip::construct(cfg.external_fact_cfg.clone()).assign(
            layouter.namespace(|| "external fact membership"),
            &fact_hash,
            Value::known(
                if matches!(self.unif.resolution, LocalStepKind::ExternalFact) {
                    Fp::one()
                } else {
                    Fp::zero()
                },
            ),
        )?;

        let goal_name_arity_cells = proof_pairs[0]
            .iter()
            .map(|(_name, arity)| arity.clone())
            .collect::<Vec<_>>();
        let builtin_branch = SoundBuiltinChip::construct(cfg.builtin_cfg.clone()).assign(
            layouter.namespace(|| "sound builtin semantics"),
            LegacyBuiltinCells {
                names: &goal_name_cell,
                arities: &goal_name_arity_cells,
                args: &goal_name_arg_cells,
            },
            Value::known(
                if matches!(self.unif.resolution, LocalStepKind::Builtin(_)) {
                    Fp::one()
                } else {
                    Fp::zero()
                },
            ),
        )?;

        // The u32 evaluator above remains the arithmetic core. This bridge
        // binds every authoritative scalar and opcode to the same canonical
        // typed goal whose digest is exposed as the parent commitment.
        TypedBuiltinChip::construct(cfg.typed_builtin_cfg.clone()).assign(
            layouter.namespace(|| "canonical typed builtin semantics"),
            &TypedTermChip::construct(cfg.typed_term_cfg.clone()),
            &assigned_typed.parent,
            &self.unif.typed.ground_goal,
            TypedBuiltinLegacyCells {
                names: &goal_name_cell,
                arities: &goal_name_arity_cells,
                args: &goal_name_arg_cells,
            },
            &builtin_branch,
        )?;

        let (resolution_tag, leaf_branch) = cfg.resolution_cfg.assign(
            layouter.namespace(|| "bind semantic resolution"),
            &typed_rule_result.rule_ok,
            &external_branch,
            &builtin_branch,
            &assigned_typed.semantic_child_count,
            self.unif.resolution,
        )?;

        let mut leaf_checks = Vec::new();
        for node in 1..MAX_NODES {
            for predicate in 0..MAX_PRED_LIST {
                leaf_checks.push((proof_pairs[node][predicate].0.clone(), Fp::zero()));
                leaf_checks.push((proof_pairs[node][predicate].1.clone(), Fp::zero()));
                for argument in 0..MAX_ARITY {
                    for list_index in 0..MAX_PRED_LIST {
                        let expected = if node == 1 && predicate == 0 {
                            -Fp::one()
                        } else {
                            Fp::zero()
                        };
                        leaf_checks.push((
                            w_cells[w_index(node, predicate, argument, list_index)].clone(),
                            expected,
                        ));
                    }
                }
            }
        }
        cfg.canonical_cfg.assign(
            layouter.namespace(|| "canonical true leaf marker"),
            &leaf_branch,
            &leaf_checks,
        )?;

        let mut external_parent_padding = Vec::new();
        for predicate in 1..MAX_PRED_LIST {
            external_parent_padding.push((proof_pairs[0][predicate].0.clone(), Fp::zero()));
            external_parent_padding.push((proof_pairs[0][predicate].1.clone(), Fp::zero()));
            for argument in 0..MAX_ARITY {
                for list_index in 0..MAX_PRED_LIST {
                    external_parent_padding.push((
                        w_cells[w_index(0, predicate, argument, list_index)].clone(),
                        Fp::zero(),
                    ));
                }
            }
        }
        cfg.canonical_cfg.assign(
            layouter.namespace(|| "canonical external fact parent"),
            &external_branch,
            &external_parent_padding,
        )?;
        cfg.canonical_cfg.assign(
            layouter.namespace(|| "builtin has no fact salt"),
            &builtin_branch,
            &[(goal_name_salt_cell.clone(), Fp::zero())],
        )?;

        layouter.constrain_instance(
            resolution_tag.cell(),
            cfg.public_inputs,
            RESOLUTION_TAG_INSTANCE_ROW,
        )?;

        Ok(())
    }
}

type SignatureCell = (AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>);

/// Fixed-shape canonical typed statement assigned by the main circuit.
///
/// The next typed-rule integration consumes the goal preimage cells directly,
/// especially each argument-root cell. Missing children are the canonical
/// all-zero `TypedGoalFp::default()` sentinel.
#[derive(Clone, Debug)]
pub(crate) struct AssignedTypedLocalStep {
    pub parent: AssignedTypedGoal,
    #[allow(dead_code)] // consumed by the immediately following typed-rule integration
    pub children: [AssignedTypedGoal; MAX_CHILDREN],
    pub ordered_children_commitment: AssignedCell<Fp, Fp>,
    pub semantic_child_count: AssignedCell<Fp, Fp>,
}

fn assign_typed_local_step_commitments(
    layouter: &mut impl Layouter<Fp>,
    typed_term_cfg: &TypedTermConfig,
    poseidon_cfg: &PoseidonHashConfig,
    typed: &crate::circuit_witness::TypedLocalStepFp,
) -> Result<AssignedTypedLocalStep, Error> {
    if typed.ground_children.len() > MAX_CHILDREN
        || typed.children.len() != typed.ground_children.len()
    {
        return Err(Error::Synthesis);
    }

    let typed_chip = TypedTermChip::construct(typed_term_cfg.clone());
    let parent = typed_chip.assign_ground_goal(
        layouter.namespace(|| "canonical typed parent goal"),
        &typed.ground_goal,
    )?;

    let empty_goal = TypedGoalPreimage {
        predicate: Fp::zero(),
        arity: 0,
        arguments: [Fp::zero(); MAX_TYPED_COMPOUND_ARITY],
    };
    let mut assigned_children = Vec::with_capacity(MAX_CHILDREN);
    for child_index in 0..MAX_CHILDREN {
        let assigned = if let Some(child) = typed.ground_children.get(child_index) {
            typed_chip.assign_ground_goal(
                layouter.namespace(|| format!("canonical typed child goal {child_index}")),
                child,
            )?
        } else {
            typed_chip.assign_goal(
                layouter.namespace(|| format!("canonical empty child goal {child_index}")),
                &empty_goal,
            )?
        };
        assigned_children.push(assigned);
    }

    let (children_domain, semantic_child_count) = layouter.assign_region(
        || "typed ordered-children commitment prefix",
        |mut region| {
            let children_domain = region.assign_advice_from_constant(
                || "typed children domain",
                poseidon_cfg.input_col,
                0,
                Fp::from(CHILDREN_COMMITMENT_DOMAIN_TAG),
            )?;
            let semantic_child_count = region.assign_advice(
                || "semantic child count",
                poseidon_cfg.input_col,
                1,
                || Value::known(Fp::from(typed.ground_children.len() as u64)),
            )?;
            Ok((children_domain, semantic_child_count))
        },
    )?;

    let mut children_inputs = Vec::with_capacity(CHILDREN_COMMITMENT_INPUTS);
    children_inputs.push(children_domain);
    children_inputs.push(semantic_child_count.clone());
    children_inputs.extend(
        assigned_children
            .iter()
            .map(|assigned| assigned.digest.clone()),
    );
    let children_inputs: [AssignedCell<Fp, Fp>; CHILDREN_COMMITMENT_INPUTS] =
        children_inputs.try_into().map_err(|_| Error::Synthesis)?;
    let children_hash = PoseidonHashChip::construct(poseidon_cfg.clone()).hash_fixed(
        layouter.namespace(|| "ordered children commitment"),
        children_inputs,
    )?;

    Ok(AssignedTypedLocalStep {
        parent,
        children: assigned_children.try_into().map_err(|_| Error::Synthesis)?,
        ordered_children_commitment: children_hash,
        semantic_child_count,
    })
}

fn parent_cells_from_bound_witness(
    proof_pairs: &[Vec<SignatureCell>],
    w_cells: &[AssignedCell<Fp, Fp>],
) -> Result<
    (
        Vec<AssignedCell<Fp, Fp>>,
        Vec<Vec<Vec<AssignedCell<Fp, Fp>>>>,
    ),
    Error,
> {
    let Some(parent_signatures) = proof_pairs.first() else {
        return Err(Error::Synthesis);
    };
    if parent_signatures.len() != MAX_PRED_LIST || w_cells.len() != MAX_DOT_DIM {
        return Err(Error::Synthesis);
    }

    let names = parent_signatures
        .iter()
        .map(|(name, _arity)| name.clone())
        .collect();
    let mut arguments = Vec::with_capacity(MAX_PRED_LIST);
    for predicate in 0..MAX_PRED_LIST {
        let mut predicate_arguments = Vec::with_capacity(MAX_ARITY);
        for argument in 0..MAX_ARITY {
            let mut list_slots = Vec::with_capacity(MAX_PRED_LIST);
            for list_index in 0..MAX_PRED_LIST {
                list_slots.push(w_cells[w_index(0, predicate, argument, list_index)].clone());
            }
            predicate_arguments.push(list_slots);
        }
        arguments.push(predicate_arguments);
    }
    Ok((names, arguments))
}

#[inline]
fn w_index(node: usize, predicate: usize, argument: usize, list_index: usize) -> usize {
    node * PER_NODE + predicate * PER_TERM + argument * MAX_PRED_LIST + list_index
}

pub fn clause_equalities_4d_tuples_fp(
    clause: &ClauseTemplateFp,
) -> Vec<(Fp, Fp, Fp, Fp, Fp, Fp, Fp, Fp)> {
    clause
        .equalities
        .iter()
        .filter_map(|eq| match (&eq.left, &eq.right) {
            (TermSideFp::Ref(l), TermSideFp::Ref(r)) => Some((
                l.children_node_list, // left node index
                l.predicate,          // left predicate index
                l.arg,                // left argument row index
                l.list_index,         // left list index
                r.children_node_list, // right node index
                r.predicate,          // right predicate index
                r.arg,                // right argument row index
                r.list_index,         // right list index
            )),
            _ => None, // ha bármelyik Value => nincs egyenlet sor
        })
        .collect()
}

pub fn build_witness_w_fp(
    goal_terms: &[TermFp],         // len ≤ MAX_PRED_LIST
    subtree_terms: &[Vec<TermFp>], // len ≤ MAX_CHILDREN, mindegyik len ≤ MAX_PRED_LIST
) -> Vec<Fp> {
    let mut w = Vec::new(); // +1 homogén 1-nek a végére

    // node 0: goal
    for p in 0..MAX_PRED_LIST {
        let term_opt = goal_terms.get(p);
        let flat = term_opt
            .map(flatten_term_args)
            .unwrap_or_else(|| vec![Fp::zero(); PER_TERM]);

        w.extend(flat);
    }

    // node 1..MAX_CHILDREN: subtree
    for n in 0..MAX_CHILDREN {
        let row_terms = subtree_terms.get(n); // Option<&Vec<TermFp>>

        for p in 0..MAX_PRED_LIST {
            let term_opt = row_terms.and_then(|row| row.get(p));
            let flat = term_opt
                .map(flatten_term_args)
                .unwrap_or_else(|| vec![Fp::zero(); PER_TERM]);

            w.extend(flat);
        }
    }

    // homogén 1 a legvégén
    w.push(Fp::one());

    w
}

fn assign_w_cells(
    layouter: &mut impl Layouter<Fp>,
    column: Column<halo2_proofs::plonk::Advice>,
    w: &[Fp],
) -> Result<Vec<AssignedCell<Fp, Fp>>, Error> {
    layouter.assign_region(
        || "bind w vector",
        |mut region| {
            let mut out = Vec::with_capacity(w.len());
            for (i, val) in w.iter().enumerate() {
                let c =
                    region.assign_advice(|| format!("w[{i}]"), column, i, || Value::known(*val))?;
                out.push(c);
            }
            Ok(out)
        },
    )
}

pub fn flatten_term_args(t: &TermFp) -> Vec<Fp> {
    let mut flat = Vec::new();

    for arg_i in 0..MAX_ARITY {
        for list_i in 0..MAX_PRED_LIST {
            flat.push(
                t.args
                    .get(arg_i)
                    .and_then(|row| row.get(list_i))
                    .copied()
                    .unwrap_or(Fp::zero()), // padding safety
            );
        }
    }

    flat
}
