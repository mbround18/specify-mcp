# Feature Specification: MCP Spec Kit Zero-Install + Pluggable Issue Providers

**Feature Branch**: `001-mcp-provider-integrations`

**Created**: 2026-07-12

**Status**: Draft

**Input**: User description: "Initialize Specify in this repo, support Rust MCP zero-install setup, and define modular GitHub/Linear-backed spec/task management configurable from `specify.toml` or `.github` config paths."

## Stage Header

- **Stage ID**: S2
- **Stage Name**: Specification
- **Purpose**: Define user outcomes, functional requirements, constraints, and measurable success criteria.
- **Entry Criteria**: Problem statement exists; initial scope captured.
- **Inputs**: User request, discovered constraints, repository context.
- **Outputs**: `spec.md` with user stories, FRs, SCs, assumptions, and key entities.
- **Exit Criteria**: Requirements and success criteria are explicit, testable, and internally consistent.
- **Owner**: Product/architecture authoring flow (human + MCP assistant).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Bootstrap without local Specify CLI (Priority: P1)

A developer opens a repository with an MCP-capable agent but does not have `specify` installed locally. They can run a Rust MCP setup action and immediately use Spec Kit workflows.

**Why this priority**: This is the on-ramp. Without this, cross-team adoption fails on day one.

**Independent Test**: In a clean environment without `specify` in `PATH`, execute MCP setup, then generate a new feature spec and plan from the agent.

**Acceptance Scenarios**:

1. **Given** `specify` is not installed, **When** the user invokes `mcp.speckit.setup`, **Then** repo-local Spec Kit assets are installed and usable.
2. **Given** setup completed through MCP, **When** the user runs a Spec Kit workflow command, **Then** it succeeds without requiring global CLI install.

---

### User Story 2 - Use GitHub Issues as spec/task backend (Priority: P1)

A team chooses GitHub-backed tracking so specs/tasks are managed as issues instead of local markdown files.

**Why this priority**: This enables workflow alignment with existing GitHub issue management.

**Independent Test**: Configure GitHub provider, create a spec from prompt, confirm issue set is created/updated and local files are optional.

**Acceptance Scenarios**:

1. **Given** provider mode is `github`, **When** the user creates a spec, **Then** the system creates a parent issue and child task issues with links.
2. **Given** both `gh` CLI and GitHub MCP are possible, **When** capability detection runs, **Then** the system selects the best available adapter and records the choice.
3. **Given** GitHub provider is unavailable/auth fails, **When** the user requests spec creation, **Then** the system returns a clear error and offers configured fallback behavior.
4. **Given** a standard spec with actionable tasks, **When** task-to-issues sync runs in default mode, **Then** the system updates one parent spec issue containing a canonical checkbox task list and creates child task issues only when split criteria are met.

---

### User Story 3 - Use Linear as spec/task backend (Priority: P2)

A team selects Linear-backed tracking so specs/tasks are represented in Linear projects/issues.

**Why this priority**: Linear is a core system used by the team and must be first-class.

**Independent Test**: Configure Linear provider, create/update spec, and verify synchronized parent work item + tasks in Linear.

**Acceptance Scenarios**:

1. **Given** provider mode is `linear`, **When** spec/task generation runs, **Then** Linear issues are created with status/priority/assignee mapping from config.
2. **Given** a spec update modifies task list, **When** sync runs, **Then** existing Linear tasks are updated idempotently instead of duplicated.

---

### User Story 4 - Add future provider modules (Priority: P3)

A maintainer can add a new integration (for example Jira) by implementing a provider interface without changing core planning logic.

**Why this priority**: Keeps long-term expansion low-risk and avoids hard-coded provider branches.

**Independent Test**: Add a stub `jira` provider module implementing the interface; register it; run provider contract tests.

**Acceptance Scenarios**:

1. **Given** a new provider module is registered, **When** config selects it, **Then** core spec/task orchestration uses it with no core code edits.
2. **Given** a provider fails a contract requirement, **When** startup validation runs, **Then** initialization fails with actionable diagnostics.

### Edge Cases

- Config exists in multiple supported locations with conflicting provider settings.
- Provider selected but required auth token is missing.
- `gh` installed but user not authenticated; GitHub MCP available but disconnected.
- Remote issue exists but was manually deleted/renamed between sync runs.
- Network outage during partial issue creation (must support resume/retry without duplication).
- Workspace intentionally mixes local-file specs and remote-provider specs across different projects.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support zero-install Spec Kit bootstrap through a Rust MCP setup capability that installs required repo-local assets.
- **FR-002**: System MUST operate even when global `specify` CLI is absent, using MCP-exposed setup/workflow operations.
- **FR-003**: System MUST support configuration discovery in this precedence order: repository root `specify.toml` → `.github/config/specify.toml` → `.github/specify.toml` → `.github/*` files matching `specify*.toml` (first deterministic lexical match).
- **FR-004**: System MUST define a provider architecture with a stable interface for spec/task CRUD, sync, status transitions, and link resolution.
- **FR-005**: System MUST include built-in providers: `local` (markdown files), `github`, and `linear`.
- **FR-006**: GitHub provider MUST support direct GraphQL execution (`https://api.github.com/graphql`) and GitHub MCP as selectable adapters.
- **FR-007**: GitHub provider MUST auto-detect adapter availability and choose based on configured preference and runtime capability.
- **FR-008**: Linear provider MUST map spec and task entities to Linear issues with configurable team/project/state/priority mappings.
- **FR-009**: System MUST persist provider metadata (remote IDs, sync hashes, timestamps) to enable idempotent upsert behavior.
- **FR-010**: System MUST support a per-project mode switch between local file storage and remote provider storage without code changes.
- **FR-011**: System MUST provide explicit, non-silent error responses for auth, permission, and network failures.
- **FR-012**: System MUST support dry-run preview of provider actions before creating/updating remote issues.
- **FR-013**: System MUST expose extension registration for future providers (e.g., Jira) via module registration in config and runtime registry.
- **FR-014**: System MUST include provider contract tests that every provider implementation must pass.
- **FR-015**: System MUST provide migration helpers to pivot an existing local spec/task set into GitHub or Linear while preserving cross-links.
- **FR-016**: Repository layout MUST follow a workspace split with `apps/specify-mcp` for the executable MCP app and `crates/*` for reusable supporting libraries.
- **FR-017**: A base provider crate MUST define provider contracts, shared domain types, capability model, and registry hooks.
- **FR-018**: A GitHub provider crate MUST implement the base provider contract and encapsulate GitHub-specific adapters (`gh` and MCP).
- **FR-019**: A Linear provider crate MUST implement the base provider contract and encapsulate Linear API-specific mapping and sync logic.
- **FR-020**: The app crate (`apps/specify-mcp`) MUST depend on provider crates through stable interfaces and avoid provider-specific business logic in the app layer.
- **FR-021**: MCP workflow MUST support a guided "auto-stage" mode so users can describe intent without naming explicit Speckit stages.
- **FR-022**: Each stage artifact MUST include a standardized header with stage ID, purpose, entry criteria, inputs, outputs, and exit criteria.
- **FR-023**: System MUST persist stage state in a machine-readable ledger to track current position, completion timestamps, and blockers.
- **FR-024**: System MUST expose a "next best step" response based on stage state, missing artifacts, and unresolved clarifications.
- **FR-025**: Stage definitions and artifact locations MUST be documented in a single operating model document in the feature spec folder.
- **FR-026**: GitHub GraphQL auth token resolution MUST follow: `GITHUB_TOKEN`, then `gh auth token` fallback.
- **FR-027**: GitHub task tracking topology MUST default to one parent issue per spec with an embedded canonical checkbox list for all tasks.
- **FR-028**: System MUST support selective child issue creation from tasks only when a task matches configurable split criteria (for example: explicit owner, blocked state, size/risk threshold, or explicit split tag).
- **FR-029**: Parent issue checklist and any child issue states MUST remain bidirectionally synchronized and idempotent across repeated sync runs.

### Key Entities *(include if feature involves data)*

- **WorkspaceConfig**: Resolved configuration including provider mode, discovery source, adapter preference, and fallback policy.
- **ProviderRegistry**: Runtime registry of available provider modules and capabilities.
- **ProviderAdapter**: Concrete transport implementation (e.g., GitHub MCP adapter, `gh` adapter, Linear API adapter).
- **SpecRecord**: Canonical feature spec metadata (title, scope, status, owner, remote/local identifiers).
- **TaskRecord**: Canonical task metadata linked to `SpecRecord`, including priority and execution state.
- **SyncLedger**: Persistent idempotency and reconciliation state (remote IDs, content hash, last sync timestamp, conflict markers).
- **ProviderCoreCrate**: Shared crate owning traits, registry interfaces, core DTOs, and provider contract test harness.
- **GithubProviderCrate**: GitHub-specific provider implementation crate depending on `ProviderCoreCrate`.
- **LinearProviderCrate**: Linear-specific provider implementation crate depending on `ProviderCoreCrate`.
- **StageDefinition**: Canonical definition of one workflow stage (ID, name, intent, required artifacts, completion gate).
- **StageStateLedger**: Persisted runtime view of stage progression and pending actions for the active feature.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In a clean environment without `specify` binary, 95% of setup attempts complete successfully via MCP in under 90 seconds.
- **SC-002**: GitHub provider can create and reconcile a 1-spec/10-task set with zero duplicate remote issues across three repeated sync runs.
- **SC-003**: Linear provider can create and reconcile a 1-spec/10-task set with zero duplicate remote issues across three repeated sync runs.
- **SC-004**: Adding a new provider requires no edits in core orchestration modules beyond registration and passes provider contract tests.
- **SC-005**: Config discovery selects the expected file/source deterministically in 100% of path-precedence test cases.
- **SC-006**: Workspace build graph compiles with `apps/specify-mcp` + provider crates and no cyclic dependencies.
- **SC-007**: In guided mode, users can complete a full spec->plan->tasks flow without explicitly naming stage commands in at least 90% of pilot sessions.
- **SC-008**: For default GitHub sync mode, at least 80% of pilot specs are tracked with parent-only checklist flow while preserving 100% task traceability and zero duplicate child issues.

## Assumptions

- MCP host environment can execute Rust-based MCP tool calls.
- Teams may prefer either repository-local markdown specs or remote issue trackers depending on project.
- GitHub and Linear credentials are supplied securely outside repository source control.
- Initial version supports one active primary provider per workspace, with migration utilities for switching.
