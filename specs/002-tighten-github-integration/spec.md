# Feature Specification: GitHub Integration Tightening

**Feature Branch**: `002-tighten-github-integration`

**Created**: 2026-07-12

**Status**: Draft

**Input**: User description: "GitHub Integration Tightening: enforce provider-core/base-driven orchestration, provider-specific adapters behind shared contracts, stronger integration metadata, first-class labels/relationships, explicit idempotency and reconciliation."

## Stage Header

- **Stage ID**: S2
- **Stage Name**: Specification
- **Purpose**: Define user outcomes, functional requirements, constraints, and measurable success criteria.
- **Entry Criteria**: Feature intent is captured and provider/integration outcomes are known.
- **Inputs**: User request, existing provider architecture direction, repository conventions.
- **Outputs**: `spec.md` with user scenarios, requirements, edge cases, entities, success criteria, and assumptions.
- **Exit Criteria**: Requirements are testable, metadata scope is explicit, and reconciliation/idempotency behavior is unambiguous.
- **Owner**: Product/architecture authoring flow (human + MCP assistant).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Pivot providers without orchestration rewrites (Priority: P1)

A maintainer needs task-to-issues orchestration to remain stable while switching source systems (GitHub today, Linear or another provider later). They can switch providers by configuration and provider registration, without rewriting orchestration logic.

**Why this priority**: Provider pivotability is the core strategic goal and prevents lock-in to any one integration.

**Independent Test**: Configure two providers that both implement shared contracts, run orchestration for the same spec/task set, and verify orchestration behavior is unchanged while provider-specific outputs differ only by target system mapping.

**Acceptance Scenarios**:

1. **Given** shared provider-core contracts are defined, **When** orchestration executes against GitHub and then another provider, **Then** orchestration code paths and expected orchestration outcomes remain consistent.
2. **Given** a provider is disabled or replaced, **When** orchestration runs, **Then** the system routes through the active provider adapter without orchestration rewrites.

---

### User Story 2 - Enrich metadata and labels for reliable traceability (Priority: P1)

A project lead needs each synchronized work item to carry strong execution, ownership, lifecycle, and relationship metadata so reruns, audits, and cross-system tracking remain reliable.

**Why this priority**: Metadata quality is required for trustworthy reconciliation and operational visibility.

**Independent Test**: Run sync for a parent/child task graph, inspect resulting records, and verify required execution context, ownership identity, source-of-truth markers, lifecycle metadata, lifecycle labels, relationship labels, and milestone/project linkage fields are present and consistent.

**Acceptance Scenarios**:

1. **Given** task orchestration runs in branch context, **When** remote items are created or updated, **Then** each item records execution context and ownership/source identity metadata.
2. **Given** parent/child or dependency links exist in the task graph, **When** sync executes, **Then** relationship metadata and relationship labels are applied consistently on all affected items.
3. **Given** milestone or project linkage is available in the source system, **When** synchronization occurs, **Then** linkage metadata is captured and maintained without removing existing valid linkage.

---

### User Story 3 - Re-run safely with explicit reconciliation (Priority: P2)

A delivery engineer re-runs task-to-issues synchronization after edits, partial failures, or interrupted runs and expects no duplicates, no drift, and clear reconciliation outcomes.

**Why this priority**: Idempotent reruns are essential for daily operations and confidence in automation.

**Independent Test**: Execute synchronization three times with no changes, then with intentional task metadata changes, and confirm no duplicate items are created and reconciliation reports expected updates only.

**Acceptance Scenarios**:

1. **Given** the same orchestration input is re-run, **When** sync executes repeatedly, **Then** the resulting remote state remains stable with no duplicate parent or child items.
2. **Given** remote state has diverged from expected metadata or labels, **When** reconciliation runs, **Then** the system identifies drift and applies deterministic updates.
3. **Given** a prior run failed after partial writes, **When** sync resumes, **Then** the system reconciles existing items instead of recreating them.

### Edge Cases

- Provider-core contracts evolve and one provider adapter is temporarily behind the new contract version.
- A task has both blocked-by and depends-on relationships that point to items in different lifecycle states.
- Source system identity changes (for example reassignment or account rename) between sync runs.
- Execution context is unavailable (no explicit branch context) and system must fall back to default/local context markers.
- Milestone/project linkage exists in one provider but is unsupported in another provider selected later.
- Manual remote edits remove labels that orchestration considers required; reconciliation must restore or flag drift deterministically.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST route task-to-issues orchestration through shared provider-core/base contracts rather than provider-specific orchestration branches.
- **FR-002**: System MUST allow provider switching through configuration/registration without requiring orchestration rewrites.
- **FR-003**: Each provider MUST implement its own concrete adapter/transport behavior behind shared orchestration interfaces.
- **FR-004**: System MUST validate provider contract conformance before provider activation and return actionable errors when conformance fails.
- **FR-005**: System MUST capture execution context metadata for each synchronization run, including explicit context type (branch or local/default) and context value.
- **FR-006**: System MUST capture ownership and source-system identity metadata for synchronized items, including current assignee/owner identity where available.
- **FR-007**: System MUST persist source-of-truth markers that identify which fields are authoritative for reconciliation.
- **FR-008**: System MUST persist lifecycle metadata for synchronized items, including lifecycle state/stage and lifecycle timestamps for key transitions.
- **FR-009**: System MUST treat lifecycle labels as first-class metadata and keep them synchronized with lifecycle state/stage.
- **FR-010**: System MUST treat relationship labels (including parent, child, blocked-by, and depends-on) as first-class metadata and keep them synchronized with relationship records.
- **FR-011**: System MUST preserve and synchronize milestone/project linkage metadata when the active provider supports those constructs.
- **FR-012**: System MUST preserve relationship integrity across parent and child items during create, update, and reconciliation operations.
- **FR-013**: System MUST perform idempotent synchronization such that repeated runs with unchanged input do not create duplicates or unintended changes.
- **FR-014**: System MUST run reconciliation on each synchronization pass to detect and resolve drift between expected and remote states.
- **FR-015**: System MUST emit deterministic reconciliation outcomes (created, updated, unchanged, drift-corrected, conflict-flagged) for each synchronized item.
- **FR-016**: System MUST support recovery from partial failures by reusing previously created remote items based on persisted identity and source-of-truth metadata.
- **FR-017**: System MUST maintain a stable mapping between internal task records and remote records that is provider-agnostic at orchestration level.
- **FR-018**: System MUST maintain compatibility with existing task-to-issues orchestration semantics while adding strengthened metadata and labels behavior.

### Key Entities *(include if feature involves data)*

- **ProviderContract**: Shared orchestration-facing contract that defines required provider capabilities, operations, and conformance rules.
- **ProviderAdapter**: Provider-specific implementation of transport and remote-system behavior behind the shared contract.
- **OrchestrationExecutionContext**: Metadata object capturing execution scope (branch vs local/default), context identifier, and run identity.
- **IntegrationIdentity**: Metadata object capturing assignee/owner identity and source system identity attributes used for tracing and reconciliation.
- **SourceOfTruthMarker**: Metadata marker identifying which fields are authoritative and which are synchronized projections.
- **LifecycleMetadata**: Record of lifecycle state/stage plus transition timestamps and status provenance.
- **RelationshipMetadata**: Record of parent/child/blocked-by/depends-on links between synchronized items.
- **LabelSet**: First-class set of lifecycle and relationship labels attached to synchronized records.
- **MilestoneProjectLinkage**: Metadata linking a synchronized item to milestone/project containers when supported by the active provider.
- **SyncLedger**: Persistent idempotency and reconciliation ledger storing stable mappings, last-known fingerprints, and reconciliation outcomes.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: For a representative spec with 1 parent and 20 tasks, three consecutive reruns with unchanged input produce 0 duplicate remote items and 0 unintended field changes.
- **SC-002**: In provider pivot tests between at least two providers implementing shared contracts, orchestration behavior remains functionally equivalent in 100% of defined contract test scenarios.
- **SC-003**: 100% of synchronized items include required execution context, identity, source-of-truth, lifecycle, and relationship metadata fields.
- **SC-004**: 100% of items with lifecycle state have corresponding lifecycle labels; 100% of linked parent/child/dependency relations have corresponding relationship labels.
- **SC-005**: For items where milestone/project linkage exists in provider capabilities, 100% of synchronized records preserve or correctly update linkage across reruns.
- **SC-006**: During controlled drift tests (manual remote edits), reconciliation restores expected metadata/labels or flags conflicts in 100% of cases within a single rerun.
- **SC-007**: At least 90% of pilot users report that reconciliation outputs are clear enough to determine whether a rerun is safe without additional investigation.

## Assumptions

- Existing orchestration semantics for parent/child task structures remain in scope and should not be reduced.
- Multiple providers can implement shared contracts, but one provider is active per run.
- Provider-specific differences in remote system capabilities are handled through contract-defined capability flags rather than orchestration forks.
- Ownership and source identity values are available from configured provider contexts for most runs; when unavailable, explicit unknown/default markers are acceptable.
- Lifecycle and relationship label vocabularies are standardized enough to map consistently across supported providers.

## MCP Implementation Gaps & Required Fixes

1. **Gap: Orchestration remains GitHub-coupled in task-to-issues execution.**  
   **Required Fix:** Move task-to-issues orchestration into provider-core use-cases and traits (`plan_sync`, `apply_sync`) so providers only supply implementations, not orchestration branches.

2. **Gap: Task-to-issues execution is not provider-selectable at runtime.**  
   **Required Fix:** Add provider selection to command/tool arguments and config, then dispatch through registry (`local|github|linear|future`) via one provider-agnostic orchestration entrypoint.

3. **Gap: Integration metadata model is incomplete for context/identity/source-of-truth.**  
   **Required Fix:** Add first-class provider-core entities for `ExecutionContext`, `IntegrationIdentity`, `SourceOfTruthMarker`, `LifecycleMetadata`, and `RelationshipMetadata`.

4. **Gap: Label behavior is ad hoc and not policy-driven.**  
   **Required Fix:** Define a provider-agnostic label policy contract for lifecycle and relationship vocabularies, with provider-specific mapping layers.

5. **Gap: Milestone/project linkage is not a first-class reconciliation dimension.**  
   **Required Fix:** Add linkage fields and capability flags in provider-core; require explicit unsupported behavior instead of implicit/no-op handling.

6. **Gap: Reconciliation output is not uniformly deterministic at item level.**  
   **Required Fix:** Standardize per-item reconciliation outcomes (`created`, `updated`, `unchanged`, `drift_corrected`, `conflict_flagged`) across providers.

7. **Gap: Status diagnostics do not expose full provider capability posture.**  
   **Required Fix:** Extend status output with structured capability matrix (read/write, labels, relationships, milestone/project, assignee support) for active provider and optional alternates.

8. **Gap: No cross-provider conformance suite for task-to-issues parity.**  
   **Required Fix:** Add provider-core contract tests that run shared orchestration fixtures against GitHub and Linear implementations and enforce parity thresholds.
