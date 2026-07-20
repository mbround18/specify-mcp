# Contract: `taskstoissues` MCP Tool External Stability

**Owner surface**: `run_taskstoissues()` in `apps/specify-mcp/src/mcp/taskstoissues.rs`

**Applies to**: User Story 1 (dependency audit and coupling reduction)

## Contract

This feature MUST NOT change, for any existing caller of the `taskstoissues` MCP tool:

1. The set of accepted request fields (`provider`, `dry_run`, and any existing scope/feature fields read via `args.get(...)`).
2. The shape or field names of the JSON `Value` returned on success.
3. The class of error surfaced on failure (still a `String` error message returned as `Result<Value, String>`), even if the *internal* error type used to construct that string changes as part of the `ProviderError` consolidation in User Story 2.

## Rationale

FR-009 explicitly forbids changing this tool's external contract as part of this effort. The audit and refactor in US1 are about *internal* call structure (which functions `run_taskstoissues()` calls and how), not about the tool's request/response shape. `ProviderRegistry`/`Provider`-trait routing changes are internal implementation details invisible to MCP clients as long as this contract holds.

## Verification

- `apps/specify-mcp/tests/taskstoissues_provider_select.rs` and any other existing test exercising `run_taskstoissues()`'s request/response shape MUST continue to pass unmodified (aside from updates required by genuine bug fixes, which must be called out explicitly, not silently).
- If any change under consideration would violate this contract, it MUST be deferred and documented as a known limitation per the spec's Edge Cases section, not implemented as part of this feature.
