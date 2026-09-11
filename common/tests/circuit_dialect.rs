use std::collections::BTreeSet;

use common::circuit_dialect::{BuiltinKind, CircuitLimits, validate_ground_goal_for_circuit};
use common::policy_ir::{
    Call, Clause, POLICY_DIALECT, POLICY_SCHEMA_VERSION, PolicyFile, PredicateDef,
    PredicateIndicator, Term,
};

fn limits() -> CircuitLimits {
    serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap()
}

fn policy_with_body(body: Term) -> PolicyFile {
    let indicator = PredicateIndicator {
        name: "p".to_owned(),
        arity: 0,
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
                variables: Vec::new(),
                head: Call {
                    predicate: indicator,
                    args: Vec::new(),
                },
                body,
            }],
        }],
    }
}

#[test]
fn current_policy_passes_and_produces_stable_analysis() {
    let policy: PolicyFile = serde_json::from_str(include_str!(
        "../../tests/fixtures/current-policy.rules.json"
    ))
    .unwrap();
    let analysis = policy.validate_for_circuit(&limits()).unwrap();

    assert_eq!(analysis.predicate_count, 10);
    assert_eq!(analysis.source_clause_count, 15);
    assert_eq!(analysis.effective_candidate_count, 18);
    assert_eq!(analysis.max_arity, 4);
    assert_eq!(analysis.max_body_goals_per_branch, 10);
    assert_eq!(analysis.max_variables_per_clause, 10);
    assert_eq!(
        analysis.used_builtins,
        BTreeSet::from([
            BuiltinKind::Unify,
            BuiltinKind::Is,
            BuiltinKind::GreaterThan,
        ])
    );
    assert_eq!(
        analysis.used_arithmetic_operators,
        BTreeSet::from([
            "+".to_owned(),
            "-".to_owned(),
            "*".to_owned(),
            "div".to_owned(),
        ])
    );
    let external: BTreeSet<_> = [
        ("currentConsumption", 1),
        ("currentPrice", 2),
        ("inputPayment", 1),
        ("monthlyConsumptions", 1),
        ("rolling_treshold", 2),
        ("savings_treshold", 2),
        ("socialCreds", 1),
        ("support_matrix", 4),
    ]
    .into_iter()
    .map(|(name, arity)| PredicateIndicator {
        name: name.to_owned(),
        arity,
    })
    .collect();
    assert_eq!(analysis.required_external_predicates, external);
}

#[test]
fn semantic_ir_can_preserve_but_circuit_rejects_unsupported_constructs() {
    for (body, expected) in [
        (Term::atom("!"), "cut"),
        (
            Term::compound("\\+", vec![Term::atom("q")]),
            "control operator",
        ),
        (
            Term::compound("->", vec![Term::atom("q"), Term::atom("r")]),
            "control operator",
        ),
        (
            Term::compound(
                "q",
                vec![Term::Float {
                    lexeme: "1.5".to_owned(),
                }],
            ),
            "float",
        ),
        (
            Term::compound(
                "q",
                vec![Term::String {
                    value: "x".to_owned(),
                }],
            ),
            "strings",
        ),
        (
            Term::compound(
                "q",
                vec![Term::BackQuoted {
                    value: "x".to_owned(),
                }],
            ),
            "back-quoted",
        ),
    ] {
        let error = policy_with_body(body)
            .validate_for_circuit(&limits())
            .unwrap_err();
        assert!(
            error.message.contains(expected),
            "expected {expected:?} in {error}"
        );
        assert!(error.path.contains(".body"));
    }
}

#[test]
fn only_the_declared_builtin_and_arithmetic_allow_lists_pass() {
    let nested_supported = policy_with_body(Term::compound(
        "is",
        vec![
            Term::Integer {
                decimal: "14".to_owned(),
            },
            Term::compound(
                "+",
                vec![
                    Term::Integer {
                        decimal: "2".to_owned(),
                    },
                    Term::compound(
                        "*",
                        vec![
                            Term::Integer {
                                decimal: "3".to_owned(),
                            },
                            Term::Integer {
                                decimal: "4".to_owned(),
                            },
                        ],
                    ),
                ],
            ),
        ],
    ));
    nested_supported.validate_for_circuit(&limits()).unwrap();

    let unsupported = policy_with_body(Term::compound(
        "is",
        vec![
            Term::Integer {
                decimal: "8".to_owned(),
            },
            Term::compound(
                "**",
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
    ));
    assert!(
        unsupported
            .validate_for_circuit(&limits())
            .unwrap_err()
            .message
            .contains("**/2")
    );

    let unary = policy_with_body(Term::compound(
        "is",
        vec![
            Term::Integer {
                decimal: "1".to_owned(),
            },
            Term::compound(
                "-",
                vec![Term::Integer {
                    decimal: "1".to_owned(),
                }],
            ),
        ],
    ));
    assert!(
        unary
            .validate_for_circuit(&limits())
            .unwrap_err()
            .message
            .contains("arity 2")
    );

    let four_operations = policy_with_body(Term::compound(
        "is",
        vec![
            Term::Integer {
                decimal: "5".to_owned(),
            },
            Term::compound(
                "+",
                vec![
                    Term::compound(
                        "+",
                        vec![
                            Term::compound(
                                "+",
                                vec![
                                    Term::compound(
                                        "+",
                                        vec![
                                            Term::Integer {
                                                decimal: "1".to_owned(),
                                            },
                                            Term::Integer {
                                                decimal: "1".to_owned(),
                                            },
                                        ],
                                    ),
                                    Term::Integer {
                                        decimal: "1".to_owned(),
                                    },
                                ],
                            ),
                            Term::Integer {
                                decimal: "1".to_owned(),
                            },
                        ],
                    ),
                    Term::Integer {
                        decimal: "1".to_owned(),
                    },
                ],
            ),
        ],
    ));
    let error = four_operations.validate_for_circuit(&limits()).unwrap_err();
    assert!(error.message.contains("maximum of 3 operations"));
}

#[test]
fn integers_and_term_shapes_obey_the_explicit_profile() {
    let non_canonical = policy_with_body(Term::compound(
        "q",
        vec![Term::Integer {
            decimal: "01".to_owned(),
        }],
    ));
    assert!(
        non_canonical
            .validate_for_circuit(&limits())
            .unwrap_err()
            .message
            .contains("canonical")
    );

    let out_of_range = policy_with_body(Term::compound(
        "q",
        vec![Term::Integer {
            decimal: "4294967296".to_owned(),
        }],
    ));
    assert!(
        out_of_range
            .validate_for_circuit(&limits())
            .unwrap_err()
            .message
            .contains("32-bit")
    );

    let malformed_list = policy_with_body(Term::compound(
        "q",
        vec![Term::List {
            items: Vec::new(),
            tail: Some(Box::new(Term::atom("tail"))),
        }],
    ));
    assert!(
        malformed_list
            .validate_for_circuit(&limits())
            .unwrap_err()
            .message
            .contains("not canonical")
    );
}

#[test]
fn proof_goals_must_be_ground_single_goals() {
    let profile = limits();
    let variable = Term::compound("p", vec![Term::Variable { id: 0 }]);
    assert!(
        validate_ground_goal_for_circuit(&variable, &profile)
            .unwrap_err()
            .message
            .contains("ground")
    );

    let conjunction = Term::compound(",", vec![Term::atom("p"), Term::atom("q")]);
    assert!(
        validate_ground_goal_for_circuit(&conjunction, &profile)
            .unwrap_err()
            .message
            .contains("one goal")
    );

    let less_equal = Term::compound(
        "=<",
        vec![
            Term::Integer {
                decimal: "1".to_owned(),
            },
            Term::Integer {
                decimal: "2".to_owned(),
            },
        ],
    );
    validate_ground_goal_for_circuit(&less_equal, &profile).unwrap();

    let wrong_spelling = Term::compound(
        "<=",
        vec![
            Term::Integer {
                decimal: "1".to_owned(),
            },
            Term::Integer {
                decimal: "2".to_owned(),
            },
        ],
    );
    assert!(
        validate_ground_goal_for_circuit(&wrong_spelling, &profile)
            .unwrap_err()
            .message
            .contains("not supported")
    );
}

#[test]
fn module_directive_and_normalized_exports_must_match() {
    let mut policy: PolicyFile = serde_json::from_str(include_str!(
        "../../tests/fixtures/current-policy.rules.json"
    ))
    .unwrap();
    policy.module.as_mut().unwrap().exports[0].name = "missing".to_owned();
    let error = policy.validate_for_circuit(&limits()).unwrap_err();
    assert!(
        error.message.contains("does not match")
            || error.message.contains("no predicate definition")
    );
}

#[test]
fn semantic_terms_and_limit_profiles_reject_unknown_json_fields() {
    assert!(serde_json::from_str::<Term>(r#"{"kind":"atom","value":"a","extra":1}"#).is_err());
    let mut profile: serde_json::Value =
        serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap();
    profile["extra"] = serde_json::json!(1);
    assert!(serde_json::from_value::<CircuitLimits>(profile).is_err());
}
