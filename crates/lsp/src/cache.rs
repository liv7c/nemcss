use std::{collections::HashMap, path::Path};

use config::{CONFIG_FILE_NAME, NemCssConfig, NemCssConfigError, ResolveTokensError};
use engine::{ResponsiveUtility, Utility};
use globset::GlobSet;
use miette::Diagnostic;
use thiserror::Error;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, Hover, HoverContents, MarkupContent,
    MarkupKind, Url,
};

use crate::file::CSS_EXTENSIONS;

/// Cache for the LSP server.
/// This cache is used to store the generated utilities, viewports, custom properties, and content globs.
/// It also stores the token references that are used to generate semantic tokens.
#[derive(Debug)]
pub struct NemCache {
    pub(crate) utilities: Vec<Utility>,
    pub(crate) responsive_utilities: Vec<ResponsiveUtility>,
    pub(crate) config: NemCssConfig,
    pub(crate) custom_properties: Vec<CustomProperty>,
    pub(crate) resolved_values: HashMap<String, String>,
    pub(crate) content_globs: GlobSet,
    pub(crate) token_references: Vec<String>,
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
    /// Indicates whether the property is an alias
    /// e.g. `--text-primary: var(--color-primary);` would be an alias, while `--color-primary: #000000;` would not
    pub is_alias: bool,
}

impl CustomProperty {
    fn parse(raw: &str) -> Option<Self> {
        let raw = raw.strip_suffix(';').unwrap_or(raw);
        let (name, value) = raw.split_once(": ")?;
        let is_alias = value.starts_with("var(");

        Some(Self {
            name: name.to_string(),
            value: value.to_string(),
            is_alias,
        })
    }
}

/// Extract the property name from a `var(--name)` expression
fn extract_var_name(value: &str) -> Option<&str> {
    value.strip_prefix("var(")?.strip_suffix(")")
}

/// Build a name to resolved value map for all custom properties,
/// following var() references one level deep.
///
/// # Examples
/// ```text
/// --text-primary: var(--color-blue-800) -> "--text-primary": "#1a356d"
/// ```
fn resolve_all_var_values(props: &[CustomProperty]) -> HashMap<String, String> {
    let raw_token_values: HashMap<&str, &str> = props
        .iter()
        .filter(|p| !p.value.starts_with("var("))
        .map(|p| (p.name.as_str(), p.value.as_str()))
        .collect();

    props
        .iter()
        .filter_map(|p| {
            let resolved_val = if p.value.starts_with("var(") {
                let referenced = extract_var_name(&p.value)?;
                let value = raw_token_values.get(referenced)?;
                value.to_string()
            } else {
                p.value.clone()
            };
            Some((p.name.clone(), resolved_val))
        })
        .collect()
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

pub struct BuildResult {
    pub cache: NemCache,
    pub warnings: Vec<String>,
}

impl NemCache {
    pub fn build(workspace_root: &Path) -> miette::Result<BuildResult, BuildCacheError> {
        let config_path = workspace_root.join(CONFIG_FILE_NAME);
        let config = NemCssConfig::from_path(&config_path)?;

        let primitive_tokens = config.resolve_all_tokens()?;

        let token_references: Vec<String> = primitive_tokens
            .iter()
            .flat_map(|(group_key, resolved)| {
                resolved
                    .tokens
                    .iter()
                    .map(move |(token_name, _)| format!("{{{}.{}}}", group_key, token_name))
            })
            .collect();

        let viewports = primitive_tokens.get("viewports");
        let (resolved_semantic_groups, semantic_warnings) = config
            .resolve_semantic_groups(&primitive_tokens)
            .map(|groups| (groups, None))
            // Permissive: if semantic groups cannot be resolved, fall back to empty rather
            // than breaking the entire cache (the user might be editing a semantic config file)
            .unwrap_or_else(|e| (Default::default(), Some(e.to_string())));

        let generated_css = engine::generate_css(
            primitive_tokens.values(),
            resolved_semantic_groups.values(),
            viewports,
            None,
        );
        let responsive_utilities =
            engine::generate_all_responsive_utilities(&generated_css.utilities, viewports)?;

        let custom_properties: Vec<CustomProperty> = generated_css
            .custom_properties
            .iter()
            .filter_map(|raw| CustomProperty::parse(raw))
            .collect();

        let resolved_values = resolve_all_var_values(&custom_properties);

        let content_globs = config.content_glob_set()?;

        Ok(BuildResult {
            cache: Self {
                utilities: generated_css.utilities,
                custom_properties,
                resolved_values,
                responsive_utilities,
                config,
                content_globs,
                token_references,
            },
            warnings: semantic_warnings.into_iter().collect(),
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
            .map(|prop| {
                let detail = self
                    .resolved_values
                    .get(&prop.name)
                    .cloned()
                    .unwrap_or(prop.value.clone());

                CompletionItem {
                    label: prop.name.to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    detail: Some(detail),
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("```css\n{}: {};\n```", prop.name, prop.value),
                    })),
                    ..Default::default()
                }
            })
            .collect()
    }

    /// Returns completion items for semantic custom properties matching the given partial name.
    pub fn semantic_property_completions(&self, partial_name: &str) -> Vec<CompletionItem> {
        self.custom_properties
            .iter()
            .filter(|prop| prop.is_alias)
            .filter(|prop| partial_name.is_empty() || prop.name.starts_with(partial_name))
            .map(|prop| {
                let detail = self
                    .resolved_values
                    .get(&prop.name)
                    .cloned()
                    .unwrap_or(prop.value.clone());

                CompletionItem {
                    label: prop.name.to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    detail: Some(detail),
                    sort_text: Some(format!("!{}", prop.name)),
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("```css\n{}: {};\n```", prop.name, prop.value),
                    })),
                    ..Default::default()
                }
            })
            .collect()
    }

    /// Returns completions for utility classes matching the given partial name
    pub fn class_completions(&self, partial_name: &str) -> Vec<CompletionItem> {
        self.utilities
            .iter()
            .filter(|u| u.class_name().starts_with(partial_name))
            .map(|u| {
                let detail = u
                    .class_value()
                    .split_once(": ")
                    .and_then(|(prop, val)| {
                        let resolved = extract_var_name(val)
                            .and_then(|var_name| self.resolved_values.get(var_name))?;
                        Some(format!("{}: {}", prop, resolved))
                    })
                    .unwrap_or(u.class_value().to_string());

                CompletionItem {
                    label: u.class_name().to_string(),
                    kind: Some(CompletionItemKind::VALUE),
                    detail: Some(detail),
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("```css\n{}\n```", u.full_class()),
                    })),
                    ..Default::default()
                }
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
                detail: Some(u.base_utility.class_value().to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```css\n{}\n```", u.full_css_definition),
                })),
                ..Default::default()
            })
            .collect()
    }

    /// Returns completions for token references matching the given partial input.
    /// Used when editing the semantic tokens section of nemcss.config.json
    pub fn token_ref_completions(&self, partial_input: &str) -> Vec<CompletionItem> {
        self.token_references
            .iter()
            .filter(|r| partial_input.is_empty() || r.contains(partial_input))
            .map(|reference| CompletionItem{
                label: reference.clone(),
                kind: Some(CompletionItemKind::REFERENCE),
                detail: Some("primitive token reference".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("References the `{}` primitive token.\n\nGenerates a semantic CSS variable pointing to this value.", reference),
                })),
                ..Default::default()
            }).collect()
    }

    /// Returns a hover response for the custom property with the given name
    pub fn hover_for_custom_property(&self, prop_name: &str) -> Option<Hover> {
        let prop = self
            .custom_properties
            .iter()
            .find(|p| p.name == prop_name)?;

        let resolved_val = if prop.value.starts_with("var(")
            && let Some(resolved) = self.resolved_values.get(prop_name)
        {
            format!(" /* {} */", resolved)
        } else {
            String::new()
        };

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "```css\n{}: {};{}\n```",
                    prop.name, prop.value, resolved_val
                ),
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
            .map(|u| self.annotate_class_css(u.full_class(), u.class_value()))
            .or_else(|| {
                self.responsive_utilities
                    .iter()
                    .find(|u| u.responsive_class_name == token)
                    .map(|u| {
                        self.annotate_class_css(
                            &u.full_css_definition,
                            u.base_utility.class_value(),
                        )
                    })
            })?;

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("```css\n{}\n```", css),
            }),
            range: None,
        })
    }

    /// Annotates a CSS string with the resolved value of a `var()` reference if one is found
    /// in `class_value`. e.g. `color: var(--color-primary)` → `color: var(--color-primary) /* #000000 */`
    fn annotate_class_css(&self, full_css: &str, class_value: &str) -> String {
        let Some((_, val)) = class_value.split_once(": ") else {
            return full_css.to_string();
        };

        let Some(var_name) = extract_var_name(val) else {
            return full_css.to_string();
        };

        let Some(resolved) = self.resolved_values.get(var_name) else {
            return full_css.to_string();
        };

        full_css.replacen(val, &format!("{} /* {} */", val, resolved), 1)
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
                "content": ["src/**/*.html"],
                "theme": {
                    "colors": {
                        "source": "design-tokens/colors.json",
                        "utilities": [
                            { "prefix": "text", "property": "color" },
                            { "prefix": "bg",   "property": "background-color" }
                        ]
                    },
                    "spacings": {
                        "source": "design-tokens/spacings.json",
                        "utilities": [
                            { "prefix": "p", "property": "padding" }
                        ]
                    }
                }
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

    fn create_semantic_test_project() -> Result<TempDir, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        temp_dir.child(CONFIG_FILE_NAME).write_str(
            r#"{
            "content": ["src/**/*.html"],
            "theme": {
                "colors": {
                    "source": "design-tokens/colors.json",
                    "utilities": [
                        { "prefix": "bg", "property": "background-color" }
                    ]
                }
            },
            "semantic": {
                "text": {
                    "property": "color",
                    "tokens": {
                        "primary":   "{colors.white}",
                        "secondary": "{colors.black}"
                    }
                }
            }
        }"#,
        )?;

        temp_dir.child("design-tokens").create_dir_all()?;
        temp_dir.child("design-tokens/colors.json").write_str(
            r#"{
            "title": "Color Tokens",
            "items": [
                { "name": "white", "value": "hsl(0, 0%, 100%)" },
                { "name": "black", "value": "hsl(0, 0%, 0%)" }
            ]
        }"#,
        )?;

        Ok(temp_dir)
    }

    mod build_cache {
        use super::*;

        #[test]
        fn test_build_cache_successfully() {
            let temp_dir = create_test_project().expect("failed to create test project");

            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

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

            assert_eq!(utility_names.len(), 6, "should have exactly 6 utilities");
            assert!(!utility_names.is_empty(), "should have generated utilities");
            assert!(utility_names.contains(&"text-primary"));
            assert!(utility_names.contains(&"text-secondary"));
            assert!(utility_names.contains(&"bg-primary"));
            assert!(utility_names.contains(&"bg-secondary"));
            assert!(utility_names.contains(&"p-sm"));
            assert!(utility_names.contains(&"p-md"));
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

            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");
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
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

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

        #[test]
        fn test_build_cache_with_unresolvable_semantic_groups_returns_warning() {
            let temp_dir = create_test_project().expect("failed to create test project");
            temp_dir
                .child(CONFIG_FILE_NAME)
                .write_str(
                    r#"
                {
                    "content": ["src/**/*.html"],
                    "theme": {
                        "colors": {
                            "source": "design-tokens/colors.json",
                            "utilities": [
                                { "prefix": "bg", "property": "background-color" }
                            ]
                        }
                    },
                    "semantic": {
                        "text": {
                            "property": "color",
                            "tokens": {
                                "primary": "{colors.does-not-exist}"
                            }
                        }
                    }
                }
                "#,
                )
                .expect("failed to write config");

            let BuildResult { cache, warnings } = NemCache::build(temp_dir.path())
                .expect("build should succeed despite bad semantic ref");
            assert!(!warnings.is_empty(), "should have warnings");
            assert!(
                warnings[0].contains("does-not-exist"),
                "warning should mention the bad reference, got: {}",
                warnings[0]
            );

            assert!(
                !cache.utilities.is_empty(),
                "should have generated utilities"
            );
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

        #[test]
        fn test_parse_primitive_property_is_not_an_alias() {
            let prop = CustomProperty::parse("--color-primary: #000000;")
                .expect("valid property should parse");
            assert!(!prop.is_alias, "primitive property should not be an alias");
        }

        #[test]
        fn test_parse_alias_property_is_marked_as_alias() {
            let prop = CustomProperty::parse("--text-primary: var(--color-primary);")
                .expect("valid property should parse");
            assert!(prop.is_alias, "alias property should be marked as alias");
        }
    }

    mod completions {
        use super::*;

        #[test]
        fn test_var_completions_returns_empty_when_no_matching_properties() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

            let completions = cache.var_completions("foo");
            assert!(completions.is_empty());
        }

        #[test]
        fn test_var_completions_returns_matching_properties() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

            let completions = cache.var_completions("--color");
            assert_eq!(completions.len(), 2);

            assert_eq!(completions[0].label, "--color-primary");
            assert_eq!(completions[1].label, "--color-secondary");
        }

        #[test]
        fn test_class_completions_returns_all_when_partial_name_is_empty() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

            let completions = cache.class_completions("");
            assert_eq!(completions.len(), cache.utilities.len());
        }

        #[test]
        fn test_class_completions_returns_matching_classes() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

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
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

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
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

            let completions = cache.responsive_class_completions("sm:");
            assert!(!completions.is_empty());
            assert!(completions.iter().all(|c| c.label.starts_with("sm:")));
        }

        #[test]
        fn test_token_ref_completions_returns_all_when_partial_input_is_empty() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

            let completions = cache.token_ref_completions("");
            assert_eq!(completions.len(), cache.token_references.len());
        }

        #[test]
        fn test_token_ref_completions_returns_matching_tokens() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

            let completions = cache.token_ref_completions("color");
            assert!(!completions.is_empty());
            assert!(
                completions.iter().all(|c| c.label.starts_with("{colors.")),
                "all completions should start with {{colors., got {:?}",
                completions
            );
        }

        #[test]
        fn test_token_ref_completions_returns_matching_tokens_with_partial_input() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

            let completions = cache.token_ref_completions("colors.p");
            assert!(!completions.is_empty());
            assert_eq!(completions.len(), 1);
            assert!(completions[0].label.starts_with("{colors.primary"));
        }

        #[test]
        fn test_semantic_property_completions_returns_only_semantic_properties() {
            let temp_dir = create_semantic_test_project().expect("failed to create test project");
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

            let completions = cache.semantic_property_completions("--");
            assert!(
                !completions.is_empty(),
                "should return completions for semantic properties, got none"
            );

            assert!(
                completions.iter().all(|c| c.label.starts_with("--text-")),
                "all completions should be for semantic properties, got: {:?}",
                completions.iter().map(|c| &c.label).collect::<Vec<_>>()
            );

            assert!(
                !completions.iter().any(|c| c.label.starts_with("--color-")),
                "should not return completions for primitive properties, got: {:?}",
                completions.iter().map(|c| &c.label).collect::<Vec<_>>()
            );
        }

        #[test]
        fn test_semantic_property_completions_filters_by_partial_name() {
            let temp_dir = create_semantic_test_project().expect("failed to create test project");
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

            let completions = cache.semantic_property_completions("--text-p");
            assert!(
                !completions.is_empty(),
                "should return completions for matching semantic properties, got none"
            );

            assert_eq!(completions.len(), 1, "only --text-primaru should match");
            assert_eq!(completions[0].label, "--text-primary");
        }

        #[test]
        fn test_semantic_property_completions_returns_empty_when_no_matches() {
            let temp_dir = create_semantic_test_project().expect("failed to create test project");
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

            let completions = cache.semantic_property_completions("--nonexistent");
            assert!(
                completions.is_empty(),
                "should return no completions when no matches, got: {:?}",
                completions
            );
        }
    }

    mod hover {
        use super::*;

        #[test]
        fn test_hover_for_custom_property_returns_none_when_property_not_found() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

            let hover = cache.hover_for_custom_property("foo");
            assert!(hover.is_none());
        }

        #[test]
        fn test_hover_for_custom_property_returns_hover() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

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
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

            let hover = cache.hover_for_class("foo");
            assert!(hover.is_none());
        }

        #[test]
        fn test_hover_for_class_returns_hover() {
            let temp_dir = create_test_project().expect("failed to create test project");
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

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
            let BuildResult { cache, .. } =
                NemCache::build(temp_dir.path()).expect("failed to build cache");

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

    mod resolved_values {
        use super::*;

        #[test]
        fn test_extract_var_name_returns_the_var_name() {
            assert_eq!(
                extract_var_name("var(--text-primary)",),
                Some("--text-primary")
            );
            assert_eq!(
                extract_var_name("var(--color-blue-800)",),
                Some("--color-blue-800")
            );
        }

        #[test]
        fn test_extract_var_name_returns_none_when_invalid_format_or_not_a_var() {
            assert_eq!(extract_var_name("--text-primary",), None);
            assert_eq!(extract_var_name("var(--color-blue-800);",), None);
        }

        #[test]
        fn test_resolve_value_chain_returns_correct_resolved_values_mapping() {
            let props = vec![
                CustomProperty {
                    name: "--color-blue-800".to_string(),
                    value: "#1a365d".to_string(),
                    is_alias: false,
                },
                CustomProperty {
                    name: "--spacing-sm".to_string(),
                    value: "0.5rem".to_string(),
                    is_alias: false,
                },
                CustomProperty {
                    name: "--text-primary".to_string(),
                    value: "var(--color-blue-800)".to_string(),
                    is_alias: true,
                },
                CustomProperty {
                    name: "--gap-sm".to_string(),
                    value: "var(--spacing-sm)".to_string(),
                    is_alias: true,
                },
            ];

            let result = resolve_all_var_values(&props);

            assert_eq!(
                result.len(),
                4,
                "expected a hashmap with all resolved values, got {:?} instead",
                result
            );

            assert_eq!(
                result
                    .get("--color-blue-800")
                    .expect("expect --color-blue-800 to exist in result hashmap"),
                "#1a365d"
            );
            assert_eq!(
                result
                    .get("--spacing-sm")
                    .expect("expect --spacing-sm to exist in result hashmap"),
                "0.5rem"
            );
            assert_eq!(
                result
                    .get("--text-primary")
                    .expect("expect --text-primary to exist in result hashmap"),
                "#1a365d"
            );
            assert_eq!(
                result
                    .get("--gap-sm")
                    .expect("expect --gap-sm to exist in result hashmap"),
                "0.5rem"
            );
        }
    }
}
