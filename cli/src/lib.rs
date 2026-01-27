//! A library crate for the `nemcss` CLI.
//!
//! This crate provides a CLI interface for the `nemcss` project, a design-token-driven CSS utility generator.
//!
//! # Commands
//!
//! The `nemcss` CLI provides the following commands:
//!
//! - `init`: Initialize a new project with the `nemcss` configuration and example design tokens.
//!
//! # Example
//!
//! To initialize a nemcss project, run the following command:
//!
//! ```bash
//! nemcss init
//! ```
//!
//! This will create at the current directory a `nemcss.config.json` file with the default configuration as well as a `design-tokens` directory (if it doesn't exist) with an example design token file.
pub mod commands;

use clap::{Parser, Subcommand};

/// A CLI for the `nemcss` project, a design-token-driven CSS utility generator.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The subcommand to run.
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize a new project with the `nemcss` configuration and example design tokens.
    Init,
}

/// The main entry point for the `nemcss` CLI.
pub fn run() -> miette::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => commands::init()?,
    }

    Ok(())
}
