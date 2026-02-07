use config::{CONFIG_FILE_NAME, NemCssConfig};
use miette::{Diagnostic, Result};

use owo_colors::OwoColorize;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::RecvTimeoutError,
    },
    time::Duration,
};
use thiserror::Error;

use crate::commands::{
    build::{BuildError, build},
    watch::{
        paths::build_glob_set,
        watcher::{FileWatcher, FilterEventsError, SetupWatcherError},
    },
};

/// WatchContext represents the context for the watch command.
pub struct WatchContext {
    /// The input file
    pub input: PathBuf,
    /// The output file
    pub output: PathBuf,
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
    pub fn new(input: PathBuf, output: PathBuf) -> Result<Self, WatchContextError> {
        let current_dir = std::env::current_dir()?;
        let config_path = current_dir.join(CONFIG_FILE_NAME);
        let config = NemCssConfig::from_path(&config_path)?;

        let glob_set = build_glob_set(&config.content)?;

        Ok(Self {
            input,
            output,
            config_path,
            config,
            glob_set,
        })
    }

    pub fn reload(&mut self) -> Result<(), WatchContextError> {
        let config = NemCssConfig::from_path(&self.config_path)?;
        let glob_set = build_glob_set(&config.content)?;
        self.config = config;
        self.glob_set = glob_set;
        Ok(())
    }
}

/// WatchErrors are errors that can occur during the watch command.
#[derive(Debug, Diagnostic, Error)]
pub enum WatchError {
    #[error("error setting up watcher: {0}")]
    SetupWatcher(#[from] SetupWatcherError),
    #[error("error creating watch context: {0}")]
    CreateWatchContext(#[from] WatchContextError),
    #[error("error filtering events: {0}")]
    FilterEvents(#[from] FilterEventsError),
    #[error("error triggering rebuild: {0}")]
    Rebuild(#[from] RebuildError),
    #[error("error setting ctrl-c handler: {0}")]
    SetCtrlCHandler(#[from] ctrlc::Error),
}

pub fn watch(input: PathBuf, output: PathBuf) -> Result<(), WatchError> {
    let mut watch_context = WatchContext::new(input, output)?;

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = FileWatcher::new(tx.clone(), &watch_context)?;

    // Create shutdown flag for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })?;

    println!(
        "{} Watching for changes...Press Ctrl+C to stop",
        "Info:".blue().bold()
    );

    while running.load(Ordering::SeqCst) {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(result) => {
                let filtered_paths = match watcher.filter_events(result, &watch_context) {
                    Ok(paths) => paths,
                    Err(err) => {
                        eprintln!("{:?}", miette::Report::from(err));
                        continue;
                    }
                };

                if filtered_paths.is_empty() {
                    continue;
                }

                // check if config file has changed
                if filtered_paths.contains(&watch_context.config_path) {
                    println!(
                        "{} configuration file changed, reloading...",
                        "Info:".blue().bold()
                    );
                    if let Err(err) = watch_context.reload() {
                        eprintln!("{:?}", miette::Report::from(err));
                        continue;
                    }
                    if let Err(err) = watcher.reset(tx.clone(), &watch_context) {
                        eprintln!("{:?}", miette::Report::from(err));
                        continue;
                    }
                }

                // trigger rebuild
                if let Err(err) = trigger_rebuild(&watch_context) {
                    eprintln!("{:?}", miette::Report::from(err));
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                // do nothing
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => {
                eprintln!(
                    "{}: Watcher has been disconnected",
                    "Warning".yellow().bold()
                );
                break;
            }
        }
    }

    println!("{} Shutting down watch mode...", "Info".blue().bold());
    Ok(())
}

#[derive(Debug, Diagnostic, Error)]
pub enum RebuildError {
    #[error("error building: {0}")]
    Build(#[from] BuildError),
    #[error("error creating output directory: {0}")]
    CreateOutputDir(std::io::Error),
}

fn trigger_rebuild(watch_context: &WatchContext) -> Result<(), RebuildError> {
    println!("{} rebuilding...", "Info:".blue().bold());

    // Ensure output directory exists
    if let Some(parent) = watch_context.output.parent() {
        std::fs::create_dir_all(parent).map_err(RebuildError::CreateOutputDir)?;
    }

    build(&watch_context.input, &watch_context.output, true)?;

    println!("{} Rebuild done!", "Info:".blue().bold());
    Ok(())
}
