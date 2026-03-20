use miette::Diagnostic;
use thiserror::Error;

use crate::NemCssConfig;

#[derive(Debug, Error, Diagnostic)]
pub enum GenerateSchemaError {
    #[error("failed to generate JSON for nemcss config: {0}")]
    #[diagnostic(code(nemcss::config::generate_schema))]
    GenerateSchemaString(#[from] serde_json::Error),
}

/// Generates a JSON schema for `NemCssConfig` as a pretty-printed JSON string.
///
/// The schema is derived directly from the Rust type definitions.
pub fn generate_schema() -> Result<String, GenerateSchemaError> {
    let schema = schemars::schema_for!(NemCssConfig);
    let result = serde_json::to_string_pretty(&schema)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_schema_produces_valid_json() {
        let output = generate_schema().expect("failed to generate schema");

        let parsed: serde_json::Value =
            serde_json::from_str(&output).expect("failed to parse schema");

        assert_eq!(parsed["type"], "object");

        let props = &parsed["properties"];
        assert!(
            props["theme"].is_object(),
            "schema should have a `theme` property"
        );
        assert!(
            props["content"].is_object(),
            "schema should have a `content` property"
        );
        assert!(
            props["tokensDir"].is_object(),
            "schema should have a `tokensDir` property"
        );
        assert!(
            props["semantic"].is_object(),
            "schema should have a `semantic` property"
        );
        assert!(
            props["$schema"].is_null(),
            "schema should not have a `$schema` property"
        );
    }

    #[test]
    fn generate_schema_is_stable() {
        let output1 = generate_schema().expect("failed to generate schema");
        let output2 = generate_schema().expect("failed to generate schema");

        assert_eq!(output1, output2);
    }
}
