use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Local,
    Github,
    Linear,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub supports_health_check: bool,
    pub supports_status_sync: bool,
    pub supports_dry_run: bool,
    pub supports_labels: bool,
    pub supports_relationships: bool,
    pub supports_lifecycle: bool,
    pub supports_milestone: bool,
    pub supports_project: bool,
    pub supports_assignee: bool,
    pub supports_reconciliation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub healthy: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecRecord {
    pub key: String,
    pub title: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskRecord {
    pub key: String,
    pub title: String,
    pub status: String,
    pub priority: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncResult {
    pub provider: ProviderKind,
    pub spec_remote_id: String,
    pub task_remote_ids: Vec<String>,
}

impl ProviderCapabilities {
    pub fn supports_contract_baseline(&self) -> bool {
        self.supports_labels
            && self.supports_relationships
            && self.supports_lifecycle
            && self.supports_assignee
            && self.supports_reconciliation
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub run_id: String,
    pub context_type: String,
    pub context_value: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationIdentity {
    pub owner_identity: String,
    pub assignee_identity: String,
    pub source_system_identity: String,
    pub identity_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceOfTruthMarker {
    pub field_name: String,
    pub authority: String,
    pub last_confirmed_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleMetadata {
    pub lifecycle_state: String,
    pub lifecycle_stage: String,
    pub entered_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RelationshipMetadata {
    pub parent_id: Option<String>,
    pub child_ids: Vec<String>,
    pub blocked_by_ids: Vec<String>,
    pub depends_on_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MilestoneProjectLinkage {
    pub milestone_ref: Option<String>,
    pub project_ref: Option<String>,
    pub capability_supported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationOutcome {
    Created,
    Updated,
    Unchanged,
    DriftCorrected,
    ConflictFlagged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncItemRecord {
    pub internal_key: String,
    pub remote_id: Option<String>,
    pub fingerprint: String,
    pub previous_fingerprint: Option<String>,
    pub integration_identity: IntegrationIdentity,
    pub source_of_truth: Vec<SourceOfTruthMarker>,
    pub lifecycle: LifecycleMetadata,
    pub relationships: RelationshipMetadata,
    pub canonical_labels: Vec<String>,
    pub linkage: Option<MilestoneProjectLinkage>,
    pub conflict_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncLedgerEntry {
    pub run_id: String,
    pub provider_kind: ProviderKind,
    pub item_key: String,
    pub outcome: ReconciliationOutcome,
    pub drift_detected: bool,
    pub conflict_reason: Option<String>,
    pub labels: Vec<String>,
    pub metadata_fields: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncRequest {
    pub context: ExecutionContext,
    pub spec: SpecRecord,
    pub tasks: Vec<TaskRecord>,
    pub items: Vec<SyncItemRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncPlan {
    pub provider: ProviderKind,
    pub context: ExecutionContext,
    pub spec: SpecRecord,
    pub tasks: Vec<TaskRecord>,
    pub desired_items: Vec<SyncItemRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncRunResult {
    pub provider: ProviderKind,
    pub spec_remote_id: String,
    pub task_remote_ids: Vec<String>,
    pub outcomes: Vec<SyncLedgerEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelPolicyContract {
    pub lifecycle_labels: BTreeSet<String>,
    pub relationship_labels: BTreeSet<String>,
}

pub fn canonical_label_policy() -> LabelPolicyContract {
    LabelPolicyContract {
        lifecycle_labels: [
            "lifecycle:todo",
            "lifecycle:in-progress",
            "lifecycle:blocked",
            "lifecycle:done",
        ]
        .into_iter()
        .map(ToString::to_string)
        .collect(),
        relationship_labels: [
            "rel:parent",
            "rel:child",
            "rel:blocked-by",
            "rel:depends-on",
        ]
        .into_iter()
        .map(ToString::to_string)
        .collect(),
    }
}
