# Phase 0 Research: Graph-Driven Codebase Health Improvements

No `NEEDS CLARIFICATION` markers remain in the Technical Context — this is an audit/refactor of an existing, well-understood Rust workspace, not a new-technology adoption. This document records the findings from a direct source read (not the graph) that ground the Phase 1 design.

## Decision: `run_taskstoissues()` call graph is mostly legitimate orchestration, not accidental coupling

**Rationale**: Reading `apps/specify-mcp/src/mcp/taskstoissues.rs` directly shows `run_taskstoissues()` calling, in order: `discover_config()` (config resolution), `build_registry()` + `select_provider_with_override()` (both go through the existing `ProviderRegistry`/`Provider` abstraction in `apps/specify-mcp/src/orchestrator/provider.rs`), `parse_scope()`, `resolve_feature_artifacts()`, `read_spec_title()`, `parse_tasks()`, `apply_scope()`, and `chrono_like_timestamp()` — the latter group are local helpers within the same MCP tool module, not internals of unrelated subsystems. This is the shape of a legitimate command-handler/orchestrator: it is *expected* to be the highest-betweenness node in an MCP-tool-per-file architecture, since it's the single entry point that stitches config, provider selection, and task parsing together for one MCP tool call.

This changes the shape of US1's work from "remove coupling" to "verify + document": the audit (FR-001, FR-002, FR-003) should confirm this reading, individually resolve the graph's 10 INFERRED edges against it, and only refactor call sites that turn out to reach *past* `ProviderRegistry`/`Provider` into a provider's internals directly (which the direct read did not find in the function body reviewed, but the full audit must check `apply_scope`, `resolve_feature_artifacts`, and downstream sync usecases too).

**Alternatives considered**: Preemptively splitting `run_taskstoissues()` into multiple smaller functions without first verifying which edges are real. Rejected — SC-001/SC-002 require the *verified* dependency list as the artifact of value; splitting code before verifying which relationships are genuine risks moving the same coupling around without reducing it, and contradicts the spec's edge case guidance to document ambiguous couplings rather than force a refactor.

## Decision: `ProviderError`'s 8 variants (`Auth`, `Permission`, `Transport`, `Validation`, `Conflict`, `Unsupported`, `Conformance`, `Internal`) are the audit's checklist

**Rationale**: `crates/specify-provider-core/src/errors.rs` defines exactly 8 variants, each wrapping a `String`. High fan-in (43 edges) across `contract.rs`, `registry.rs`, `usecases/sync.rs`, and both provider crates (`specify-provider-github`, `specify-provider-linear`) plus `apps/specify-mcp/src/orchestrator/provider.rs` is architecturally expected for a shared provider-boundary error type — providers *should* funnel their failures through one enum so callers can match on `.kind()`-style behavior uniformly (this is the intended pattern per the existing `Provider` trait's `.health_check()`/`.capabilities()` methods observed in the graph's community summaries). The audit's job (FR-004, FR-005) is to confirm each of the 8 variants is used only for genuine provider-boundary failures (network/auth/contract violations) at every call site, not to assume misuse exists.

**Alternatives considered**: Introducing a new, more granular error enum per subsystem before auditing. Rejected — would add a new abstraction (violating the "no new external dependencies / consolidation not proliferation" framing in FR-005/FR-006) before establishing whether the current 8-variant enum is actually being misused anywhere.

## Decision: Regression coverage comes from existing Makefile targets, no new test harness needed

**Rationale**: `Makefile` already defines `test-us1` (`orchestration_dispatch`, `taskstoissues_provider_select`), `test-us2` (`metadata_required_fields`, `label_policy_contract`), `test-us3` (`specify-conformance`), and `test-rust` (full workspace). These map directly onto this feature's three user stories' regression surface. FR-008 ("MUST NOT remove or weaken existing coverage") is satisfied by running these targets before and after each change.

**Alternatives considered**: Writing a new integration test suite specifically for this audit. Rejected — the existing suites already exercise `run_taskstoissues()`'s provider-selection path and `ProviderError`-surfacing contract paths; duplicating them would violate the "no new abstractions beyond what the task requires" principle.

## Decision: Shell script documentation via header comment, not a new docs site

**Rationale**: `.specify/scripts/bash/common.sh`, `check-prerequisites.sh`, and `create-new-feature.sh` are already sourced/invoked by the `speckit.*.agent.md` command files (confirmed via `source "$SCRIPT_DIR/common.sh"` in `create-new-feature.sh`). The gap is discoverability from the script side, not the agent side — a 2-4 line header comment naming the invoking command(s) closes this without new tooling.

**Alternatives considered**: A dedicated `SCRIPTS.md` index. Rejected as unnecessary indirection for 3 files; a header comment is co-located with the code it describes and can't drift out of sync as easily as a separate index.
