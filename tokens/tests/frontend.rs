use common::circuit_dialect::{CircuitLimits, validate_ground_goal_for_circuit};
use common::data::{ClauseTemplate, ProofNode, RuleTemplateFile, TermSide};
use common::policy_ir::{Clause, PolicyFile, PredicateDef, Term};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tokens::lowering::compile_policy_for_current_circuit;
use tokens::{parse_goal, parse_policy, to_canonical_json};

fn limits() -> CircuitLimits {
    serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap()
}

fn predicate<'a>(policy: &'a PolicyFile, name: &str, arity: usize) -> &'a PredicateDef {
    policy
        .predicates
        .iter()
        .find(|predicate| predicate.indicator.name == name && predicate.indicator.arity == arity)
        .unwrap_or_else(|| panic!("missing predicate {name}/{arity}"))
}

fn clause<'a>(policy: &'a PolicyFile, name: &str, arity: usize, ordinal: usize) -> &'a Clause {
    &predicate(policy, name, arity).clauses[ordinal]
}

fn compound<'a>(term: &'a Term, expected: &str, arity: usize) -> &'a [Term] {
    match term {
        Term::Compound { functor, args } if functor == expected && args.len() == arity => args,
        other => panic!("expected {expected}/{arity}, got {other:?}"),
    }
}

#[test]
fn operator_precedence_and_associativity_are_structural() {
    let source = r#"
        p(R,A,B,C) :- R is A + B * C.
        q(R,A,B,C) :- R is (A + B) * C.
        r(R,A,B,C) :- R is A - B - C.
        s(R,A,B,C) :- R is A ^ B ^ C.
        t(R,A,B) :- R is A * B div 100.
        u(R,A,B) :- R is -A^B.
        v(R,A,B) :- R is (-A)^B.
        w(R,A,B) :- R is A^ -B.
    "#;
    let policy = parse_policy(source, "operators.pl").unwrap();

    let p_is = compound(&clause(&policy, "p", 4, 0).body, "is", 2);
    let plus = compound(&p_is[1], "+", 2);
    assert_eq!(plus[0], Term::Variable { id: 1 });
    assert_eq!(compound(&plus[1], "*", 2)[0], Term::Variable { id: 2 });
    assert_eq!(compound(&plus[1], "*", 2)[1], Term::Variable { id: 3 });

    let q_is = compound(&clause(&policy, "q", 4, 0).body, "is", 2);
    let multiply = compound(&q_is[1], "*", 2);
    assert_eq!(compound(&multiply[0], "+", 2)[0], Term::Variable { id: 1 });

    let r_is = compound(&clause(&policy, "r", 4, 0).body, "is", 2);
    let outer_subtract = compound(&r_is[1], "-", 2);
    assert_eq!(
        compound(&outer_subtract[0], "-", 2)[0],
        Term::Variable { id: 1 }
    );
    assert_eq!(outer_subtract[1], Term::Variable { id: 3 });

    let s_is = compound(&clause(&policy, "s", 4, 0).body, "is", 2);
    let power = compound(&s_is[1], "^", 2);
    assert_eq!(power[0], Term::Variable { id: 1 });
    assert_eq!(compound(&power[1], "^", 2)[1], Term::Variable { id: 3 });

    let t_is = compound(&clause(&policy, "t", 3, 0).body, "is", 2);
    let division = compound(&t_is[1], "div", 2);
    assert_eq!(compound(&division[0], "*", 2)[1], Term::Variable { id: 2 });
    assert_eq!(
        division[1],
        Term::Integer {
            decimal: "100".to_owned()
        }
    );

    let u_is = compound(&clause(&policy, "u", 3, 0).body, "is", 2);
    let outer_minus = compound(&u_is[1], "-", 1);
    assert_eq!(
        compound(&outer_minus[0], "^", 2)[0],
        Term::Variable { id: 1 }
    );

    let v_is = compound(&clause(&policy, "v", 3, 0).body, "is", 2);
    let v_power = compound(&v_is[1], "^", 2);
    assert_eq!(compound(&v_power[0], "-", 1)[0], Term::Variable { id: 1 });

    let w_is = compound(&clause(&policy, "w", 3, 0).body, "is", 2);
    let w_power = compound(&w_is[1], "^", 2);
    assert_eq!(compound(&w_power[1], "-", 1)[0], Term::Variable { id: 2 });
}

#[test]
fn lists_compounds_and_empty_list_are_lossless() {
    let source = "p([], [A,B], [H|T], [(_,S,V)|Tail], f(g(A), [B|T])).";
    let policy = parse_policy(source, "lists.pl").unwrap();
    let args = &clause(&policy, "p", 5, 0).head.args;

    assert!(matches!(
        &args[0],
        Term::List { items, tail: None } if items.is_empty()
    ));
    assert!(matches!(
        &args[1],
        Term::List { items, tail: None } if items.len() == 2
    ));
    assert!(matches!(
        &args[2],
        Term::List { items, tail: Some(_) } if items.len() == 1
    ));
    let tuple_list = match &args[3] {
        Term::List {
            items,
            tail: Some(_),
        } => items,
        other => panic!("expected an open list, got {other:?}"),
    };
    let comma = compound(&tuple_list[0], ",", 2);
    assert!(matches!(comma[0], Term::Variable { .. }));
    assert_eq!(compound(&comma[1], ",", 2).len(), 2);

    let outer = compound(&args[4], "f", 2);
    assert_eq!(compound(&outer[0], "g", 1)[0], Term::Variable { id: 0 });
    assert!(matches!(outer[1], Term::List { .. }));
}

#[test]
fn variables_are_clause_local_and_only_bare_underscore_is_anonymous() {
    let source = "p(X,X,_Named,_Named,_,_) :- q(X,_Named,_,Y), r(Y). p(X,X,N,N,_,_) :- r(X,N).";
    let policy = parse_policy(source, "variables.pl").unwrap();
    let first_clause = clause(&policy, "p", 6, 0);

    assert_eq!(first_clause.variables[0].source_name.as_deref(), Some("X"));
    assert_eq!(
        first_clause.variables[1].source_name.as_deref(),
        Some("_Named")
    );
    assert_eq!(first_clause.variables[2].source_name, None);
    assert_eq!(first_clause.variables[3].source_name, None);
    assert_eq!(first_clause.variables[4].source_name, None);
    assert_eq!(first_clause.variables[5].source_name.as_deref(), Some("Y"));
    assert_eq!(first_clause.head.args[0], first_clause.head.args[1]);
    assert_eq!(first_clause.head.args[2], first_clause.head.args[3]);
    assert_ne!(first_clause.head.args[4], first_clause.head.args[5]);

    let second = clause(&policy, "p", 6, 1);
    assert_eq!(second.variables[0].id, 0);
    assert_eq!(second.variables[0].source_name.as_deref(), Some("X"));
    assert_eq!(second.variables[1].id, 1);
    assert_eq!(second.variables[1].source_name.as_deref(), Some("N"));
}

#[test]
fn literal_types_and_integer_radices_stay_distinct() {
    let source = r#"p(atom,'atom',1,'1',"1",0b10,0o10,0x10,1.25)."#;
    let policy = parse_policy(source, "literals.pl").unwrap();
    let args = &clause(&policy, "p", 9, 0).head.args;

    assert_eq!(args[0], Term::atom("atom"));
    assert_eq!(args[1], Term::atom("atom"));
    assert!(matches!(&args[2], Term::Integer { decimal } if decimal == "1"));
    assert_eq!(args[3], Term::atom("1"));
    assert!(matches!(&args[4], Term::String { value } if value == "1"));
    assert!(matches!(&args[5], Term::Integer { decimal } if decimal == "2"));
    assert!(matches!(&args[6], Term::Integer { decimal } if decimal == "8"));
    assert!(matches!(&args[7], Term::Integer { decimal } if decimal == "16"));
    assert!(matches!(&args[8], Term::Float { lexeme } if lexeme == "1.25"));
}

#[test]
fn conjunction_and_disjunction_remain_one_clause_tree() {
    let policy = parse_policy("p :- a,b; c,d.", "control.pl").unwrap();
    let predicate = predicate(&policy, "p", 0);
    assert_eq!(predicate.clauses.len(), 1);

    let branches = compound(&predicate.clauses[0].body, ";", 2);
    assert_eq!(compound(&branches[0], ",", 2)[0], Term::atom("a"));
    assert_eq!(compound(&branches[1], ",", 2)[1], Term::atom("d"));
}

#[test]
fn current_policy_regressions_are_fixed() {
    let source = include_str!("../../prolog/policy.pl");
    let policy = parse_policy(source, "prolog/policy.pl").unwrap();

    assert_eq!(policy.schema_version, 2);
    assert_eq!(policy.predicates.len(), 10);
    assert_eq!(predicate(&policy, "savingsClass", 3).clauses.len(), 1);

    let sum_base = clause(&policy, "sumOfMonthlyConsumptions", 2, 1);
    assert!(matches!(
        &sum_base.head.args[0],
        Term::List { items, tail: None } if items.is_empty()
    ));

    let social_base = clause(&policy, "applySocialSupports", 3, 1);
    assert!(matches!(
        &social_base.head.args[1],
        Term::List { items, tail: None } if items.is_empty()
    ));

    let percent = clause(&policy, "applySupport", 4, 1);
    let goals = compound(&percent.body, ",", 2);
    let first_is = compound(&goals[0], "is", 2);
    let division = compound(&first_is[1], "div", 2);
    let multiplication = compound(&division[0], "*", 2);
    assert_eq!(multiplication[1], Term::Variable { id: 1 });

    let compiled = compile_policy_for_current_circuit(&policy, &limits()).unwrap();
    let circuit = compiled.layout();
    assert_eq!(circuit.predicates.len(), 10);
    assert_eq!(
        circuit
            .predicates
            .iter()
            .find(|predicate| predicate.name == "savingsClass")
            .unwrap()
            .clauses
            .len(),
        4
    );
    validate_circuit_references(&circuit);

    let circuit_percent = &circuit
        .predicates
        .iter()
        .find(|predicate| predicate.name == "applySupport")
        .unwrap()
        .clauses[1];
    assert!(has_head_variable_reference(circuit_percent, 2));
}

#[test]
fn matrix_and_input_policy_files_parse_with_the_same_frontend() {
    parse_policy(include_str!("../../prolog/matrix.pl"), "prolog/matrix.pl").unwrap();
    parse_policy(include_str!("../../prolog/input.pl"), "prolog/input.pl").unwrap();
}

#[test]
fn output_is_byte_deterministic_and_round_trips() {
    let source = include_str!("../../prolog/policy.pl");
    let baseline = to_canonical_json(&parse_policy(source, "prolog/policy.pl").unwrap()).unwrap();
    for _ in 0..20 {
        let current =
            to_canonical_json(&parse_policy(source, "prolog/policy.pl").unwrap()).unwrap();
        assert_eq!(current, baseline);
    }

    let decoded: PolicyFile = serde_json::from_str(&baseline).unwrap();
    assert_eq!(to_canonical_json(&decoded).unwrap(), baseline);

    let different_path =
        to_canonical_json(&parse_policy(source, "/another/location/policy.pl").unwrap()).unwrap();
    assert_eq!(different_path, baseline);
}

#[test]
fn formatting_and_source_locations_are_not_serialized() {
    let compact = to_canonical_json(&parse_policy("p(X):-q(X).", "compact.pl").unwrap()).unwrap();
    let formatted = to_canonical_json(
        &parse_policy(
            "% leading comment\n\n p( X ) :-\n    q( X ) .\n",
            "formatted.pl",
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(formatted, compact);

    let json: serde_json::Value = serde_json::from_str(&compact).unwrap();
    assert!(json.get("source").is_none());
    assert!(json["predicates"][0]["clauses"][0].get("source").is_none());
}

#[test]
fn syntax_errors_never_produce_a_partial_policy() {
    let error = parse_policy("p(X) :- q(X). trailing", "broken.pl").unwrap_err();
    let message = error.to_string();
    assert!(message.contains("broken.pl"));
    assert!(message.contains("mismatched input") || message.contains("extraneous input"));

    assert!(parse_policy("p([a|]).", "broken-list.pl").is_err());
    assert!(parse_policy("p(X) :- X < 2 < 3.", "chained-xfx.pl").is_err());
    assert!(parse_policy("p(X) :- X is A ** B ** C.", "chained-power.pl").is_err());
    assert!(parse_policy("p. /* never closed", "comment.pl").is_err());
    assert!(parse_policy(":- dynamic(p/1). p(a).", "directive.pl").is_err());
}

#[test]
fn semantic_ir_may_preserve_constructs_that_circuit_v1_rejects_explicitly() {
    let cut = parse_policy("p :- !.", "cut.pl").unwrap();
    assert!(matches!(
        clause(&cut, "p", 0, 0).body,
        Term::Atom { ref value } if value == "!"
    ));
    assert!(
        compile_policy_for_current_circuit(&cut, &limits())
            .unwrap_err()
            .to_string()
            .contains("cut")
    );

    let excessive_arity = parse_policy("p(A,B,C,D,E).", "arity.pl").unwrap();
    assert!(
        compile_policy_for_current_circuit(&excessive_arity, &limits())
            .unwrap_err()
            .to_string()
            .contains("exceeds configured maximum")
    );
}

#[test]
fn every_current_proof_tree_goal_is_fully_parsed_and_in_the_circuit_dialect() {
    fn visit(node: &ProofNode, limits: &CircuitLimits, count: &mut usize) {
        if let ProofNode::GoalNode(goal) = node {
            let term = parse_goal(&goal.goal, "input/proof_tree.json").unwrap();
            validate_ground_goal_for_circuit(&term, limits).unwrap();
            *count += 1;
            for child in &goal.subtree {
                visit(child, limits, count);
            }
        }
    }

    let limits: CircuitLimits =
        serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap();
    let tree: Vec<ProofNode> =
        serde_json::from_str(include_str!("../../tests/fixtures/current-proof-tree.json")).unwrap();
    let mut count = 0;
    for node in &tree {
        visit(node, &limits, &mut count);
    }
    assert_eq!(count, 50);
}

#[test]
fn goal_parser_rejects_partial_input_and_validator_rejects_unsupported_goals() {
    let limits: CircuitLimits =
        serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap();

    assert!(parse_goal("p(a). trailing", "trailing-goal").is_err());
    assert!(parse_goal("", "empty-goal").is_err());

    let variable = parse_goal("p(X)", "variable-goal").unwrap();
    assert!(
        validate_ground_goal_for_circuit(&variable, &limits)
            .unwrap_err()
            .message
            .contains("ground")
    );

    let power = parse_goal("4 is 2 ** 2", "power-goal").unwrap();
    assert!(
        validate_ground_goal_for_circuit(&power, &limits)
            .unwrap_err()
            .message
            .contains("**/2")
    );

    let conjunction = parse_goal("p(a), q(a)", "conjunction-goal").unwrap();
    assert!(
        validate_ground_goal_for_circuit(&conjunction, &limits)
            .unwrap_err()
            .message
            .contains("one goal")
    );
}

#[test]
fn cli_output_is_byte_identical_across_processes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source = root.join("../prolog/policy.pl");
    let limits = root.join("../config/circuit-limits.json");
    let directory = std::env::temp_dir().join(format!("batchman-tokens-{}", std::process::id()));
    if directory.exists() {
        fs::remove_dir_all(&directory).unwrap();
    }
    fs::create_dir_all(&directory).unwrap();

    let semantic_one = directory.join("semantic-one.json");
    let circuit_one = directory.join("circuit-one.json");
    let semantic_two = directory.join("semantic-two.json");
    let circuit_two = directory.join("circuit-two.json");

    for (semantic, circuit) in [(&semantic_one, &circuit_one), (&semantic_two, &circuit_two)] {
        let output = Command::new(env!("CARGO_BIN_EXE_tokens"))
            .arg(&source)
            .arg(semantic)
            .arg(circuit)
            .arg(&limits)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "tokens failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    assert_eq!(
        fs::read(semantic_one).unwrap(),
        fs::read(semantic_two).unwrap()
    );
    assert_eq!(
        fs::read(circuit_one).unwrap(),
        fs::read(circuit_two).unwrap()
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn cli_rejects_output_collisions_and_compiles_before_writing() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let limits = root.join("../config/circuit-limits.json");
    let directory =
        std::env::temp_dir().join(format!("batchman-tokens-failure-{}", std::process::id()));
    if directory.exists() {
        fs::remove_dir_all(&directory).unwrap();
    }
    fs::create_dir_all(&directory).unwrap();

    let source = directory.join("unsupported.pl");
    let semantic = directory.join("semantic.json");
    let compatibility = directory.join("compatibility.json");
    fs::write(&source, "p :- !.\n").unwrap();

    let failed_compile = Command::new(env!("CARGO_BIN_EXE_tokens"))
        .arg(&source)
        .arg(&semantic)
        .arg(&compatibility)
        .arg(&limits)
        .output()
        .unwrap();
    assert!(!failed_compile.status.success());
    assert!(!semantic.exists());
    assert!(!compatibility.exists());

    let same_output = directory.join("same.json");
    let collision = Command::new(env!("CARGO_BIN_EXE_tokens"))
        .arg(&source)
        .arg(&same_output)
        .arg(&same_output)
        .arg(&limits)
        .output()
        .unwrap();
    assert!(!collision.status.success());
    assert!(!same_output.exists());

    fs::remove_dir_all(directory).unwrap();
}

fn has_head_variable_reference(clause: &ClauseTemplate, head_argument: usize) -> bool {
    clause.equalities.iter().any(|equality| {
        [&equality.left, &equality.right].iter().any(|side| {
            matches!(
                side,
                TermSide::Ref(reference)
                    if reference.children_node_list == 0 && reference.arg == head_argument
            )
        })
    })
}

fn validate_circuit_references(rules: &RuleTemplateFile) {
    for predicate in &rules.predicates {
        for clause in &predicate.clauses {
            for equality in &clause.equalities {
                for side in [&equality.left, &equality.right] {
                    let TermSide::Ref(reference) = side else {
                        continue;
                    };
                    if reference.children_node_list == 0 {
                        assert_eq!(reference.predicate, 0);
                        assert!(reference.arg < predicate.arity);
                    } else {
                        let row = &clause.children[reference.children_node_list - 1];
                        let child = &row[reference.predicate];
                        assert!(reference.arg < child.arity);
                    }
                }
            }
        }
    }
}
