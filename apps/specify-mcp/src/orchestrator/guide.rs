use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::ledger::{load_ledger, write_ledger, StageEntry, StageStateLedger, StageStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    S0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
}

impl Stage {
    pub fn as_str(self) -> &'static str {
        match self {
            Stage::S0 => "S0",
            Stage::S1 => "S1",
            Stage::S2 => "S2",
            Stage::S3 => "S3",
            Stage::S4 => "S4",
            Stage::S5 => "S5",
            Stage::S6 => "S6",
            Stage::S7 => "S7",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GuidedResponse {
    pub feature: String,
    pub current_stage: String,
    pub completed_action: String,
    pub next_best_step: String,
    pub blockers: Vec<String>,
}

pub fn run_guided(repo_root: &Path, intent: &str) -> Result<GuidedResponse, String> {
    let feature_dir = detect_feature_dir(repo_root)?;
    let feature_name = feature_dir
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or_else(|| "invalid feature path".to_string())?
        .to_string();
    let stage = infer_stage_with_intent(&feature_dir, intent);
    let next = next_best_step(stage);

    let ledger_path = repo_root.join(".specify/metadata/stage-state.json");
    let mut ledger = load_ledger(&ledger_path).unwrap_or(StageStateLedger {
        feature: feature_name.clone(),
        current_stage: stage.as_str().to_string(),
        stages: BTreeMap::new(),
        next_best_step: next.to_string(),
        blockers: Vec::new(),
    });
    ledger.feature = feature_name.clone();
    ledger.current_stage = stage.as_str().to_string();
    ledger.next_best_step = next.to_string();
    ledger.stages.insert(
        stage.as_str().to_string(),
        StageEntry {
            status: StageStatus::InProgress,
            updated_at: now_unix().to_string(),
            blockers: Vec::new(),
        },
    );
    write_ledger(&ledger_path, &ledger)?;

    Ok(GuidedResponse {
        feature: feature_name,
        current_stage: stage.as_str().to_string(),
        completed_action: format!("resolved intent '{intent}' to stage {}", stage.as_str()),
        next_best_step: next.to_string(),
        blockers: Vec::new(),
    })
}

fn detect_feature_dir(repo_root: &Path) -> Result<PathBuf, String> {
    let feature_json = repo_root.join(".specify/feature.json");
    if feature_json.exists() {
        let raw = fs::read_to_string(&feature_json)
            .map_err(|e| format!("failed to read {}: {e}", feature_json.display()))?;
        let v: serde_json::Value = serde_json::from_str(&raw)
            .map_err(|e| format!("failed to parse {}: {e}", feature_json.display()))?;
        if let Some(rel) = v.get("feature_directory").and_then(|x| x.as_str()) {
            return Ok(repo_root.join(rel));
        }
    }
    Err("could not resolve active feature; run create-new-feature first".to_string())
}

pub fn infer_stage(feature_dir: &Path) -> Stage {
    let spec = feature_dir.join("spec.md");
    let plan = feature_dir.join("plan.md");
    let tasks = feature_dir.join("tasks.md");

    if !spec.exists() {
        return Stage::S2;
    }
    if !plan.exists() {
        return Stage::S3;
    }
    if !tasks.exists() {
        return Stage::S4;
    }
    Stage::S5
}

pub fn infer_stage_with_intent(feature_dir: &Path, intent: &str) -> Stage {
    let lowered = intent.to_ascii_lowercase();
    if lowered.contains("setup") || lowered.contains("init") {
        return Stage::S0;
    }
    if lowered.contains("clarify") || lowered.contains("discovery") {
        return Stage::S1;
    }
    if lowered.contains("converge") || lowered.contains("gap") {
        return Stage::S6;
    }
    if lowered.contains("rollout") || lowered.contains("handoff") {
        return Stage::S7;
    }
    infer_stage(feature_dir)
}

fn next_best_step(stage: Stage) -> &'static str {
    match stage {
        Stage::S0 => "Run setup and resolve workspace prerequisites",
        Stage::S1 => "Capture clarifications and convert to accepted scope",
        Stage::S2 => "Write a complete feature spec with user stories and requirements",
        Stage::S3 => "Generate implementation plan and architecture decisions",
        Stage::S4 => "Generate dependency-aware tasks from plan",
        Stage::S5 => "Implement tasks in priority order and update stage ledger",
        Stage::S6 => "Run converge pass and append missing tasks",
        Stage::S7 => "Finalize rollout notes and handoff package",
    }
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{infer_stage, Stage};

    #[test]
    fn infers_tasking_when_plan_missing() {
        let base = std::env::temp_dir().join(format!(
            "specify-stage-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(base.join("spec.md"), "ok").unwrap();
        assert_eq!(infer_stage(&base), Stage::S3);
        let _ = std::fs::remove_dir_all(base);
    }
}
