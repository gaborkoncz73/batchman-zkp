//! The model of the repository's policy (`prolog/policy.pl`) against its proof
//! tree (`input/proof_tree.json`).

mod support;

use std::collections::BTreeSet;

use policy_constraints::ConstraintSystem;
use serde_json::Value;
use support::{assignment, satisfied};

const POLICY: &str = include_str!("../../prolog/policy.pl");
const PROOF_TREE: &str = include_str!("../../input/proof_tree.json");

fn energy_policy() -> ConstraintSystem {
    support::system(POLICY)
}

#[test]
fn branches_cover_every_clause_alternative_builtin_form_and_fact() {
    let system = energy_policy();
    let clause_alternatives = [
        ("applySavingsSupport", 4, 1),
        ("applySocialSupports", 3, 2),
        ("applySupport", 4, 2),
        ("consumptionClass", 2, 3),
        ("endPrice", 1, 1),
        ("inputPriceOk", 0, 1),
        ("priceBase", 1, 1),
        ("rollingConsumption", 2, 1),
        ("savingsClass", 3, 4),
        ("sumOfMonthlyConsumptions", 2, 2),
    ];
    for (name, arity, count) in clause_alternatives {
        assert_eq!(
            system.clause_branches(name, arity).count(),
            count,
            "{name}/{arity}"
        );
    }

    let forms: Vec<String> = system
        .builtin_checks()
        .map(|check| check.to_string())
        .collect();
    assert_eq!(
        forms,
        [
            "_ is (_ - _)",
            "_ is ((_ * _) div _)",
            "_ > _",
            "_ = _",
            "_ is (_ * _)",
            "_ is (_ div _)",
            "_ is (_ + _)",
        ]
    );

    let externals: Vec<String> = system
        .external_predicates()
        .map(|p| format!("{}/{}", p.name, p.arity))
        .collect();
    assert_eq!(
        externals,
        [
            "currentConsumption/1",
            "currentPrice/2",
            "inputPayment/1",
            "monthlyConsumptions/1",
            "rolling_treshold/2",
            "savings_treshold/2",
            "socialCreds/1",
            "support_matrix/4",
        ]
    );
    assert_eq!(system.branches().len(), 18 + 7 + 8);
    assert_eq!(system.layout().child_slots(), 10);
}

#[test]
fn percent_support_clause_has_the_rows_of_the_worked_example() {
    let system = energy_policy();
    let branch = system.clause_branches("applySupport", 4).nth(1).unwrap();
    let rows: BTreeSet<&str> = branch
        .rows
        .iter()
        .map(|row| row.description.as_str())
        .collect();
    for expected in [
        "payload(P) = applySupport",
        "arity(P) = 4",
        "children = 2",
        "payload(C0) = is",
        "payload(C1) = is",
        "digest(P.1) = H(percent)",
        "payload(C0.1) = div",
        "payload(C0.1.0) = *",
        "digest(C0.1.1) = H(100)",
        "payload(C1.1) = -",
        "digest(P.0) = digest(C0.1.0.0)   (variable ApplySupportInput)",
        "digest(P.0) = digest(C1.1.0)   (variable ApplySupportInput)",
        "digest(P.2) = digest(C0.1.0.1)   (variable ApplySupportValue)",
        "digest(P.3) = digest(C1.0)   (variable ApplySupportOutput)",
        "digest(C0.0) = digest(C1.1.1)   (variable AmountToBeSubtracted)",
        "tag(C2) = absent",
        "tag(C9) = absent",
    ] {
        assert!(rows.contains(expected), "missing row {expected:?}");
    }
}

struct Node {
    goal: String,
    children: Vec<String>,
}

fn proof_tree_nodes() -> Vec<Node> {
    fn walk(node: &Value, nodes: &mut Vec<Node>) {
        let subtree = node["subtree"].as_array().expect("subtree is a list");
        let children: Vec<&Value> = subtree.iter().filter(|child| child.is_object()).collect();
        nodes.push(Node {
            goal: node["goal"].as_str().expect("goal is a string").to_owned(),
            children: children
                .iter()
                .map(|child| child["goal"].as_str().unwrap().to_owned())
                .collect(),
        });
        for child in children {
            walk(child, nodes);
        }
    }
    let roots: Value = serde_json::from_str(PROOF_TREE).expect("proof tree is JSON");
    let mut nodes = Vec::new();
    for root in roots.as_array().expect("proof tree is a list of roots") {
        walk(root, &mut nodes);
    }
    nodes
}

#[test]
fn every_proof_tree_node_satisfies_exactly_one_branch() {
    let system = energy_policy();
    let nodes = proof_tree_nodes();
    assert_eq!(nodes.len(), 50);

    let mut used = BTreeSet::new();
    for node in &nodes {
        let children: Vec<&str> = node.children.iter().map(String::as_str).collect();
        let honest = assignment(&system, &node.goal, &children);
        let branches = satisfied(&system, &honest);
        assert_eq!(branches.len(), 1, "{}: {branches:?}", node.goal);
        used.insert(branches[0].clone());
    }
    for expected in [
        "clause endPrice/1 #0 alternative 0",
        "clause savingsClass/3 #0 alternative 1",
        "clause consumptionClass/2 #2 alternative 0",
        "clause applySupport/4 #1 alternative 0",
        "clause sumOfMonthlyConsumptions/2 #1 alternative 0",
        "builtin _ is ((_ * _) div _)",
        "builtin _ = _",
        "external fact support_matrix/4",
    ] {
        assert!(
            used.contains(expected),
            "{expected} not used; used: {used:?}"
        );
    }
}

#[test]
fn altered_proof_tree_nodes_satisfy_no_branch() {
    let system = energy_policy();
    let cheats: [(&str, &[&str]); 6] = [
        // The percent clause applied with the wrong result.
        (
            "applySupport(1045800,percent,10,941221)",
            &["104580 is 1045800*10 div 100", "941220 is 1045800-104580"],
        ),
        // `nominal` claimed while the children compute a percentage.
        (
            "applySupport(1045800,nominal,10,941220)",
            &["104580 is 1045800*10 div 100", "941220 is 1045800-104580"],
        ),
        // Children swapped.
        (
            "applySupport(1045800,percent,10,941220)",
            &["941220 is 1045800-104580", "104580 is 1045800*10 div 100"],
        ),
        // A child left out.
        (
            "applySupport(1045800,percent,10,941220)",
            &["104580 is 1045800*10 div 100"],
        ),
        // The class 'mid' proved with the threshold of 'high'.
        (
            "savingsClass(2000,1400,mid)",
            &[
                "savings_treshold(high,500)",
                "600 is 2000-1400",
                "600>500",
                "mid=mid",
            ],
        ),
        // The builtin call written with a different expression.
        ("2000 is 24006 div 12", &["24006 is 2000 * 12"]),
    ];
    for (parent, children) in cheats {
        let cheat = assignment(&system, parent, children);
        assert!(
            satisfied(&system, &cheat).is_empty(),
            "{parent} {children:?} satisfies {:?}",
            satisfied(&system, &cheat)
        );
    }
}

#[test]
fn a_type_error_is_caught_at_the_builtin_node() {
    let system = energy_policy();
    // The clause `rollingConsumption(Sum, Result) :- Result is Sum div 12`
    // does not constrain types, so the parent node still matches it...
    let parent = assignment(
        &system,
        "rollingConsumption(24006,'2000')",
        &["'2000' is 24006 div 12"],
    );
    assert_eq!(
        satisfied(&system, &parent),
        ["clause rollingConsumption/2 #0 alternative 0"]
    );
    // ...but its child, an `is/2` node with an atom result, matches no branch.
    let child = assignment(&system, "'2000' is 24006 div 12", &[]);
    assert!(satisfied(&system, &child).is_empty());
}
