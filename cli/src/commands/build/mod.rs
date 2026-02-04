//! Build command implementation for NemCSS.
//!
//! This module contains the implementation of the `build` command which generates CSS
//! from design tokens by scanning content files for used utility classes.

mod command;
mod glob;

pub use command::build;
