# Quickstart: Validating Graph-Driven Codebase Health Improvements

## Prerequisites

- Rust toolchain matching `Cargo.toml` (`edition = "2024"`), `cargo` on `PATH`.
- Repository checked out at this feature's branch/state.
- Optional: `graphify` installed (`uv tool install graphifyy` or `pip install graphifyy`) to re-run the knowledge-graph baseline comparison for SC-001.

## US1 — Verify the `run_taskstoissues()` dependency audit (P1)

1. Open `apps/specify-mcp/src/mcp/taskstoissues.rs` and list every function call inside `run_taskstoissues()`.
2. Cross-reference against the Dependency Audit Record produced by this feature (see `data-model.md`) — confirm each call is marked `intentional-direct`, `intentional-via-registry`, `accidental-bypass`, or `false-inference`.
3. Run the targeted regression suite:
   ```bash
   make test-us1
   ```
   Expected: both `orchestration_dispatch` and `taskstoissues_provider_select` test binaries pass.
4. (Optional, requires graphify) Rebuild the knowledge graph and inspect `run_taskstoissues()`'s edges:
   ```bash
   /graphify --update
   /graphify query "What does run_taskstoissues call?"
   ```
   Note: the audit (`audit-notes/run-taskstoissues-dependencies.md`) found no accidental coupling, so no refactor was made — the *code* call graph is unchanged from the 22-edge baseline. The rebuilt graph's total edge count for this node may be higher than 22 because this feature's own spec/plan/research/audit-note files now reference it by name (documentation edges, not code edges). See the audit note's "SC-001" section for the full explanation before treating a higher count as a regression.

## US2 — Verify the `ProviderError` consolidation (P2)

1. Open `crates/specify-provider-core/src/errors.rs` and confirm each of the 8 variants has a one-line doc comment stating its intended meaning (matching `contracts/provider-error-contract.md`).
2. Search for `ProviderError::Internal` usage and cross-check against the contract:
   ```bash
   grep -rn "ProviderError::Internal" crates/specify-provider-github crates/specify-provider-linear apps/specify-mcp/src/orchestrator/provider.rs
   ```
   Expected: matches in `specify-provider-github` (adapter-resolution invariant, `~lines 127/140/157`) and `apps/specify-mcp/src/orchestrator/provider.rs` (`LocalProvider` filesystem I/O, `~lines 162/166/178/181`) — both are documented as legitimate per the corrected contract (see `contracts/provider-error-contract.md`'s "Post-audit correction" note and `audit-notes/provider-error-sites.md`). No matches expected in `specify-provider-linear`.
3. Run the targeted regression suite:
   ```bash
   make test-us2
   make test-conformance
   ```
   Expected: `metadata_required_fields`, `label_policy_contract`, and the full conformance suite pass.

## US3 — Verify shell script documentation (P3)

1. Open each of the following and confirm a header comment names the invoking speckit command(s):
   ```bash
   head -n 10 .specify/scripts/bash/check-prerequisites.sh
   head -n 10 .specify/scripts/bash/common.sh
   head -n 10 .specify/scripts/bash/create-new-feature.sh
   ```
2. Spot-check one named command against its agent file to confirm accuracy, e.g. if `common.sh`'s header names `speckit.specify`, confirm `create-new-feature.sh` (which sources `common.sh`) is indeed invoked from the `speckit.specify.agent.md` workflow.

## Full regression gate (all user stories)

```bash
make test
```

Expected: all targets (`test-rust`, `test-config`, `test-status`, `test-guide`, `test-init`, `test-sync-sample`, `test-github-graphql-dryrun`, `test-conformance`) pass with zero regressions (SC-005).
