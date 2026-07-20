use serde::{Deserialize, Serialize};
use specify_provider_core::SyncLedgerEntry;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    Pending,
    InProgress,
    Done,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageEntry {
    pub status: StageStatus,
    pub updated_at: String,
    #[serde(default)]
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageStateLedger {
    pub feature: String,
    pub current_stage: String,
    pub stages: BTreeMap<String, StageEntry>,
    pub next_best_step: String,
    #[serde(default)]
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSyncLedger {
    pub run_id: String,
    pub provider: String,
    pub feature: String,
    pub entries: Vec<SyncLedgerEntry>,
}

pub fn load_ledger(path: &Path) -> Option<StageStateLedger> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn write_ledger(path: &Path, ledger: &StageStateLedger) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }
    let out = serde_json::to_string_pretty(ledger)
        .map_err(|e| format!("failed to serialize stage ledger: {e}"))?;
    fs::write(path, out).map_err(|e| format!("failed writing {}: {e}", path.display()))
}

pub fn write_provider_sync_ledger(path: &Path, ledger: &ProviderSyncLedger) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }
    let out = serde_json::to_string_pretty(ledger)
        .map_err(|e| format!("failed to serialize provider sync ledger: {e}"))?;
    fs::write(path, out).map_err(|e| format!("failed writing {}: {e}", path.display()))
}

pub fn load_latest_provider_sync_ledger(
    dir: &Path,
    feature: &str,
    provider: &str,
) -> Result<Option<ProviderSyncLedger>, String> {
    if !dir.exists() {
        return Ok(None);
    }

    let mut candidates: Vec<(std::time::SystemTime, std::path::PathBuf)> = Vec::new();
    let entries =
        fs::read_dir(dir).map_err(|e| format!("failed to read {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("failed reading ledger directory entry: {e}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.starts_with(feature) || !name.ends_with(".json") {
            continue;
        }
        let modified = entry
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        candidates.push((modified, path));
    }

    candidates.sort_by_key(|(modified, _)| *modified);
    candidates.reverse();

    for (_, path) in candidates {
        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("failed reading {}: {e}", path.display()))?;
        let parsed: ProviderSyncLedger = serde_json::from_str(&raw)
            .map_err(|e| format!("failed parsing {}: {e}", path.display()))?;
        if parsed.feature == feature && parsed.provider.eq_ignore_ascii_case(provider) {
            return Ok(Some(parsed));
        }
    }
    Ok(None)
}
