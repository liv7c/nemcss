//! A library crate for the `nemcss` CLI.
//!
//! This crate provides a CLI interface for the `nemcss` project, a design-token-driven CSS utility generator.
//!
//! # Commands
//!
//! The `nemcss` CLI provides the following commands:
//!
//! - `init`: Initializes a new project with the `nemcss` configuration and example design tokens.
//!
//! # Example
//!
//! To initialize a nemcss project, run the following command:
//!
//! ```bash
//! nemcss init
//! ```
//!
//! This will create, at the root of your current directory, a `nemcss.config.json` file as well as a `design-tokens` directory (if it doesn't already exist) with two example design token files.
pub mod commands;

use std::path::PathBuf;

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
    /// Initializes a new project with the `nemcss` configuration and example design tokens.
    Init,

    /// Builds the CSS files for the project.
    Build {
        /// The path to the CSS input file.
        #[arg(short, long)]
        input: PathBuf,

        /// The path to the CSS output file.
        #[arg(short, long)]
        output: PathBuf,
    },
}

/// The main entry point for the `nemcss` CLI.
pub fn run() -> miette::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => commands::init()?,
        Command::Build { input, output } => commands::build(input, output)?,
    }

    Ok(())
}
