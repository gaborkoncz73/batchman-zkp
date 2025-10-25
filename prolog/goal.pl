:- use_module(meta_interpreter).
:- use_module(library(http/json)).
:- consult(rules).
:- consult(policy).

% --- goal to prove ---
goal(ancestor(alice, john)).

% --- run the proof ---
run_proof(Tree) :-
    goal(Goal),
    prove(Goal, Tree).

% --- export proof tree as pretty JSON ---
export_proof(File) :-
    (   run_proof(Tree)
    ->  setup_call_cleanup(
            open(File, write, Stream),
            json_write_dict(Stream, Tree,
                [width(80), indent(4), serialize_unknown(true)]
            ),
            close(Stream)
        ),
        format("Proof found and exported to '~w'~n", [File])
    ;   setup_call_cleanup(
            open(File, write, Stream),
            json_write_dict(Stream, _{error:"no_proof"},
                [width(80), indent(4)]
            ),
            close(Stream)
        ),
        setup_call_cleanup(
            open(File, write, Stream),
            json_write_dict(Stream, _{error:"no_proof"},
                [width(128), indent(4)]
            ),
            close(Stream)
        ),
        format("No proof found, wrote fallback JSON to '~w'~n", [File])
    ).

% --- auto-run on startup ---
:- initialization(main, main).

main :-
    export_proof('input/proof_tree.json'),
    halt.
