//! The well-formedness chip decides like the reference
//! (`Layout::check_well_formed`): honest assignments pass, and every cheat the
//! reference rejects fails a gate or the shape lookup, in the block of the
//! subterm the reference names.

use ff::Field;
use halo2_poseidon::poseidon::primitives::P128Pow5T3;
use halo2_proofs::dev::MockProver;
use node_circuit::{Shape, WellFormedCircuit, minimum_k};
use policy_constraints::term::{hash_node, symbol};
use policy_constraints::{
    Cell, ConstraintSystem, Fp, GroundTerm, MAX_ARITY, Position, Slot, TermTag, compile,
};
use policy_parser::{parse_goal, parse_policy};

/// Opens a cons cell (`P.0`), a compound (`P.1`) and an atom goal (`C2`).
const POLICY: &str = "p([H|T], f(Y)) :- q(H), r(T, Y), s.\n";

fn system(source: &str) -> ConstraintSystem {
    compile(&parse_policy(source, "test.pl").unwrap()).unwrap()
}

fn ground(goal: &str) -> GroundTerm {
    GroundTerm::from_term(&parse_goal(goal, "goal").unwrap()).unwrap()
}

fn assignment(system: &ConstraintSystem, parent: &str, children: &[&str]) -> Vec<Fp> {
    let children: Vec<GroundTerm> = children.iter().map(|goal| ground(goal)).collect();
    system.layout().assign(&ground(parent), &children).unwrap()
}

fn honest(system: &ConstraintSystem) -> Vec<Fp> {
    assignment(system, "p([1,2], f(b))", &["q(1)", "r([2], b)", "s"])
}

fn at(slot: Slot, path: &[usize]) -> Position {
    path.iter().fold(Position::root(slot), |position, &index| {
        position.child(index)
    })
}

fn column(system: &ConstraintSystem, cell: Cell) -> usize {
    system.layout().column(&cell).unwrap()
}

/// Recomputes the digests of `position` and its ancestors from their cells,
/// as a cheating prover can, so that only the targeted rule breaks.
fn rehash(system: &ConstraintSystem, w: &mut [Fp], position: &Position) {
    let mut chain: Vec<Position> = position.ancestors().collect();
    chain.push(position.clone());
    for position in chain.iter().rev() {
        let node = system
            .layout()
            .opened_nodes()
            .iter()
            .find(|node| &node.position == position)
            .unwrap();
        let children = node.children.map(|column| w[column]);
        w[node.digest] = hash_node(w[node.tag], w[node.payload], w[node.arity], &children);
    }
}

/// The failures of the circuit over these nodes, as MockProver prints them.
fn circuit_failures(system: &ConstraintSystem, nodes: Vec<Vec<Fp>>) -> Vec<String> {
    let circuit = WellFormedCircuit::<Fp, P128Pow5T3>::new(system.layout().clone(), nodes);
    let k = minimum_k(&circuit);
    match MockProver::run(k, &circuit, vec![]).unwrap().verify() {
        Ok(()) => Vec::new(),
        Err(failures) => failures.iter().map(ToString::to_string).collect(),
    }
}

/// The reference and the circuit both reject `w`; every circuit failure is in
/// the block of `subterm`, and one of them is `expected`.
fn both_reject(system: &ConstraintSystem, w: Vec<Fp>, subterm: &str, expected: &str) {
    let reference = system
        .layout()
        .check_well_formed(&w)
        .expect_err("the reference rejects");
    assert!(
        reference
            .to_string()
            .contains(&format!("subterm {subterm}:")),
        "{reference}"
    );
    let failures = circuit_failures(system, vec![w]);
    let region = format!("'subterm {subterm}'");
    assert!(
        !failures.is_empty(),
        "the circuit accepts what the reference rejects: {reference}"
    );
    assert!(
        failures.iter().all(|failure| failure.contains(&region)),
        "{failures:#?}"
    );
    assert!(
        failures.iter().any(|failure| failure.contains(expected)),
        "{failures:#?}"
    );
}

const SHAPE: &str = "subterm shape";
const FLAGS: &str = "subterm cells follow the shape flags";
const HASH: &str = "parent-side digest is the hash";

#[test]
fn the_table_allows_exactly_the_shapes_of_the_reference() {
    assert_eq!(Shape::all().len(), 9);
    for tag in 0..8_u64 {
        for arity in 0..=MAX_ARITY as u64 + 1 {
            let allowed = TermTag::from_field(Fp::from(tag)).is_some_and(|tag| {
                let (smallest, largest) = tag.arities();
                (smallest as u64..=largest as u64).contains(&arity)
            });
            assert_eq!(
                Shape::of(Fp::from(tag), Fp::from(arity)).is_some(),
                allowed,
                "tag {tag}, arity {arity}"
            );
        }
    }
}

#[test]
fn honest_assignments_are_accepted() {
    let system = system(POLICY);
    // Fewer children than slots and missing arguments: absent, all zero.
    let short = assignment(&system, "q(1)", &[]);
    assert_eq!(
        circuit_failures(&system, vec![honest(&system), short]),
        Vec::<String>::new()
    );
}

#[test]
fn a_parent_side_digest_must_be_the_hash() {
    let system = system(POLICY);
    let mut cheat = honest(&system);
    cheat[column(&system, Cell::Digest(at(Slot::Parent, &[1])))] = ground("f(c)").digest();
    rehash(&system, &mut cheat, &at(Slot::Parent, &[]));
    both_reject(&system, cheat, "P.1", HASH);
}

#[test]
fn child_side_digests_are_left_to_the_child() {
    let system = system(POLICY);
    let mut cheat = honest(&system);
    cheat[column(&system, Cell::Digest(at(Slot::Child(0), &[])))] = ground("q(2)").digest();
    system.layout().check_well_formed(&cheat).unwrap();
    assert_eq!(circuit_failures(&system, vec![cheat]), Vec::<String>::new());
}

#[test]
fn an_absent_node_must_be_all_zero() {
    let system = system(POLICY);
    let mut cheat = assignment(&system, "q(1)", &[]);
    cheat[column(&system, Cell::Payload(at(Slot::Child(0), &[])))] = symbol("q");
    both_reject(&system, cheat, "C0", FLAGS);

    // q(1) has no second argument: P.1 is absent, and a payload there is
    // caught on the parent side too (P's hash does not cover P.1's payload).
    let mut cheat = assignment(&system, "q(1)", &[]);
    cheat[column(&system, Cell::Payload(at(Slot::Parent, &[1])))] = symbol("b");
    both_reject(&system, cheat, "P.1", FLAGS);
}

#[test]
fn the_tag_must_be_a_term_tag() {
    let system = system(POLICY);
    let position = at(Slot::Parent, &[1]);
    let mut cheat = honest(&system);
    cheat[column(&system, Cell::Tag(position.clone()))] = Fp::from(9);
    rehash(&system, &mut cheat, &position);
    both_reject(&system, cheat, "P.1", SHAPE);
}

#[test]
fn the_arity_must_fit_the_tag() {
    let system = system(POLICY);
    let position = at(Slot::Child(2), &[]);

    // The atom goal s claims one argument.
    let mut cheat = honest(&system);
    cheat[column(&system, Cell::Arity(position.clone()))] = Fp::ONE;
    both_reject(&system, cheat, "C2", SHAPE);

    // The atom goal s is turned into a compound s/0.
    let mut cheat = honest(&system);
    cheat[column(&system, Cell::Tag(position))] = TermTag::Compound.field();
    both_reject(&system, cheat, "C2", SHAPE);
}

#[test]
fn a_cons_cell_has_no_payload() {
    let system = system(POLICY);
    let position = at(Slot::Parent, &[0]);
    let mut cheat = honest(&system);
    cheat[column(&system, Cell::Payload(position.clone()))] = Fp::ONE;
    rehash(&system, &mut cheat, &position);
    both_reject(&system, cheat, "P.0", FLAGS);
}

#[test]
fn children_must_match_the_arity() {
    let system = system(POLICY);
    let position = at(Slot::Parent, &[]);

    // p/2 with a third argument.
    let mut cheat = honest(&system);
    cheat[column(&system, Cell::Digest(at(Slot::Parent, &[2])))] = ground("c").digest();
    rehash(&system, &mut cheat, &position);
    both_reject(&system, cheat, "P", FLAGS);

    // p/2 whose second argument is missing (all of its cells are zero).
    let mut cheat = honest(&system);
    let second = at(Slot::Parent, &[1]);
    for cell in [
        Cell::Digest(second.clone()),
        Cell::Tag(second.clone()),
        Cell::Payload(second.clone()),
        Cell::Arity(second.clone()),
        Cell::Digest(second.child(0)),
    ] {
        cheat[column(&system, cell)] = Fp::ZERO;
    }
    rehash(&system, &mut cheat, &position);
    both_reject(&system, cheat, "P", FLAGS);
}
