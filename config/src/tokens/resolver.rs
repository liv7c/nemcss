use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use miette::{Diagnostic, Result};
use thiserror::Error;

use crate::tokens::token::{TokenFile, TokenValue};
use crate::tokens::utilities::{default_prefix_for_token_type, get_utilities_for_token_type};
use crate::{NemCSSConfig, TokenUtilityConfig};

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
) -> Result<HashMap<String, TokenValue>, LoadTokensFromFileError> {
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
    pub tokens: HashMap<String, TokenValue>,
    /// The utilities for the token.
    pub utilities: Vec<TokenUtilityConfig>,
    /// The prefix for the token.
    pub prefix: String,
}

/// Resolve all tokens, based on auto-discovered token files in the tokens directory and the theme
/// configuration.
pub fn resolve_all_tokens(
    config: &NemCSSConfig,
) -> Result<HashMap<String, ResolvedToken>, ResolveTokensError> {
    let mut resolved_tokens = HashMap::new();

    let tokens_dir = config.base_dir.join(&config.tokens_dir);
    let token_files = scan_tokens_dir(&tokens_dir)?;

    for (name, path) in token_files {
        let prefix = default_prefix_for_token_type(&name);
        let tokens = load_tokens_from_file(&path)?;
        let utilities = get_utilities_for_token_type(&name, config.theme.as_ref());

        resolved_tokens.insert(
            name,
            ResolvedToken {
                tokens,
                utilities,
                prefix,
            },
        );
    }

    Ok(resolved_tokens)
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
}
