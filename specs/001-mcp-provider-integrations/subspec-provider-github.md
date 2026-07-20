# Sub-Spec: GitHub Provider Crate (`crates/specify-provider-github`)

## Scope

Implement GitHub-backed spec/task management using provider-core contracts with dual adapters: GitHub MCP and `gh` CLI.

## Requirements

- Implement provider-core `Provider` contract fully.
- Support adapter capability detection for MCP and `gh`.
- Support adapter preference ordering from `specify.toml`.
- Map one spec to a parent GitHub issue with canonical task checklist as the default topology.
- Support policy-based promotion of selected checklist tasks into child/linked issues when split criteria are met.
- Persist sync metadata for idempotent upserts and reconciliation.
- Return explicit errors for auth, permission, rate limit, and transport failures.

## Out of Scope

- Linear support.
- Generic config precedence engine implementation.

## Acceptance

1. Provider passes provider-core contract test suite.
2. Same spec/task set synced repeatedly creates no duplicate issues.
3. Adapter arbitration behaves deterministically based on config + runtime capability.
4. Parent checklist and any promoted child issues stay synchronized without duplicate or drifting task state.
