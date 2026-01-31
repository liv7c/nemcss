use config::ResolvedToken;

/// A struct that contains generated CSS output for utilities and custom properties.
///
/// Use the `to_css` method to get the generated CSS as a string.
///
/// # Output format
///
/// The generated CSS will be in the following format:
///
/// ```css
/// :root {
///   --color-primary: yellow;
///   --color-secondary: #c1c1c1;
/// }
///
/// .text-primary {
///   color: var(--color-primary);
/// }
///
/// .text-secondary {
///   color: var(--color-secondary);
/// }
/// ```
pub struct GeneratedCss {
    /// A list of custom properties to generate.
    /// Each custom property is a CSS variable.
    pub custom_properties: Vec<String>,
    /// A list of utilities to generate.
    /// Each utility is a tuple containing the full class, the class name, and the class value.
    /// For example: (".text-primary {\n  color: var(--color-primary);\n}", "text-primary", "color: var(--color-primary)")
    pub utilities: Vec<(String, String, String)>,
}

const INDENT_AND_NEWLINE_PER_PROPERTY: usize = 3;
const ROOT_BLOCK_OVERHEAD: usize = 20;

impl GeneratedCss {
    pub fn new(custom_properties: Vec<String>, utilities: Vec<(String, String, String)>) -> Self {
        GeneratedCss {
            custom_properties,
            utilities,
        }
    }

    /// Combines the custom properties and utilities into a single CSS string.
    pub fn to_css(&self) -> String {
        // Estimate the capacity of the string to avoid reallocations.
        let estimated_capacity = self
            .custom_properties
            .iter()
            .map(|s| s.len())
            .sum::<usize>()
            + self.utilities.iter().map(|s| s.0.len()).sum::<usize>()
            + self.custom_properties.len() * INDENT_AND_NEWLINE_PER_PROPERTY
            + self.utilities.len()
            + ROOT_BLOCK_OVERHEAD;
        let mut css = String::with_capacity(estimated_capacity);
        css.push_str(":root {\n");

        for custom_property in &self.custom_properties {
            css.push_str("  "); // 2-space indentation
            css.push_str(custom_property);
            css.push('\n');
        }
        css.push_str("}\n\n");

        for utility in &self.utilities {
            css.push_str(&utility.0);
            css.push('\n');
        }

        css
    }
}

/// Generates CSS custom properties and utilities from resolved design tokens.
///
/// # Arguments
///
/// * `resolved_tokens` - Any collection or iterator that yields `&ResolvedToken`
///
/// # Returns
///
/// A `GeneratedCss` struct containing custom properties and utility classes.
pub fn generate_css<'a>(
    resolved_tokens: impl IntoIterator<Item = &'a ResolvedToken>,
) -> GeneratedCss {
    let tokens: Vec<_> = resolved_tokens.into_iter().collect();
    let custom_properties = generate_custom_properties(&tokens);
    let utilities = generate_utilities(&tokens);

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
pub fn generate_utilities(resolved_tokens: &[&ResolvedToken]) -> Vec<(String, String, String)> {
    let estimated_capacity = resolved_tokens.iter().fold(0, |acc, curr| {
        acc + curr.tokens.len() * curr.utilities.len()
    });

    let mut utilities = Vec::with_capacity(estimated_capacity);

    for resolved_token in resolved_tokens.iter().filter(|r| r.prefix != "viewport") {
        for utility in resolved_token.utilities.iter() {
            for (token_name, _token_value) in resolved_token.tokens.iter() {
                let custom_property_name =
                    format!("var(--{}-{})", resolved_token.prefix, token_name);
                let utility_class_value = format!("{}: {}", utility.property, custom_property_name);
                let utility_class_name = format!("{}-{}", utility.prefix, token_name);
                let utility_full_class =
                    format!(".{} {{\n  {};\n}}", utility_class_name, utility_class_value);
                let tuple = (utility_full_class, utility_class_name, custom_property_name);
                utilities.push(tuple);
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
            result[0].0,
            ".text-primary {\n  color: var(--color-primary);\n}"
        );
        assert_eq!(
            result[1].0,
            ".text-secondary {\n  color: var(--color-secondary);\n}"
        );
        assert_eq!(
            result[2].0,
            ".bg-primary {\n  background-color: var(--color-primary);\n}"
        );
        assert_eq!(
            result[3].0,
            ".bg-secondary {\n  background-color: var(--color-secondary);\n}"
        );
    }

    #[test]
    fn test_ignore_viewports_when_generating_utilities() {
        let mut resolved_tokens = HashMap::new();
        resolved_tokens.insert(
            "border-radii".to_string(),
            ResolvedToken {
                tokens: vec![
                    ("xs".to_string(), TokenValue::Simple("2px".to_string())),
                    ("sm".to_string(), TokenValue::Simple("4px".to_string())),
                ],
                utilities: vec![TokenUtilityConfig {
                    prefix: "rounded".to_string(),
                    property: "border-radius".to_string(),
                }],
                prefix: "radius".to_string(),
            },
        );
        resolved_tokens.insert(
            "viewports".to_string(),
            ResolvedToken {
                tokens: vec![
                    ("xs".to_string(), TokenValue::Simple("320px".to_string())),
                    ("md".to_string(), TokenValue::Simple("768px".to_string())),
                    ("lg".to_string(), TokenValue::Simple("1024px".to_string())),
                ],
                utilities: vec![],
                prefix: "viewport".to_string(),
            },
        );
        let resolved_tokens: Vec<_> = resolved_tokens.values().collect();
        let result = generate_utilities(&resolved_tokens);

        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].0,
            ".rounded-xs {\n  border-radius: var(--radius-xs);\n}"
        );
        assert_eq!(
            result[1].0,
            ".rounded-sm {\n  border-radius: var(--radius-sm);\n}"
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
        let css_to_generate = generate_css(resolved_tokens.values(), None);

        let result = css_to_generate.to_css();
        let expected_root_css =
            ":root {\n  --color-primary: yellow;\n  --color-secondary: #c1c1c1;\n}\n\n";
        let expected_utilities_css = ".text-primary {\n  color: var(--color-primary);\n}\n.text-secondary {\n  color: var(--color-secondary);\n}\n";

        assert!(
            result.contains(expected_root_css),
            "expected: {}, got {result}",
            expected_root_css
        );
        assert!(result.contains(expected_utilities_css));
    }
}
