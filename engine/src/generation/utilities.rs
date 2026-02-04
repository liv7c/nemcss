//! Utilities generation logic for NemCSS.
//! This module contains functions to generate utility classes based on the design tokens and the nemcss configuration.

use config::{ResolvedToken, TokenUtilityConfig};

/// The prefix for viewport tokens.
/// This prefix is used to identify viewport tokens in the resolved tokens and generate
/// responsive utility classes.
///
/// **Note**: This is primarily used for internal purposes. External users typically do not need to know about this prefix.
pub const VIEWPORT_TOKEN_PREFIX: &str = "viewport";

/// A struct that represents a utility class.
///
/// It contains the full CSS class definition and its constituent parts (class name and value) for
/// flexible composition.
///
/// # Example
///
/// ```no_run
/// use engine::Utility;
///
/// let utility = Utility::new(
///     ".text-primary {\n  color: var(--color-primary);\n}",
///     "text-primary",
///     "color: var(--color-primary)",
/// );
///
/// assert_eq!(utility.full_class(), ".text-primary {\n  color: var(--color-primary);\n}");
/// assert_eq!(utility.class_name(), "text-primary");
/// assert_eq!(utility.class_value(), "color: var(--color-primary)");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Utility {
    /// The complete CSS class definition (e.g., ".text-primary {\n  color: var(--color-primary);\n}")
    pub full_class: String,
    /// The class name without the leading dot (e.g., "text-primary")
    pub class_name: String,
    /// The class property and value (e.g., "color: var(--color-primary)")
    pub class_value: String,
}

impl Utility {
    /// Creates a new `Utility` instance.
    ///
    /// # Arguments
    ///
    /// * `full_class` - The complete CSS class definition (e.g., ".text-primary {\n  color: var(--color-primary);\n}")
    /// * `class_name` - The class name without the leading dot (e.g., "text-primary")
    /// * `class_value` - The class property and value (e.g., "color: var(--color-primary)")
    pub fn new(full_class: &str, class_name: &str, class_value: &str) -> Self {
        Utility {
            full_class: full_class.to_string(),
            class_name: class_name.to_string(),
            class_value: class_value.to_string(),
        }
    }

    /// Returns the full CSS class definition.
    pub fn full_class(&self) -> &str {
        &self.full_class
    }

    /// Returns the class name without the leading dot.
    pub fn class_name(&self) -> &str {
        &self.class_name
    }

    /// Returns the class property and value.
    pub fn class_value(&self) -> &str {
        &self.class_value
    }
}

/// Generate utility classes based on the design tokens and utilities defined
/// in resolved tokens.
///
/// Utility classes are generated for each token and utility combination.
/// For example, with a token `colors` with a primary color `primary` and a utility `text-primary`,
/// this generates:
///
/// ```css
/// .text-primary {
///   color: var(--color-primary);
/// }
/// ```
///
/// # Arguments
///
/// * `resolved_tokens` - A slice of `ResolvedToken` instances representing the tokens and utilities to generate utility classes for.
///
/// # Returns
///
/// A vector of `Utility` instances representing the generated utility classes.
/// Each utility contains the full CSS class definition and its constituent parts (class name and value).
pub fn generate_utilities(resolved_tokens: &[&ResolvedToken]) -> Vec<Utility> {
    let estimated_capacity = resolved_tokens
        .iter()
        .map(|token| token.tokens.len() * token.utilities.len())
        .sum();

    let mut utilities = Vec::with_capacity(estimated_capacity);

    for resolved_token in resolved_tokens
        .iter()
        .filter(|r| r.prefix != VIEWPORT_TOKEN_PREFIX)
    {
        for utility in resolved_token.utilities.iter() {
            for (token_name, _token_value) in resolved_token.tokens.iter() {
                utilities.push(create_utility(resolved_token, utility, token_name));
            }
        }
    }

    utilities
}

/// Create a single utility class from a token and utility configuration.
pub fn create_utility(
    resolved_token: &ResolvedToken,
    utility: &TokenUtilityConfig,
    token_name: &str,
) -> Utility {
    let custom_property_name = format!("var(--{}-{})", resolved_token.prefix, token_name);
    let utility_class_value = format!("{}: {}", utility.property, custom_property_name);
    let utility_class_name = format!("{}-{}", utility.prefix, token_name);
    let utility_full_class = format!(".{} {{\n  {};\n}}", utility_class_name, utility_class_value);
    Utility::new(
        &utility_full_class,
        &utility_class_name,
        &utility_class_value,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use config::{TokenUtilityConfig, TokenValue};

    use super::*;

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
            result[0].full_class(),
            ".text-primary {\n  color: var(--color-primary);\n}"
        );
        assert_eq!(
            result[1].full_class(),
            ".text-secondary {\n  color: var(--color-secondary);\n}"
        );
        assert_eq!(
            result[2].full_class(),
            ".bg-primary {\n  background-color: var(--color-primary);\n}"
        );
        assert_eq!(
            result[3].full_class(),
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
                prefix: VIEWPORT_TOKEN_PREFIX.to_string(),
            },
        );
        let resolved_tokens: Vec<_> = resolved_tokens.values().collect();
        let result = generate_utilities(&resolved_tokens);

        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].full_class(),
            ".rounded-xs {\n  border-radius: var(--radius-xs);\n}"
        );
        assert_eq!(
            result[1].full_class(),
            ".rounded-sm {\n  border-radius: var(--radius-sm);\n}"
        );
    }
}
