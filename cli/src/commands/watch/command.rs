use miette::{Diagnostic, Result};
use std::path::PathBuf;
use thiserror::Error;

/// WatchErrors are errors that can occur during the watch command.
#[derive(Debug, Diagnostic, Error)]
pub enum WatchErrors {}

pub fn watch(input: PathBuf, output: PathBuf, quiet: bool) -> Result<(), WatchErrors> {
    println!("watch mode!");
    Ok(())
}
