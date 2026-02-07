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

/// Context for the watch command containing configuration and state.
///
/// This struct holds all the necessary information for the watch command to watch files
/// and trigger rebuilds. It includes the parsed configuration, glob patterns for matching files,
/// and the paths to the input and output files.
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

    /// Reloads the configuration from disk and updates the glob set.
    ///
    /// This is called when the nemcss configuration file changes.
    /// It reparses the configuration, rebuilds the glob set, and updates the context.
    ///
    /// # Errors
    ///
    /// Return [`WatchContextError`] if:
    /// - The configuration file cannot be read.
    /// - The glob patterns in the config are invalid
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
    #[error("failed to reset watcher after config reload: {0}")]
    ResetWatcherAfterReload(SetupWatcherError),
}

/// Watches the file system changes and triggers CSS rebuilds when necessary.
///
/// This function sets up file system watchers on:
/// - Content files matching blogs in the nemcss configuration (`content` field)
/// - The configuration file itself (`nemcss.config.json`)
/// - The tokens directory (`tokens-dir` field)
/// - The input CSS file
///
/// It also sets up a ctrl-c handler to gracefully shutdown the watcher.
///
/// # Arguments
/// - `input`: The input CSS file.
/// - `output`: The output CSS file.
///
/// # Errors
///
/// This function returns an error if:
/// - The input CSS file does not exist.
/// - The output CSS file cannot be created.
/// - The configuration file cannot be read.
/// - The configuration file cannot be parsed.
/// - The content glob set cannot be built.
/// - The watcher cannot be set up.
/// - The watcher cannot be reset after a configuration reload.
/// - The watcher cannot be disconnected.
///
/// # Example
///
/// ```ignore
/// use std::path::PathBuf;
/// use cli::commands::watch;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// watch(PathBuf::from("input.css"), PathBuf::from("output.css"))?;
/// # Ok(())
/// # }
/// ```
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
                let filtered_paths = match FileWatcher::filter_events(result, &watch_context) {
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
                    match watch_context.reload() {
                        Ok(_) => {
                            if let Err(err) = watcher.reset(tx.clone(), &watch_context) {
                                eprintln!(
                                    "{} Failed to reset the watcher after updating the configuration file",
                                    "Error:".red().bold()
                                );
                                eprintln!(
                                    "{}: Exiting to avoid inconsistencies...Please restart.",
                                    "Info:".yellow().bold()
                                );

                                return Err(WatchError::ResetWatcherAfterReload(err));
                            }

                            println!(
                                "{} Configuration reloaded successfully!",
                                "Success:".green().bold()
                            );
                        }
                        Err(err) => {
                            eprintln!(
                                "Failed to reload configuration file: {:?}",
                                miette::Report::from(err)
                            );
                            eprintln!(
                                "{} Continuing with previous configuration...",
                                "Warning:".yellow().bold()
                            );
                            continue;
                        }
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
