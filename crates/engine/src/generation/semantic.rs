use config::ResolvedSemanticGroup;

use super::utilities::Utility;

/// Generates CSS custom property declarations for all semantic groups.
///
/// For group "text" with token ("primary", "{colors.blue-500}"), this generates:
/// "--text-primary: var(--colors-blue-500);"
pub fn generate_semantic_custom_properties(groups: &[&ResolvedSemanticGroup]) -> Vec<String> {
    groups
        .iter()
        .flat_map(|group| {
            group.tokens.iter().map(|(name, resolved_var)| {
                format!("--{}-{}: {};", group.prefix, name, resolved_var)
            })
        })
        .collect()
}

/// Generates CSS utility classes for all semantic groups.
///
/// For group "text" with token ("primary", "{colors.blue-500}"), this generates:
/// ".text-primary { color: var(--colors-blue-500); }"
/// Note: this is useful for theming as it allows you to override the default semantic tokens.
pub fn generate_semantic_utilities(groups: &[&ResolvedSemanticGroup]) -> Vec<Utility> {
    groups
        .iter()
        .flat_map(|group| {
            group.tokens.iter().map(|(name, _)| {
                let utility_class_name = format!("{}-{}", group.prefix, name);
                let utility_class_value =
                    format!("{}: var(--{}-{})", group.property, group.prefix, name);
                let utility_full_class =
                    format!(".{} {{\n  {};\n}}", utility_class_name, utility_class_value);

                Utility::new(
                    &utility_full_class,
                    &utility_class_name,
                    &utility_class_value,
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::ResolvedSemanticGroup;

    fn text_group() -> ResolvedSemanticGroup {
        ResolvedSemanticGroup {
            prefix: "text".to_string(),
            property: "color".to_string(),
            tokens: vec![
                ("primary".to_string(), "var(--color-blue-500)".to_string()),
                ("secondary".to_string(), "var(--color-red-500)".to_string()),
            ],
        }
    }

    #[test]
    fn test_generate_semantic_custom_properties() {
        let group = text_group();
        let result = generate_semantic_custom_properties(&[&group]);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "--text-primary: var(--color-blue-500);");
        assert_eq!(result[1], "--text-secondary: var(--color-red-500);");
    }

    #[test]
    fn test_generate_semantic_utilities() {
        let group = text_group();
        let result = generate_semantic_utilities(&[&group]);

        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].full_class(),
            ".text-primary {\n  color: var(--text-primary);\n}"
        );
        assert_eq!(
            result[1].full_class(),
            ".text-secondary {\n  color: var(--text-secondary);\n}"
        );
    }

    #[test]
    fn test_empty_group_produces_nothing() {
        let group = ResolvedSemanticGroup {
            prefix: "empty".to_string(),
            property: "color".to_string(),
            tokens: vec![],
        };
        let result = generate_semantic_custom_properties(&[&group]);
        assert!(result.is_empty());

        let result = generate_semantic_utilities(&[&group]);
        assert!(result.is_empty());
    }
}
