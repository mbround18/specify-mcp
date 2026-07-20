use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RootConfig {
    pub specify: SpecifySection,
    #[serde(default)]
    pub providers: ProvidersSection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpecifySection {
    #[serde(default = "default_version")]
    pub version: u32,
    pub mode: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProvidersSection {
    #[serde(default)]
    pub local: LocalProviderConfig,
    #[serde(default)]
    pub github: GithubProviderConfig,
    #[serde(default)]
    pub linear: LinearProviderConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocalProviderConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_spec_root")]
    pub spec_root: String,
}

impl Default for LocalProviderConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            spec_root: default_spec_root(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GithubProviderConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub repo: String,
    #[serde(default = "default_github_adapter")]
    pub adapter: String, // auto | graphql | mcp
    #[serde(default = "default_github_adapter_preference")]
    pub adapter_preference: Vec<String>,
    #[serde(default = "default_github_labels")]
    pub issue_labels: Vec<String>,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub issue_sync: GithubIssueSyncConfig,
}

impl Default for GithubProviderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            repo: String::new(),
            adapter: default_github_adapter(),
            adapter_preference: default_github_adapter_preference(),
            issue_labels: default_github_labels(),
            dry_run: false,
            issue_sync: GithubIssueSyncConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GithubIssueSyncConfig {
    #[serde(default = "default_issue_topology")]
    pub topology: String, // parent-only | hybrid | all-children
    #[serde(default = "default_split_signals")]
    pub split_signals: Vec<String>,
    #[serde(default = "default_true")]
    pub close_orphaned_children: bool,
}

impl Default for GithubIssueSyncConfig {
    fn default() -> Self {
        Self {
            topology: default_issue_topology(),
            split_signals: default_split_signals(),
            close_orphaned_children: default_true(),
        }
    }
}

fn default_version() -> u32 {
    1
}

fn default_true() -> bool {
    true
}

fn default_spec_root() -> String {
    "specs".to_string()
}

fn default_github_adapter() -> String {
    "auto".to_string()
}

fn default_github_adapter_preference() -> Vec<String> {
    vec!["graphql".to_string(), "mcp".to_string()]
}

fn default_github_labels() -> Vec<String> {
    vec!["spec".to_string(), "task".to_string()]
}

fn default_issue_topology() -> String {
    "hybrid".to_string()
}

fn default_split_signals() -> Vec<String> {
    vec![
        "[split]".to_string(),
        "#split".to_string(),
        "[blocked]".to_string(),
        "[risk:high]".to_string(),
        "[risk:critical]".to_string(),
        "[size:l]".to_string(),
        "[size:xl]".to_string(),
    ]
}

#[derive(Debug, Clone, Deserialize)]
pub struct LinearProviderConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub team_key: String,
    #[serde(default)]
    pub project_id: String,
    #[serde(default = "default_linear_priority")]
    pub default_priority: u8,
}

fn default_linear_priority() -> u8 {
    2
}

impl Default for LinearProviderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            team_key: String::new(),
            project_id: String::new(),
            default_priority: default_linear_priority(),
        }
    }
}

impl RootConfig {
    pub fn parse_provider_mode(raw: &str) -> Result<&str, String> {
        match raw {
            "local" | "github" | "linear" => Ok(raw),
            _ => Err(format!(
                "invalid provider mode '{raw}', expected local|github|linear"
            )),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.specify.version != 1 {
            return Err(format!(
                "unsupported [specify].version '{}', expected 1",
                self.specify.version
            ));
        }

        Self::parse_provider_mode(self.specify.mode.as_str()).map_err(|_| {
            format!(
                "invalid [specify].mode '{}', expected one of: local, github, linear",
                self.specify.mode
            )
        })?;

        match self.providers.github.adapter.as_str() {
            "auto" | "graphql" | "mcp" => {}
            _ => {
                return Err(format!(
                    "invalid [providers.github].adapter '{}', expected auto|graphql|mcp",
                    self.providers.github.adapter
                ))
            }
        }

        match self.providers.github.issue_sync.topology.as_str() {
            "parent-only" | "hybrid" | "all-children" => {}
            _ => {
                return Err(format!(
                    "invalid [providers.github.issue_sync].topology '{}', expected parent-only|hybrid|all-children",
                    self.providers.github.issue_sync.topology
                ))
            }
        }

        if self.specify.mode == "github" {
            if !self.providers.github.enabled {
                return Err("mode=github requires [providers.github].enabled = true".to_string());
            }
            if self.providers.github.repo.trim().is_empty() {
                return Err(
                    "mode=github requires [providers.github].repo = \"owner/name\"".to_string(),
                );
            }
        }

        if self.specify.mode == "linear" && !self.providers.linear.enabled {
            return Err("mode=linear requires [providers.linear].enabled = true".to_string());
        }

        if self.providers.local.spec_root.trim().is_empty() {
            return Err("[providers.local].spec_root cannot be empty".to_string());
        }

        Ok(())
    }
}
