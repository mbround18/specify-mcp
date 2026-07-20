---

description: "Task list for Graph-Driven Codebase Health Improvements"
---

# Tasks: Graph-Driven Codebase Health Improvements

**Input**: Design documents from `/specs/003-graph-driven-code-health/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: No new test suites are added — the spec's FR-008 requires the existing `make test-us1`/`test-us2`/`test-us3`/`test-conformance` regression suites to keep passing, so verification tasks re-run those instead of writing new ones.

**Organization**: Tasks are grouped by user story (US1 → US2 → US3, matching spec.md priority order) so each can be delivered and verified independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)

## Path Conventions

Existing Rust Cargo workspace (see plan.md's Project Structure) — no new directories. All paths below are repo-root-relative.

---

## Phase 1: Setup

**Purpose**: Establish a clean baseline before any change.

- [x] T001 Run `make test` from repo root and confirm all targets pass on the unmodified `main` baseline; record any pre-existing failures separately so they are never attributed to this feature's changes.

---

## Phase 2: Foundational

**Purpose**: Blocking prerequisites shared by multiple stories.

None — this feature audits and refactors existing code in place; there is no new shared infrastructure that US1/US2/US3 depend on. Each story's audit artifact (see data-model.md) is self-contained. Proceed directly to Phase 3.

---

## Phase 3: User Story 1 - Verify and simplify the tasks-to-issues coupling (Priority: P1) 🎯 MVP

**Goal**: Produce a verified, documented dependency list for `run_taskstoissues()` and remove any call that bypasses the existing `ProviderRegistry`/`Provider` abstraction.

**Independent Test**: Read `apps/specify-mcp/src/mcp/taskstoissues.rs`'s Dependency Audit Record, confirm every call is classified, then run `make test-us1` and (optionally) re-run graphify to confirm reduced betweenness.

### Implementation for User Story 1

- [x] T002 [US1] Trace every outgoing call inside `run_taskstoissues()` in `apps/specify-mcp/src/mcp/taskstoissues.rs` (currently: `discover_config`, `build_registry`, `resolve_mode_with_override`, `select_provider_with_override`, `parse_scope`, `resolve_feature_artifacts`, `read_spec_title`, `parse_tasks`, `apply_scope`, `chrono_like_timestamp`, plus any calls further down the 492-line file not yet reviewed) and classify each as `intentional-direct`, `intentional-via-registry`, `accidental-bypass`, or `false-inference` per data-model.md's Dependency Audit Record schema. Write the table to `specs/003-graph-driven-code-health/audit-notes/run-taskstoissues-dependencies.md`.
- [x] T003 [US1] Cross-reference the audit table from T002 against the graph's 10 previously-INFERRED edges (e.g. relationships to `call_tool()` and `discover_config()` noted in the 2026-07-20 GRAPH_REPORT.md) and mark each `confirmed` or `refuted` in the same file. (depends on T002)
- [x] T004 [US1] For every call classified `accidental-bypass` in T002/T003, refactor the call site in `apps/specify-mcp/src/mcp/taskstoissues.rs` (and/or `apps/specify-mcp/src/orchestrator/provider.rs` if the fix belongs on the registry side) to route through `ProviderRegistry`/`Provider`; if a bypass cannot be cleanly fixed without breaking the `taskstoissues-tool-stability-contract.md`, document it as an accepted exception in the audit file instead of refactoring. (depends on T003)
- [x] T005 [US1] Run `make test-us1` (covers `crates/specify-provider-core/tests/orchestration_dispatch.rs` and `apps/specify-mcp/tests/taskstoissues_provider_select.rs`) and confirm both pass with zero regressions. (depends on T004)
- [x] T006 [P] [US1] If graphify is available, re-run `/graphify --update` and record `run_taskstoissues()`'s new edge count/betweenness in `specs/003-graph-driven-code-health/audit-notes/run-taskstoissues-dependencies.md` against the 2026-07-20 baseline (22 edges, rank #1 betweenness) to close out SC-001; otherwise note that the metric was not re-measured and why. (depends on T005)

**Checkpoint**: User Story 1 is independently complete and testable — `run_taskstoissues()`'s dependencies are fully verified and documented, `make test-us1` is green.

---

## Phase 4: User Story 2 - Consolidate provider error handling (Priority: P2)

**Goal**: Confirm `ProviderError`'s 8 variants are used only for genuine provider-boundary failures across all call sites, migrating any misuse to a more specific existing error type, per `contracts/provider-error-contract.md`.

**Independent Test**: Read the doc comments on `ProviderError` in `crates/specify-provider-core/src/errors.rs`, confirm they match the contract table, then run `make test-us2` and `make test-conformance`.

### Implementation for User Story 2

- [x] T007 [P] [US2] Add a one-line "when to use" doc comment to each of the 8 `ProviderError` variants in `crates/specify-provider-core/src/errors.rs`, matching the ownership table in `specs/003-graph-driven-code-health/contracts/provider-error-contract.md`.
- [x] T008 [P] [US2] Audit the three `ProviderError::Internal` construction sites in `crates/specify-provider-github/src/lib.rs` (unresolved `GithubAdapter::Auto` cases, around lines 127, 140, 157) against the contract — `Internal` is documented as provider-core-only, so classify each as `genuine-provider-boundary` (contract needs updating to allow this) or `misused-internal-error` (needs migration to `Unsupported` or another existing variant), and record the classification in `specs/003-graph-driven-code-health/audit-notes/provider-error-sites.md`.
- [x] T009 [P] [US2] Audit the `ProviderError::Internal` construction sites in `apps/specify-mcp/src/orchestrator/provider.rs`'s `LocalProvider` implementation (local filesystem I/O failures writing spec/task files, around lines 162–181) against the contract; classify and record in the same audit file as T008.
- [x] T010 [P] [US2] Audit the remaining `ProviderError` construction/match sites in `crates/specify-provider-core/src/registry.rs`, `crates/specify-provider-core/src/contract.rs`, `crates/specify-provider-core/src/usecases/sync.rs`, and `crates/specify-provider-linear/src/lib.rs` for variant correctness per the contract table; record in the same audit file as T008.
- [x] T011 [US2] Based on the classifications from T008–T010, either (a) update `specs/003-graph-driven-code-health/contracts/provider-error-contract.md` to explicitly permit the confirmed genuine cases (e.g. allow provider crates to use `Internal` for adapter-resolution logic errors that are not attributable to any specific provider call), or (b) migrate each `misused-internal-error` call site to a more specific existing `ProviderError` variant or other existing error type. (depends on T008, T009, T010)
- [x] T012 [US2] Run `make test-us2` (covers `crates/specify-provider-core/tests/metadata_required_fields.rs` and `crates/specify-provider-core/tests/label_policy_contract.rs`) and `make test-conformance` (covers `tests/conformance/`) and confirm all pass with zero regressions. (depends on T011)

**Checkpoint**: User Stories 1 AND 2 both work independently — `ProviderError` has a documented, audited contract and `make test-us2`/`make test-conformance` are green.

---

## Phase 5: User Story 3 - Document and connect the orphaned shell script utilities (Priority: P3)

**Goal**: Every `.specify/scripts/bash/` utility states which speckit command(s) invoke it.

**Independent Test**: Read the header of each script and correctly state its invoking command(s) without opening the speckit agent files first.

### Implementation for User Story 3

- [x] T013 [P] [US3] Add a header comment to `.specify/scripts/bash/common.sh` stating it is sourced by `check-prerequisites.sh`, `create-new-feature.sh`, `setup-plan.sh`, and `setup-tasks.sh` as part of the `speckit.specify`/`speckit.plan`/`speckit.tasks`/`speckit.clarify`/`speckit.analyze`/`speckit.implement` workflow.
- [x] T014 [P] [US3] Add a header comment to `.specify/scripts/bash/check-prerequisites.sh` stating which speckit command(s) invoke it (confirm by grepping `.github/agents/speckit.*.agent.md` for its filename before writing the comment).
- [x] T015 [P] [US3] Add a header comment to `.specify/scripts/bash/create-new-feature.sh` stating it backs the `speckit.specify` command's feature-directory creation step.
- [x] T016 [US3] If graphify is available, re-run `/graphify --update` and confirm the three scripts are no longer flagged as weakly-connected outliers in `GRAPH_REPORT.md`; if they still are, note that this is expected (they are entry points invoked by shell, not by other code, so some residual low connectivity is normal) rather than treating it as a failure. (depends on T013, T014, T015)

**Checkpoint**: All three user stories are independently functional and verified.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final cross-story validation.

- [x] T017 Run `make test` (full suite: `test-rust`, `test-config`, `test-status`, `test-guide`, `test-init`, `test-sync-sample`, `test-github-graphql-dryrun`, `test-conformance`) and confirm zero regressions across all three stories, satisfying SC-005. (depends on T006, T012, T016)
- [x] T018 [P] Update `specs/003-graph-driven-code-health/quickstart.md` if any verification command or expected outcome changed while implementing T002–T016.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: Empty — no blocking work.
- **User Stories (Phase 3–5)**: Each depends only on Phase 1 (clean baseline). US1, US2, and US3 touch disjoint files (`taskstoissues.rs`/`provider.rs` vs. `errors.rs`+provider crates vs. `.specify/scripts/bash/`) and can proceed in parallel or in priority order (P1 → P2 → P3).
- **Polish (Phase 6)**: Depends on all three stories' final verification tasks (T006, T012, T016).

### Within Each User Story

- Audit/classify before refactor/migrate.
- Refactor/migrate before running that story's regression suite.
- Story's regression suite green before its optional graphify re-check.

### Parallel Opportunities

- T007, T008, T009, T010 (US2 audits, different files/sections) can run in parallel.
- T013, T014, T015 (US3 header comments, different files) can run in parallel.
- T006 (US1 graphify check) can run in parallel with US2/US3 work once T005 is done.
- T018 (quickstart update) can run in parallel with T017.

---

## Parallel Example: User Story 2

```bash
# Launch all four ProviderError audit tasks together (different files/sections):
Task: "Add doc comments to ProviderError variants in crates/specify-provider-core/src/errors.rs"
Task: "Audit ProviderError::Internal sites in crates/specify-provider-github/src/lib.rs"
Task: "Audit ProviderError::Internal sites in apps/specify-mcp/src/orchestrator/provider.rs"
Task: "Audit remaining ProviderError sites in registry.rs, contract.rs, usecases/sync.rs, and specify-provider-linear/src/lib.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001).
2. Complete Phase 3: User Story 1 (T002–T006).
3. **STOP and VALIDATE**: `make test-us1` green, dependency audit record complete.
4. This alone delivers SC-001/SC-002 and is a shippable increment.

### Incremental Delivery

1. Setup → clean baseline confirmed.
2. Add User Story 1 → verify independently → MVP.
3. Add User Story 2 → verify independently (`make test-us2`, `make test-conformance`).
4. Add User Story 3 → verify independently.
5. Phase 6 Polish → full `make test` run, zero regressions (SC-005).

---

## Notes

- No new test files are introduced; verification relies entirely on the existing `make test-us1`/`test-us2`/`test-us3`/`test-conformance`/`test` targets per FR-008.
- Audit findings are recorded in `specs/003-graph-driven-code-health/audit-notes/` (new directory, created by T002/T008) rather than in `spec.md` itself, keeping the spec stable while the audit's working notes evolve.
- If T004's or T011's proposed fix would break `contracts/taskstoissues-tool-stability-contract.md`, stop and document the limitation per spec.md's Edge Cases section rather than forcing a breaking change (FR-009).
- Commit after each checkpoint (end of Phase 3, 4, 5, 6) rather than after every individual task, since several tasks in this feature are read-only audits with no diff to commit on their own.
