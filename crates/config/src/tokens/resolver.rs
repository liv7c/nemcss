use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use miette::{Diagnostic, Result};
use thiserror::Error;

use crate::tokens::token::{TokenFile, TokenValue};
use crate::tokens::utilities::{default_prefix_for_token_type, get_utilities_for_token_type};
use crate::{NemCssConfig, SemanticConfig, TokenUtilityConfig};

/// Represents the error type when scanning the tokens directory.
#[derive(Debug, Diagnostic, Error)]
pub enum ScanTokensDirError {
    #[error("failed to read tokens directory: {0}")]
    #[diagnostic(code(config::tokens::scan_dir::read_error))]
    ReadDirError(std::io::Error),

    #[error("failed to read token file: {0}")]
    #[diagnostic(code(config::tokens::scan_dir::file_read_error))]
    FileReadError(std::io::Error),
}

/// Scans the given tokens directory and returns a mapping of token names to their file paths.
/// It enables discovering all design token files in the specified directory.
/// The token name is derived from the file name without the `.json` extension.
fn scan_tokens_dir(path: &Path) -> Result<HashMap<String, PathBuf>, ScanTokensDirError> {
    let mut tokens = HashMap::new();

    if !path.exists() || !path.is_dir() {
        return Err(ScanTokensDirError::ReadDirError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "tokens directory {:?} does not exist or is not a directory",
                path
            ),
        )));
    }

    let entries = fs::read_dir(path).map_err(ScanTokensDirError::ReadDirError)?;

    for entry in entries {
        let entry = entry.map_err(ScanTokensDirError::FileReadError)?;
        let path = entry.path();

        // Skip non-JSON files
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let prefix_from_filename = path.file_stem().and_then(|s| s.to_str());

        if let Some(prefix) = prefix_from_filename {
            tokens.insert(prefix.to_string(), path);
        }
    }

    Ok(tokens)
}

/// Represents the error type when loading tokens from a file.
#[derive(Debug, Diagnostic, Error)]
pub enum LoadTokensFromFileError {
    /// Represents an error when reading the token file.
    #[error("failed to read token file: {0}")]
    #[diagnostic(code(config::tokens::load_tokens_from_file::read_file_error))]
    ReadFileError(std::io::Error),
    /// Represents an error when parsing the token file.
    #[error("failed to parse token file: {0}")]
    #[diagnostic(code(config::tokens::load_tokens_from_file::parse_error))]
    ParseError(serde_json::Error),
}

/// Load tokens from the given token file.
fn load_tokens_from_file(
    path: &Path,
) -> Result<Vec<(String, TokenValue)>, LoadTokensFromFileError> {
    let file = fs::read_to_string(path).map_err(LoadTokensFromFileError::ReadFileError)?;
    let token_file: TokenFile =
        serde_json::from_str(&file).map_err(LoadTokensFromFileError::ParseError)?;
    Ok(token_file.into_tokens())
}

/// Represents the error type when resolving tokens.
#[derive(Debug, Diagnostic, Error)]
pub enum ResolveTokensError {
    #[error("failed to scan tokens directory: {0}")]
    ScanTokensDirError(#[from] ScanTokensDirError),

    #[error("failed to load tokens from file: {0}")]
    LoadTokensFromFileError(#[from] LoadTokensFromFileError),
}

/// Represents a resolved token.
/// It contains the tokens and their values, as well as the utilities and prefix for the token.
/// For certain token types, the prefix is automatically generated based on the token type.
#[derive(Debug, PartialEq)]
pub struct ResolvedToken {
    /// The tokens and their values.
    pub tokens: Vec<(String, TokenValue)>,
    /// The utilities for the token.
    pub utilities: Vec<TokenUtilityConfig>,
    /// The prefix for the token.
    pub prefix: String,
}

/// Resolve all tokens, based on auto-discovered token files in the tokens directory and the theme
/// configuration.
///
/// # Resolution process
///
/// 1. Scans `tokensDir` for all `.json` files
/// 2. Derives token names from filenames (e.g., `fonts.json` -> `fonts`)
/// 3. Applies theme configuration overrides:
///     - Custom `source` paths
///     - Custom `prefix` value
///     - Additional `utilities` configurations
/// 4. Merges default utilities with user-defined utilities
///
/// # Examples
///
/// ```no_run
/// use config::NemCssConfig;
///
/// let config = NemCssConfig::from_path("nemcss.config.json")?;
/// let resolved = config.resolve_all_tokens()?;
/// println!("{resolved:?}");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn resolve_all_tokens(
    config: &NemCssConfig,
) -> Result<HashMap<String, ResolvedToken>, ResolveTokensError> {
    let mut resolved_tokens = HashMap::new();

    let tokens_dir = config.tokens_dir();
    let token_files = scan_tokens_dir(&tokens_dir)?;

    // Map the token source to the token name in the theme configuration.
    let config_by_token_source: HashMap<PathBuf, String> = config
        .theme
        .as_ref()
        .map(|theme| {
            theme
                .tokens
                .iter()
                .map(|(name, cfg)| (config.base_dir.join(&cfg.source), name.clone()))
                .collect()
        })
        .unwrap_or_default();

    for (name, path) in token_files {
        // check if the token name is overridden in the theme configuration
        // if not, use the token name is derived from the file name.
        let token_name_in_config = config_by_token_source
            .get(&path)
            .cloned()
            .unwrap_or(name.clone());

        let token_config = config
            .theme
            .as_ref()
            .and_then(|t| t.tokens.get(&token_name_in_config));

        let prefix = token_config
            .and_then(|cfg| cfg.prefix.clone())
            .unwrap_or_else(|| default_prefix_for_token_type(&name));

        let tokens = load_tokens_from_file(&path)?;
        let utilities = get_utilities_for_token_type(&token_name_in_config, config.theme.as_ref());

        resolved_tokens.insert(
            token_name_in_config,
            ResolvedToken {
                tokens,
                utilities,
                prefix,
            },
        );
    }

    Ok(resolved_tokens)
}

/// A resolved semantic group for CSS generation.
/// It contains the group name, the CSS property for CSS generation as well as a vector
/// of tuples containing both the semantic name and resolved var pairs for a given semantic group
pub struct ResolvedSemanticGroup {
    /// Group name used for both CSS variable prefix and utility class prefix (e.g. "text-", "--text-*")
    pub prefix: String,
    /// CSS property for utility classes (e.g. "background-color")
    pub property: String,
    /// (semantic-name, resolved-var) pairs (e.g. ("primary", "var(--color-blue-500)"))
    pub tokens: Vec<(String, String)>,
}

/// Resolves a "{group.token}" reference to "var(--prefix-token)"
/// Returns None if the syntax is wrong or the group does not exist
fn resolve_semantic_reference(
    reference: &str,
    primitive_tokens: &HashMap<String, ResolvedToken>,
) -> Option<String> {
    let inner = reference.strip_prefix('{')?.strip_suffix('}')?;
    let (group_key, token_name) = inner.split_once('.')?;

    if group_key.is_empty() || token_name.is_empty() {
        return None;
    }

    let resolved = primitive_tokens.get(group_key)?;

    let mut token_exist = false;
    for (primitive_token_name, _) in resolved.tokens.iter() {
        if token_name == primitive_token_name {
            token_exist = true;
            break;
        }
    }

    if !token_exist {
        return None;
    }

    Some(format!("var(--{}-{})", resolved.prefix, token_name))
}

#[derive(Debug, Diagnostic, Error)]
pub enum ResolveSemanticError {
    #[error("unresolvable reference '{reference}' in semantic group '{group}'")]
    #[diagnostic(code(config::semantic::unresolvable_reference))]
    UnresolvableReference { group: String, reference: String },
}

pub fn resolve_all_semantic_groups(
    semantic: Option<&SemanticConfig>,
    primitive_tokens: &HashMap<String, ResolvedToken>,
) -> Result<HashMap<String, ResolvedSemanticGroup>, ResolveSemanticError> {
    let Some(semantic) = semantic else {
        return Ok(HashMap::new());
    };

    // TODO: check if needs to give base capacity to HashMap
    let mut result = HashMap::new();

    for (group_name, group_cfg) in &semantic.groups {
        let mut tokens = Vec::with_capacity(group_cfg.tokens.len());
        for (token_name, reference) in &group_cfg.tokens {
            let resolved =
                resolve_semantic_reference(reference, primitive_tokens).ok_or_else(|| {
                    ResolveSemanticError::UnresolvableReference {
                        group: group_name.clone(),
                        reference: reference.clone(),
                    }
                })?;

            tokens.push((token_name.clone(), resolved));
        }

        result.insert(
            group_name.clone(),
            ResolvedSemanticGroup {
                prefix: group_name.clone(),
                property: group_cfg.property.clone(),
                tokens,
            },
        );
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{self, tempdir};

    #[test]
    fn test_scan_tokens_dir_returns_all_tokens() {
        let tokens_tmp_dir = tempdir().unwrap();

        let colors_token_path = tokens_tmp_dir.path().join("colors.json");
        fs::write(&colors_token_path, "{}").unwrap();

        let fonts_token_path = tokens_tmp_dir.path().join("fonts.json");
        fs::write(&fonts_token_path, "{}").unwrap();

        let spacings_token_path = tokens_tmp_dir.path().join("spacings.json");
        fs::write(&spacings_token_path, "{}").unwrap();

        let viewport_token_path = tokens_tmp_dir.path().join("screen_viewports.json");
        fs::write(&viewport_token_path, "{}").unwrap();

        let result = scan_tokens_dir(tokens_tmp_dir.path()).unwrap();

        assert_eq!(result.len(), 4);
        assert!(result.contains_key("colors"));
        assert!(result.get("colors").unwrap().eq(&colors_token_path));

        assert!(result.contains_key("fonts"));
        assert!(result.get("fonts").unwrap().eq(&fonts_token_path));

        assert!(result.contains_key("spacings"));
        assert!(result.get("spacings").unwrap().eq(&spacings_token_path));

        assert!(result.contains_key("screen_viewports"));
        assert!(
            result
                .get("screen_viewports")
                .unwrap()
                .eq(&viewport_token_path)
        );
    }

    #[test]
    fn test_scan_tokens_dir_returns_error_if_directory_does_not_exist() {
        let result = scan_tokens_dir(Path::new("/non-existent-directory"));

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("failed to read tokens directory")
        );
    }

    mod semantic_tokens_resolution {
        use crate::{SemanticConfig, SemanticGroupConfig};

        use super::*;

        #[test]
        fn test_resolve_reference_valid() {
            let mut primitives = HashMap::new();
            primitives.insert(
                "colors".to_string(),
                ResolvedToken {
                    tokens: vec![(
                        "blue-700".to_string(),
                        TokenValue::Simple("#3b82f6".to_string()),
                    )],
                    utilities: vec![],
                    prefix: "color".to_string(),
                },
            );
            assert_eq!(
                resolve_semantic_reference("{colors.blue-700}", &primitives),
                Some("var(--color-blue-700)".to_string())
            );
        }

        #[test]
        fn test_resolve_reference_token_does_not_exist_returns_none() {
            let mut primitives = HashMap::new();
            primitives.insert(
                "colors".to_string(),
                ResolvedToken {
                    tokens: vec![(
                        "blue-700".to_string(),
                        TokenValue::Simple("#3b82f6".to_string()),
                    )],
                    utilities: vec![],
                    prefix: "color".to_string(),
                },
            );
            assert!(resolve_semantic_reference("{colors.blue-600}", &primitives).is_none());
        }

        #[test]
        fn test_resolve_reference_missing_group_returns_none() {
            let primitives = HashMap::new();
            assert!(resolve_semantic_reference("{colors.blue-700}", &primitives).is_none());
        }

        #[test]
        fn test_resolve_reference_invalid_syntax_returns_none() {
            let mut primitives = HashMap::new();
            primitives.insert(
                "colors".to_string(),
                ResolvedToken {
                    tokens: vec![(
                        "blue-700".to_string(),
                        TokenValue::Simple("#3b82f6".to_string()),
                    )],
                    utilities: vec![],
                    prefix: "color".to_string(),
                },
            );
            assert!(resolve_semantic_reference("no-braces", &primitives).is_none());
            assert!(resolve_semantic_reference("{blue-700}", &primitives).is_none());
            assert!(resolve_semantic_reference("{}", &primitives).is_none());
        }

        #[test]
        fn test_resolve_all_semantic_groups() {
            let mut primitives = HashMap::new();
            primitives.insert(
                "colors".to_string(),
                ResolvedToken {
                    tokens: vec![(
                        "blue-700".to_string(),
                        TokenValue::Simple("#3b82f6".to_string()),
                    )],
                    utilities: vec![],
                    prefix: "color".to_string(),
                },
            );

            let semantic = SemanticConfig {
                groups: HashMap::from([(
                    "text".to_string(),
                    SemanticGroupConfig {
                        property: "color".to_string(),
                        tokens: HashMap::from([(
                            "primary".to_string(),
                            "{colors.blue-700}".to_string(),
                        )]),
                    },
                )]),
            };

            let result = resolve_all_semantic_groups(Some(&semantic), &primitives)
                .expect("expect resolve_all_semantic_groups to return a HashMap");
            let text = result
                .get("text")
                .expect("expect result to contain a text semantic group");
            assert_eq!(text.prefix, "text");
            assert_eq!(text.property, "color");
            assert_eq!(
                text.tokens[0],
                ("primary".to_string(), "var(--color-blue-700)".to_string())
            );
        }

        #[test]
        fn test_resolve_unresolvable_reference_returns_error() {
            let primitives = HashMap::new();
            let semantic = SemanticConfig {
                groups: HashMap::from([(
                    "text".to_string(),
                    SemanticGroupConfig {
                        property: "color".to_string(),
                        tokens: HashMap::from([(
                            "primary".to_string(),
                            "{missing.token}".to_string(),
                        )]),
                    },
                )]),
            };

            assert!(
                resolve_all_semantic_groups(Some(&semantic), &primitives).is_err(),
                "expect semantic group containing unknown token to return an error"
            );
        }

        #[test]
        fn test_resolve_with_no_semantic_config_returns_empty() {
            let primitives = HashMap::new();
            let result = resolve_all_semantic_groups(None, &primitives).expect("expect resolve_all_semantic_groups not to return an error when semantic groups is none");

            assert!(result.is_empty());
        }
    }
}
