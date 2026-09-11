use common::{circuit_dialect::CircuitLimits, data::FactEntry, witness_ir::GroundTerm};
use tokens::parse_fact_registry;

fn limits() -> CircuitLimits {
    serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap()
}

fn entry(goal: &str, salt: &str) -> FactEntry {
    FactEntry {
        goal: goal.to_owned(),
        salt: salt.to_owned(),
    }
}

#[test]
fn registry_preserves_proper_lists_and_tuple_structure() {
    let facts = parse_fact_registry(
        &[
            entry("months([1,2])", "7"),
            entry("credential([('role',nominal,10000)])", "8"),
        ],
        &limits(),
        "facts-test",
    )
    .unwrap();

    assert_eq!(facts[0].goal.predicate.name, "months");
    assert_eq!(facts[0].goal.predicate.arity, 1);
    assert!(matches!(facts[0].goal.args[0], GroundTerm::Cons { .. }));

    let GroundTerm::Cons { head, tail } = &facts[1].goal.args[0] else {
        panic!("tuple fact argument must remain a proper list")
    };
    assert!(matches!(
        head.as_ref(),
        GroundTerm::Compound { functor, args } if functor == "," && args.len() == 2
    ));
    assert!(matches!(tail.as_ref(), GroundTerm::Nil));
}

#[test]
fn digit_atom_and_integer_are_distinct_registry_facts() {
    let facts = parse_fact_registry(
        &[entry("tag('1')", "11"), entry("tag(1)", "12")],
        &limits(),
        "facts-test",
    )
    .unwrap();

    assert_ne!(facts[0].goal, facts[1].goal);
    assert_ne!(
        facts[0].commitment().unwrap(),
        facts[1].commitment().unwrap()
    );
    assert!(matches!(facts[0].goal.args[0], GroundTerm::Atom { .. }));
    assert!(matches!(facts[1].goal.args[0], GroundTerm::Integer { .. }));
}

#[test]
fn exact_duplicates_builtins_and_noncanonical_salts_are_rejected() {
    assert!(
        parse_fact_registry(
            &[entry("p([a])", "1"), entry(" p( [ a ] ) ", "2")],
            &limits(),
            "facts-test",
        )
        .unwrap_err()
        .to_string()
        .contains("duplicates")
    );
    assert!(
        parse_fact_registry(&[entry("1=1", "1")], &limits(), "facts-test")
            .unwrap_err()
            .to_string()
            .contains("cannot be an external fact")
    );
    assert!(
        parse_fact_registry(&[entry("p(a)", "01")], &limits(), "facts-test")
            .unwrap_err()
            .to_string()
            .contains("canonical")
    );

    let legacy_json = r#"[{"predicate":"p","args":[["a"]],"salt":"1"}]"#;
    assert!(serde_json::from_str::<Vec<FactEntry>>(legacy_json).is_err());
}
