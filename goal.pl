:- use_module(meta_interpreter).
:- use_module(library(http/json)).

:- consult(rules).
:- consult(policy).

% run the proof for ancestor(x,y)
run_proof(Tree) :-
    Goal = ancestor(alice,john),
    prove(Goal, Tree).

% export proof tree as JSON
export_proof(File) :-
    (   run_proof(Tree)
    ->  open(File, write, Stream),
        json_write_dict(Stream, Tree, [width(128), serialize_unknown(true)]),
        close(Stream),
        writeln('Proof found and exported')
    ;   writeln('No proof could be found'),
        open(File, write, Stream),
        json_write_dict(Stream, _{error:"no_proof"}, [width(128)]),
        close(Stream)
    ).

% run automatically on startup
:- initialization(main).

main :-
    export_proof('proof_tree.json'),
    halt.
