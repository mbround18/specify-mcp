#[test]
fn placeholder_provider_parity() {
    let fixture = super::fixtures::task_graph::parent_plus_twenty_fixture();
    assert_eq!(fixture.tasks.first().map(String::as_str), Some("T001"));
}
