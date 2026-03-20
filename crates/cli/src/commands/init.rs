use std::fs;
use std::path::{Path, PathBuf};

use config::CONFIG_FILE_NAME;
use miette::Diagnostic;
use owo_colors::OwoColorize;
use thiserror::Error;

/// Name of the design tokens directory.
const DESIGN_TOKENS_DIR_NAME: &str = "design-tokens";

const SCHEMA_URL: &str = concat!(
    "https://raw.githubusercontent.com/liv7c/nemcss/v",
    env!("CARGO_PKG_VERSION"),
    "/packages/nemcss/schemas/nemcss.config.schema.json"
);

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

    /// Error creating the configuration file
    #[error("failed to create the configuration file: {0}")]
    #[diagnostic(code(nemcss::init::create_config_file))]
    CreateConfigFile(std::io::Error),

    /// Error creating the design tokens directory
    #[error("failed to create the design tokens directory: {0}")]
    #[diagnostic(code(nemcss::init::create_design_tokens_dir))]
    CreateDesignTokensDir(std::io::Error),

    /// Error creating the design token file
    #[error("failed to create the design token file: {0}")]
    #[diagnostic(code(nemcss::init::create_design_token_file))]
    CreateDesignTokenFile(std::io::Error),
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
    create_design_tokens_and_example_tokens(&current_dir)?;

    println!();
    println!("{}", "✓ Initialization complete!".green().bold());
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

    let config_file_content =
        include_str!("../templates/nemcss.config.json").replace("NEMCSS_SCHEMA_URL", SCHEMA_URL);
    fs::write(&config_file_path, config_file_content).map_err(InitError::CreateConfigFile)?;

    println!(
        "  ✓ Created {} at {}",
        CONFIG_FILE_NAME.green(),
        config_file_path.display()
    );

    Ok(())
}

/// Create the design tokens directory and example tokens at the root of the current directory.
/// If the directory already exists, it will be skipped.
///
/// # Errors
///
/// This function fails if there is an error while creating the directory or the example tokens.
fn create_design_tokens_and_example_tokens(current_dir: &Path) -> miette::Result<(), InitError> {
    let design_tokens_dir_path = current_dir.join(DESIGN_TOKENS_DIR_NAME);

    // Skip the design tokens + example tokens creation if user has already a design tokens
    // directory.
    if design_tokens_dir_path.exists() {
        println!(
            "  ℹ Directory {} already exists, skipping directory and example tokens creation",
            DESIGN_TOKENS_DIR_NAME.yellow()
        );
        return Ok(());
    }

    fs::create_dir(&design_tokens_dir_path).map_err(InitError::CreateDesignTokensDir)?;

    println!(
        "  ✓ Created directory {} at {}",
        DESIGN_TOKENS_DIR_NAME.green(),
        design_tokens_dir_path.display()
    );

    let colors_content = include_str!("../templates/colors.json");
    create_design_token_file(&design_tokens_dir_path, "colors", colors_content)?;
    let spacings_content = include_str!("../templates/spacings.json");
    create_design_token_file(&design_tokens_dir_path, "spacings", spacings_content)?;

    Ok(())
}

/// Create a design token file at the design tokens directory.
/// If the file already exists, it will be skipped.
///
/// # Errors
///
/// This function fails if there is an error while creating the file.
fn create_design_token_file(
    design_tokens_dir_path: &Path,
    token_type: &str,
    content: &str,
) -> miette::Result<(), InitError> {
    let json_file_name = format!("{token_type}.json");
    let design_token_file_path = design_tokens_dir_path.join(&json_file_name);

    if design_token_file_path.exists() {
        return Ok(());
    }

    fs::write(&design_token_file_path, content).map_err(InitError::CreateDesignTokenFile)?;

    println!(
        "  ✓ Created {} at {}",
        json_file_name.green(),
        design_token_file_path.display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_config_template_is_valid_nemcss_config() {
        let raw = include_str!("../templates/nemcss.config.json");
        let content = raw.replace("NEMCSS_SCHEMA_URL", "https://example.com/schema.json");
        serde_json::from_str::<config::NemCssConfig>(&content)
            .expect("Config template should be a valid NemCssConfig after substitution");
    }

    #[test]
    fn test_colors_template_is_valid_json() {
        let colors_content = include_str!("../templates/colors.json");
        serde_json::from_str::<serde_json::Value>(colors_content)
            .expect("Colors template should be valid JSON");
    }

    #[test]
    fn test_spacings_template_is_valid_json() {
        let spacings_content = include_str!("../templates/spacings.json");
        serde_json::from_str::<serde_json::Value>(spacings_content)
            .expect("Spacings template should be valid JSON");
    }
}
