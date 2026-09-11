use common::{
    circuit_dialect::CircuitLimits,
    policy_compiler::{compile_policy_for_current_circuit, to_circuit_v1_json},
    policy_ir::{
        Call, Clause, POLICY_DIALECT, POLICY_SCHEMA_VERSION, PolicyFile, PredicateDef,
        PredicateIndicator, Term, VariableDecl,
    },
    typed_policy::{TypedGoalSlot, TypedPatternCheck, TypedTermRef, typed_policy_commitment},
    typed_term::{TermPath, TermPathStep, TermTag, hash_ground_term, symbol_to_field},
    witness_ir::GroundTerm,
};
use halo2_proofs::pasta::Fp;

fn limits() -> CircuitLimits {
    serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap()
}

fn current_policy() -> PolicyFile {
    serde_json::from_str(include_str!(
        "../../tests/fixtures/current-policy.rules.json"
    ))
    .unwrap()
}

fn single_clause_policy(
    head_argument: Term,
    variables: Vec<VariableDecl>,
    body: Term,
) -> PolicyFile {
    let indicator = PredicateIndicator {
        name: "p".to_owned(),
        arity: 1,
    };
    PolicyFile {
        schema_version: POLICY_SCHEMA_VERSION,
        dialect: POLICY_DIALECT.to_owned(),
        module: None,
        directives: Vec::new(),
        predicates: vec![PredicateDef {
            indicator: indicator.clone(),
            clauses: vec![Clause {
                ordinal: 0,
                variables,
                head: Call {
                    predicate: indicator,
                    args: vec![head_argument],
                },
                body,
            }],
        }],
    }
}

fn typed_ref(goal: TypedGoalSlot, argument: u8, steps: Vec<TermPathStep>) -> TypedTermRef {
    TypedTermRef {
        goal,
        argument,
        path: TermPath::new(steps),
    }
}

fn exact_head_argument_digest(term: Term) -> Fp {
    let compiled = compile_policy_for_current_circuit(
        &single_clause_policy(term, Vec::new(), Term::atom("true")),
        &limits(),
    )
    .unwrap();
    let candidate = &compiled.typed_plan().predicates[0].candidates[0];
    assert_eq!(candidate.patterns.len(), 1);
    match &candidate.patterns[0] {
        TypedPatternCheck::Exact { at, digest } => {
            assert_eq!(at, &typed_ref(TypedGoalSlot::Parent, 0, Vec::new()));
            *digest
        }
        other => panic!("expected one exact typed pattern, got {other:?}"),
    }
}

#[test]
fn semantic_rules_compile_to_the_existing_golden_layout_in_memory() {
    let compiled = compile_policy_for_current_circuit(&current_policy(), &limits()).unwrap();
    assert_eq!(compiled.analysis().predicate_count, 10);
    assert_eq!(compiled.analysis().effective_candidate_count, 18);
    assert_eq!(
        to_circuit_v1_json(compiled.layout()).unwrap(),
        include_str!("../../tests/fixtures/current-policy.circuit-v1.json")
    );
    assert_eq!(
        compiled
            .typed_plan()
            .predicates
            .iter()
            .map(|predicate| predicate.candidates.len())
            .sum::<usize>(),
        18
    );
    compiled.typed_plan().validate().unwrap();
}

#[test]
fn compilation_is_deterministic_and_independent_of_json_formatting() {
    let policy = current_policy();
    let expected_compiled = compile_policy_for_current_circuit(&policy, &limits()).unwrap();
    let expected = to_circuit_v1_json(expected_compiled.layout()).unwrap();
    let expected_typed = expected_compiled.typed_plan().clone();
    let expected_typed_commitment = typed_policy_commitment(&expected_typed);

    for _ in 0..20 {
        let reparsed: PolicyFile =
            serde_json::from_str(&serde_json::to_string(&policy).unwrap()).unwrap();
        let actual_compiled = compile_policy_for_current_circuit(&reparsed, &limits()).unwrap();
        let actual = to_circuit_v1_json(actual_compiled.layout()).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual_compiled.typed_plan(), &expected_typed);
        assert_eq!(
            typed_policy_commitment(actual_compiled.typed_plan()),
            expected_typed_commitment
        );
    }
}

#[test]
fn typed_plan_uses_argument_root_paths_for_recursive_lists_and_tuple_compounds() {
    let compiled = compile_policy_for_current_circuit(&current_policy(), &limits()).unwrap();
    let predicate = compiled
        .typed_plan()
        .predicates
        .iter()
        .find(|predicate| predicate.name == symbol_to_field("applySocialSupports"))
        .unwrap();
    let candidate = predicate
        .candidates
        .iter()
        .find(|candidate| {
            candidate.id.source_clause_ordinal == 0 && candidate.id.branch_ordinal == 0
        })
        .unwrap();

    assert_eq!(candidate.children.len(), 2);
    assert_eq!(candidate.children[0].name, symbol_to_field("applySupport"));
    assert_eq!(
        candidate.children[1].name,
        symbol_to_field("applySocialSupports")
    );

    let support_type = typed_ref(
        TypedGoalSlot::Parent,
        1,
        vec![
            TermPathStep::Head,
            TermPathStep::Argument { index: 1 },
            TermPathStep::Argument { index: 0 },
        ],
    );
    let support_value = typed_ref(
        TypedGoalSlot::Parent,
        1,
        vec![
            TermPathStep::Head,
            TermPathStep::Argument { index: 1 },
            TermPathStep::Argument { index: 1 },
        ],
    );
    let tail = typed_ref(TypedGoalSlot::Parent, 1, vec![TermPathStep::Tail]);
    assert!(candidate.equalities.iter().any(|equality| {
        equality.left == support_type
            && equality.right == typed_ref(TypedGoalSlot::Child { index: 0 }, 1, Vec::new())
    }));
    assert!(candidate.equalities.iter().any(|equality| {
        equality.left == support_value
            && equality.right == typed_ref(TypedGoalSlot::Child { index: 0 }, 2, Vec::new())
    }));
    assert!(candidate.equalities.iter().any(|equality| {
        equality.left == tail
            && equality.right == typed_ref(TypedGoalSlot::Child { index: 1 }, 1, Vec::new())
    }));

    let has_shape = |at: TypedTermRef, tag: TermTag, payload: Fp, arity: u8| {
        candidate.patterns.iter().any(|pattern| {
            matches!(
                pattern,
                TypedPatternCheck::Shape {
                    at: actual,
                    tag: actual_tag,
                    payload: actual_payload,
                    arity: actual_arity,
                } if actual == &at
                    && *actual_tag == tag
                    && *actual_payload == payload
                    && *actual_arity == arity
            )
        })
    };
    assert!(has_shape(
        typed_ref(TypedGoalSlot::Parent, 1, Vec::new()),
        TermTag::Cons,
        Fp::zero(),
        2,
    ));
    assert!(has_shape(
        typed_ref(TypedGoalSlot::Parent, 1, vec![TermPathStep::Head]),
        TermTag::Compound,
        symbol_to_field(","),
        2,
    ));
    assert!(has_shape(
        typed_ref(
            TypedGoalSlot::Parent,
            1,
            vec![TermPathStep::Head, TermPathStep::Argument { index: 1 }]
        ),
        TermTag::Compound,
        symbol_to_field(","),
        2,
    ));
}

#[test]
fn typed_arithmetic_remains_one_child_and_keeps_its_ast_paths() {
    let result = Term::Variable { id: 0 };
    let left = Term::Variable { id: 1 };
    let right = Term::Variable { id: 2 };
    let policy = single_clause_policy(
        result.clone(),
        vec![
            VariableDecl {
                id: 0,
                source_name: Some("R".to_owned()),
            },
            VariableDecl {
                id: 1,
                source_name: Some("A".to_owned()),
            },
            VariableDecl {
                id: 2,
                source_name: Some("B".to_owned()),
            },
        ],
        Term::compound(
            "is",
            vec![
                result,
                Term::compound(
                    "div",
                    vec![
                        Term::compound("*", vec![left, right]),
                        Term::Integer {
                            decimal: "100".to_owned(),
                        },
                    ],
                ),
            ],
        ),
    );
    let compiled = compile_policy_for_current_circuit(&policy, &limits()).unwrap();
    let candidate = &compiled.typed_plan().predicates[0].candidates[0];
    assert_eq!(candidate.children.len(), 1);
    assert_eq!(candidate.children[0].name, symbol_to_field("is"));
    assert_eq!(candidate.children[0].arity, 2);
    assert_eq!(candidate.equalities.len(), 1);
    assert_eq!(
        candidate.equalities[0].left,
        typed_ref(TypedGoalSlot::Parent, 0, Vec::new())
    );
    assert_eq!(
        candidate.equalities[0].right,
        typed_ref(TypedGoalSlot::Child { index: 0 }, 0, Vec::new())
    );

    assert!(candidate.patterns.iter().any(|pattern| matches!(
        pattern,
        TypedPatternCheck::Shape {
            at,
            tag: TermTag::Compound,
            payload,
            arity: 2,
        } if at == &typed_ref(TypedGoalSlot::Child { index: 0 }, 1, Vec::new())
            && *payload == symbol_to_field("div")
    )));
    assert!(candidate.patterns.iter().any(|pattern| matches!(
        pattern,
        TypedPatternCheck::Shape {
            at,
            tag: TermTag::Compound,
            payload,
            arity: 2,
        } if at == &typed_ref(
                TypedGoalSlot::Child { index: 0 },
                1,
                vec![TermPathStep::Argument { index: 0 }],
            ) && *payload == symbol_to_field("*")
    )));
    let hundred = hash_ground_term(&GroundTerm::Integer { value: 100 }).unwrap();
    assert!(candidate.patterns.iter().any(|pattern| matches!(
        pattern,
        TypedPatternCheck::Exact { at, digest }
            if at == &typed_ref(
                TypedGoalSlot::Child { index: 0 },
                1,
                vec![TermPathStep::Argument { index: 1 }],
            ) && *digest == hundred
    )));
}

#[test]
fn typed_variable_equalities_are_the_minimal_n_minus_one_spanning_star() {
    let variable = Term::Variable { id: 0 };
    let policy = single_clause_policy(
        variable.clone(),
        vec![VariableDecl {
            id: 0,
            source_name: Some("X".to_owned()),
        }],
        Term::compound(
            ",",
            vec![
                Term::compound("q", vec![variable.clone()]),
                Term::compound("r", vec![variable]),
            ],
        ),
    );
    let compiled = compile_policy_for_current_circuit(&policy, &limits()).unwrap();
    let candidate = &compiled.typed_plan().predicates[0].candidates[0];
    assert_eq!(candidate.children.len(), 2);
    assert_eq!(candidate.equalities.len(), 2);
    assert!(
        candidate
            .equalities
            .iter()
            .all(|equality| equality.left == typed_ref(TypedGoalSlot::Parent, 0, Vec::new()))
    );
}

#[test]
fn typed_list_plan_collapses_a_maximal_ground_suffix_to_one_exact_check() {
    let variable = Term::Variable { id: 0 };
    let policy = single_clause_policy(
        Term::List {
            items: vec![
                variable,
                Term::atom("a"),
                Term::Integer {
                    decimal: "2".to_owned(),
                },
            ],
            tail: None,
        },
        vec![VariableDecl {
            id: 0,
            source_name: Some("X".to_owned()),
        }],
        Term::atom("true"),
    );
    let compiled = compile_policy_for_current_circuit(&policy, &limits()).unwrap();
    let candidate = &compiled.typed_plan().predicates[0].candidates[0];
    assert_eq!(candidate.equalities.len(), 0);
    assert_eq!(candidate.patterns.len(), 2);
    assert!(matches!(
        &candidate.patterns[0],
        TypedPatternCheck::Shape {
            at,
            tag: TermTag::Cons,
            payload,
            arity: 2,
        } if at == &typed_ref(TypedGoalSlot::Parent, 0, Vec::new())
            && *payload == Fp::zero()
    ));

    let ground_suffix = GroundTerm::Cons {
        head: Box::new(GroundTerm::Atom {
            value: "a".to_owned(),
        }),
        tail: Box::new(GroundTerm::Cons {
            head: Box::new(GroundTerm::Integer { value: 2 }),
            tail: Box::new(GroundTerm::Nil),
        }),
    };
    let expected = hash_ground_term(&ground_suffix).unwrap();
    assert!(matches!(
        &candidate.patterns[1],
        TypedPatternCheck::Exact { at, digest }
            if at == &typed_ref(
                TypedGoalSlot::Parent,
                0,
                vec![TermPathStep::Tail],
            ) && *digest == expected
    ));
}

#[test]
fn typed_exact_patterns_preserve_literal_types() {
    let digit_atom = exact_head_argument_digest(Term::atom("1"));
    let integer = exact_head_argument_digest(Term::Integer {
        decimal: "1".to_owned(),
    });
    assert_ne!(digit_atom, integer);

    let nil = exact_head_argument_digest(Term::List {
        items: Vec::new(),
        tail: None,
    });
    let bracket_atom = exact_head_argument_digest(Term::atom("[]"));
    assert_ne!(nil, bracket_atom);
}

#[test]
fn compiled_policy_transfers_legacy_and_typed_views_together() {
    let compiled = compile_policy_for_current_circuit(&current_policy(), &limits()).unwrap();
    let expected = compiled.typed_plan().clone();
    let (legacy, typed) = compiled.into_circuit_parts();
    assert_eq!(typed, expected);
    assert_eq!(
        legacy
            .predicates
            .iter()
            .map(|predicate| predicate.clauses.len())
            .sum::<usize>(),
        18
    );
}

#[test]
fn representable_semantic_terms_are_never_silently_truncated() {
    let nested = single_clause_policy(
        Term::compound("f", vec![Term::atom("a")]),
        Vec::new(),
        Term::atom("true"),
    );
    let error = compile_policy_for_current_circuit(&nested, &limits())
        .unwrap_err()
        .to_string();
    assert!(error.contains("nested term"), "unexpected error: {error}");

    let variable = Term::Variable { id: 0 };
    let right_nested_arithmetic = single_clause_policy(
        variable.clone(),
        vec![VariableDecl {
            id: 0,
            source_name: Some("R".to_owned()),
        }],
        Term::compound(
            "is",
            vec![
                variable,
                Term::compound(
                    "+",
                    vec![
                        Term::Integer {
                            decimal: "1".to_owned(),
                        },
                        Term::compound(
                            "*",
                            vec![
                                Term::Integer {
                                    decimal: "2".to_owned(),
                                },
                                Term::Integer {
                                    decimal: "3".to_owned(),
                                },
                            ],
                        ),
                    ],
                ),
            ],
        ),
    );
    let error = compile_policy_for_current_circuit(&right_nested_arithmetic, &limits())
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("right-nested expression"),
        "unexpected error: {error}"
    );
}

#[test]
fn production_binaries_do_not_read_the_compatibility_sidecar() {
    for (name, source) in [
        ("prove", include_str!("../../prove/src/main.rs")),
        ("verify", include_str!("../../verify/src/main.rs")),
    ] {
        assert!(
            !source.contains("rules.circuit-v1.json"),
            "{name} must compile its circuit policy directly from rules.json"
        );
    }
}

#[test]
fn nested_disjunctions_are_fully_distributed_and_true_is_an_identity() {
    let variable = Term::Variable { id: 0 };
    let call = |name: &str| Term::compound(name, vec![variable.clone()]);
    let policy = single_clause_policy(
        variable.clone(),
        vec![VariableDecl {
            id: 0,
            source_name: Some("X".to_owned()),
        }],
        Term::compound(
            ",",
            vec![
                Term::atom("true"),
                Term::compound(
                    ",",
                    vec![call("q"), Term::compound(";", vec![call("r"), call("s")])],
                ),
            ],
        ),
    );

    let compiled = compile_policy_for_current_circuit(&policy, &limits()).unwrap();
    assert_eq!(compiled.analysis().effective_candidate_count, 2);
    let clauses = &compiled.layout().predicates[0].clauses;
    assert_eq!(clauses.len(), 2);
    let child_names = clauses
        .iter()
        .map(|clause| {
            clause
                .children
                .iter()
                .map(|row| row[0].name.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(child_names, vec![vec!["q", "r"], vec!["q", "s"]]);
    assert!(child_names.iter().flatten().all(|name| *name != "true"));
}

#[test]
fn conjunction_uses_the_cartesian_product_of_disjunction_branches() {
    let variable = Term::Variable { id: 0 };
    let call = |name: &str| Term::compound(name, vec![variable.clone()]);
    let policy = single_clause_policy(
        variable.clone(),
        vec![VariableDecl {
            id: 0,
            source_name: Some("X".to_owned()),
        }],
        Term::compound(
            ",",
            vec![
                Term::compound(";", vec![call("q"), call("r")]),
                Term::compound(";", vec![call("s"), call("t")]),
            ],
        ),
    );

    let compiled = compile_policy_for_current_circuit(&policy, &limits()).unwrap();
    assert_eq!(compiled.analysis().effective_candidate_count, 4);
    let child_names = compiled.layout().predicates[0]
        .clauses
        .iter()
        .map(|clause| {
            clause
                .children
                .iter()
                .map(|row| row[0].name.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        child_names,
        vec![
            vec!["q", "s"],
            vec!["q", "t"],
            vec!["r", "s"],
            vec!["r", "t"]
        ]
    );
}

#[test]
fn prolog_less_than_or_equal_uses_the_shared_legacy_token() {
    let variable = Term::Variable { id: 0 };
    let policy = single_clause_policy(
        variable.clone(),
        vec![VariableDecl {
            id: 0,
            source_name: Some("X".to_owned()),
        }],
        Term::compound(
            "=<",
            vec![
                variable,
                Term::Integer {
                    decimal: "10".to_owned(),
                },
            ],
        ),
    );
    let compiled = compile_policy_for_current_circuit(&policy, &limits()).unwrap();
    assert_eq!(
        compiled.layout().predicates[0].clauses[0].children[0][0].name,
        "<="
    );
}
