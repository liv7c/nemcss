use config::ResolvedToken;

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
}
