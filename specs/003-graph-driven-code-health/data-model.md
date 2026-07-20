# Phase 1 Data Model: Graph-Driven Codebase Health Improvements

This feature does not introduce new persisted data entities — it audits and documents the relationships between existing code entities. The "entities" below are the audit's subject matter and its output artifacts.

## Audit Subject Entities (existing code, unchanged shape)

### `run_taskstoissues()` dependency edge
- **Represents**: One outgoing call from `run_taskstoissues()` (`apps/specify-mcp/src/mcp/taskstoissues.rs`) to another function.
- **Attributes**: `callee_name`, `callee_module`, `classification` (`intentional-direct` | `intentional-via-registry` | `accidental-bypass` | `false-inference`), `disposition` (`kept` | `refactored` | `documented-exception`).
- **Relationships**: Many-to-one with `run_taskstoissues()`; each edge references at most one `ProviderRegistry`/`Provider` trait method when `classification = intentional-via-registry`.
- **Validation rule** (from FR-002/FR-003): every edge classified `accidental-bypass` MUST have `disposition = refactored` unless documented as an exception per the spec's Edge Cases section.

### `ProviderError` construction/match site
- **Represents**: One place in the codebase that constructs or `match`es a `ProviderError` value.
- **Attributes**: `file_path`, `line`, `variant` (one of `Auth`, `Permission`, `Transport`, `Validation`, `Conflict`, `Unsupported`, `Conformance`, `Internal`), `subsystem` (`provider-core` | `github-provider` | `linear-provider` | `mcp-app`), `classification` (`genuine-provider-boundary` | `misused-internal-error`), `disposition` (`kept` | `migrated`).
- **Relationships**: Many sites reference one of the 8 `ProviderError` variants defined in `crates/specify-provider-core/src/errors.rs`.
- **Validation rule** (from FR-005): every site classified `misused-internal-error` MUST have `disposition = migrated` to a more specific existing error type.

### Shell script documentation entry
- **Represents**: The header comment block added to one of the three target scripts.
- **Attributes**: `script_path`, `invoking_commands` (list of speckit command names), `contract_summary` (one line).
- **Relationships**: One script → one or more invoking speckit agent/command files under `.github/agents/speckit.*.agent.md`.

## Audit Output Artifacts (produced by this feature)

### Dependency Audit Record (US1)
- **Represents**: The documented outcome required by FR-001/FR-003/SC-002 — a complete list of `run_taskstoissues()`'s outgoing dependency edges with classification and disposition.
- **Format**: A table in the PR description or a short markdown note (not a new persisted file type); at minimum covers all edges found by direct source reading plus the 10 previously-INFERRED graph edges by name.
- **Completion rule**: 100% of the 10 INFERRED edges must appear with a `confirmed` or `refuted` outcome (SC-002).

### Error Contract Note (US2)
- **Represents**: The documentation required by FR-004/FR-006 — what each of the 8 `ProviderError` variants means and which subsystems may construct them.
- **Format**: A doc comment block on the `ProviderError` enum itself (`crates/specify-provider-core/src/errors.rs`), since that keeps the contract co-located with the type it describes and visible to any future provider implementation via `cargo doc`.
- **Completion rule**: every variant has a one-line "when to use" doc comment; SC-003 requires 100% of call sites audited.
