//! This module contains the logic for setting up the watcher.

use config::CONFIG_FILE_NAME;
use miette::{Diagnostic, Result};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::Duration,
};
use thiserror::Error;

use notify_debouncer_full::{
    DebounceEventResult, Debouncer, RecommendedCache, new_debouncer_opt, notify::*,
};

use crate::commands::watch::{command::WatchContext, paths::extract_watch_dirs};

/// Debounce time for the debouncer
const DEBOUNCE_TIME: u64 = 500;

/// FileWatcher is the watcher used to watch for file changes.
/// It is a wrapper around the notify-full-debouncer Debouncer.
/// It also contains the glob set of paths to watch.
pub struct FileWatcher {
    /// The debouncer from the notify-full-debouncer crate.
    pub watcher: Debouncer<RecommendedWatcher, RecommendedCache>,
}

#[derive(Debug, Diagnostic, Error)]
pub enum SetupWatcherError {
    #[error("error creating watcher: {0}")]
    #[diagnostic(code(notify_debouncer_full::notify::Error))]
    CreateWatcherError(notify::Error),
    #[error("error watching path: {path:?} - {source:?}")]
    #[diagnostic(code(notify_debouncer_full::notify::Error))]
    WatchDirectoryError { path: String, source: notify::Error },
}

#[derive(Debug, Diagnostic, Error)]
pub enum FilterEventsError {
    #[error("error receiving event: {0:?}")]
    #[diagnostic(code(notify_debouncer_full::notify::Error))]
    ReceiveEvent(Vec<notify::Error>),
}

impl FileWatcher {
    /// Creates a new FileWatcher.
    ///
    /// Returns the watcher along with any content directories from the configuration
    /// that do not exist on disk yet and were therefore skipped.
    pub fn new(
        tx: std::sync::mpsc::Sender<DebounceEventResult>,
        watch_context: &WatchContext,
    ) -> Result<(Self, Vec<PathBuf>), SetupWatcherError> {
        let (watcher, skipped_dirs) = Self::create_debounced_watcher(tx, watch_context)?;

        Ok((Self { watcher }, skipped_dirs))
    }

    /// Resets the watcher by creating a new debounced watcher with the updated watch context.
    /// This is used when the configuration file changes and we need to update the watch paths.
    ///
    /// Returns any content directories from the configuration that
    /// do not exist on disk yet and were therefore skipped.
    pub fn reset(
        &mut self,
        tx: std::sync::mpsc::Sender<DebounceEventResult>,
        watch_context: &WatchContext,
    ) -> Result<Vec<PathBuf>, SetupWatcherError> {
        let (watcher, skipped_dirs) = Self::create_debounced_watcher(tx, watch_context)?;
        self.watcher = watcher;
        Ok(skipped_dirs)
    }

    /// Filters out events based on the event kind and the glob set in the watch context.
    pub fn filter_events(
        result: DebounceEventResult,
        watch_context: &WatchContext,
    ) -> Result<Vec<PathBuf>, FilterEventsError> {
        match result {
            Ok(events) => {
                // Use a HashSet to deduplicate paths; note the resulting Vec has no
                // guaranteed order since HashSet iteration order is non-deterministic.
                let files: HashSet<PathBuf> = events
                    .iter()
                    .filter(|event| {
                        matches!(
                            event.kind,
                            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                        )
                    })
                    .flat_map(|event| &event.paths)
                    .filter(|path| is_relevant_path(path, watch_context))
                    .cloned()
                    .collect();

                Ok(files.into_iter().collect())
            }
            Err(error) => Err(FilterEventsError::ReceiveEvent(error)),
        }
    }

    /// Internal function to create the debounced watcher.
    fn create_debounced_watcher(
        tx: std::sync::mpsc::Sender<DebounceEventResult>,
        watch_context: &WatchContext,
    ) -> Result<
        (
            Debouncer<RecommendedWatcher, RecommendedCache>,
            Vec<PathBuf>,
        ),
        SetupWatcherError,
    > {
        let mut watcher = new_debouncer_opt::<_, _, RecommendedCache>(
            Duration::from_millis(DEBOUNCE_TIME),
            None,
            tx,
            RecommendedCache::default(),
            Config::default(),
        )
        .map_err(SetupWatcherError::CreateWatcherError)?;

        let watch_dirs = extract_watch_dirs(&watch_context.config.content);
        let mut skipped_dirs = Vec::new();

        for (dir, mode) in watch_dirs {
            if !dir.exists() {
                skipped_dirs.push(dir);
                continue;
            }

            watcher
                .watch(&dir, mode)
                .map_err(|e| SetupWatcherError::WatchDirectoryError {
                    path: dir.display().to_string(),
                    source: e,
                })?;
        }

        // Watch the config file for changes.
        watcher
            .watch(&watch_context.config_path, RecursiveMode::NonRecursive)
            .map_err(|e| SetupWatcherError::WatchDirectoryError {
                path: watch_context.config_path.display().to_string(),
                source: e,
            })?;

        // Watch for input file changes.
        watcher
            .watch(&watch_context.input, RecursiveMode::NonRecursive)
            .map_err(|e| SetupWatcherError::WatchDirectoryError {
                path: watch_context.input.display().to_string(),
                source: e,
            })?;

        // Watch for token directory changes.
        watcher
            .watch(watch_context.config.tokens_dir(), RecursiveMode::Recursive)
            .map_err(|e| SetupWatcherError::WatchDirectoryError {
                path: watch_context.config.tokens_dir().display().to_string(),
                source: e,
            })?;

        Ok((watcher, skipped_dirs))
    }
}

/// Checks if a path is relevant for the watch context.
///
/// Relevant paths are:
/// - the CSS input file
/// - The config file
/// - The tokens directory
/// - Any path in the config content glob set
fn is_relevant_path(path: &Path, watch_context: &WatchContext) -> bool {
    let matches_config_content_glob = path
        .strip_prefix(&watch_context.config.base_dir)
        .ok()
        .map(|relative_path| watch_context.glob_set.is_match(relative_path))
        .unwrap_or(false);
    let is_config_file = path.ends_with(CONFIG_FILE_NAME);
    let is_in_tokens_dir = path.starts_with(watch_context.config.tokens_dir());
    let is_input_file = path == watch_context.input;

    is_input_file || matches_config_content_glob || is_config_file || is_in_tokens_dir
}

#[cfg(test)]
mod tests {
    use super::*;

    mod is_relevant_path {
        use super::*;
        use crate::commands::watch::command::WatchContext;
        use config::NemCssConfig;

        fn make_context(input: PathBuf) -> WatchContext {
            let mut config: NemCssConfig =
                serde_json::from_str(r#"{ "content": ["src/**/*.html"] }"#).unwrap();
            config.base_dir = PathBuf::from("/project");
            let glob_set = config.content_glob_set().unwrap();

            WatchContext {
                input,
                output: PathBuf::from("/project/dist/styles.css"),
                config_path: PathBuf::from("/project/nemcss.config.json"),
                config,
                glob_set,
            }
        }

        #[test]
        fn css_input_file_change_is_relevant() {
            let ctx = make_context(PathBuf::from("/project/styles.css"));

            assert!(is_relevant_path(Path::new("/project/styles.css"), &ctx));
        }

        #[test]
        fn unrelated_file_is_not_relevant() {
            let ctx = make_context(PathBuf::from("/project/styles.css"));

            assert!(!is_relevant_path(Path::new("/project/other.css"), &ctx));
        }
    }
}
