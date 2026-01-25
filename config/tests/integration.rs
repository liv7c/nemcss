use std::path::PathBuf;

use config::{CONFIG_FILE_NAME, NemCSSConfig};

// Helper function to get the path to a fixture file.
// All fixtures should be located in the "fixtures" directory.
fn get_config_fixture_path(fixture_name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.join("tests")
        .join("fixtures")
        .join(fixture_name)
        .join(CONFIG_FILE_NAME)
}

#[test]
fn test_loads_config_from_path() {
    let config_path = get_config_fixture_path("basic");

    let loaded_config = NemCSSConfig::from_path(&config_path).unwrap();

    assert_eq!(
        loaded_config.base_dir,
        PathBuf::from(config_path.parent().unwrap())
    );
    assert_eq!(
        loaded_config.content,
        vec!["src/**/*.html".to_string(), "src/**/*.svelte".to_string()]
    );
    assert_eq!(loaded_config.tokens_dir, PathBuf::from("design-tokens"));
}

#[test]
fn test_returns_error_on_missing_config_file() {
    let config_path = PathBuf::from("missing_config_file.json");

    let loaded_config = NemCSSConfig::from_path(&config_path);
    assert!(loaded_config.is_err());

    let error = loaded_config.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("failed to read NemCSS config file")
    );
}

#[test]
fn test_returns_error_on_invalid_json() {
    let config_path = get_config_fixture_path("error_invalid_json");

    let loaded_config = NemCSSConfig::from_path(&config_path);
    assert!(loaded_config.is_err());

    let error = loaded_config.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("failed to parse NemCSS config file")
    );
}

#[test]
fn test_resolves_tokens_automatically_detected() {
    let config_path = get_config_fixture_path("autodetection_tokens");
    let config = NemCSSConfig::from_path(&config_path).unwrap();

    let tokens = config.resolve_all_tokens().unwrap();

    assert_eq!(tokens.len(), 2);
    dbg!(tokens);
}
