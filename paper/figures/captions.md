# Figure captions

## Figure 1

Short: The assurance claim is established across policy semantics, local circuits, proof composition, verification, and complementary static analysis.

Full: **End-to-end assurance chain for declarative policy evaluation.** Read left to right, an authenticated public Prolog policy is compiled into a canonical typed policy, used to construct a local certificate tree, and checked by independently generated Halo2 proofs. Authenticated facts enter only as private prover inputs. The lower lane supplies constraint-level evidence but does not decide proof acceptance. Solid stages are implemented; the dotted authenticated tree package and complete-statement check denote the target final architecture. The central takeaway is that semantic preservation, circuit soundness, composition, and cryptographic verification are distinct links in one assurance chain.

## Figure 2

Short: Acceptance is rooted in verifier-controlled policy, registry, compiler configuration, verification-key provenance, and public Halo2 parameters—not trust in the prover.

Full: **System roles, trust anchors, and data flows.** The Policy Author distributes an authenticated public policy; the Fact/VC Issuer privately provides a typed fact and salt to the Prover while registering only its commitment. The untrusted Prover returns local proof artifacts (π) and public inputs. The Verifier derives policy-specific verification material (VK_P) from the trusted policy, compiler, and bounds. Issuer authentication is outside the circuit, and neither private fact nor salt crosses into the verifier boundary. The dotted tree-manifest flow is proposed rather than fully implemented.

## Figure 3

Short: A proof-tree node becomes a bounded relation ∃w:R_step(x,w)=1 whose private certificate is hidden by a local Halo2 proof.

Full: **From a resolution step to a zero-knowledge relation.** Panel a shows the certificate for a generic Prolog step: parent goal, selected clause, substitution θ, and ordered children. Panel b partitions the statement into public data x and private witness w. Panel c checks exactly one rule, fact, or builtin interpretation and emits a local proof π. Proof search remains outside the circuit; Halo2 validates the supplied bounded certificate rather than executing a search engine.

## Figure 4

Short: A shared compiler rejects unsupported policies and preserves the semantic information needed to derive policy-specific Halo2 material.

Full: **Semantics-preserving policy compilation.** The pipeline parses `policy.pl`, lowers its parse tree into canonical `rules.json` schema v2, validates explicit circuit bounds, and derives a typed circuit plan and policy-specific proving and verification material (PK_P/VK_P). The ribbon lists preserved attributes, including names, arities, clause order, clause-local variable identities, typed literals, nested terms, operator structure, child order, and equality paths. Out-of-dialect or oversized inputs are rejected rather than truncated. The present artifact additionally exposes an explicit public policy commitment; policy-specific keys also bind circuit shape. The figure claims deterministic shared implementation, not formal compiler verification.

## Figure 5

Short: Canonical typed trees preserve Prolog kinds, ordering, and nesting, preventing atom/integer and structural ambiguity.

Full: **Canonical, type-preserving representation of Prolog terms.** Panel a distinguishes Atom, Integer, Nil, Cons, and Compound nodes and makes explicit that `Atom("1")` is not `Integer(1)`. Panel b decomposes a neutral nested term into an ordered typed tree. Canonical means that one supported semantic term has one deterministic representation; typed means that node kind participates in the representation and digest. No field encoding or variable-equality mechanism is implied by this conceptual view.

## Figure 6

Short: A prover proves authorized use of a salted typed-fact commitment without disclosing the fact or salt.

Full: **Authenticated fact issuance, commitment, and zero-knowledge use.** Panel a shows canonical typed-goal encoding and the implemented Poseidon commitment c_f=H(domain tag, schema version, typed-goal digest, salt); the private preimage goes to the holder, whereas c_f enters an issuer-controlled registry. Digital-signature verification is not performed in the circuit. Panel b recomputes the digest and commitment, enforces membership in the 33-slot public registry, and checks external-fact resolution semantics before producing π. The verifier receives neither fact nor salt. Commitment privacy is not encryption and depends on a sufficiently unpredictable salt and the precise preimage distribution.

## Figure 7

Short: A local node is accepted only if exactly one rule, authenticated-fact, or builtin branch validates and matches the public resolution tag.

Full: **Checking one proof-tree step.** The local statement commits to a parent goal, ordered children, and a public resolution tag; private certificate data supplies the selected clause, fact preimage, or builtin intermediates. Implemented branch checks output Boolean indicators r, f, and b, constrained by r+f+b=1, while tag=r+2f+3b binds the chosen interpretation to the public instance. The output is one independently verifiable local proof π.

## Figure 8

Short: Parent–child commitments can authenticate linked independent proofs, but complete tree-manifest verification remains target architecture and is not recursive aggregation.

Full: **Authenticated composition of independent local proofs.** Panel a shows sibling proofs produced in parallel, with each parent’s ordered-child commitment intended to match its children’s parent-goal commitments. Panel b adds a trusted root, exact count, ordered topology, leaf positions, and deterministic proof order so that omission, duplication, and reordering are rejected. Local proof generation and verification are implemented, and the prover-side host validates current edges; the dotted manifest and complete verifier-side cross-link orchestration are not yet complete. The construction links independent proofs and must not be read as recursive proof aggregation.

## Figure 9

Short: PLONKish-aware structural checks and UCP reduce the target set before finite-field SMT supplies constraint-level evidence.

Full: **Static assurance pipeline: structural analysis, UCP, and finite-field SMT.** The analyzer extracts gates, selectors, rotations, cell classes, copy constraints, and supported lookups from a Halo2 circuit. Structural checks and unique-constraint propagation (UCP) identify cheap defects and determined cells; only residual semantic targets proceed to cvc5 solving over F_p. Outcomes distinguish determined targets, alternative accepted witnesses, algebraic recoverability, and inconclusive results. The analyzer fork implements this architecture, but the Batchman circuit adapter and evaluation campaign remain pending. Results are constraint-level evidence, not proof of whole-system correctness, general computational recovery, or failure of the formal zero-knowledge definition.

## Figure 10

Short: The evaluation layout jointly targets seeded-defect detection, proof-tree scaling, parallel speedup, and analyzer cost, but final measurements are not yet available.

Full: **Security, analyzer effectiveness, and scalability results—data-required draft.** Panel a is the preregistered detection matrix for regression tests, structural checks, UCP, and SMT; panels b–d reserve the prescribed views for proving-time scaling, parallel speedup, and decomposed analyzer cost. Repository inspection found no complete reproducible measurement series from which these panels could be populated without fabrication. Consequently, the SVG/PDF/PNG are visibly marked internal draft, and the accompanying CSV files contain schemas only. This figure must be regenerated after the documented experiments.
