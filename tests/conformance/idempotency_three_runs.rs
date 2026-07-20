#[test]
fn placeholder_idempotency_three_runs() {
    let fixture = super::fixtures::task_graph::parent_plus_twenty_fixture();
    assert_eq!(fixture.tasks.len(), 20);
}
