# Data Model: GitHub Integration Tightening

## Stage Header

- **Stage ID**: S3-P1
- **Stage Name**: Design & Data Modeling
- **Purpose**: Define normalized provider-core entities and validation rules.
- **Inputs**: `spec.md`, `research.md`.
- **Outputs**: Entity definitions for orchestration, metadata, labels, reconciliation.

## Entities

## 1) ProviderContract
- **Fields**: `provider_kind`, `contract_version`, `capabilities`, `status`
- **Validation**:
  - `provider_kind` must be registered.
  - `contract_version` must satisfy minimum orchestrator version.
  - Required capabilities must be present before activation.
- **Relationships**:
  - One `ProviderContract` has one active `ProviderAdapter`.

## 2) ProviderAdapter
- **Fields**: `provider_kind`, `transport_mode`, `config_ref`, `health`
- **Validation**:
  - `transport_mode` must be one of declared supported adapters.
  - Health check must pass before write operations.
- **Relationships**:
  - Implements one `ProviderContract`.

## 3) OrchestrationExecutionContext
- **Fields**: `run_id`, `context_type` (`branch|local_default`), `context_value`, `timestamp`
- **Validation**:
  - `run_id` must be unique per sync execution.
  - `context_type` required; `context_value` required even for fallback marker.
- **Relationships**:
  - Attached to each `SyncLedgerEntry`.

## 4) IntegrationIdentity
- **Fields**: `owner_identity`, `assignee_identity`, `source_system_identity`, `identity_version`
- **Validation**:
  - `source_system_identity` required.
  - Missing assignee must be explicit (`unknown`) not null.
- **Relationships**:
  - Linked to `SyncItemRecord`.

## 5) SourceOfTruthMarker
- **Fields**: `field_name`, `authority` (`local|remote|merged`), `last_confirmed_at`
- **Validation**:
  - Each synchronized field must declare an authority.
- **Relationships**:
  - Many markers per `SyncItemRecord`.

## 6) LifecycleMetadata
- **Fields**: `lifecycle_state`, `lifecycle_stage`, `entered_at`, `updated_at`
- **Validation**:
  - State/stage pair must match label policy map.
- **Relationships**:
  - One-to-one with `LabelSet.lifecycle_labels`.

## 7) RelationshipMetadata
- **Fields**: `parent_id`, `child_ids[]`, `blocked_by_ids[]`, `depends_on_ids[]`
- **Validation**:
  - No self-reference.
  - Relationship IDs must resolve to known sync item mappings.
- **Relationships**:
  - One-to-one with `LabelSet.relationship_labels`.

## 8) LabelSet
- **Fields**: `lifecycle_labels[]`, `relationship_labels[]`, `provider_label_refs[]`
- **Validation**:
  - Canonical labels must map to provider-native labels or explicit unsupported result.
  - Required lifecycle/relationship labels must be present when capabilities allow.
- **Relationships**:
  - Belongs to `SyncItemRecord`.

## 9) MilestoneProjectLinkage
- **Fields**: `milestone_ref`, `project_ref`, `capability_supported`
- **Validation**:
  - If `capability_supported=false`, writes must return explicit unsupported outcome.
- **Relationships**:
  - Optional one-to-one with `SyncItemRecord`.

## 10) SyncItemRecord
- **Fields**: `internal_key`, `remote_id`, `fingerprint`, `last_outcome`, `last_synced_at`
- **Validation**:
  - `internal_key` + `provider_kind` unique.
  - `fingerprint` required for idempotency comparison.
- **Relationships**:
  - Has one `IntegrationIdentity`, `LifecycleMetadata`, `RelationshipMetadata`, `LabelSet`, optional `MilestoneProjectLinkage`.

## 11) SyncLedgerEntry
- **Fields**: `run_id`, `provider_kind`, `item_key`, `outcome`, `drift_detected`, `conflict_reason`
- **Validation**:
  - `outcome` enum: `created|updated|unchanged|drift_corrected|conflict_flagged`.
  - `conflict_reason` required when outcome is `conflict_flagged`.
- **Relationships**:
  - Many entries per `OrchestrationExecutionContext`.

## State Transitions

### Reconciliation outcome transition
`created|updated|unchanged|drift_corrected|conflict_flagged` are terminal per run; next run recalculates from latest fingerprint and remote state.

### Lifecycle transition
Lifecycle state changes must trigger synchronized lifecycle label updates in the same transaction boundary where provider supports labels.
