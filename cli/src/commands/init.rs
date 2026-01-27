use std::fs;
use std::path::{Path, PathBuf};

use miette::Diagnostic;
use owo_colors::OwoColorize;
use thiserror::Error;

/// The configuration file name.
const CONFIG_FILE_NAME: &str = "nemcss.config.json";

/// Error type for the `init` command.
#[derive(Debug, Error, Diagnostic)]
pub enum InitError {
    /// Fail to retrieve the current directory.
    #[error("failed to retrieve the current directory: {0}")]
    #[diagnostic(code(nemcss::init::current_dir))]
    RetrieveCurrentDir(std::io::Error),

    /// The configuration file already exists.
    #[error("the configuration file already exists at {path:?}")]
    #[diagnostic(code(nemcss::init::config_file_exists))]
    ConfigFileExists { path: PathBuf },

    #[error("failed to create the configuration file: {0}")]
    #[diagnostic(code(nemcss::init::create_config_file))]
    CreateConfigFile(std::io::Error),
}

/// Initialize a new project with the `nemcss.config.json` configuration and example design tokens.
pub fn init() -> miette::Result<(), InitError> {
    let current_dir = std::env::current_dir().map_err(InitError::RetrieveCurrentDir)?;

    println!(
        "{}",
        "Initializing a new nemcss project in the current directory...".green()
    );
    println!();

    create_config_file(&current_dir)?;

    Ok(())
}

/// Create the configuration file at the root of the current directory.
///
/// # Errors
///
/// This function fails if the configuration file already exists or if there is an error while creating the file.
fn create_config_file(current_dir: &Path) -> miette::Result<(), InitError> {
    let config_file_path = current_dir.join(CONFIG_FILE_NAME);

    if config_file_path.exists() {
        return Err(InitError::ConfigFileExists {
            path: config_file_path,
        });
    }

    let config_file_content = include_str!("../templates/nemcss.config.json");
    fs::write(&config_file_path, config_file_content).map_err(InitError::CreateConfigFile)?;

    println!(
        "  ✓ Created {} at {}",
        CONFIG_FILE_NAME.green(),
        config_file_path.display()
    );

    Ok(())
}
