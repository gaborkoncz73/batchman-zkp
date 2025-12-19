#!/usr/bin/swipl -q

:- use_module('./meta_interpreter.pl').
:- use_module('./knowledge_base.pl').
:- use_module(library(http/json)).
:- use_module('./policy.pl').
:- use_module('./matrix.pl').
:- use_module('./input.pl').
:- initialization write_proof.



write_kb :-
    /* debug
    ,trace */
    knowledge_base_dict(KB)
    ,json_write_dict(current_output, KB,[width(100),serialize_unknown(true)])
    ,halt
    .


write_proof :-
    prove(endPrice(931220), [Tree]),
    open('input/proof_tree.json', write, Stream),           % fájl megnyitása írásra
    json_write_dict(Stream, [Tree], [width(100), serialize_unknown(true)]),
    close(Stream),                                    % fájl lezárása
    halt.


% simply print the proof tree to the terminal
print_proof_tree(A):-
    write("Proof tree for: "), write(A), nl,
    prove(A, [Tree]),
    write("Proof tree: "), write(Tree), nl.

% print a tree representation of the proof tree to the terminal
print_tree_pretty(Tree):-
    print_tree_pretty(Tree, 0).
print_tree_pretty([true], Indent):-
    tab(Indent),
    write("< "),
    write(true), nl.
print_tree_pretty([{goal:Goal,goal_unification:U,substitution:S,subtree:Children}], Indent):-
    tab(Indent),
    write( ":> "),
    write(Goal),
    write(" <> "),
    write(U),
    write(" >< "),
    write(S),
    nl,
    NewIndent is Indent + 4,
    print_children_pretty(Children, NewIndent).

print_children_pretty([], _).
print_children_pretty([Child|OtherChildren], Indent):-
    print_tree_pretty([Child], Indent),
    print_children_pretty(OtherChildren, Indent).