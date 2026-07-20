# Sub-Spec: Linear Provider Crate (`crates/specify-provider-linear`)

## Scope

Implement Linear-backed spec/task management using provider-core contracts and Linear GraphQL integration.

## Requirements

- Implement provider-core `Provider` contract fully.
- Map spec/task domain to Linear issues with configurable team/project/state/priority.
- Persist sync metadata for idempotent upserts and reconciliation.
- Handle missing mapping/config with explicit validation errors.
- Support migration path from local markdown spec/tasks to Linear issues.

## Out of Scope

- GitHub adapter logic.
- MCP setup/bootstrap logic in app crate.

## Acceptance

1. Provider passes provider-core contract test suite.
2. Repeated sync of the same spec/task payload creates no duplicate Linear issues.
3. Configured state/priority/team mappings are applied consistently during create/update.
