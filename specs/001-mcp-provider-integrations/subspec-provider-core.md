# Sub-Spec: Provider Core Crate (`crates/specify-provider-core`)

## Scope

Define the shared provider contract, canonical domain models, registry interfaces, capability model, and error taxonomy used by all provider crates.

## Requirements

- Provide stable traits for provider lifecycle operations (create/update/sync/status/health).
- Provide shared models: `SpecRecord`, `TaskRecord`, `SyncLedger`, `ProviderCapability`.
- Provide deterministic idempotency helpers (hashing + reconciliation intent types).
- Provide structured error types (auth, permission, transport, validation, conflict).
- Provide provider contract test harness that provider crates must pass.

## Out of Scope

- Any GitHub- or Linear-specific API code.
- App-level CLI/MCP orchestration behavior.

## Acceptance

1. `apps/specify-mcp` compiles against provider-core traits without referencing provider internals.
2. Both GitHub and Linear crates can implement the same trait set without trait changes.
3. Contract test harness can run against a provider implementation and report conformance.
