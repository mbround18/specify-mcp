---
description: Convert tasks into a parent-issue-first GitHub workflow with checklist sync and selective child issues.
tools: ['github/github-mcp-server/list_issues', 'github/github-mcp-server/issue_write']
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Pre-Execution Checks

**Check for extension hooks (before tasks-to-issues conversion)**:
- Check if `.specify/extensions.yml` exists in the project root.
- If it exists, read it and look for entries under the `hooks.before_taskstoissues` key
- If the YAML cannot be parsed or is invalid, skip hook checking silently and continue normally
- Filter out hooks where `enabled` is explicitly `false`. Treat hooks without an `enabled` field as enabled by default.
- For each remaining hook, do **not** attempt to interpret or evaluate hook `condition` expressions:
  - If the hook has no `condition` field, or it is null/empty, treat the hook as executable
  - If the hook defines a non-empty `condition`, skip the hook and leave condition evaluation to the HookExecutor implementation
- For each executable hook, output the following based on its `optional` flag:
  - **Optional hook** (`optional: true`):
    ```
    ## Extension Hooks

    **Optional Pre-Hook**: {extension}
    Command: `/{command}`
    Description: {description}

    Prompt: {prompt}
    To execute: `/{command}`
    ```
  - **Mandatory hook** (`optional: false`):
    ```
    ## Extension Hooks

    **Automatic Pre-Hook**: {extension}
    Executing: `/{command}`
    EXECUTE_COMMAND: {command}

    Wait for the result of the hook command before proceeding to the Outline.
    ```
    After emitting the block above you MUST actually invoke the hook and wait for it to finish before continuing. Run it the same way you would run the command yourself in this agent/session (the invocation may differ from the literal `{command}` id shown above, e.g. a skills-mode agent runs it as `/skill:speckit-...` or `$speckit-...`). Emitting the block alone does not run the hook.
- If no hooks are registered or `.specify/extensions.yml` does not exist, skip silently

## Outline

1. Run `.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks` from repo root and parse FEATURE_DIR and AVAILABLE_DOCS list. All paths must be absolute. For single quotes in args like "I'm Groot", use escape syntax: e.g 'I'\''m Groot' (or double-quote if possible: "I'm Groot").
1. **IF EXISTS**: Load `.specify/memory/constitution.md` for project principles and governance constraints.
1. From the executed script, extract the path to **tasks**. Also load `spec.md` from `FEATURE_DIR` if present to derive a human-readable feature title.
1. Get the Git remote by running:

```bash
git config --get remote.origin.url
```

> [!CAUTION]
> ONLY PROCEED TO NEXT STEPS IF THE REMOTE IS A GITHUB URL

1. Parse all task rows from `tasks.md`:
   - Match task IDs using `\bT\d{3}[a-z]?\b` (supports `T001` and suffixed IDs like `T017a`).
   - Strip the leading `- [ ]` and markers like `[P]`, `[US#]` to obtain clean `{task_id}` and `{description}` values.
   - Preserve original order for checklist rendering.
1. Determine split policy (default: parent-first, selective child issues):
   - Use **default behavior** when no explicit policy is found:
     - Keep every task in the parent issue checklist.
     - Create child issues only for tasks that match explicit split signals.
   - If `specify.toml` provides a split policy section, honor it.
   - Allow user input to override policy for this run (for example: "all child issues" or "parent only").
   - Default split signals:
     - task line includes `[split]` or `#split`
     - task line includes `[blocked]`
     - task line includes an explicit owner marker (for example `@username`)
     - task line includes high-risk/large markers such as `[risk:high]`, `[risk:critical]`, `[size:l]`, `[size:xl]`
1. **Fetch existing issues for deduplication and reconciliation** using `list_issues`:
   - Do not pass `state` so both open and closed issues are returned.
   - Use `perPage: 100` and paginate with `after` from `endCursor`.
   - Build:
     - `existing_task_issues`: title matches `\bT\d{3}[a-z]?\b` and maps to task IDs.
     - `existing_parent_issue`: issue body contains marker `<!-- speckit:feature=<feature_slug>;role=parent -->`.
   - Stop early once all needed task IDs plus parent marker state are resolved, or when pagination ends.
1. Create or update the **single parent issue** for the feature:
   - Parent title format: `Spec: <feature title> [<feature_slug>]`.
   - Parent body MUST include:
     - marker line: `<!-- speckit:feature=<feature_slug>;role=parent -->`
     - links to `spec.md`, `plan.md`, and `tasks.md` (if present)
     - canonical checklist of every task in source order as `- [ ] T001 <description>`
   - If parent issue already exists, update it in place instead of creating a duplicate.
1. For each task that matches split policy, ensure a child task issue exists:
   - Canonical child title: `T001: <description>`.
   - Reuse existing child issue when one already exists for the task ID.
   - Otherwise create a new child issue and include backlink to the parent issue and feature marker:
     - `Parent: #<parent_issue_number>`
     - `<!-- speckit:feature=<feature_slug>;task=T001;role=child -->`
1. Reconcile parent checklist with child issue links:
   - For split tasks with child issues, append stable references in checklist text (for example `- [ ] T001 <description> (#123)`).
   - Keep non-split tasks as checklist-only items in the parent issue.
   - Update parent issue once with final checklist content.
1. Report created and reused artifacts clearly:
   - parent issue URL
   - created child issue URLs
   - reused/skipped task IDs

> [!CAUTION]
> UNDER NO CIRCUMSTANCES EVER CREATE ISSUES IN REPOSITORIES THAT DO NOT MATCH THE REMOTE URL

## Post-Execution Checks

**Check for extension hooks (after tasks-to-issues conversion)**:
Check if `.specify/extensions.yml` exists in the project root.
- If it exists, read it and look for entries under the `hooks.after_taskstoissues` key
- If the YAML cannot be parsed or is invalid, skip hook checking silently and continue normally
- Filter out hooks where `enabled` is explicitly `false`. Treat hooks without an `enabled` field as enabled by default.
- For each remaining hook, do **not** attempt to interpret or evaluate hook `condition` expressions:
  - If the hook has no `condition` field, or it is null/empty, treat the hook as executable
  - If the hook defines a non-empty `condition`, skip the hook and leave condition evaluation to the HookExecutor implementation
- For each executable hook, output the following based on its `optional` flag:
  - **Optional hook** (`optional: true`):
    ```
    ## Extension Hooks

    **Optional Hook**: {extension}
    Command: `/{command}`
    Description: {description}

    Prompt: {prompt}
    To execute: `/{command}`
    ```
  - **Mandatory hook** (`optional: false`):
    ```
    ## Extension Hooks

    **Automatic Hook**: {extension}
    Executing: `/{command}`
    EXECUTE_COMMAND: {command}
    ```
    After emitting the block above you MUST actually invoke the hook and wait for it to finish before continuing. Run it the same way you would run the command yourself in this agent/session (the invocation may differ from the literal `{command}` id shown above, e.g. a skills-mode agent runs it as `/skill:speckit-...` or `$speckit-...`). Emitting the block alone does not run the hook.
- If no hooks are registered or `.specify/extensions.yml` does not exist, skip silently
