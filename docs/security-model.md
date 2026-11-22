# Security model of the sharded Halo2 proof system

This note states what the proof system of `policy-constraints`, `proof-witness`
and `node-circuit` proves, under which assumptions, and what it reveals. It
covers the system as of step 5: one proof for a whole proof tree (`K = 1`,
[`TreeCircuit`] without link places) and a tree split into `K` independently
proved shards linked by salted commitments (`K > 1`).

Every argument below has an executable counterpart: the reference check in
`proof-witness` (`check`, `check_sharded`) states the same conditions in plain
Rust, and the tests listed in [Evidence](#evidence) run each attack against
both the reference and the circuit.

## 1. The statement

### Parties and public data

| Party | Provides | Trusted for |
|---|---|---|
| Policy author | the policy `P` (a Prolog program in the supported dialect) | the meaning of the policy |
| Fact issuer | each fact `f` with a secret salt `s_f` to the prover; the list of commitments `Com_f = H(FACT, digest(f), s_f, 0)` to everyone | issuing only true facts |
| Verifier | the trusted root goals `G₁ … G_T` it wants proved | — |
| Prover | the proof(s) | nothing |

The verifier derives the verifying key(s) itself from the public policy with
the public compiler (`policy-constraints`) and the fixed sizes below; it does
not accept keys from the prover.

**Public sizes**, part of the verifying key:

- `R`, the node places of one proof;
- `T`, the number of root places (`T` in the first shard, `0` in the others);
- `B_in` and `B_out`, the incoming and outgoing link places per shard (`0`
  for `K = 1`);
- the layout derived from `P`: for the repository's energy policy, 574 cells
  per node, 50 cells per goal, 10 child slots, 2 arithmetic step slots, 33
  branches of at most 67 rows.

### What an accepted proof establishes

Let `F` be the set of facts whose commitments are in the public list, and let
`P ∪ F` be read as a definite (Horn) program over ground terms, with the
builtins of the dialect (`is/2` over `+`, `-`, `*` and `div`, the comparisons
`<`, `=<`, `>`, `>=`, `=:=`, `=\=`, and `=/2`) interpreted over the integers.

> **Theorem (soundness).** If the verifier accepts, then, except with
> negligible probability and under the assumptions of §3, every trusted root
> `Gᵢ` has a derivation tree in `P ∪ F` whose root step uses a clause of `P`
> and whose builtin steps evaluate without leaving the signed 64-bit range.

For `K > 1` "the verifier accepts" means: every shard proof verifies under its
key (exactly one under the key with `T` root places, the rest under the key
without), all shard proofs use the same fact commitment list, and the
outgoing link commitments of all shards are exactly the incoming ones as
multisets, zeros skipped (`check_links_between`).

Notes on the statement:

- The dialect has no cut, negation, if-then-else or `\=` (they are rejected
  by the compiler), so derivability in the Horn program is exactly success of
  the Prolog query; the proof does not certify a particular SLD search order.
- Arithmetic is integer arithmetic. A step that would leave `[−2⁶³, 2⁶³)` has
  no proof: this loses completeness for huge values, not soundness.
- A root goal proved directly by a fact is not accepted (`is_clause = 1` on
  roots); this mirrors the reference check and is a policy choice.

## 2. How the circuit establishes it

A node is one derivation step: its goal `P` and its body goals `C0 … C9`, all
encoded as cells of one assignment `w`. Per node the circuit checks:

1. **Well-formedness** (`well_formed.rs`). Every opened position is a valid
   term node (tag, arity and payload fit; missing children are zero, present
   ones are not). Every parent-side position is hashed with Poseidon, so
   `digest(P)` fixes all 50 goal cells. Child-slot positions are not hashed
   here; see step 5.
2. **Branch** (`branch.rs`). The node satisfies one branch (a clause
   alternative, a builtin call form, or an external-fact predicate), chosen in
   secret. With challenge `c` the branch matrix compresses to `cv`, with `γ`
   the vector to a number looked up among the branches; then `⟨cv, (w,1)⟩ = 0`.
3. **Arithmetic** (`arith.rs`). Every step slot holds 64-bit integers (8-bit
   limb range checks) and the operation the branch flags holds.
4. **Facts** (`fact.rs`). If the branch is an external fact (`is_fact = 1`,
   fixed by the branch), `H(FACT, digest(P), salt, 0)` is in the public list.
5. **Links** (`link.rs`). Each goal slot is compressed by Horner's rule in a
   challenge `c'` drawn after `w` is committed. A shuffle proves that the
   multiset of `(1, h(P))` over active non-root nodes and incoming links
   equals the multiset of `(1, h(Cₖ))` over present child slots of active
   nodes and outgoing links. The flag `(1, ·)` keeps a zero `h` apart from
   the `(0, 0)` rows. A slot is present exactly when its tag is not absent,
   and the branch fixes the tags of the slots it uses, so a body goal cannot
   be skipped.
6. **Roots**. The first `T` places are active by a constant, prove a clause
   (`is_clause = 1`), and their `digest(P)` is copied to the trusted root
   instance.
7. **Shard links** (`boundary.rs`). An incoming place exports a goal whose
   cells are checked and hashed as in step 1. An outgoing place imports a
   goal, which the shuffle ties to a node that hashes it. Both publish
   `active · H(LINK, digest, salt, 0)`.

## 3. Assumptions

| # | Assumption | Used for |
|---|---|---|
| A1 | Knowledge soundness of Halo2 with the IPA commitment over the Pasta curves (discrete logarithm hardness) | every check |
| A2 | Fiat–Shamir with Blake2b modelled as a random oracle | the challenges `c`, `γ`, `c'` and those inside Halo2 |
| A3 | Collision resistance of Poseidon `P128Pow5T3` (about 128-bit) | digests fix terms; commitments fix digests |
| A4 | The compiler (`policy-parser`, `policy-constraints`) and the circuit code are correct, and the verifier builds its key from them | the circuit means the policy |
| A5 | The fact issuer issues only true facts and keeps salts unpredictable | `F` is the intended fact set |
| A6 | Link and fact salts are uniformly random 254-bit values, fresh per link | hiding of commitments (zero knowledge only) |

The trusted computing base is therefore: the policy text, the compiler, the
`node-circuit` chips, the PSE `halo2_proofs` v0.3.0 and `poseidon-gadget`
libraries, the fact registry, and the verifier's key generation.

## 4. Soundness argument

Write `p ≈ 2²⁵⁴` for the field size, `W = 574` for the node width, `G = 50`
for the goal width, `B = 33` for the branch count and `M = 67` for the largest
branch.

**Lemma 1 (one node).** If a node's checks pass, then its goal `P` and its
present body goals `Cₖ` are ground terms, and `P ← C₀, …` is a ground
instance of the branch's clause alternative, or `P` is a true builtin call of
that branch's form, or `P` is a fact of `F`. This fails with probability at
most `B·(W+1)/p` (the claimed `cv` is not a branch's) plus `M/p` (a violated
row cancels in the dot product).

*Why.* Well-formedness and A3 make `digest(P)` the digest of a unique ground
term whose cells are the ones the rows read. The branch rows encode the
clause head and body pattern, shared variables and constants (compiler,
A4). Arithmetic and fact membership are exact checks. The two error terms
are Schwartz–Zippel bounds, valid because `c` and `γ` are drawn after `w` and
`cv` are committed (A2).

**Lemma 2 (compressed goals bind).** If two goal cell lists of length `G`
compress to the same `h` under `c'`, they are equal, except with probability
`(G−1)/p` per pair. With a union bound over all export/import pairs of a
proof, at most `((R·11 + B_in + B_out)²)·(G−1)/p`, which is below `2⁻²²⁰` for
any realistic `R`.

**Lemma 3 (matching).** In one proof, the shuffle gives a bijection between
exports (active non-root nodes and incoming links) and imports (present
slots of active nodes and outgoing links) that preserves the goal (Lemma 2).
Roots and inactive nodes export nothing.

**Lemma 4 (shards compose).** Equal link commitments imply equal goal digests
(A3). The incoming side's digest fixes its cells (step 7, A3), and the
outgoing side's cells equal a node's hashed goal (Lemma 3). So the
verifier's matching of commitments is a goal-preserving bijection between
outgoing and incoming links. Composing it with the per-shard bijections of
Lemma 3 gives a goal-preserving bijection between all present body goals of
all active nodes and all active non-root nodes. Chains that pass through
several links, or cycle among links only, change nothing.

**Lemma 5 (tree).** Consider the graph with an edge from every present body
goal's node to the node its goal is matched with. Every active non-root node
has in-degree exactly 1 and every root in-degree 0. Then the nodes reachable
from the roots form a finite tree.

*Why.* Suppose a reachable node `v` lay on a cycle. Its unique parent is its
predecessor on the reaching path and also on the cycle. Walking back along
the cycle, every node's parent is on the cycle, so the root would be on it.
But the root has no parent. A cycle can still exist among unreachable nodes
(for example a node for `p :- p.` that proves its own body goal); it is
detached and irrelevant (test `a_detached_cycle_is_harmless`). Finiteness
follows from `K·R` places.

**Theorem proof.** Induct over the tree of Lemma 5 from the leaves. By
Lemma 1 each node is a valid step whose body goals are, by Lemmas 3–4,
exactly the goals of its children, which are derivable by induction. The
roots are clause steps for the trusted goals (step 6). The total failure
probability is the Halo2 soundness error plus the sum of the Lemma 1 and 2
terms over all nodes and shards: far below `2⁻²⁰⁰`.

**What the verifier must not skip (K > 1).**

- Exactly one shard must be verified under the root key with the trusted
  roots as its instance. Otherwise a set of rootless shards proves nothing.
- Every shard must use the same, current fact commitment list.
- Zero commitments are skipped, but nonzero ones are matched with
  multiplicity. An active place cannot publish 0 without a Poseidon preimage
  of 0.

## 5. What is public and what is hidden

The proofs are zero-knowledge in the standard Halo2 sense. The PSE prover
fills the unusable rows of every advice column with random values, commits
with random blinds, and blinds the permutation, lookup and shuffle
polynomials. Fiat–Shamir makes this a non-interactive argument in the random
oracle model. Beyond the public inputs, an accepted proof reveals:

| Information | K = 1 | K > 1 |
|---|---|---|
| the policy, the verifying key(s), `R`, `T`, `B_in`, `B_out` | public | public |
| the trusted root goals | public | public |
| the fact commitment list (all issued facts, not which are used) | public | public |
| number of nodes in the tree | hidden (≤ R), **if** `R` does not depend on the tree | per shard hidden (≤ R); `K` is public |
| goals, branches, rule choice, facts used, arithmetic values | hidden | hidden |
| tree shape | hidden | hidden inside shards |
| number of links per shard | — | **public** (nonzero commitments) |
| which shard is linked to which | — | **public** (equal commitments) |
| proof size, verification time | depends on `k` only | `K` proofs |

Consequences and recommendations:

- **Fix `R` independently of the tree.** With `R` equal to the tree size,
  the node count leaks. A deployment should use one `R` per policy, for
  example the largest that fits a chosen `k` (about 54 places at `k = 16`
  for the energy policy).
- **The shard graph is a coarse view of the tree.** For the energy tree, the
  first shard receives most links because the root has 10 children. A
  verifier learns `K`, the shard sizes' upper bound and this graph, but not
  the goals or the shape inside a shard. Hiding it would need dummy links or
  recursive aggregation; neither is implemented.
- **Salts must be fresh per link.** Reusing a salt for two links with the
  same goal publishes equal commitments and reveals that the goals are equal.
  Goals with low entropy (`q(1)`) are only hidden because of the salt.
- **Timing.** Prover time depends on `k` and on the number of shards, not on
  the witness. Side channels of the implementation are out of scope.

## 6. Limitations

- **No machine-checked proof.** The argument above is a paper proof. The
  tests show that the circuit and the reference agree on every attack that
  was tried, not on all attacks.
- **No aggregation.** For `K > 1` the output is `K` proofs and a matching
  check. Verification work and proof size grow with `K` (about 10 KB and
  15–25 ms per shard proof with IPA). One succinct proof would need recursion
  (for example KZG/BN254 with `snark-verifier`), which is not used.
- **Bounded model.** Arity at most 4, at most 10 body goals (the policy's
  maximum), 64-bit arithmetic, `R` nodes per proof.
- **Fact issuance and revocation** are outside the circuit. A changed list
  invalidates proofs made against the old one, because the list is a public
  input.

## Evidence

| Claim | Reference (`proof-witness`) | Circuit (`node-circuit`) |
|---|---|---|
| Lemma 1: cell/digest mismatch, wrong branch, false arithmetic, unpublished fact, wrong salt | `tests/reference_check.rs` | `tests/node.rs`, `tests/arith.rs`, `tests/well_formed.rs`, `tests/real_prover.rs` |
| Lemma 3: cycles, double use, leftover node, inactive needed node, forged slot | `tests/reference_check.rs` (`LOOPS`) | `tests/tree.rs` (each case also run by the reference) |
| Lemma 5: detached cycle accepted, circular proof rejected | `a_detached_cycle_is_harmless`, `a_circular_proof_cannot_reach_a_root` | `tests/tree.rs`, `tests/real_prover.rs` |
| Roots: untrusted goal, fact root | `tests/reference_check.rs` | `tests/tree.rs`, `tests/real_prover.rs` |
| Lemma 4: wrong salt, unhashed incoming goal, forged incoming goal, missing link end, duplicated commitment, roots outside the first shard | `tests/shards.rs` | — |
| Lemma 4 in the circuit: wrong salt (the shard proof verifies, the matching fails), unused place publishes 0, unhashed incoming goal, forged outgoing goal | — | `tests/shards.rs` (MockProver and the real prover) |
| Whole tree, `K = 1` and `K = 2` | `tests/energy_policy.rs`, `tests/shards.rs` | `tests/tree.rs`, `tests/shards.rs`, `examples/measure.rs` |

[`TreeCircuit`]: ../node-circuit/src/circuit.rs
