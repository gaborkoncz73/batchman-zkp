# Soundness regression baseline

The fast suite separates three kinds of evidence:

- `tokens/tests/reference_semantics.rs` is the circuit-independent semantic oracle and golden policy test.
- `common/tests/circuit_soundness.rs` exercises the current honest circuit witness path with `MockProver`.
- `common/tests/real_proof.rs` performs a real `create_proof` / `verify_proof` round trip in the opt-in release test.

The five former P0 specifications are now active regression tests and green:

- a rule literal such as `p(a)` cannot be used as `p(b)` because selected
  typed pattern digests and shapes are constrained directly;
- every selected variable occurrence comes from an authenticated,
  compiler-fixed typed term path. The compiler's minimal equality matrix is
  checked by a candidate-gated dot product whose residual tuple is compressed
  with Halo2's post-advice-commitment lookup challenge, so a prover cannot
  arrange cancellation against coefficients known while choosing the witness;
- quoted atom `'1'` and integer `1` have different typed tags and digests;
- bounded integer comparison rejects `1 > 2`;
- selected integer division constrains a non-zero divisor and rejects division
  by zero.

The production bridge has an active invariant test in `prove/src/helpers.rs`:
on all 49 current proof-tree edges, the parent's typed child hash equals the
child's typed goal hash. Step 7 now exposes and verifies the same invariant in
the production artifact: the current 50 local proofs are linked by those 49
ordered edges and anchored at the separately trusted root.

Run one former blocker explicitly, for example:

```bash
cargo test -p common --test circuit_soundness \
  rejects_a_goal_that_violates_rule_literals -- --nocapture
```

The current checked Step 7 cost baseline at `k=16` is 30 gates, 165 custom
constraint polynomials, one lookup, and 17,168 maximum assigned rows. It uses
37 advice columns, 22 fixed columns, one instance column, and 60 columns in
total. Halo2's analytical model and the real serialized transcript both report
a 9,088-byte proof. These figures are regression assertions in
`common/tests/circuit_cost.rs` and `common/tests/real_proof.rs`; they should
change only with a deliberate circuit modification.

## Variable-equality matrix dot product

The thesis-style construction is now implemented on the production typed-rule
path by `common/src/chips/typed_variable_dot_chip.rs`. For one candidate, every
compiler-produced equality is an incidence-matrix row with coefficients `+1`
and `-1`; applying it to the vector of authenticated typed-term digest
occurrences yields the equality-residual vector. One native 11-component Halo2
zero lookup compresses this vector as a dot product with transcript `theta`
powers. Since `theta` is sampled after all advice commitments, a non-zero
selected residual vector passes only with probability at most `10 / |Fp|`, in
addition to the standard proof-system soundness error. The candidate flag is
the same boolean cell used by typed clause selection, and every operand is
copy-constrained from an authenticated compiler-fixed term opening.

The production tests include opposite residuals that cancel in an unweighted
sum, isolated selected/inactive candidate cases, and the complete typed-rule
soundness regressions. The historical gate-free `DotExistsChip` is not used.

The fair before/after measurement on the current canonical typed circuit is:

| Metric | Exact selected-row gate | Typed matrix dot product |
| --- | ---: | ---: |
| Lookup arguments | 0 | 1 |
| Advice columns | 37 | 37 |
| Fixed columns | 20 | 22 |
| Total columns | 58 | 60 |
| Maximum rows | 17,168 | 17,168 |
| Analytical proof size | 8,128 bytes | 9,088 bytes |
| Serialized release proof | 8,096 bytes | 9,088 bytes |

The maximum degree, gate count, custom-polynomial count, permutation columns,
and `k=16` remain unchanged. The dot-product chip reuses the typed rule's
`left`, `right`, and `flag` advice columns, so it adds no advice column, but the
native lookup requires a complex selector and a zero table. In this layout the
exact gate also reused one polynomial across all equality rows; replacing it
therefore does not reduce the custom-polynomial count or the circuit-wide row
maximum. The sound dot product increases the serialized proof by 992 bytes
(about 12.3%) on the current policy. It is retained because the intended
thesis construction is now the requested production semantics, not because
this particular policy demonstrates a performance improvement.

Step 3 removes persisted `rules.circuit-v1.json` drift: prover and verifier now compile the same semantic `rules.json` with the shared `common::policy_compiler`. It also introduces a typed, lossless `GroundTerm` / `GroundCall` / `LocalStepWitness` layer with explicit policy-rule, external-fact, and builtin transitions. The compatibility bridge consumes that typed step directly and selects list projections from the compiled clause signatures and equality references; missing, ambiguous, and out-of-bound projections are rejected.

Step 4 introduced the fixed public-input layout in the existing instance
column, Step 5 extended it with the resolution tag, and Step 6 changed the
goal, children, policy, fact, rule, and builtin bindings to their canonical
typed form without changing the row indices. The current layout is:

```text
0..32  public fact hashes
33     composite compiled circuit-policy commitment (v3)
34     parent canonical typed-goal commitment
35     ordered canonical typed-children commitment
36     semantic resolution tag (1 rule, 2 external fact, 3 builtin)
```

The parent and every child use the same domain-separated, fixed-width typed
goal hash over the predicate symbol, exact arity, and four typed argument-root
digests. Each argument root recursively preserves atom, integer, nil, cons, or
compound identity and structure. The ordered-child commitment hashes its own
domain tag, the explicit child count, and ten fixed typed child hashes. The
circuit constrains the policy, parent, children, and resolution values directly
to the instance column.

Policy compilation remains verifier-controlled and key generation remains
policy-specific. The v3 composite policy commitment covers both the bounded
`RuleTemplateFileFp` compatibility layout and the canonical `TypedPolicyPlan`.
The latter commits stable candidate identities, ordered child signatures,
parent/child argument references with exact term-tree paths, typed patterns,
and variable equalities. Its constants and authenticated opening paths are
therefore fixed by the same policy-specific circuit. A future universal-key
design would require in-circuit policy authentication instead.

Every local proof carries the v3 commitment object: policy, parent, ordered
children, and resolution tag. The top-level v1 compositional artifact contains
a canonical DFS manifest beside the proof array. The verifier recomputes and
requires the policy commitment, validates the exact tree, then verifies each
independent proof against its own 37-element instance. Old commitment-free,
pre-v3, and manifest-free artifacts are rejected with a regeneration error.
The release regression suite covers round-trip verification,
public-input/proof-byte tamper rejection, and cross-policy rejection.

Steps 5 and 6 replace the host-only local decision path with the following
algebraic checks:

- `TypedRuleChip` allocates flags only for actual compiled candidates. Each
  flag is boolean, the constrained fold permits at most one selected
  candidate, and the boolean `rule_ok` cell equals the final sum. A selected
  candidate must match the canonical typed parent signature, ordered child
  signatures, and exact semantic child count. Compiler-fixed term paths are
  authenticated against the typed goal roots; selected exact/shape patterns
  are constrained exactly, while variable occurrences are checked by the
  transcript-challenge-compressed typed matrix dot product over those same
  openings. The previous legacy signature/equality rule path, fixed-alpha
  signature RLC, and gate-free dot-product existence check are no longer
  authoritative.
- The external-fact path hashes a versioned domain tag, the constrained typed
  parent-goal digest, and the salt. Its running product
  `prod(hash - public_hash[i])` is constrained across all 33 public hashes; a
  selected external branch requires the final product to be zero.
- The builtin path retains the bounded unsigned-32-bit evaluator and adds a
  typed bridge to the publicly committed parent goal. It constrains exact
  relation and arithmetic functors, integer node tags, integer payloads,
  expression-tree child links, selector booleanity, ground equality, `is`
  transitions, bounded comparisons, and `+`, `-`, `*`, and `div`. Division
  constrains a non-zero divisor together with quotient/remainder relations.
  Conditional 32-bit decompositions reconstruct every value used by the
  configured unsigned-integer semantics.
- The resolution gate makes rule, external-fact, and builtin branches boolean
  and one-hot, derives tags `1`, `2`, and `3`, and constrains the selected tag
  to instance row 36. It also constrains the canonical legacy leaf marker and
  the relevant leaf/child-count conditions.

These changes close the recorded local-circuit regressions and wire the native
typed-term foundation into `UnificationCircuit`: domain-separated,
type-preserving hashes for atoms, integers, nil, cons, and compounds; fixed
typed goal records; authenticated term-path openings; typed rule matching;
typed external-fact commitments; and typed bindings for builtin semantics. A
legacy bounded projection remains as an internal compatibility input for the
existing arithmetic core and canonical input markers, but it is no longer the
public parent/child identity or the authoritative selected-rule and fact
representation.

The local rule relation deliberately treats a policy-unreferenced variable's
argument root as an opaque term commitment. It proves equality between repeated
variables and authenticates every constructor/literal path demanded by the
selected clause, but it does not recursively open every node below every
singleton variable. The official prover always derives these roots from a
parser-validated `GroundTerm`; that host derivation is not, by itself, a full
in-circuit preimage proof. Step 7 closes this boundary for the official
end-to-end pipeline: `input/trusted_roots.json` is parsed through the same
frontend and supplies the verifier trust anchor, and every opaque child root is
cross-linked to a local proof carrying that exact parent commitment. A future
standalone local-statement relation, used without this composition layer, would
instead need a fixed-width full-term preimage gadget.

## Compositional proof-tree validation

Step 7 intentionally preserves independent local proofs rather than adding
recursive aggregation. The top-level manifest contains a root-index list and
one ordered child-index list per proof. Before accepting the artifact, the
verifier checks:

- the manifest schema/dialect and all configured tree bounds;
- exact equality between proof count and manifest node count;
- the policy commitment and resolution tag of every node;
- exact ordered-child commitment reconstruction from child parent-goal
  commitments, including child count and canonical empty padding;
- zero children for external-fact and builtin resolutions;
- one incoming edge for every non-root and none for every root;
- reachability, acyclicity, canonical DFS preorder, and maximum depth;
- exact ordered equality between manifest roots and the commitments derived
  from the verifier-controlled trusted root goals.

Consequently a proof cannot be omitted, inserted as an orphan, referenced by
two different parents, moved to a semantically different child position, or
used under another root while the verifier still accepts the artifact. If two
positions intentionally have the same goal commitment, their proofs are
statement-equivalent; the topology still requires two node occurrences. The
manifest adds no circuit rows or proof bytes to an individual Halo2 transcript,
so the circuit-cost figures above remain unchanged. Its serialized JSON size
is orchestration overhead and scales linearly with the number of local proofs.

The root file is a trust input, not prover evidence. In deployment the verifier
must obtain it independently (for example from the application request or an
authenticated policy-evaluation job) and must not accept a root selected from
the proof artifact itself.
