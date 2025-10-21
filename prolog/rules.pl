% --- Alap ---
ancestor(X,Y) :- parent(X,Y).
ancestor(X,Y) :- parent(X,Z), ancestor(A,Y).
ancestor(X,Y,T) :- parent(X,Z), ancestor(Z,Y), ancestor(T,X).
