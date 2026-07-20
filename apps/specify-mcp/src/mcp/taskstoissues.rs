use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use serde_json::Value;
use specify_provider_core::usecases::sync::{apply_sync, plan_sync};
use specify_provider_core::{
    ExecutionContext, IntegrationIdentity, LifecycleMetadata, ProviderKind, RelationshipMetadata,
    SourceOfTruthMarker, SpecRecord, SyncItemRecord, SyncRequest, TaskRecord,
};

use crate::config::discovery::{discover_config, resolve_mode_with_override};
use crate::orchestrator::ledger::{
    load_latest_provider_sync_ledger, write_provider_sync_ledger, ProviderSyncLedger,
};
use crate::orchestrator::provider::{build_registry, select_provider_with_override};

#[derive(Debug, Clone, Serialize)]
pub struct TaskToIssuesResponse {
    pub feature: String,
    pub provider: ProviderKind,
    pub dry_run: bool,
    pub planned_items: usize,
    pub spec_remote_id: String,
    pub task_remote_ids: Vec<String>,
    pub outcomes: Vec<specify_provider_core::SyncLedgerEntry>,
}

#[derive(Debug, Clone)]
struct Scope {
    feature: Option<String>,
    story: Option<String>,
    task_range: Option<String>,
}

#[derive(Debug, Clone)]
struct FeatureArtifacts {
    slug: String,
    spec_path: PathBuf,
    tasks_path: PathBuf,
}

#[derive(Debug, Clone)]
struct ParsedTask {
    id: String,
    description: String,
    story: Option<String>,
    is_complete: bool,
}

pub fn run_taskstoissues(repo_root: &Path, args: &Value) -> Result<Value, String> {
    let resolved = discover_config(repo_root)?;
    let registry = build_registry(&resolved.config)?;
    let provider_override = args.get("provider").and_then(Value::as_str);
    let mode = resolve_mode_with_override(&resolved.config, provider_override)?;
    let provider = select_provider_with_override(&registry, &mode, None, repo_root)?;

    let scope = parse_scope(args);
    let dry_run = args
        .get("dry_run")
        .and_then(Value::as_bool)
        .unwrap_or(resolved.config.providers.github.dry_run);

    let artifacts = resolve_feature_artifacts(repo_root, scope.feature.as_deref())?;
    let spec_title =
        read_spec_title(&artifacts.spec_path).unwrap_or_else(|| "Untitled Feature".to_string());
    let tasks_raw = fs::read_to_string(&artifacts.tasks_path)
        .map_err(|e| format!("failed to read {}: {e}", artifacts.tasks_path.display()))?;
    let mut tasks = parse_tasks(&tasks_raw);
    tasks = apply_scope(tasks, &scope)?;
    if tasks.is_empty() {
        return Err("no tasks matched requested taskstoissues scope".to_string());
    }

    let run_id = format!(
        "{}-{}",
        artifacts.slug,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("system clock before epoch: {e}"))?
            .as_millis()
    );
    let context = ExecutionContext {
        run_id: run_id.clone(),
        context_type: "feature".to_string(),
        context_value: artifacts.slug.clone(),
        timestamp: chrono_like_timestamp(),
    };

    let spec = SpecRecord {
        key: artifacts.slug.clone(),
        title: spec_title,
        status: if dry_run {
            "dry-run".to_string()
        } else {
            "active".to_string()
        },
    };

    let task_records = tasks
        .iter()
        .map(|task| TaskRecord {
            key: task.id.clone(),
            title: task.description.clone(),
            status: if task.is_complete {
                "done".to_string()
            } else {
                "open".to_string()
            },
            priority: "normal".to_string(),
        })
        .collect::<Vec<_>>();

    let provider_name = format!("{:?}", provider.kind()).to_ascii_lowercase();
    let resume_state = load_latest_provider_sync_ledger(
        &repo_root.join(".specify").join("state").join("sync-ledger"),
        &artifacts.slug,
        &provider_name,
    )?;

    let items = tasks
        .iter()
        .map(|task| to_sync_item(&run_id, &artifacts.slug, task, resume_state.as_ref()))
        .collect::<Vec<_>>();

    let request = SyncRequest {
        context,
        spec,
        tasks: task_records,
        items,
    };
    let plan = plan_sync(provider.as_ref(), &request).map_err(|e| format!("{e}"))?;
    let result = apply_sync(provider.as_ref(), &plan).map_err(|e| format!("{e}"))?;
    let ledger_path = repo_root
        .join(".specify")
        .join("state")
        .join("sync-ledger")
        .join(format!("{}-{}.json", artifacts.slug, run_id));
    write_provider_sync_ledger(
        &ledger_path,
        &ProviderSyncLedger {
            run_id: run_id.clone(),
            provider: provider_name,
            feature: artifacts.slug.clone(),
            entries: result.outcomes.clone(),
        },
    )?;

    let out = TaskToIssuesResponse {
        feature: artifacts.slug,
        provider: result.provider.clone(),
        dry_run,
        planned_items: plan.desired_items.len(),
        spec_remote_id: result.spec_remote_id,
        task_remote_ids: result.task_remote_ids,
        outcomes: result.outcomes,
    };
    serde_json::to_value(out).map_err(|e| format!("failed to serialize taskstoissues: {e}"))
}

fn to_sync_item(
    run_id: &str,
    feature: &str,
    task: &ParsedTask,
    resume: Option<&ProviderSyncLedger>,
) -> SyncItemRecord {
    let lifecycle_state = if task.is_complete { "done" } else { "todo" };
    let previous = resume.and_then(|ledger| {
        ledger
            .entries
            .iter()
            .find(|entry| entry.item_key == task.id)
    });
    let previous_fingerprint = previous.and_then(|entry| {
        entry
            .metadata_fields
            .get("fingerprint")
            .map(ToString::to_string)
    });
    let remote_id = previous.and_then(|entry| {
        entry
            .metadata_fields
            .get("remote_id")
            .map(ToString::to_string)
            .filter(|value| !value.trim().is_empty())
    });
    SyncItemRecord {
        internal_key: task.id.clone(),
        remote_id,
        fingerprint: format!("{feature}:{}:{}", task.id, task.description),
        previous_fingerprint,
        integration_identity: IntegrationIdentity {
            owner_identity: feature.to_string(),
            assignee_identity: "unknown".to_string(),
            source_system_identity: "specify-mcp".to_string(),
            identity_version: 1,
        },
        source_of_truth: vec![SourceOfTruthMarker {
            field_name: "task".to_string(),
            authority: "local".to_string(),
            last_confirmed_at: chrono_like_timestamp(),
        }],
        lifecycle: LifecycleMetadata {
            lifecycle_state: lifecycle_state.to_string(),
            lifecycle_stage: "implementation".to_string(),
            entered_at: chrono_like_timestamp(),
            updated_at: chrono_like_timestamp(),
        },
        relationships: RelationshipMetadata {
            parent_id: Some(format!("{run_id}:parent")),
            child_ids: vec![],
            blocked_by_ids: vec![],
            depends_on_ids: vec![],
        },
        canonical_labels: vec![
            format!("lifecycle:{lifecycle_state}"),
            "rel:child".to_string(),
        ],
        linkage: None,
        conflict_reason: None,
    }
}

fn resolve_feature_artifacts(
    repo_root: &Path,
    feature: Option<&str>,
) -> Result<FeatureArtifacts, String> {
    if let Some(feature_slug) = feature {
        let dir = repo_root.join("specs").join(feature_slug);
        return Ok(FeatureArtifacts {
            slug: feature_slug.to_string(),
            spec_path: dir.join("spec.md"),
            tasks_path: dir.join("tasks.md"),
        });
    }
    let pre = run_command(
        repo_root,
        ".specify/scripts/bash/check-prerequisites.sh",
        ["--json", "--require-tasks", "--include-tasks"],
    )?;
    let parsed: Value = serde_json::from_str(&pre)
        .map_err(|e| format!("failed to parse prerequisite output as JSON: {e}"))?;
    let feature_dir = parsed
        .get("FEATURE_DIR")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing FEATURE_DIR in prerequisite output".to_string())?;
    let dir = PathBuf::from(feature_dir);
    let slug = dir
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| "unable to derive feature slug".to_string())?;
    Ok(FeatureArtifacts {
        slug: slug.to_string(),
        spec_path: dir.join("spec.md"),
        tasks_path: dir.join("tasks.md"),
    })
}

fn parse_scope(args: &Value) -> Scope {
    Scope {
        feature: args
            .get("feature")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        story: args
            .get("story")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        task_range: args
            .get("task_range")
            .and_then(Value::as_str)
            .map(ToString::to_string),
    }
}

fn parse_tasks(tasks_md: &str) -> Vec<ParsedTask> {
    let mut out = Vec::new();
    for line in tasks_md.lines() {
        let trimmed = line.trim();
        if !(trimmed.starts_with("- [ ]")
            || trimmed.starts_with("- [x]")
            || trimmed.starts_with("- [X]"))
        {
            continue;
        }
        let Some(id) = extract_task_id(trimmed) else {
            continue;
        };
        let story = extract_story(trimmed);
        let description = strip_prefix_and_markers(trimmed)
            .strip_prefix(&id)
            .unwrap_or(trimmed)
            .trim()
            .to_string();
        out.push(ParsedTask {
            id,
            description,
            story,
            is_complete: trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]"),
        });
    }
    out
}

fn apply_scope(tasks: Vec<ParsedTask>, scope: &Scope) -> Result<Vec<ParsedTask>, String> {
    let mut out = tasks;
    if let Some(story) = &scope.story {
        let normalized = story.to_ascii_uppercase();
        out.retain(|task| {
            task.story.as_deref().map(str::to_ascii_uppercase) == Some(normalized.clone())
        });
    }
    if let Some(range) = &scope.task_range {
        let (start, end) = parse_task_range(range)?;
        out.retain(|task| {
            task_numeric_id(&task.id)
                .map(|id| id >= start && id <= end)
                .unwrap_or(false)
        });
    }
    Ok(out)
}

fn parse_task_range(raw: &str) -> Result<(u32, u32), String> {
    let (left, right) = raw
        .split_once('-')
        .ok_or_else(|| "task_range must look like T001-T010".to_string())?;
    let start = task_numeric_id(left).ok_or_else(|| format!("invalid task id '{left}'"))?;
    let end = task_numeric_id(right).ok_or_else(|| format!("invalid task id '{right}'"))?;
    if start > end {
        return Err("task_range start must be <= end".to_string());
    }
    Ok((start, end))
}

fn task_numeric_id(raw: &str) -> Option<u32> {
    let token = raw
        .trim()
        .trim_matches(|c: char| !c.is_ascii_alphanumeric());
    let token = token
        .strip_prefix('T')
        .or_else(|| token.strip_prefix('t'))?;
    let digits: String = token.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.len() != 3 {
        return None;
    }
    digits.parse().ok()
}

fn extract_story(line: &str) -> Option<String> {
    line.split_whitespace().find_map(|token| {
        let token = token.trim_matches(|c: char| !c.is_ascii_alphanumeric());
        if token.starts_with("US") && token[2..].chars().all(|c| c.is_ascii_digit()) {
            Some(token.to_string())
        } else {
            None
        }
    })
}

fn extract_task_id(line: &str) -> Option<String> {
    line.split_whitespace().find_map(|token| {
        let token = token.trim_matches(|c: char| !c.is_ascii_alphanumeric());
        is_task_id_token(token).then(|| token.to_string())
    })
}

fn is_task_id_token(token: &str) -> bool {
    let mut chars = token.chars();
    match chars.next() {
        Some('T') | Some('t') => {}
        _ => return false,
    }
    let tail: String = chars.collect();
    let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.len() == 3
}

fn strip_prefix_and_markers(line: &str) -> String {
    let mut s = line
        .trim()
        .trim_start_matches("- [ ]")
        .trim_start_matches("- [x]")
        .trim_start_matches("- [X]")
        .trim()
        .to_string();
    while s.starts_with('[') {
        let Some(end) = s.find(']') else {
            break;
        };
        s = s[end + 1..].trim_start().to_string();
    }
    s
}

fn read_spec_title(spec_path: &Path) -> Option<String> {
    let raw = fs::read_to_string(spec_path).ok()?;
    raw.lines()
        .find_map(|line| line.trim().strip_prefix("# ").map(|v| v.trim().to_string()))
}

fn chrono_like_timestamp() -> String {
    // Keep dependency-free RFC3339-ish timestamp suitable for ledgers/tests.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{secs}Z")
}

fn run_command<const N: usize>(
    cwd: &Path,
    program: &str,
    args: [&str; N],
) -> Result<String, String> {
    let output = Command::new(program)
        .current_dir(cwd)
        .args(args)
        .output()
        .map_err(|e| format!("failed to execute `{program}`: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "`{program}` failed (exit {}): {}",
            output.status,
            stderr.trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::{apply_scope, parse_task_range, parse_tasks, to_sync_item, ParsedTask, Scope};
    use crate::orchestrator::ledger::ProviderSyncLedger;
    use specify_provider_core::{ProviderKind, ReconciliationOutcome, SyncLedgerEntry};
    use std::collections::BTreeMap;

    #[test]
    fn parses_and_filters_scope() {
        let input = "- [ ] T011 [US1] Alpha\n- [X] T022 [US2] Beta\n";
        let tasks = parse_tasks(input);
        assert_eq!(tasks.len(), 2);
        let scope = Scope {
            feature: None,
            story: Some("US2".to_string()),
            task_range: None,
        };
        let filtered = apply_scope(tasks, &scope).expect("scope");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "T022");
        assert!(filtered[0].is_complete);
    }

    #[test]
    fn parses_range() {
        assert_eq!(parse_task_range("T005-T010").expect("range"), (5, 10));
    }

    #[test]
    fn reuses_previous_fingerprint_from_ledger() {
        let mut metadata = BTreeMap::new();
        metadata.insert("fingerprint".to_string(), "old-fp".to_string());
        metadata.insert("remote_id".to_string(), "remote-123".to_string());
        let ledger = ProviderSyncLedger {
            run_id: "run-old".to_string(),
            provider: "local".to_string(),
            feature: "feat".to_string(),
            entries: vec![SyncLedgerEntry {
                run_id: "run-old".to_string(),
                provider_kind: ProviderKind::Local,
                item_key: "T011".to_string(),
                outcome: ReconciliationOutcome::Updated,
                drift_detected: false,
                conflict_reason: None,
                labels: vec!["lifecycle:todo".to_string(), "rel:child".to_string()],
                metadata_fields: metadata,
            }],
        };
        let task = ParsedTask {
            id: "T011".to_string(),
            description: "Sample".to_string(),
            story: Some("US1".to_string()),
            is_complete: false,
        };
        let item = to_sync_item("run-new", "feat", &task, Some(&ledger));
        assert_eq!(item.previous_fingerprint.as_deref(), Some("old-fp"));
        assert_eq!(item.remote_id.as_deref(), Some("remote-123"));
    }
}
