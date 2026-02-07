//! This module contains the logic for extracting and managing paths for watching.

use std::{collections::HashSet, path::PathBuf};

use notify::RecursiveMode;

/// Extracts the watch directories from the given configuration content field.
/// It returns a vector of tuples, where the first element is the directory path and the second element is the recursive mode.
pub fn extract_watch_dirs(config_content: &[String]) -> Vec<(PathBuf, RecursiveMode)> {
    let mut watch_dirs = HashSet::new();

    for pattern in config_content {
        let is_recursive = pattern.contains("**");

        let parts: Vec<_> = pattern.split('/').collect();

        let first_glob_idx = parts
            .iter()
            .position(|p| p.contains('*') || p.contains('?') || p.contains('{') || p.contains('['))
            .unwrap_or(parts.len());

        let dir_parts = &parts[..first_glob_idx];

        let dir = if dir_parts.is_empty() {
            PathBuf::from(".")
        } else {
            dir_parts.iter().collect::<PathBuf>()
        };

        let mode = if is_recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        watch_dirs.insert((dir, mode));
    }

    watch_dirs.into_iter().collect()
}

/// Builds a glob set from the given configuration content field.
pub fn build_glob_set(content: &[String]) -> Result<globset::GlobSet, globset::Error> {
    let mut builder = globset::GlobSetBuilder::new();
    for pattern in content {
        builder.add(globset::Glob::new(pattern)?);
    }
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_watch_dirs() {
        let config_content = vec![
            "src/**/*.html".to_string(),
            "src/**/*.tsx".to_string(),
            "lib/**/*.tsx".to_string(),
            "a/b/**/*.tsx".to_string(),
            "source.json".to_string(),
        ];

        let result = extract_watch_dirs(&config_content);
        assert_eq!(result.len(), 4);
        assert!(
            result.contains(&(PathBuf::from("src"), RecursiveMode::Recursive)),
            "expected src to be in the result"
        );
        assert!(
            result.contains(&(PathBuf::from("lib"), RecursiveMode::Recursive)),
            "expected lib to be in the result"
        );
        assert!(
            result.contains(&(PathBuf::from("a/b"), RecursiveMode::Recursive)),
            "expected a/b to be in the result"
        );
        assert!(result.contains(&(PathBuf::from("source.json"), RecursiveMode::NonRecursive)));
    }
}
