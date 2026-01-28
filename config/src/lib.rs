//! Configuration management for the NemCSS project.
//!
//! This crate handles loading, parsing, and resolving design tokens and user configurations
//! to generate CSS utility classes.
//!
//! ## Configuration
//!
//! Configuration is loaded from `nemcss.config.json` in the project root.
//!
//! ## Design tokens
//!
//! Design tokens are loaded from the `tokensDir` directory (default: `design-tokens/`)
//! and resolved into concrete values that can be used for custom properties and utility class
//! generation.
//!
//! ## Utility Generation
//!
//! Based on the resolved design tokens and user-defined [`TokenUtilityConfig`],
//! this crate enables generation of CSS utility classes for your design system.
mod config;
mod tokens;

pub use config::{
    CONFIG_FILE_NAME, NemCssConfig, NemCssConfigError, ThemeConfig, TokenConfig, TokenUtilityConfig,
};
pub use tokens::{ResolveTokensError, ResolvedToken, TokenValue, resolve_all_tokens};
