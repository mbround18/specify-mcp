# Tasks: MCP Spec Kit Zero-Install + Pluggable Issue Providers

**Input**: Design documents from `/specs/001-mcp-provider-integrations/`

**Prerequisites**: `spec.md`, `plan.md`

## Stage Header

- **Stage ID**: S4
- **Stage Name**: Tasking
- **Purpose**: Convert plan and requirements into dependency-aware, executable work items.
- **Entry Criteria**: Specification and planning artifacts are complete and current.
- **Inputs**: `spec.md`, `plan.md`, sub-spec documents.
- **Outputs**: `tasks.md` with phased backlog, dependencies, milestones, and rollout tasks.
- **Exit Criteria**: Work can begin immediately with clear sequencing and ownership handoff.
- **Owner**: Delivery orchestration flow (human + MCP assistant).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (no file overlap/dependency)
- **[Story]**: US1/US2/US3/US4 alignment

## Program Closeout Snapshot (2026-07-12)

### Reconciled completion from `002-tighten-github-integration`

The following 001 tasks are now treated as **complete** or **superseded** based on the implemented 002 deliverables and current repository state:

- [X] T001, T002, T004, T005, T006, T007
- [X] T009, T010, T011, T012, T013, T014
- [X] T017a, T017b, T017c, T017d, T017e
- [X] T018, T020, T020a, T021, T022, T023, T024, T025, T026, T027
- [X] T030 *(superseded by MCP `init` tool-first entrypoint)*
- [X] T034
- [X] T039, T042, T043, T046, T047
- [X] T054, T057, T060, T061, T062
- [X] T063a, T063b, T063c

### v1.0.0 scope freeze (bring-it-home cut)

`v1.0.0` release scope is frozen to **M3 (GitHub GA Candidate)**, with targeted closeout on:

- [ ] T031, T032, T033, T035, T036, T037, T038
- [ ] T040, T041, T044, T045, T048, T049, T053
- [ ] T070, T071, T072, T073, T074, T075, T076
- [ ] T079, T080, T081, T082, T083
- [ ] T085

### Deferred to V2 (explicitly out of v1.0.0)

- [ ] T055, T056, T058, T059, T063 *(Linear perf/expansion tail)*
- [ ] T064, T065, T066, T067, T068, T069 *(extension/Jira path)*
- [ ] T077, T078, T084, T086 *(post-GA hardening/adoption backlog)*

## Phase 1: Program Setup & Baseline Architecture

**Goal**: Establish project skeleton, quality gates, and shared architecture.

- [ ] T001 Create Rust workspace layout with `apps/specify-mcp` and `crates/*` supporting crates
- [ ] T002 Define crate/module boundaries for `specify-provider-core`, `specify-provider-github`, `specify-provider-linear`
- [ ] T003 [P] Add CI baseline for formatting, linting, unit tests
- [ ] T004 [P] Add fixture test harness for provider simulation and replay
- [ ] T005 Define canonical domain models (`SpecRecord`, `TaskRecord`, `SyncLedger`, `ProviderCapability`)
- [ ] T006 Define error taxonomy and propagation rules (no silent fallbacks)
- [ ] T007 Create architecture decision notes for adapter split (GitHub GraphQL vs MCP)
- [ ] T008 Create telemetry contract (events/log fields) for setup and sync operations

**Checkpoint**: Shared architecture and quality gates established.

---

## Phase 2: Configuration Discovery, Validation, and Policy Engine

**Goal**: Deterministic config resolution and strict validation.

- [ ] T009 Implement config loader for `specify.toml` at repository root
- [ ] T010 [P] Implement config loader for `.github/config/specify.toml`
- [ ] T011 [P] Implement config loader for `.github/specify.toml`
- [ ] T012 [P] Implement `.github/specify*.toml` lexical fallback scan
- [ ] T013 Implement precedence resolver + source reporting
- [ ] T014 Implement schema validation for provider sections and adapter preferences
- [ ] T015 Implement policy engine (`mode`, fallback strategy, dry-run defaults, auth policy)
- [ ] T016 [P] Add test matrix for all precedence and conflict combinations
- [ ] T017 [P] Add invalid-config diagnostics tests with actionable messages

**Checkpoint**: Config behavior is deterministic and validated.

---

## Phase 2.5: Speckit Stage Formalization & Guided Flow

**Goal**: Let users operate by intent, not by memorizing stage commands.

- [ ] T017a Define canonical stage catalog (S0-S7) in a shared operating model doc
- [ ] T017b Implement standardized stage headers for `spec.md`, `plan.md`, and `tasks.md`
- [ ] T017c Implement stage state ledger at `.specify/metadata/stage-state.json`
- [ ] T017d Implement "next best step" inference from artifact/ledger state
- [ ] T017e Implement guided entrypoint (`mcp.speckit.guide`) to auto-select stage
- [ ] T017f [P] Add tests for stage transition rules and blocked-state handling
- [ ] T017g [P] Add tests ensuring full flow completion without explicit stage command names

**Checkpoint**: Guided mode can move users through stage flow with explicit progress tracking.

---

## Phase 3: Provider Core Contract + Registry (Foundational Blocker)

**Goal**: Stable plugin architecture for current and future providers.

- [ ] T018 Define provider trait surface (create/update/delete/list/link, health check, capabilities)
- [ ] T019 Define adapter trait surface (transport/auth/probing/request abstractions)
- [ ] T020 Define provider registry and dynamic selection strategy
- [ ] T020a Implement `crates/specify-provider-core` crate surface (`contract`, `registry`, `models`, `errors`)
- [ ] T021 [P] Implement provider capability probing contract
- [ ] T022 [P] Implement provider health checks with structured failure reasons
- [ ] T023 Implement provider selection algorithm (explicit mode > preferred providers > fallback policy)
- [ ] T024 Implement provider contract conformance suite scaffolding
- [ ] T025 Implement sync intent planner (diff local desired state vs remote actual state)
- [ ] T026 Implement sync idempotency key strategy and deterministic hashing
- [ ] T027 Implement sync ledger persistence in `.specify/metadata/sync-ledger.json`
- [ ] T028 [P] Add contract tests for unsupported-operation behavior
- [ ] T029 [P] Add contract tests for idempotent upsert requirements

**Checkpoint**: Core provider platform complete; story work can scale in parallel.

---

## Phase 4: User Story 1 - Zero-Install MCP Bootstrap (Priority: P1) 🎯 MVP

**Goal**: Setup and operate Spec Kit workflows without globally installed `specify`.

**Independent Test**: Fresh environment, no `specify` in `PATH`; run MCP setup + create spec + generate tasks.

- [ ] T030 [US1] Implement `mcp.speckit.setup` entrypoint
- [ ] T031 [US1] Implement repo-local asset installer (templates, scripts, workflow metadata)
- [ ] T032 [P] [US1] Implement setup preflight checks (filesystem permissions, path viability)
- [ ] T033 [P] [US1] Implement setup repair mode for partial installs
- [ ] T034 [US1] Implement MCP workflow command bridge for `specify/spec/plan/tasks` lifecycle
- [ ] T035 [US1] Add smoke test: setup without CLI and run first workflow
- [ ] T036 [US1] Add upgrade test: existing assets patched safely
- [ ] T037 [US1] Add failure test: interrupted setup + resume behavior
- [ ] T038 [US1] Add UX diagnostics for missing MCP runtime prerequisites

---

## Phase 5: User Story 2 - GitHub Issue Provider (Priority: P1)

**Goal**: Support GitHub-backed spec/task lifecycle with MCP/GraphQL adapter flexibility.

**Independent Test**: Create spec + 10 tasks; sync thrice; verify no duplicate issues in both adapter modes.

- [ ] T039 [US2] Implement `crates/specify-provider-github` provider domain module
- [ ] T040 [P] [US2] Implement GitHub mapping schema with parent spec issue + canonical in-issue task checklist (default mode)
- [ ] T041 [P] [US2] Implement task split policy engine for selective child issue creation (owner/risk/blocker/tag-driven)
- [ ] T042 [US2] Implement GitHub adapter probe: detect MCP availability and auth status
- [ ] T043 [US2] Implement GraphQL token resolver (`GITHUB_TOKEN` then `gh auth token`)
- [ ] T044 [P] [US2] Implement GitHub MCP adapter (create/update/query/close/reopen)
- [ ] T045 [P] [US2] Implement direct GraphQL adapter (create/update/query/close/reopen)
- [ ] T046 [US2] Implement adapter arbitration logic using config preference and capabilities
- [ ] T047 [US2] Implement incremental sync and reconciliation for parent checklist + child issue renames/splits/merges
- [ ] T048 [US2] Implement migration: local markdown spec/tasks -> GitHub issues
- [ ] T049 [US2] Implement reverse import: GitHub issue set -> local snapshot (optional safety export)
- [ ] T050 [P] [US2] Add integration tests for each adapter path
- [ ] T051 [P] [US2] Add integration tests for auth failure and permission-denied scenarios
- [ ] T052 [US2] Add idempotency regression suite (multi-run, deleted issue, edited issue title/body)
- [ ] T053 [US2] Add performance test for 1 spec / 100 tasks synchronization

---

## Phase 6: User Story 3 - Linear Provider (Priority: P2)

**Goal**: Support Linear-backed spec/task lifecycle with configurable mappings.

**Independent Test**: Create/update/reconcile one spec and 10 tasks in Linear with repeat-sync idempotency.

- [ ] T054 [US3] Implement `crates/specify-provider-linear` provider domain module
- [ ] T055 [P] [US3] Implement Linear GraphQL adapter client wrapper
- [ ] T056 [P] [US3] Implement mapping config (team/project/state/priority/labels)
- [ ] T057 [US3] Implement sync metadata mapping (external IDs, hash stamps)
- [ ] T058 [US3] Implement migration: local markdown spec/tasks -> Linear issues
- [ ] T059 [US3] Implement reverse import: Linear issues -> local snapshot (optional safety export)
- [ ] T060 [P] [US3] Add integration tests for create/update/status transitions
- [ ] T061 [P] [US3] Add integration tests for missing team/project mapping errors
- [ ] T062 [US3] Add idempotency regression suite (multi-run with manual remote edits)
- [ ] T063 [US3] Add performance test for 1 spec / 100 tasks synchronization

---

## Phase 6.5: App Crate Orchestration Wiring

**Goal**: Keep `apps/specify-mcp` as the orchestration/runtime entrypoint only.

- [ ] T063a Wire provider-core and provider crates into `apps/specify-mcp` through trait-based dependency injection
- [ ] T063b Keep provider-specific API details out of app crate modules
- [ ] T063c Add integration test verifying app can switch `local`/`github`/`linear` via config without recompilation changes

---

## Phase 7: User Story 4 - Extension-Ready Provider System (Priority: P3)

**Goal**: Make adding Jira/future providers straightforward and safe.

**Independent Test**: Register stub Jira provider with zero core orchestration edits.

- [ ] T064 [US4] Finalize provider contract test suite with required capabilities matrix
- [ ] T065 [US4] Add provider registration API and declarative registry config
- [ ] T066 [P] [US4] Add example provider template package (scaffold/starter)
- [ ] T067 [US4] Add stub Jira provider behind feature flag
- [ ] T068 [US4] Validate stub Jira provider against contract tests
- [ ] T069 [US4] Document extension guide (required traits, tests, release checklist)

---

## Phase 8: Cross-Cutting Reliability, Security, and Observability

- [ ] T070 Implement dry-run mode for all provider operations with change previews
- [ ] T071 Implement conflict policy engine (local wins, remote wins, manual resolve)
- [ ] T072 Implement resilient retry/backoff strategy for transient provider failures
- [ ] T073 Implement secret handling and redaction across logs/errors
- [ ] T074 [P] Add audit event stream for create/update/delete sync actions
- [ ] T075 [P] Add structured logs + correlation IDs for each sync run
- [ ] T076 Add end-to-end chaos tests (network interruption, rate limits, partial failures)
- [ ] T077 Add security review checklist for provider adapters and auth paths
- [ ] T078 Add backup/export command for spec/task state before migrations

---

## Phase 9: Rollout, Migration, and Adoption

- [ ] T079 Build progressive rollout flags (`local-only`, `github-beta`, `linear-beta`, `mixed`)
- [ ] T080 Define migration playbooks for local->GitHub and local->Linear
- [ ] T081 Add command/docs for safe rollback to local mode
- [ ] T082 Add team onboarding quickstart (CLI-present and zero-install paths)
- [ ] T083 Publish operator runbook for incidents and recovery
- [ ] T084 Add KPI collection dashboards for SC-001..SC-005
- [ ] T085 Run pilot with at least one GitHub-heavy and one Linear-heavy team
- [ ] T086 Capture pilot findings and prioritize V2 backlog (Jira, multi-provider federation)

---

## Dependencies & Execution Order

1. **Foundation path**: Phase 1 -> Phase 2 -> Phase 3
2. **MVP path**: Phase 4 (US1) starts after Phase 3
3. **Scale path**: Phase 5 (US2) and Phase 6 (US3) start after Phase 3 and run in parallel
4. **Extensibility path**: Phase 7 (US4) starts after Phase 3 and can overlap with Phase 5/6
5. **Stabilization path**: Phase 8 follows first completion of US1 + (US2 or US3)
6. **Adoption path**: Phase 9 follows stabilization acceptance

## Milestones

- **M1 (Foundation Ready)**: In practice complete via `002-tighten-github-integration` reconciliation
- **M2 (Zero-Install MVP)**: Partially complete; remaining zero-install hardening tracked in v1.0.0 scope freeze
- **M3 (GitHub GA Candidate)**: **v1.0.0 release target**
- **M4 (Linear GA Candidate)**: Deferred to V2
- **M5 (Extensibility Ready)**: Deferred to V2
- **M6 (Org Rollout Ready)**: Deferred to post-v1 rollout waves
