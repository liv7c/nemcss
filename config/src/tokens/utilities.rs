use crate::{ThemeConfig, config::TokenUtilityConfig};

/// Generates the default prefix for a token type.
/// It applies a simple singularization for the most common token types.
/// Other token types are returned as is.
pub fn default_prefix_for_token_type(token_type: &str) -> String {
    match token_type {
        "colors" => "color",
        "fonts" => "font",
        "shadows" => "shadow",
        "borders" => "border",
        "radii" => "radius",
        "viewports" => "viewport",
        other => other,
    }
    .to_string()
}

/// Returns the default utilities for a given token type.
pub fn get_default_utilities_for_token_type(token_type: &str) -> Vec<TokenUtilityConfig> {
    match token_type {
        "colors" => vec![
            TokenUtilityConfig {
                prefix: "text".to_string(),
                property: "color".to_string(),
            },
            TokenUtilityConfig {
                prefix: "bg".to_string(),
                property: "background-color".to_string(),
            },
        ],
        "spacing" => vec![
            TokenUtilityConfig {
                prefix: "p".to_string(),
                property: "padding".to_string(),
            },
            TokenUtilityConfig {
                prefix: "pt".to_string(),
                property: "padding-top".to_string(),
            },
            TokenUtilityConfig {
                prefix: "pr".to_string(),
                property: "padding-right".to_string(),
            },
            TokenUtilityConfig {
                prefix: "pb".to_string(),
                property: "padding-bottom".to_string(),
            },
            TokenUtilityConfig {
                prefix: "pl".to_string(),
                property: "padding-left".to_string(),
            },
            TokenUtilityConfig {
                prefix: "px".to_string(),
                property: "padding-inline".to_string(),
            },
            TokenUtilityConfig {
                prefix: "py".to_string(),
                property: "padding-block".to_string(),
            },
            TokenUtilityConfig {
                prefix: "m".to_string(),
                property: "margin".to_string(),
            },
            TokenUtilityConfig {
                prefix: "mt".to_string(),
                property: "margin-top".to_string(),
            },
            TokenUtilityConfig {
                prefix: "mr".to_string(),
                property: "margin-right".to_string(),
            },
            TokenUtilityConfig {
                prefix: "mb".to_string(),
                property: "margin-bottom".to_string(),
            },
            TokenUtilityConfig {
                prefix: "ml".to_string(),
                property: "margin-left".to_string(),
            },
            TokenUtilityConfig {
                prefix: "mx".to_string(),
                property: "margin-inline".to_string(),
            },
            TokenUtilityConfig {
                prefix: "my".to_string(),
                property: "margin-block".to_string(),
            },
        ],
        "fonts" => vec![TokenUtilityConfig {
            prefix: "font".to_string(),
            property: "font-family".to_string(),
        }],
        "font-sizes" => vec![TokenUtilityConfig {
            prefix: "text".to_string(),
            property: "font-size".to_string(),
        }],
        "font-weights" => vec![TokenUtilityConfig {
            prefix: "font".to_string(),
            property: "font-weight".to_string(),
        }],
        "shadows" => vec![TokenUtilityConfig {
            prefix: "shadow".to_string(),
            property: "box-shadow".to_string(),
        }],
        "borders" => vec![TokenUtilityConfig {
            prefix: "border".to_string(),
            property: "border".to_string(),
        }],
        "radii" => vec![TokenUtilityConfig {
            prefix: "rounded".to_string(),
            property: "border-radius".to_string(),
        }],
        "viewports" => vec![],
        _ => vec![],
    }
}

/// Returns the utilities for a given token type.
/// It combines the default utilities with the user-defined ones.
pub fn get_utilities_for_token_type(
    name: &str,
    theme: Option<&ThemeConfig>,
) -> Vec<TokenUtilityConfig> {
    let mut combined_utilities = get_default_utilities_for_token_type(name);
    let user_defined_utilities = theme
        .and_then(|t| t.tokens.get(name))
        .and_then(|t| t.utilities.clone())
        .unwrap_or_default();

    for user_utility in user_defined_utilities {
        if let Some(existing) = combined_utilities
            .iter_mut()
            .find(|u| u.prefix == user_utility.prefix)
        {
            *existing = user_utility;
        } else {
            combined_utilities.push(user_utility);
        }
    }

    combined_utilities
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

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
    fn test_get_utilities_for_token_type_combines_default_utilities_with_user_defined_utilities() {
        let mock_theme = ThemeConfig {
            tokens: HashMap::from([(
                "colors".to_string(),
                TokenConfig {
                    source: None,
                    prefix: None,
                    utilities: Some(vec![
                        TokenUtilityConfig {
                            prefix: "highlight".to_string(),
                            property: "background-color".to_string(),
                        },
                        TokenUtilityConfig {
                            prefix: "bg".to_string(),
                            property: "background-color".to_string(),
                        },
                    ]),
                },
            )]),
        };

        let resolved_tokens = get_utilities_for_token_type("colors", Some(&mock_theme));

        let expected = vec![
            TokenUtilityConfig {
                prefix: "text".to_string(),
                property: "color".to_string(),
            },
            TokenUtilityConfig {
                prefix: "bg".to_string(),
                property: "background-color".to_string(),
            },
            TokenUtilityConfig {
                prefix: "highlight".to_string(),
                property: "background-color".to_string(),
            },
        ];
        assert_eq!(resolved_tokens, expected);
    }

    #[test]
    fn test_get_utilities_for_token_type_returns_default_utilities_if_no_user_defined_utilities() {
        let resolved_tokens = get_utilities_for_token_type("colors", None);

        let expected = vec![
            TokenUtilityConfig {
                prefix: "text".to_string(),
                property: "color".to_string(),
            },
            TokenUtilityConfig {
                prefix: "bg".to_string(),
                property: "background-color".to_string(),
            },
        ];
        assert_eq!(resolved_tokens, expected);
    }

    #[test]
    fn test_get_utilities_for_token_type_returns_user_defined_utilities_if_no_default_utilities() {
        let mock_theme = ThemeConfig {
            tokens: HashMap::from([(
                "weights".to_string(),
                TokenConfig {
                    source: None,
                    prefix: Some("w".to_string()),
                    utilities: Some(vec![
                        TokenUtilityConfig {
                            prefix: "thin".to_string(),
                            property: "font-weight".to_string(),
                        },
                        TokenUtilityConfig {
                            prefix: "bold".to_string(),
                            property: "font-weight".to_string(),
                        },
                    ]),
                },
            )]),
        };
        let resolved_tokens = get_utilities_for_token_type("weights", Some(&mock_theme));
        let expected = vec![
            TokenUtilityConfig {
                prefix: "thin".to_string(),
                property: "font-weight".to_string(),
            },
            TokenUtilityConfig {
                prefix: "bold".to_string(),
                property: "font-weight".to_string(),
            },
        ];
        assert_eq!(resolved_tokens, expected);
    }
}
