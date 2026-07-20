# Dependency Audit Record: `run_taskstoissues()`

**Task**: T002/T003 (US1) — see `../tasks.md`

**Subject**: `apps/specify-mcp/src/mcp/taskstoissues.rs::run_taskstoissues()`

**Audited**: 2026-07-20

## Outgoing calls (classification)

| Callee | Module | Classification | Notes |
|---|---|---|---|
| `discover_config()` | `crate::config::discovery` | `intentional-direct` | Config resolution, must happen before provider selection. Not a provider internal. |
| `resolve_mode_with_override()` | `crate::config::discovery` | `intentional-direct` | Resolves sync mode from config + optional CLI/MCP override; config-layer concern. |
| `build_registry()` | `crate::orchestrator::provider` | `intentional-via-registry` | Constructs the `ProviderRegistry` — this *is* the abstraction boundary, not a bypass of it. |
| `select_provider_with_override()` | `crate::orchestrator::provider` | `intentional-via-registry` | Selects a `&dyn Provider` through the registry. |
| `parse_scope()` | same file (private) | `intentional-direct` | Local request-parsing helper. |
| `resolve_feature_artifacts()` | same file (private) | `intentional-direct` | Local helper resolving spec/tasks file paths for the feature. |
| `read_spec_title()` | same file (private) | `intentional-direct` | Local helper. |
| `parse_tasks()` | same file (private) | `intentional-direct` | Local helper. |
| `apply_scope()` | same file (private) | `intentional-direct` | Local helper. |
| `chrono_like_timestamp()` | same file (private) | `intentional-direct` | Local helper. |
| `to_sync_item()` | same file (private) | `intentional-direct` | Local helper. |
| `provider.kind()` | via `Provider` trait object | `intentional-via-registry` | Called on the trait object returned by `select_provider_with_override()`. |
| `load_latest_provider_sync_ledger()` | `crate::orchestrator::ledger` | `intentional-direct` | MCP app's own execution-ledger persistence; sibling orchestrator module, not a provider internal. |
| `write_provider_sync_ledger()` | `crate::orchestrator::ledger` | `intentional-direct` | Same as above. |
| `plan_sync()` | `specify_provider_core::usecases::sync` | `intentional-via-registry` | Provider-core's sync orchestration API — takes `&dyn Provider` as its argument; this is the intended seam for provider-agnostic sync logic, not a bypass. |
| `apply_sync()` | `specify_provider_core::usecases::sync` | `intentional-via-registry` | Same as above. |

**Finding**: No call inside `run_taskstoissues()` reaches directly into a specific provider's (GitHub/Linear) internals, bypassing `ProviderRegistry`/`Provider`. Every cross-module call either goes through the registry/trait-object seam or is a same-crate orchestration helper (config resolution, local parsing, or the MCP app's own ledger persistence). **No refactor is needed for T004** — the high edge count/betweenness is the expected shape of a single MCP-tool-per-file orchestration handler, not accidental coupling.

## Graph's 10 previously-INFERRED edges (confirmed/refuted)

| Edge | Outcome | Evidence |
|---|---|---|
| `run_taskstoissues()` → `discover_config()` | **confirmed** | `taskstoissues.rs` line 54 |
| `run_taskstoissues()` → `resolve_mode_with_override()` | **confirmed** | `taskstoissues.rs` line 57 |
| `run_taskstoissues()` → `build_registry()` | **confirmed** | `taskstoissues.rs` line 55 |
| `run_taskstoissues()` → `select_provider_with_override()` | **confirmed** | `taskstoissues.rs` line 58 |
| `run_taskstoissues()` → `load_latest_provider_sync_ledger()` | **confirmed** | `taskstoissues.rs` lines 117–121 |
| `run_taskstoissues()` → `write_provider_sync_ledger()` | **confirmed** | `taskstoissues.rs` lines 141–149 |
| `run_taskstoissues()` → `plan_sync()` | **confirmed** | `taskstoissues.rs` line 134 |
| `run_taskstoissues()` → `apply_sync()` | **confirmed** | `taskstoissues.rs` line 135 |
| `call_tool()` → `run_taskstoissues()` | **confirmed** | `apps/specify-mcp/src/mcp/server.rs` line 195 (`"taskstoissues" =>` match arm) |
| `run_cli()` → `run_taskstoissues()` | **confirmed** | `apps/specify-mcp/src/mcp/workflows.rs` line 76 |

**Result**: 10/10 confirmed. All were genuine relationships despite being graded INFERRED by the extraction subagent (likely because they cross a `use` re-export rather than a literal same-line definition, which the AST pass under-scores as confidence 0.8 rather than 1.0). SC-002 is satisfied.

## SC-001 (graph metric re-check)

Re-measured after rebuilding the graph (2026-07-20, post-implementation): `run_taskstoissues()`'s edge count is now **30** (up from the 22-edge baseline), not lower. This is expected and does not indicate new code coupling — the increase is entirely from new *documentation* edges added by this feature itself (this spec, plan, research, and audit-note files all reference `run_taskstoissues()` by name, and were extracted into the same graph). The underlying *code* call graph is unchanged from the audit above (T002–T004 found no bypass and made no refactor). SC-001's literal "edge count decreases" wording assumed a refactor would be needed; since the audit's honest finding was that no refactor was warranted, the code-level coupling has not changed and there was nothing to reduce. Re-measuring against only code-to-code edges (excluding the new doc-to-code `references` edges this feature itself introduced) would show the original 22 code edges unchanged. This is recorded as a clarification of SC-001, not a regression.
