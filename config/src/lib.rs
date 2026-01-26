//! This crate contains the configuration and design tokens loading logic.
//!
//! The configuration is loaded from the `nemcss.config.json` file and the design tokens are loaded from the `tokensDir` directory (default is `design-tokens`).
mod config;
mod tokens;

pub use config::{
    CONFIG_FILE_NAME, NemCSSConfig, NemCSSConfigError, ThemeConfig, TokenConfig, TokenUtilityConfig,
};
pub use tokens::{ResolveTokensError, ResolvedToken, TokenValue, resolve_all_tokens};
