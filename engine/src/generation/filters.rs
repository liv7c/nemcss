//! Filtering logic for generating CSS based on classes being in use in the user project.

use std::collections::{HashMap, HashSet};

use crate::generation::{responsive::create_media_query_block, utilities::create_utility};

use super::utilities::{Utility, VIEWPORT_TOKEN_PREFIX};
use config::ResolvedToken;

/// Generates filtered utilities based on the used classes.
///
/// Returns a vec of utilities and a map of used responsive utility classes.
pub fn generate_filtered_utilities(
    resolved_tokens: &[&ResolvedToken],
    used_utilities: &[String],
    used_responsive_utilities: &HashMap<String, Vec<String>>,
) -> (Vec<Utility>, HashMap<String, Vec<Utility>>) {
    let mut utilities = Vec::new();
    let mut responsive_utilities: HashMap<String, Vec<Utility>> = HashMap::new();

    for resolved_token in resolved_tokens
        .iter()
        .filter(|r| r.prefix != VIEWPORT_TOKEN_PREFIX)
    {
        for utility in resolved_token.utilities.iter() {
            for (token_name, _token_value) in resolved_token.tokens.iter() {
                let utility_class_name = format!("{}-{}", utility.prefix, token_name);

                let is_used = used_utilities.contains(&utility_class_name);
                let is_used_responsive =
                    used_responsive_utilities.contains_key(&utility_class_name);

                if !is_used && !is_used_responsive {
                    continue;
                }

                let utility = create_utility(resolved_token, utility, token_name);

                if is_used {
                    utilities.push(utility.clone());
                }

                if let Some(viewports) = used_responsive_utilities.get(&utility_class_name) {
                    for vw in viewports {
                        responsive_utilities
                            .entry(vw.clone())
                            .or_default()
                            .push(utility.clone());
                    }
                }
            }
        }
    }

    (utilities, responsive_utilities)
}

/// Generate the filtered responsive utilities
pub fn generate_filtered_responsive_utilities(
    responsive_utilities_map: &HashMap<String, Vec<Utility>>,
    viewports: Option<&ResolvedToken>,
) -> Vec<String> {
    let Some(viewports) = viewports else {
        return Vec::new();
    };

    let mut utilities_media_blocks = Vec::with_capacity(responsive_utilities_map.len());

    for (viewport_name, viewport_value) in viewports.tokens.iter() {
        let Some(utilities) = responsive_utilities_map.get(viewport_name) else {
            continue;
        };

        utilities_media_blocks.push(create_media_query_block(
            viewport_name,
            viewport_value,
            utilities,
        ));
    }

    utilities_media_blocks
}

/// Splits the used classes into utility classes and responsive utility classes.
pub fn parse_used_classes(
    used_classes: &HashSet<String>,
) -> (Vec<String>, HashMap<String, Vec<String>>) {
    let mut used_utility_classes = Vec::new();
    let mut used_responsive_utilities: HashMap<String, Vec<String>> = HashMap::new();

    for class_name in used_classes {
        if let Some((vw, utility_name)) = class_name.split_once(':') {
            used_responsive_utilities
                .entry(utility_name.to_string())
                .or_default()
                .push(vw.to_string());
            continue;
        }
        used_utility_classes.push(class_name.to_string());
    }

    (used_utility_classes, used_responsive_utilities)
}

#[cfg(test)]
mod tests {
    use config::{TokenUtilityConfig, TokenValue};

    use super::*;

    #[test]
    fn test_parse_used_classes() {
        let used_classes = HashSet::from([
            "text-primary".to_string(),
            "text-secondary".to_string(),
            "m-2".to_string(),
            "md:bg-primary".to_string(),
            "lg:bg-primary".to_string(),
            "xl:text-secondary".to_string(),
        ]);

        let result = parse_used_classes(&used_classes);

        let (used_utility_classes, used_responsive_utilities) = result;

        assert_eq!(used_utility_classes.len(), 3);
        assert_eq!(used_responsive_utilities.len(), 2);

        assert!(used_utility_classes.contains(&"text-primary".to_string()));
        assert!(used_utility_classes.contains(&"text-secondary".to_string()));
        assert!(used_utility_classes.contains(&"m-2".to_string()));

        assert!(
            used_responsive_utilities
                .get("bg-primary")
                .unwrap()
                .contains(&"md".to_string())
        );
        assert!(
            used_responsive_utilities
                .get("bg-primary")
                .unwrap()
                .contains(&"lg".to_string())
        );
        assert_eq!(
            used_responsive_utilities.get("text-secondary").unwrap(),
            &vec!["xl".to_string()]
        );
    }

    #[test]
    fn test_generate_filtered_utilities() {
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
        let used_utilities = vec!["text-primary".to_string()];
        let used_responsive_utilities = HashMap::from([(
            "bg-primary".to_string(),
            vec!["sm".to_string(), "md".to_string()],
        )]);
        let (utilities, responsive_utilities) =
            generate_filtered_utilities(&all_tokens, &used_utilities, &used_responsive_utilities);
        dbg!(&utilities);
        dbg!(&responsive_utilities);

        assert_eq!(utilities.len(), 1);
        assert_eq!(responsive_utilities.len(), 2);

        assert!(utilities.contains(&Utility::new(
            ".text-primary {\n  color: var(--color-primary);\n}",
            "text-primary",
            "color: var(--color-primary)"
        )));
    }

    #[test]
    fn test_generate_filtered_responsive_utilities() {
        let responsive_utilities_map = HashMap::from([
            (
                "md".to_string(),
                vec![
                    Utility::new(
                        ".bg-primary {\n  background-color: var(--color-primary);\n}",
                        "bg-primary",
                        "background-color: var(--color-primary)",
                    ),
                    Utility::new(
                        ".bg-secondary {\n  background-color: var(--color-secondary);\n}",
                        "bg-secondary",
                        "background-color: var(--color-secondary)",
                    ),
                ],
            ),
            (
                "lg".to_string(),
                vec![Utility::new(
                    ".bg-primary {\n  background-color: var(--color-primary);\n}",
                    "bg-primary",
                    "background-color: var(--color-primary)",
                )],
            ),
        ]);
        let viewports = ResolvedToken {
            tokens: vec![
                ("sm".to_string(), TokenValue::Simple("320px".to_string())),
                ("md".to_string(), TokenValue::Simple("768px".to_string())),
                ("lg".to_string(), TokenValue::Simple("1024px".to_string())),
            ],
            utilities: vec![],
            prefix: VIEWPORT_TOKEN_PREFIX.to_string(),
        };

        let result =
            generate_filtered_responsive_utilities(&responsive_utilities_map, Some(&viewports));

        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            "@media (min-width: 768px) {\n.md:bg-primary {\n  background-color: var(--color-primary);\n}\n.md:bg-secondary {\n  background-color: var(--color-secondary);\n}\n}"
        );

        assert_eq!(
            result[1],
            "@media (min-width: 1024px) {\n.lg:bg-primary {\n  background-color: var(--color-primary);\n}\n}"
        );
    }
}
