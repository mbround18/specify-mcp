# Speckit MCP Operating Model (Stage-Guided)

## Purpose

Define one consistent, stage-driven process for using Speckit through MCP so users do not need to remember or invoke specific stage commands manually.

## Canonical Stage Catalog

| Stage ID | Name | User Intent | Primary Output |
|---|---|---|---|
| S0 | Setup & Readiness | "Get me ready" | Workspace initialized, config resolved |
| S1 | Discovery & Clarification | "Help me shape this" | Clarified scope, open questions resolved |
| S2 | Specification | "Write the spec" | `spec.md` with stories, FRs, SCs |
| S3 | Planning | "Design how we'll build it" | `plan.md` with architecture and delivery waves |
| S4 | Tasking | "Break this into work" | `tasks.md` with dependency-aware execution backlog |
| S5 | Execution | "Start implementation" | Code changes linked to tasks |
| S6 | Convergence & Gaps | "What did we miss?" | Remaining gaps appended as tasks |
| S7 | Rollout & Handoff | "Ship safely" | Rollout checklist, runbook, migration notes |

## Stage Header Standard

Every stage artifact MUST include this header block at the top:

```md
## Stage Header
- Stage ID: S#
- Stage Name: <name>
- Purpose: <why this stage exists>
- Entry Criteria: <conditions to start>
- Inputs: <required artifacts/context>
- Outputs: <files/results produced>
- Exit Criteria: <conditions to mark done>
- Owner: <human/team/agent role>
- Last Updated: <ISO timestamp>
```

## Artifact Positions

- `specs/<feature>/spec.md` -> S2
- `specs/<feature>/plan.md` -> S3
- `specs/<feature>/tasks.md` -> S4
- `specs/<feature>/speckit-mcp-operating-model.md` -> Global stage definitions
- `specs/<feature>/subspec-*.md` -> Deep-dive design slices for S3/S4

## Guided Mode (No Stage Command Required)

MCP should expose a guided entrypoint (`mcp.speckit.guide`) that:

1. Detects active feature and reads stage ledger.
2. Infers likely stage from user intent + artifact state.
3. Executes the appropriate stage workflow.
4. Returns:
   - completed action
   - current stage
   - next best step
   - any blocker needing user input

## Stage State Ledger

Persist stage state at `.specify/metadata/stage-state.json`.

Suggested schema:

```json
{
  "feature": "001-mcp-provider-integrations",
  "current_stage": "S3",
  "stages": {
    "S0": {"status":"done","updated_at":"..."},
    "S1": {"status":"done","updated_at":"..."},
    "S2": {"status":"done","updated_at":"..."},
    "S3": {"status":"in_progress","updated_at":"...","blockers":[]}
  },
  "next_best_step": "Generate dependency-grouped implementation tasks",
  "blockers": []
}
```

## Stage Completion Rules

- A stage is **done** only when its exit criteria are met and recorded in the ledger.
- If required artifacts are missing or stale, stage status is **blocked** or **in_progress**.
- `next_best_step` must always be emitted after each MCP workflow operation.

## Operational Policies

- Prefer guided mode by default; explicit stage commands remain optional overrides.
- Never silently skip a required stage gate.
- Always surface unresolved clarifications before moving from S1->S2 and S2->S3.
- Track all stage transitions with timestamped ledger updates.
