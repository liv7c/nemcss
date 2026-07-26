use std::fs;
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};

use miette::Diagnostic;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::tokens::{
    ResolveModeError, ResolveTokensError, ResolvedMode, ResolvedToken, ScanTokensDirError,
    resolve_all_modes, resolve_all_semantic_groups, resolve_all_tokens, resolve_registered_tokens,
    unregistered_token_files,
};
use crate::{ResolveSemanticError, ResolvedSemanticGroup};

/// The name of the NemCSS configuration file.
pub const CONFIG_FILE_NAME: &str = "nemcss.config.json";

/// Custom deserializer for non-empty prefixes.
/// It deserializes a string and returns an error if the string is empty.
fn deserialize_non_empty_prefix<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<String, D::Error> {
    let s = String::deserialize(d)?;
    if s.is_empty() {
        return Err(serde::de::Error::custom("prefix must not be empty"));
    }
    Ok(s)
}

/// Custom deserializer for non-empty properties.
/// It deserializes a string and returns an error if the string is empty.
fn deserialize_non_empty_property<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<String, D::Error> {
    let s = String::deserialize(d)?;
    if s.is_empty() {
        return Err(serde::de::Error::custom("property must not be empty"));
    }
    Ok(s)
}

/// Custom deserializer for an optional non-empty property.
fn deserialize_optional_non_empty_property<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<String>, D::Error> {
    let opt = Option::<String>::deserialize(d)?;
    if let Some(s) = &opt
        && s.is_empty()
    {
        return Err(serde::de::Error::custom("property must not be empty"));
    }

    Ok(opt)
}

/// Configuration for a NemCSS project, loaded from a nemcss.config.json.
/// Says which files to scan for utility classes, where your design tokens live, and
/// how tokens map to CSS.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NemCssConfig {
    /// Accepts a manually added "$schema" key without failing validation.
    /// Most editors detect the schema from the file name instead.
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub(crate) schema_url: Option<String>,

    /// Glob patterns for the files NemCSS should scan for class names.
    /// e.g. ["src/**/*.html"]
    pub content: Vec<String>,

    /// Path to directory containing the design tokens.
    #[serde(rename = "tokensDir", default = "get_default_tokens_dir")]
    pub tokens_dir: PathBuf,

    /// Registers each design token file you want NemCSS to use: where it
    /// lives, what prefix its custom properties get, and any utility classes
    /// to generate from it.
    pub theme: Option<ThemeConfig>,

    /// Named groups of tokens (e.g. "text", "background") that map to a CSS property,
    /// so you can style by intent instead of by raw token name.
    pub semantic: Option<SemanticConfig>,

    /// Named color modes (e.g. dark mode) that override semantic tokens.
    /// In the mode config, you can declare a custom selector (e.g. [data-theme="dark"]), a media
    /// and overrides per semantic group.
    /// ## Example
    /// ```json
    /// "modes": {
    ///   "dark": {
    ///     "selector": "[data-theme='dark']",
    ///     "media": "(prefers-color-scheme: dark)",
    ///     "overrides": {
    ///         "text": {
    ///             "default": "{colors.gray-100}"
    ///        }
    ///     }
    /// }
    /// }
    /// ```
    #[schemars(extend("defaultSnippets" = serde_json::json!([
        {
            "label": "mode (dark mode)",
            "description": "Scaffold a color mode. It is activated by a selector (manual toggle, e.g. a data-mode attribute) and/or a media query (system preference). Fill `overrides` with your own semantic groups and tokens. The editor completes them.",
            "body": {
                "${1:dark}": {
                    "selector": "[data-mode=\"${1:dark}\"]",
                    "overrides": {}
                }
            }
        }
    ])))]
    pub modes: Option<HashMap<String, ModeConfig>>,

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

    /// Resolves every token registered in the `theme` configuration, ignoring
    /// unregistered files.
    pub fn resolve_registered_tokens(
        &self,
    ) -> Result<HashMap<String, ResolvedToken>, ResolveTokensError> {
        resolve_registered_tokens(self)
    }

    /// Token files in the tokens directory that no theme entry points at
    pub fn unregistered_token_files(&self) -> Result<Vec<PathBuf>, ScanTokensDirError> {
        unregistered_token_files(self)
    }

    pub fn resolve_semantic_groups(
        &self,
        primitive_tokens: &HashMap<String, ResolvedToken>,
    ) -> Result<HashMap<String, ResolvedSemanticGroup>, ResolveSemanticError> {
        resolve_all_semantic_groups(self.semantic.as_ref(), primitive_tokens)
    }

    /// Resolves modes
    pub fn resolve_modes(
        &self,
        semantic_groups: &HashMap<String, ResolvedSemanticGroup>,
        primitive_tokens: &HashMap<String, ResolvedToken>,
    ) -> Result<Vec<ResolvedMode>, ResolveModeError> {
        resolve_all_modes(self.modes.as_ref(), semantic_groups, primitive_tokens)
    }

    /// Get absolute path to the tokens directory.
    pub fn tokens_dir(&self) -> PathBuf {
        self.base_dir.join(&self.tokens_dir)
    }

    /// Get a glob set of the content paths.
    pub fn content_glob_set(&self) -> Result<globset::GlobSet, globset::Error> {
        let mut builder = globset::GlobSetBuilder::new();
        for pattern in &self.content {
            let normalized = pattern.strip_prefix("./").unwrap_or(pattern);
            builder.add(globset::Glob::new(normalized)?);
        }
        builder.build()
    }
}

/// A design token file registered under `theme`, keyed by its token type
/// (e.g. "colors", "spacings").
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ThemeConfig {
    /// Maps a token type name (e.g. "colors") to where its source file is
    /// and how to generate CSS from it.
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
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TokenConfig {
    /// Path to the tokens file.
    /// It will make it possible to override the default configuration for a given token.
    pub source: PathBuf,

    /// The token prefix is used to generate the custom properties for the given token.
    /// If prefix is set to "color", the token prefix will be "color-".
    /// The generated custom properties will take the shape --<PREFIX>-<TOKEN_NAME>.
    #[serde(deserialize_with = "deserialize_non_empty_prefix")]
    #[schemars(extend("minLength"=1))]
    pub prefix: String,

    /// Utilities are used to generate utility classes for the given token.
    pub utilities: Option<Vec<TokenUtilityConfig>>,
}

/// Defines one utility class NemCSS generates from a token, e.g. `.bg-primary`.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TokenUtilityConfig {
    /// The prefix that will be used to generate the utility class.
    /// For example, if the prefix is "bg", the utility class will be "bg-[TOKEN VARIANT]".
    #[serde(deserialize_with = "deserialize_non_empty_prefix")]
    #[schemars(extend("minLength"=1))]
    pub prefix: String,
    /// The CSS property that will use the design token value.
    #[serde(deserialize_with = "deserialize_non_empty_property")]
    #[schemars(extend("minLength"=1))]
    pub property: String,
}

/// The semantic config enables the creation of groups of semantic tokens (e.g. for "text",
/// "background") with their own configuration (e.g. the CSS property they target and the tokens
/// they use)
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SemanticConfig {
    /// Maps a semantic group name (e.g. "text") to its configuration.
    #[serde(flatten)]
    pub groups: HashMap<String, SemanticGroupConfig>,
}

/// One semantic group: the CSS property it targets and the tokens it maps to.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SemanticGroupConfig {
    /// CSS property this group targets (e.g. "color", "background-color")
    /// This is optional. Omit it when you generate only CSS custom properties for
    /// the group.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_empty_property"
    )]
    #[schemars(extend("minLength"=1))]
    pub property: Option<String>,
    /// Mapping between a semantic name and an existing design token value
    /// e.g. "primary" -> "{colors.blue-800}"
    pub tokens: HashMap<String, String>,
}

/// The mode config enables the creation of multiple color modes. Define which
/// semantic tokens you want to override and how to activate the color mode (using a selector or a media query)
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModeConfig {
    /// CSS selector for a mode.
    /// Defaults to `[data-mode="<name>"]`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// Optional media query for the given color mode.
    /// When set, the overrides get applied when the media query matches and no explicit `data-mode` is present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media: Option<String>,
    /// Semantic token overrides for the given color mode.
    /// Every group and token must already exist in the `semantic` section.
    #[serde(default)]
    pub overrides: HashMap<String, HashMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_glob_matches_correct_files() {
        let config = NemCssConfig {
            content: vec![
                "content/**/*.md".to_string(),
                "src/**/*.svelte".to_string(),
                "./index.html".to_string(),
            ],
            ..Default::default()
        };

        let glob_set = config.content_glob_set().unwrap();

        // Matches the correct files in `content`
        assert!(glob_set.is_match("content/foo.md"));
        assert!(glob_set.is_match("content/bar/baz.md"));
        assert!(!glob_set.is_match("content/foo.txt"));

        // Matches the correct files in `src`
        assert!(glob_set.is_match("src/App.svelte"));
        assert!(glob_set.is_match("src/components/Button.svelte"));
        assert!(!glob_set.is_match("src/components/Error.tsx"));

        // Matches the files prefixed with './' in the content globs
        assert!(glob_set.is_match("index.html"));
    }

    #[test]
    fn test_content_glob_set_invalid_pattern() {
        let config = NemCssConfig {
            content: vec!["[invalid/**/*.md".to_string()],
            ..Default::default()
        };

        let glob_set = config.content_glob_set();

        assert!(glob_set.is_err());
    }

    mod schema_robustness {
        use super::*;

        #[test]
        fn valid_config_round_trips() {
            let json = r#"
            {
            "content": ["src/**/*.svelte"],
            "tokensDir": "custom-tokens"
            }
            "#;

            assert!(serde_json::from_str::<NemCssConfig>(json).is_ok());
        }

        #[test]
        fn schema_field_is_accepted() {
            let json = r#"
            {
            "$schema": "./nemcss.schema.json",
            "content": ["src/**/*.svelte"],
            "tokensDir": "custom-tokens"
            }
            "#;
            assert!(
                serde_json::from_str::<NemCssConfig>(json).is_ok(),
                "$schema field should be accepted in the config"
            );
        }

        #[test]
        fn unknown_top_level_field_is_rejected() {
            let json = r#"
            {
                "cotent": ["src/**/*.svelte"]
            }
            "#;
            assert!(
                serde_json::from_str::<NemCssConfig>(json).is_err(),
                "typo in top-level fields should be a parse error"
            );
        }
    }

    mod deserialize_non_empty_string {
        use super::*;

        #[test]
        fn test_deserialize_fails_on_empty_utility_prefix() {
            let json = r#"
            {
                "content": [],
                "theme": {
                    "colors": {
                        "source": "design-tokens/colors.json",
                        "utilities": [
                            { "prefix": "", "property": "background-color" }
                        ]
                    }
                }
            }
                "#;
            assert!(serde_json::from_str::<NemCssConfig>(json).is_err());
        }

        #[test]
        fn test_deserialize_fails_on_empty_semantic_property() {
            let json = r#"
            {
                "content": [],
                "theme": {
                    "colors": {
                        "source": "design-tokens/colors.json"
                    }
                },
                "semantic": {
                    "text": {
                        "property": "",
                        "tokens": {
                            "primary": "{colors.blue-500}"
                        }
                    }
                }
            }"#;
            assert!(serde_json::from_str::<NemCssConfig>(json).is_err());
        }
    }

    mod semantic_tokens {
        use super::*;

        #[test]
        fn test_deserialize_config_with_semantic_tokens() {
            let json = r#"{
                "content": [],
                "semantic": {
                    "text": {
                        "property": "color",
                        "tokens": {
                            "primary": "{colors.blue-500}",
                            "error": "{colors.red-700}"
                        }
                    }
                }
            }"#;

            let config: NemCssConfig = serde_json::from_str(json).unwrap();
            let semantic = config.semantic.unwrap();
            let text = semantic.groups.get("text").unwrap();
            assert_eq!(text.property, Some("color".to_string()));
            assert_eq!(text.tokens.get("primary").unwrap(), "{colors.blue-500}");
        }

        #[test]
        fn test_deserialize_semantic_group_with_no_property() {
            let json = r#"{
                "content": [],
                "semantic": {
                    "text": {
                        "tokens": {
                            "primary": "{colors.blue-500}",
                            "secondary": "{colors.red-500}"
                        }
                    }
                }
            }"#;

            let config: NemCssConfig = serde_json::from_str(json).unwrap();
            let semantic = config.semantic.unwrap();
            let text = semantic.groups.get("text").unwrap();
            assert_eq!(text.property, None);
            assert_eq!(text.tokens.get("primary").unwrap(), "{colors.blue-500}");
            assert_eq!(text.tokens.get("secondary").unwrap(), "{colors.red-500}");
        }

        #[test]
        fn test_config_without_semantic_is_valid() {
            let json = r#"{ "content": [] }"#;
            let config: NemCssConfig = serde_json::from_str(json).unwrap();
            assert!(config.semantic.is_none());
        }
    }

    mod modes_config {
        use super::*;

        fn parse(json: &str) -> Result<NemCssConfig, serde_json::Error> {
            serde_json::from_str(json)
        }

        #[test]
        fn parses_a_selector_activated_mode() {
            let json = r#"{
                "content": ["src/**/*html"],
                "semantic": {
                    "text": {
                        "property": "color",
                        "tokens": {
                            "default": "{colors.gray-800}"
                        }
                    }
                },
                "modes": {
                    "dark": {
                        "selector": "[data-theme='dark']",
                        "overrides": {
                            "text": { "default": "{colors.gray-100}" }
                        }
                    }
                }
            }"#;

            let config = parse(json).expect("failed to parse config");
            let modes = config.modes.expect("modes should be parsed");
            let dark = modes.get("dark").expect("dark mode should exist");

            assert_eq!(dark.selector.as_deref(), Some("[data-theme='dark']"));
            assert_eq!(dark.media, None);
            assert_eq!(
                dark.overrides.get("text").unwrap().get("default").unwrap(),
                "{colors.gray-100}"
            );
        }

        #[test]
        fn parses_a_media_activated_mode() {
            let json = r#"{
                "content": ["src/**/*html"],
                "semantic": {
                    "text": {
                        "property": "color",
                        "tokens": {
                            "default": "{colors.gray-800}"
                        }
                    }
                },
                "modes": {
                    "dark": {
                        "media": "(prefers-color-scheme: dark)",
                        "overrides": {
                            "text": { "default": "{colors.gray-100}" }
                        }
                    }
                }
            }"#;

            let config = parse(json).expect("failed to parse config");
            let modes = config.modes.expect("modes should be parsed");
            let dark = modes.get("dark").expect("dark mode should exist");

            assert_eq!(dark.media.as_deref(), Some("(prefers-color-scheme: dark)"));
            assert_eq!(
                dark.overrides.get("text").unwrap().get("default").unwrap(),
                "{colors.gray-100}"
            );
        }

        #[test]
        fn mode_fields_are_optional_at_parse_time() {
            let config = parse(
                r#"{
                "content": ["src/**/*html"],
                "semantic": {
                    "text": {
                        "property": "color",
                        "tokens": {
                            "default": "{colors.gray-800}"
                        }
                    }
                },
                "modes": {
                    "dark": {}
                }
            }"#,
            )
            .expect("failed to parse the config");

            let modes = config.modes.unwrap();
            let dark = modes.get("dark").unwrap();

            assert_eq!(dark.selector, None);
            assert_eq!(dark.media, None);
            assert!(dark.overrides.is_empty());
        }

        #[test]
        fn config_without_modes_still_parses() {
            let config = parse(
                r#"{
                    "content": ["src/**/*html"]
                }"#,
            )
            .expect("failed to parse the config");

            assert!(config.modes.is_none());
        }

        #[test]
        fn unknown_field_inside_mode_is_an_error() {
            let json = r#"{
                "content": ["src/**/*html"],
                "semantic": {
                    "text": {
                        "property": "color",
                        "tokens": {
                            "default": "{colors.gray-800}"
                        }
                    }
                },
                "modes": {
                    "dark": {
                        "selector": "[data-theme='dark']",
                        "media": "(prefers-color-scheme: dark)",
                        "overides": {
                            "text": { "default": "{colors.gray-100}" }
                        }
                    }
                }
            }"#;

            let config = parse(json);
            assert!(config.is_err());
        }
    }
}
