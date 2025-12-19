# Cloud-ready, compositionally verifiable zero-knowledge evaluation of declarative policies
## How to use
### Rule generator
Upload the relevant policy into prolog/policy.pl, then run:
```bash
    cargo run -p tokens --release
```
This will create the input/rules.json which is the json representation of the rules

### Producing the commitment hashes
Upload the relevant facts with their salt into issue/src/facts.yaml, then run:
```bash
    cargo run -p issue --release
```
This will create the input/fact_hashes.json which contains the public hashes

### Creating the proof tree with the Meta-interpreter
1. Make the file executable:
```bash
    chmod +x prolog/main.pl
```
2. Run it:
```bash
    ./prolog/main.pl
```
This will create the input/proof_tree.json which correctness the ZKP system has to prove

### Generating the proofs
```bash
    cargo run -p prove --release
```
This will generate output/unif_proofs.json, which is able to prove that the proof tree is valid with respect to the rules defined in rules.json, that all facts are signed by the issuer for the prover creating the proof, and that all built-in predicates are correctly applied

### Verifying the proofs
```bash
    cargo run -p verify --release
```
The result determines whether the prover’s claim is valid and authentic.