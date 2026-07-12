use inquire::{CustomType, Select};
use inquire::{InquireError, Text};
use miette::Result;

use crate::commands::TokenFileRequest;
use crate::commands::new_token_file::error::NewTokenFileError;
use crate::commands::{ScaleSource, split_values};

const MODE_EXPLICIT: &str = "Explicit values (hand-curated scale - e.g. orange,blue,green)";
const MODE_GENERATED: &str = "Generated scale (uniform steps - e.g. 0, 0.5, 1, 1.5)";
const MODE_PLACEHOLDER: &str = "Placeholder (empty items to fill after file generation)";

/// Collects a TokenFileRequest by prompting.
/// Instead of collecting the new token file options via flag, it prompts
/// a user through a series of questions to then build a TokenFileRequest used down
/// the line to create a new token file.
pub fn interative_request(name: Option<String>) -> Result<TokenFileRequest, NewTokenFileError> {
    let name = match name {
        Some(name) => name,
        None => Text::new("Token file name (e.g. spacing):")
            .prompt()
            .map_err(prompt_err)?,
    };

    let mode = Select::new(
        "How should the values be produced?",
        vec![MODE_EXPLICIT, MODE_GENERATED, MODE_PLACEHOLDER],
    )
    .prompt()
    .map_err(prompt_err)?;

    let (source, names, unit) = match mode {
        MODE_EXPLICIT => {
            let raw = Text::new("Values (comma-separated):")
                .prompt()
                .map_err(prompt_err)?;

            (
                ScaleSource::Explicit(split_values(&raw)),
                prompt_names()?,
                prompt_unit()?,
            )
        }
        MODE_GENERATED => {
            let step = CustomType::<f64>::new("Step between values:")
                .prompt()
                .map_err(prompt_err)?;
            let count = CustomType::<usize>::new("How many values?")
                .prompt()
                .map_err(prompt_err)?;
            let start = CustomType::<f64>::new("First value:")
                .with_default(step)
                .prompt()
                .map_err(prompt_err)?;

            (
                ScaleSource::Generated { start, step, count },
                prompt_names()?,
                prompt_unit()?,
            )
        }
        _ => (ScaleSource::Placeholder, None, String::new()),
    };

    let prefix = Text::new(
        "Prefix for custom properties: (e.g. color, space) - defaults to token file name",
    )
    .with_default(&name)
    .prompt()
    .map_err(prompt_err)?;

    Ok(TokenFileRequest {
        prefix,
        name,
        source,
        names,
        unit,
    })
}

fn prompt_names() -> Result<Option<Vec<String>>, NewTokenFileError> {
    let raw = Text::new("Names (comma-separated, empty to name by value):")
        .prompt()
        .map_err(prompt_err)?;
    if raw.trim().is_empty() {
        return Ok(None);
    }

    Ok(Some(raw.split(',').map(|s| s.trim().to_string()).collect()))
}

fn prompt_unit() -> Result<String, NewTokenFileError> {
    Text::new("Unit (e.g. px, rem - leave empty when no unit):")
        .prompt()
        .map_err(prompt_err)
}

/// Helper to distinguish not a terminal error from other types of errors
fn prompt_err(err: InquireError) -> NewTokenFileError {
    match err {
        InquireError::NotTTY => NewTokenFileError::NotATerminal,
        other => NewTokenFileError::Prompt(other),
    }
}
