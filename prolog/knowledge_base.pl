:- module(knowledge_base, [knowledge_base_dict/1]).

:- use_module(library(http/json)).   % If you want JSON output in SWI-Prolog
:- use_module(['./policy.pl', 
               './matrix.pl',
               './input.pl'],[]).

%%----------------------------------------------------------------------------
%% 1) Top-level: produce a full knowledge base dictionary
%%----------------------------------------------------------------------------

modules([policy, policy_matrix, policy_input]).

knowledge_base_dict(KBDict) :-
    findall(ModuleDict,
		(
			% 1. Gather all relevant predicates from our module
			modules(Modules),
			member(Module, Modules),
			findall((Name/Arity, P),
        		( 
					current_predicate(Name/Arity),
          			functor(P, Name, Arity),
          			predicate_property(P, imported_from(Module))
        		),				
        		Predicates),
			% 2. Convert each predicate to a dictionary
    		maplist(predicate_props, Predicates, PredicateDicts),
    		% 3. Build a dictionary for the module
			ModuleDict = _{
				module_name : Module,
				predicates  : PredicateDicts
				}),
		ModuleDicts),

    KBDict = _{ modules: ModuleDicts }.

%%----------------------------------------------------------------------------
%% 2) Per-predicate: gather properties & clauses
%%----------------------------------------------------------------------------

predicate_props((Name/Arity, P), PredDict) :-
    % gather built-in properties if you like
    findall(Prop, predicate_property(P, Prop), Props),
    findall(ClauseDict,
        ( nth_clause(P, Index, ClauseRef),
          clause_props(ClauseRef, Index, P, ClauseDict)
        ),
        ClauseDicts),
    PredDict = _{
        predicate_name : Name,
        arity          : Arity,
        clauses        : ClauseDicts
        /* properties     : Props, */
    }.

%%----------------------------------------------------------------------------
%% 3) Per-clause: retrieve head and body structure
%%----------------------------------------------------------------------------

clause_props(ClauseRef, Index, _Pred, ClauseDict) :-
    % gather any additional clause properties
    findall(Prop, clause_property(ClauseRef, Prop), ClauseProps),
    % retrieve the actual Head/Body from the clause
    clause(Head, Body, ClauseRef),
    % numbervars so we can see variable identity (via $VAR(N))
    % This modifies Head/Body in place:
    numbervars((Head,Body), 0, _MaxVar, [attvar(bind)]),

    % convert body to a top-level list of goals

    % get structured info for each
    term_props(Head, HeadDict),
    body_to_tree(b,Body, Tree),
    tree_to_dict(Tree, BodyDict),
    ClauseDict = _{
       index     : Index,
       head      : HeadDict,
       body      : BodyDict
       /* properties: ClauseProps, */
    }.

%%----------------------------------------------------------------------------
%% 4) Decompose a Body into sub-goals 
%%----------------------------------------------------------------------------

% body_to_tree(+Body, -Tree)
% --------------------------------------------
%  Decompose a clause Body into a parse tree that captures
%  conjunction, disjunction, implication, negation, cut
%  then convert the parse tree into a nested dictionary.

body_to_tree(n,(A, B), op(and, Goals)) :-
    !,
    body_to_tree(n,A, TreeA),
    body_to_tree(n,B, TreeB),
    ( TreeA = op(and, AGoals) ->
        CombinedA = AGoals
      ; CombinedA = [TreeA]
    ),
    ( TreeB = op(and, BGoals) ->
        CombinedB = BGoals
      ; CombinedB = [TreeB]
    ),
    append(CombinedA, CombinedB, Goals).
body_to_tree(n,(A ; B), op(or, Goals)) :-
    !,
    body_to_tree(n,A, TreeA),
    body_to_tree(n,B, TreeB),
    ( TreeA = op(or, AGoals) ->
        CombinedA = AGoals
      ; CombinedA = [TreeA]
    ),
    ( TreeB = op(or, BGoals) ->
        CombinedB = BGoals
      ; CombinedB = [TreeB]
    ),
    append(CombinedA, CombinedB, Goals).

body_to_tree(b,(A, B), op(and, [TreeA, TreeB])) :-
    !,
    body_to_tree(b,A, TreeA),
    body_to_tree(b,B, TreeB).

body_to_tree(b,(A; B), op(or, [TreeA, TreeB])) :-
    !,
    body_to_tree(b,A, TreeA),
    body_to_tree(b,B, TreeB).

body_to_tree(Arity,(A -> B), op(implication, [TreeA, TreeB])) :-
    !,
    body_to_tree(Arity,A, TreeA),
    body_to_tree(Arity,B, TreeB).

body_to_tree(Arity,\+ A, op(negation, [TreeA])) :-
    !,
    body_to_tree(Arity,A, TreeA).

body_to_tree(_Arity,!, op(cut, [])) :-
    !.
body_to_tree(_Arity,A, op(goal, [A])).

%% tree_to_dict(+Tree, -Dict)
%% --------------------------------------------
%%  Convert the operator-based parse tree into a nested dictionary
%%  ready for JSON output.

tree_to_dict(op(and, SubTrees), _{type:"conjunction", subgoals:SubDicts}) :-
    maplist(tree_to_dict, SubTrees, SubDicts).

tree_to_dict(op(or, SubTrees), _{type:"disjunction", subgoals:SubDicts}) :-
    maplist(tree_to_dict, SubTrees, SubDicts).

tree_to_dict(op(implication, [Left, Right]),
    _{ type: "implication",
       subgoals: [DictLeft, DictRight] }) :-
    tree_to_dict(Left, DictLeft),
    tree_to_dict(Right, DictRight).

tree_to_dict(op(negation, [Inner]),
    _{ type: "negation",
       subgoals: [DictInner] }) :-
    tree_to_dict(Inner, DictInner).

tree_to_dict(op(cut, []),
    _{ type: "cut" }).

tree_to_dict(op(goal, [Goal]), Dict) :-
    term_props(Goal, TermDict),
    Dict = _{ type: "goal",
              goal: TermDict }.

%%----------------------------------------------------------------------------
%% 5) Decompose a term into a nested dictionary
%%----------------------------------------------------------------------------

%% If it’s a module:goal, store module name separately.
%% If it’s a variable in numbervar form, store it as such.
%% If it’s atomic, just store it plainly.
%% If it’s compound, store the functor and a list of its arguments.
%% TODO: Handle lists: '[|]'(a,'[|]'(b,'[|]'(c,[])))
%% TODO: Handle empty lists : []

term_props(Term, Dict) :-
    ( Term = Module:Goal -> 
        term_props(Goal, GoalDict),
		Dict = GoalDict.put(_{module:Module})
	; Term = '$VAR'(N)   -> 
        Dict = _{ type: variable, number_in_clause: N }
    ; Term = []          -> 
        Dict = _{ type: atomic, value: "[]" }
	; var(Term)          -> 
        % In principle, if we didn't numbervars, we'd do something else here
        Dict = _{ type: var, value: "un-numbered-var" }
    ; atomic(Term)       -> 
        Dict = _{ type: atomic, value: Term }
    ; compound(Term)     -> 
        compound_name_arguments(Term, Functor, Args),
        maplist(term_props, Args, ArgDicts),
        Dict = _{
           type: compound,
           functor: Functor,
           arguments: ArgDicts
		   /* raw_term: Term */
        }
    ).
