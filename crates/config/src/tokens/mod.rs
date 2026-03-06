//! This module contains the logic for discovering, loading and parsing design tokens.
//!
//! # Tokens Discovery
//!
//! The tokens are discovered by traversing the `tokensDir` directory and finding all files with the `.json` extension.
//! By default, the `tokensDir` is set to `design-tokens` in the NemCSS configuration file.
//!
//! # Configuration overrides
//!
//! The `tokensDir` can be overridden in the NemCSS configuration file.
//! Token configurations can be overridden in the `theme` section of the configuration file.
//! Explicit token configuration will override the default configuration.
//! The token configuration enables the user to specify the source file, prefix, and custom utilities for a given token.
//! The crate currently supports two token types: simple and list.

mod resolver;
mod token;
mod utilities;

pub use resolver::{
    ResolveSemanticError, ResolveTokensError, ResolvedSemanticGroup, ResolvedToken,
    resolve_all_semantic_groups, resolve_all_tokens,
};
pub use token::TokenValue;
