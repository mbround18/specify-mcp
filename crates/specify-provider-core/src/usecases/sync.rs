use crate::contract::Provider;
use crate::errors::ProviderError;
use crate::models::{
    canonical_label_policy, ReconciliationOutcome, SyncLedgerEntry, SyncPlan, SyncRequest,
    SyncRunResult,
};

pub fn plan_sync(
    provider: &dyn Provider,
    request: &SyncRequest,
) -> Result<SyncPlan, ProviderError> {
    provider.plan_sync(request)
}

pub fn apply_sync(
    provider: &dyn Provider,
    plan: &SyncPlan,
) -> Result<SyncRunResult, ProviderError> {
    enforce_label_policy(plan)?;
    enforce_metadata_label_consistency(plan)?;
    provider.apply_sync(plan)
}

pub fn evaluate_outcome(
    previous_fingerprint: Option<&str>,
    fingerprint: &str,
    remote_id_exists: bool,
    has_conflict: bool,
) -> ReconciliationOutcome {
    if has_conflict {
        return ReconciliationOutcome::ConflictFlagged;
    }
    match previous_fingerprint {
        None => ReconciliationOutcome::Created,
        Some(prev) if prev == fingerprint => ReconciliationOutcome::Unchanged,
        Some(_) if remote_id_exists => ReconciliationOutcome::DriftCorrected,
        Some(_) => ReconciliationOutcome::Updated,
    }
}

pub fn to_ledger_entries(result: &SyncRunResult) -> Vec<SyncLedgerEntry> {
    result.outcomes.clone()
}

fn enforce_label_policy(plan: &SyncPlan) -> Result<(), ProviderError> {
    let policy = canonical_label_policy();
    for item in &plan.desired_items {
        for label in &item.canonical_labels {
            if !policy.lifecycle_labels.contains(label)
                && !policy.relationship_labels.contains(label)
            {
                return Err(ProviderError::Validation(format!(
                    "item '{}' contains unknown canonical label '{}'",
                    item.internal_key, label
                )));
            }
        }
    }
    Ok(())
}

fn enforce_metadata_label_consistency(plan: &SyncPlan) -> Result<(), ProviderError> {
    for item in &plan.desired_items {
        let expected_lifecycle = format!("lifecycle:{}", item.lifecycle.lifecycle_state);
        if !item
            .canonical_labels
            .iter()
            .any(|l| l == &expected_lifecycle)
        {
            return Err(ProviderError::Validation(format!(
                "item '{}' lifecycle '{}' missing canonical lifecycle label",
                item.internal_key, item.lifecycle.lifecycle_state
            )));
        }
        if item.relationships.parent_id.is_some()
            && !item.canonical_labels.iter().any(|l| l == "rel:child")
        {
            return Err(ProviderError::Validation(format!(
                "item '{}' has parent relation but missing rel:child label",
                item.internal_key
            )));
        }
    }
    Ok(())
}
