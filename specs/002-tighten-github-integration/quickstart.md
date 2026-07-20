# Quickstart: Validate GitHub Integration Tightening

## Stage Header

- **Stage ID**: S3-P1
- **Stage Name**: Design Validation Guide
- **Purpose**: Define end-to-end validation flow for the planned architecture.
- **Inputs**: `plan.md`, `research.md`, `data-model.md`, `contracts/*`.
- **Outputs**: Runnable scenarios and expected outcomes.

## Prerequisites

- Rust toolchain installed.
- GitHub auth available for GraphQL flow (`GITHUB_TOKEN` or `gh auth token`).
- Workspace checked out at repository root.
- Feature docs available under `specs/002-tighten-github-integration/`.

## Setup

1. From repo root:
   - `cargo check`
   - `cargo test -p specify-provider-core --test orchestration_dispatch --test metadata_required_fields --test label_policy_contract`
   - `cargo test -p specify-mcp --test taskstoissues_provider_select`
   - `cargo test -p specify-conformance`
2. Ensure provider configs can select provider kind via config/registry (`github`, `linear`, `local`).
3. Prepare sample parent + 20-task fixture data with dependencies.

## Scenario A: Provider-core orchestration pivotability

1. Run sync fixture through GitHub provider.
2. Run same fixture through Linear provider with no orchestration code changes.
3. Compare planned/applied orchestration outcomes.

**Expected**
- Same orchestration path and outcome categories.
- Provider-specific remote IDs differ, but mapping and reconciliation semantics stay equivalent.

## Scenario B: Metadata and label policy completeness

1. Execute sync for fixture containing lifecycle states and relationship edges.
2. Inspect produced metadata/ledger records.
3. Verify label mappings against `contracts/label-policy-contract.md`.

**Expected**
- Required metadata fields present (execution context, identity, source-of-truth, lifecycle, relationships).
- Lifecycle and relationship labels stay synchronized with metadata.

## Scenario C: Idempotency + reconciliation determinism

1. Run same sync input three consecutive times.
2. Manually edit remote labels/metadata to introduce drift.
3. Re-run sync and inspect per-item outcomes.

**Expected**
- No duplicate item creation.
- Outcomes limited to contract enum:
  - `created`, `updated`, `unchanged`, `drift_corrected`, `conflict_flagged`
- Drift either corrected or conflict-flagged deterministically.

## Scenario D: Cross-provider conformance suite

1. Run provider-core conformance fixture tests:
   - `cargo test -p specify-provider-core conformance`
2. Include GitHub and Linear adapter-backed runs.

**Expected**
- Contract tests pass for both providers.
- Parity thresholds met for orchestration and reconciliation assertions.

## References

- Data model: `specs/002-tighten-github-integration/data-model.md`
- Provider contract: `specs/002-tighten-github-integration/contracts/provider-core-sync-contract.md`
- Label policy contract: `specs/002-tighten-github-integration/contracts/label-policy-contract.md`
- Conformance contract: `specs/002-tighten-github-integration/contracts/conformance-suite-contract.md`

## Validation Log (Current Implementation Run)

- `cargo check -q` ✅
- `cargo test -q -p specify-provider-core` ✅
- `cargo test -q -p specify-mcp` ✅
- `cargo test -q -p specify-conformance` ✅

## v1.0.0 M3 Release Gate Checklist (2026-07-12)

- [X] US1 validation target passes (`make test-us1`)
- [X] US2 validation target passes (`make test-us2`)
- [X] US3/conformance target passes (`make test-us3`)
- [ ] GitHub dry-run integration smoke executed against sandbox repo (`make test-github-graphql-dryrun`)
- [ ] Pilot run completed with one GitHub-heavy team
- [ ] Rollback playbook exercised and documented for operator handoff

Current status: GitHub dry-run smoke target is wired and currently skips without `SPECIFY_GITHUB_TEST_REPO`.
