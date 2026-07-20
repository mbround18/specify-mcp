#[test]
fn placeholder_drift_reconciliation() {
    let fixture = super::fixtures::task_graph::parent_plus_twenty_fixture();
    assert!(fixture.tasks.iter().all(|t| t.starts_with('T')));
}
