# Implementation Plan: GitHub Integration Tightening

**Branch**: `002-tighten-github-integration` | **Date**: 2026-07-12 | **Spec**: `specs/002-tighten-github-integration/spec.md`

**Input**: Feature specification from `/specs/002-tighten-github-integration/spec.md`

## Stage Header

- **Stage ID**: S3
- **Stage Name**: Planning
- **Purpose**: Define executable architecture and validation strategy for provider-pivotable task-to-issues synchronization.
- **Entry Criteria**: S2 spec approved with measurable outcomes and explicit gaps.
- **Inputs**: `spec.md`, workspace crate layout, provider-core contracts.
- **Outputs**: `plan.md`, `research.md`, `data-model.md`, `contracts/*`, `quickstart.md`.
- **Exit Criteria**: No unresolved clarifications; architecture and tests cover pivotability, metadata, labels, and conformance.
- **Owner**: Technical planning flow (human + MCP assistant).

## Summary

Refactor task-to-issues flow so orchestration is provider-core-driven, provider adapters remain transport-only, and sync behavior stays stable when switching providers. Strengthen integration metadata into first-class provider-core entities, formalize lifecycle/relationship label policies as contracts, and add cross-provider conformance tests to enforce parity between GitHub, Linear, and future providers. Deliver deterministic reconciliation outcomes and idempotent reruns without changing existing orchestration semantics.

## Technical Context

**Language/Version**: Rust (workspace with crate edition 2021; workspace policy edition 2024)

**Primary Dependencies**: `specify-provider-core`, `specify-provider-github`, `specify-provider-linear`, `serde`, `serde_json`, `reqwest` (GitHub GraphQL), `toml`, `jsonschema`

**Storage**: Filesystem-backed specs (`specs/**`) plus provider sync ledger/metadata persisted under `.specify/` (new/expanded)

**Testing**: `cargo test` unit + integration, provider contract tests, cross-provider conformance fixtures

**Target Platform**: Linux/macOS development and CI runners for Rust workspace

**Project Type**: MCP/CLI orchestration app with pluggable provider crates

**Performance Goals**: Zero duplicate creations across repeated sync runs; deterministic item-level reconciliation outputs each run

**Constraints**: No provider-specific orchestration forks; provider switching via config/registry only; unsupported provider capabilities must be explicit

**Scale/Scope**: Provider-agnostic orchestration for parent + 20+ task graphs, GitHub and Linear parity baseline, extensible to future providers

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Current constitution file (`.specify/memory/constitution.md`) is a placeholder template with no enforceable principles or gates.  
**Pre-Phase 0 Gate Result**: PASS (no active constitutional rule violations detectable).  
**Post-Phase 1 Re-check Result**: PASS (design remains modular, testable, and provider-agnostic; still no enforceable constitution clauses present).

## Implementation Strategy

### 1) Provider-core orchestration pivotability

- Move orchestration entrypoints to provider-core use cases (`plan_sync`, `apply_sync`) and keep provider crates implementing shared contracts only.
- Add runtime provider selection (`local|github|linear|future`) through config + registry dispatch; orchestration caller does not branch by provider kind.
- Add contract conformance validation at provider activation; block provider start with actionable errors when required capabilities are missing.

### 2) Metadata model strengthening

- Expand provider-core models to include: `ExecutionContext`, `IntegrationIdentity`, `SourceOfTruthMarker`, `LifecycleMetadata`, `RelationshipMetadata`, `MilestoneProjectLinkage`, and `SyncLedgerEntry`.
- Define required vs optional fields explicitly (capability-gated optional linkage fields; required run identity and source-of-truth provenance).
- Persist deterministic reconciliation outputs per item (`created`, `updated`, `unchanged`, `drift_corrected`, `conflict_flagged`) to support safe reruns and partial-failure recovery.

### 3) Label policy contracts

- Define provider-agnostic label policy contract for lifecycle labels and relationship labels (`parent`, `child`, `blocked-by`, `depends-on`).
- Add provider mapping layer that translates canonical labels into provider-native structures while preserving canonical meaning.
- Enforce bidirectional consistency checks: metadata state implies labels, and labels imply metadata (unless provider capability explicitly lacks labels).

### 4) Cross-provider conformance testing

- Build shared fixture suite in provider-core that runs identical sync/reconciliation scenarios across GitHub and Linear adapters.
- Add parity assertions for orchestration outcomes, metadata completeness, label synchronization, and deterministic reconciliation states.
- Gate provider onboarding on passing conformance thresholds (including idempotent rerun and drift-correction scenarios).

## Delivery Waves

### Wave 1: Core contract and orchestration pivot
- Introduce provider-core sync orchestration interfaces and registry dispatch path.
- Remove provider-specific orchestration branching from app flow.

### Wave 2: Metadata + label contract hardening
- Implement enriched metadata entities and policy-driven label contracts.
- Add capability matrix reporting for active provider.

### Wave 3: Reconciliation determinism + recovery
- Add per-item reconciliation outcomes and partial-failure resume behavior via ledger identity reuse.
- Validate idempotency and deterministic outcomes across reruns.

### Wave 4: Cross-provider conformance suite
- Add shared conformance fixtures and parity thresholds for GitHub + Linear.
- Use suite as required gate for future providers.

## Project Structure

### Documentation (this feature)

```text
specs/002-tighten-github-integration/
├── plan.md
├── spec.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── provider-core-sync-contract.md
│   ├── label-policy-contract.md
│   └── conformance-suite-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
apps/
└── specify-mcp/
    └── src/
        ├── main.rs
        ├── mcp/
        ├── config/
        └── orchestrator/

crates/
├── specify-provider-core/
│   └── src/
│       ├── contract.rs
│       ├── models.rs
│       ├── registry.rs
│       └── errors.rs
├── specify-provider-github/
│   └── src/lib.rs
└── specify-provider-linear/
    └── src/lib.rs

tests/
└── integration/ (to be expanded for cross-provider conformance)
```

**Structure Decision**: Keep orchestration semantics centralized in provider-core and expose provider-specific behavior only through adapter implementations in each provider crate.

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Provider contract drift | Runtime failures after provider switch | Activation-time conformance validation + compatibility checks |
| Metadata inflation without ownership rules | Inconsistent reconciliation | Source-of-truth markers and required-field validation |
| Label vocabulary mismatch across providers | Broken lifecycle/relationship traceability | Canonical label policy + per-provider mapping tables |
| False parity in conformance suite | Undetected provider behavior divergence | Shared fixtures with deterministic outcome assertions and threshold gating |

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |
