use config::ResolvedToken;
use std::fmt::Write;

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
///     ".text-primary {\n  color: var(--color-primary);\n}".to_string(),
///     "text-primary".to_string(),
///     "color: var(--color-primary)".to_string(),
/// );
///
/// assert_eq!(utility.full_class(), ".text-primary {\n  color: var(--color-primary);\n}");
/// assert_eq!(utility.class_name(), "text-primary");
/// assert_eq!(utility.class_value(), "color: var(--color-primary)");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Utility {
    /// The complete CSS class definition (e.g., ".text-primary {\n  color: var(--color-primary);\n}")
    full_class: String,
    /// The class name without the leading dot (e.g., "text-primary")
    class_name: String,
    /// The class property and value (e.g., "color: var(--color-primary)")
    class_value: String,
}

impl Utility {
    /// Creates a new `Utility` instance.
    ///
    /// # Arguments
    ///
    /// * `full_class` - The complete CSS class definition (e.g., ".text-primary {\n  color: var(--color-primary);\n}")
    /// * `class_name` - The class name without the leading dot (e.g., "text-primary")
    /// * `class_value` - The class property and value (e.g., "color: var(--color-primary)")
    pub fn new(full_class: String, class_name: String, class_value: String) -> Self {
        Utility {
            full_class,
            class_name,
            class_value,
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

/// The prefix for viewport tokens.
/// This prefix is used to identify viewport tokens in the resolved tokens and generate
/// responsive utility classes.
///
/// **Note**: This is primarily used for internal purposes. External users typically do not need to know about this prefix.
pub const VIEWPORT_TOKEN_PREFIX: &str = "viewport";

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
) -> GeneratedCss {
    let tokens: Vec<_> = resolved_tokens.into_iter().collect();
    let custom_properties = generate_custom_properties(&tokens);
    let utilities = generate_utilities(&tokens);
    let responsive_utilities = generate_responsive_utilities(&utilities, viewports);

    GeneratedCss::new(custom_properties, utilities, responsive_utilities)
}

/// Generate CSS custom properties from resolved tokens.
///
/// Custom properties are generated for each token and its value.
/// For example, with a token `colors` with a primary color `primary` and a value `yellow`,
/// this generates:
///
/// ```css
/// :root {
///   --color-primary: yellow;
/// }
/// ```
///
/// # Arguments
///
/// * `resolved_tokens` - A slice of `ResolvedToken` instances representing the tokens to generate custom properties for.
///
/// # Returns
///
/// A vector of CSS custom property definitions.
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
    let estimated_capacity = resolved_tokens.iter().fold(0, |acc, curr| {
        acc + curr.tokens.len() * curr.utilities.len()
    });

    let mut utilities = Vec::with_capacity(estimated_capacity);

    for resolved_token in resolved_tokens
        .iter()
        .filter(|r| r.prefix != VIEWPORT_TOKEN_PREFIX)
    {
        for utility in resolved_token.utilities.iter() {
            for (token_name, _token_value) in resolved_token.tokens.iter() {
                let custom_property_name =
                    format!("var(--{}-{})", resolved_token.prefix, token_name);
                let utility_class_value = format!("{}: {}", utility.property, custom_property_name);
                let utility_class_name = format!("{}-{}", utility.prefix, token_name);
                let utility_full_class =
                    format!(".{} {{\n  {};\n}}", utility_class_name, utility_class_value);
                let utility =
                    Utility::new(utility_full_class, utility_class_name, utility_class_value);
                utilities.push(utility);
            }
        }
    }

    utilities
}

/// Generate responsive utility variants wrapped in media queries based on the design tokens and viewports defined.
///
/// Creates a responsive variant for each utility class at each viewport breakpoint.
/// For example, with viewports `sm` (640px) and `md` (768px) and a utility class `text-primary`,
/// this generates:
///
/// ```css
/// @media (min-width: 640px) {
///   .sm:text-primary {
///     color: var(--color-primary);
///   }
/// }
///
/// @media (min-width: 768px) {
///   .md:text-primary {
///     color: var(--color-primary);
///   }
/// }
/// ```
///
/// # Arguments
///
/// * `utilities` - Base utility classes to generate responsive variants for.
/// * `viewports` - An optional `ResolvedToken` instance representing the viewports to generate responsive variants for.
///
/// # Returns
///
/// A vector of media query blocks, one per viewport, each containing the responsive utility variants.
pub fn generate_responsive_utilities(
    utilities: &[Utility],
    viewports: Option<&ResolvedToken>,
) -> Vec<String> {
    let Some(viewports) = viewports else {
        return Vec::new();
    };

    let mut utilities_media_blocks = Vec::with_capacity(viewports.tokens.len());

    for (viewport_name, viewport_value) in viewports.tokens.iter() {
        // estimate of roughly 60 characters per utility class
        let estimated_capacity = utilities.len() * (viewport_name.len() + 60);
        let mut media_block_content = String::with_capacity(estimated_capacity);

        for utility in utilities.iter() {
            let _ = writeln!(
                &mut media_block_content,
                ".{}:{} {{\n  {};\n}}",
                viewport_name,
                utility.class_name(),
                utility.class_value()
            );
        }

        let media_block = format!(
            "@media (min-width: {}) {{\n{}\n}}",
            viewport_value,
            media_block_content.trim_end()
        );
        utilities_media_blocks.push(media_block);
    }

    utilities_media_blocks
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

        let css_to_generate =
            generate_css(resolved_tokens.values(), resolved_tokens.get("viewports"));

        let result = css_to_generate.to_css();

        // Check for individual custom properties (order is not guaranteed due to HashMap)
        assert!(result.contains("--color-primary: yellow;"));
        assert!(result.contains("--color-secondary: #c1c1c1;"));
        assert!(result.contains("--viewport-sm: 320px;"));
        assert!(result.contains("--viewport-md: 768px;"));

        let expected_utilities_css = ".text-primary {\n  color: var(--color-primary);\n}\n.text-secondary {\n  color: var(--color-secondary);\n}\n";
        let expected_responsive_utilities_sm = "@media (min-width: 320px) {\n.sm:text-primary {\n  color: var(--color-primary);\n}\n.sm:text-secondary {\n  color: var(--color-secondary);\n}\n}";
        let expected_responsive_utilities_md = "@media (min-width: 768px) {\n.md:text-primary {\n  color: var(--color-primary);\n}\n.md:text-secondary {\n  color: var(--color-secondary);\n}\n}";

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

    #[test]
    fn test_generate_responsive_utilities() {
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

        let all_tokens: Vec<_> = resolved_tokens.values().collect();
        let all_utilities = generate_utilities(&all_tokens);
        let result =
            generate_responsive_utilities(&all_utilities, resolved_tokens.get("viewports"));
        assert_eq!(result.len(), 2);

        assert!(result[0].contains("@media (min-width: 320px) {"));
        assert!(
            result[0].contains(".sm:text-primary {\n  color: var(--color-primary);\n}"),
            "Got {:?}",
            result[0]
        );
        assert!(
            result[0].contains(".sm:text-secondary {\n  color: var(--color-secondary);\n}"),
            "Got {:?}",
            result[0]
        );
        assert!(
            result[0].contains(".sm:bg-primary {\n  background-color: var(--color-primary);\n}"),
            "Got {:?}",
            result[0]
        );
        assert!(
            result[0]
                .contains(".sm:bg-secondary {\n  background-color: var(--color-secondary);\n}"),
            "Got {:?}",
            result[0]
        );

        assert!(result[1].contains("@media (min-width: 768px) {"));
        assert!(
            result[1].contains(".md:text-primary {\n  color: var(--color-primary);\n}"),
            "Got {:?}",
            result[1]
        );
        assert!(
            result[1].contains(".md:text-secondary {\n  color: var(--color-secondary);\n}"),
            "Got {:?}",
            result[1]
        );
        assert!(
            result[1].contains(".md:bg-primary {\n  background-color: var(--color-primary);\n}"),
            "Got {:?}",
            result[1]
        );
        assert!(
            result[1]
                .contains(".md:bg-secondary {\n  background-color: var(--color-secondary);\n}"),
            "Got {:?}",
            result[1]
        );
    }
}
