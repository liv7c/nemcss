use config::{CONFIG_FILE_NAME, NemCssConfig};
use miette::{Diagnostic, Result};

use std::path::PathBuf;
use thiserror::Error;

use crate::commands::watch::{
    paths::build_glob_set,
    watcher::{FileWatcher, SetupWatcherError},
};

/// WatchContext represents the context for the watch command.
pub struct WatchContext {
    /// The input directory.
    pub input: PathBuf,
    /// Configuration file path.
    pub config_path: PathBuf,
    /// Parsed configuration.
    pub config: NemCssConfig,
    /// Glob set of paths to watch
    pub glob_set: globset::GlobSet,
}

#[derive(Debug, Diagnostic, Error)]
pub enum WatchContextError {
    #[error("error reading current directory: {0}")]
    ReadCurrentDir(#[from] std::io::Error),
    #[error("error parsing nemcss configuration: {0}")]
    ParseConfig(#[from] config::NemCssConfigError),
    #[error("error building glob set: {0}")]
    BuildGlobSet(#[from] globset::Error),
}

impl WatchContext {
    /// Creates a new WatchContext instance from the given input and output directories.
    pub fn new(input: PathBuf) -> Result<Self, WatchContextError> {
        let current_dir = std::env::current_dir()?;
        let config_path = current_dir.join(CONFIG_FILE_NAME);
        let config = NemCssConfig::from_path(&config_path)?;

        let glob_set = build_glob_set(&config.content)?;

        Ok(Self {
            input,
            config_path,
            config,
            glob_set,
        })
    }
}

/// WatchErrors are errors that can occur during the watch command.
#[derive(Debug, Diagnostic, Error)]
pub enum WatchError {
    #[error("error setting up watcher: {0}")]
    SetupWatcher(#[from] SetupWatcherError),
    #[error("error creating watch context: {0}")]
    CreateWatchContext(#[from] WatchContextError),
}

pub fn watch(input: PathBuf, output: PathBuf) -> Result<(), WatchError> {
    let watch_context = WatchContext::new(input)?;

    let (tx, rx) = std::sync::mpsc::channel();

    let mut _watcher = FileWatcher::new(tx, &watch_context)?;

    loop {
        match rx.recv() {
            Ok(event) => println!("{event:?}"),
            Err(err) => println!("{err:?}"),
        }
    }
}
