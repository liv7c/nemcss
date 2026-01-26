use std::path::PathBuf;

use config::TokenValue;
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

    let color_token = tokens.get("colors").unwrap();
    assert_eq!(color_token.prefix, "color");

    assert!(color_token.tokens.contains_key("dark"));
    assert_eq!(
        color_token.tokens.get("dark"),
        Some(&TokenValue::Simple("#171406".to_string()))
    );

    assert!(color_token.tokens.contains_key("light"));
    assert_eq!(
        color_token.tokens.get("light"),
        Some(&TokenValue::Simple("#ffffff".to_string()))
    );

    assert!(color_token.tokens.contains_key("primary"));
    assert_eq!(
        color_token.tokens.get("primary"),
        Some(&TokenValue::Simple("#fccd26".to_string()))
    );
    assert_eq!(color_token.tokens.len(), 3);

    let font_token = tokens.get("fonts").unwrap();
    assert_eq!(font_token.prefix, "font");

    assert!(font_token.tokens.contains_key("base"));
    assert_eq!(
        font_token.tokens.get("base"),
        Some(&TokenValue::List(vec![
            "\"Satoshi\"".to_string(),
            "\"Inter\"".to_string(),
        ]))
    );

    assert!(font_token.tokens.contains_key("mono"));
    assert_eq!(
        font_token.tokens.get("mono"),
        Some(&TokenValue::List(vec![
            "\"DM Mono\"".to_string(),
            "\"monospace\"".to_string()
        ]))
    );
}

#[test]
fn test_generates_utilities_for_explicitly_configured_tokens() {
    let config_path = get_config_fixture_path("explicit_tokens_with_custom_utils");
    let config = NemCSSConfig::from_path(&config_path).unwrap();
    let tokens = config.resolve_all_tokens().unwrap();
    dbg!(&tokens);

    let spacing_token = tokens.get("allSpacings").unwrap();
    assert_eq!(spacing_token.prefix, "spacing");

    assert!(spacing_token.tokens.contains_key("xxs"));
    assert_eq!(
        spacing_token.tokens.get("xxs"),
        Some(&TokenValue::Simple("0.125rem".to_string()))
    );

    assert!(spacing_token.tokens.contains_key("xs"));
    assert_eq!(
        spacing_token.tokens.get("xs"),
        Some(&TokenValue::Simple("0.25rem".to_string()))
    );

    assert_eq!(spacing_token.tokens.len(), 2);

    let color_token = tokens.get("allColors").unwrap();
    assert_eq!(color_token.prefix, "color");

    assert!(color_token.tokens.contains_key("dark"));
    assert_eq!(
        color_token.tokens.get("dark"),
        Some(&TokenValue::Simple("#171406".to_string()))
    );

    let spacing_utilities = spacing_token.utilities.clone();
    assert_eq!(spacing_utilities.len(), 2);
    assert!(spacing_utilities.first().unwrap().prefix == "p");
    assert!(spacing_utilities.last().unwrap().prefix == "m");

    let color_utilities = color_token.utilities.clone();
    assert_eq!(color_utilities.len(), 3);
    assert!(color_utilities.first().unwrap().prefix == "text");
    assert!(color_utilities.get(1).unwrap().prefix == "bg");
    assert!(color_utilities.last().unwrap().prefix == "highlight");
}

#[test]
fn test_overrides_default_configuration_for_explicitly_configured_tokens() {
    let config_path = get_config_fixture_path("default_and_overrides_combination");
    let config = NemCSSConfig::from_path(&config_path).unwrap();
    let tokens = config.resolve_all_tokens().unwrap();

    let spacing_token = tokens.get("spacings").unwrap();
    assert_eq!(spacing_token.prefix, "spacing");

    // check tokens for spacing
    assert!(spacing_token.tokens.contains_key("xxs"));
    assert_eq!(
        spacing_token.tokens.get("xxs"),
        Some(&TokenValue::Simple("0.125rem".to_string()))
    );

    assert!(spacing_token.tokens.contains_key("xs"));
    assert_eq!(
        spacing_token.tokens.get("xs"),
        Some(&TokenValue::Simple("0.25rem".to_string()))
    );

    let spacing_utilities = spacing_token.utilities.clone();
    assert!(spacing_utilities.len() > 2);
    assert!(spacing_utilities.iter().any(|u| u.prefix == "pb"));
    assert!(
        spacing_utilities
            .iter()
            .find(|u| u.prefix == "pb")
            .unwrap()
            .property
            == "padding-block"
    );

    let color_token = tokens.get("colors").unwrap();
    let color_utilities = color_token.utilities.clone();
    assert!(color_utilities.len() > 2);
    assert!(color_utilities.iter().any(|u| u.prefix == "highlight"));
    assert!(
        color_utilities
            .iter()
            .find(|u| u.prefix == "highlight")
            .unwrap()
            .property
            == "background-color"
    );
}
