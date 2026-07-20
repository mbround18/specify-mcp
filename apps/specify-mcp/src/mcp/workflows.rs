use std::env;
use std::path::PathBuf;

use serde::Serialize;
use specify_provider_core::SpecRecord;

use crate::config::discovery::discover_config;
use crate::config::validation::validate_toml_with_schema;
use crate::mcp::setup::ensure_runtime_paths;
use crate::mcp::taskstoissues::run_taskstoissues;
use crate::orchestrator::guide::run_guided;
use crate::orchestrator::provider::{build_registry, select_provider_with_override};

#[derive(Debug, Serialize)]
struct StatusResponse {
    config_source: String,
    mode: String,
    provider_health: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    github_write_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    github_write_detail: Option<String>,
}

pub fn run_cli() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let repo_root = env::current_dir().map_err(|e| format!("cwd unavailable: {e}"))?;
    ensure_runtime_paths(&repo_root)?;

    if args.first().map(|s| s.as_str()) == Some("init") {
        return Err(
            "init is MCP tool-first. Run tool call `init` from your MCP client (recommended)."
                .to_string(),
        );
    }

    if args.first().map(|s| s.as_str()) == Some("guide") {
        let intent = if args.len() > 1 {
            args[1..].join(" ")
        } else {
            "continue".to_string()
        };
        let result = run_guided(&repo_root, &intent)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&result)
                .map_err(|e| format!("failed to serialize guided response: {e}"))?
        );
        return Ok(());
    }

    if args.first().map(|s| s.as_str()) == Some("status") {
        let out = status(repo_root)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&out)
                .map_err(|e| format!("failed to serialize status response: {e}"))?
        );
        return Ok(());
    }

    if args.first().map(|s| s.as_str()) == Some("sync-sample") {
        sync_sample(repo_root)?;
        return Ok(());
    }

    if args.first().map(|s| s.as_str()) == Some("taskstoissues") {
        let payload = if args.len() > 1 {
            let raw = args[1..].join(" ");
            serde_json::from_str::<serde_json::Value>(&raw).map_err(|e| {
                format!("taskstoissues arguments must be JSON object, parse failed: {e}")
            })?
        } else {
            serde_json::json!({})
        };
        let out = run_taskstoissues(repo_root.as_path(), &payload)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&out)
                .map_err(|e| format!("failed to serialize taskstoissues response: {e}"))?
        );
        return Ok(());
    }

    if args.first().map(|s| s.as_str()) == Some("validate-config") {
        validate_config(repo_root)?;
        return Ok(());
    }

    println!("usage:");
    println!("  specify-mcp guide <intent text>");
    println!("  specify-mcp status");
    println!("  specify-mcp sync-sample");
    println!("  specify-mcp taskstoissues '<json args>'");
    println!("  specify-mcp validate-config");
    println!("  (init is available as MCP tool and is recommended first)");
    Ok(())
}

fn status(repo_root: PathBuf) -> Result<StatusResponse, String> {
    let resolved = discover_config(&repo_root)?;
    let registry = build_registry(&resolved.config)?;
    let provider =
        select_provider_with_override(&registry, &resolved.config.specify.mode, None, &repo_root)?;
    let health = provider.health_check().map_err(|e| format!("{e}"))?;
    let mut github_write_access = None;
    let mut github_write_detail = None;
    if resolved.config.specify.mode == "github" || resolved.config.providers.github.enabled {
        let check = std::process::Command::new("gh")
            .args([
                "repo",
                "view",
                &resolved.config.providers.github.repo,
                "--json",
                "id",
            ])
            .output();
        match check {
            Ok(out) if out.status.success() => {
                github_write_access = Some(true);
                github_write_detail = Some("gh repo view succeeded".to_string());
            }
            Ok(out) => {
                github_write_access = Some(false);
                github_write_detail = Some(String::from_utf8_lossy(&out.stderr).trim().to_string());
            }
            Err(e) => {
                github_write_access = Some(false);
                github_write_detail = Some(format!("failed to execute gh: {e}"));
            }
        }
    }

    Ok(StatusResponse {
        config_source: resolved.source_path.display().to_string(),
        mode: resolved.config.specify.mode,
        provider_health: health.detail,
        github_write_access,
        github_write_detail,
    })
}

fn sync_sample(repo_root: PathBuf) -> Result<(), String> {
    let resolved = discover_config(&repo_root)?;
    let registry = build_registry(&resolved.config)?;
    let provider =
        select_provider_with_override(&registry, &resolved.config.specify.mode, None, &repo_root)?;
    let spec = SpecRecord {
        key: "001-mcp-provider-integrations".to_string(),
        title: "MCP Provider Integrations".to_string(),
        status: "draft".to_string(),
    };
    let tasks = Vec::new();
    let result = provider.sync(&spec, &tasks).map_err(|e| format!("{e}"))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&result)
            .map_err(|e| format!("failed to serialize sync result: {e}"))?
    );
    Ok(())
}

fn validate_config(repo_root: PathBuf) -> Result<(), String> {
    let path = repo_root.join("specify.toml");
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    validate_toml_with_schema(&raw)?;
    let resolved = discover_config(&repo_root)?;
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "source": resolved.source_path.display().to_string(),
            "mode": resolved.config.specify.mode
        })
    );
    Ok(())
}
