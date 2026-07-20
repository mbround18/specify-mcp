use specify_provider_core::contract::Provider;
use specify_provider_core::usecases::sync::{apply_sync, plan_sync};
use specify_provider_core::{
    ExecutionContext, IntegrationIdentity, LifecycleMetadata, ProviderCapabilities, ProviderError,
    ProviderHealth, ProviderKind, RelationshipMetadata, SourceOfTruthMarker, SpecRecord,
    SyncItemRecord, SyncRequest, TaskRecord,
};

struct MockProvider;

impl Provider for MockProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Local
    }
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_health_check: true,
            supports_status_sync: true,
            supports_dry_run: true,
            supports_labels: true,
            supports_relationships: true,
            supports_lifecycle: true,
            supports_milestone: false,
            supports_project: false,
            supports_assignee: true,
            supports_reconciliation: true,
        }
    }
    fn health_check(&self) -> Result<ProviderHealth, ProviderError> {
        Ok(ProviderHealth {
            healthy: true,
            detail: "ok".to_string(),
        })
    }
    fn upsert_spec(&self, _spec: &SpecRecord) -> Result<String, ProviderError> {
        Ok("spec".to_string())
    }
    fn upsert_tasks(
        &self,
        _spec_key: &str,
        _tasks: &[TaskRecord],
    ) -> Result<Vec<String>, ProviderError> {
        Ok(vec!["task".to_string()])
    }
}

#[test]
fn requires_lifecycle_label_for_metadata() {
    let provider = MockProvider;
    let request = SyncRequest {
        context: ExecutionContext {
            run_id: "r1".to_string(),
            context_type: "feature".to_string(),
            context_value: "f".to_string(),
            timestamp: "1Z".to_string(),
        },
        spec: SpecRecord {
            key: "f".to_string(),
            title: "feature".to_string(),
            status: "active".to_string(),
        },
        tasks: vec![TaskRecord {
            key: "T001".to_string(),
            title: "x".to_string(),
            status: "open".to_string(),
            priority: "normal".to_string(),
        }],
        items: vec![SyncItemRecord {
            internal_key: "T001".to_string(),
            remote_id: None,
            fingerprint: "f".to_string(),
            previous_fingerprint: None,
            integration_identity: IntegrationIdentity {
                owner_identity: "f".to_string(),
                assignee_identity: "u".to_string(),
                source_system_identity: "specify".to_string(),
                identity_version: 1,
            },
            source_of_truth: vec![SourceOfTruthMarker {
                field_name: "task".to_string(),
                authority: "local".to_string(),
                last_confirmed_at: "1Z".to_string(),
            }],
            lifecycle: LifecycleMetadata {
                lifecycle_state: "todo".to_string(),
                lifecycle_stage: "impl".to_string(),
                entered_at: "1Z".to_string(),
                updated_at: "1Z".to_string(),
            },
            relationships: RelationshipMetadata {
                parent_id: None,
                child_ids: vec![],
                blocked_by_ids: vec![],
                depends_on_ids: vec![],
            },
            canonical_labels: vec!["rel:child".to_string()],
            linkage: None,
            conflict_reason: None,
        }],
    };
    let plan = plan_sync(&provider, &request).expect("plan");
    assert!(apply_sync(&provider, &plan).is_err());
}
