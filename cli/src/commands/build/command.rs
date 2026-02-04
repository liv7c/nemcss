use owo_colors::OwoColorize;
use std::collections::HashSet;
use std::fs;

use miette::Diagnostic;
use thiserror::Error;

use config::{CONFIG_FILE_NAME, NemCssConfig};

use crate::commands::build::glob::{GetContentFilesError, get_content_files};

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

    #[error("missing `@nemcss base;` directive in input css file: {0}")]
    #[diagnostic(code(nemcss::build::missing_base_directive))]
    MissingBaseDirective(String),

    #[error("failed to get the content files: {0}")]
    #[diagnostic(code(nemcss::build::get_content_files))]
    GetContentFiles(#[from] GetContentFilesError),

    #[error("failed to read file content: {0}")]
    #[diagnostic(code(nemcss::build::read_file_content))]
    ReadFileContent(#[from] std::io::Error),

    #[error("unable to write the generated CSS to the output file: {0}")]
    #[diagnostic(code(nemcss::build::write_css))]
    WriteCss(std::io::Error),
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

    let files_to_scan = get_content_files(&config.content, current_dir.as_path())?;

    let mut used_classes = HashSet::new();

    // generate the css via css_extractor
    for file in files_to_scan {
        // TODO: see how to optimize to pass an iterator maybe instead of the file content?
        let content = std::fs::read_to_string(file)?;
        let css = class_extractor::extract_classes(&content);
        used_classes.extend(css);
    }

    // write the css to the output directory
    let generated_css =
        engine::generate_css(resolved_tokens.values(), viewports, Some(&used_classes));

    // replace the @nemcss directives
    let input_content = std::fs::read_to_string(&input)?;

    if !input_content.contains("@nemcss base;") {
        return Err(BuildError::MissingBaseDirective(
            input.display().to_string(),
        ));
    }

    let output_css = input_content.replace("@nemcss base;", &generated_css.to_css());

    fs::write(&output, output_css).map_err(BuildError::WriteCss)?;

    println!("  {} CSS written to {}", "✔".green(), output.display());

    Ok(())
}
