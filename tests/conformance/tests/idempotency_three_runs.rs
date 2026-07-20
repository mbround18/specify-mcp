#[test]
fn idempotency_fixture_has_twenty_tasks() {
    let fixture = specify_conformance::fixtures::parent_plus_twenty_fixture();
    assert_eq!(fixture.tasks.len(), 20);
}
