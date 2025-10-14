% --- Alap ---
ancestor(X,Y) :- parent(X,Y).
ancestor(X,Y) :- parent(X,Z), ancestor(Z,Y).

% --- Testvérek, unokatestvérek, nagynéni/nagybácsi ---
sibling(X,Y) :- parent(P,X), parent(P,Y).       % (X \= Y feltételt most nem használjuk)
cousin(X,Y) :- parent(A,X), parent(B,Y), sibling(A,B).
aunt_or_uncle(X,Y) :- sibling(X,P), parent(P,Y).

% --- Leszármazott ---
descendant(X,Y) :- ancestor(Y,X).

% --- „rokon” reláció sok ággal ---
related(X,Y) :- ancestor(X,Y).
related(X,Y) :- ancestor(Y,X).
related(X,Y) :- sibling(X,Y).
related(X,Y) :- cousin(X,Y).
related(X,Y) :- aunt_or_uncle(X,Y).
related(X,Y) :- aunt_or_uncle(Y,X).
related(X,Y) :- parent(X,Z), related(Z,Y).      % terjesztés
