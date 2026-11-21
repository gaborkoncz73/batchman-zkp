//! Soundness regression harness for the current local-step circuit.
//!
//! Positive and negative local-step regressions. Every rejection below is an
//! active test: host-side witness calculations are not accepted as evidence
//! unless the corresponding relation is constrained in the circuit.

use common::{
    circuit_dialect::{BuiltinKind, CircuitLimits},
    circuit_witness::{TypedGoalFp, TypedLocalStepFp},
    commitments::{
        LOCAL_STEP_PUBLIC_INPUT_COUNT, append_local_step_public_inputs, local_step_commitments,
    },
    data::{ChildSig, LocalStepKind, RuleTemplateFileFp, TermFp, UnificationInputFp},
    policy_compiler::compile_policy_for_current_circuit,
    policy_ir::{Clause, PolicyFile, PredicateDef, PredicateIndicator, Term},
    typed_fact::typed_fact_hash,
    typed_policy::TypedPolicyPlan,
    unification_checker_circuit::UnificationCircuit,
    utils_2::{
        common_helpers::to_fp_value,
        common_helpers::{MAX_ARITY, MAX_FACTS_HASHES, MAX_PRED_LIST},
    },
    witness_ir::{GroundCall, GroundTerm},
};
use halo2_proofs::{dev::MockProver, pasta::Fp};

const TEST_K: u32 = 16;

#[derive(Clone)]
struct CircuitPolicy {
    rules: RuleTemplateFileFp,
    typed_policy: TypedPolicyPlan,
}

fn term(name: &str, args: &[&str]) -> TermFp {
    let padding = -Fp::one();
    let mut encoded_args = vec![vec![padding; MAX_PRED_LIST]; MAX_ARITY];

    for (arg_index, value) in args.iter().enumerate() {
        encoded_args[arg_index][0] = to_fp_value(value);
    }

    TermFp {
        name: to_fp_value(name),
        args: encoded_args,
        fact_hashes: Fp::zero(),
    }
}

fn term_with_rows(name: &str, args: &[&[&str]]) -> TermFp {
    let padding = -Fp::one();
    let mut encoded_args = vec![vec![padding; MAX_PRED_LIST]; MAX_ARITY];

    for (arg_index, values) in args.iter().enumerate() {
        for (list_index, value) in values.iter().enumerate() {
            encoded_args[arg_index][list_index] = to_fp_value(value);
        }
    }

    TermFp {
        name: to_fp_value(name),
        args: encoded_args,
        fact_hashes: Fp::zero(),
    }
}

fn ground_scalar(value: &str) -> GroundTerm {
    if let Some(atom) = value
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
    {
        GroundTerm::Atom {
            value: atom.to_owned(),
        }
    } else if let Ok(integer) = value.parse::<u64>() {
        if integer.to_string() == value {
            GroundTerm::Integer { value: integer }
        } else {
            GroundTerm::Atom {
                value: value.to_owned(),
            }
        }
    } else {
        GroundTerm::Atom {
            value: value.to_owned(),
        }
    }
}

fn ground_goal(name: &str, args: &[&str]) -> GroundCall {
    GroundCall {
        predicate: PredicateIndicator {
            name: name.to_owned(),
            arity: args.len(),
        },
        args: args.iter().map(|value| ground_scalar(value)).collect(),
    }
}

fn ground_call(name: &str, args: Vec<GroundTerm>) -> GroundCall {
    GroundCall {
        predicate: PredicateIndicator {
            name: name.to_owned(),
            arity: args.len(),
        },
        args,
    }
}

fn typed_local_step(parent: GroundCall, children: Vec<GroundCall>) -> TypedLocalStepFp {
    TypedLocalStepFp {
        goal: TypedGoalFp::from_ground_call(&parent).unwrap(),
        children: children
            .iter()
            .map(|child| TypedGoalFp::from_ground_call(child).unwrap())
            .collect(),
        ground_goal: parent,
        ground_children: children,
    }
}

fn compile_policy(policy: PolicyFile) -> CircuitPolicy {
    let limits: CircuitLimits =
        serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap();
    let (rules, typed_policy) = compile_policy_for_current_circuit(&policy, &limits)
        .unwrap()
        .into_circuit_parts();
    CircuitPolicy {
        rules,
        typed_policy,
    }
}

fn one_predicate_policy(
    head_args: Vec<Term>,
    variables: Vec<Option<String>>,
    body: Term,
) -> PolicyFile {
    let indicator = PredicateIndicator {
        name: "p".to_owned(),
        arity: head_args.len(),
    };
    PolicyFile {
        predicates: vec![PredicateDef {
            indicator: indicator.clone(),
            clauses: vec![Clause {
                variables,
                head: head_args,
                body,
            }],
        }],
    }
}

fn literal_policy_term(literal: Term) -> CircuitPolicy {
    compile_policy(one_predicate_policy(
        vec![literal.clone()],
        Vec::new(),
        Term::compound("q", vec![literal]),
    ))
}

fn literal_policy(literal: &str) -> CircuitPolicy {
    literal_policy_term(Term::atom(literal))
}

fn variable_policy(arity: usize, children: Vec<Vec<ChildSig>>) -> CircuitPolicy {
    let variables = (0..arity)
        .map(|id| Some(format!("X{id}")))
        .collect::<Vec<_>>();
    let head_args = (0..arity)
        .map(|id| Term::Variable { id: id as u32 })
        .collect();
    let mut goals = children.into_iter().map(|row| {
        assert_eq!(row.len(), 1, "this fixture helper expects one goal per row");
        let child = row.into_iter().next().unwrap();
        assert!(child.arity <= arity);
        Term::compound(
            child.name,
            (0..child.arity)
                .map(|id| Term::Variable { id: id as u32 })
                .collect(),
        )
    });
    let body = goals
        .next()
        .map(|first| goals.fold(first, |left, right| Term::compound(",", vec![left, right])))
        .unwrap_or_else(|| Term::atom("true"));

    compile_policy(one_predicate_policy(head_args, variables, body))
}

fn child(name: &str, arity: usize) -> ChildSig {
    ChildSig {
        name: name.to_owned(),
        arity,
    }
}

fn tuple_list_policy() -> CircuitPolicy {
    let variable = |id| Term::Variable { id };
    compile_policy(one_predicate_policy(
        vec![Term::List {
            items: vec![Term::compound(",", vec![variable(0), variable(1)])],
            tail: None,
        }],
        vec![Some("A".to_owned()), Some("B".to_owned())],
        Term::compound("q", vec![variable(0), variable(1)]),
    ))
}

fn unification(parent_arg: &str, child_arg: &str) -> UnificationInputFp {
    UnificationInputFp {
        goal_name: vec![term("p", &[parent_arg])],
        subtree_goals: vec![vec![term("q", &[child_arg])]],
        resolution: LocalStepKind::Rule,
        typed: typed_local_step(
            ground_goal("p", &[parent_arg]),
            vec![ground_goal("q", &[child_arg])],
        ),
    }
}

fn variable_unification(parent_args: &[&str], child_args: &[&str]) -> UnificationInputFp {
    UnificationInputFp {
        goal_name: vec![term("p", parent_args)],
        subtree_goals: vec![vec![term("q", child_args)]],
        resolution: LocalStepKind::Rule,
        typed: typed_local_step(
            ground_goal("p", parent_args),
            vec![ground_goal("q", child_args)],
        ),
    }
}

fn tuple_list_unification(parent_argument: GroundTerm) -> UnificationInputFp {
    UnificationInputFp {
        goal_name: vec![term_with_rows("p", &[&["a", "b", "[]"]])],
        subtree_goals: vec![vec![term("q", &["a", "b"])]],
        resolution: LocalStepKind::Rule,
        typed: typed_local_step(
            ground_call("p", vec![parent_argument]),
            vec![ground_goal("q", &["a", "b"])],
        ),
    }
}

fn atom(value: &str) -> GroundTerm {
    GroundTerm::Atom {
        value: value.to_owned(),
    }
}

fn tuple_list_term(functor: &str, args: Vec<GroundTerm>) -> GroundTerm {
    GroundTerm::Cons {
        head: Box::new(GroundTerm::Compound {
            functor: functor.to_owned(),
            args,
        }),
        tail: Box::new(GroundTerm::Nil),
    }
}

fn verify_current_circuit(
    policy: CircuitPolicy,
    unif: UnificationInputFp,
) -> Result<(), Vec<halo2_proofs::dev::VerifyFailure>> {
    verify_current_circuit_with_facts(policy, unif, vec![Fp::zero(); MAX_FACTS_HASHES])
}

fn verify_current_circuit_with_facts(
    policy: CircuitPolicy,
    unif: UnificationInputFp,
    public_fact_hashes: Vec<Fp>,
) -> Result<(), Vec<halo2_proofs::dev::VerifyFailure>> {
    let commitments = local_step_commitments(&policy.rules, &policy.typed_policy, &unif)
        .expect("the soundness fixture must have a bounded commitment shape");
    let public_inputs = append_local_step_public_inputs(&public_fact_hashes, &commitments);
    let circuit = UnificationCircuit {
        rules: policy.rules,
        typed_policy: policy.typed_policy,
        unif,
    };
    MockProver::run(TEST_K, &circuit, vec![public_inputs])
        .expect("the soundness fixture must fit in the configured test circuit")
        .verify()
}

fn fact_unification(argument: &str, salt: &str) -> UnificationInputFp {
    let mut fact = term("external", &[argument]);
    fact.fact_hashes = to_fp_value(salt);
    let true_marker = TermFp {
        name: Fp::zero(),
        args: vec![vec![-Fp::one(); MAX_PRED_LIST]; MAX_ARITY],
        fact_hashes: Fp::zero(),
    };

    let typed = typed_local_step(ground_goal("external", &[argument]), Vec::new());

    UnificationInputFp {
        goal_name: vec![fact],
        subtree_goals: vec![vec![true_marker]],
        resolution: LocalStepKind::ExternalFact,
        typed,
    }
}

fn fact_instances(argument: &str, salt: &str) -> Vec<Fp> {
    let mut instances = vec![Fp::zero(); MAX_FACTS_HASHES];
    let goal = ground_goal("external", &[argument]);
    instances[0] = typed_fact_hash(
        TypedGoalFp::from_ground_call(&goal).unwrap().hash(),
        to_fp_value(salt),
    );
    instances
}

fn builtin_term(name: &str, args: &[&str]) -> TermFp {
    term(name, args)
}

fn builtin_unification(terms: Vec<TermFp>, ground_goal: GroundCall) -> UnificationInputFp {
    let true_marker = TermFp {
        name: Fp::zero(),
        args: vec![vec![-Fp::one(); MAX_PRED_LIST]; MAX_ARITY],
        fact_hashes: Fp::zero(),
    };
    let builtin = match terms.first().map(|term| term.name) {
        Some(name) if name == to_fp_value("=") => BuiltinKind::Unify,
        Some(name) if name == to_fp_value("is") => BuiltinKind::Is,
        Some(name) if name == to_fp_value("<") => BuiltinKind::LessThan,
        Some(name) if name == to_fp_value("<=") => BuiltinKind::LessThanOrEqual,
        Some(name) if name == to_fp_value(">") => BuiltinKind::GreaterThan,
        Some(name) if name == to_fp_value(">=") => BuiltinKind::GreaterThanOrEqual,
        _ => BuiltinKind::Unify,
    };
    UnificationInputFp {
        goal_name: terms,
        subtree_goals: vec![vec![true_marker]],
        resolution: LocalStepKind::Builtin(builtin),
        typed: typed_local_step(ground_goal, Vec::new()),
    }
}

fn empty_policy() -> CircuitPolicy {
    CircuitPolicy {
        rules: RuleTemplateFileFp {
            predicates: Vec::new(),
        },
        typed_policy: TypedPolicyPlan {
            predicates: Vec::new(),
        },
    }
}

#[test]
fn honest_literal_rule_witness_satisfies_current_circuit() {
    assert!(verify_current_circuit(literal_policy("a"), unification("a", "a")).is_ok());
}

#[test]
fn honest_repeated_variable_witness_satisfies_current_circuit() {
    assert!(
        verify_current_circuit(
            variable_policy(1, vec![vec![child("q", 1)]]),
            variable_unification(&["a"], &["a"]),
        )
        .is_ok()
    );
}

#[test]
fn honest_prover_rejects_a_repeated_variable_mismatch() {
    assert!(
        verify_current_circuit(
            variable_policy(1, vec![vec![child("q", 1)]]),
            variable_unification(&["a"], &["b"]),
        )
        .is_err(),
        "the same Prolog variable cannot be instantiated as both a and b"
    );
}

#[test]
fn honest_prover_rejects_the_wrong_child_signature() {
    let rules = variable_policy(1, vec![vec![child("q", 1)]]);
    let unif = UnificationInputFp {
        goal_name: vec![term("p", &["a"])],
        subtree_goals: vec![vec![term("r", &["a"])]],
        resolution: LocalStepKind::Rule,
        typed: typed_local_step(ground_goal("p", &["a"]), vec![ground_goal("r", &["a"])]),
    };

    assert!(verify_current_circuit(rules, unif).is_err());
}

#[test]
fn honest_prover_enforces_child_count_and_order() {
    let rules = variable_policy(1, vec![vec![child("q", 1)], vec![child("r", 1)]]);
    let unif = |children: &[(&str, &str)]| UnificationInputFp {
        goal_name: vec![term("p", &["a"])],
        subtree_goals: children
            .iter()
            .map(|(name, argument)| vec![term(name, &[*argument])])
            .collect(),
        resolution: LocalStepKind::Rule,
        typed: typed_local_step(
            ground_goal("p", &["a"]),
            children
                .iter()
                .map(|(name, argument)| ground_goal(name, &[*argument]))
                .collect(),
        ),
    };

    assert!(verify_current_circuit(rules.clone(), unif(&[("q", "a"), ("r", "a")])).is_ok());
    assert!(verify_current_circuit(rules.clone(), unif(&[("r", "a"), ("q", "a")])).is_err());
    assert!(verify_current_circuit(rules.clone(), unif(&[("q", "a")])).is_err());
    assert!(verify_current_circuit(rules, unif(&[("q", "a"), ("r", "a"), ("r", "a")]),).is_err());
}

#[test]
fn local_proof_is_bound_to_policy_parent_and_ordered_children_commitments() {
    let policy = variable_policy(1, vec![vec![child("q", 1)], vec![child("r", 1)]]);
    let unif = UnificationInputFp {
        goal_name: vec![term("p", &["a"])],
        subtree_goals: vec![vec![term("q", &["a"])], vec![term("r", &["a"])]],
        resolution: LocalStepKind::Rule,
        typed: typed_local_step(
            ground_goal("p", &["a"]),
            vec![ground_goal("q", &["a"]), ground_goal("r", &["a"])],
        ),
    };
    let facts = vec![Fp::zero(); MAX_FACTS_HASHES];
    let commitments = local_step_commitments(&policy.rules, &policy.typed_policy, &unif).unwrap();
    let honest_inputs = append_local_step_public_inputs(&facts, &commitments);
    let circuit = UnificationCircuit {
        rules: policy.rules,
        typed_policy: policy.typed_policy,
        unif,
    };

    assert!(
        MockProver::run(TEST_K, &circuit, vec![honest_inputs.clone()])
            .unwrap()
            .verify()
            .is_ok()
    );
    for row in MAX_FACTS_HASHES..LOCAL_STEP_PUBLIC_INPUT_COUNT {
        let mut tampered = honest_inputs.clone();
        tampered[row] += Fp::one();
        assert!(
            MockProver::run(TEST_K, &circuit, vec![tampered])
                .unwrap()
                .verify()
                .is_err(),
            "tampering public commitment row {row} must be rejected"
        );
    }
}

#[test]
fn honest_fact_witness_requires_the_matching_public_hash_and_salt() {
    let empty_policy = empty_policy();
    assert!(
        verify_current_circuit_with_facts(
            empty_policy.clone(),
            fact_unification("a", "100"),
            fact_instances("a", "100"),
        )
        .is_ok()
    );
    assert!(
        verify_current_circuit_with_facts(
            empty_policy.clone(),
            fact_unification("a", "100"),
            fact_instances("b", "100"),
        )
        .is_err(),
        "a public hash for another fact must be rejected"
    );
    assert!(
        verify_current_circuit_with_facts(
            empty_policy,
            fact_unification("a", "101"),
            fact_instances("a", "100"),
        )
        .is_err(),
        "a witness using the wrong fact salt must be rejected"
    );
}

#[test]
fn typed_external_fact_rejects_atom_integer_aliasing() {
    assert!(
        verify_current_circuit_with_facts(
            empty_policy(),
            fact_unification("'1'", "100"),
            fact_instances("1", "100"),
        )
        .is_err(),
        "an authorization for integer 1 must not authorize the distinct Prolog atom '1'"
    );
}

#[test]
fn honest_builtin_witness_accepts_the_right_result_and_rejects_a_wrong_one() {
    let multiplication = |result: &str| {
        builtin_unification(
            vec![
                builtin_term("is", &[result, "2"]),
                builtin_term("*", &["5"]),
            ],
            ground_call(
                "is",
                vec![
                    GroundTerm::Integer {
                        value: result.parse().unwrap(),
                    },
                    GroundTerm::Compound {
                        functor: "*".to_owned(),
                        args: vec![
                            GroundTerm::Integer { value: 2 },
                            GroundTerm::Integer { value: 5 },
                        ],
                    },
                ],
            ),
        )
    };

    assert!(verify_current_circuit(empty_policy(), multiplication("10")).is_ok());
    assert!(
        verify_current_circuit(empty_policy(), multiplication("9")).is_err(),
        "9 is 2 * 5 must be rejected"
    );
}

#[test]
fn typed_builtin_equality_rejects_atom_integer_aliasing_in_legacy_cells() {
    // Both legacy slots deliberately contain the same field scalar, so the
    // old cell-wise equality succeeds. Canonical typed roots must still keep
    // the atom `1` distinct from the integer 1.
    let equality = builtin_unification(
        vec![builtin_term("=", &["1", "1"])],
        ground_call(
            "=",
            vec![
                GroundTerm::Atom {
                    value: "1".to_owned(),
                },
                GroundTerm::Integer { value: 1 },
            ],
        ),
    );
    assert!(
        verify_current_circuit(empty_policy(), equality).is_err(),
        "=/2 must compare canonical typed argument roots, not aliased legacy scalars"
    );
}

#[test]
fn typed_builtin_numeric_relation_rejects_a_non_integer_argument() {
    let comparison = builtin_unification(
        vec![builtin_term(">", &["1", "0"])],
        ground_call(
            ">",
            vec![
                GroundTerm::Atom {
                    value: "1".to_owned(),
                },
                GroundTerm::Integer { value: 0 },
            ],
        ),
    );
    assert!(
        verify_current_circuit(empty_policy(), comparison).is_err(),
        "numeric relations must open Integer nodes before using their u32 payloads"
    );
}

#[test]
fn typed_builtin_is_rejects_a_functor_mismatched_with_the_legacy_opcode() {
    let mismatched = builtin_unification(
        vec![builtin_term("is", &["10", "2"]), builtin_term("*", &["5"])],
        ground_call(
            "is",
            vec![
                GroundTerm::Integer { value: 10 },
                GroundTerm::Compound {
                    functor: "+".to_owned(),
                    args: vec![
                        GroundTerm::Integer { value: 2 },
                        GroundTerm::Integer { value: 5 },
                    ],
                },
            ],
        ),
    );
    assert!(
        verify_current_circuit(empty_policy(), mismatched).is_err(),
        "the constrained expression functor must equal the evaluated legacy opcode"
    );
}

#[test]
fn typed_builtin_is_rejects_commuted_expression_children() {
    // Multiplication has the same numeric result after swapping its children;
    // the typed expression commitment must nevertheless preserve their order.
    let reordered = builtin_unification(
        vec![builtin_term("is", &["10", "2"]), builtin_term("*", &["5"])],
        ground_call(
            "is",
            vec![
                GroundTerm::Integer { value: 10 },
                GroundTerm::Compound {
                    functor: "*".to_owned(),
                    args: vec![
                        GroundTerm::Integer { value: 5 },
                        GroundTerm::Integer { value: 2 },
                    ],
                },
            ],
        ),
    );
    assert!(
        verify_current_circuit(empty_policy(), reordered).is_err(),
        "the expression digest chain must bind left and right children in order"
    );
}

#[test]
fn rejects_a_goal_that_violates_rule_literals() {
    assert!(
        verify_current_circuit(literal_policy("a"), unification("b", "b")).is_err(),
        "p(b) :- q(b) must not satisfy the policy rule p(a) :- q(a)"
    );
}

#[test]
fn typed_rule_rejects_atom_integer_substitution_hidden_from_the_legacy_view() {
    assert_ne!(
        to_fp_value("'1'"),
        to_fp_value("1"),
        "the Prolog atom '1' and integer 1 must remain different terms"
    );

    let policy = literal_policy("1");
    assert!(
        verify_current_circuit(policy.clone(), unification("'1'", "'1'")).is_ok(),
        "the canonical atom witness must satisfy p('1') :- q('1')"
    );

    let integer = GroundTerm::Integer { value: 1 };
    let mut tampered = unification("'1'", "'1'");
    tampered.typed = typed_local_step(
        ground_call("p", vec![integer.clone()]),
        vec![ground_call("q", vec![integer])],
    );
    assert!(
        verify_current_circuit(policy, tampered).is_err(),
        "typed integer 1 must not open a compiler-fixed atom '1' pattern"
    );
}

#[test]
fn typed_rule_accepts_the_compiler_fixed_nested_list_paths() {
    let parent = tuple_list_term(",", vec![atom("a"), atom("b")]);
    assert!(
        verify_current_circuit(tuple_list_policy(), tuple_list_unification(parent)).is_ok(),
        "p([(a,b)]) :- q(a,b) must satisfy both legacy and typed policy views"
    );
}

#[test]
fn typed_rule_rejects_a_nested_functor_substitution() {
    let parent = tuple_list_term("pair", vec![atom("a"), atom("b")]);
    assert!(
        verify_current_circuit(tuple_list_policy(), tuple_list_unification(parent)).is_err(),
        "a pair/2 opening must not satisfy the compiler-fixed ','/2 constructor"
    );
}

#[test]
fn typed_rule_rejects_a_nested_arity_substitution() {
    let parent = tuple_list_term(",", vec![atom("a")]);
    assert!(
        verify_current_circuit(tuple_list_policy(), tuple_list_unification(parent)).is_err(),
        "a ','/1 opening must not satisfy the compiler-fixed ','/2 constructor"
    );
}

#[test]
fn typed_rule_rejects_values_swapped_at_compiler_fixed_list_paths() {
    let parent = tuple_list_term(",", vec![atom("b"), atom("a")]);
    assert!(
        verify_current_circuit(tuple_list_policy(), tuple_list_unification(parent)).is_err(),
        "the Head/Argument(0) and Head/Argument(1) variable paths must bind q/2 in order"
    );
}

#[test]
fn rejects_two_false_equalities_even_when_their_differences_cancel() {
    let rules = variable_policy(2, vec![vec![child("q", 2)]]);

    assert!(
        verify_current_circuit(rules, variable_unification(&["1", "0"], &["0", "1"])).is_err(),
        "(1 = 0) and (0 = 1) are both false; their field differences must not cancel"
    );
}

#[test]
fn rejects_one_greater_than_two() {
    let comparison = builtin_unification(
        vec![builtin_term(">", &["1", "2"])],
        ground_call(
            ">",
            vec![
                GroundTerm::Integer { value: 1 },
                GroundTerm::Integer { value: 2 },
            ],
        ),
    );
    assert!(
        verify_current_circuit(empty_policy(), comparison).is_err(),
        "1 > 2 is false in the configured integer semantics"
    );
}

#[test]
fn rejects_division_by_zero() {
    let division = builtin_unification(
        vec![builtin_term("is", &["0", "1"]), builtin_term("div", &["0"])],
        ground_call(
            "is",
            vec![
                GroundTerm::Integer { value: 0 },
                GroundTerm::Compound {
                    functor: "div".to_owned(),
                    args: vec![
                        GroundTerm::Integer { value: 1 },
                        GroundTerm::Integer { value: 0 },
                    ],
                },
            ],
        ),
    );
    assert!(
        verify_current_circuit(empty_policy(), division).is_err(),
        "division by zero must make the local proof step invalid"
    );
}
