use crate::commands::new_token_file::{error::NewTokenFileError, scale::ScaleSource};
use miette::Result;

/// Command to generate a new token file
pub fn new_token_file(
    name: &str,
    source: ScaleSource,
    names: Option<Vec<String>>,
    unit: &str,
    prefix: Option<String>,
    force: bool,
) -> Result<(), NewTokenFileError> {
    todo!()
}
