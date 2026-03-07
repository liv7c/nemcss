use crate::{ThemeConfig, config::TokenUtilityConfig};

/// Generates the default prefix for a token type.
/// It applies a simple singularization for the most common token types.
/// Other token types are returned as is.
pub fn default_prefix_for_token_type(token_type: &str) -> String {
    match token_type {
        "colors" => "color",
        "spacings" => "spacing",
        "fonts" => "font",
        "shadows" => "shadow",
        "borders" => "border",
        "radii" => "radius",
        "viewports" => "viewport",
        other => other,
    }
    .to_string()
}

/// Returns the utilities for a given token type.
/// It combines the default utilities with the user-defined ones.
pub fn get_utilities_for_token_type(
    name: &str,
    theme: Option<&ThemeConfig>,
) -> Vec<TokenUtilityConfig> {
    theme
        .and_then(|t| t.tokens.get(name))
        .and_then(|t| t.utilities.clone())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use crate::TokenConfig;

    use super::*;

    #[test]
    fn test_default_prefix_for_token_type() {
        assert_eq!(default_prefix_for_token_type("colors"), "color");
        assert_eq!(default_prefix_for_token_type("fonts"), "font");
        assert_eq!(default_prefix_for_token_type("shadows"), "shadow");
        assert_eq!(default_prefix_for_token_type("borders"), "border");
        assert_eq!(default_prefix_for_token_type("radii"), "radius");
        assert_eq!(default_prefix_for_token_type("viewports"), "viewport");
        assert_eq!(default_prefix_for_token_type("spacing"), "spacing");

        assert_eq!(
            default_prefix_for_token_type("font-weights"),
            "font-weights"
        );
    }

    #[test]
    fn test_returns_empty_when_no_theme() {
        let result = get_utilities_for_token_type("colors", None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_returns_empty_when_no_utilities_configured() {
        let mock_theme = ThemeConfig {
            tokens: HashMap::from([(
                "colors".to_string(),
                TokenConfig {
                    source: PathBuf::from("colors.json"),
                    prefix: None,
                    utilities: None,
                },
            )]),
        };

        let result = get_utilities_for_token_type("colors", Some(&mock_theme));
        assert!(result.is_empty());
    }

    #[test]
    fn test_returns_only_explicitly_configured_utilities() {
        let mock_theme = ThemeConfig {
            tokens: HashMap::from([(
                "spacings".to_string(),
                TokenConfig {
                    source: PathBuf::from("design-tokens/spacings.json"),
                    prefix: Some("spacing".to_string()),
                    utilities: Some(vec![
                        TokenUtilityConfig {
                            prefix: "p".to_string(),
                            property: "padding".to_string(),
                        },
                        TokenUtilityConfig {
                            prefix: "px".to_string(),
                            property: "padding-inline".to_string(),
                        },
                    ]),
                },
            )]),
        };
        let result = get_utilities_for_token_type("spacings", Some(&mock_theme));
        let expected = vec![
            TokenUtilityConfig {
                prefix: "p".to_string(),
                property: "padding".to_string(),
            },
            TokenUtilityConfig {
                prefix: "px".to_string(),
                property: "padding-inline".to_string(),
            },
        ];
        assert_eq!(result, expected);
    }
}
