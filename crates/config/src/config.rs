use std::fs;
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};

use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::tokens::{ResolveTokensError, ResolvedToken, resolve_all_tokens};

/// The name of the NemCSS configuration file.
pub const CONFIG_FILE_NAME: &str = "nemcss.config.json";

/// NemCssConfig represents the configuration of the NemCSS util.
/// It contains the paths to the content files and the design tokens, as well as the theme configuration.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NemCssConfig {
    /// Vector of glob patterns to determine which content files to target.
    /// Content files will be used to determine which CSS classes to generate.
    pub content: Vec<String>,

    /// Path to directory containing the design tokens.
    #[serde(rename = "tokensDir", default = "get_default_tokens_dir")]
    pub tokens_dir: PathBuf,

    /// Theme configuration.
    pub theme: Option<ThemeConfig>,

    /// The base directory of the NemCSS project.
    #[serde(skip)]
    pub base_dir: PathBuf,
}

/// Returns the default value for the tokensDir field.
fn get_default_tokens_dir() -> PathBuf {
    PathBuf::from("design-tokens")
}

/// NemCssConfigError represents the error type when loading the NemCSS configuration.
#[derive(Debug, Error, Diagnostic)]
pub enum NemCssConfigError {
    #[error("failed to read NemCSS config file: {0}")]
    #[diagnostic(code(nemcss::config::read_config_file))]
    ReadConfigFile(std::io::Error),

    #[error("failed to parse NemCSS config file: {0}")]
    #[diagnostic(code(nemcss::config::parse_config_file))]
    ParseConfigFile(serde_json::Error),
}

impl NemCssConfig {
    /// Creates a new NemCssConfig instance from a given path.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use config::NemCssConfig;
    /// use std::path::Path;
    ///
    /// let config = NemCssConfig::from_path("nemcss.config.json")?;
    /// println!("Tokens directory: {:?}", config.tokens_dir);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_path<T: AsRef<Path>>(path: T) -> Result<Self, NemCssConfigError> {
        let config_path = path.as_ref();

        let content = fs::read_to_string(config_path).map_err(NemCssConfigError::ReadConfigFile)?;

        let mut config: NemCssConfig =
            serde_json::from_str(&content).map_err(NemCssConfigError::ParseConfigFile)?;

        let base_dir = config_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        config.base_dir = base_dir;

        Ok(config)
    }

    /// Resolves all design tokens based on the configuration.
    ///
    /// It scans the tokens directory and loads the design tokens from the files.
    /// It then generates the utilities for each token based on the configuration.
    /// Finally, it returns a map of resolved tokens.
    ///
    /// # Returns
    ///
    /// A map of resolved tokens, where the key is the name of the token and the value is the resolved token (containing the token values, prefix, and utilities).
    ///
    /// # Errors
    /// This function returns an error if the tokens directory is not found or if the design tokens
    /// cannot be loaded.
    pub fn resolve_all_tokens(&self) -> Result<HashMap<String, ResolvedToken>, ResolveTokensError> {
        resolve_all_tokens(self)
    }

    /// Get absolute path to the tokens directory.
    pub fn tokens_dir(&self) -> PathBuf {
        self.base_dir.join(&self.tokens_dir)
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
    pub tokens: HashMap<String, TokenConfig>,
}

/// TokenConfig represents the configuration of a single design token.
/// You can override the default configuration by specifying the source, prefix, and utilities.
///
/// # Examples
///
/// In your `nemcss.config.json`:
///
/// ```json
/// {
///   "theme": {
///      "colors": {
///        "source": "design-tokens/colors.json",
///        "prefix": "color",
///        "utilities": [
///           {
///             "prefix": "highlight",
///             "property": "background-color"
///           }
///        ]
///      }
///
///   }
/// }
/// ```
///
/// With a token variant "primary" in colors.json, this generates:
/// - CSS custom property: `--color-primary`
/// - Utility class: `.highlight-primary { background-color: var(--color-primary); }`
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TokenConfig {
    /// Path to the tokens file.
    /// It will make it possible to override the default configuration for a given token.
    pub source: PathBuf,

    /// The token prefix is used to generate the custom properties for the given token.
    /// For example, if prefix is "color", the token prefix will be "color-".
    /// If the color tokens include a "primary" variant, the custom property generated will be
    /// "--color-primary".
    pub prefix: Option<String>,

    /// Utilities are used to generate utility classes for the given token.
    pub utilities: Option<Vec<TokenUtilityConfig>>,
}

/// TokenUtilityConfig represents the configuration of a utility class for a given token.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenUtilityConfig {
    /// The prefix that will be used to generate the utility class.
    /// For example, if the prefix is "bg", the utility class will be "bg-[TOKEN VARIANT]".
    pub prefix: String,
    /// The CSS property that will use the design token value.
    pub property: String,
}
