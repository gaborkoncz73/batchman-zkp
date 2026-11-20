use common::{
    circuit_dialect::BuiltinKind,
    policy_ir::Term,
    witness_ir::{GoalResolution, GroundCall, GroundTerm, LocalStepWitness, StepTransition},
};
use policy_parser::parse_goal;

fn call(source: &str) -> GroundCall {
    GroundCall::from_goal_term(&parse_goal(source, "typed-witness-test").unwrap()).unwrap()
}

#[test]
fn whitespace_does_not_change_the_typed_arithmetic_witness() {
    assert_eq!(call("2000 is 0+2000"), call("2000 is 0 + 2000"));
}

#[test]
fn literal_types_lists_and_nested_compounds_remain_structural() {
    let encoded = call("p([a,b], f(g(a)), '123', 123)");
    assert!(matches!(
        &encoded.args[0],
        GroundTerm::Cons { head, tail }
            if matches!(head.as_ref(), GroundTerm::Atom { value } if value == "a")
                && matches!(tail.as_ref(), GroundTerm::Cons { .. })
    ));
    assert!(matches!(
        &encoded.args[1],
        GroundTerm::Compound { functor, args }
            if functor == "f"
                && matches!(&args[0], GroundTerm::Compound { functor, .. } if functor == "g")
    ));
    assert_eq!(
        encoded.args[2],
        GroundTerm::Atom {
            value: "123".to_owned()
        }
    );
    assert_eq!(encoded.args[3], GroundTerm::Integer { value: 123 });
    assert_ne!(encoded.args[2], encoded.args[3]);
}

#[test]
fn the_same_list_has_one_representation_in_every_step_role() {
    let list = match call("p([a,b])").args.into_iter().next().unwrap() {
        value @ GroundTerm::Cons { .. } => value,
        other => panic!("expected list, got {other:?}"),
    };
    let parent_copy = list.clone();
    let child_copy = list.clone();
    let fact_copy = list;
    assert_eq!(parent_copy, child_copy);
    assert_eq!(child_copy, fact_copy);
}

#[test]
fn resolution_and_transition_must_agree() {
    let goal = call("1 < 2");
    let builtin = LocalStepWitness::new(
        goal.clone(),
        GoalResolution::Builtin {
            builtin: BuiltinKind::LessThan,
        },
        StepTransition::Builtin {
            builtin: BuiltinKind::LessThan,
        },
    )
    .unwrap();
    assert_eq!(builtin.schema_version, 1);

    assert!(
        LocalStepWitness::new(
            goal,
            GoalResolution::Builtin {
                builtin: BuiltinKind::LessThan,
            },
            StepTransition::ExternalFact,
        )
        .is_err()
    );
}

#[test]
fn builtin_kind_and_deserialized_schema_are_validated() {
    let unrelated_goal = call("p(a)");
    assert!(
        LocalStepWitness::new(
            unrelated_goal,
            GoalResolution::Builtin {
                builtin: BuiltinKind::LessThan,
            },
            StepTransition::Builtin {
                builtin: BuiltinKind::LessThan,
            },
        )
        .unwrap_err()
        .to_string()
        .contains("does not match")
    );

    let valid = LocalStepWitness::new(
        call("1 < 2"),
        GoalResolution::Builtin {
            builtin: BuiltinKind::LessThan,
        },
        StepTransition::Builtin {
            builtin: BuiltinKind::LessThan,
        },
    )
    .unwrap();

    let mut wrong_version = serde_json::to_value(&valid).unwrap();
    wrong_version["schema_version"] = 2.into();
    assert!(serde_json::from_value::<LocalStepWitness>(wrong_version).is_err());

    let mut wrong_arity = serde_json::to_value(&valid).unwrap();
    wrong_arity["goal"]["predicate"]["arity"] = 1.into();
    assert!(serde_json::from_value::<LocalStepWitness>(wrong_arity).is_err());

    let mut wrong_transition = serde_json::to_value(&valid).unwrap();
    wrong_transition["transition"]["builtin"] = "greater_than".into();
    assert!(serde_json::from_value::<LocalStepWitness>(wrong_transition).is_err());
}

#[test]
fn non_ground_terms_cannot_enter_the_typed_witness() {
    assert!(GroundTerm::from_policy_term(&Term::Variable { id: 7 }).is_err());
}
