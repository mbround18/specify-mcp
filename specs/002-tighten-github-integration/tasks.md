# Tasks: GitHub Integration Tightening

**Input**: Design documents from `/specs/002-tighten-github-integration/`

**Prerequisites**: `plan.md` (required), `spec.md` (required), `research.md`, `data-model.md`, `contracts/`, `quickstart.md`

## Stage Header

- **Stage ID**: S4
- **Stage Name**: Tasking
- **Purpose**: Convert provider-pivot and reconciliation requirements into dependency-aware, executable implementation work.
- **Entry Criteria**: `spec.md` and `plan.md` are complete with contract/data-model artifacts.
- **Inputs**: `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/*`, `quickstart.md`.
- **Outputs**: `tasks.md` with phased implementation, dependencies, and validation checkpoints.
- **Exit Criteria**: Team can execute tasks in order with independent story-level validation.
- **Owner**: Delivery orchestration flow (human + MCP assistant).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Task is parallelizable (different files, no incomplete dependency)
- **[Story]**: User story association (`[US1]`, `[US2]`, `[US3]`)
- Every task includes an executable action and concrete file path(s)

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare scaffolding and test harness files needed by all stories.

- [X] T001 Create cross-provider conformance test module scaffold in tests/conformance/mod.rs
- [X] T002 Create reusable provider fixture builders for parent+20 graph in tests/conformance/fixtures/task_graph.rs
- [X] T003 [P] Add conformance test package wiring in Cargo.toml and crates/specify-provider-core/Cargo.toml
- [X] T004 [P] Document feature validation command set in specs/002-tighten-github-integration/quickstart.md

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core provider-core and app wiring that must exist before any user story implementation.

**⚠️ CRITICAL**: Complete this phase before starting user story phases.

- [X] T005 Define provider-core orchestration trait surface (`plan_sync`, `apply_sync`) in crates/specify-provider-core/src/contract.rs
- [X] T006 [P] Add provider capability matrix model and validation helpers in crates/specify-provider-core/src/models.rs
- [X] T007 Implement registry dispatch for selected provider kind in crates/specify-provider-core/src/registry.rs
- [X] T008 Implement provider selection parsing (`local|github|linear`) in apps/specify-mcp/src/config/schema.rs and apps/specify-mcp/src/config/validation.rs
- [X] T009 Wire provider selection through taskstoissues entrypoint in apps/specify-mcp/src/mcp/taskstoissues.rs and apps/specify-mcp/src/orchestrator/provider.rs
- [X] T010 Implement activation-time contract conformance failure errors in crates/specify-provider-core/src/errors.rs and apps/specify-mcp/src/orchestrator/provider.rs

**Checkpoint**: Provider selection + contract activation baseline complete.

---

## Phase 3: User Story 1 - Pivot providers without orchestration rewrites (Priority: P1) 🎯 MVP

**Goal**: Make task-to-issues orchestration provider-core driven and provider-selectable at runtime without app-level provider branches.

**Independent Test**: Run the same task graph through GitHub and Linear with provider switch config only; verify orchestration flow and outcome categories are equivalent.

### Tests for User Story 1

- [X] T011 [P] [US1] Add provider-core orchestration dispatch tests in crates/specify-provider-core/tests/orchestration_dispatch.rs
- [X] T012 [P] [US1] Add provider-selectable taskstoissues integration tests in apps/specify-mcp/tests/taskstoissues_provider_select.rs

### Implementation for User Story 1

- [X] T013 [US1] Implement provider-core sync use case module in crates/specify-provider-core/src/usecases/sync.rs and crates/specify-provider-core/src/lib.rs
- [X] T014 [P] [US1] Implement GitHub adapter against shared sync contract in crates/specify-provider-github/src/lib.rs
- [X] T015 [P] [US1] Implement Linear adapter against shared sync contract in crates/specify-provider-linear/src/lib.rs
- [X] T016 [US1] Remove provider-specific branching from taskstoissues orchestration in apps/specify-mcp/src/mcp/taskstoissues.rs
- [X] T017 [US1] Add runtime provider override plumbing for taskstoissues command args in apps/specify-mcp/src/mcp/workflows.rs and apps/specify-mcp/src/config/discovery.rs

**Checkpoint**: US1 is independently testable and delivers provider pivotability MVP.

---

## Phase 4: User Story 2 - Enrich metadata and labels for reliable traceability (Priority: P1)

**Goal**: Introduce first-class metadata objects and canonical label policy synchronization across providers.

**Independent Test**: Sync a parent/child graph and confirm execution context, identity, source-of-truth, lifecycle/relationship metadata, canonical labels, and milestone/project linkage behavior are complete and consistent.

### Tests for User Story 2

- [X] T018 [P] [US2] Add metadata required-field validation tests in crates/specify-provider-core/tests/metadata_required_fields.rs
- [X] T019 [P] [US2] Add lifecycle/relationship label policy contract tests in crates/specify-provider-core/tests/label_policy_contract.rs

### Implementation for User Story 2

- [X] T020 [US2] Add `ExecutionContext`, `IntegrationIdentity`, and `SourceOfTruthMarker` models in crates/specify-provider-core/src/models.rs
- [X] T021 [US2] Add `LifecycleMetadata`, `RelationshipMetadata`, and `MilestoneProjectLinkage` models in crates/specify-provider-core/src/models.rs
- [X] T022 [US2] Implement canonical lifecycle/relationship label vocab contract in crates/specify-provider-core/src/contract.rs
- [X] T023 [P] [US2] Implement GitHub canonical label mapping + linkage handling in crates/specify-provider-github/src/lib.rs
- [X] T024 [P] [US2] Implement Linear canonical label mapping + capability-gated linkage handling in crates/specify-provider-linear/src/lib.rs
- [X] T025 [US2] Persist metadata and label state in sync ledger records in apps/specify-mcp/src/orchestrator/ledger.rs
- [X] T026 [US2] Enforce metadata↔label consistency checks during `apply_sync` in crates/specify-provider-core/src/usecases/sync.rs

**Checkpoint**: US2 metadata and label traceability are independently verifiable.

---

## Phase 5: User Story 3 - Re-run safely with explicit reconciliation (Priority: P2)

**Goal**: Deliver deterministic item-level reconciliation outcomes, idempotent reruns, and resume-safe partial failure behavior with cross-provider parity coverage.

**Independent Test**: Execute three unchanged reruns, then drift and partial-failure scenarios, and verify no duplicates plus deterministic per-item outcomes (`created|updated|unchanged|drift_corrected|conflict_flagged`) across GitHub and Linear fixtures.

### Tests for User Story 3

- [X] T027 [P] [US3] Add idempotent triple-run fixture test in tests/conformance/idempotency_three_runs.rs
- [X] T028 [P] [US3] Add drift correction vs conflict-flag reconciliation test in tests/conformance/drift_reconciliation.rs
- [X] T029 [P] [US3] Add partial failure resume fixture test in tests/conformance/partial_failure_resume.rs
- [X] T030 [P] [US3] Add cross-provider parity conformance test harness in tests/conformance/provider_parity.rs

### Implementation for User Story 3

- [X] T031 [US3] Add `ReconciliationOutcome` enum + conflict reason payloads in crates/specify-provider-core/src/models.rs
- [X] T032 [US3] Implement deterministic fingerprint/outcome evaluator in crates/specify-provider-core/src/usecases/sync.rs
- [X] T033 [US3] Persist per-item reconciliation outcomes per run in apps/specify-mcp/src/orchestrator/ledger.rs
- [X] T034 [US3] Implement resume-from-ledger identity reuse flow in apps/specify-mcp/src/mcp/taskstoissues.rs
- [X] T035 [US3] Emit structured reconciliation outcome summary in apps/specify-mcp/src/mcp/server.rs
- [X] T036 [US3] Enforce provider activation gate on conformance suite status in apps/specify-mcp/src/orchestrator/provider.rs

**Checkpoint**: US3 rerun safety and reconciliation determinism are independently testable.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final hardening, docs alignment, and regression execution across all stories.

- [X] T037 Update provider-core, label policy, and conformance contract docs to match implementation in specs/002-tighten-github-integration/contracts/provider-core-sync-contract.md, specs/002-tighten-github-integration/contracts/label-policy-contract.md, and specs/002-tighten-github-integration/contracts/conformance-suite-contract.md
- [X] T038 [P] Add reproducible conformance and story validation targets in Makefile
- [X] T039 Run full regression and capture verification commands/results in specs/002-tighten-github-integration/quickstart.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: no dependencies
- **Phase 2 (Foundational)**: depends on Phase 1; blocks all user stories
- **Phase 3 (US1)**: depends on Phase 2
- **Phase 4 (US2)**: depends on Phase 2 and on US1 sync use-case scaffold (T013) for shared apply path
- **Phase 5 (US3)**: depends on US1 orchestration pivot + US2 metadata model/label policy completion
- **Phase 6 (Polish)**: depends on all user story phases

### User Story Completion Order

1. **US1 (P1)**: provider-core orchestration pivot + provider-selectable taskstoissues (MVP)
2. **US2 (P1)**: metadata model and label policy hardening
3. **US3 (P2)**: reconciliation determinism, idempotent recovery, cross-provider conformance

### Within-Story Dependency Rules

- Tests first, then implementation
- Core provider-core model/contract updates before provider adapter updates
- Ledger persistence updates before reconciliation reporting output

## Parallel Opportunities

- **Setup**: T003 and T004 can run in parallel after T001/T002 start.
- **US1**: T014 and T015 can run in parallel after T013; T011 and T012 can run in parallel.
- **US2**: T023 and T024 can run in parallel after T022; T018 and T019 can run in parallel.
- **US3**: T027-T030 can run in parallel; T033 and T035 can run in parallel after T032.

## Parallel Example: User Story 1

```bash
# Parallel tests
Task: "T011 [US1] crates/specify-provider-core/tests/orchestration_dispatch.rs"
Task: "T012 [US1] apps/specify-mcp/tests/taskstoissues_provider_select.rs"

# Parallel provider adapter work
Task: "T014 [US1] crates/specify-provider-github/src/lib.rs"
Task: "T015 [US1] crates/specify-provider-linear/src/lib.rs"
```

## Parallel Example: User Story 2

```bash
# Parallel tests
Task: "T018 [US2] crates/specify-provider-core/tests/metadata_required_fields.rs"
Task: "T019 [US2] crates/specify-provider-core/tests/label_policy_contract.rs"

# Parallel provider mappings
Task: "T023 [US2] crates/specify-provider-github/src/lib.rs"
Task: "T024 [US2] crates/specify-provider-linear/src/lib.rs"
```

## Parallel Example: User Story 3

```bash
# Parallel conformance fixtures
Task: "T027 [US3] tests/conformance/idempotency_three_runs.rs"
Task: "T028 [US3] tests/conformance/drift_reconciliation.rs"
Task: "T029 [US3] tests/conformance/partial_failure_resume.rs"
Task: "T030 [US3] tests/conformance/provider_parity.rs"
```

## Implementation Strategy

### MVP First (US1)

1. Complete Phase 1 + Phase 2.
2. Deliver Phase 3 (US1) end-to-end.
3. Validate provider pivot behavior through US1 independent test before expanding scope.

### Incremental Delivery

1. Ship US1 (provider-core orchestration pivot + provider-selectable taskstoissues).
2. Add US2 (metadata + labels) without changing US1 orchestration contract.
3. Add US3 (reconciliation determinism + conformance) as reliability hardening.
4. Complete Phase 6 polish and run full regression.
