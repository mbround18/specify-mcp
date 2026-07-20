pub mod contract;
pub mod errors;
pub mod models;
pub mod registry;
pub mod usecases;

pub use contract::Provider;
pub use errors::ProviderError;
pub use models::{
    canonical_label_policy, ExecutionContext, IntegrationIdentity, LifecycleMetadata,
    MilestoneProjectLinkage, ProviderCapabilities, ProviderHealth, ProviderKind,
    ReconciliationOutcome, RelationshipMetadata, SourceOfTruthMarker, SpecRecord, SyncItemRecord,
    SyncLedgerEntry, SyncPlan, SyncRequest, SyncResult, SyncRunResult, TaskRecord,
};
pub use registry::ProviderRegistry;
