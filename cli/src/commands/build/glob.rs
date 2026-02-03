//! This module contains the logic to get the content files from the given patterns.
//! It uses the `globset` crate to build the globset from the patterns and the `ignore` crate to
//! optimize the file walking.
use std::path::{Path, PathBuf};

use globset::{Glob, GlobSetBuilder};
use ignore::Walk;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum GetContentFilesError {
    #[error("failed to build the globset from the content patterns: {0}")]
    #[diagnostic(code(nemcss::build::glob::glob))]
    Glob(#[from] globset::Error),

    #[error("failed to walk the directory: {0}")]
    #[diagnostic(code(nemcss::build::glob::walk))]
    Walk(#[from] ignore::Error),

    #[error("failed to get relative path of the file: {0}")]
    #[diagnostic(code(nemcss::build::glob::strip_prefix))]
    StripPrefix(#[from] std::path::StripPrefixError),
}

pub fn get_content_files(
    patterns: &[String],
    base_dir: &Path,
) -> miette::Result<Vec<PathBuf>, GetContentFilesError> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    let globset = builder.build()?;

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

        let patterns = vec!["src/**/*.astro".to_string()];

        let files = get_content_files(&patterns, temp_dir.path()).unwrap();

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

        let patterns = vec!["src/**/*.astro".to_string(), "src/**/*.css".to_string()];

        let files = get_content_files(&patterns, temp_dir.path()).unwrap();

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

        let patterns = vec!["src/**/*.*".to_string()];

        let files = get_content_files(&patterns, temp_dir.path()).unwrap();

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

        let patterns = vec!["**/*.astro".to_string(), "**/*.html".to_string()];

        let files = get_content_files(&patterns, temp_dir.path()).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files.contains(&temp_dir.child("src/page.astro").path().to_path_buf()));
    }
}
