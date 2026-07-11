use config::{TokenItem, TokenValue};

use crate::commands::new_token_file::error::NewTokenFileError;

/// Format a number into a string
pub fn format_number(value: f64) -> String {
    value.to_string()
}

/// Split a `--values` string on top-level commas only.
/// Commas inside of parentheses remain untouched.
///
/// ```
/// split_values("1rem, clamp(1rem, 2vw+1.5rem, 4rem), 5rem");
/// # vec!["1rem", "clamp(1rem, 2vw+1.5rem, 4rem)", "5rem"]
/// ```
fn split_values(raw: &str) -> Vec<String> {
    let mut output = Vec::new();
    // buffer used to keep track of current substring
    let mut current = String::new();
    let mut depth: usize = 0;

    for c in raw.chars() {
        match c {
            '(' => {
                depth += 1;
                current.push(c);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                current.push(c);
            }
            ',' if depth == 0 => {
                output.push(current.trim().to_string());
                current.clear()
            }
            _ => current.push(c),
        }
    }

    output.push(current.trim().to_string());
    output.retain(|item| !item.is_empty());
    output
}

/// Scale system used to generate the token values.
/// It can have 3 values:
/// - generated: generates a scale based on other options provided such as step, start and count
/// - explicit: user provides their own token values
/// - placeholder: user only wants to generate a token file with no design tokens defined yet
pub enum ScaleSource {
    Generated { start: f64, step: f64, count: usize },
    Explicit(Vec<String>),
    Placeholder,
}

/// Builds the token items for a scale, pairing each value with a name.
/// Numeric values get the unit appended (except for zero). Anything else gets passed through as such.
pub fn build_items(
    source: &ScaleSource,
    names: Option<&[String]>,
    unit: &str,
) -> Result<Vec<TokenItem>, NewTokenFileError> {
    let values: Vec<String> = match source {
        ScaleSource::Explicit(values) => values.clone(),
        ScaleSource::Generated { start, step, count } => (0..*count)
            .map(|i| format_number(start + step * i as f64))
            .collect(),
        ScaleSource::Placeholder => Vec::new(),
    };

    let names: Vec<String> = match names {
        Some(names) if names.len() == values.len() => names.to_vec(),
        Some(names) => {
            return Err(NewTokenFileError::NameCountMismatch {
                expected: values.len(),
                got: names.len(),
            });
        }
        None => values
            .iter()
            .map(|value| match value.parse::<f64>() {
                Ok(num) => Ok(format_number(num)),
                Err(_) => Err(NewTokenFileError::NameRequiredForValue {
                    value: value.clone(),
                }),
            })
            .collect::<Result<_, _>>()?,
    };

    let items = values
        .into_iter()
        .zip(names)
        .map(|(value, name)| {
            let rendered = match value.parse::<f64>() {
                Ok(0.0) => "0".to_string(),
                Ok(num) => format!("{}{unit}", format_number(num)),
                Err(_) => value,
            };

            TokenItem {
                name,
                value: TokenValue::Simple(rendered),
            }
        })
        .collect();

    Ok(items)
}

#[cfg(test)]
mod tests {
    use crate::commands::new_token_file::error::NewTokenFileError;

    use super::*;
    use config::TokenValue;

    /// Test helper to build Explicit source from string literals
    fn explicit(values: &[&str]) -> ScaleSource {
        ScaleSource::Explicit(values.iter().map(|v| v.to_string()).collect())
    }

    #[test]
    fn format_number_drops_trailing_zero_fraction() {
        assert_eq!(format_number(1.0), "1");
        assert_eq!(format_number(16.0), "16");
    }

    #[test]
    fn format_number_keeps_meaningful_fraction() {
        assert_eq!(format_number(1.5), "1.5");
        assert_eq!(format_number(0.125), "0.125");
        assert_eq!(format_number(1.25), "1.25");
    }

    #[test]
    fn split_values_splits_on_commas() {
        assert_eq!(split_values("8,16,24"), vec!["8", "16", "24"]);
    }

    #[test]
    fn split_values_ignores_commas_inside_parentheses() {
        assert_eq!(
            split_values("1rem, clamp(1.5rem, 1rem + 2vw, 2.5rem), 2rem"),
            vec!["1rem", "clamp(1.5rem, 1rem + 2vw, 2.5rem)", "2rem"]
        );
    }

    #[test]
    fn split_values_trims_whitespace_and_drops_empty_items() {
        assert_eq!(split_values(" 8,   16"), vec!["8", "16"]);
        assert_eq!(split_values("8,,16"), vec!["8", "16"]);
    }

    #[test]
    fn explicit_numeric_values_become_items_named_after_their_value() {
        let source = explicit(&["8", "16", "24"]);
        let items = build_items(&source, None, "px").unwrap();

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].name, "8");
        assert_eq!(items[0].value, TokenValue::Simple("8px".to_string()));
        assert_eq!(items[1].name, "16");
        assert_eq!(items[1].value, TokenValue::Simple("16px".to_string()));
        assert_eq!(items[2].name, "24");
        assert_eq!(items[2].value, TokenValue::Simple("24px".to_string()));
    }

    #[test]
    fn numeric_values_are_normalized_through_format_number() {
        let items = build_items(&explicit(&["8.0"]), None, "px").unwrap();

        assert_eq!(items[0].name, "8");
        assert_eq!(items[0].value, TokenValue::Simple("8px".to_string()));
    }

    #[test]
    fn generated_scale_produces_uniform_steps() {
        let source = ScaleSource::Generated {
            start: 0.5,
            step: 0.5,
            count: 4,
        };
        let items = build_items(&source, None, "rem").unwrap();

        let names: Vec<&str> = items.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, vec!["0.5", "1", "1.5", "2"]);

        assert_eq!(items[1].value, TokenValue::Simple("1rem".to_string()));
    }

    #[test]
    fn non_numeric_values_require_names() {
        let result = build_items(&explicit(&["orange", "purple"]), None, "");
        assert!(matches!(
            result,
            Err(NewTokenFileError::NameRequiredForValue { value }) if value == "orange"
        ))
    }

    #[test]
    fn zero_values_serialize_without_unit() {
        let source = ScaleSource::Generated {
            start: 0.0,
            step: 8.0,
            count: 3,
        };

        let items = build_items(&source, None, "px").unwrap();

        assert_eq!(items[0].name, "0");
        assert_eq!(items[0].value, TokenValue::Simple("0".to_string()));
        assert_eq!(items[1].name, "8");
        assert_eq!(items[1].value, TokenValue::Simple("8px".to_string()));
        assert_eq!(items[2].name, "16");
        assert_eq!(items[2].value, TokenValue::Simple("16px".to_string()));
    }

    #[test]
    fn names_overlay_zips_with_values() {
        let source = &explicit(&["1.0", "1.25", "1.50", "2.0"]);
        let names = vec![
            "sm".to_string(),
            "md".to_string(),
            "lg".to_string(),
            "xl".to_string(),
        ];
        let items = build_items(&source, Some(&names), "rem").unwrap();

        assert_eq!(items[1].name, "md");
        assert_eq!(items[1].value, TokenValue::Simple("1.25rem".to_string()));
    }

    #[test]
    fn name_count_mismatch_is_an_error() {
        // 4 values vs only 3 names
        let source = &explicit(&["1.0", "1.25", "1.50", "2.0"]);
        let names = vec!["sm".to_string(), "md".to_string(), "lg".to_string()];
        let result = build_items(&source, Some(&names), "rem");

        assert!(matches!(
            result,
            Err(NewTokenFileError::NameCountMismatch {
                expected: 4,
                got: 3
            })
        ));
    }

    #[test]
    fn placeholder_produces_no_items() {
        let items = build_items(&ScaleSource::Placeholder, None, "px").unwrap();
        assert!(items.is_empty());
    }
}
