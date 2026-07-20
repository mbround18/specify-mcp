#[test]
fn placeholder_partial_failure_resume() {
    let fixture = super::fixtures::task_graph::parent_plus_twenty_fixture();
    assert_eq!(fixture.parent, "PARENT");
}
