# Contract: `ProviderError` Usage Contract

**Owner surface**: `crates/specify-provider-core/src/errors.rs`

**Applies to**: User Story 2 (provider error consolidation)

## Contract

Each `ProviderError` variant has exactly one intended meaning, enforced by doc-comment and audit rather than the type system (this is an existing `String`-payload enum; adding newtype wrappers per variant is out of scope per the Assumptions section — no new external dependencies):

| Variant | Intended meaning | Allowed callers |
|---|---|---|
| `Auth(String)` | Provider rejected credentials or the session/token is invalid | Any provider crate (`specify-provider-github`, `specify-provider-linear`), provider-core registry on activation |
| `Permission(String)` | Authenticated but the acting identity lacks rights for the requested operation | Any provider crate |
| `Transport(String)` | Network/HTTP/protocol-level failure reaching the provider | Any provider crate's client layer |
| `Validation(String)` | Provider rejected a request due to malformed/invalid input data | Any provider crate, provider-core's `usecases/sync.rs` when relaying a provider's validation response |
| `Conflict(String)` | Provider reports a state conflict (e.g. concurrent edit, duplicate resource) | Any provider crate |
| `Unsupported(String)` | Requested capability is not implemented by this provider | `specify-provider-core::registry`, any provider crate's `.capabilities()`/`.supports_contract_baseline()` implementation |
| `Conformance(String)` | Provider's behavior violates the shared provider contract (conformance suite failure) | `specify-provider-core::contract`, `tests/conformance` |
| `Internal(String)` | Unexpected failure not attributable to a specific remote provider call: orchestration-internal invariant violations, or local filesystem I/O for a local/mock provider implementation | Any crate — but only for failures that are *not* attributable to a real remote provider call. If a provider can trace a failure to auth, transport, validation, or a conflict from the actual remote call, it MUST use that more specific variant instead of `Internal` |

A construction or match site is **out of contract** if it uses `ProviderError` for a failure that is not a provider-boundary condition at all (e.g. a local file-parsing error inside `apps/specify-mcp` that has nothing to do with a provider) — such sites must migrate to an existing, more specific error type (e.g. the `String`-based `Result<_, String>` already used elsewhere in `apps/specify-mcp`, per FR-005).

## Rationale

This table is the artifact FR-004/FR-006 require: it lets a future provider implementer choose the right variant without reading the GitHub/Linear implementations first, and gives the audit (FR-005, SC-003) a checklist to classify every existing call site against.

## Verification

- `crates/specify-provider-core/tests/orchestration_dispatch.rs` and `crates/specify-provider-core/tests/metadata_required_fields.rs` MUST continue to pass.
- The `tests/conformance` suite (`make test-conformance`) MUST continue to pass, since it is the primary regression gate for provider-boundary error behavior across GitHub and Linear.
- After the audit, this table (or its doc-comment equivalent on the enum) is the source of truth referenced by SC-003/SC-004 — no separate spec document should duplicate it once the doc comments land in `errors.rs`.

## Post-audit correction (2026-07-20)

The `Internal` row above was broadened from the original design-time draft ("provider-core only") after the audit (`../audit-notes/provider-error-sites.md`) found legitimate `Internal` usage in `specify-provider-github` (unreachable adapter-resolution invariant) and in the MCP app's `LocalProvider` (local filesystem I/O). See the audit note for the full site-by-site classification. The doc comments on `ProviderError` in `crates/specify-provider-core/src/errors.rs` reflect this corrected scope.
