% közös ős
parent(alice, bob).
parent(alice, helen).         % → sibling(bob,helen)

% bob-ág
parent(bob, laura).
parent(laura, john).

% helen-ág
parent(helen, oliver).

% még pár kapcsolat, hogy több út legyen
parent(bob, carol).
parent(helen, mike).
parent(mike, nancy).
