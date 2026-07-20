use specify_provider_core::canonical_label_policy;
use specify_provider_core::contract::{canonical_lifecycle_labels, canonical_relationship_labels};

#[test]
fn exposes_canonical_label_sets() {
    let policy = canonical_label_policy();
    assert!(policy.lifecycle_labels.contains("lifecycle:todo"));
    assert!(policy.relationship_labels.contains("rel:blocked-by"));
}

#[test]
fn lifecycle_and_relationship_helpers_include_required_labels() {
    let lifecycle = canonical_lifecycle_labels();
    let relationship = canonical_relationship_labels();
    assert!(lifecycle.contains(&"lifecycle:done".to_string()));
    assert!(relationship.contains(&"rel:depends-on".to_string()));
}
