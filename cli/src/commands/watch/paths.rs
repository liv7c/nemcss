//! This module contains the logic for extracting and managing paths for watching.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use notify::RecursiveMode;

/// Extracts the watch directories from the given configuration content field.
/// It returns a vector of tuples, where the first element is the directory path and the second element is the recursive mode.
pub fn extract_watch_dirs(config_content: &[String]) -> Vec<(PathBuf, RecursiveMode)> {
    let mut watch_dirs = HashSet::new();

    for pattern in config_content {
        if pattern.is_empty() {
            continue;
        }

        let is_recursive = pattern.contains("**");

        let path = Path::new(pattern);

        let mut dir_components = Vec::new();
        for component in path.components() {
            let component_str = component.as_os_str().to_string_lossy();
            if component_str.contains('*')
                || component_str.contains('?')
                || component_str.contains('{')
                || component_str.contains('[')
            {
                break;
            }
            dir_components.push(component);
        }

        let dir = if dir_components.is_empty() {
            PathBuf::from(".")
        } else {
            dir_components.iter().collect()
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
