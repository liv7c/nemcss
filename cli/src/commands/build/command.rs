use miette::Diagnostic;
use thiserror::Error;

use config::{CONFIG_FILE_NAME, NemCssConfig};

#[derive(Debug, Error, Diagnostic)]
pub enum BuildError {
    #[error("failed to retrieve the current directory: {0}")]
    #[diagnostic(code(nemcss::build::current_dir))]
    RetrieveCurrentDir(std::io::Error),

    #[error("failed to load the NemCSS configuration: {0}")]
    #[diagnostic(code(nemcss::build::load_config))]
    LoadConfig(#[from] config::NemCssConfigError),

    #[error("failed to resolve the design tokens: {0}")]
    #[diagnostic(code(nemcss::build::resolve_tokens))]
    ResolveTokens(#[from] config::ResolveTokensError),
}

pub fn build(
    input: std::path::PathBuf,
    output: std::path::PathBuf,
) -> miette::Result<(), BuildError> {
    let current_dir = std::env::current_dir().map_err(BuildError::RetrieveCurrentDir)?;
    let config_path = current_dir.join(CONFIG_FILE_NAME);
    let config = NemCssConfig::from_path(&config_path)?;

    let resolved_tokens = config.resolve_all_tokens()?;
    let viewports = resolved_tokens.get("viewports");

    let generated_css = engine::generate_css(resolved_tokens.values(), viewports);
    dbg!(&generated_css);
    todo!()
}
