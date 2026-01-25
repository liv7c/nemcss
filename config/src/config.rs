use std::fs;
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};

use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CONFIG_FILE_NAME: &str = "nemcss.config.json";

/// NemCSSConfig represents the configuration of the NemCSS util.
/// It contains the paths to the content files and the design tokens, as well as the theme configuration.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NemCSSConfig {
    /// Vector of glob patterns to determine which content files to target.
    /// Content files will be used to determine which CSS classes to generate.
    pub content: Vec<String>,

    /// Path to the directory containing the design tokens.
    #[serde(rename = "tokensDir")]
    pub tokens_dir: PathBuf,

    /// Theme configuration.
    pub theme: Option<ThemeConfig>,

    /// The base directory of the NemCSS project.
    #[serde(skip)]
    pub base_dir: PathBuf,
}

#[derive(Debug, Error, Diagnostic)]
pub enum NemCSSConfigError {
    #[error("failed to read NemCSS config file: {0}")]
    #[diagnostic(code(nemcss::config::read_config_file))]
    ReadConfigFile(std::io::Error),

    #[error("failed to parse NemCSS config file: {0}")]
    #[diagnostic(code(nemcss::config::parse_config_file))]
    ParseConfigFile(serde_json::Error),
}

impl NemCSSConfig {
    /// Creates a new NemCSSConfig instance from a given path.
    pub fn from_path<T: AsRef<Path>>(path: T) -> Result<Self, NemCSSConfigError> {
        let config_path = path.as_ref();

        let content = fs::read_to_string(config_path).map_err(NemCSSConfigError::ReadConfigFile)?;

        let mut config: NemCSSConfig =
            serde_json::from_str(&content).map_err(NemCSSConfigError::ParseConfigFile)?;

        let base_dir = config_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        config.base_dir = base_dir;

        Ok(config)
    }
}

/// ThemeConfig represents the configuration of the theme.
/// It contains the design tokens configuration per token type.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// The design tokens configuration.
    /// This is a map of the design tokens to their configuration.
    /// The key is the name of the design token. The value is the configuration of the design token.
    #[serde(flatten)]
    tokens: HashMap<String, TokenConfig>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TokenConfig {
    /// Path to the tokens file. If not specified, it will be auto-discovered based
    /// on the token directory and the token name.
    pub source: Option<PathBuf>,

    /// The token prefix is used to generate the custom properties for the given token.
    /// For example, if prefix is "color", the token prefix will be "color-".
    /// If the color tokens include a "primary" variant, the custom property generated will be
    /// "--color-primary".
    pub prefix: Option<String>,

    /// Utilities are used to generate utility classes for the given token.
    pub utilities: Option<Vec<TokenUtilityConfig>>,
}

/// TokenUtilityConfig represents the configuration of a utility class for a given token.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TokenUtilityConfig {
    /// The prefix that will be used to generate the utility class.
    /// For example, if the prefix is "bg", the utility class will be "bg-[TOKEN VARIANT]".
    pub prefix: String,
    /// The CSS property that will use the design token value.
    pub property: String,
}
