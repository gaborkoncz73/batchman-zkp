//! Well-formedness of the opened nodes of an assignment, read from the cells
//! alone.

mod support;

use ff::Field;
use policy_constraints::Fp;
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
    // The cells say f(b), the digest is that of f(c): rows reading the cells
    // and rows comparing the digest would talk about different terms.
    let mut cheat = honest(&system);
    cheat[column(&system, Cell::Digest(at(Slot::Parent, &[1])))] = support::ground("f(c)").digest();
    // P's digest follows the forged one, so only P.1 is inconsistent.
    rehash(&system, &mut cheat, &at(Slot::Parent, &[]));
    assert!(
        error(&system, &cheat).contains("subterm P.1: the digest is not the hash"),
        "{}",
        error(&system, &cheat)
    );
}

#[test]
fn child_slot_digests_are_left_to_the_child() {
    // The same mismatch under a child slot passes here: the slot's cells are
    // linked to the child node's parent-side cells, and the child hashes them
    // (proof_witness::check rejects this cheat at the link).
    let system = system(POLICY);
    let mut cheat = honest(&system);
    cheat[column(&system, Cell::Digest(at(Slot::Child(0), &[])))] =
        support::ground("q(2)").digest();
    system.layout().check_well_formed(&cheat).unwrap();
}

#[test]
fn every_goal_slot_opens_the_same_paths() {
    // Only the head opens P.0 and P.1, only q(H) and r(T, Y) open C0 and C1,
    // yet every slot opens all of them: a child slot has exactly the cells of
    // a parent goal, in the same order, so a body goal can be matched with the
    // node that proves it as one list of cells.
    let system = system(POLICY);
    let layout = system.layout();
    let slots = std::iter::once(Slot::Parent).chain((0..3).map(Slot::Child));
    for slot in slots {
        for node in layout.opened_nodes() {
            let moved = node.position.in_slot(slot);
            assert!(
                layout.column(&Cell::Tag(moved.clone())).is_some(),
                "{} is opened, {moved} is not",
                node.position
            );
        }
    }
    let goal = layout.goal_columns();
    for slot in 0..layout.child_slots() {
        let links = layout.slot_links(slot);
        assert_eq!(links.len(), goal.len());
        for (link, &column) in links.iter().zip(goal) {
            let child_cell = &layout.cells()[link.in_child];
            assert_eq!(link.in_child, column);
            assert_eq!(child_cell.position().unwrap().slot(), Slot::Parent);
            assert_eq!(
                child_cell.position().unwrap().path(),
                link.cell.position().unwrap().path()
            );
            assert_eq!(layout.cells()[link.in_parent], link.cell);
        }
    }
    // Four goal slots of equal size, the child count, the fact cells (q, r and
    // s are external facts) and is_clause.
    assert_eq!(layout.width(), 4 * goal.len() + 4);
    assert_eq!(layout.cells()[layout.is_clause()], Cell::IsClause);
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

#[test]
fn a_goal_alone_is_checked_like_a_parent_goal() {
    // A link between two shards carries a goal's cells without its node: the
    // cells must hash to their digests, so the digest fixes them all.
    let system = system(POLICY);
    let layout = system.layout();
    let goal = layout.goal_of(&honest(&system));
    assert_eq!(
        layout.goal_digest(&goal),
        support::ground("p([1,2], f(b))").digest()
    );
    layout.check_goal(&goal).unwrap();

    let payload = column(&system, Cell::Payload(at(Slot::Parent, &[1])));
    let index = layout
        .goal_columns()
        .iter()
        .position(|&column| column == payload)
        .unwrap();
    let mut forged = goal.clone();
    forged[index] = symbol("g");
    assert_eq!(
        layout.check_goal(&forged).unwrap_err().to_string(),
        "subterm P.1: the digest is not the hash of the other cells"
    );
    assert!(layout.check_goal(&goal[1..]).is_err());
}
