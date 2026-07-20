use std::fs;
use std::path::{Path, PathBuf};

use super::schema::RootConfig;
use super::validation::validate_toml_with_schema;

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub source_path: PathBuf,
    pub config: RootConfig,
}

pub fn discover_config(repo_root: &Path) -> Result<ResolvedConfig, String> {
    let mut candidates = vec![
        repo_root.join("specify.toml"),
        repo_root.join(".github/config/specify.toml"),
        repo_root.join(".github/specify.toml"),
    ];

    let mut github_fallback = github_specify_fallbacks(repo_root)?;
    candidates.append(&mut github_fallback);

    for path in candidates {
        if !path.exists() {
            continue;
        }

        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read config {}: {e}", path.display()))?;
        validate_toml_with_schema(&raw).map_err(|e| format!("{e} ({})", path.display()))?;
        let parsed: RootConfig = toml::from_str(&raw)
            .map_err(|e| format!("failed to parse config {}: {e}", path.display()))?;
        parsed
            .validate()
            .map_err(|e| format!("invalid config {}: {e}", path.display()))?;
        return Ok(ResolvedConfig {
            source_path: path,
            config: parsed,
        });
    }

    Err("no config found (checked specify.toml and .github paths)".to_string())
}

pub fn resolve_mode_with_override(
    config: &RootConfig,
    provider_override: Option<&str>,
) -> Result<String, String> {
    if let Some(mode) = provider_override {
        RootConfig::parse_provider_mode(mode)?;
        return Ok(mode.to_string());
    }
    Ok(config.specify.mode.clone())
}

fn github_specify_fallbacks(repo_root: &Path) -> Result<Vec<PathBuf>, String> {
    let github_dir = repo_root.join(".github");
    if !github_dir.exists() {
        return Ok(Vec::new());
    }

    let mut matches = Vec::new();
    let entries = fs::read_dir(&github_dir)
        .map_err(|e| format!("failed reading {}: {e}", github_dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("failed reading .github entry: {e}"))?;
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(v) => v,
            None => continue,
        };
        if path.is_file() && name.starts_with("specify") && name.ends_with(".toml") {
            if name != "specify.toml" {
                matches.push(path);
            }
        }
    }
    matches.sort();
    Ok(matches)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::discover_config;

    fn write(path: &PathBuf, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn prefers_root_specify_toml() {
        let base = std::env::temp_dir().join(format!(
            "specify-mcp-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        write(
            &base.join("specify.toml"),
            "[specify]\nversion = 1\nmode = \"local\"\n[providers.github]\nadapter_preference = [\"graphql\"]\n",
        );
        write(
            &base.join(".github/config/specify.toml"),
            "[specify]\nversion = 1\nmode = \"github\"\n",
        );

        let resolved = discover_config(&base).unwrap();
        assert!(resolved.source_path.ends_with("specify.toml"));
        assert_eq!(resolved.config.specify.mode, "local");
        let _ = fs::remove_dir_all(base);
    }
}
