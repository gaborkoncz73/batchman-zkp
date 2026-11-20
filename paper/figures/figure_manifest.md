# Figure manifest

Status vocabulary: **implemented** is enforced by current code; **partial** exists but lacks complete end-to-end orchestration; **proposed** is final architecture and uses dotted artwork; **unsupported** is deliberately absent.

## Figure 1 — End-to-end assurance chain

- Manuscript: Section 1, after the solution idea and before contributions.
- Width: 180 mm.
- Sources: `README.md`, `docs/soundness-baseline.md`, `policy-parser/src/{syntax/,policy.rs}`, `common/src/{policy_compiler,unification_checker_circuit}.rs`, prover/verifier entry points, analyzer fork at local revision `722b057` (dirty working tree inspected read-only).
- Implemented: typed compilation, local certificates, independent local proving/verification, constraint analysis architecture in the analyzer fork.
- Proposed: authenticated whole-tree package and complete verifier-side statement check.
- Open: define and implement the final manifest protocol and its public binding.

## Figure 2 — Roles, trust anchors, and data flows

- Manuscript: Section 3.2, immediately after actor definitions.
- Width: 180 mm.
- Sources: `issue/src/main.rs`, `common/src/typed_fact.rs`, `prove/src/main.rs`, `verify/src/main.rs`, `README.md`.
- Implemented: issuer-generated registry commitments, private fact/salt prover input, common policy compilation, policy-specific key generation, proof/public-input transfer.
- Proposed: proof-tree manifest flow.
- Unsupported/absent: in-circuit issuer signature verification.
- Open: specify how registry authenticity and policy distribution are realized in deployment (PKI, transparency log, authenticated file, or application trust root).

## Figure 3 — Resolution step to ZK

- Manuscript: Section 4.2.
- Width: 180 mm.
- Sources: `prove/src/helpers.rs`, `common/src/witness_ir.rs`, `common/src/unification_checker_circuit.rs`.
- Implemented: host-side proof search/certificate preparation and bounded circuit verification of rule/fact/builtin local steps.
- Proposed: none in the core relation diagram.
- Open: formalize the exact relation and statement syntax in the paper.

## Figure 4 — Semantics-preserving compilation

- Manuscript: opening of Section 5.
- Width: 180 mm.
- Sources: `policy-parser/prolog.g4`, parser generation script, `policy-parser/src/{syntax/,policy.rs}`, `common/src/{policy_ir,policy_compiler,circuit_dialect}.rs`, `config/circuit-limits.json`, prover/verifier entry points.
- Implemented: semantic schema v2, explicit dialect/bounds validation, shared policy compiler, current explicit policy commitment, policy-specific PK/VK derivation.
- Proposed: none; check marks mean tested/validated boundaries, not a mechanized compiler proof.
- Open: state whether the final protocol retains the explicit policy commitment when VK provenance is guaranteed, and provide determinism evidence across supported environments.

## Figure 5 — Typed Prolog terms

- Manuscript: Section 5.3.
- Width: 89 mm.
- Sources: `common/src/{policy_ir,typed_term,typed_policy}.rs`, frontend/lowering tests.
- Implemented: distinct atom/integer/nil/cons/compound kinds, ordered nested representation, typed hashing.
- Proposed: none.
- Open: none at the conceptual level; the manuscript should give the exact encoding/domain-separation specification separately.

## Figure 6 — Fact issuance and ZK use

- Manuscript: Section 6.5.
- Width: 180 mm.
- Sources: `issue/src/main.rs`, `common/src/typed_fact.rs`, `common/src/chips/fact_check/external_fact_chip.rs`, `config/circuit-limits.json`.
- Implemented: Poseidon commitment over domain tag, schema version, typed-goal digest and salt; 33 public registry slots; in-circuit recomputation and membership; external-fact branch.
- Partial/deployment assumption: registry authentication is external to the circuit.
- Unsupported/absent: digital signature creation or in-circuit signature verification.
- Open: salt-generation requirements, registry governance, replay/revocation semantics, and issuer-authentication mechanism.

## Figure 7 — Local resolution check

- Manuscript: Section 6.7.
- Width: 180 mm.
- Sources: `common/src/chips/resolution_chip.rs`, typed rule/fact/builtin chips, `common/src/unification_checker_circuit.rs`.
- Implemented: rule/fact/builtin branch checks; Booleanity; exact one-hot equation; public tags 1, 2, and 3; typed parent/children commitments.
- Proposed: none.
- Open: the formal soundness statement must make circuit limits and supported builtin semantics explicit.

## Figure 8 — Proof-tree composition

- Manuscript: Section 6.8.
- Width: 180 mm.
- Sources: `prove/src/{main,writer}.rs`, `verify/src/{main,reader}.rs`, `common/src/commitments.rs`, project README and soundness baseline.
- Implemented: independent local proofs and their public parent/ordered-child/tag values; deterministic proof order; prover-side validation of current tree edges.
- Partial: local proofs are independently verifier-checkable.
- Proposed: verifier-authenticated trusted root, exact count/topology, exhaustive child-to-parent cross-links, omission/duplication/reordering rejection as one manifest protocol.
- Unsupported/absent: recursive proof aggregation.
- Open: exact manifest encoding, commitment domain separation, leaf treatment, failure policy, and whether linkage is checked by verifier orchestration or an additional circuit.

## Figure 9 — Static assurance pipeline

- Manuscript: Section 7.1.
- Width: 180 mm.
- Sources: local analyzer fork revision `722b057`, especially `korrekt/src/circuit_analyzer/ucp/`; analyzer documentation/tests; Batchman circuit code and baseline.
- Implemented in analyzer fork: structural analyses, PLONKish-aware UCP, learned-variable propagation, targeted cvc5 finite-field solving.
- Proposed/pending: robust Batchman-to-analyzer adapter and complete evaluation on the current circuit; drawn dotted.
- Open: halo2 crate-version/type-identity compatibility (Batchman uses the 0.3.1 line while the local analyzer fork is based on a patched 0.3.0 line), lookup coverage, target-selection protocol, hash abstraction, timeout policy, and false-positive/negative methodology.

## Figure 10 — Results

- Manuscript: Section 9, RQ3/RQ4 results.
- Width: 180 mm.
- Sources: only CSV schemas in `data/`; no values were inferred from old thesis results or unrelated baselines.
- Implemented: plotting/layout template and explicit no-data guard.
- Proposed/pending: the complete experimental campaign and statistical plotting.
- Open/required experiments: seeded mutations with ground truth; proving time by tree depth/width and Halo2 k; parallel runs for available worker counts; analyzer phase timing by constraint/target count; fixed hardware/software metadata; warm-up policy; fixed seeds; repetitions; median/IQR or justified alternative; timeout reporting.
- Publication status: **not manuscript-ready** until populated with reproducible measurements. The exported draft is intentionally and visibly labeled.

## Repository facts used across figures

- `config/circuit-limits.json` currently bounds integers to 32 bits, predicate arity to 4, body/child count to 10, term nodes to 64, term depth to 16, proof nodes to 128, proof depth to 32, and external facts to 33.
- The local public instance contains 37 field elements: 33 fact-registry values plus policy commitment, parent-goal commitment, ordered-children commitment, and resolution tag.
- The current typed-fact preimage is `[domain, schema_version, typed_goal_digest, salt]` and is hashed with Poseidon.
- Current local resolution tags are 1=policy rule, 2=external fact, 3=builtin and are constrained by one-hot branch selection.
- Current baseline documentation reports k=16, 30 gates, 165 custom polynomials, maximum degree 6, 17,168 rows, 37 advice, 20 fixed and 1 instance column, no lookups, and an 8,096-byte real proof. These values are context only and were not turned into an experimental results series.
- Production equality checking is represented as exact circuit constraints; experimental RLC material is not presented as the authoritative current path.
