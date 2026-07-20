# Contract: Provider-Core Sync Orchestration

## Purpose

Define provider-agnostic orchestration contract that provider adapters must satisfy.

## Required Operations

1. `plan_sync(input) -> SyncPlan`
   - Produces deterministic desired operations from spec/task graph + current remote state.
2. `apply_sync(plan) -> SyncRunResult`
   - Executes planned operations and emits item-level reconciliation outcomes.
   - MUST enforce metadata↔label consistency before provider mutations.
3. `health_check() -> ProviderHealth`
   - Validates runtime readiness before writes.
4. `capabilities() -> ProviderCapabilities`
   - Returns structured capability matrix used for activation and behavior branching.

## Capability Requirements

Required capability keys:
- `supports_labels`
- `supports_relationships`
- `supports_lifecycle`
- `supports_milestone`
- `supports_project`
- `supports_assignee`
- `supports_reconciliation`

Providers failing required baseline capabilities must not activate.

Runtime selection is via mode/override values constrained to `local|github|linear`.

## Reconciliation Outcome Contract

Each synchronized item MUST emit one outcome:
- `created`
- `updated`
- `unchanged`
- `drift_corrected`
- `conflict_flagged`

`conflict_flagged` MUST include machine-readable reason.

## Idempotency Contract

- Same input + unchanged remote state MUST produce no duplicate creates.
- Stable mapping key: `{provider_kind, internal_item_key}`.
- Fingerprint changes drive updates; unchanged fingerprint drives `unchanged`.
