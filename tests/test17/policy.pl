% policy.pl

% Mixed benchmark:
% - depth controlled by the parent-chain length
% - width controlled by the number of tx_checkN/2 predicates

deepwide(X, Y) :-
    parent(X, Y),
    tx_check1(X,Y),
    tx_check2(X,Y),
    tx_check3(X,Y),
    tx_check4(X,Y),
    tx_check5(X,Y),
    tx_check6(X,Y),
    tx_check7(X,Y),
    tx_check8(X,Y),
    tx_check9(X,Y),
    tx_check10(X,Y),
    tx_check11(X,Y),
    tx_check12(X,Y),
    tx_check13(X,Y,[(a,b,c)]).

deepwide(X, Y) :-
    parent(X, Z),
    tx_check1(X,Y),
    tx_check2(X,Y),
    tx_check3(X,Y),
    tx_check4(X,Y),
    tx_check5(X,Y),
    tx_check6(X,Y),
    tx_check7(X,Y),
    tx_check8(X,Y),
    tx_check9(X,Y),
    tx_check10(X,Y),
    tx_check11(X,Y),
    tx_check12(X,Y),
    tx_check13(X,Y,[(a,b,c)]),
    deepwide(Z, Y).
