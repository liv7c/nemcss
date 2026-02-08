use std::path::Path;

use config::{CONFIG_FILE_NAME, NemCssConfig, NemCssConfigError, ResolveTokensError};
use engine::{ResponsiveUtility, Utility};
use globset::GlobSet;
use miette::Diagnostic;
use thiserror::Error;

/// Cache for the LSP server.
/// This cache is used to store the generated utilities, viewports, custom properties, and content globs.
#[derive(Debug)]
pub struct NemCache {
    pub utilities: Vec<Utility>,
    pub responsive_utilities: Vec<ResponsiveUtility>,
    pub config: NemCssConfig,
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
}
