SHELL := /usr/bin/env bash

.PHONY: build build-release clippy test test-rust test-config test-status test-guide test-init test-sync-sample test-github-graphql-dryrun test-conformance test-us1 test-us2 test-us3

build:
	cargo build -q -p specify-mcp

build-release:
	cargo build --release -q -p specify-mcp

clippy:
	cargo clippy -q --workspace --all-targets

test: test-rust test-config test-status test-guide test-init test-sync-sample test-github-graphql-dryrun test-conformance

test-rust:
	cargo test -q

test-config:
	cargo run -q -p specify-mcp -- validate-config

test-status:
	cargo run -q -p specify-mcp -- status

test-guide:
	cargo run -q -p specify-mcp -- guide "setup project"

test-init:
	@bash -ec 'if cargo run -q -p specify-mcp -- init >/tmp/specify-mcp-test-init.out 2>&1; then echo "expected init CLI subcommand to fail"; cat /tmp/specify-mcp-test-init.out; exit 1; fi; grep -q "init is MCP tool-first" /tmp/specify-mcp-test-init.out'

test-sync-sample:
	cargo run -q -p specify-mcp -- sync-sample

test-conformance:
	cargo test -q -p specify-conformance

test-us1:
	cargo test -q -p specify-provider-core --test orchestration_dispatch
	cargo test -q -p specify-mcp --test taskstoissues_provider_select

test-us2:
	cargo test -q -p specify-provider-core --test metadata_required_fields
	cargo test -q -p specify-provider-core --test label_policy_contract

test-us3:
	cargo test -q -p specify-conformance

# Optional integration smoke test:
# - Requires GitHub sandbox repo and token access.
# - Uses a temporary config file so repo config is restored.
test-github-graphql-dryrun:
	@bash -ec 'if [[ -z "$$SPECIFY_GITHUB_TEST_REPO" ]]; then echo "SKIP test-github-graphql-dryrun: set SPECIFY_GITHUB_TEST_REPO=owner/repo to enable"; exit 0; fi; tmp="$$(mktemp)"; cp specify.toml "$$tmp"; trap "cp \"$$tmp\" specify.toml; rm -f \"$$tmp\"" EXIT; printf "%s\n" "[specify]" "version = 1" "mode = \"github\"" "" "[providers.local]" "enabled = true" "spec_root = \"specs\"" "" "[providers.github]" "enabled = true" "repo = \"$$SPECIFY_GITHUB_TEST_REPO\"" "adapter = \"graphql\"" "adapter_preference = [\"graphql\", \"mcp\"]" "issue_labels = [\"spec\", \"task\", \"speckit\"]" "dry_run = true" "" "[providers.linear]" "enabled = false" "team_key = \"\"" "project_id = \"\"" "default_priority = 2" > specify.toml; cargo run -q -p specify-mcp -- validate-config; cargo run -q -p specify-mcp -- status; cargo run -q -p specify-mcp -- sync-sample'
