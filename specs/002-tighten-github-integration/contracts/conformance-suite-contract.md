# Contract: Cross-Provider Conformance Suite

## Purpose

Specify parity expectations for provider implementations behind shared orchestration.

## Scope

Providers in scope now:
- `github`
- `linear`

Future providers must satisfy the same suite before activation.

## Required Fixture Scenarios

1. Parent + 20-task graph creation
2. Repeat run with unchanged input (idempotency)
3. Metadata/label drift correction
4. Partial failure resume
5. Capability-gated milestone/project behavior

## Parity Assertions

- Orchestration outcome categories equivalent across providers.
- Required metadata fields present for every synchronized item.
- Label policy behavior matches canonical contract where label capabilities are supported.
- Reconciliation outcomes constrained to approved enum with deterministic semantics.

## Pass Criteria

- 100% pass on mandatory fixture scenarios.
- 0 duplicate item creations in idempotency fixture.
- 100% metadata completeness for required fields.
- 100% deterministic reconciliation outcome validity.

## Current Harness Entry Points

- `cargo test -p specify-provider-core --test orchestration_dispatch`
- `cargo test -p specify-provider-core --test metadata_required_fields`
- `cargo test -p specify-provider-core --test label_policy_contract`
- `cargo test -p specify-conformance`
