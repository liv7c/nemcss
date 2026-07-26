use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use miette::{Diagnostic, Result};
use thiserror::Error;

use crate::tokens::token::{TokenFile, TokenValue};
use crate::{ModeConfig, NemCssConfig, SemanticConfig, TokenUtilityConfig};

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
fn scan_tokens_dir(path: &Path) -> Result<Vec<PathBuf>, ScanTokensDirError> {
    let mut tokens = Vec::new();

    let entries = fs::read_dir(path).map_err(ScanTokensDirError::ReadDirError)?;

    for entry in entries {
        let entry = entry.map_err(ScanTokensDirError::FileReadError)?;
        let path = entry.path();

        // Skip non-JSON files
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        tokens.push(path);
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

    #[error("token file not found for theme entry `{token_name}`: {}", source_path.display())]
    #[diagnostic(
        code(config::tokens::resolve::source_not_found),
        help("check the `source` path for this entry in nemcss.config.json")
    )]
    SourceFileNotFound {
        token_name: String,
        source_path: PathBuf,
    },

    #[error("token file(s) not registered in nemcss.config.json: {}", paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", "))]
    #[diagnostic(
        code(config::tokens::resolve::unregistered_token_file),
        help(
            "add a theme entry with `source` pointing to these files and a `prefix`, or remove these files"
        )
    )]
    UnregisteredTokenFile { paths: Vec<PathBuf> },
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

/// Resolve all tokens registered in the theme configuration.
/// For each entry under `theme` in `nemcss.config.json`, loads and parses its `source` file.
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
pub fn resolve_registered_tokens(
    config: &NemCssConfig,
) -> Result<HashMap<String, ResolvedToken>, ResolveTokensError> {
    let mut resolved_tokens = HashMap::new();

    if let Some(theme) = config.theme.as_ref() {
        for (name, token_config) in &theme.tokens {
            let path = config.base_dir.join(&token_config.source);
            if !path.is_file() {
                return Err(ResolveTokensError::SourceFileNotFound {
                    token_name: name.clone(),
                    source_path: token_config.source.clone(),
                });
            }

            let tokens = load_tokens_from_file(&path)?;
            let utilities = token_config.utilities.clone().unwrap_or_default();

            resolved_tokens.insert(
                name.clone(),
                ResolvedToken {
                    tokens,
                    utilities,
                    prefix: token_config.prefix.clone(),
                },
            );
        }
    }

    Ok(resolved_tokens)
}

/// Checks for unregistered token files by comparing the token files registered in the config file with the
/// token files inside of the design tokens directory.
/// It returns a sorted list of unregistered token files.
pub fn unregistered_token_files(config: &NemCssConfig) -> Result<Vec<PathBuf>, ScanTokensDirError> {
    let tokens_dir = config.tokens_dir();
    if !tokens_dir.is_dir() {
        return Ok(Vec::new());
    }

    let registered_sources: HashSet<PathBuf> = config
        .theme
        .as_ref()
        .map(|theme| {
            theme
                .tokens
                .values()
                .map(|cfg| config.base_dir.join(&cfg.source))
                .collect()
        })
        .unwrap_or_default();

    let mut unregistered: Vec<PathBuf> = scan_tokens_dir(&tokens_dir)?
        .into_iter()
        .filter(|path| !registered_sources.contains(path))
        .map(|path| path.to_path_buf())
        .collect();

    unregistered.sort();
    Ok(unregistered)
}

/// Resolves all tokens, erroring out if there are unregistered token files in the design-token directory.
/// An unregistered token file is a file that exists in the design-tokens dir without being registered inside `theme` in `nemcss.config.json`
pub fn resolve_all_tokens(
    config: &NemCssConfig,
) -> Result<HashMap<String, ResolvedToken>, ResolveTokensError> {
    let resolved = resolve_registered_tokens(config)?;

    let unregistered = unregistered_token_files(config)?;

    if !unregistered.is_empty() {
        return Err(ResolveTokensError::UnregisteredTokenFile {
            paths: unregistered,
        });
    }

    Ok(resolved)
}

/// A resolved semantic group for CSS generation.
/// It contains the group name, the CSS property for CSS generation as well as a vector
/// of tuples containing both the semantic name and resolved var pairs for a given semantic group
#[derive(Debug, PartialEq, Default)]
pub struct ResolvedSemanticGroup {
    /// Group name used for both CSS variable prefix and utility class prefix (e.g. "text-", "--text-*")
    pub prefix: String,
    /// CSS property for utility classes (e.g. "background-color")
    pub property: Option<String>,
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

    if !resolved.tokens.iter().any(|(name, _)| name == token_name) {
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

/// Resolved mode, ready for CSS generation
#[derive(Debug, PartialEq)]
pub struct ResolvedMode {
    /// Mode name from the config (e.g. "dark")
    pub name: String,
    /// Selector that activates the mode, defaulted to `[data-mode="<name>"]`
    pub selector: String,
    /// Optional media query
    pub media: Option<String>,
    /// Pairs of semantic tokens - primitive token sorted by property name
    /// e.g. ("--text-default", "var(--color-gray-100)")
    pub declarations: Vec<(String, String)>,
}

#[derive(Debug, Diagnostic, Error)]
pub enum ResolveModeError {
    #[error("mode '{mode}' declares both \"selector\" and \"media\"")]
    #[diagnostic(
        code(config::modes::conflicting_activation),
        help(
            "a mode is activated either by a selector or by a media query, not both: remove one of \"{selector}\" or \"{media}\""
        )
    )]
    ConflictingActivation {
        mode: String,
        selector: String,
        media: String,
    },

    #[error("mode '{mode}' overrides unknown semantic group '{group}'")]
    #[diagnostic(
        code(config::modes::unknown_group),
        help("groups in a mode overrides must be declared in the \"semantic\" section")
    )]
    UnknownSemanticGroup { mode: String, group: String },

    #[error("mode '{mode}' overrides unknown token '{token}' in semantic group '{group}'")]
    #[diagnostic(
        code(config::modes::unknown_token),
        help("available tokens: {available}")
    )]
    UnknownSemanticToken {
        mode: String,
        group: String,
        token: String,
        available: String,
    },

    #[error("unresolvable reference '{reference}' in mode '{mode}', group '{group}'")]
    #[diagnostic(code(config::modes::unresolvable_reference))]
    UnresolvableReference {
        mode: String,
        group: String,
        reference: String,
    },
}

pub fn resolve_all_modes(
    modes: Option<&HashMap<String, ModeConfig>>,
    semantic_groups: &HashMap<String, ResolvedSemanticGroup>,
    primitive_tokens: &HashMap<String, ResolvedToken>,
) -> Result<Vec<ResolvedMode>, ResolveModeError> {
    let Some(modes) = modes else {
        return Ok(vec![]);
    };

    let mut mode_names: Vec<&String> = modes.keys().collect();
    // sort modes by alphabetic order
    mode_names.sort();

    let mut resolved_modes = Vec::with_capacity(mode_names.len());

    for name in mode_names {
        let definition = &modes[name];

        if let (Some(selector), Some(media)) = (&definition.selector, &definition.media) {
            return Err(ResolveModeError::ConflictingActivation {
                mode: name.clone(),
                selector: selector.clone(),
                media: media.clone(),
            });
        }

        let mut declarations = Vec::new();

        for (group_name, overrides) in &definition.overrides {
            // make sure the group exists in the semantic groups first
            let group = semantic_groups.get(group_name).ok_or_else(|| {
                ResolveModeError::UnknownSemanticGroup {
                    mode: name.clone(),
                    group: group_name.clone(),
                }
            })?;

            for (token_name, reference) in overrides {
                if !group.tokens.iter().any(|(t, _)| t == token_name) {
                    let mut available: Vec<&str> =
                        group.tokens.iter().map(|(t, _)| t.as_str()).collect();
                    available.sort();

                    return Err(ResolveModeError::UnknownSemanticToken {
                        mode: name.clone(),
                        group: group_name.clone(),
                        token: token_name.clone(),
                        available: available.join(", "),
                    });
                }

                let resolved =
                    resolve_semantic_reference(reference, primitive_tokens).ok_or_else(|| {
                        ResolveModeError::UnresolvableReference {
                            mode: name.clone(),
                            group: group_name.clone(),
                            reference: reference.clone(),
                        }
                    })?;

                declarations.push((format!("--{}-{}", group.prefix, token_name), resolved));
            }
        }

        declarations.sort_by(|a, b| a.0.cmp(&b.0));

        resolved_modes.push(ResolvedMode {
            name: name.clone(),
            selector: definition
                .selector
                .clone()
                .unwrap_or_else(|| format!("[data-mode=\"{name}\"]")),
            media: definition.media.clone(),
            declarations,
        })
    }

    Ok(resolved_modes)
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

        let mut result = scan_tokens_dir(tokens_tmp_dir.path()).unwrap();
        result.sort();

        let mut expected = vec![
            colors_token_path,
            fonts_token_path,
            spacings_token_path,
            viewport_token_path,
        ];
        expected.sort();

        assert_eq!(result, expected);
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
                        property: Some("color".to_string()),
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
            assert_eq!(text.property, Some("color".to_string()));
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
                        property: Some("color".to_string()),
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

    mod mode_resolution {
        use super::*;
        use crate::config::ModeConfig;

        fn primitives() -> HashMap<String, ResolvedToken> {
            let mut map = HashMap::new();

            map.insert(
                "colors".to_string(),
                ResolvedToken {
                    tokens: vec![
                        (
                            "black".to_string(),
                            TokenValue::Simple("hsl(0deg 0% 0%)".to_string()),
                        ),
                        (
                            "gray-100".to_string(),
                            TokenValue::Simple("hsl(0deg 0% 95%)".to_string()),
                        ),
                    ],
                    utilities: vec![],
                    prefix: "color".to_string(),
                },
            );

            map
        }

        fn semantic_groups() -> HashMap<String, ResolvedSemanticGroup> {
            let mut map = HashMap::new();

            map.insert(
                "text".to_string(),
                ResolvedSemanticGroup {
                    prefix: "text".to_string(),
                    property: Some("color".to_string()),
                    tokens: vec![
                        ("default".to_string(), "var(--color-black)".to_string()),
                        ("muted".to_string(), "var(--color-gray-100)".to_string()),
                    ],
                },
            );

            map
        }

        fn dark_mode(overrides: &[(&str, &[(&str, &str)])]) -> HashMap<String, ModeConfig> {
            let mut map = HashMap::new();

            map.insert(
                "dark".to_string(),
                ModeConfig {
                    selector: None,
                    media: None,
                    overrides: overrides
                        .iter()
                        .map(|(group, tokens)| {
                            (
                                group.to_string(),
                                tokens
                                    .iter()
                                    .map(|(k, v)| (k.to_string(), v.to_string()))
                                    .collect(),
                            )
                        })
                        .collect(),
                },
            );

            map
        }

        #[test]
        fn resolves_a_mode_with_defaulted_selector() {
            let modes = dark_mode(&[("text", &[("default", "{colors.gray-100}")])]);

            let resolved =
                resolve_all_modes(Some(&modes), &semantic_groups(), &primitives()).unwrap();

            assert_eq!(resolved.len(), 1);
            let dark = &resolved[0];
            assert_eq!(dark.name, "dark");
            assert_eq!(dark.selector, r#"[data-mode="dark"]"#);
            assert_eq!(dark.media, None);
            assert_eq!(
                dark.declarations,
                vec![(
                    "--text-default".to_string(),
                    "var(--color-gray-100)".to_string()
                )]
            );
        }

        #[test]
        fn no_modes_resolves_to_empty_vec() {
            let resolved = resolve_all_modes(None, &semantic_groups(), &primitives()).unwrap();

            assert!(resolved.is_empty());
        }

        #[test]
        fn overriding_an_unknown_group_is_an_error() {
            let modes = dark_mode(&[("surface", &[("page", "{colors.black}")])]);

            let err =
                resolve_all_modes(Some(&modes), &semantic_groups(), &primitives()).unwrap_err();

            assert!(matches!(
                err,
                ResolveModeError::UnknownSemanticGroup{ ref mode, ref group, .. }
                if mode == "dark" && group == "surface"
            ))
        }

        #[test]
        fn overriding_an_unknown_token_is_an_error() {
            // type in 'default' token
            let modes = dark_mode(&[("text", &[("deafult", "{colors.gray-100}")])]);

            let err =
                resolve_all_modes(Some(&modes), &semantic_groups(), &primitives()).unwrap_err();

            assert!(matches!(
                err,
                ResolveModeError::UnknownSemanticToken { ref mode, ref group, ref token, .. }
                if mode == "dark" && group == "text" && token == "deafult"
            ))
        }

        #[test]
        fn unresolvable_reference_is_an_error() {
            // reference a color token that does not exist
            let modes = dark_mode(&[("text", &[("default", "{colors.gray-99999}")])]);

            let err =
                resolve_all_modes(Some(&modes), &semantic_groups(), &primitives()).unwrap_err();

            assert!(matches!(
                err,
                ResolveModeError::UnresolvableReference { ref mode, ref reference, ..}
                if mode == "dark" && reference == "{colors.gray-99999}"
            ))
        }

        #[test]
        fn selector_and_media_together_is_an_error() {
            let mut modes = HashMap::new();
            modes.insert(
                "dark".to_string(),
                ModeConfig {
                    selector: Some("[data-theme='dark']".to_string()),
                    media: Some("(prefers-color-scheme: dark)".to_string()),
                    overrides: HashMap::from([(
                        "text".to_string(),
                        HashMap::from([("default".to_string(), "{colors.gray-100}".to_string())]),
                    )]),
                },
            );

            let result =
                resolve_all_modes(Some(&modes), &semantic_groups(), &primitives()).unwrap_err();

            assert!(matches!(
                result,
                ResolveModeError::ConflictingActivation { .. }
            ));
        }
    }
}
