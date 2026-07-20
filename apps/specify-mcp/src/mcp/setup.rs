use std::fs;
use std::path::Path;
use std::process::Command;

pub fn ensure_runtime_paths(repo_root: &Path) -> Result<(), String> {
    for rel in [".specify", ".specify/metadata", "specs"] {
        let path = repo_root.join(rel);
        fs::create_dir_all(&path)
            .map_err(|e| format!("failed to create {}: {e}", path.display()))?;
    }
    Ok(())
}

pub fn initialize_repo(repo_root: &Path) -> Result<Option<String>, String> {
    ensure_runtime_paths(repo_root)?;
    autofill_github_repo_from_origin(repo_root)
}

pub fn autofill_github_repo_from_origin(repo_root: &Path) -> Result<Option<String>, String> {
    let config_path = repo_root.join("specify.toml");
    if !config_path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&config_path)
        .map_err(|e| format!("failed to read {}: {e}", config_path.display()))?;
    let mut parsed: toml::Value = toml::from_str(&raw)
        .map_err(|e| format!("failed to parse {}: {e}", config_path.display()))?;

    let current = current_repo_value(&parsed);
    if let Some(v) = current {
        if !v.trim().is_empty() && v.trim() != "owner/repo" {
            return Ok(None);
        }
    }

    let inferred = infer_repo_from_origin(repo_root)?;
    let Some(repo_slug) = inferred else {
        return Ok(None);
    };

    set_repo_value(&mut parsed, &repo_slug)?;
    let out = toml::to_string_pretty(&parsed)
        .map_err(|e| format!("failed to serialize updated TOML: {e}"))?;
    fs::write(&config_path, out)
        .map_err(|e| format!("failed to write {}: {e}", config_path.display()))?;
    Ok(Some(repo_slug))
}

fn current_repo_value(root: &toml::Value) -> Option<String> {
    root.get("providers")
        .and_then(|v| v.get("github"))
        .and_then(|v| v.get("repo"))
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
}

fn set_repo_value(root: &mut toml::Value, repo: &str) -> Result<(), String> {
    let providers = root
        .get_mut("providers")
        .and_then(|v| v.as_table_mut())
        .ok_or_else(|| "missing [providers] table in specify.toml".to_string())?;
    let github = providers
        .entry("github".to_string())
        .or_insert_with(|| toml::Value::Table(Default::default()));
    let github_table = github
        .as_table_mut()
        .ok_or_else(|| "invalid [providers.github] table in specify.toml".to_string())?;
    github_table.insert("repo".to_string(), toml::Value::String(repo.to_string()));
    Ok(())
}

fn infer_repo_from_origin(repo_root: &Path) -> Result<Option<String>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["remote", "get-url", "origin"])
        .output()
        .map_err(|e| format!("failed to execute git remote get-url origin: {e}"))?;
    if !out.status.success() {
        return Ok(None);
    }
    let remote = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(parse_github_repo_slug(&remote))
}

fn parse_github_repo_slug(remote: &str) -> Option<String> {
    let trimmed = remote.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut slug = if let Some(idx) = trimmed.find("github.com/") {
        trimmed[(idx + "github.com/".len())..].to_string()
    } else if let Some(rest) = trimmed.strip_prefix("git@github.com:") {
        rest.to_string()
    } else {
        return None;
    };

    if let Some(qidx) = slug.find('?') {
        slug.truncate(qidx);
    }
    slug = slug.trim_matches('/').to_string();
    if let Some(stripped) = slug.strip_suffix(".git") {
        slug = stripped.to_string();
    }

    let parts: Vec<&str> = slug.split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return None;
    }
    Some(format!("{}/{}", parts[0], parts[1]))
}

#[cfg(test)]
mod tests {
    use super::parse_github_repo_slug;

    #[test]
    fn parses_https_remote() {
        let r = parse_github_repo_slug("https://github.com/octo-org/speckit.git");
        assert_eq!(r.as_deref(), Some("octo-org/speckit"));
    }

    #[test]
    fn parses_ssh_remote() {
        let r = parse_github_repo_slug("git@github.com:octo-org/speckit.git");
        assert_eq!(r.as_deref(), Some("octo-org/speckit"));
    }
}
