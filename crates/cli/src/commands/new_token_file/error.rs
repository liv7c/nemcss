use std::path::PathBuf;

use config::NemCssConfigError;
use miette::Diagnostic;
use thiserror::Error;

/// Error type for the `new-token-file` command.
#[derive(Debug, Diagnostic, Error)]
pub enum NewTokenFileError {
    #[error(
        "name count does not match number of values provided: expected {expected} names, got {got} instead"
    )]
    #[diagnostic(code(nemcss::new_token_file::name_count_mismatch))]
    NameCountMismatch { expected: usize, got: usize },

    #[error(
        "value {value:?} is not a number, so it cannot automically be named automatically. Use the --names argument"
    )]
    #[diagnostic(code(nemcss::new_token_file::name_required_for_value))]
    NameRequiredForValue { value: String },

    #[error("no `nemcss.config.json` found at {path:?} - run `nemcss init` first")]
    #[diagnostic(code(nemcss::new_token_file::config_file_not_found))]
    ConfigFileNotFound { path: PathBuf },

    #[error("unable to retrieve current directory: {0}")]
    #[diagnostic(code(nemcss::new_token_file::retrieve_current_dir))]
    RetrieveCurrentDir(std::io::Error),

    #[error("unable to load configuration file")]
    #[diagnostic(code(nemcss::new_token_file::load_config))]
    LoadConfig(NemCssConfigError),

    #[error("unable to create the tokens dir: {0}")]
    #[diagnostic(code(nemcss::new_token_file::create_tokens_dir))]
    CreateTokensDir(std::io::Error),

    #[error(
        "token file already exists at {path:?} in the tokens directory. Use --force to overwrite the existing token file."
    )]
    #[diagnostic(code(nemcss::new_token_file::token_file_exists))]
    TokenFileExists { path: PathBuf },

    #[error("unable to serialize token file: {0}")]
    #[diagnostic(code(nemcss::new_token_file::serialize))]
    Serialize(serde_json::Error),

    #[error("unable to write token file: {0}")]
    #[diagnostic(code(nemcss::new_token_file::write_token_file))]
    WriteTokenFile(std::io::Error),
}
