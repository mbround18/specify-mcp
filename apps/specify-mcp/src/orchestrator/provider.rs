use std::fs;
use std::path::Path;

use specify_provider_core::{
    Provider, ProviderCapabilities, ProviderError, ProviderHealth, ProviderKind, ProviderRegistry,
    SpecRecord, TaskRecord,
};
use specify_provider_github::{GithubAdapter, GithubProvider, GithubProviderConfig};
use specify_provider_linear::LinearProvider;

use crate::config::schema::RootConfig;

pub fn build_registry(config: &RootConfig) -> Result<ProviderRegistry, String> {
    let mut registry = ProviderRegistry::new();
    if config.providers.local.enabled || config.specify.mode == "local" {
        registry.register(LocalProvider {
            spec_root: config.providers.local.spec_root.clone(),
        });
    }

    if config.providers.github.enabled || config.specify.mode == "github" {
        let adapter = match config.providers.github.adapter.as_str() {
            "auto" => GithubAdapter::Auto,
            "graphql" => GithubAdapter::Graphql,
            "mcp" => GithubAdapter::Mcp,
            _ => {
                return Err(format!(
                    "invalid github adapter '{}'",
                    config.providers.github.adapter
                ))
            }
        };
        let github_cfg = GithubProviderConfig {
            repo: config.providers.github.repo.clone(),
            adapter,
            adapter_preference: config.providers.github.adapter_preference.clone(),
            issue_labels: config.providers.github.issue_labels.clone(),
            dry_run: config.providers.github.dry_run,
        };
        let github = GithubProvider::with_config(github_cfg)
            .map_err(|e| format!("failed to configure github provider: {e}"))?;
        registry.register(github);
    }

    if config.providers.linear.enabled || config.specify.mode == "linear" {
        registry.register(LinearProvider::new(
            config.providers.linear.team_key.clone(),
            config.providers.linear.project_id.clone(),
            config.providers.linear.default_priority,
        ));
    }
    Ok(registry)
}

pub fn select_provider<'a>(
    registry: &'a ProviderRegistry,
    mode: &str,
    repo_root: &Path,
) -> Result<std::sync::Arc<dyn Provider>, String> {
    let kind = match mode {
        "local" => ProviderKind::Local,
        "github" => ProviderKind::Github,
        "linear" => ProviderKind::Linear,
        _ => {
            return Err(format!(
                "unsupported mode '{mode}', expected local|github|linear"
            ))
        }
    };
    ensure_conformance_gate(repo_root, &kind)?;
    registry
        .activate(&kind)
        .map_err(|e| format!("provider activation failed: {e}"))
}

pub fn select_provider_with_override<'a>(
    registry: &'a ProviderRegistry,
    default_mode: &str,
    override_mode: Option<&str>,
    repo_root: &Path,
) -> Result<std::sync::Arc<dyn Provider>, String> {
    let mode = override_mode.unwrap_or(default_mode);
    select_provider(registry, mode, repo_root)
}

fn ensure_conformance_gate(repo_root: &Path, kind: &ProviderKind) -> Result<(), String> {
    if matches!(kind, ProviderKind::Local) {
        return Ok(());
    }
    if std::env::var("SPECIFY_SKIP_CONFORMANCE_GATE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return Ok(());
    }

    let provider = match kind {
        ProviderKind::Github => "github",
        ProviderKind::Linear => "linear",
        ProviderKind::Local => return Ok(()),
    };
    let path = repo_root
        .join(".specify")
        .join("state")
        .join("conformance")
        .join(format!("{provider}.json"));
    if !path.exists() {
        return Err(format!(
            "conformance gate failed for provider '{provider}': missing {}",
            path.display()
        ));
    }
    let raw = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read conformance status {}: {e}", path.display()))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("invalid conformance status JSON {}: {e}", path.display()))?;
    let status = parsed.get("status").and_then(|v| v.as_str()).unwrap_or("");
    let ok = parsed.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    if status.eq_ignore_ascii_case("pass") || ok {
        return Ok(());
    }
    Err(format!(
        "conformance gate failed for provider '{provider}' via {}: expected status=pass or ok=true",
        path.display()
    ))
}

pub struct LocalProvider {
    spec_root: String,
}

impl Provider for LocalProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Local
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_health_check: true,
            supports_status_sync: true,
            supports_dry_run: true,
            supports_labels: true,
            supports_relationships: true,
            supports_lifecycle: true,
            supports_milestone: false,
            supports_project: false,
            supports_assignee: true,
            supports_reconciliation: true,
        }
    }

    fn health_check(&self) -> Result<ProviderHealth, ProviderError> {
        Ok(ProviderHealth {
            healthy: true,
            detail: "local provider ready".to_string(),
        })
    }

    fn upsert_spec(&self, spec: &SpecRecord) -> Result<String, ProviderError> {
        let path = Path::new(&self.spec_root).join(&spec.key).join("spec.md");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| ProviderError::Internal(e.to_string()))?;
        }
        if !path.exists() {
            let content = format!("# {}\n", spec.title);
            fs::write(&path, content).map_err(|e| ProviderError::Internal(e.to_string()))?;
        }
        Ok(path.display().to_string())
    }

    fn upsert_tasks(
        &self,
        spec_key: &str,
        tasks: &[TaskRecord],
    ) -> Result<Vec<String>, ProviderError> {
        let path = Path::new(&self.spec_root).join(spec_key).join("tasks.md");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| ProviderError::Internal(e.to_string()))?;
        }
        if !path.exists() {
            fs::write(&path, "# Tasks\n").map_err(|e| ProviderError::Internal(e.to_string()))?;
        }
        Ok(tasks
            .iter()
            .map(|t| format!("local-task-{}", t.key))
            .collect())
    }
}
