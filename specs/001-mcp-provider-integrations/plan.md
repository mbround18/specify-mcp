# Implementation Plan: MCP Spec Kit Zero-Install + Pluggable Issue Providers

**Branch**: `001-mcp-provider-integrations` | **Date**: 2026-07-12 | **Spec**: `specs/001-mcp-provider-integrations/spec.md`

**Input**: Feature specification from `/specs/001-mcp-provider-integrations/spec.md`

## Stage Header

- **Stage ID**: S3
- **Stage Name**: Planning
- **Purpose**: Translate requirements into architecture, delivery waves, and risk-managed implementation strategy.
- **Entry Criteria**: Specification stage complete with accepted scope and success criteria.
- **Inputs**: `spec.md`, repo structure, platform constraints.
- **Outputs**: `plan.md` with structure decisions, waves, risk mitigation, and implementation direction.
- **Exit Criteria**: Plan is executable, modular, and mapped to measurable outcomes.
- **Owner**: Technical planning flow (human + MCP assistant).

## Summary

Implement a Rust MCP-first setup and execution flow for Spec Kit that works without a globally installed `specify` CLI, then add a modular provider layer so specs/tasks can be stored in local files, GitHub Issues, or Linear. GitHub must support both MCP and direct GraphQL execution selected by capability detection and config preference, with a default tracking topology of one parent spec issue plus an in-issue task checklist and selective child-issue splitting only when needed.

## Closeout Addendum (2026-07-12)

To bring this feature home, scope is frozen to a **v1.0.0 M3 cut**:

- **In-scope**: Zero-install setup path + GitHub provider GA candidate + conformance/reconciliation release gates.
- **Out-of-scope for v1.0.0**: Linear expansion/perf tail, extension/Jira onboarding, and post-GA adoption analytics.

Execution now follows: reconcile 001 status to 002-delivered work, close remaining M3 deltas, run release gates, execute one GitHub pilot, then tag.

Given the expected size of this initiative, execution is organized as a staged program: foundational platform work, zero-install MVP, provider parallelization (GitHub + Linear), reliability hardening, and production rollout.

## Technical Context

**Language/Version**: Rust stable (for MCP service), shell scripts for existing Spec Kit orchestration, TOML for configuration

**Primary Dependencies**: MCP Rust SDK/runtime, GitHub GraphQL API and/or GitHub MCP, Linear GraphQL API client

**Storage**: Local files for default mode (`specs/**`) plus provider metadata state file under `.specify/`

**Testing**: Rust unit/integration tests, provider contract tests, shell smoke tests

**Target Platform**: Linux/macOS development environments with MCP-capable agent hosts

**Project Type**: CLI/MCP tooling

**Performance Goals**: Setup <90s p95; provider sync operations idempotent and deterministic

**Constraints**: Must function with no global `specify` install; no silent fallback on provider failure

**Scale/Scope**: 3 first-party providers (`local`, `github`, `linear`) + extension point for future providers (e.g., Jira)

## Constitution Check

No constitutional blockers identified; this plan keeps behavior explicit, modular, and test-driven via provider contracts.

## Delivery Waves

### Wave 1: Core platform + deterministic config

- Build provider contracts, registry, sync ledger, and config precedence resolver.
- Exit criteria: all config precedence tests pass; provider contract harness operational.

### Wave 2: Zero-install MCP MVP

- Deliver setup/install/repair via MCP and first end-to-end spec lifecycle without global CLI.
- Exit criteria: clean-machine workflow success meets SC-001 target.

### Wave 3: GitHub + Linear parallel integration tracks

- Run GitHub and Linear provider development in parallel, each with migration tooling and idempotency test suites.
- In the GitHub track, implement parent-checklist-first orchestration and policy-based child issue splitting for tasks that require standalone execution.
- Exit criteria: SC-002 and SC-003 test criteria consistently met.

### Wave 4: Reliability and rollout

- Add dry-run, conflict handling, retry/backoff, observability, and incident runbooks.
- Exit criteria: pilot teams complete adoption with rollback path verified.

## Project Structure

### Documentation (this feature)

```text
specs/001-mcp-provider-integrations/
├── plan.md
├── spec.md
├── speckit-mcp-operating-model.md
├── specify-toml-standard.md
├── subspec-provider-core.md
├── subspec-provider-github.md
├── subspec-provider-linear.md
└── tasks.md
```

### Source Code (repository root)

```text
apps/
└── specify-mcp/
    ├── Cargo.toml
    ├── src/
    │   ├── main.rs
    │   ├── mcp/
    │   │   ├── setup.rs
    │   │   └── workflows.rs
    │   ├── config/
    │   │   ├── discovery.rs
    │   │   └── schema.rs
    │   └── orchestrator/
    │       ├── mod.rs
    │       └── sync.rs
    └── tests/
        ├── mcp/
        └── config/

crates/
├── specify-provider-core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── contract.rs
│       ├── registry.rs
│       ├── models.rs
│       └── errors.rs
├── specify-provider-github/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── provider.rs
│       ├── adapter_graphql.rs
│       └── adapter_mcp.rs
└── specify-provider-linear/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── provider.rs
        └── adapter_graphql.rs

tests/
└── integration/
    ├── provider_contract/
    ├── github/
    └── linear/
```

**Structure Decision**: Rust workspace split into one app crate (`apps/specify-mcp`) and supporting provider crates under `crates/`, with provider implementations isolated behind a shared provider-core contract.

## Key Design Refinements

1. **Provider split**: separate provider domain logic from transport adapters to avoid GH/Linear API coupling in orchestration.
2. **Strict config precedence**: resolve once at startup; always report chosen source file in logs and diagnostics.
3. **Idempotency-first sync**: all remote mutations go through a diff-and-intent planner keyed by stable content hashes.
4. **Migration safety**: always support snapshot export before remote pivot to preserve rollback.
5. **Extension contract**: future providers (Jira, etc.) must pass the same contract suite before enablement.
6. **Workspace modularity**: keep provider implementations out of the app crate except via traits from `specify-provider-core`.
7. **Issue topology strategy**: default to one spec parent issue containing the canonical task checklist; generate child issues only for tasks that satisfy explicit split policy, and keep parent/child status synchronized.

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Adapter auth variability (GraphQL token vs MCP) | Setup/sync failures | Dual-probe + explicit diagnostics + configurable preference order |
| API rate limits and partial writes | Duplicate/incomplete remote states | Idempotency keys, resume-aware ledger, backoff/retry |
| Config drift across multiple files | Wrong provider selected | Deterministic precedence + startup source report + tests |
| Manual edits to remote issues | Reconciliation conflicts | Conflict policy engine + dry-run previews + operator guidance |
| Large task sets (100+) | Slow sync performance | Batched operations + delta sync + performance gates |

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |
