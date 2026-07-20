use std::fs;

#[test]
fn taskstoissues_respects_provider_override() {
    let root = std::env::temp_dir().join(format!(
        "specify-mcp-provider-select-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("specs/feat")).expect("mkdir");
    fs::write(
        root.join("specify.toml"),
        r#"[specify]
version = 1
mode = "local"

[providers.local]
enabled = true
spec_root = "specs"

[providers.linear]
enabled = true
team_key = "TEAM"
project_id = "PROJECT"
default_priority = 2
"#,
    )
    .expect("config");
    fs::write(root.join("specs/feat/spec.md"), "# Feature\n").expect("spec");
    fs::write(
        root.join("specs/feat/tasks.md"),
        "- [ ] T011 [US1] Sample task\n",
    )
    .expect("tasks");
    fs::create_dir_all(root.join(".specify/state/conformance")).expect("conformance dir");
    fs::write(
        root.join(".specify/state/conformance/linear.json"),
        r#"{"provider":"linear","status":"pass"}"#,
    )
    .expect("conformance status");

    let args = serde_json::json!({
        "feature": "feat",
        "provider": "linear",
        "dry_run": true
    });
    let out = specify_mcp::mcp::taskstoissues::run_taskstoissues(root.as_path(), &args)
        .expect("taskstoissues");
    assert_eq!(out["provider"], "linear");

    let _ = fs::remove_dir_all(root);
}
