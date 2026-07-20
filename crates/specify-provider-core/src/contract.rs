use crate::errors::ProviderError;
use crate::models::{
    canonical_label_policy, ProviderCapabilities, ProviderHealth, ProviderKind, SpecRecord,
    SyncPlan, SyncRequest, SyncResult, SyncRunResult, TaskRecord,
};

pub trait Provider: Send + Sync {
    fn kind(&self) -> ProviderKind;
    fn capabilities(&self) -> ProviderCapabilities;
    fn health_check(&self) -> Result<ProviderHealth, ProviderError>;
    fn map_canonical_labels(&self, labels: &[String]) -> Result<Vec<String>, ProviderError> {
        Ok(labels.to_vec())
    }
    fn upsert_spec(&self, spec: &SpecRecord) -> Result<String, ProviderError>;
    fn upsert_tasks(
        &self,
        spec_key: &str,
        tasks: &[TaskRecord],
    ) -> Result<Vec<String>, ProviderError>;

    fn plan_sync(&self, input: &SyncRequest) -> Result<SyncPlan, ProviderError> {
        Ok(SyncPlan {
            provider: self.kind(),
            context: input.context.clone(),
            spec: input.spec.clone(),
            tasks: input.tasks.clone(),
            desired_items: input.items.clone(),
        })
    }

    fn apply_sync(&self, plan: &SyncPlan) -> Result<SyncRunResult, ProviderError> {
        let spec_id = self.upsert_spec(&plan.spec)?;
        let task_ids = self.upsert_tasks(&plan.spec.key, &plan.tasks)?;
        let outcomes = plan
            .desired_items
            .iter()
            .map(|item| {
                let (outcome, drift_detected) = if item.conflict_reason.is_some() {
                    (crate::models::ReconciliationOutcome::ConflictFlagged, false)
                } else if item.previous_fingerprint.is_none() {
                    (crate::models::ReconciliationOutcome::Created, false)
                } else if item
                    .previous_fingerprint
                    .as_ref()
                    .is_some_and(|v| v == &item.fingerprint)
                {
                    (crate::models::ReconciliationOutcome::Unchanged, false)
                } else if item.remote_id.is_some() {
                    (crate::models::ReconciliationOutcome::DriftCorrected, true)
                } else {
                    (crate::models::ReconciliationOutcome::Updated, false)
                };
                Ok(crate::models::SyncLedgerEntry {
                    run_id: plan.context.run_id.clone(),
                    provider_kind: self.kind(),
                    item_key: item.internal_key.clone(),
                    outcome,
                    drift_detected,
                    conflict_reason: item.conflict_reason.clone(),
                    labels: self.map_canonical_labels(&item.canonical_labels)?,
                    metadata_fields: std::collections::BTreeMap::from([
                        (
                            "lifecycle_state".to_string(),
                            item.lifecycle.lifecycle_state.clone(),
                        ),
                        (
                            "source_identity".to_string(),
                            item.integration_identity.source_system_identity.clone(),
                        ),
                        ("fingerprint".to_string(), item.fingerprint.clone()),
                        (
                            "remote_id".to_string(),
                            item.remote_id.clone().unwrap_or_default(),
                        ),
                    ]),
                })
            })
            .collect::<Result<Vec<_>, ProviderError>>()?;
        Ok(SyncRunResult {
            provider: self.kind(),
            spec_remote_id: spec_id,
            task_remote_ids: task_ids,
            outcomes,
        })
    }

    fn sync(&self, spec: &SpecRecord, tasks: &[TaskRecord]) -> Result<SyncResult, ProviderError> {
        let spec_id = self.upsert_spec(spec)?;
        let task_ids = self.upsert_tasks(&spec.key, tasks)?;
        Ok(SyncResult {
            provider: self.kind(),
            spec_remote_id: spec_id,
            task_remote_ids: task_ids,
        })
    }
}

pub fn canonical_lifecycle_labels() -> Vec<String> {
    canonical_label_policy()
        .lifecycle_labels
        .into_iter()
        .collect()
}

pub fn canonical_relationship_labels() -> Vec<String> {
    canonical_label_policy()
        .relationship_labels
        .into_iter()
        .collect()
}
