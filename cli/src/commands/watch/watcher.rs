//! This module contains the logic for setting up the watcher.

use config::CONFIG_FILE_NAME;
use miette::{Diagnostic, Result};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use thiserror::Error;

use notify_debouncer_full::{
    DebounceEventResult, Debouncer, FileIdMap, RecommendedCache, new_debouncer_opt, notify::*,
};

use crate::commands::watch::{command::WatchContext, paths::extract_watch_dirs};

/// FileWatcher is the watcher used to watch for file changes.
/// It is a wrapper around the notify-full-debouncer Debouncer.
/// It also contains the glob set of paths to watch.
pub struct FileWatcher {
    /// The debouncer from the notify-full-debouncer crate.
    pub watcher: Debouncer<RecommendedWatcher, FileIdMap>,
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
    pub fn new(
        tx: std::sync::mpsc::Sender<DebounceEventResult>,
        watch_context: &WatchContext,
    ) -> Result<Self, SetupWatcherError> {
        Ok(Self {
            watcher: Self::create_debounced_watcher(tx, watch_context)?,
        })
    }

    /// Resets the watcher by creating a new debounced watcher with the updated watch context.
    /// This is used when the configuration file changes and we need to update the watch paths.
    pub fn reset(
        &mut self,
        tx: std::sync::mpsc::Sender<DebounceEventResult>,
        watch_context: &WatchContext,
    ) -> Result<(), SetupWatcherError> {
        self.watcher = Self::create_debounced_watcher(tx, watch_context)?;
        Ok(())
    }

    /// Filters out events based on the event kind and the glob set in the watch context.
    pub fn filter_events(
        result: DebounceEventResult,
        watch_context: &WatchContext,
    ) -> Result<Vec<PathBuf>, FilterEventsError> {
        match result {
            Ok(events) => {
                use std::collections::HashSet;

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
    ) -> Result<Debouncer<RecommendedWatcher, FileIdMap>, SetupWatcherError> {
        let mut watcher = new_debouncer_opt::<_, _, FileIdMap>(
            Duration::from_secs(2),
            None,
            tx,
            RecommendedCache::default(),
            Config::default(),
        )
        .map_err(SetupWatcherError::CreateWatcherError)?;

        let watch_dirs = extract_watch_dirs(&watch_context.config.content);

        for (dir, mode) in watch_dirs {
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
            .watch(&watch_context.config.tokens_dir, RecursiveMode::Recursive)
            .map_err(|e| SetupWatcherError::WatchDirectoryError {
                path: watch_context.config.tokens_dir.display().to_string(),
                source: e,
            })?;

        Ok(watcher)
    }
}

/// Checks if a path is relevant for the watch context.
///
/// Relevant paths are:
/// - The config file
/// - The tokens directory
/// - Any path in the config content glob set
fn is_relevant_path(path: &Path, watch_context: &WatchContext) -> bool {
    let matches_config_content_glob = path
        .strip_prefix(&watch_context.config.base_dir)
        .is_ok_and(|relative_path| watch_context.glob_set.is_match(relative_path));
    let is_config_file = path.ends_with(CONFIG_FILE_NAME);
    let is_in_tokens_dir = path.starts_with(watch_context.config.tokens_dir());

    matches_config_content_glob || is_config_file || is_in_tokens_dir
}
