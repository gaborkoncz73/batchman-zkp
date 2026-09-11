use common::{
    circuit_dialect::CircuitLimits,
    io::read_fact_hashes::{parse_fact_hashes, validate_fact_hashes_for_current_circuit},
};

#[test]
fn current_fact_hash_fixture_has_the_exact_legacy_public_input_shape() {
    let hashes = parse_fact_hashes(include_str!(
        "../../tests/fixtures/current-fact-hashes.json"
    ))
    .unwrap();
    let limits: CircuitLimits =
        serde_json::from_str(include_str!("../../config/circuit-limits.json")).unwrap();
    validate_fact_hashes_for_current_circuit(&hashes, &limits).unwrap();

    assert!(validate_fact_hashes_for_current_circuit(&hashes[..32], &limits).is_err());
    let mut wrong_limits = limits;
    wrong_limits.max_external_facts = 34;
    assert!(validate_fact_hashes_for_current_circuit(&hashes, &wrong_limits).is_err());
}
