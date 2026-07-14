//! This module contains the logic to get the content files matching a globset.
//! It uses the `ignore` crate to walk the directory efficiently.
use std::path::{Path, PathBuf};

use globset::GlobSet;
use ignore::Walk;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum GetContentFilesError {
    #[error("failed to walk the directory: {0}")]
    #[diagnostic(code(nemcss::build::glob::walk))]
    Walk(#[from] ignore::Error),

    #[error("failed to get relative path of the file: {0}")]
    #[diagnostic(code(nemcss::build::glob::strip_prefix))]
    StripPrefix(#[from] std::path::StripPrefixError),
}

/// Retrieves all files matching the given globset.
///
/// The globset must be built from `NemCssConfig::content_glob_set` so that patterns are
/// normalized consistently with the rest of NemCSS (e.g. a leading `./` is stripped).
///
/// This function uses the `ignore` crate to walk the directory efficiently.
pub fn get_content_files(
    globset: &GlobSet,
    base_dir: &Path,
) -> miette::Result<Vec<PathBuf>, GetContentFilesError> {
    let mut files = Vec::new();

    for result in Walk::new(base_dir) {
        let entry = result?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let relative_path = path.strip_prefix(base_dir)?;
        if globset.is_match(relative_path) {
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use config::NemCssConfig;

    /// Builds a globset the same way the build command does: through
    /// `NemCssConfig::content_glob_set`, so tests exercise the exact pattern
    /// normalization (e.g. a leading `./`) that production code relies on.
    fn glob_set(patterns: &[&str]) -> GlobSet {
        let config: NemCssConfig = serde_json::from_str(
            &serde_json::json!({
                "content": patterns,
            })
            .to_string(),
        )
        .unwrap();
        config.content_glob_set().unwrap()
    }

    #[test]
    fn test_get_content_files_single_pattern() {
        let temp_dir = TempDir::new().unwrap();
        temp_dir.child("src/page.astro").touch().unwrap();
        temp_dir
            .child("src/components/button.astro")
            .touch()
            .unwrap();
        temp_dir.child("src/components/styles.css").touch().unwrap();
        temp_dir
            .child("src/components/analytics.ts")
            .touch()
            .unwrap();

        let globset = glob_set(&["src/**/*.astro"]);

        let files = get_content_files(&globset, temp_dir.path()).unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.contains(&temp_dir.child("src/page.astro").path().to_path_buf()));
        assert!(
            files.contains(
                &temp_dir
                    .child("src/components/button.astro")
                    .path()
                    .to_path_buf()
            )
        );
    }

    #[test]
    fn test_get_content_files_multiple_patterns() {
        let temp_dir = TempDir::new().unwrap();
        temp_dir.child("src/page.astro").touch().unwrap();
        temp_dir
            .child("src/components/button.astro")
            .touch()
            .unwrap();
        temp_dir.child("src/components/styles.css").touch().unwrap();
        temp_dir
            .child("src/components/analytics.ts")
            .touch()
            .unwrap();

        let globset = glob_set(&["src/**/*.astro", "src/**/*.css"]);

        let files = get_content_files(&globset, temp_dir.path()).unwrap();

        assert_eq!(files.len(), 3);
        assert!(files.contains(&temp_dir.child("src/page.astro").path().to_path_buf()));
        assert!(
            files.contains(
                &temp_dir
                    .child("src/components/button.astro")
                    .path()
                    .to_path_buf()
            )
        );
        assert!(
            files.contains(
                &temp_dir
                    .child("src/components/styles.css")
                    .path()
                    .to_path_buf()
            )
        );
    }

    #[test]
    fn test_get_content_files_with_globstar() {
        let temp_dir = TempDir::new().unwrap();
        temp_dir.child("src/page.astro").touch().unwrap();
        temp_dir
            .child("src/components/button.astro")
            .touch()
            .unwrap();
        temp_dir.child("src/components/styles.css").touch().unwrap();
        temp_dir
            .child("src/components/analytics.ts")
            .touch()
            .unwrap();

        let globset = glob_set(&["src/**/*.*"]);

        let files = get_content_files(&globset, temp_dir.path()).unwrap();

        assert_eq!(files.len(), 4);
        assert!(files.contains(&temp_dir.child("src/page.astro").path().to_path_buf()));
        assert!(
            files.contains(
                &temp_dir
                    .child("src/components/button.astro")
                    .path()
                    .to_path_buf()
            )
        );
        assert!(
            files.contains(
                &temp_dir
                    .child("src/components/styles.css")
                    .path()
                    .to_path_buf()
            )
        );
        assert!(
            files.contains(
                &temp_dir
                    .child("src/components/analytics.ts")
                    .path()
                    .to_path_buf()
            )
        );
    }

    #[test]
    fn test_respects_ignore_file() {
        let temp_dir = TempDir::new().unwrap();
        temp_dir.child(".ignore").write_str("dist/*.*\n").unwrap();
        temp_dir.child("src/page.astro").touch().unwrap();
        temp_dir.child("dist/page.html").touch().unwrap();

        let globset = glob_set(&["**/*.astro", "**/*.html"]);

        let files = get_content_files(&globset, temp_dir.path()).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files.contains(&temp_dir.child("src/page.astro").path().to_path_buf()));
    }

    #[test]
    fn test_get_content_files_strips_leading_dot_slash() {
        // Regression test: `get_content_files` used to build its own globset directly
        // from the raw patterns, bypassing the `./` normalization that
        // `NemCssConfig::content_glob_set` applies. A pattern like "./*.html" silently
        // matched nothing, since walked paths are relative (no leading "./").
        let temp_dir = TempDir::new().unwrap();
        temp_dir.child("index.html").touch().unwrap();

        let globset = glob_set(&["./*.html"]);

        let files = get_content_files(&globset, temp_dir.path()).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files.contains(&temp_dir.child("index.html").path().to_path_buf()));
    }
}
