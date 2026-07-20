#[test]
fn provider_parity_fixture_is_deterministic() {
    let fixture = specify_conformance::fixtures::parent_plus_twenty_fixture();
    assert_eq!(fixture.tasks.first().map(String::as_str), Some("T001"));
}
