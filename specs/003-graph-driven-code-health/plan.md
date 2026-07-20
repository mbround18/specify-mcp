# Implementation Plan: Graph-Driven Codebase Health Improvements

**Branch**: `003-graph-driven-code-health` | **Date**: 2026-07-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-graph-driven-code-health/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Audit and tighten two structural hotspots identified by knowledge-graph analysis of this Rust workspace: (1) `run_taskstoissues()` in `apps/specify-mcp/src/mcp/taskstoissues.rs`, the highest-betweenness node bridging sync, config, and provider-registry concerns, and (2) `ProviderError` in `crates/specify-provider-core/src/errors.rs`, the most-connected type in the codebase, used across the GitHub/Linear provider crates and the registry/contract layer. The approach is verification-first: trace every real call/construction site with `grep`/`cargo doc`, confirm or refute the graph's inferred edges against source, and only then apply minimal, test-covered refactors (routing bypassed calls through `ProviderRegistry`, splitting misused `ProviderError` variants into more specific types). A lower-priority pass adds header comments to `.specify/scripts/bash/*.sh` documenting which speckit command each script serves.

## Technical Context

**Language/Version**: Rust 2024 edition (workspace `Cargo.toml`), rustc via `cargo` toolchain already pinned in this repo

**Primary Dependencies**: existing workspace crates only — `specify-provider-core`, `specify-provider-github`, `specify-provider-linear`, `specify-mcp` (no new crates)

**Storage**: N/A (no persistence changes; this is a code-structure/error-handling refactor)

**Testing**: `cargo test` (workspace + per-crate suites already listed in `Makefile`: `test-rust`, `test-us1`, `test-us2`, `test-us3`, `test-conformance`)

**Target Platform**: Linux/macOS developer machines and CI (MCP server binary + Cargo workspace, no platform-specific code touched)

**Project Type**: Rust Cargo workspace — MCP server app (`apps/specify-mcp`) + provider crates (`crates/specify-provider-{core,github,linear}`) + conformance test crate (`tests/conformance`)

**Performance Goals**: N/A — this is a structural/maintainability change, not a performance change; existing perf characteristics must not regress

**Constraints**: No breaking changes to public MCP tool request/response shapes (FR-009); no reduction in existing test coverage (FR-008); no new external dependencies

**Scale/Scope**: 3 provider-facing crates + 1 MCP app crate; scope is bounded to the call sites of `run_taskstoissues()`, `ProviderError` usages, and 3 shell scripts under `.specify/scripts/bash/` — not a full-codebase refactor

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

`.specify/memory/constitution.md` is still the unfilled template (`[PROJECT_NAME] Constitution` with all principle placeholders unresolved) — no ratified project-specific principles exist yet to gate against. This check is therefore satisfied vacuously; no violations to justify. If the constitution is ratified before this feature is implemented, re-run this check against the ratified version.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
apps/specify-mcp/
├── src/
│   ├── mcp/
│   │   └── taskstoissues.rs      # run_taskstoissues() — US1 target
│   ├── orchestrator/
│   │   ├── provider.rs           # ProviderRegistry / Provider trait boundary
│   │   └── guide.rs
│   ├── config/
│   │   └── discovery.rs          # discover_config() — called from run_taskstoissues()
│   └── lib.rs
└── tests/
    └── taskstoissues_provider_select.rs

crates/specify-provider-core/
├── src/
│   ├── errors.rs                 # ProviderError — US2 target
│   ├── contract.rs
│   ├── registry.rs
│   ├── usecases/sync.rs          # plan_sync() / apply_sync()
│   └── lib.rs
└── tests/
    ├── metadata_required_fields.rs
    └── orchestration_dispatch.rs

crates/specify-provider-github/src/lib.rs   # GithubProvider — ProviderError consumer
crates/specify-provider-linear/src/lib.rs   # LinearProvider — ProviderError consumer

tests/conformance/                          # cross-provider conformance suite (US1/US2 regression gate)

.specify/scripts/bash/
├── check-prerequisites.sh                  # US3 target
├── common.sh                               # US3 target
├── create-new-feature.sh                   # US3 target
├── setup-plan.sh
└── setup-tasks.sh
```

**Structure Decision**: Existing Cargo workspace structure is unchanged — no new crates, modules, or directories. Work happens in place inside `apps/specify-mcp/src/mcp/taskstoissues.rs` (US1), `crates/specify-provider-core/src/errors.rs` plus its call sites across the three provider crates (US2), and header comments added to the three named scripts in `.specify/scripts/bash/` (US3). Regression coverage comes from the existing `test-us1`/`test-us2`/`test-us3`/`test-conformance` Makefile targets, which already exercise this exact surface.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |
