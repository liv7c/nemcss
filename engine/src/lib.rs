//! # NemCSS Engine
//!
//! The NemCSS engine is responsible for generating CSS custom properties and
//! utility classes based on the design tokens. It uses both the auto-detected
//! design tokens and the user-defined configuration from the `nemcss.config.json` to do so.
//!
//! # Example
//!
//! ```no_run
//! use std::collections::HashMap;
//! use config::TokenValue;
//! use engine::{generate_css};
//!
//! let mut resolved_tokens = HashMap::new();
//! resolved_tokens.insert(
//!     "colors".to_string(),
//!     config::ResolvedToken {
//!         tokens: vec![
//!             (
//!                 "primary".to_string(),
//!                 TokenValue::Simple("yellow".to_string()),
//!             )
//!         ],
//!         utilities: vec![
//!             config::TokenUtilityConfig {
//!                 prefix: "text".to_string(),
//!                 property: "color".to_string(),
//!             },
//!             config::TokenUtilityConfig {
//!                 prefix: "bg".to_string(),
//!                 property: "background-color".to_string(),
//!             },
//!         ],
//!         prefix: "color".to_string(),
//!     },
//! );
//! let generated_css = generate_css(resolved_tokens.values(), resolved_tokens.get("viewports"), None);
//! let css = generated_css.to_css();
//! # assert!(css.contains("--color-primary: yellow;"));
//! # assert!(css.contains(".text-primary {\n  color: var(--color-primary);\n}"));
//! # assert!(css.contains(".bg-primary {\n  background-color: var(--color-primary);\n}"));
//! ```

mod generation;

pub use generation::{GeneratedCss, Utility, VIEWPORT_TOKEN_PREFIX, generate_css};
