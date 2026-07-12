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

    #[error("unable to read config file content: {0}")]
    #[diagnostic(code(nemcss::new_token_file::read_config_file))]
    ReadConfigFile(std::io::Error),

    #[error("unable to parse config file content: {0}")]
    #[diagnostic(code(nemcss::new_token_file::parse_config_file))]
    ParseConfigFile(serde_json::Error),

    #[error("config file content is not an object")]
    #[diagnostic(code(nemcss::new_token_file::config_not_an_object))]
    ConfigNotAnObject,

    #[error("theme entry in the config is not an object")]
    #[diagnostic(code(nemcss::new_token_file::theme_not_an_object))]
    ThemeNotAnObject,

    #[error("theme entry with name {name} already exists in the config")]
    #[diagnostic(code(nemcss::new_token_file::theme_entry_exists))]
    ThemeEntryExists { name: String },

    #[error("patched config file is invalid: {0}")]
    #[diagnostic(code(nemcss::new_token_file::patched_config_invalid))]
    PatchedConfigInvalid(serde_json::Error),

    #[error("unable to overwrite current nemcss config file: {0}")]
    #[diagnostic(code(nemcss::new_token_file::write_config_file))]
    WriteConfigFile(std::io::Error),

    #[error("interactive mode requires a terminal")]
    #[diagnostic(code(nemcss::new_token_file::not_a_terminal))]
    NotATerminal,

    #[error("prompt failed: {0}")]
    #[diagnostic(code(nemcss::new_token_file::prompt))]
    Prompt(inquire::InquireError),
}
