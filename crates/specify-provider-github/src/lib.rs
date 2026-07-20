use std::collections::HashMap;
use std::process::Command;

use reqwest::blocking::Client;
use serde_json::Value as JsonValue;
use specify_provider_core::{
    Provider, ProviderCapabilities, ProviderError, ProviderHealth, ProviderKind, SpecRecord,
    TaskRecord,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GithubAdapter {
    Auto,
    Graphql,
    Mcp,
}

#[derive(Debug, Clone)]
pub struct GithubProviderConfig {
    pub repo: String,
    pub adapter: GithubAdapter,
    pub adapter_preference: Vec<String>,
    pub issue_labels: Vec<String>,
    pub dry_run: bool,
}

pub struct GithubProvider {
    config: GithubProviderConfig,
    adapter: GithubAdapter,
    token: Option<String>,
    client: Client,
}

impl GithubProvider {
    pub fn with_config(config: GithubProviderConfig) -> Result<Self, ProviderError> {
        if config.repo.trim().is_empty() {
            return Err(ProviderError::Validation(
                "github repo must be set as owner/name".to_string(),
            ));
        }

        let token = resolve_github_token()?;
        let adapter = match config.adapter {
            GithubAdapter::Auto => {
                select_auto_adapter(&config.adapter_preference, token.is_some())?
            }
            GithubAdapter::Graphql => {
                if token.is_none() {
                    return Err(ProviderError::Auth(
                        "github adapter=graphql requires GITHUB_TOKEN or `gh auth token`"
                            .to_string(),
                    ));
                }
                GithubAdapter::Graphql
            }
            GithubAdapter::Mcp => {
                if !is_mcp_available() {
                    return Err(ProviderError::Auth(
                        "github mcp adapter selected but unavailable".to_string(),
                    ));
                }
                GithubAdapter::Mcp
            }
        };

        Ok(Self {
            config,
            adapter,
            token,
            client: Client::new(),
        })
    }
}

impl Provider for GithubProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Github
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_health_check: true,
            supports_status_sync: true,
            supports_dry_run: true,
            supports_labels: true,
            supports_relationships: true,
            supports_lifecycle: true,
            supports_milestone: true,
            supports_project: true,
            supports_assignee: true,
            supports_reconciliation: true,
        }
    }

    fn map_canonical_labels(&self, labels: &[String]) -> Result<Vec<String>, ProviderError> {
        Ok(labels
            .iter()
            .map(|label| match label.as_str() {
                "rel:blocked-by" => "blocked-by".to_string(),
                "rel:depends-on" => "depends-on".to_string(),
                _ => label.replace(':', "/"),
            })
            .collect())
    }

    fn health_check(&self) -> Result<ProviderHealth, ProviderError> {
        match self.adapter {
            GithubAdapter::Graphql => {
                self.ensure_graphql_ready()?;
                Ok(ProviderHealth {
                    healthy: true,
                    detail: format!("github adapter=graphql repo={}", self.config.repo),
                })
            }
            GithubAdapter::Mcp => {
                if is_mcp_available() {
                    Ok(ProviderHealth {
                        healthy: true,
                        detail: format!("github adapter=mcp repo={}", self.config.repo),
                    })
                } else {
                    Err(ProviderError::Auth(
                        "github mcp adapter unavailable".to_string(),
                    ))
                }
            }
            GithubAdapter::Auto => Err(ProviderError::Internal(
                "auto adapter should be resolved at construction".to_string(),
            )),
        }
    }

    fn upsert_spec(&self, spec: &SpecRecord) -> Result<String, ProviderError> {
        match self.adapter {
            GithubAdapter::Graphql => self.upsert_spec_with_graphql(spec),
            GithubAdapter::Mcp => Err(ProviderError::Unsupported(
                "github mcp mutations are not wired yet; use adapter='graphql' for real mutations"
                    .to_string(),
            )),
            GithubAdapter::Auto => Err(ProviderError::Internal(
                "auto adapter should be resolved at construction".to_string(),
            )),
        }
    }

    fn upsert_tasks(
        &self,
        spec_key: &str,
        tasks: &[TaskRecord],
    ) -> Result<Vec<String>, ProviderError> {
        match self.adapter {
            GithubAdapter::Graphql => self.upsert_tasks_with_graphql(spec_key, tasks),
            GithubAdapter::Mcp => Err(ProviderError::Unsupported(
                "github mcp mutations are not wired yet; use adapter='graphql' for real mutations"
                    .to_string(),
            )),
            GithubAdapter::Auto => Err(ProviderError::Internal(
                "auto adapter should be resolved at construction".to_string(),
            )),
        }
    }
}

impl GithubProvider {
    fn ensure_graphql_ready(&self) -> Result<(), ProviderError> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| ProviderError::Auth("missing github token".to_string()))?;
        let response = self.graphql(token, "query { viewer { login } }", serde_json::json!({}))?;
        if response.get("errors").is_some() {
            return Err(ProviderError::Auth(format!(
                "github graphql auth failed: {}",
                response["errors"]
            )));
        }
        Ok(())
    }

    fn upsert_spec_with_graphql(&self, spec: &SpecRecord) -> Result<String, ProviderError> {
        if self.config.dry_run {
            return Ok(format!("dry-run://{}#{}", self.config.repo, spec.key));
        }
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| ProviderError::Auth("missing github token".to_string()))?;
        let (owner, name) = parse_repo(&self.config.repo)?;
        let marker = format!("specify:key:{}", spec.key);
        let title = format!("[spec] {}", spec.title);
        let body = format!(
            "Spec record for `{}`\n\nstatus: {}\n\n<!-- {} -->",
            spec.key, spec.status, marker
        );

        let repo_ctx = self.fetch_repo_context(token, owner, name)?;
        if let Some(issue) = self.search_issue_by_marker(token, owner, name, &marker)? {
            self.update_issue(token, &issue.id, &title, &body)?;
            return Ok(issue.url);
        }

        self.create_issue(
            token,
            &repo_ctx.repository_id,
            &title,
            &body,
            &repo_ctx.label_ids,
        )
    }

    fn upsert_tasks_with_graphql(
        &self,
        spec_key: &str,
        tasks: &[TaskRecord],
    ) -> Result<Vec<String>, ProviderError> {
        if self.config.dry_run {
            return Ok(tasks
                .iter()
                .map(|t| format!("dry-run://{}#{}-{}", self.config.repo, spec_key, t.key))
                .collect());
        }
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| ProviderError::Auth("missing github token".to_string()))?;
        let (owner, name) = parse_repo(&self.config.repo)?;
        let repo_ctx = self.fetch_repo_context(token, owner, name)?;
        let mut ids = Vec::with_capacity(tasks.len());
        for t in tasks {
            let marker = format!("specify:task:{}:{}:{}", spec_key, t.key, t.status);
            let title = format!("[task:{}] {}", spec_key, t.title);
            let body = format!(
                "Task for spec `{}`\n\npriority: {}\nstatus: {}\n\n<!-- {} -->",
                spec_key, t.priority, t.status, marker
            );
            if let Some(issue) = self.search_issue_by_marker(token, owner, name, &marker)? {
                self.update_issue(token, &issue.id, &title, &body)?;
                ids.push(issue.url);
            } else {
                let created = self.create_issue(
                    token,
                    &repo_ctx.repository_id,
                    &title,
                    &body,
                    &repo_ctx.label_ids,
                )?;
                ids.push(created);
            }
        }
        Ok(ids)
    }

    fn fetch_repo_context(
        &self,
        token: &str,
        owner: &str,
        name: &str,
    ) -> Result<RepoContext, ProviderError> {
        let response = self.graphql(
            token,
            r#"
            query($owner: String!, $name: String!) {
              repository(owner: $owner, name: $name) {
                id
                labels(first: 100) {
                  nodes { id name }
                }
              }
            }
            "#,
            serde_json::json!({ "owner": owner, "name": name }),
        )?;
        ensure_no_graphql_errors(&response)?;
        let repo = response
            .get("data")
            .and_then(|d| d.get("repository"))
            .ok_or_else(|| ProviderError::Validation("repository not found".to_string()))?;
        let repository_id = repo
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProviderError::Transport("missing repository id".to_string()))?
            .to_string();

        let mut label_map = HashMap::new();
        if let Some(nodes) = repo
            .get("labels")
            .and_then(|l| l.get("nodes"))
            .and_then(|n| n.as_array())
        {
            for n in nodes {
                if let (Some(name), Some(id)) = (
                    n.get("name").and_then(|v| v.as_str()),
                    n.get("id").and_then(|v| v.as_str()),
                ) {
                    label_map.insert(name.to_string(), id.to_string());
                }
            }
        }

        let label_ids = self
            .config
            .issue_labels
            .iter()
            .filter_map(|l| label_map.get(l).cloned())
            .collect::<Vec<_>>();

        Ok(RepoContext {
            repository_id,
            label_ids,
        })
    }

    fn search_issue_by_marker(
        &self,
        token: &str,
        owner: &str,
        name: &str,
        marker: &str,
    ) -> Result<Option<FoundIssue>, ProviderError> {
        let query = format!("repo:{owner}/{name} is:issue in:body \"{marker}\"");
        let response = self.graphql(
            token,
            r#"
            query($query: String!) {
              search(query: $query, type: ISSUE, first: 1) {
                nodes {
                  ... on Issue {
                    id
                    number
                    url
                  }
                }
              }
            }
            "#,
            serde_json::json!({ "query": query }),
        )?;
        ensure_no_graphql_errors(&response)?;
        let maybe = response
            .get("data")
            .and_then(|d| d.get("search"))
            .and_then(|s| s.get("nodes"))
            .and_then(|n| n.as_array())
            .and_then(|arr| arr.first());
        if let Some(node) = maybe {
            let id = node
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ProviderError::Transport("search result missing id".to_string()))?
                .to_string();
            let url = node
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ProviderError::Transport("search result missing url".to_string()))?
                .to_string();
            return Ok(Some(FoundIssue { id, url }));
        }
        Ok(None)
    }

    fn create_issue(
        &self,
        token: &str,
        repository_id: &str,
        title: &str,
        body: &str,
        label_ids: &[String],
    ) -> Result<String, ProviderError> {
        let response = self.graphql(
            token,
            r#"
            mutation($repositoryId: ID!, $title: String!, $body: String!, $labelIds: [ID!]) {
              createIssue(input: {
                repositoryId: $repositoryId,
                title: $title,
                body: $body,
                labelIds: $labelIds
              }) {
                issue { url }
              }
            }
            "#,
            serde_json::json!({
                "repositoryId": repository_id,
                "title": title,
                "body": body,
                "labelIds": label_ids
            }),
        )?;
        ensure_no_graphql_errors(&response)?;
        let url = response
            .get("data")
            .and_then(|d| d.get("createIssue"))
            .and_then(|ci| ci.get("issue"))
            .and_then(|i| i.get("url"))
            .and_then(|u| u.as_str())
            .ok_or_else(|| ProviderError::Transport("createIssue missing issue url".to_string()))?;
        Ok(url.to_string())
    }

    fn update_issue(
        &self,
        token: &str,
        issue_id: &str,
        title: &str,
        body: &str,
    ) -> Result<(), ProviderError> {
        let response = self.graphql(
            token,
            r#"
            mutation($id: ID!, $title: String!, $body: String!) {
              updateIssue(input: {id: $id, title: $title, body: $body}) {
                issue { id }
              }
            }
            "#,
            serde_json::json!({
                "id": issue_id,
                "title": title,
                "body": body
            }),
        )?;
        ensure_no_graphql_errors(&response)?;
        Ok(())
    }

    fn graphql(
        &self,
        token: &str,
        query: &str,
        variables: JsonValue,
    ) -> Result<JsonValue, ProviderError> {
        let response = self
            .client
            .post("https://api.github.com/graphql")
            .bearer_auth(token)
            .header("User-Agent", "specify-mcp")
            .json(&serde_json::json!({
                "query": query,
                "variables": variables
            }))
            .send()
            .map_err(|e| ProviderError::Transport(format!("github graphql request failed: {e}")))?;

        let status = response.status();
        let payload: JsonValue = response.json().map_err(|e| {
            ProviderError::Transport(format!("failed to decode github graphql response: {e}"))
        })?;
        if !status.is_success() {
            return Err(ProviderError::Transport(format!(
                "github graphql http {}: {}",
                status, payload
            )));
        }
        Ok(payload)
    }
}

fn select_auto_adapter(
    preference: &[String],
    has_token: bool,
) -> Result<GithubAdapter, ProviderError> {
    for p in preference {
        match p.as_str() {
            "graphql" if has_token => return Ok(GithubAdapter::Graphql),
            "mcp" if is_mcp_available() => return Ok(GithubAdapter::Mcp),
            _ => {}
        }
    }
    if has_token {
        return Ok(GithubAdapter::Graphql);
    }
    if is_mcp_available() {
        return Ok(GithubAdapter::Mcp);
    }
    Err(ProviderError::Auth(
        "github provider unavailable: no token for graphql and mcp unavailable".to_string(),
    ))
}

fn is_mcp_available() -> bool {
    std::env::var("SPECIFY_GITHUB_MCP_AVAILABLE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn resolve_github_token() -> Result<Option<String>, ProviderError> {
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        let trimmed = token.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(Some(trimmed));
        }
    }
    let out = Command::new("gh").args(["auth", "token"]).output();
    match out {
        Ok(result) if result.status.success() => {
            let token = String::from_utf8_lossy(&result.stdout).trim().to_string();
            if token.is_empty() {
                Ok(None)
            } else {
                Ok(Some(token))
            }
        }
        Ok(_) => Ok(None),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(ProviderError::Transport(format!(
            "failed to resolve token via gh auth token: {e}"
        ))),
    }
}

fn parse_repo(repo: &str) -> Result<(&str, &str), ProviderError> {
    let mut split = repo.split('/');
    let owner = split
        .next()
        .ok_or_else(|| ProviderError::Validation("repo must be owner/name".to_string()))?;
    let name = split
        .next()
        .ok_or_else(|| ProviderError::Validation("repo must be owner/name".to_string()))?;
    if owner.is_empty() || name.is_empty() || split.next().is_some() {
        return Err(ProviderError::Validation(
            "repo must be owner/name".to_string(),
        ));
    }
    Ok((owner, name))
}

fn ensure_no_graphql_errors(response: &JsonValue) -> Result<(), ProviderError> {
    if let Some(errors) = response.get("errors") {
        return Err(ProviderError::Transport(format!(
            "github graphql errors: {errors}"
        )));
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct RepoContext {
    repository_id: String,
    label_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct FoundIssue {
    id: String,
    url: String,
}
