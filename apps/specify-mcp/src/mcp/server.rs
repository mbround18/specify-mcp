use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use serde_json::{json, Value};
use specify_provider_core::SpecRecord;

use crate::config::discovery::discover_config;
use crate::config::validation::validate_toml_with_schema;
use crate::mcp::setup::{ensure_runtime_paths, initialize_repo};
use crate::mcp::taskstoissues::run_taskstoissues;
use crate::orchestrator::guide::run_guided;
use crate::orchestrator::provider::{build_registry, select_provider_with_override};

#[derive(Clone, Copy)]
enum WireFormat {
    ContentLength,
    JsonLine,
}

pub fn run_stdio_server() -> Result<(), String> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();

    while let Some((req, wire_format)) = read_message(&mut reader)? {
        let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");
        let id = req.get("id").cloned();

        match method {
            "initialize" => {
                if let Some(id) = id {
                    let protocol_version = req
                        .get("params")
                        .and_then(|v| v.get("protocolVersion"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("2025-11-25");
                    let result = json!({
                        "protocolVersion": protocol_version,
                        "capabilities": { "tools": { "listChanged": false } },
                        "serverInfo": { "name": "specify-mcp", "version": "0.1.0" }
                    });
                    write_response(&mut writer, &id, result, wire_format)?;
                }
            }
            "notifications/initialized" | "initialized" => {}
            "ping" => {
                if let Some(id) = id {
                    write_response(&mut writer, &id, json!({}), wire_format)?;
                }
            }
            "tools/list" => {
                if let Some(id) = id {
                    write_response(
                        &mut writer,
                        &id,
                        json!({
                            "tools": [
                                {
                                    "name": "init",
                                    "description": "Recommended first run: initialize MCP runtime and autofill providers.github.repo from git origin",
                                    "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
                                },
                                {
                                    "name": "status",
                                    "description": "Show selected mode and provider health",
                                    "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
                                },
                                {
                                    "name": "validate-config",
                                    "description": "Validate specify.toml against schema and semantic rules",
                                    "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
                                },
                                {
                                    "name": "guide",
                                    "description": "Run guided stage flow from natural-language intent",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": { "intent": { "type": "string" } },
                                        "required": ["intent"],
                                        "additionalProperties": false
                                    }
                                },
                                {
                                    "name": "sync-sample",
                                    "description": "Run sample provider sync for quick integration check",
                                    "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
                                },
                                {
                                    "name": "taskstoissues",
                                    "description": "Plan/apply provider-core task sync from tasks.md with runtime provider selection",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "feature": { "type": "string" },
                                            "story": { "type": "string" },
                                            "task_range": { "type": "string" },
                                            "provider": { "type": "string", "enum": ["local", "github", "linear"] },
                                            "dry_run": { "type": "boolean" },
                                            "topology": { "type": "string", "enum": ["parent-only", "hybrid", "all-children"] },
                                            "close_orphaned_children": { "type": "boolean" }
                                        },
                                        "additionalProperties": false
                                    }
                                }
                            ]
                        }),
                        wire_format,
                    )?;
                }
            }
            "tools/call" => {
                if let Some(id) = id {
                    let params = req.get("params").cloned().unwrap_or_else(|| json!({}));
                    let tool_name = params
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let args = params
                        .get("arguments")
                        .cloned()
                        .unwrap_or_else(|| json!({}));
                    match call_tool(&tool_name, &args) {
                        Ok(result) => {
                            write_response(
                                &mut writer,
                                &id,
                                json!({
                                    "content": [{ "type": "text", "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string()) }],
                                    "isError": false
                                }),
                                wire_format,
                            )?;
                        }
                        Err(err) => {
                            write_response(
                                &mut writer,
                                &id,
                                json!({
                                    "content": [{ "type": "text", "text": err }],
                                    "isError": true
                                }),
                                wire_format,
                            )?;
                        }
                    }
                }
            }
            "shutdown" => {
                if let Some(id) = id {
                    write_response(&mut writer, &id, json!({}), wire_format)?;
                }
                break;
            }
            "exit" => break,
            _ => {
                if let Some(id) = id {
                    write_error(
                        &mut writer,
                        &id,
                        -32601,
                        format!("method not found: {method}"),
                        wire_format,
                    )?;
                }
            }
        }
    }

    Ok(())
}

fn call_tool(name: &str, args: &Value) -> Result<Value, String> {
    let repo_root = std::env::current_dir().map_err(|e| format!("cwd unavailable: {e}"))?;
    ensure_runtime_paths(&repo_root)?;

    match name {
        "init" => {
            let autofilled = initialize_repo(&repo_root)?;
            Ok(json!({ "ok": true, "autofilled_repo": autofilled }))
        }
        "status" => status(&repo_root),
        "validate-config" => validate_config(&repo_root),
        "guide" => {
            let intent = args
                .get("intent")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "guide requires { intent: string }".to_string())?;
            let out = run_guided(&repo_root, intent)?;
            serde_json::to_value(out).map_err(|e| format!("serialize guided output failed: {e}"))
        }
        "sync-sample" => sync_sample(&repo_root),
        "taskstoissues" => {
            let raw = run_taskstoissues(repo_root.as_path(), args)?;
            Ok(attach_reconciliation_summary(raw))
        }
        _ => Err(format!("unknown tool: {name}")),
    }
}

fn attach_reconciliation_summary(raw: Value) -> Value {
    let mut counts = std::collections::BTreeMap::<String, u64>::new();
    if let Some(outcomes) = raw.get("outcomes").and_then(|v| v.as_array()) {
        for outcome in outcomes {
            if let Some(kind) = outcome.get("outcome").and_then(|v| v.as_str()) {
                let entry = counts.entry(kind.to_string()).or_insert(0);
                *entry += 1;
            }
        }
    }
    serde_json::json!({
        "sync": raw,
        "reconciliation_summary": counts
    })
}

fn status(repo_root: &PathBuf) -> Result<Value, String> {
    let resolved = discover_config(repo_root)?;
    let registry = build_registry(&resolved.config)?;
    let provider =
        select_provider_with_override(&registry, &resolved.config.specify.mode, None, repo_root)?;
    let health = provider.health_check().map_err(|e| format!("{e}"))?;
    let mut github_write_access = Value::Null;
    let mut github_write_detail = Value::Null;
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
                github_write_access = json!(true);
                github_write_detail = json!("gh repo view succeeded");
            }
            Ok(out) => {
                github_write_access = json!(false);
                github_write_detail =
                    json!(String::from_utf8_lossy(&out.stderr).trim().to_string());
            }
            Err(e) => {
                github_write_access = json!(false);
                github_write_detail = json!(format!("failed to execute gh: {e}"));
            }
        }
    }
    Ok(json!({
        "config_source": resolved.source_path.display().to_string(),
        "mode": resolved.config.specify.mode,
        "provider_health": health.detail,
        "github_write_access": github_write_access,
        "github_write_detail": github_write_detail
    }))
}

fn validate_config(repo_root: &PathBuf) -> Result<Value, String> {
    let path = repo_root.join("specify.toml");
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    validate_toml_with_schema(&raw)?;
    let resolved = discover_config(repo_root)?;
    Ok(json!({
        "ok": true,
        "source": resolved.source_path.display().to_string(),
        "mode": resolved.config.specify.mode
    }))
}

fn sync_sample(repo_root: &PathBuf) -> Result<Value, String> {
    let resolved = discover_config(repo_root)?;
    let registry = build_registry(&resolved.config)?;
    let provider =
        select_provider_with_override(&registry, &resolved.config.specify.mode, None, repo_root)?;
    let spec = SpecRecord {
        key: "001-mcp-provider-integrations".to_string(),
        title: "MCP Provider Integrations".to_string(),
        status: "draft".to_string(),
    };
    let tasks = Vec::new();
    let result = provider.sync(&spec, &tasks).map_err(|e| format!("{e}"))?;
    serde_json::to_value(result).map_err(|e| format!("serialize sync output failed: {e}"))
}

fn read_message(reader: &mut impl BufRead) -> Result<Option<(Value, WireFormat)>, String> {
    loop {
        let mut first_line = String::new();
        let read = reader
            .read_line(&mut first_line)
            .map_err(|e| format!("failed reading header: {e}"))?;
        if read == 0 {
            return Ok(None);
        }

        let first_trim = first_line.trim_end_matches(['\r', '\n']);
        if first_trim.is_empty() {
            continue;
        }

        // Compatibility fallback: some clients send one JSON object per line
        // without Content-Length framing.
        if first_trim.starts_with('{') {
            let value: Value = serde_json::from_str(first_trim)
                .map_err(|e| format!("invalid JSON message: {e}"))?;
            return Ok(Some((value, WireFormat::JsonLine)));
        }

        let mut content_length: Option<usize> = None;
        if let Some((name, value)) = first_trim.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-length") {
                let n = value
                    .trim()
                    .parse::<usize>()
                    .map_err(|e| format!("invalid Content-Length header: {e}"))?;
                content_length = Some(n);
            }
        }

        loop {
            let mut line = String::new();
            let read = reader
                .read_line(&mut line)
                .map_err(|e| format!("failed reading header: {e}"))?;
            if read == 0 {
                return Ok(None);
            }
            let line_trim = line.trim_end_matches(['\r', '\n']);
            if line_trim.is_empty() {
                break;
            }
            if let Some((name, value)) = line_trim.split_once(':') {
                if name.trim().eq_ignore_ascii_case("content-length") {
                    let n = value
                        .trim()
                        .parse::<usize>()
                        .map_err(|e| format!("invalid Content-Length header: {e}"))?;
                    content_length = Some(n);
                }
            }
        }

        let len = content_length.ok_or_else(|| "missing Content-Length header".to_string())?;
        let mut body = vec![0u8; len];
        reader
            .read_exact(&mut body)
            .map_err(|e| format!("failed reading body: {e}"))?;
        let value: Value =
            serde_json::from_slice(&body).map_err(|e| format!("invalid JSON message: {e}"))?;
        return Ok(Some((value, WireFormat::ContentLength)));
    }
}

fn write_response(
    writer: &mut impl Write,
    id: &Value,
    result: Value,
    wire_format: WireFormat,
) -> Result<(), String> {
    write_message(
        writer,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        }),
        wire_format,
    )
}

fn write_error(
    writer: &mut impl Write,
    id: &Value,
    code: i64,
    message: String,
    wire_format: WireFormat,
) -> Result<(), String> {
    write_message(
        writer,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": code,
                "message": message
            }
        }),
        wire_format,
    )
}

fn write_message(
    writer: &mut impl Write,
    payload: Value,
    wire_format: WireFormat,
) -> Result<(), String> {
    let data =
        serde_json::to_vec(&payload).map_err(|e| format!("serialize message failed: {e}"))?;
    match wire_format {
        WireFormat::ContentLength => {
            let header = format!("Content-Length: {}\r\n\r\n", data.len());
            writer
                .write_all(header.as_bytes())
                .map_err(|e| format!("failed writing header: {e}"))?;
            writer
                .write_all(&data)
                .map_err(|e| format!("failed writing payload: {e}"))?;
        }
        WireFormat::JsonLine => {
            writer
                .write_all(&data)
                .map_err(|e| format!("failed writing payload: {e}"))?;
            writer
                .write_all(b"\n")
                .map_err(|e| format!("failed writing line terminator: {e}"))?;
        }
    }
    writer
        .flush()
        .map_err(|e| format!("failed to flush: {e}"))?;
    Ok(())
}
