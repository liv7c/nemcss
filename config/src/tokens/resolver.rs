use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use miette::{Diagnostic, Result};
use thiserror::Error;

/// Represents the error type when scanning the tokens directory.
#[derive(Debug, Diagnostic, Error)]
enum ScanTokensDirError {
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
