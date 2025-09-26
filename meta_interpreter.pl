:- module(meta_interpreter, [prove/2]).

%  === base case, end(leaf) of the proof tree ===
prove(true, [true]) :- !.

% === conjunction ===
prove((A, B), Tree) :-
    prove(A, TreeA),
    prove(B, TreeB),
    append([TreeA, TreeB], Tree),
    !. % is append part of ISO prolog? If not, its source is just a few clauses

% === disjunciton ===
prove((A; _), Tree) :-
    prove(A, Tree).
prove((_; B), Tree) :-
    prove(B, Tree).

% === handle built-in predicates ===
prove(Goal, [State]) :-
    predicate_property(Goal, built_in), % is this property part of ISO prolog?
    !,
    call(Goal),
    copy_term(Goal, OriginalGoal),
    findall((Var, Value), (member(Var, Goal), Var = Value), Substitution),
    State = state{
        goal:Goal,
        goal_unification:_{goal:OriginalGoal,body:[]},
        substitution:Substitution,
        subtree:[true]
    }.

% === general case ===
prove(Goal, [State]) :-
    % Goal \= true, %  predicate_property(A, built_in) already filters these, but i don't know if its part of ISO prolog
    % Goal \= (_,_),
    % Goal \= (_\=_),
    clause(Goal, Body),
    copy_term((Goal, Body), (OriginalGoal, _OriginalBody)),
    prove(Body, Tree),
    findall((Var, Value), (member(Var, Goal), Var = Value), Substitution),
    extract_predicates(Body,OriginalBodyPredicates),
    getTermDictFromTerm(Goal, GoalDict),
    getTermDictFromTerm(OriginalGoal, _OriginalGoalDict),
%    write("GoalDict: "), write(OriginalGoalDict), nl, %% TODO: Test if pre-eval substitution can be extrected
    State = state{
        goal:Goal,
        goal_term: GoalDict,
        goal_unification:_{
            goal:Goal,
%            goalTerm:OriginalGoalDict,
            body:OriginalBodyPredicates
%            bodyTerm:OriginalBodyPredicatesDict
            },
        substitution:Substitution,
        subtree:Tree
        }.

extract_predicates(Term, PredicatesList) :-
% If Term is a conjunction (A, B)
(Term = (A, B) ->
    extract_predicates(A, PredListA),
    extract_predicates(B, PredListB),
    append(PredListA, PredListB, PredicatesList);

% If Term is a disjunction (A ; B)
Term = (A ; B) ->
    extract_predicates(A, PredListA),
    extract_predicates(B, PredListB),
    append(PredListA, PredListB, PredicatesList);

% If Term is a simple predicate
PredicatesList = [Term]
).

getTermDictFromTerm(Term, TermDict):-
    Term =.. [Name|Args],
    TermDict = _{name:Name, args:Args}.

getTermDictFromTermList([], []).
getTermDictFromTermList([Term|OtherTerms], [TermDict|OtherTermDicts]):-
    getTermDictFromTerm(Term, TermDict),
    getTermDictFromTermList(OtherTerms, OtherTermDicts).