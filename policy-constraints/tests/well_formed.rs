//! Well-formedness of the opened nodes of an assignment, read from the cells
//! alone.

mod support;

use ff::Field;
use halo2_proofs::pasta::Fp;
use policy_constraints::term::{hash_node, symbol};
use policy_constraints::{Cell, ConstraintSystem, MAX_ARITY, Position, Slot, TermTag};
use support::{assignment, system};

/// The layout opens a cons cell (`P.0`), a compound (`P.1`) and an atom goal
/// (`C2`).
const POLICY: &str = "p([H|T], f(Y)) :- q(H), r(T, Y), s.\n";

fn honest(system: &ConstraintSystem) -> Vec<Fp> {
    assignment(system, "p([1,2], f(b))", &["q(1)", "r([2], b)", "s"])
}

fn column(system: &ConstraintSystem, cell: Cell) -> usize {
    system
        .layout()
        .column(&cell)
        .expect("cell is in the layout")
}

fn at(slot: Slot, path: &[usize]) -> Position {
    path.iter().fold(Position::root(slot), |position, &index| {
        position.child(index)
    })
}

fn error(system: &ConstraintSystem, assignment: &[Fp]) -> String {
    system
        .layout()
        .check_well_formed(assignment)
        .expect_err("the assignment is rejected")
        .to_string()
}

/// Recomputes the digests of the node at `position` and of its ancestors from
/// their other cells, as a cheating prover can, so that only the targeted
/// rule is broken.
fn rehash(system: &ConstraintSystem, assignment: &mut [Fp], position: &Position) {
    let mut chain: Vec<Position> = position.ancestors().collect();
    chain.push(position.clone());
    for position in chain.iter().rev() {
        let node = system
            .layout()
            .opened_nodes()
            .iter()
            .find(|node| &node.position == position)
            .expect("node is opened");
        let children = node.children.map(|column| assignment[column]);
        assignment[node.digest] = hash_node(
            assignment[node.tag],
            assignment[node.payload],
            assignment[node.arity],
            &children,
        );
    }
}

#[test]
fn honest_assignments_are_well_formed() {
    let system = system(POLICY);
    system.layout().check_well_formed(&honest(&system)).unwrap();
    // Fewer children than slots and a goal with fewer arguments: the unused
    // positions are absent and zero.
    let short = assignment(&system, "q(1)", &[]);
    assert_eq!(
        short[column(&system, Cell::Tag(at(Slot::Child(2), &[])))],
        Fp::ZERO
    );
    system.layout().check_well_formed(&short).unwrap();
}

#[test]
fn a_digest_that_does_not_match_its_cells_is_rejected() {
    let system = system(POLICY);
    // The payload says q(1), the digest is that of q(2): rows comparing the
    // payload and rows linking the digest would talk about different goals.
    let mut cheat = honest(&system);
    cheat[column(&system, Cell::Digest(at(Slot::Child(0), &[])))] =
        support::ground("q(2)").digest();
    assert!(
        error(&system, &cheat).contains("subterm C0: the digest is not the hash"),
        "{}",
        error(&system, &cheat)
    );
}

#[test]
fn an_absent_node_must_be_all_zero() {
    let system = system(POLICY);
    let mut cheat = assignment(&system, "q(1)", &[]);
    cheat[column(&system, Cell::Payload(at(Slot::Child(0), &[])))] = symbol("q");
    assert_eq!(
        error(&system, &cheat),
        "subterm C0: the node is absent, but not all of its cells are zero"
    );
}

#[test]
fn the_tag_must_be_a_term_tag() {
    let system = system(POLICY);
    let mut cheat = honest(&system);
    let position = at(Slot::Parent, &[1]);
    cheat[column(&system, Cell::Tag(position.clone()))] = Fp::from(9);
    assert!(error(&system, &cheat).contains("subterm P.1: the tag is not a term tag"));
}

#[test]
fn the_arity_must_fit_the_tag() {
    let system = system(POLICY);
    let position = at(Slot::Child(2), &[]);

    // The atom goal s claims one argument.
    let mut cheat = honest(&system);
    cheat[column(&system, Cell::Arity(position.clone()))] = Fp::ONE;
    rehash(&system, &mut cheat, &position);
    assert!(
        error(&system, &cheat).contains("subterm C2: a atom node must have arity 0..=0"),
        "{}",
        error(&system, &cheat)
    );

    // The atom goal s is turned into a compound s/0.
    let mut cheat = honest(&system);
    cheat[column(&system, Cell::Tag(position.clone()))] = TermTag::Compound.field();
    rehash(&system, &mut cheat, &position);
    assert!(
        error(&system, &cheat)
            .contains(&format!("a compound node must have arity 1..={MAX_ARITY}")),
        "{}",
        error(&system, &cheat)
    );
}

#[test]
fn a_cons_cell_has_no_payload() {
    let system = system(POLICY);
    let position = at(Slot::Parent, &[0]);
    let mut cheat = honest(&system);
    cheat[column(&system, Cell::Payload(position.clone()))] = Fp::ONE;
    rehash(&system, &mut cheat, &position);
    assert!(error(&system, &cheat).contains("subterm P.0: a cons node must have payload zero"));
}

#[test]
fn children_must_match_the_arity() {
    let system = system(POLICY);
    let position = at(Slot::Parent, &[]);

    // p/2 with a third argument.
    let mut cheat = honest(&system);
    cheat[column(&system, Cell::Digest(at(Slot::Parent, &[2])))] = support::ground("c").digest();
    rehash(&system, &mut cheat, &position);
    assert!(
        error(&system, &cheat).contains("subterm P: child 2 is present, but the arity is 2"),
        "{}",
        error(&system, &cheat)
    );

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
    assert!(
        error(&system, &cheat).contains("subterm P: child 1 is missing, but the arity is 2"),
        "{}",
        error(&system, &cheat)
    );
}

#[test]
fn the_assignment_must_have_the_layout_width() {
    let system = system(POLICY);
    let mut short = honest(&system);
    short.pop();
    assert!(error(&system, &short).contains("cells, the layout has"));
}
