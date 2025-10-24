% --- Alap ---
ancestor(X,Y) :- parent(X,Y).
ancestor(X,Y) :- parent(X,Z), ancestor(Z,Y).
ancestor(X,Y,T) :- parent(X,Z), ancestor(Z,Y), ancestor(T,X).


is_adult(User) :-
    age(User, Age),
    Age >= 18.             % built-in comparison

is_minor(User) :-
    age(User, Age),
    Age < 18.              % built-in comparison

can_access(User, resourceX) :-
    user(User),
    is_adult(User),
    has_role(User, admin).

can_access(User, resourceY) :-
    user(User),
    \+ has_role(User, admin),   % built-in negation
    is_adult(User).


% --- Szabályok ---


% 1️⃣ összeadás (canonical form)
compute_sum(X, Y, Sum) :-
    Sum is (+(X, Y)).

% 2️⃣ szorzás (canonical form)
compute_prod(X, Y, Prod) :-
    Prod is (*(X, Y)).

% 3️⃣ kombinált művelet: (A+B)*C → *(+(A,B),C)
compute_expression(A, B, C, Res) :-
    compute_sum(A, B, S),
    compute_prod(S, C, Res).

% 4️⃣ összehasonlítás kanonikus formában
check_large(A, B) :-
    compute_sum(A, B, S),
    D is (*(S, 2)),
    E is (*(B, 3)),
    >(D, E).

% 5️⃣ összetett szabály: összeadás, szorzás és reláció vegyesen
full_rule(X, Y) :-
    num(X),
    num(Y),
    compute_expression(X, Y, 2, R),
    >=(R, 10),
    check_large(X, Y).