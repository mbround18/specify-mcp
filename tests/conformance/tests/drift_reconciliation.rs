#[test]
fn drift_fixture_has_stable_prefix() {
    let fixture = specify_conformance::fixtures::parent_plus_twenty_fixture();
    assert!(fixture.tasks.iter().all(|task| task.starts_with('T')));
}
