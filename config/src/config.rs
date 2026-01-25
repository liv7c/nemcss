use std::collections::HashMap;
use std::path::PathBuf;

/// NemCSSConfig represents the configuration of the NemCSS util.
/// It contains the paths to the content files and the design tokens, as well as the theme configuration.
pub struct NemCSSConfig {
    /// Vector of glob patterns to determine which content files to target.
    /// Content files will be used to determine which CSS classes to generate.
    pub content: Vec<String>,

    /// Path to the directory containing the design tokens.
    pub tokens_dir: PathBuf,

    /// Theme configuration.
    pub theme: Option<ThemeConfig>,

    /// The base directory of the NemCSS project.
    pub base_dir: PathBuf,
}

/// ThemeConfig represents the configuration of the theme.
/// It contains the design tokens configuration per token type.
pub struct ThemeConfig {
    /// The design tokens configuration.
    /// This is a map of the design tokens to their configuration.
    /// The key is the name of the design token. The value is the configuration of the design token.
    tokens: HashMap<String, TokenConfig>,
}

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
pub struct TokenUtilityConfig {
    /// The prefix that will be used to generate the utility class.
    /// For example, if the prefix is "bg", the utility class will be "bg-[TOKEN VARIANT]".
    pub prefix: String,
    /// The CSS property that will use the design token value.
    pub property: String,
}
