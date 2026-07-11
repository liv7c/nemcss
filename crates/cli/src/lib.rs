//! CLI for nemcss - a design-token-driven CSS utility generator.
//!
//! This crate provides a command-line interface for generating CSS from design tokens.
//! It scans your source files for used utility classes, and generates only the CSS utilities that are actually used.
//!
//! # Commands
//!
//! ## init
//!
//! Initializes a new project with the `nemcss` configuration and example design tokens.
//!
//! ```bash
//! nemcss init
//! ```
//!
//! Creates:
//! - `nemcss.config.json`: The configuration file for the project.
//! - `design-tokens`: A directory containing example design tokens.
//!
//! ## build
//!
//! Generates CSS from design tokens and source files.
//!
//! ```bash
//! nemcss build -i src/input.css -o dist/output.css
//! ```
//!
//! This command scans files matching the content glob pattern in your `nemcss.config.json` file.
//! It extracts the utility classes used in those files, and generates only the CSS utilities that
//! are actually used.
//!
//! ## watch
//!
//! Watches for changes in your design tokens and source files, and automatically rebuilds the CSS.
//!
//! ```bash
//! nemcss watch -i src/input.css -o dist/output.css
//! ```
//!
//! This command watches:
//! - Content files (matching the content glob pattern in your `nemcss.config.json` file)
//! - Design token files
//! - Configuration file (`nemcss.config.json`)
//! - Input CSS file
//!
//! ## new-token-file
//!
//! Creates a new token file in the design tokens folder and registers it in the `nemcss.config.json`.
//!
//! ```bash
//! nemcss new-token-file radius --names "xs,sm,md" --values "2,4,8" --unit "px"
//! nemcss new-token-file spacing --unit "rem" --step 0.5 --count 12
//! nemcss new-token-file font-size --unit "rem" --values "1,clamp(1.5rem, 1rem + 2vw, 2.5rem)" --names "md,fluid"
//! nemcss new-token-file max-width
//! ```
//!
//! This command:
//! - create a new design token file named based on the first positional argument provided
//! - registers the new design token config in the `nemcss.config.json`
//!
//! Values can be numbers (`--unit` is appended) or arbitrary CSS like `clamp(...)`.
//! You can also generate scaled values via the `--step` and `--count` flags.
//! Existing token files are never overwritten without the `--force` flag.
//!
//! ## schema
//!
//! Prints to stdout the JSON schema for `NemCssConfig`.
//! The schema is derived directly from the Rust type definitions.
//! Useful for piping into a file or validating the schema shape.
//!
//! ```bash
//! nemcss schema > nemcss.schema.json
//! ```
//!
//! # Configuration
//!
//! The `nemcss.config.json` file controls which files are scanned:
//!
//! ```json
//! {
//!     "content": ["src/**/*.html", "src/**/*.css"],
//!     "tokensDir": "design-tokens",
//! }
//! ```
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
    /// This command monitors your file system for changes and triggers a rebuild as needed.
    /// It watches:
    /// - Content files (matching the content glob pattern in your `nemcss.config.json` file)
    /// - Design token files
    /// - Configuration file (`nemcss.config.json`)
    /// - Input CSS file
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
    },

    /// Generates a new token file and registers it in `nemcss.config.json`.
    ///
    /// Provide explicit values or generate a uniform scale from a step and count. There is also
    /// the option to generate a placeholder token file.
    ///
    /// Example usage:
    /// nemcss new-token-file spacing --unit px --values "8,16,24,32" --names "sm,md,lg,xl"
    /// newmcss new-tokenf-file spacing --unit rem --step 0.5 --count 12
    #[command(visible_alias = "ntf")]
    NewTokenFile {
        /// Name of the token file (e.g. "spacing" creates a spacing.json file)
        name: String,
        /// Comma-separated values: numbers get --unit appended. Anything else is kept as such.
        #[arg(long, conflicts_with_all = ["step", "count", "start"])]
        values: Option<String>,

        /// Step between generated scale values. Requires --count
        #[arg(long, requires = "count")]
        step: Option<f64>,

        /// Number of values to generate. Requires --step
        #[arg(long, requires = "step")]
        count: Option<usize>,

        /// First value of generated scale (defaults to --step)
        #[arg(long, requires = "step")]
        start: Option<f64>,

        /// Comma-separated names for the values (e.g. "sm, md, lg, xl")
        #[arg(long, value_delimiter = ',')]
        names: Option<Vec<String>>,

        /// CSS unit appended to each value (e.g. "px", "rem")
        #[arg(long, default_value = "")]
        unit: String,

        /// Prefix for the generated custom properties (defaults to the file name)
        #[arg(long)]
        prefix: Option<String>,

        /// Overwrite an existing token file and theme entry.
        #[arg(long, default_value_t = false)]
        force: bool,
    },

    /// Generates a JSON schema for `NemCssConfig` as a pretty-printed JSON string.
    /// The schema is derived directly from the Rust type definitions.
    ///
    /// Useful for piping into a file or validating the schema shape.
    /// Example: nemcss schema > nemcss.schema.json
    Schema,
}

/// Parses command-line arguments and runs the requested command.
///
/// This is the main entry point for the `nemcss` CLI.
pub fn run() -> miette::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => commands::init()?,
        Command::Build {
            input,
            output,
            quiet,
        } => commands::build(input, output, quiet)?,
        Command::Watch { input, output } => commands::watch(input, output)?,
        Command::Schema => commands::schema()?,
        Command::NewTokenFile {
            name,
            values,
            step,
            count,
            start,
            names,
            unit,
            prefix,
            force,
        } => {
            let source = match (values, step) {
                (Some(values), _) => {
                    commands::ScaleSource::Explicit(commands::split_values(&values))
                }
                (None, Some(step)) => commands::ScaleSource::Generated {
                    start: start.unwrap_or(step),
                    step,
                    count: count.expect("clap enforces --step requires --count"),
                },
                (None, None) => commands::ScaleSource::Placeholder,
            };

            let request = commands::TokenFileRequest {
                prefix: prefix.unwrap_or(name.clone()),
                name,
                source,
                names,
                unit,
            };
            commands::new_token_file(request, force)?
        }
    }

    Ok(())
}
