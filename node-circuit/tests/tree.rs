//! The tree circuit (5a) on small hand-made trees: for every honest tree and
//! every cheat, the circuit decides like proof-witness' reference check.

use ff::Field;
use halo2_poseidon::poseidon::primitives::P128Pow5T3;
use halo2_proofs::dev::MockProver;
use node_circuit::{BranchTable, Claim, NodeBranch, TreeCircuit, TreeNode, minimum_k};
use policy_constraints::term::fact_commitment;
use policy_constraints::{Cell, ConstraintSystem, Fp, GroundTerm, Position, Slot, compile};
use policy_parser::{parse_goal, parse_policy};
use proof_witness::{NodeWitness, ProofWitness, TrustedRoots, check};

/// `p` has a circular clause next to an honest one, `s` asks for the same
/// fact twice.
const LOOPS: &str = "r :- p.\np :- p.\np :- q(1).\ns :- q(1), q(1).\n";

/// Node places of every circuit here; the rest is padding.
const CAPACITY: usize = 6;

const SHUFFLE: &str = "link: the body goals are the nodes' goals";

type Node = (&'static str, &'static [&'static str], &'static str);

const R: Node = ("r", &["p"], "clause r/0 #0 alternative 0");
const P_LOOP: Node = ("p", &["p"], "clause p/0 #0 alternative 0");
const P_FACT: Node = ("p", &["q(1)"], "clause p/0 #1 alternative 0");
const Q: Node = ("q(1)", &[], "external fact q/1");
const S: Node = ("s", &["q(1)", "q(1)"], "clause s/0 #0 alternative 0");

fn system() -> ConstraintSystem {
    compile(&parse_policy(LOOPS, "loops.pl").unwrap()).unwrap()
}

fn ground(goal: &str) -> GroundTerm {
    GroundTerm::from_term(&parse_goal(goal, "goal").unwrap()).unwrap()
}

/// A tree to check: its nodes (the first is the root) with whether each is
/// active, the trusted root, and a cheat applied to the assignments.
struct Case {
    nodes: Vec<(Node, bool)>,
    trusted: &'static str,
    tamper: fn(&ConstraintSystem, &mut [Vec<Fp>]),
}

fn case(nodes: &[Node]) -> Case {
    Case {
        nodes: nodes.iter().map(|&node| (node, true)).collect(),
        trusted: nodes[0].0,
        tamper: |_, _| {},
    }
}

impl Case {
    fn inactive(mut self, index: usize) -> Self {
        self.nodes[index].1 = false;
        self
    }

    fn trusting(mut self, goal: &'static str) -> Self {
        self.trusted = goal;
        self
    }

    fn tampered(mut self, tamper: fn(&ConstraintSystem, &mut [Vec<Fp>])) -> Self {
        self.tamper = tamper;
        self
    }

    /// The reference check's verdict (on the active nodes) and the circuit's
    /// failures (none if it is satisfied).
    fn run(&self) -> (Result<(), String>, Vec<String>) {
        let system = system();
        let branch = |label: &str| {
            system
                .branches()
                .iter()
                .find(|branch| branch.kind.to_string() == label)
                .unwrap_or_else(|| panic!("no branch {label}"))
                .id
        };
        let mut assignments: Vec<Vec<Fp>> = self
            .nodes
            .iter()
            .map(|((goal, body, label), _)| {
                let body: Vec<GroundTerm> = body.iter().map(|goal| ground(goal)).collect();
                let mut w = system.layout().assign(&ground(goal), &body).unwrap();
                system.complete(branch(label), &mut w, Fp::ZERO).unwrap();
                w
            })
            .collect();
        (self.tamper)(&system, &mut assignments);

        let trusted = TrustedRoots::new(&[self.trusted], "roots").unwrap();
        let published = vec![fact_commitment(ground("q(1)").digest(), Fp::ZERO)];
        let roots = vec![ground(self.trusted).digest()];

        // The reference sees the active nodes, the root first.
        let witness = ProofWitness {
            nodes: self
                .nodes
                .iter()
                .zip(&assignments)
                .filter(|((_, active), _)| *active)
                .map(|(((goal, _, label), _), w)| NodeWitness {
                    text: (*goal).to_owned(),
                    goal: ground(goal),
                    parent: None,
                    children: Vec::new(),
                    branch: branch(label),
                    assignment: w.clone(),
                })
                .collect(),
            roots: vec![0],
        };
        let reference = check(&system, &witness, &trusted, &published, Fp::from(7))
            .map_err(|error| format!("{error:#}"));

        // The circuit sees every node, padded with inactive copies of the root.
        let mut nodes: Vec<TreeNode<Fp>> = self
            .nodes
            .iter()
            .zip(&assignments)
            .map(|(((_, _, label), active), w)| TreeNode {
                branch: NodeBranch {
                    assignment: w.clone(),
                    claim: Claim::Branch(branch(label)),
                },
                active: *active,
            })
            .collect();
        let padding = TreeNode {
            active: false,
            ..nodes[0].clone()
        };
        nodes.resize(CAPACITY, padding);
        let circuit = TreeCircuit::<Fp, P128Pow5T3>::new(
            system.layout().clone(),
            BranchTable::from_policy(&system),
            1,
            nodes,
        );
        let failures = match MockProver::run(
            minimum_k(&circuit),
            &circuit,
            vec![published, roots, Vec::new()],
        )
        .unwrap()
        .verify()
        {
            Ok(()) => Vec::new(),
            Err(failures) => failures.iter().map(ToString::to_string).collect(),
        };
        (reference, failures)
    }

    /// Both accept.
    fn passes(&self) {
        let (reference, failures) = self.run();
        reference.unwrap();
        assert!(failures.is_empty(), "{failures:#?}");
    }

    /// Both reject: the reference with `reason`, the circuit with a failure
    /// that mentions `failure`.
    fn fails(&self, reason: &str, failure: &str) {
        let (reference, failures) = self.run();
        let error = reference.expect_err("the reference rejects it");
        assert!(error.contains(reason), "{error}");
        assert!(
            failures.iter().any(|found| found.contains(failure)),
            "no {failure:?} in {failures:#?}"
        );
    }
}

#[test]
fn honest_trees_pass_in_any_order() {
    case(&[R, P_FACT, Q]).passes();
    case(&[R, Q, P_FACT]).passes();
    case(&[S, Q, Q]).passes();
}

#[test]
fn a_circular_proof_cannot_reach_the_root() {
    case(&[R, P_LOOP]).fails("no node proves the body goal", SHUFFLE);
    case(&[R, P_LOOP, P_LOOP]).fails("no node proves the body goal", SHUFFLE);
}

#[test]
fn a_detached_cycle_is_harmless() {
    case(&[R, P_FACT, Q, P_LOOP]).passes();
}

#[test]
fn one_node_cannot_prove_two_body_goals() {
    case(&[S, Q]).fails("no node proves the body goal in child slot 1", SHUFFLE);
}

#[test]
fn every_active_node_proves_a_body_goal() {
    case(&[R, P_FACT, Q, Q]).fails("proves a goal that no body goal uses", SHUFFLE);
    // Marked inactive, the extra node is padding.
    case(&[R, P_FACT, Q, Q]).inactive(3).passes();
}

#[test]
fn a_needed_node_cannot_be_switched_off() {
    case(&[R, P_FACT, Q])
        .inactive(2)
        .fails("no node proves the body goal", SHUFFLE);
}

#[test]
fn a_forged_body_goal_is_no_node_goal() {
    // r's branch fixes the name of its body goal, not its digest; a node
    // hashes none of its slots. The forged digest matches no node's goal.
    case(&[R, P_FACT, Q])
        .tampered(|system, w| {
            let digest = system
                .layout()
                .column(&Cell::Digest(Position::root(Slot::Child(0))))
                .unwrap();
            w[0][digest] += Fp::ONE;
        })
        .fails("no node proves the body goal in child slot 0", SHUFFLE);
}

#[test]
fn the_root_must_be_the_trusted_goal() {
    case(&[R, P_FACT, Q]).trusting("s").fails(
        "proves a goal that is not trusted",
        // The root's goal digest against the trusted list.
        "not satisfied by cell (Column('Instance', 1 - ), outside any region, on row 0)",
    );
}

#[test]
fn the_root_must_be_proved_by_a_clause() {
    // is_clause against the constant 1.
    case(&[Q]).fails(
        "is proved by external fact q/1, not by a clause",
        "Equality constraint not satisfied by cell (Column('Fixed'",
    );
}

mod energy {
    //! The repository's whole proof tree in one tree circuit.

    use super::*;
    use proof_witness::{FactRegistry, ProofTree, build, parse_commitments};

    const POLICY: &str = include_str!("../../prolog/policy.pl");
    const TREE: &str = include_str!("../../input/proof_tree.json");
    const FACTS: &str = include_str!("../../registry/facts.yaml");
    const ROOTS: &str = include_str!("../../input/trusted_roots.json");
    const COMMITMENTS: &str = include_str!("../../registry/commitments.json");

    #[test]
    fn the_whole_tree_is_one_proof() {
        let system = compile(&parse_policy(POLICY, "policy.pl").unwrap()).unwrap();
        let roots = TrustedRoots::from_json(ROOTS, "trusted_roots.json").unwrap();
        let witness = build(
            &system,
            &ProofTree::from_json(TREE, "proof_tree.json").unwrap(),
            &FactRegistry::from_yaml(FACTS, "facts.yaml").unwrap(),
            &roots,
        )
        .unwrap();
        let circuit =
            TreeCircuit::<Fp, P128Pow5T3>::from_witness(&system, &witness, &roots, 64).unwrap();
        assert_eq!(circuit.capacity(), 64);
        let instances = vec![
            parse_commitments(COMMITMENTS, "commitments.json").unwrap(),
            node_circuit::root_digests(&roots),
            Vec::new(),
        ];
        let prover = MockProver::run(minimum_k(&circuit), &circuit, instances).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }
}
