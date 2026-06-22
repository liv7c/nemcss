use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, PartialEq, Clone)]
pub enum SemanticReferenceProblem {
    /// The token reference is not of the form `{group.token}`
    InvalidSyntax { reference: String },
    /// The token group does not exist
    UnknownGroup {
        reference: String,
        group: String,
        available_groups: Vec<String>,
    },
    /// `{group.token}` where `group` exists but `token` does not
    UnknownToken {
        reference: String,
        group: String,
        token: String,
    },
}

impl SemanticReferenceProblem {
    pub fn message(&self) -> String {
        match self {
            SemanticReferenceProblem::InvalidSyntax { reference } => {
                format!(
                    "Invalid token reference syntax: {}. Expected the form '{{group.token}}'",
                    reference
                )
            }
            SemanticReferenceProblem::UnknownGroup {
                reference,
                group,
                available_groups,
            } => {
                format!(
                    "Unknown token group `{}` in reference `{}`. Available groups are: {}",
                    group,
                    reference,
                    available_groups.join(", ")
                )
            }
            SemanticReferenceProblem::UnknownToken {
                reference,
                group,
                token,
            } => {
                format!(
                    "Unknown token `{}` in group `{}` (reference '{}')",
                    token, group, reference
                )
            }
        }
    }
}

/// Set of valid groups with their token names.
#[derive(Debug, Default)]
pub struct KnownTokens {
    groups: BTreeMap<String, BTreeSet<String>>,
}

impl KnownTokens {
    pub fn from_references(references: &[String]) -> Self {
        let mut groups: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for reference in references {
            if let Some((group, token)) = parse_semantic_reference(reference) {
                groups
                    .entry(group.to_string())
                    .or_default()
                    .insert(token.to_string());
            }
        }
        Self { groups }
    }

    fn group_names(&self) -> Vec<String> {
        self.groups.keys().cloned().collect()
    }
}

fn parse_semantic_reference(reference: &str) -> Option<(&str, &str)> {
    let inner = reference.strip_prefix('{')?.strip_suffix('}')?;
    let (group, token) = inner.split_once('.')?;
    if group.is_empty() || token.is_empty() {
        return None;
    }
    Some((group, token))
}

/// Determines if a semantic token reference is valid.
/// Returns `None` if the reference is valid, otherwise returns a `SemanticReferenceProblem`.
pub fn validate_semantic_reference(
    reference: &str,
    known_tokens: &KnownTokens,
) -> Option<SemanticReferenceProblem> {
    let Some((group, token)) = parse_semantic_reference(reference) else {
        return Some(SemanticReferenceProblem::InvalidSyntax {
            reference: reference.to_string(),
        });
    };

    let Some(tokens) = known_tokens.groups.get(group) else {
        return Some(SemanticReferenceProblem::UnknownGroup {
            reference: reference.to_string(),
            group: group.to_string(),
            available_groups: known_tokens.group_names(),
        });
    };

    if !tokens.contains(token) {
        return Some(SemanticReferenceProblem::UnknownToken {
            reference: reference.to_string(),
            group: group.to_string(),
            token: token.to_string(),
        });
    }

    None
}

mod tests {
    use super::*;

    fn known() -> KnownTokens {
        KnownTokens::from_references(&[
            "{colors.blue-700}".to_string(),
            "{colors.neutral-100}".to_string(),
            "{spacings.md}".to_string(),
        ])
    }

    #[test]
    fn valid_reference_returns_none() {
        let result = validate_semantic_reference("{colors.blue-700}", &known());
        assert!(result.is_none());
    }

    #[test]
    fn unknown_group_is_reported_with_available_groups() {
        let result = validate_semantic_reference("{unknown.token}", &known());
        assert_eq!(
            result,
            Some(SemanticReferenceProblem::UnknownGroup {
                reference: "{unknown.token}".to_string(),
                group: "unknown".to_string(),
                available_groups: vec!["colors".to_string(), "spacings".to_string()],
            })
        );
    }

    #[test]
    fn unknown_token_is_reported() {
        let result = validate_semantic_reference("{colors.unknown}", &known());
        assert_eq!(
            result,
            Some(SemanticReferenceProblem::UnknownToken {
                reference: "{colors.unknown}".to_string(),
                group: "colors".to_string(),
                token: "unknown".to_string(),
            })
        );
    }

    #[test]
    fn invalid_syntax_is_reported() {
        let bad_syntaxes = vec![
            "colors.blue-700".to_string(),
            "{colors.blue-700".to_string(),
            "{colors.}".to_string(),
            "{.blue-700}".to_string(),
        ];

        for bad in bad_syntaxes {
            let result = validate_semantic_reference(&bad, &known());
            assert_eq!(
                result,
                Some(SemanticReferenceProblem::InvalidSyntax { reference: bad })
            );
        }
    }
}
