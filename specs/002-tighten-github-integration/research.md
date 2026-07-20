# Research: GitHub Integration Tightening

## Stage Header

- **Stage ID**: S3-P0
- **Stage Name**: Outline & Research
- **Purpose**: Resolve technical decisions needed before design/contracts.
- **Inputs**: `spec.md`, current provider-core and provider adapter crates.
- **Outputs**: Decision log for orchestration, metadata, labels, and conformance.
- **Exit Criteria**: No unresolved clarifications in plan technical context.

## Decisions

### Decision 1: Orchestration pivot must live in provider-core use-cases
- **Decision**: Introduce provider-core orchestration interfaces (`plan_sync`, `apply_sync`) and route all provider execution through registry dispatch.
- **Rationale**: Keeps orchestration behavior stable across provider pivots and satisfies FR-001/FR-002/FR-017.
- **Alternatives considered**:
  - Keep orchestration in app crate with provider-specific branches (rejected: preserves coupling).
  - Duplicate orchestration logic per provider (rejected: high drift risk).

### Decision 2: Use first-class metadata objects with explicit required fields
- **Decision**: Strengthen provider-core model with `ExecutionContext`, `IntegrationIdentity`, `SourceOfTruthMarker`, `LifecycleMetadata`, `RelationshipMetadata`, `MilestoneProjectLinkage`, and ledger-backed reconciliation records.
- **Rationale**: Addresses FR-005..FR-012 and creates deterministic reconciliation inputs.
- **Alternatives considered**:
  - Keep metadata as ad-hoc provider payload maps (rejected: weak validation, poor portability).
  - Store only remote IDs and status (rejected: insufficient for drift correction).

### Decision 3: Define a canonical label policy contract
- **Decision**: Create provider-agnostic lifecycle/relationship label vocabularies plus provider mapping adapters.
- **Rationale**: Required for FR-009/FR-010 and prevents provider-specific label semantics from leaking into orchestration.
- **Alternatives considered**:
  - Let each provider label freely (rejected: no conformance baseline).
  - Drop labels for providers with weaker support (rejected: violates first-class label requirement).

### Decision 4: Deterministic reconciliation outcomes are a contract, not diagnostics only
- **Decision**: Persist and emit per-item outcomes (`created`, `updated`, `unchanged`, `drift_corrected`, `conflict_flagged`) on every run.
- **Rationale**: Enables safe reruns and partial-failure recovery (FR-013..FR-016).
- **Alternatives considered**:
  - Emit run-level summary only (rejected: cannot safely recover item-level state).
  - Best-effort reconciliation without explicit outcome enum (rejected: nondeterministic operators experience).

### Decision 5: Conformance suite must run identical fixtures across providers
- **Decision**: Build cross-provider conformance harness in provider-core and require parity for GitHub and Linear.
- **Rationale**: Enforces SC-002 and prevents regression in pivotability.
- **Alternatives considered**:
  - Separate provider-specific test suites only (rejected: does not prove parity).
  - Snapshot-only golden tests at app level (rejected: weak contract coverage).

### Decision 6: Capability matrix is required for activation and diagnostics
- **Decision**: Extend provider capability reporting to include labels, relationships, milestone/project linkage, assignee, and reconciliation support.
- **Rationale**: Supports FR-004 and explicit unsupported behavior handling.
- **Alternatives considered**:
  - Keep coarse booleans (`supports_dry_run`, etc.) only (rejected: insufficient granularity).
  - Fail late at mutation time (rejected: poor operator feedback).
