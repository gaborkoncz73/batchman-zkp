% input.pl

% 5 parent edges => depth 6 for query deepwide(alice, dora)
parent(alice, bob).
parent(bob, laura).
parent(laura, jack).
parent(jack, peter).
parent(peter, dora).

% 13 independent checks (meaningful "transaction validation" style).
% We assert them for each X on the chain with the same target Y=dora.

% --- alice -> dora checks ---
tx_check1(alice,dora).   % identity_verified
tx_check2(alice,dora).   % balance_sufficient
tx_check3(alice,dora).   % limit_not_exceeded
tx_check4(alice,dora).   % aml_cleared
tx_check5(alice,dora).   % fraud_check_passed
tx_check6(alice,dora).   % sanctions_cleared
tx_check7(alice,dora).   % geo_restriction_ok
tx_check8(alice,dora).   % velocity_check_ok
tx_check9(alice,dora).   % device_fingerprint_ok
tx_check10(alice,dora).  % session_valid
tx_check11(alice,dora).  % policy_version_ok
tx_check12(alice,dora).  % consent_recorded
tx_check13(alice,dora,[(a,b,c,d)]).  % audit_log_ok

% --- bob -> dora checks ---
tx_check1(bob,dora).  tx_check2(bob,dora).  tx_check3(bob,dora).
tx_check4(bob,dora).  tx_check5(bob,dora).  tx_check6(bob,dora).
tx_check7(bob,dora).  tx_check8(bob,dora).  tx_check9(bob,dora).
tx_check10(bob,dora). tx_check11(bob,dora). tx_check12(bob,dora).
tx_check13(bob,dora,[(a,b,c,d)]).

% --- laura -> dora checks ---
tx_check1(laura,dora).  tx_check2(laura,dora).  tx_check3(laura,dora).
tx_check4(laura,dora).  tx_check5(laura,dora).  tx_check6(laura,dora).
tx_check7(laura,dora).  tx_check8(laura,dora).  tx_check9(laura,dora).
tx_check10(laura,dora). tx_check11(laura,dora). tx_check12(laura,dora).
tx_check13(laura,dora,[(a,b,c,d)]).

% --- jack -> dora checks ---
tx_check1(jack,dora).  tx_check2(jack,dora).  tx_check3(jack,dora).
tx_check4(jack,dora).  tx_check5(jack,dora).  tx_check6(jack,dora).
tx_check7(jack,dora).  tx_check8(jack,dora).  tx_check9(jack,dora).
tx_check10(jack,dora). tx_check11(jack,dora). tx_check12(jack,dora).
tx_check13(jack,dora,[(a,b,c,d)]).

% --- peter -> dora checks ---
tx_check1(peter,dora).  tx_check2(peter,dora).  tx_check3(peter,dora).
tx_check4(peter,dora).  tx_check5(peter,dora).  tx_check6(peter,dora).
tx_check7(peter,dora).  tx_check8(peter,dora).  tx_check9(peter,dora).
tx_check10(peter,dora). tx_check11(peter,dora). tx_check12(peter,dora).
tx_check13(peter,dora,[(a,b,c,d)]).
