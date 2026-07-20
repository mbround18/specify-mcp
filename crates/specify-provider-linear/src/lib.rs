use specify_provider_core::{
    Provider, ProviderCapabilities, ProviderError, ProviderHealth, ProviderKind, SpecRecord,
    TaskRecord,
};

pub struct LinearProvider {
    team_key: String,
    project_id: String,
    default_priority: u8,
}

impl LinearProvider {
    pub fn new(
        team_key: impl Into<String>,
        project_id: impl Into<String>,
        default_priority: u8,
    ) -> Self {
        Self {
            team_key: team_key.into(),
            project_id: project_id.into(),
            default_priority,
        }
    }
}

impl Provider for LinearProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Linear
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
            supports_project: true,
            supports_assignee: true,
            supports_reconciliation: true,
        }
    }

    fn map_canonical_labels(&self, labels: &[String]) -> Result<Vec<String>, ProviderError> {
        Ok(labels
            .iter()
            .map(|label| {
                if label.starts_with("lifecycle:") {
                    format!("state/{}", label.trim_start_matches("lifecycle:"))
                } else if label.starts_with("rel:") {
                    format!("link/{}", label.trim_start_matches("rel:"))
                } else {
                    label.clone()
                }
            })
            .collect())
    }

    fn health_check(&self) -> Result<ProviderHealth, ProviderError> {
        if self.team_key.is_empty() {
            return Err(ProviderError::Validation(
                "linear provider requires team_key".to_string(),
            ));
        }
        Ok(ProviderHealth {
            healthy: true,
            detail: format!(
                "linear team {} project {} default_priority {}",
                self.team_key, self.project_id, self.default_priority
            ),
        })
    }

    fn upsert_spec(&self, spec: &SpecRecord) -> Result<String, ProviderError> {
        Ok(format!("linear-spec-{}", spec.key))
    }

    fn upsert_tasks(
        &self,
        spec_key: &str,
        tasks: &[TaskRecord],
    ) -> Result<Vec<String>, ProviderError> {
        Ok(tasks
            .iter()
            .map(|t| format!("linear-task-{spec_key}-{}", t.key))
            .collect())
    }
}
