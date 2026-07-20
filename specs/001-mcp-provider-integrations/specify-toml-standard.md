# `specify.toml` Standard

## Schema Support

TOML does not include native schema enforcement in the format itself.  
For this project, schema validation is implemented by:

1. Parsing TOML.
2. Converting to JSON.
3. Validating against `schemas/specify.schema.json`.
4. Applying runtime semantic validation rules in Rust (`RootConfig::validate`).

## Canonical Shape

```toml
[specify]
version = 1
mode = "local" # local | github | linear

[providers.local]
enabled = true
spec_root = "specs"

[providers.github]
enabled = false
repo = "owner/repo"
adapter = "auto" # auto | graphql | mcp
adapter_preference = ["graphql", "mcp"]
issue_labels = ["spec", "task"]
dry_run = false

[providers.github.issue_sync]
topology = "hybrid" # parent-only | hybrid | all-children
split_signals = ["[split]", "#split", "[blocked]", "[risk:high]", "[risk:critical]", "[size:l]", "[size:xl]"]
close_orphaned_children = true

[providers.linear]
enabled = false
team_key = ""
project_id = ""
default_priority = 2
```

## GitHub Adapter Selection Rules

1. `adapter = "graphql"`: force direct GitHub GraphQL API mutations (`https://api.github.com/graphql`).
2. `adapter = "mcp"`: force GitHub MCP adapter (fails explicitly if unavailable).
3. `adapter = "auto"`: use first available adapter in `adapter_preference`.
4. GraphQL token resolution order: `GITHUB_TOKEN` -> `gh auth token` (fallback).

## Validation Commands

- Initialization + repo autofill: `cargo run -q -p specify-mcp -- init`
- Runtime validation: `cargo run -q -p specify-mcp -- validate-config`
- Issue sync (native): `cargo run -q -p specify-mcp -- taskstoissues '{"feature":"001-mcp-provider-integrations","story":"US3","dry_run":true}'`
- Normal operation also validates config before command execution.

## Repo Autofill on Init

During `init`, if `[providers.github].repo` is empty or set to `owner/repo`, the MCP app tries:

1. `git remote get-url origin`
2. Parse GitHub remote URL formats (`https://github.com/owner/repo.git`, `git@github.com:owner/repo.git`, etc.)
3. Write the inferred `owner/repo` back to `specify.toml`
