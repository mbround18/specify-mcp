# Audit Record: `ProviderError` Construction/Match Sites

**Tasks**: T008/T009/T010/T011 (US2) — see `../tasks.md`

**Audited**: 2026-07-20

## `crates/specify-provider-github/src/lib.rs` (T008)

| Site | Variant | Classification | Disposition |
|---|---|---|---|
| `health_check()`, `GithubAdapter::Auto` arm (~line 127) | `Internal` | genuine (orchestration-internal invariant violation — "auto adapter should be resolved at construction" is a should-never-happen state, not a remote-provider failure and not a legitimately-missing capability) | kept |
| `upsert_spec()`, `GithubAdapter::Auto` arm (~line 140) | `Internal` | genuine (same invariant) | kept |
| `upsert_tasks()`, `GithubAdapter::Auto` arm (~line 157) | `Internal` | genuine (same invariant) | kept |

All other sites in this file (`Validation`, `Auth`, `Unsupported`, `Transport`) match their documented meaning: `Auth` for missing/invalid GitHub tokens, `Unsupported` for the not-yet-wired MCP-adapter mutation paths, `Transport` for GraphQL response/field decoding failures. No migrations needed.

## `apps/specify-mcp/src/orchestrator/provider.rs` (T009)

| Site | Variant | Classification | Disposition |
|---|---|---|---|
| `LocalProvider::upsert_spec()` — `fs::create_dir_all`/`fs::write` failures (~lines 162, 166) | `Internal` | genuine (local filesystem I/O for the local/mock provider implementation — not a remote-provider failure, but also not orchestration-unrelated business logic; it's the "provider" boundary for local mode) | kept |
| `LocalProvider::upsert_tasks()` — same pattern (~lines 178, 181) | `Internal` | genuine | kept |

**Contract update**: `ProviderError::Internal`'s doc comment (`crates/specify-provider-core/src/errors.rs`, updated in T007) was broadened to explicitly cover "local filesystem I/O for a local/mock provider" and "orchestration-internal logic errors" — both categories found here and in the GitHub crate are legitimate uses, not misuse. This resolves the tension flagged in `research.md`/`plan.md` between the original narrower "provider-core only" contract draft and the real call sites.

## `crates/specify-provider-core/src/{registry.rs,contract.rs,usecases/sync.rs}` and `crates/specify-provider-linear/src/lib.rs` (T010)

| Site | Variant | Classification | Disposition |
|---|---|---|---|
| `registry.rs` — provider not registered (~line 29) | `Unsupported` | genuine (capability genuinely not registered) | kept |
| `registry.rs` — conformance gate failure (~line 37) | `Conformance` | genuine (matches variant exactly) | kept |
| `contract.rs` | — | no `ProviderError` construction/match sites found | N/A |
| `usecases/sync.rs` — label policy violations (~lines 52, 70, 78) | `Validation` | genuine (malformed/invalid sync request data) | kept |
| `specify-provider-linear/src/lib.rs` (~line 63) | `Validation` | genuine | kept |

## T011 outcome

Every `ProviderError` construction/match site audited (T008–T010) is a genuine use once `Internal`'s contract was correctly scoped (T007) to include local-provider I/O and orchestration-internal invariant violations alongside provider-core-internal failures. **No call-site migrations were required** — the original narrower contract draft in `plan.md`'s Phase 1 design was the thing that needed correcting, not the code. `contracts/provider-error-contract.md` should be read together with the broadened doc comments in `errors.rs` as the final, accurate contract (see note below).

## Correction to `contracts/provider-error-contract.md`

The original contract table said `Internal` is "provider-core only — provider crates MUST NOT construct `Internal` for their own failures." The audit shows this was too strict: `specify-provider-github` legitimately uses `Internal` for an unreachable invariant, and the MCP app's `LocalProvider` legitimately uses it for local I/O. The corrected rule (now reflected in the `errors.rs` doc comments) is: **`Internal` is for failures not attributable to a specific remote provider call** — orchestration invariant violations and local/mock-provider I/O both qualify; failures a provider *can* attribute to a real remote call (auth, transport, validation, conflict) must still use the more specific variant instead.
