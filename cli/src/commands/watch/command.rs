use config::{CONFIG_FILE_NAME, NemCssConfig};
use miette::{Diagnostic, Result};

use notify_debouncer_full::{
    DebounceEventResult, Debouncer, FileIdMap, RecommendedCache, new_debouncer_opt, notify::*,
};
use std::{path::PathBuf, time::Duration};
use thiserror::Error;

use crate::commands::watch::paths::extract_watch_dirs;

/// WatchContext represents the context for the watch command.
pub struct WatchContext {
    /// The input directory.
    pub input: PathBuf,
    /// Configuration file path.
    pub config_path: PathBuf,
    /// Parsed configuration.
    pub config: NemCssConfig,
}

#[derive(Debug, Diagnostic, Error)]
pub enum WatchContextError {
    #[error("error reading current directory: {0}")]
    ReadCurrentDirError(#[from] std::io::Error),
    #[error("error parsing nemcss configuration: {0}")]
    ParseConfigError(#[from] config::NemCssConfigError),
}

impl WatchContext {
    /// Creates a new WatchContext instance from the given input and output directories.
    pub fn new(input: PathBuf) -> Result<Self, WatchContextError> {
        let current_dir = std::env::current_dir()?;
        let config_path = current_dir.join(CONFIG_FILE_NAME);
        let config = NemCssConfig::from_path(&config_path)?;

        Ok(Self {
            input,
            config_path,
            config,
        })
    }
}

/// WatchErrors are errors that can occur during the watch command.
#[derive(Debug, Diagnostic, Error)]
pub enum WatchErrors {
    #[error("error setting up watcher: {0}")]
    SetupWatcherErrors(#[from] SetupWatcherError),
    #[error("error creating watch context: {0}")]
    CreateWatchContextError(#[from] WatchContextError),
}

pub fn watch(input: PathBuf, output: PathBuf) -> Result<(), WatchErrors> {
    let watch_context = WatchContext::new(input)?;

    let (tx, rx) = std::sync::mpsc::channel();

    let _watcher = setup_watcher(tx, watch_context)?;
    todo!()
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

/// Set up the watcher with the given channel.
/// This function also configures what events and paths to watch.
fn setup_watcher(
    tx: std::sync::mpsc::Sender<DebounceEventResult>,
    watch_context: WatchContext,
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
