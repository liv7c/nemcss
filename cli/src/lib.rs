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
    ///
    /// Example usage:
    /// nemcss init
    Init,

    /// Generates CSS from design tokens by scanning content files for used utility classes.
    ///
    /// This command reads your desing tokens, scans your source files (HTML, JSX, Vue, etc.)
    /// for used utility classes, and generates only the CSS utilities that are actually used.
    /// The generated CSS replaces the `@nemcss base;` directive in your input file.
    ///
    /// Example usage:
    /// nemcss build -i src/input.css -o dist/output.css
    Build {
        /// The path to the CSS input file.
        ///
        /// This file must contain the `@nemcss base;` directive., which will be
        /// replaced with the generated CSS custom properties and utility classes.
        #[arg(short, long)]
        input: PathBuf,

        /// The path to the CSS output file.
        ///
        /// The final CSS with all custom properties and utility classes will be written to this file.
        #[arg(short, long)]
        output: PathBuf,

        /// Suppress output messages
        #[arg(short, long, default_value_t = false)]
        quiet: bool,
    },
    /// Watches for changes in your design tokens and source files, and automatically rebuilds the CSS.
    ///
    /// Example usage:
    /// nemcss watch -i src/input.css -o dist/output.css
    Watch {
        /// The path to the CSS input file.
        #[arg(short, long)]
        input: PathBuf,
        /// The path to the CSS output file.
        #[arg(short, long)]
        output: PathBuf,
        /// Suppress output messages
        #[arg(short, long, default_value_t = false)]
        quiet: bool,
    },
}

/// The main entry point for the `nemcss` CLI.
pub fn run() -> miette::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => commands::init()?,
        Command::Build {
            input,
            output,
            quiet,
        } => commands::build(input, output, quiet)?,
        Command::Watch {
            input,
            output,
            quiet,
        } => commands::watch(input, output, quiet)?,
    }

    Ok(())
}
