# Feature Specification: Graph-Driven Codebase Health Improvements

**Feature Branch**: `003-graph-driven-code-health`

**Created**: 2026-07-20

**Status**: Draft

**Input**: User description: "Improve codebase modularity and clarity based on knowledge-graph analysis: reduce god-node coupling in tasks-to-issues sync, consolidate provider error handling, and close documentation gaps around orphaned shell script utilities"

## Background

A knowledge-graph analysis of this repository (graphify, 2026-07-20) surfaced four structural findings that this spec turns into actionable improvements:

1. `run_taskstoissues()` is the single highest-betweenness node in the codebase (22 edges, bridges 4+ communities: Tasks-to-Issues Sync Logic, Repo Setup & Initialization, Label Policy & Data Contracts, Provider Registry & Conformance) and carries 10 model-inferred (unverified) call relationships.
2. `ProviderError` is the most-connected node overall (43 edges), touched by the GitHub GraphQL client, provider capability/mocking layer, label policy contracts, the provider registry, and the Linear provider — a sign that error semantics are duplicated or overly entangled instead of centralized.
3. The `.specify/scripts/bash/` shell utilities (`check-prerequisites.sh`, `common.sh`, `create-new-feature.sh`) are weakly connected in the graph relative to the rest of the codebase, suggesting they are undocumented/under-referenced from the maintainer-facing docs.
4. The "Provider Integration Spec (001)" documentation community has very low cohesion (0.06), meaning its planning artifacts don't clearly cross-reference each other — a documentation navigability issue, not a code issue, and is therefore out of scope for this spec (tracked as a follow-up note only).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Verify and simplify the tasks-to-issues coupling (Priority: P1)

As a maintainer extending or debugging the tasks-to-issues sync flow, I need `run_taskstoissues()` to have a clear, verified set of dependencies so that a change in one area (e.g. label policy) doesn't produce a surprising break in an unrelated area (e.g. repo setup).

**Why this priority**: This is the highest-betweenness node in the codebase — it is the most likely single point of failure when refactoring any adjacent subsystem, and it currently carries unverified (INFERRED) relationships that may be wrong or may reveal accidental coupling.

**Independent Test**: Can be fully tested by tracing every call originating in `run_taskstoissues()`, confirming each is either a legitimate direct dependency or a boundary that should be mediated through an existing seam (e.g. `ProviderRegistry`), and delivers a documented, verified dependency list with any accidental coupling removed.

**Acceptance Scenarios**:

1. **Given** the current `run_taskstoissues()` implementation, **When** a maintainer reviews its call graph, **Then** every outgoing call is confirmed as either an intentional direct dependency or flagged and refactored to go through an existing abstraction (e.g. `ProviderRegistry`, `Provider` trait).
2. **Given** the 10 previously-inferred relationships between `run_taskstoissues()` and functions like `call_tool()` and `discover_config()`, **When** a maintainer inspects the actual source, **Then** each relationship is confirmed as real (and kept) or determined to be a false inference (and no code change is needed for that one).
3. **Given** the refactor is complete, **When** the knowledge graph is rebuilt, **Then** `run_taskstoissues()`'s edge count and betweenness centrality are reduced relative to the pre-refactor baseline, without any loss of existing test coverage.

---

### User Story 2 - Consolidate provider error handling (Priority: P2)

As a maintainer implementing or reviewing a provider integration (GitHub, Linear, or a future provider), I need a single, well-documented error model so that I don't need to understand five different subsystems' error-handling conventions to add a new provider or debug a failure.

**Why this priority**: `ProviderError` is the most-connected node in the entire graph (43 edges) spanning the GitHub client, capability/mocking layer, label policy, provider registry, and Linear provider. High fan-in on an error type across unrelated subsystems is a strong signal of either healthy centralization (good) or leaky abstraction (bad) — this needs to be confirmed and, if leaky, tightened.

**Independent Test**: Can be fully tested by auditing every call site that constructs or matches on `ProviderError`, confirming each subsystem uses it only for its intended purpose (provider-boundary failures) rather than as a general-purpose error dumping ground, and delivers a documented error-handling contract that new providers can follow.

**Acceptance Scenarios**:

1. **Given** the current `ProviderError` usages across `crates/specify-provider-core`, the GitHub client, and the Linear provider, **When** a maintainer audits each call site, **Then** every usage is confirmed as a genuine provider-boundary error (network, auth, contract violation) rather than an internal/business-logic error miscategorized as a provider error.
2. **Given** any call sites found to be misusing `ProviderError` for non-provider-boundary failures, **When** the audit is complete, **Then** those call sites use a more specific error type instead, and `ProviderError`'s variant list documents exactly what each variant means and when to use it.
3. **Given** a new provider is added in the future, **When** a maintainer follows the documented error contract, **Then** they can determine which `ProviderError` variant to use (or whether a new variant is needed) without reading every existing provider's implementation.

---

### User Story 3 - Document and connect the orphaned shell script utilities (Priority: P3)

As a new contributor or an AI coding agent reading this repository for the first time, I need the `.specify/scripts/bash/` utilities to be discoverable and clearly connected to the speckit commands that use them, so that I don't mistake them for dead code or duplicate their logic.

**Why this priority**: These scripts (`check-prerequisites.sh`, `common.sh`, `create-new-feature.sh`) are weakly connected in the dependency graph — lowest priority because it's a documentation/discoverability gap rather than a correctness risk, but it compounds the confusion in User Story 1 and 2 since these scripts are part of the same operational surface.

**Independent Test**: Can be fully tested by checking that each script's header comment (or an adjacent doc) states which speckit command(s) invoke it and what contract it fulfills, and that the `GRAPH_REPORT.md` re-run shows these nodes with materially higher connectivity (or clearly annotated as intentionally standalone).

**Acceptance Scenarios**:

1. **Given** `check-prerequisites.sh`, `common.sh`, and `create-new-feature.sh`, **When** a contributor opens any one of them, **Then** they can identify within the first few lines which speckit agent/command(s) call it and why.
2. **Given** the documentation gap is closed, **When** the knowledge graph is rebuilt, **Then** these scripts are no longer flagged as weakly-connected outliers relative to the rest of the codebase.

---

### Edge Cases

- What happens when a call site in `run_taskstoissues()` cannot be cleanly attributed to either "legitimate dependency" or "accidental coupling" (i.e., it's ambiguous)? → Document it explicitly as a known, accepted coupling with a one-line rationale rather than silently leaving it unresolved.
- How does the error-consolidation effort handle a `ProviderError` variant that is used correctly in one subsystem but incorrectly in another? → The variant stays; only the incorrect call sites are migrated to a more specific error type.
- What happens if fixing a piece of accidental coupling would require an API-breaking change to a public MCP tool response? → Flag it as a known limitation and document the tradeoff rather than making a breaking change as part of this effort.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Maintainers MUST be able to view a complete, verified list of `run_taskstoissues()`'s outgoing dependencies, with each dependency labeled as intentional or removed.
- **FR-002**: The codebase MUST NOT contain calls from `run_taskstoissues()` directly into internals of unrelated subsystems (e.g. config discovery, repo setup) that bypass the existing `ProviderRegistry`/`Provider` abstraction, unless explicitly documented as an accepted exception.
- **FR-003**: Each of the 10 previously-inferred relationships from `run_taskstoissues()` MUST be individually confirmed or refuted against the actual source, with the outcome recorded (e.g., in the PR description or a short audit note).
- **FR-004**: `ProviderError`'s variants MUST be documented with their intended meaning and the subsystem(s) allowed to construct them.
- **FR-005**: Call sites that construct or match `ProviderError` for a non-provider-boundary failure MUST be migrated to use a more specific, existing error type.
- **FR-006**: A future provider implementation MUST be able to follow the documented error contract without needing to read the GitHub and Linear provider implementations first.
- **FR-007**: Each shell script under `.specify/scripts/bash/` MUST state, in a header comment, which speckit command(s) invoke it.
- **FR-008**: None of the changes in this effort MAY remove or weaken existing automated test coverage; any test files touched MUST continue to pass.
- **FR-009**: This effort MUST NOT change any public MCP tool's external contract (request/response shape) without flagging it as a known limitation per the Edge Cases section.

### Key Entities

- **`run_taskstoissues()`**: The MCP tool handler that orchestrates syncing tasks to provider issues; the highest-betweenness node in the codebase.
- **`ProviderError`**: The shared error type for provider-boundary failures; currently the most-connected node in the codebase.
- **`ProviderRegistry` / `Provider` trait**: The existing abstraction boundary that provider-specific logic (GitHub, Linear) should be accessed through.
- **`.specify/scripts/bash/*.sh`**: Shell utilities backing the speckit command workflow (prerequisite checks, common helpers, feature-directory creation).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After a graph rebuild, `run_taskstoissues()`'s edge count and betweenness centrality both decrease relative to the 2026-07-20 baseline (22 edges, rank #1 by betweenness).
- **SC-002**: 100% of the 10 previously-inferred `run_taskstoissues()` relationships are individually confirmed or refuted, with the outcome documented.
- **SC-003**: 100% of `ProviderError` construction/match sites are audited; any found to be misused for non-provider-boundary failures (target: at least the outliers identified in the audit) are migrated to a more specific error type.
- **SC-004**: A new contributor (or agent) can read `.specify/scripts/bash/common.sh`'s header and correctly state which speckit commands depend on it, without reading the speckit agent files first.
- **SC-005**: All existing automated tests continue to pass after the changes (zero regressions).

## Assumptions

- The audit and refactor target the current `main` branch state as of 2026-07-20; new code merged after this spec is written is out of scope unless it touches the same call sites.
- "Accidental coupling" is judged by whether a call bypasses the `ProviderRegistry`/`Provider` abstraction that already exists for this purpose — not by an arbitrary architectural preference.
- The "Provider Integration Spec (001)" low-cohesion documentation finding is a documentation-navigability issue, not a code-quality issue, and is explicitly out of scope for this spec.
- No new external dependencies (crates) are introduced; this is a refactor/consolidation effort using existing abstractions.
- Shell script documentation (User Story 3) is satisfied by header comments; no new documentation site or tooling is required.
