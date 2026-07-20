use jsonschema::validator_for;
use serde_json::Value as JsonValue;

const SPECIFY_SCHEMA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../schemas/specify.schema.json"
));

pub fn validate_toml_with_schema(raw_toml: &str) -> Result<(), String> {
    let toml_value: toml::Value =
        toml::from_str(raw_toml).map_err(|e| format!("invalid TOML: {e}"))?;
    let json_value: JsonValue =
        serde_json::to_value(toml_value).map_err(|e| format!("toml->json failed: {e}"))?;

    let schema: JsonValue = serde_json::from_str(SPECIFY_SCHEMA)
        .map_err(|e| format!("invalid bundled schema JSON: {e}"))?;
    let compiled = validator_for(&schema).map_err(|e| format!("schema compile failed: {e}"))?;
    let errors = compiled.iter_errors(&json_value).collect::<Vec<_>>();
    if !errors.is_empty() {
        let joined = errors
            .into_iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(format!("specify.toml schema validation failed: {joined}"));
    }
    Ok(())
}
