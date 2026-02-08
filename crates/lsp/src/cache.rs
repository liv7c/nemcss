use std::path::Path;

use config::{CONFIG_FILE_NAME, NemCssConfig, NemCssConfigError, ResolveTokensError};
use engine::{ResponsiveUtility, Utility};
use globset::GlobSet;
use miette::Diagnostic;
use thiserror::Error;
use tower_lsp::lsp_types::Url;

/// Cache for the LSP server.
/// This cache is used to store the generated utilities, viewports, custom properties, and content globs.
#[derive(Debug)]
pub struct NemCache {
    pub utilities: Vec<Utility>,
    pub responsive_utilities: Vec<ResponsiveUtility>,
    pub config: NemCssConfig,
    /// CSS custom properties
    /// TODO: use it for custom property auto completion in future PR
    #[allow(dead_code)]
    pub custom_properties: Vec<String>,
    pub content_globs: GlobSet,
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
            custom_properties: generated_css.custom_properties,
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
}
