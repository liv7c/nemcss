use std::path::Path;

use config::{CONFIG_FILE_NAME, NemCssConfig, NemCssConfigError, ResolveTokensError};
use engine::{ResponsiveUtility, Utility};
use globset::GlobSet;
use miette::Diagnostic;
use thiserror::Error;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, Hover, HoverContents, MarkupContent,
    MarkupKind, Url,
};

/// Cache for the LSP server.
/// This cache is used to store the generated utilities, viewports, custom properties, and content globs.
#[derive(Debug)]
pub struct NemCache {
    pub(crate) utilities: Vec<Utility>,
    pub(crate) responsive_utilities: Vec<ResponsiveUtility>,
    pub(crate) config: NemCssConfig,
    pub(crate) custom_properties: Vec<CustomProperty>,
    pub(crate) content_globs: GlobSet,
}

/// A parsed CSS custom property with its name and resolved value.
///
/// # Example
/// ```ignore
/// let custom_property = CustomProperty {
///   name: String::from("--color-primary"),
///   value: String::from("#f1f1f1"),
/// };
/// ```
#[derive(Debug, PartialEq)]
pub struct CustomProperty {
    /// The full custom property name including the `--` prefix
    pub name: String,
    /// The resolved value from the design tokens
    pub value: String,
}

impl CustomProperty {
    fn parse(raw: &str) -> Option<Self> {
        let raw = raw.strip_suffix(';').unwrap_or(raw);
        let (name, value) = raw.split_once(": ")?;
        Some(Self {
            name: name.to_string(),
            value: value.to_string(),
        })
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum BuildCacheError {
    #[error("failed to build cache: {0}")]
    #[diagnostic(code(build_cache_error::config_error))]
    NemCssConfig(#[from] NemCssConfigError),
    #[error("failed to resolve tokens: {0}")]
    #[diagnostic(code(build_cache_error::token_resolution_error))]
    TokenResolution(#[from] ResolveTokensError),
    #[error("failed to build glob set: {0}")]
    #[diagnostic(code(build_cache_error::globset_error))]
    GlobSet(#[from] globset::Error),
    #[error("failed to generate responsive utilities: {0}")]
    #[diagnostic(code(build_cache_error::generate_responsive_utilities_error))]
    GenerateResponsiveUtilities(#[from] engine::GenerateResponsiveUtilitiesError),
}

/// File extensions that always get custom property completions
/// regardless of the content globs in the config
const CSS_EXTENSIONS: &[&str] = &["css", "scss", "sass", "less"];

impl NemCache {
    pub fn build(workspace_root: &Path) -> miette::Result<Self, BuildCacheError> {
        let config_path = workspace_root.join(CONFIG_FILE_NAME);
        let config = NemCssConfig::from_path(&config_path)?;

        let resolved_tokens = config.resolve_all_tokens()?;
        let viewports = resolved_tokens.get("viewports");

        let generated_css = engine::generate_css(resolved_tokens.values(), viewports, None);
        let responsive_utilities =
            engine::generate_all_responsive_utilities(&generated_css.utilities, viewports)?;

        let content_globs = config.content_glob_set()?;

        Ok(Self {
            utilities: generated_css.utilities,
            custom_properties: generated_css
                .custom_properties
                .iter()
                .filter_map(|raw| CustomProperty::parse(raw))
                .collect(),
            responsive_utilities,
            config,
            content_globs,
        })
    }

    /// Checks if the given URL is a content file by comparing it to the content globs in the config
    /// We retrieve the url from the CompletionParams textDocumentPosition field.
    pub fn is_content_file(&self, url: &Url) -> bool {
        let path = match url.to_file_path() {
            Ok(path) => path,
            Err(_) => return false,
        };

        let relative_path = match path.strip_prefix(&self.config.base_dir) {
            Ok(path) => path,
            Err(_) => return false,
        };

        self.content_globs.is_match(relative_path)
    }

    /// Checks if a given URL is relevant for any LSP feature.
    /// We mainly use to determine when to trigger some features such as completions and hovers in
    /// files that are not content files (e.g. CSS files)
    pub fn is_relevant_file(&self, url: &Url) -> bool {
        if self.is_content_file(url) {
            return true;
        }

        url.to_file_path()
            .ok()
            .and_then(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|ext| CSS_EXTENSIONS.contains(&ext))
            })
            .unwrap_or(false)
    }

    /// Returns completion items for custom properties matching the given partial name
    pub fn var_completions(&self, partial_name: &str) -> Vec<CompletionItem> {
        self.custom_properties
            .iter()
            .filter(|prop| partial_name.is_empty() || prop.name.starts_with(partial_name))
            .map(|prop| CompletionItem {
                label: prop.name.to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```css\n{}: {};\n```", prop.name, prop.value),
                })),
                ..Default::default()
            })
            .collect()
    }

    /// Returns completions for utility classes matching the given partial name
    pub fn class_completions(&self, partial_name: &str) -> Vec<CompletionItem> {
        self.utilities
            .iter()
            .filter(|u| u.class_name().starts_with(partial_name))
            .map(|u| CompletionItem {
                label: u.class_name().to_string(),
                kind: Some(CompletionItemKind::VALUE),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```css\n{}\n```", u.full_class()),
                })),
                ..Default::default()
            })
            .collect()
    }

    /// Returns completions for responsive utility classes matching the given partial name
    pub fn responsive_class_completions(&self, partial_name: &str) -> Vec<CompletionItem> {
        self.responsive_utilities
            .iter()
            .filter(|u| partial_name.is_empty() || u.responsive_class_name.contains(partial_name))
            .map(|u| CompletionItem {
                label: u.responsive_class_name.to_string(),
                kind: Some(CompletionItemKind::VALUE),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```css\n{}\n```", u.full_css_definition),
                })),
                ..Default::default()
            })
            .collect()
    }

    /// Returns a hover response for the custom property with the given name
    pub fn hover_for_custom_property(&self, prop_name: &str) -> Option<Hover> {
        let prop = self
            .custom_properties
            .iter()
            .find(|p| p.name == prop_name)?;

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("```css\n{}: {};\n```", prop.name, prop.value),
            }),
            range: None,
        })
    }

    /// Returns a hover response for the utility class or responsive utility class matching the
    /// given token
    pub fn hover_for_class(&self, token: &str) -> Option<Hover> {
        let css = self
            .utilities
            .iter()
            .find(|u| u.class_name() == token)
            .map(|u| u.full_class().to_string())
            .or_else(|| {
                self.responsive_utilities
                    .iter()
                    .find(|u| u.responsive_class_name == token)
                    .map(|u| u.full_css_definition.to_string())
            })?;

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("```css\n{}\n```", css),
            }),
            range: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;
    use assert_fs::prelude::*;

    fn create_test_project() -> Result<TempDir, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        temp_dir.child(CONFIG_FILE_NAME).write_str(
            r#"{
                "content": ["src/**/*.html"]
            }"#,
        )?;

        temp_dir.child("design-tokens").create_dir_all()?;

        temp_dir.child("design-tokens/colors.json").write_str(
            r##"{
                "title": "colors",
                "items": [
                    {"name": "primary", "value": "#000000"},
                    {"name": "secondary", "value": "#ffffff"}
                ]
            }"##,
        )?;

        temp_dir.child("design-tokens/spacings.json").write_str(
            r##"{
                "title": "spacings",
                "items": [
                    {"name": "sm", "value": "0.5rem"},
                    {"name": "md", "value": "1rem"}
                ]
            }"##,
        )?;

        Ok(temp_dir)
    }

    mod build_cache {
        use super::*;

        #[test]
        fn test_build_cache_successfully() {
            let temp_dir = create_test_project().expect("failed to create test project");

            let cache = NemCache::build(temp_dir.path()).expect("failed to build cache");

            assert!(
                !cache.utilities.is_empty(),
                "should have generated utilities"
            );
            assert!(
                !cache.config.content.is_empty(),
                "should have content globs"
            );
            assert!(
                cache.responsive_utilities.is_empty(),
                "should not have generated responsive utilities"
            );

            let utility_names: Vec<_> = cache.utilities.iter().map(|u| u.class_name()).collect();

            assert_eq!(utility_names.len(), 32, "should have 32 utilities");
            assert!(!utility_names.is_empty(), "should have generated utilities");
            assert!(utility_names.contains(&"p-sm"));
            assert!(utility_names.contains(&"ml-md"));
        }

        #[test]
        fn test_build_cache_with_viewports() {
            let temp_dir = create_test_project().expect("failed to create test project");
            temp_dir
                .child("design-tokens/viewports.json")
                .write_str(
                    r##"{
                  "title": "viewports",
                  "items": [
                      {"name": "sm", "value": "640px"},
                      {"name": "md", "value": "768px"}
                  ]
              }"##,
                )
                .expect("failed to write viewports.json");

            let cache = NemCache::build(temp_dir.path()).expect("failed to build cache");
            assert!(
                !cache.responsive_utilities.is_empty(),
                "should have responsive utilities"
            );

            let responsive_class_names: Vec<&str> = cache
                .responsive_utilities
                .iter()
                .map(|u| u.responsive_class_name.as_str())
                .collect();

            assert!(responsive_class_names.contains(&"sm:bg-primary"));
            assert!(responsive_class_names.contains(&"md:bg-primary"));
            assert!(responsive_class_names.contains(&"sm:text-primary"));
            assert!(responsive_class_names.contains(&"md:text-primary"));

            assert!(responsive_class_names.contains(&"sm:bg-secondary"));
            assert!(responsive_class_names.contains(&"md:bg-secondary"));
            assert!(responsive_class_names.contains(&"sm:text-secondary"));
            assert!(responsive_class_names.contains(&"md:text-secondary"));
        }

        #[test]
        fn test_build_cache_missing_config() {
            let temp_dir = TempDir::new().expect("failed to create temp dir");

            let cache = NemCache::build(temp_dir.path());
            assert!(
                cache.is_err(),
                "should fail to build cache when config is missing"
            );
        }

        #[test]
        fn test_build_cache_fails_missing_design_tokens() {
            let temp_dir = TempDir::new().expect("failed to create temp dir");
            temp_dir
                .child(CONFIG_FILE_NAME)
                .write_str(
                    r#"{
                  "content": ["src/**/*.html"]
              }"#,
                )
                .expect("failed to write config");

            let cache = NemCache::build(temp_dir.path());
            assert!(
                cache.is_err(),
                "should fail to build cache when design-tokens is missing"
            );
        }

        #[test]
        fn test_cache_custom_properties_are_structured() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let cache = NemCache::build(temp_dir.path()).expect("failed to build cache");

            assert!(!cache.custom_properties.is_empty());

            let primary = cache
                .custom_properties
                .iter()
                .find(|p| p.name == "--color-primary")
                .expect("should have --color-primary");
            assert_eq!(primary.value, "#000000");

            let spacing_sm = cache
                .custom_properties
                .iter()
                .find(|p| p.name == "--spacing-sm")
                .expect("should have --spacing-sm");
            assert_eq!(spacing_sm.value, "0.5rem");
        }
    }

    mod custom_properties_parse {
        use super::*;

        #[test]
        fn test_parse_simple_property() {
            let prop = CustomProperty::parse("--color-primary: yellow;").unwrap();
            assert_eq!(prop.name, "--color-primary".to_string());
            assert_eq!(prop.value, "yellow".to_string());
        }

        #[test]
        fn test_parse_property_without_semi_colon() {
            let prop = CustomProperty::parse("--color-primary: yellow").unwrap();
            assert_eq!(prop.name, "--color-primary".to_string());
            assert_eq!(prop.value, "yellow".to_string());
        }

        #[test]
        fn test_parse_returns_none_when_invalid_format() {
            assert!(CustomProperty::parse("invalid").is_none());
            assert!(CustomProperty::parse("--property_with_no_value").is_none());
            assert!(CustomProperty::parse("").is_none());
        }
    }

    mod completions {
        use super::*;

        #[test]
        fn test_var_completions_returns_empty_when_no_matching_properties() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let cache = NemCache::build(temp_dir.path()).expect("failed to build cache");

            let completions = cache.var_completions("foo");
            assert!(completions.is_empty());
        }

        #[test]
        fn test_var_completions_returns_matching_properties() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let cache = NemCache::build(temp_dir.path()).expect("failed to build cache");

            let completions = cache.var_completions("--color");
            assert_eq!(completions.len(), 2);

            assert_eq!(completions[0].label, "--color-primary");
            assert_eq!(completions[1].label, "--color-secondary");
        }

        #[test]
        fn test_class_completions_returns_all_when_partial_name_is_empty() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let cache = NemCache::build(temp_dir.path()).expect("failed to build cache");

            let completions = cache.class_completions("");
            assert_eq!(completions.len(), cache.utilities.len());
        }

        #[test]
        fn test_class_completions_returns_matching_classes() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let cache = NemCache::build(temp_dir.path()).expect("failed to build cache");

            let completions = cache.class_completions("bg-");
            assert!(!completions.is_empty());
            assert!(completions.iter().all(|c| c.label.starts_with("bg-")));
        }

        #[test]
        fn test_responsive_class_completions_returns_all_when_partial_name_is_empty() {
            let temp_dir = create_test_project().expect("failed to create test project");
            temp_dir
                .child("design-tokens/viewports.json")
                .write_str(
                    r##"{
                  "title": "viewports",
                  "items": [
                      {"name": "sm", "value": "640px"},
                      {"name": "md", "value": "768px"}
                  ]
              }"##,
                )
                .expect("failed to write viewports.json");
            let cache = NemCache::build(temp_dir.path()).expect("failed to build cache");

            let completions = cache.responsive_class_completions("");
            assert_eq!(completions.len(), cache.responsive_utilities.len());
        }

        #[test]
        fn test_responsive_class_completions_returns_matching_classes() {
            let temp_dir = create_test_project().expect("failed to create test project");
            temp_dir
                .child("design-tokens/viewports.json")
                .write_str(
                    r##"{
                  "title": "viewports",
                  "items": [
                      {"name": "sm", "value": "640px"},
                      {"name": "md", "value": "768px"}
                  ]
              }"##,
                )
                .expect("failed to write viewports.json");
            let cache = NemCache::build(temp_dir.path()).expect("failed to build cache");

            let completions = cache.responsive_class_completions("sm:");
            assert!(!completions.is_empty());
            assert!(completions.iter().all(|c| c.label.starts_with("sm:")));
        }
    }

    mod hover {
        use super::*;

        #[test]
        fn test_hover_for_custom_property_returns_none_when_property_not_found() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let cache = NemCache::build(temp_dir.path()).expect("failed to build cache");

            let hover = cache.hover_for_custom_property("foo");
            assert!(hover.is_none());
        }

        #[test]
        fn test_hover_for_custom_property_returns_hover() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let cache = NemCache::build(temp_dir.path()).expect("failed to build cache");

            let hover = cache.hover_for_custom_property("--color-primary");
            assert!(hover.is_some());
            match hover.unwrap().contents {
                HoverContents::Markup(MarkupContent { kind, value }) => {
                    assert_eq!(kind, MarkupKind::Markdown);
                    assert!(value.starts_with("```css\n--color-primary"));
                }
                _ => panic!("invalid hover contents"),
            }
        }

        #[test]
        fn test_hover_for_class_returns_none_when_class_not_found() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let cache = NemCache::build(temp_dir.path()).expect("failed to build cache");

            let hover = cache.hover_for_class("foo");
            assert!(hover.is_none());
        }

        #[test]
        fn test_hover_for_class_returns_hover() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let cache = NemCache::build(temp_dir.path()).expect("failed to build cache");

            let hover = cache.hover_for_class("bg-primary");
            assert!(hover.is_some());
            match hover.unwrap().contents {
                HoverContents::Markup(MarkupContent { kind, value }) => {
                    assert_eq!(kind, MarkupKind::Markdown);
                    assert!(
                        value.starts_with("```css\n.bg-primary"),
                        "got {} instead",
                        value
                    );
                }
                _ => panic!("invalid hover contents"),
            }
        }

        #[test]
        fn test_hover_for_class_returns_hover_for_responsive_class() {
            let temp_dir = create_test_project().expect("failed to create test project");
            temp_dir
                .child("design-tokens/viewports.json")
                .write_str(
                    r##"{
                  "title": "viewports",
                  "items": [
                      {"name": "sm", "value": "640px"},
                      {"name": "md", "value": "768px"}
                  ]
              }"##,
                )
                .expect("failed to write viewports.json");
            let cache = NemCache::build(temp_dir.path()).expect("failed to build cache");

            let hover = cache.hover_for_class("sm:bg-primary");
            assert!(hover.is_some());
            match hover.unwrap().contents {
                HoverContents::Markup(MarkupContent { kind, value }) => {
                    assert_eq!(kind, MarkupKind::Markdown);
                    let expected_content = "@media (min-width: 640px) {\n  .sm\\:bg-primary";
                    assert!(value.contains(expected_content), "got {} instead", value);
                }
                _ => panic!("invalid hover contents"),
            }
        }
    }
}
