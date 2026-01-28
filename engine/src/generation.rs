use config::ResolvedToken;

/// A struct that contains the generated CSS for utilities and custom properties.
pub struct GeneratedCss {
    /// A list of custom properties to generate.
    /// Each custom property is a CSS variable.
    pub custom_properties: Vec<String>,
    /// A list of utilities to generate.
    /// Each utility is a class that contains a property and a value.
    pub utilities: Vec<String>,
}

impl GeneratedCss {
    pub fn new(custom_properties: Vec<String>, utilities: Vec<String>) -> Self {
        GeneratedCss {
            custom_properties,
            utilities,
        }
    }

    /// Combines the custom properties and utilities into a single CSS string.
    pub fn to_css(&self) -> String {
        let mut css = String::new();
        css.push_str("\n:root {\n");
        css.push_str(&self.custom_properties.join("\n"));
        css.push_str("\n}\n\n");
        css.push_str(&self.utilities.join("\n"));
        css.push('\n');
        css
    }
}

/// Returns all the utilities and custom utilities derived from both the
/// design tokens and utilities defined in resolved tokens.
pub fn generate_css(resolved_tokens: &[&ResolvedToken]) -> GeneratedCss {
    let custom_properties = generate_custom_properties(resolved_tokens);
    let utilities = generate_utilities(resolved_tokens);

    GeneratedCss::new(custom_properties, utilities)
}

/// Generate CSS custom properties from resolved tokens.
pub fn generate_custom_properties(resolved_tokens: &[&ResolvedToken]) -> Vec<String> {
    let estimated_capacity = resolved_tokens
        .iter()
        .fold(0, |acc, curr| acc + curr.tokens.len());
    let mut custom_properties: Vec<_> = Vec::with_capacity(estimated_capacity);

    for resolved_token in resolved_tokens {
        for (token_name, token_value) in resolved_token.tokens.iter() {
            let custom_property_name = format!("--{}-{}", resolved_token.prefix, token_name);
            let custom_property_value = match token_value {
                config::TokenValue::Simple(v) => v.to_string(),
                config::TokenValue::List(items) => items.join(", "),
            };

            custom_properties.push(format!(
                "{}: {};",
                custom_property_name, custom_property_value
            ));
        }
    }

    custom_properties
}

/// Generate utility classes based on the design tokens and utilities defined
/// in resolved tokens.
pub fn generate_utilities(resolved_tokens: &[&ResolvedToken]) -> Vec<String> {
    let estimated_capacity = resolved_tokens.iter().fold(0, |acc, curr| {
        acc + curr.tokens.len() * curr.utilities.len()
    });

    let mut utilities = Vec::with_capacity(estimated_capacity);

    for resolved_token in resolved_tokens {
        for utility in resolved_token.utilities.iter() {
            for (token_name, _token_value) in resolved_token.tokens.iter() {
                let custom_property_name =
                    format!("var(--{}-{})", resolved_token.prefix, token_name);
                let utility_class_name = format!("{}-{}", utility.prefix, token_name);
                utilities.push(format!(
                    ".{} {{\n  {}: {};\n}}",
                    utility_class_name, utility.property, custom_property_name
                ));
            }
        }
    }

    utilities
}

#[cfg(test)]
mod tests {
    use config::{TokenUtilityConfig, TokenValue};
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_generate_custom_properties() {
        let mut resolved_tokens = HashMap::new();
        resolved_tokens.insert(
            "colors".to_string(),
            ResolvedToken {
                tokens: vec![
                    (
                        "primary".to_string(),
                        TokenValue::Simple("yellow".to_string()),
                    ),
                    (
                        "secondary".to_string(),
                        TokenValue::Simple("#c1c1c1".to_string()),
                    ),
                ],
                utilities: vec![
                    TokenUtilityConfig {
                        prefix: "text".to_string(),
                        property: "color".to_string(),
                    },
                    TokenUtilityConfig {
                        prefix: "bg".to_string(),
                        property: "background-color".to_string(),
                    },
                ],
                prefix: "color".to_string(),
            },
        );
        let resolved_tokens: Vec<_> = resolved_tokens.values().collect();
        let result = generate_custom_properties(&resolved_tokens);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "--color-primary: yellow;");
        assert_eq!(result[1], "--color-secondary: #c1c1c1;");
    }

    #[test]
    fn test_generate_custom_properties_returns_empty_vec_when_no_tokens() {
        let mut resolved_tokens = HashMap::new();
        resolved_tokens.insert(
            "colors".to_string(),
            ResolvedToken {
                tokens: vec![],
                utilities: vec![],
                prefix: "color".to_string(),
            },
        );

        let resolved_tokens: Vec<_> = resolved_tokens.values().collect();
        let result = generate_custom_properties(&resolved_tokens);
        assert!(result.is_empty());
    }

    #[test]
    fn test_generate_utilities() {
        let mut resolved_tokens = HashMap::new();
        resolved_tokens.insert(
            "colors".to_string(),
            ResolvedToken {
                tokens: vec![
                    (
                        "primary".to_string(),
                        TokenValue::Simple("yellow".to_string()),
                    ),
                    (
                        "secondary".to_string(),
                        TokenValue::Simple("#c1c1c1".to_string()),
                    ),
                ],
                utilities: vec![
                    TokenUtilityConfig {
                        prefix: "text".to_string(),
                        property: "color".to_string(),
                    },
                    TokenUtilityConfig {
                        prefix: "bg".to_string(),
                        property: "background-color".to_string(),
                    },
                ],
                prefix: "color".to_string(),
            },
        );
        let resolved_tokens: Vec<_> = resolved_tokens.values().collect();
        let result = generate_utilities(&resolved_tokens);

        assert_eq!(result.len(), 4);
        assert_eq!(
            result[0],
            ".text-primary {\n  color: var(--color-primary);\n}"
        );
        assert_eq!(
            result[1],
            ".text-secondary {\n  color: var(--color-secondary);\n}"
        );
        assert_eq!(
            result[2],
            ".bg-primary {\n  background-color: var(--color-primary);\n}"
        );
        assert_eq!(
            result[3],
            ".bg-secondary {\n  background-color: var(--color-secondary);\n}"
        );
    }

    #[test]
    fn test_to_css() {
        let mut resolved_tokens = HashMap::new();
        resolved_tokens.insert(
            "colors".to_string(),
            ResolvedToken {
                tokens: vec![
                    (
                        "primary".to_string(),
                        TokenValue::Simple("yellow".to_string()),
                    ),
                    (
                        "secondary".to_string(),
                        TokenValue::Simple("#c1c1c1".to_string()),
                    ),
                ],
                utilities: vec![TokenUtilityConfig {
                    prefix: "text".to_string(),
                    property: "color".to_string(),
                }],
                prefix: "color".to_string(),
            },
        );
        let resolved_tokens: Vec<_> = resolved_tokens.values().collect();
        let css_to_generate = generate_css(&resolved_tokens);

        let result = css_to_generate.to_css();
        let expected_root_css =
            ":root {\n--color-primary: yellow;\n--color-secondary: #c1c1c1;\n}\n\n";
        let expected_utilities_css = ".text-primary {\n  color: var(--color-primary);\n}\n.text-secondary {\n  color: var(--color-secondary);\n}\n";

        assert!(result.contains(expected_root_css));
        assert!(result.contains(expected_utilities_css));
    }
}
