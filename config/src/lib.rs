//! This crate contains the configuration and design tokens loading logic.
//!
//! The configuration is loaded from the `nemcss.config.json` file and the design tokens are loaded from the `design-tokens` directory.
mod config;

pub use config::{
    CONFIG_FILE_NAME, NemCSSConfig, NemCSSConfigError, ThemeConfig, TokenConfig, TokenUtilityConfig,
};
