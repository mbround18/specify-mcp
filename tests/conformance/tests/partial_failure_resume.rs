#[test]
fn partial_failure_fixture_has_parent() {
    let fixture = specify_conformance::fixtures::parent_plus_twenty_fixture();
    assert_eq!(fixture.parent, "PARENT");
}
