//! The `generation` module is responsible for generating CSS custom properties and utility classes based on resolved design tokens.
mod custom_properties;
mod filters;
mod responsive;
mod utilities;

pub use responsive::{
    GenerateResponsiveUtilitiesError, ResponsiveUtility, generate_all_responsive_utilities,
};
pub use utilities::{Utility, VIEWPORT_TOKEN_PREFIX};

use config::ResolvedToken;
use std::collections::HashSet;

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
#[derive(Debug, PartialEq)]
pub struct GeneratedCss {
    /// A list of custom properties to generate.
    /// Each custom property is a CSS variable.
    pub custom_properties: Vec<String>,
    /// A list of utilities to generate.
    /// Each utility contains the full CSS class definition and its constituent parts (class name and value).
    /// For example: (".text-primary {\n  color: var(--color-primary);\n}", "text-primary", "color: var(--color-primary)")
    pub utilities: Vec<Utility>,

    /// A list of media queries and their corresponding utility classes.
    /// Each media query is a string containing the media query and the utility classes.
    pub responsive_utilities: Vec<String>,
}

const INDENT_AND_NEWLINE_PER_PROPERTY: usize = 3;
const ROOT_BLOCK_OVERHEAD: usize = 20;

impl GeneratedCss {
    pub fn new(
        custom_properties: Vec<String>,
        utilities: Vec<Utility>,
        responsive_utilities: Vec<String>,
    ) -> Self {
        GeneratedCss {
            custom_properties,
            utilities,
            responsive_utilities,
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
            + self
                .utilities
                .iter()
                .map(|s| s.full_class.len())
                .sum::<usize>()
            + self.custom_properties.len() * INDENT_AND_NEWLINE_PER_PROPERTY
            + self.utilities.len()
            + self.responsive_utilities.len()
            + self
                .responsive_utilities
                .iter()
                .map(|s| s.len())
                .sum::<usize>()
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
            css.push_str(&utility.full_class);
            css.push('\n');
        }

        for responsive_utility in &self.responsive_utilities {
            css.push_str(responsive_utility);
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
    viewports: Option<&ResolvedToken>,
    used_classes: Option<&HashSet<String>>,
) -> GeneratedCss {
    let tokens: Vec<_> = resolved_tokens.into_iter().collect();
    let custom_properties = custom_properties::generate_custom_properties(&tokens);

    match used_classes {
        Some(used_classes) => {
            if used_classes.is_empty() {
                return GeneratedCss::new(custom_properties, vec![], vec![]);
            }

            let (used_utility_classes, used_responsive_utilities) =
                filters::parse_used_classes(used_classes);
            let (utilities, used_responsive_utilities_map) = filters::generate_filtered_utilities(
                &tokens,
                &used_utility_classes,
                &used_responsive_utilities,
            );
            let responsive_utilities = filters::generate_filtered_responsive_utilities(
                &used_responsive_utilities_map,
                viewports,
            );

            GeneratedCss::new(custom_properties, utilities, responsive_utilities)
        }
        None => {
            let utilities = utilities::generate_utilities(&tokens);
            let responsive_utilities =
                responsive::generate_responsive_utilities(&utilities, viewports);
            GeneratedCss::new(custom_properties, utilities, responsive_utilities)
        }
    }
}

#[cfg(test)]
mod tests {
    use config::{TokenUtilityConfig, TokenValue};
    use std::collections::HashMap;

    use super::*;

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

        let css_to_generate = generate_css(resolved_tokens.values(), None, None);

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

    #[test]
    fn test_to_css_with_responsive_utilities() {
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
        resolved_tokens.insert(
            "viewports".to_string(),
            ResolvedToken {
                tokens: vec![
                    ("sm".to_string(), TokenValue::Simple("320px".to_string())),
                    ("md".to_string(), TokenValue::Simple("768px".to_string())),
                ],
                utilities: vec![],
                prefix: VIEWPORT_TOKEN_PREFIX.to_string(),
            },
        );

        let css_to_generate = generate_css(
            resolved_tokens.values(),
            resolved_tokens.get("viewports"),
            None,
        );

        let result = css_to_generate.to_css();

        // Check for individual custom properties (order is not guaranteed due to HashMap)
        assert!(result.contains("--color-primary: yellow;"));
        assert!(result.contains("--color-secondary: #c1c1c1;"));
        assert!(result.contains("--viewport-sm: 320px;"));
        assert!(result.contains("--viewport-md: 768px;"));

        let expected_utilities_css = ".text-primary {\n  color: var(--color-primary);\n}\n.text-secondary {\n  color: var(--color-secondary);\n}\n";
        let expected_responsive_utilities_sm = "@media (min-width: 320px) {\n.sm\\:text-primary {\n  color: var(--color-primary);\n}\n.sm\\:text-secondary {\n  color: var(--color-secondary);\n}\n}";
        let expected_responsive_utilities_md = "@media (min-width: 768px) {\n.md\\:text-primary {\n  color: var(--color-primary);\n}\n.md\\:text-secondary {\n  color: var(--color-secondary);\n}\n}";

        assert!(result.contains(expected_utilities_css));
        assert!(
            result.contains(expected_responsive_utilities_sm),
            "expected sm media query, got {result}",
        );
        assert!(
            result.contains(expected_responsive_utilities_md),
            "expected md media query, got {result}",
        );
    }
}
