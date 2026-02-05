use owo_colors::OwoColorize;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;

use miette::Diagnostic;
use thiserror::Error;

use config::{CONFIG_FILE_NAME, NemCssConfig};

use crate::commands::build::glob::{GetContentFilesError, get_content_files};

/// Errors that can occur while building the CSS
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

    #[error("failed to read file content {path}: {source}")]
    #[diagnostic(code(nemcss::build::read_file_content))]
    ReadFileContent {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("unable to write the generated CSS to the output file: {0}")]
    #[diagnostic(code(nemcss::build::write_css))]
    WriteCss(std::io::Error),
}

/// Builds the CSS output file from design tokens and content files.
///
/// This command:
/// - Loads the NemCSS configuration file.
/// - Resolves the design tokens.
/// - Scans the content files for used utility classes.
/// - Generates only the CSS utilities that are actually used
/// - Replaces the `@nemcss base;` directive with the generated CSS.
/// - Writes the generated CSS to the output file.
///
/// # Arguments
///
/// - `input`: The input CSS file containing the `@nemcss base;` directive.
/// - `output`: The output CSS file to write the generated CSS to.
///
/// # Errors
///
/// This function returns an error if any of the following occurs:
/// - Failed to retrieve the current directory.
/// - Failed to load the NemCSS configuration.
/// - Failed to resolve the design tokens.
/// - Failed to get the content files.
/// - Failed to read a file content.
/// - Failed to write the generated CSS to the output file.
pub fn build(
    input: std::path::PathBuf,
    output: std::path::PathBuf,
    quiet: bool,
) -> miette::Result<(), BuildError> {
    let current_dir = std::env::current_dir().map_err(BuildError::RetrieveCurrentDir)?;
    let config_path = current_dir.join(CONFIG_FILE_NAME);
    let config = NemCssConfig::from_path(&config_path)?;

    let resolved_tokens = config.resolve_all_tokens()?;
    let viewports = resolved_tokens.get("viewports");

    let files_to_scan = get_content_files(&config.content, current_dir.as_path())?;

    // Generate the css via css_extractor
    // With try_fold, each thread maintains its own HashSet (no lock, mutex needed)
    // Then, each HashSet result gets merged together with try_reduce
    let used_classes = files_to_scan
        .par_iter()
        .try_fold(HashSet::new, |mut acc, file| {
            let content =
                std::fs::read_to_string(file).map_err(|e| BuildError::ReadFileContent {
                    path: file.to_path_buf(),
                    source: e,
                })?;
            let css = class_extractor::extract_classes(&content);
            acc.extend(css);
            Ok(acc)
        })
        .try_reduce(
            HashSet::new,
            |mut set1, set2| -> Result<HashSet<String>, BuildError> {
                set1.extend(set2);
                Ok(set1)
            },
        )?;

    // write the css to the output directory
    let generated_css =
        engine::generate_css(resolved_tokens.values(), viewports, Some(&used_classes));

    // replace the @nemcss directives
    let input_content =
        std::fs::read_to_string(&input).map_err(|e| BuildError::ReadFileContent {
            path: input.clone(),
            source: e,
        })?;

    if !input_content.contains("@nemcss base;") {
        return Err(BuildError::MissingBaseDirective(
            input.display().to_string(),
        ));
    }

    let output_css = input_content.replace("@nemcss base;", &generated_css.to_css());

    fs::write(&output, output_css).map_err(BuildError::WriteCss)?;

    if !quiet {
        println!("  {} CSS written to {}", "✔".green(), output.display());
    }

    Ok(())
}
