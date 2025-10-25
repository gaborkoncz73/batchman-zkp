:- module(meta_interpreter, [prove/2]).

% --- base case: true ---
prove(true, [true]) :- !.

% --- conjunction (A, B) ---
prove((A, B), Tree) :-
    prove(A, TreeA),
    prove(B, TreeB),
    append([TreeA, TreeB], Tree),
    !.

% --- disjunction (A; B) ---
prove((A; _), Tree) :-
    prove(A, Tree).
prove((_; B), Tree) :-
    prove(B, Tree).

% --- built-in predicates ---
prove(Goal, [state{goal:Goal, subtree:[true]}]) :-
    predicate_property(Goal, built_in),
    !,
    call(Goal).

% --- general rule ---
prove(Goal, [state{goal:Goal, subtree:Tree}]) :-
    clause(Goal, Body),
    prove(Body, Tree).
