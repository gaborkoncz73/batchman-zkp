# Cloud-ready, compositionally verifiable zero-knowledge evaluation of declarative policies

## How to use

### Compile the Prolog policy

Put the policy in `prolog/policy.pl`, then run:

```bash
cargo run -p tokens --release
```

The compiler parses the supported Batchman policy-Prolog dialect with the generated ANTLR parser. It rejects syntax errors instead of emitting a partial result. A successful default run writes one deterministic source-of-truth file:

- `input/rules.json`: canonical semantic IR (`schema_version: 2`). It preserves predicate names and arities, source-order clauses, typed literals, clause-local variable identities, compound terms, proper and improper lists, and the real operator tree.

Both `prove` and `verify` read this same file, validate it against `config/circuit-limits.json`, and compile the current bounded circuit layout in memory. Neither runtime reads `rules.circuit-v1.json`.

The semantic representation therefore keeps, for example, `Input * Value div 100` as `div(*(Input, Value), 100)`; operators are not converted into an artificial sequence of unary predicates in `rules.json`.

The command accepts explicit source and semantic-output paths:

```bash
cargo run -p tokens --release -- \
  path/to/policy.pl \
  path/to/rules.json
```

A third path optionally writes the legacy circuit-v1 projection for debugging or golden tests. It is not a production input. When that output is requested, a fourth path may select the circuit-limit profile; its default is `config/circuit-limits.json`.

To regenerate the checked-in Rust parser after changing `tokens/prolog.g4`:

```bash
./tokens/generate-parser.sh
```

### Producing the commitment hashes

Upload the relevant facts with their salt into issue/src/facts.yaml, then run:

```bash
cargo run -p issue --release
```

This creates `input/fact_hashes.json`. Each public fact hash binds the
domain and schema version, the canonical typed goal digest, and the salt. The
goal digest preserves predicate identity, arity, and every argument's Prolog
term type and tree structure. The current circuit expects exactly 33 registry
slots.

### Creating the proof tree with the Meta-interpreter

1. Make the file executable:

```bash
chmod +x prolog/main.pl
```

2. Run it:

```bash
./prolog/main.pl
```

This creates `input/proof_tree.json`, whose local steps the ZKP system proves.

Independently specify the public goal that the verifier is expected to trust in
`input/trusted_roots.json`. For the current single-root policy it is:

```json
{
  "schema_version": 1,
  "dialect": "batchman_trusted_roots_v1",
  "root_goals": ["endPrice(931220)"]
}
```

In a deployment this file is verifier-controlled input. It must not be copied
from the prover's proof tree or proof artifact, because it is the trust anchor
for the claimed computation.

### Generating the proofs

```bash
cargo run -p prove --release
```

This generates the independent local proofs in `output/unif_proofs.json` and
wraps them in one compositional proof-tree artifact.
Policy compilation and proof-tree validation use `rules.json`; the circuit
assigns every parent and child as a canonical typed goal. A bounded legacy
compatibility projection remains for the arithmetic core and input shape, but
typed cells are authoritative for public goal identity, selected policy-rule
paths, external facts, and their connection to builtin semantics. Every typed
step is validated before key generation. Proofs are then computed in parallel,
collected in deterministic proof-tree DFS order, and atomically written as one
complete file. Its manifest records only root and ordered child indices; it
does not reveal goal terms beyond the separately public trusted root.

Each entry contains the proof bytes plus four required public fields: the
compiled policy commitment, the parent typed-goal commitment, the ordered
typed-children commitment, and the semantic resolution tag. The v3 policy
commitment binds both the bounded compatibility layout and the canonical typed
policy plan, including candidate identities, term paths, patterns, and
equalities. Parent/children commitments are calculated from the actual typed
circuit cells, and the policy commitment is tied to the policy-specific fixed
columns and key. Instance rows 33–36 follow the 33 public fact hashes. Row 36
is `1` for a policy rule, `2` for an external fact, and `3` for a builtin. A
commitment-free, manifest-free, or older proof artifact must be regenerated.

### Verifying the proofs

```bash
cargo run -p verify --release
```

The command first reconstructs the public tree statement from the manifest and
the proof-bound commitments. It rejects a wrong trusted root, missing or extra
proofs, invalid indices, duplicate edges, cycles, orphan nodes, non-canonical
DFS order, excessive depth, children on fact/builtin leaves, reordered child
goals, a policy mismatch, or an invalid resolution tag. It then independently
verifies every local Halo2 proof against the common fact registry and its own
bound public commitments. This is compositional verification, not recursive
proof aggregation.

### Regression tests and current soundness boundary

Run the fast workspace suite with `cargo test --workspace`. The real Halo2 create/verify smoke test is intentionally separate because key generation and proving at `k=16` are slow:

```bash
cargo test --release -p common --test real_proof -- --ignored --nocapture
```

The former P0 rejection tests are active and green. The rule path uses boolean
actual-clause selectors, authenticates compiler-fixed paths in the canonical
typed term trees, and constrains the selected signatures and patterns exactly.
Variable occurrences use the compiler's minimal equality matrix and a
candidate-gated dot product compressed by Halo2's post-advice-commitment
lookup challenge; a false residual vector therefore cannot be chosen to
cancel against known coefficients. External
facts use the typed goal digest and an algebraically constrained membership
check against all 33 public hashes. Builtin relation and arithmetic opcodes,
integer node types and payloads, expression-tree links, arithmetic transitions,
comparisons, non-zero division, and conditional unsigned-32-bit range checks
are constrained in-circuit. A one-hot resolution gate binds exactly one of the
rule, external-fact, or builtin branches to public instance row 36.

The supported end-to-end claim is now anchored at the verifier-controlled root
and composed across the exact ordered tree. Each local proof is still generated
and verified independently; proof aggregation is intentionally outside the
current design. Policy-unreferenced variable roots remain opaque term
commitments inside one local relation, but the official end-to-end pipeline
anchors the root and connects every child commitment to the corresponding
proved parent commitment. The exact trust assumptions and current circuit cost
are documented in `docs/soundness-baseline.md`.
