//! Responsive utilities generation logic for NemCSS.
//!
//! This module contains functions to generate responsive utility classes based on the design tokens and the viewports defined in the configuration.

use config::{ResolvedToken, TokenValue};
use miette::Diagnostic;
use std::fmt::Write;
use thiserror::Error;

use super::utilities::Utility;

/// Generate responsive utility variants wrapped in media queries based on the design tokens and viewports defined.
///
/// Creates a responsive variant for each utility class at each viewport breakpoint.
/// For example, with viewports `sm` (640px) and `md` (768px) and a utility class `text-primary`,
/// this generates:
///
/// ```css
/// @media (min-width: 640px) {
///   .sm\:text-primary {
///     color: var(--color-primary);
///   }
/// }
///
/// @media (min-width: 768px) {
///   .md\:text-primary {
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
        utilities_media_blocks.push(create_media_query_block(
            viewport_name,
            viewport_value,
            utilities,
        ));
    }

    utilities_media_blocks
}

/// Creates a media query block for utilities at a specific viewport.
pub fn create_media_query_block(
    viewport_name: &str,
    viewport_value: &TokenValue,
    utilities: &[Utility],
) -> String {
    // estimate of roughly 60 characters per utility class
    let estimated_capacity = utilities.len() * (viewport_name.len() + 60);
    let mut media_block_content = String::with_capacity(estimated_capacity);

    for utility in utilities.iter() {
        let _ = writeln!(
            &mut media_block_content,
            ".{}\\:{} {{\n  {};\n}}",
            viewport_name,
            utility.class_name(),
            utility.class_value()
        );
    }

    format!(
        "@media (min-width: {}) {{\n{}\n}}",
        viewport_value,
        media_block_content.trim_end()
    )
}

/// A responsive utility variant with its viewport information.
/// Its main use case is for the LSP in order to keep track of all responsive utilities
#[derive(Debug, PartialEq, Clone)]
pub struct ResponsiveUtility {
    /// The responsive class name (e.g.`sm:text-primary`)
    pub responsive_class_name: String,
    /// The base utility this is derived from
    pub base_utility: Utility,
    /// The viewport name this responsive utility is associated with (e.g. `sm`)
    pub viewport_name: String,
    /// The viewport value this responsive utility is associated with (e.g. `640px`)
    pub viewport_value: String,
    /// The full CSS media query and class definition for this responsive utility
    pub full_css_definition: String,
}

#[derive(Debug, Error, Diagnostic)]
pub enum GenerateResponsiveUtilitiesError {
    #[error(
        "The viewports token is not in the expected format. Expected a simple token with viewport names as keys and viewport values as values."
    )]
    #[diagnostic(code(generate_responsive_utilities_error::invalid_viewports_format))]
    /// Error when the viewports token is not in the expected format
    InvalidViewportsFormat,
}

/// Generate responsive utility variants with their viewport information based on the design tokens and viewports defined.
pub fn generate_all_responsive_utilities(
    utilities: &[Utility],
    viewports: Option<&ResolvedToken>,
) -> miette::Result<Vec<ResponsiveUtility>, GenerateResponsiveUtilitiesError> {
    let Some(viewports) = viewports else {
        return Ok(Vec::new());
    };

    let mut responsive_utilities = Vec::with_capacity(utilities.len() * viewports.tokens.len());

    for (viewport_name, viewport_value) in viewports.tokens.iter() {
        let viewport_value = match viewport_value {
            TokenValue::Simple(val) => val.to_string(),
            _ => {
                return Err(GenerateResponsiveUtilitiesError::InvalidViewportsFormat);
            }
        };

        for utility in utilities.iter() {
            responsive_utilities.push(ResponsiveUtility {
                responsive_class_name: format!("{}:{}", viewport_name, utility.class_name()),
                base_utility: utility.clone(),
                viewport_name: viewport_name.clone(),
                viewport_value: viewport_value.clone(),
                full_css_definition: create_responsive_utility_full_css_documentation(
                    viewport_name,
                    &viewport_value,
                    utility,
                ),
            })
        }
    }

    Ok(responsive_utilities)
}

/// Creates a media query block for for a single utility at a specific viewport.
/// This is used when generating the LSP's completion items documentation
fn create_responsive_utility_full_css_documentation(
    viewport_name: &str,
    viewport_value: &str,
    base_utility: &Utility,
) -> String {
    let mut media_block_content = String::new();

    let _ = writeln!(
        &mut media_block_content,
        "  .{}\\:{} {{\n    {};\n  }}",
        viewport_name,
        base_utility.class_name(),
        base_utility.class_value()
    );

    format!(
        "@media (min-width: {}) {{\n{}\n}}",
        viewport_value,
        media_block_content.trim_end()
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use config::{TokenUtilityConfig, TokenValue};

    use crate::{VIEWPORT_TOKEN_PREFIX, generation::utilities::generate_utilities};

    use super::*;

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
            result[0].contains(".sm\\:text-primary {\n  color: var(--color-primary);\n}"),
            "Got {:?}",
            result[0]
        );
        assert!(
            result[0].contains(".sm\\:text-secondary {\n  color: var(--color-secondary);\n}"),
            "Got {:?}",
            result[0]
        );
        assert!(
            result[0].contains(".sm\\:bg-primary {\n  background-color: var(--color-primary);\n}"),
            "Got {:?}",
            result[0]
        );
        assert!(
            result[0]
                .contains(".sm\\:bg-secondary {\n  background-color: var(--color-secondary);\n}"),
            "Got {:?}",
            result[0]
        );

        assert!(result[1].contains("@media (min-width: 768px) {"));
        assert!(
            result[1].contains(".md\\:text-primary {\n  color: var(--color-primary);\n}"),
            "Got {:?}",
            result[1]
        );
        assert!(
            result[1].contains(".md\\:text-secondary {\n  color: var(--color-secondary);\n}"),
            "Got {:?}",
            result[1]
        );
        assert!(
            result[1].contains(".md\\:bg-primary {\n  background-color: var(--color-primary);\n}"),
            "Got {:?}",
            result[1]
        );
        assert!(
            result[1]
                .contains(".md\\:bg-secondary {\n  background-color: var(--color-secondary);\n}"),
            "Got {:?}",
            result[1]
        );
    }

    /// Test helper to assert that a ResponsiveUtility exits with the given values
    fn assert_contains_responsive_utility(
        result: &[ResponsiveUtility],
        responsive_class_name: &str,
        class_name: &str,
        class_value: &str,
        viewport_name: &str,
        viewport_value: &str,
    ) {
        assert!(
            result.iter().any(|ru| {
                ru.responsive_class_name == responsive_class_name
                    && ru.base_utility.class_name == class_name
                    && ru.base_utility.class_value == class_value
                    && ru.viewport_name == viewport_name
                    && ru.viewport_value == viewport_value
            }),
            "Expected to find responsive utility '{}' with viewport '{}:{}', but it was not found in the results {:?}",
            responsive_class_name,
            viewport_name,
            viewport_value,
            result
        );
    }

    #[test]
    fn test_create_all_responsive_utilities() {
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
            generate_all_responsive_utilities(&all_utilities, resolved_tokens.get("viewports"))
                .unwrap();

        assert_eq!(result.len(), 8);

        assert_contains_responsive_utility(
            &result,
            "sm:text-primary",
            "text-primary",
            "color: var(--color-primary)",
            "sm",
            "320px",
        );
        assert_contains_responsive_utility(
            &result,
            "sm:text-secondary",
            "text-secondary",
            "color: var(--color-secondary)",
            "sm",
            "320px",
        );
        assert_contains_responsive_utility(
            &result,
            "sm:bg-primary",
            "bg-primary",
            "background-color: var(--color-primary)",
            "sm",
            "320px",
        );
        assert_contains_responsive_utility(
            &result,
            "sm:bg-secondary",
            "bg-secondary",
            "background-color: var(--color-secondary)",
            "sm",
            "320px",
        );

        assert_contains_responsive_utility(
            &result,
            "md:text-primary",
            "text-primary",
            "color: var(--color-primary)",
            "md",
            "768px",
        );
        assert_contains_responsive_utility(
            &result,
            "md:text-secondary",
            "text-secondary",
            "color: var(--color-secondary)",
            "md",
            "768px",
        );
        assert_contains_responsive_utility(
            &result,
            "md:bg-primary",
            "bg-primary",
            "background-color: var(--color-primary)",
            "md",
            "768px",
        );
        assert_contains_responsive_utility(
            &result,
            "md:bg-secondary",
            "bg-secondary",
            "background-color: var(--color-secondary)",
            "md",
            "768px",
        );
    }
}
