//! The reference check: honest witnesses pass, and every kind of tampering
//! with the assignments is caught by the rule that is meant to catch it.

use ff::Field;
use policy_constraints::Fp;
use policy_constraints::arith::StepCell;
use policy_constraints::term::{fact_commitment, symbol};
use policy_constraints::{BranchKind, Cell, ConstraintSystem, GroundTerm, Position, Slot, compile};
use policy_parser::{parse_goal, parse_policy};
use proof_witness::{
    FactRegistry, NodeWitness, ProofTree, ProofWitness, TrustedRoots, build, check,
    parse_commitments,
};

const POLICY: &str = include_str!("../../prolog/policy.pl");
const TREE: &str = include_str!("../../input/proof_tree.json");
const FACTS: &str = include_str!("../../registry/facts.yaml");
const COMMITMENTS: &str = include_str!("../../registry/commitments.json");
const ROOTS: &str = include_str!("../../input/trusted_roots.json");

struct Energy {
    system: ConstraintSystem,
    witness: ProofWitness,
    roots: TrustedRoots,
    commitments: Vec<Fp>,
}

fn energy() -> Energy {
    let system = compile(&parse_policy(POLICY, "policy.pl").unwrap()).unwrap();
    let roots = TrustedRoots::from_json(ROOTS, "trusted_roots").unwrap();
    let witness = build(
        &system,
        &ProofTree::from_json(TREE, "proof_tree").unwrap(),
        &FactRegistry::from_yaml(FACTS, "facts").unwrap(),
        &roots,
    )
    .unwrap();
    Energy {
        system,
        witness,
        roots,
        commitments: parse_commitments(COMMITMENTS, "commitments").unwrap(),
    }
}

/// A fixed stand-in for the verifier's challenge, so that the tests are
/// deterministic.
fn challenge() -> Fp {
    symbol("test challenge")
}

fn ground(goal: &str) -> GroundTerm {
    GroundTerm::from_term(&parse_goal(goal, "goal").unwrap()).unwrap()
}

impl Energy {
    fn check(&self) -> anyhow::Result<()> {
        check(
            &self.system,
            &self.witness,
            &self.roots,
            &self.commitments,
            challenge(),
        )
    }

    fn error(&self) -> String {
        format!("{:#}", self.check().unwrap_err())
    }

    fn column(&self, cell: Cell) -> usize {
        self.system.layout().column(&cell).unwrap()
    }

    /// The node whose goal is written as `text`.
    fn node(&self, text: &str) -> usize {
        self.witness
            .nodes
            .iter()
            .position(|node| node.text == text)
            .unwrap_or_else(|| panic!("no node {text}"))
    }

    /// Replaces the assignment of `node` with an honest assignment of other
    /// goals, keeping its branch and its place in the tree.
    fn reassign(&mut self, node: usize, parent: &str, children: &[&str]) {
        let children: Vec<GroundTerm> = children.iter().map(|goal| ground(goal)).collect();
        let mut assignment = self
            .system
            .layout()
            .assign(&ground(parent), &children)
            .unwrap();
        let branch = self.witness.nodes[node].branch;
        self.system
            .complete(branch, &mut assignment, Fp::ZERO)
            .unwrap();
        self.witness.nodes[node].assignment = assignment;
    }
}

#[test]
fn the_honest_witness_passes() {
    let energy = energy();
    energy.check().unwrap();
    // Any challenge works for an honest witness, even a degenerate one.
    for challenge in [Fp::ZERO, Fp::ONE, -Fp::ONE] {
        check(
            &energy.system,
            &energy.witness,
            &energy.roots,
            &energy.commitments,
            challenge,
        )
        .unwrap();
    }
}

/// For every node and every branch, the compressed branch is zero exactly
/// when all rows hold: one random combination decides like all the rows.
#[test]
fn compression_decides_like_the_rows() {
    let energy = energy();
    let width = energy.system.layout().width();
    let mut holding = 0;
    for branch in energy.system.branches() {
        let weights: Vec<Fp> =
            std::iter::successors(Some(Fp::ONE), |power| Some(*power * challenge()))
                .take(branch.rows.len())
                .collect();
        let compressed = branch.compress(width, &weights);
        for node in &energy.witness.nodes {
            let rows_hold = branch.is_satisfied_by(&node.assignment);
            let compressed_holds = compressed.evaluate(&node.assignment) == Fp::ZERO;
            assert_eq!(
                rows_hold, compressed_holds,
                "{} on {}",
                branch.kind, node.text
            );
            holding += usize::from(rows_hold);
        }
    }
    // Every node satisfies at least its own branch, and most pairs fail.
    assert!(holding >= energy.witness.nodes.len());
    assert!(holding < energy.witness.nodes.len() * energy.system.branches().len() / 2);
}

#[test]
fn a_cell_that_disagrees_with_its_digest_is_rejected() {
    let mut energy = energy();
    // The integer 931220 in endPrice(931220) becomes 1, digest left alone.
    let payload = energy.column(Cell::Payload(Position::root(Slot::Parent).child(0)));
    energy.witness.nodes[0].assignment[payload] = Fp::ONE;
    let error = energy.error();
    assert!(
        error.contains("node 0: subterm P.0: the digest is not the hash"),
        "{error}"
    );
}

#[test]
fn swapping_a_fact_breaks_the_link_to_the_parent() {
    // The leaf claims another support_matrix entry. Unissued, it is caught
    // at the leaf by its commitment.
    let mut energy = energy();
    let leaf = energy.node("support_matrix(low,mid,percent,10)");
    let other = "support_matrix(low,mid,percent,11)";
    energy.reassign(leaf, other, &[]);
    assert_eq!(
        energy.error(),
        format!("node {leaf}: the fact's commitment is not a published one")
    );
    // Even if it were issued, the parent still asks for the original entry,
    // which no node proves now.
    energy
        .commitments
        .push(fact_commitment(ground(other).digest(), Fp::ZERO));
    let parent = energy.witness.nodes[leaf].parent.unwrap();
    let slot = energy.witness.nodes[parent]
        .children
        .iter()
        .position(|&child| child == leaf)
        .unwrap();
    let error = energy.error();
    assert!(
        error.contains(&format!(
            "node {parent}: no node proves the body goal in child slot {slot}"
        )),
        "{error}"
    );
}

#[test]
fn a_child_slot_cell_must_equal_the_cell_of_the_child() {
    // A node hashes none of its child slots. Forging a cell under C0 (here the
    // digest of the monthlyConsumptions list) and leaving the child alone is
    // caught by the matching: no node has that parent goal. (The branch says
    // C1.0 is the same list, so the forgery changes both.)
    let mut energy = energy();
    for slot in [0, 1] {
        let list = energy.column(Cell::Digest(Position::root(Slot::Child(slot)).child(0)));
        energy.witness.nodes[0].assignment[list] += Fp::ONE;
    }
    let error = energy.error();
    assert!(
        error.contains("node 0: no node proves the body goal in child slot 0"),
        "{error}"
    );
}

#[test]
fn a_wrong_branch_choice_is_rejected_with_its_violated_rows() {
    let mut energy = energy();
    let fact = energy
        .system
        .branches()
        .iter()
        .find(|branch| matches!(branch.kind, BranchKind::ExternalFact { .. }))
        .unwrap()
        .id;
    energy.witness.nodes[0].branch = fact;
    let error = energy.error();
    assert!(error.contains("node 0: external fact"), "{error}");
    assert!(
        error.contains("does not hold, violating payload(P) ="),
        "{error}"
    );
}

#[test]
fn the_child_count_cell_is_fixed_by_the_branch() {
    let mut energy = energy();
    let count = energy.column(Cell::ChildCount);
    energy.witness.nodes[0].assignment[count] += Fp::ONE;
    assert!(
        energy.error().contains("node 0: clause endPrice/1")
            && energy.error().contains("violating children = "),
        "{}",
        energy.error()
    );
}

#[test]
fn the_root_must_prove_the_trusted_goal() {
    // An honest-looking root for another end price.
    let mut energy = energy();
    let root = &energy.witness.nodes[0];
    let children: Vec<String> = root
        .children
        .iter()
        .map(|&child| energy.witness.nodes[child].text.clone())
        .collect();
    let children: Vec<&str> = children.iter().map(String::as_str).collect();
    energy.reassign(0, "endPrice(931221)", &children);
    let error = energy.error();
    // The changed price no longer matches the price the last body goal
    // computed.
    assert!(
        error.contains("node 0: clause endPrice/1 #0 alternative 0 does not hold")
            && error.contains("(variable Price)"),
        "{error}"
    );

    let energy = Energy {
        roots: TrustedRoots::new(&["endPrice(931221)"], "roots").unwrap(),
        ..self::energy()
    };
    assert!(
        energy
            .error()
            .contains("root node 0 proves a goal that is not trusted"),
        "{}",
        energy.error()
    );
}

#[test]
fn the_tree_shape_is_not_read() {
    // parent and children are the prover's bookkeeping: the check (and the
    // circuit) only matches body goals with goals.
    let mut energy = energy();
    for node in &mut energy.witness.nodes {
        node.parent = None;
        node.children.clear();
    }
    energy.witness.nodes.reverse();
    let last = energy.witness.nodes.len() - 1;
    energy.witness.roots = vec![last];
    energy.check().unwrap();
}

#[test]
fn a_root_cannot_prove_a_body_goal() {
    // A root exports nothing: marking a child as a root leaves its parent's
    // body goal unproved (and the child is not a trusted goal either).
    let mut energy = energy();
    let child = energy.witness.nodes[0].children[0];
    energy.witness.roots.push(child);
    assert!(
        energy
            .error()
            .contains("node 0: no node proves the body goal in child slot 0"),
        "{}",
        energy.error()
    );
}

#[test]
fn indices_must_exist() {
    let mut energy = energy();
    energy.witness.nodes[0].branch = usize::MAX;
    assert!(energy.error().contains("does not exist"));

    let mut energy = self::energy();
    energy.witness.roots.push(usize::MAX);
    assert!(energy.error().contains("is not a node"));
}

#[test]
fn a_root_proved_by_a_fact_is_rejected() {
    let system = compile(&parse_policy("p :- q(1).\n", "test.pl").unwrap()).unwrap();
    let fact = branch_of(&system, "external fact q/1");
    let goal = ground("q(1)");
    let witness = single(&system, "q(1)", fact, Fp::ZERO);
    let roots = TrustedRoots::new(&["q(1)"], "roots").unwrap();
    let published = [fact_commitment(goal.digest(), Fp::ZERO)];
    let error = format!(
        "{:#}",
        check(&system, &witness, &roots, &published, challenge()).unwrap_err()
    );
    assert!(
        error.contains("root node 0 is proved by external fact q/1"),
        "{error}"
    );
}

/// The id of the branch printed as `label`.
fn branch_of(system: &ConstraintSystem, label: &str) -> usize {
    system
        .branches()
        .iter()
        .find(|branch| branch.kind.to_string() == label)
        .unwrap_or_else(|| panic!("no branch {label}"))
        .id
}

/// A one-node witness proving `text` by `branch`, completed with `salt`.
fn single(system: &ConstraintSystem, text: &str, branch: usize, salt: Fp) -> ProofWitness {
    let goal = ground(text);
    let mut assignment = system.layout().assign(&goal, &[]).unwrap();
    system.complete(branch, &mut assignment, salt).unwrap();
    ProofWitness {
        nodes: vec![NodeWitness {
            text: text.to_owned(),
            assignment,
            goal,
            parent: None,
            children: Vec::new(),
            branch,
        }],
        roots: vec![0],
    }
}

#[test]
fn a_false_sum_is_caught_by_its_step() {
    // `6 is 2+3`: the form is that of `_ is _ + _`, and with z[0] = 6 every row
    // holds — only the step itself objects. (In a whole tree the parent's
    // rows also bind 6, so the cheat would have to reach the trusted root.)
    let system = compile(&parse_policy("p(X) :- X is 2 + 3.\n", "test.pl").unwrap()).unwrap();
    let add = branch_of(&system, "builtin _ is (_ + _)");
    let goal = ground("6 is 2+3");
    let mut assignment = system.layout().assign(&goal, &[]).unwrap();
    assert!(system.complete(add, &mut assignment, Fp::ZERO).is_ok());
    let z = system.layout().step_column(0, StepCell::Z);
    assignment[z] = Fp::from(6);
    let mut witness = single(&system, "5 is 2+3", add, Fp::ZERO);
    witness.nodes[0].assignment = assignment;
    let roots = TrustedRoots::new(&["6 is 2+3"], "roots").unwrap();
    let error = format!(
        "{:#}",
        check(&system, &witness, &roots, &[], challenge()).unwrap_err()
    );
    assert_eq!(
        error,
        "node 0: step 0: add does not hold for x = 2, y = 3, z = 6"
    );
}

#[test]
fn an_unpublished_fact_is_rejected() {
    // q(2) has the form of the fact branch, but only q(1) is published.
    let system = compile(&parse_policy("p :- q(1).\n", "test.pl").unwrap()).unwrap();
    let fact = branch_of(&system, "external fact q/1");
    let witness = single(&system, "q(2)", fact, Fp::from(7));
    let roots = TrustedRoots::new(&["q(2)"], "roots").unwrap();
    let published = [fact_commitment(ground("q(1)").digest(), Fp::from(7))];
    let error = format!(
        "{:#}",
        check(&system, &witness, &roots, &published, challenge()).unwrap_err()
    );
    assert_eq!(
        error,
        "node 0: the fact's commitment is not a published one"
    );
}

#[test]
fn a_fact_with_a_wrong_salt_is_rejected() {
    // monthlyConsumptions(…) is issued, but under another salt.
    let mut energy = energy();
    let node = energy
        .node("monthlyConsumptions([2001,2001,2001,2001,2001,2001,2000,2000,2000,2000,2000,2000])");
    let (_, salt) = energy.system.layout().fact_cells().unwrap();
    energy.witness.nodes[node].assignment[salt] += Fp::ONE;
    assert_eq!(
        energy.error(),
        format!("node {node}: the fact's commitment is not a published one")
    );
}

/// A policy whose `p` has a circular clause next to an honest one, and whose
/// `s` asks for the same fact twice.
const LOOPS: &str = "r :- p.\np :- p.\np :- q(1).\ns :- q(1), q(1).\n";

/// A hand-made witness for [`LOOPS`]: one node per `(goal, body, branch)`,
/// roots as listed. The tree shape is left empty: the check does not read it.
fn loops(nodes: &[(&str, &[&str], &str)], roots: &[usize]) -> Result<(), String> {
    let system = compile(&parse_policy(LOOPS, "loops.pl").unwrap()).unwrap();
    let nodes = nodes
        .iter()
        .map(|&(goal, body, label)| {
            let body: Vec<GroundTerm> = body.iter().map(|goal| ground(goal)).collect();
            let mut assignment = system.layout().assign(&ground(goal), &body).unwrap();
            let branch = branch_of(&system, label);
            system.complete(branch, &mut assignment, Fp::ZERO).unwrap();
            NodeWitness {
                text: goal.to_owned(),
                goal: ground(goal),
                parent: None,
                children: Vec::new(),
                branch,
                assignment,
            }
        })
        .collect();
    let witness = ProofWitness {
        nodes,
        roots: roots.to_vec(),
    };
    let trusted: Vec<&str> = roots
        .iter()
        .map(|&root| witness.nodes[root].text.as_str())
        .collect();
    let trusted = TrustedRoots::new(&trusted, "roots").unwrap();
    let published = [fact_commitment(ground("q(1)").digest(), Fp::ZERO)];
    check(&system, &witness, &trusted, &published, challenge())
        .map_err(|error| format!("{error:#}"))
}

const R: (&str, &[&str], &str) = ("r", &["p"], "clause r/0 #0 alternative 0");
const P_LOOP: (&str, &[&str], &str) = ("p", &["p"], "clause p/0 #0 alternative 0");
const P_FACT: (&str, &[&str], &str) = ("p", &["q(1)"], "clause p/0 #1 alternative 0");
const Q: (&str, &[&str], &str) = ("q(1)", &[], "external fact q/1");
const S: (&str, &[&str], &str) = ("s", &["q(1)", "q(1)"], "clause s/0 #0 alternative 0");

#[test]
fn honest_hand_made_trees_pass() {
    loops(&[R, P_FACT, Q], &[0]).unwrap();
    // The order of the nodes is free.
    loops(&[Q, P_FACT, R], &[2]).unwrap();
    loops(&[S, Q, Q], &[0]).unwrap();
}

#[test]
fn a_circular_proof_cannot_reach_a_root() {
    // r needs p, and p :- p needs p again: every p node brings one more body
    // goal p than it proves, so the counts never balance.
    for nodes in [&[R, P_LOOP][..], &[R, P_LOOP, P_LOOP], &[P_LOOP, R]] {
        let root = nodes.iter().position(|node| node.0 == "r").unwrap();
        let error = loops(nodes, &[root]).unwrap_err();
        assert!(
            error.contains("no node proves the body goal in child slot 0"),
            "{error}"
        );
    }
}

#[test]
fn a_detached_cycle_is_harmless() {
    // A p :- p node that proves its own body goal balances itself. No root
    // reaches it, so it proves nothing about r, which an honest p proves.
    loops(&[R, P_FACT, Q, P_LOOP], &[0]).unwrap();
}

#[test]
fn one_node_cannot_prove_two_body_goals() {
    let error = loops(&[S, Q], &[0]).unwrap_err();
    assert_eq!(
        error,
        "node 0: no node proves the body goal in child slot 1"
    );
}

#[test]
fn every_non_root_node_proves_a_body_goal() {
    // One of the two q(1) nodes is left over; the first one is named.
    let error = loops(&[R, P_FACT, Q, Q], &[0]).unwrap_err();
    assert_eq!(error, "node 2 proves a goal that no body goal uses");
}
