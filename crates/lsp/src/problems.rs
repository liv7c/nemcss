use std::path::{Path, PathBuf};

use config::{NemCssConfigError, ResolveSemanticError, ResolveTokensError};
use miette::Diagnostic;
use tower_lsp::lsp_types::{Diagnostic as LspDiagnostic, DiagnosticSeverity};
use tower_lsp::lsp_types::{Position, Range};

/// Represents the severity of the problem.
/// Warning will be for unused files for instance.
/// Error will be for broken tokens or files we cannot parse.
#[derive(Debug, PartialEq, Clone)]
pub enum Severity {
    Warning,
    Error,
}

/// Problem contains the key information we need to show
/// good errors in the user editor.
#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    pub path: PathBuf,
    pub severity: Severity,
    pub message: String,
    pub help: Option<String>,
    pub range: Range,
}

impl Problem {
    /// Create a problem struct for unregistered token file errors
    pub fn unregistered_token_file(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            severity: Severity::Warning,
            message: "token file is not registered in nemcss.config.json; ignoring it".into(),
            help: Some("add a theme entry with \"source\" pointing to this file and a \"prefix\", or remove this file".into()),
            range: Range::default()
        }
    }

    /// Create a Problem from a token error.
    pub fn from_token_error(err: &ResolveTokensError, config_path: &Path) -> Self {
        let help = err.help().map(|h| h.to_string());
        let message = config::display_error_chain(err);

        match err {
            ResolveTokensError::LoadTokensFromFileError(load_tokens_from_file_error) => {
                let range = match load_tokens_from_file_error {
                    config::LoadTokensFromFileError::ReadFileError { .. } => Range::default(),
                    config::LoadTokensFromFileError::ParseError { source, .. } => {
                        range_from_json_error(source)
                    }
                    config::LoadTokensFromFileError::InvalidTokenName { name, path } => {
                        let file_content = std::fs::read_to_string(path).unwrap_or_default();
                        range_of_field(&file_content, "name", name)
                    }
                };

                Self {
                    path: load_tokens_from_file_error.path().to_path_buf(),
                    severity: Severity::Error,
                    message,
                    help,
                    range,
                }
            }
            ResolveTokensError::ScanTokensDirError(_) => Self {
                path: config_path.to_path_buf(),
                severity: Severity::Error,
                message,
                help,
                range: Range::default(),
            },
            ResolveTokensError::SourceFileNotFound { source_path, .. } => {
                let config_text = std::fs::read_to_string(config_path).unwrap_or_default();
                Self {
                    path: config_path.to_path_buf(),
                    severity: Severity::Error,
                    message,
                    help,
                    range: range_of_field(
                        &config_text,
                        "source",
                        &source_path.display().to_string(),
                    ),
                }
            }
            ResolveTokensError::UnregisteredTokenFile { .. } => Self {
                path: config_path.to_path_buf(),
                severity: Severity::Error,
                message,
                help,
                range: Range::default(),
            },
        }
    }

    /// Show problem for semantic token error
    pub fn from_semantic_error(err: &ResolveSemanticError, config_path: &Path) -> Self {
        let ResolveSemanticError::UnresolvableReference { reference, .. } = err;
        let config_text = std::fs::read_to_string(config_path).unwrap_or_default();

        Self {
            path: config_path.to_path_buf(),
            severity: Severity::Error,
            message: config::display_error_chain(err),
            help: err.help().map(|h| h.to_string()),
            range: range_of_quoted(&config_text, reference),
        }
    }

    /// Show problem for config error
    pub fn from_config_error(err: &NemCssConfigError, config_path: &Path) -> Self {
        let range = match err {
            NemCssConfigError::ReadConfigFile(_) => Range::default(),
            NemCssConfigError::ParseConfigFile(json_err) => range_from_json_error(json_err),
        };

        Self {
            path: config_path.to_path_buf(),
            severity: Severity::Error,
            message: config::display_error_chain(err),
            help: err.help().map(|h| h.to_string()),
            range,
        }
    }

    /// Converts a custom Problem into a LSP diagnostic.
    pub fn to_diagnostic(&self) -> LspDiagnostic {
        let message = match &self.help {
            Some(help) => format!("{}\n\nhelp: {help}", self.message),
            None => self.message.clone(),
        };

        LspDiagnostic {
            range: self.range,
            severity: Some(match self.severity {
                Severity::Warning => DiagnosticSeverity::WARNING,
                Severity::Error => DiagnosticSeverity::ERROR,
            }),
            source: Some("nemcss".into()),
            message,
            ..Default::default()
        }
    }
}

/// Compute the LSP range based on the full file text, the byte offset and the string to match for.
fn range_at(text: &str, offset: usize, needle: &str) -> Range {
    let before = &text[..offset];
    let line = before.matches('\n').count() as u32;
    let column = before.rsplit('\n').next().unwrap_or(before).chars().count() as u32;

    let start = Position::new(line, column);
    let end = Position::new(line, column + needle.chars().count() as u32);

    Range::new(start, end)
}

/// Get range value for quoted value.
/// It will be used to show errors for unresolvable reference for instance.
fn range_of_quoted(text: &str, value: &str) -> Range {
    let quoted = format!("\"{value}\"");
    let Some(offset) = text.find(&quoted) else {
        return Range::default();
    };

    range_at(text, offset, &quoted)
}

/// Get range value for errors such as invalid token name or source file not found.
fn range_of_field(text: &str, key: &str, value: &str) -> Range {
    let quoted_value = format!("\"{value}\"");
    let needle = format!("\"{key}\": {quoted_value}");
    let Some(offset) = text.find(&needle) else {
        return Range::default();
    };

    // get the offset to land at the start of the "value" itself
    let value_offset = offset + needle.len() - quoted_value.len();

    range_at(text, value_offset, &quoted_value)
}

/// Get range value from json error.
/// A serde JSON error already has information such as line and column numbers.
fn range_from_json_error(err: &serde_json::Error) -> Range {
    let position = Position::new(
        // LSP protocol is 0-based while JSON error for line and column are 1-based
        (err.line() as u32).saturating_sub(1),
        (err.column() as u32).saturating_sub(1),
    );

    Range::new(position, position)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_of_quoted_finds_line_and_column_of_the_quoted_needle() {
        let text = r#"{
    "items": [
        { "name": "0.5" }
    ]
}"#;
        let range = range_of_quoted(text, "0.5");
        assert_eq!(range.start, Position::new(2, 18));
        assert_eq!(
            range.end,
            Position::new(2, 23),
            "end covers the closing quote"
        );
    }

    #[test]
    fn range_of_quoted_falls_back_to_the_top_of_the_file() {
        assert_eq!(range_of_quoted("{}", "missing"), Range::default());
    }

    #[test]
    fn range_of_field_finds_the_value_under_the_given_key() {
        let text = r#"{
    "items": [
        { "name": "0.5" }
    ]
}"#;
        let range = range_of_field(text, "name", "0.5");

        assert_eq!(range.start, Position::new(2, 18));
        assert_eq!(
            range.end,
            Position::new(2, 23),
            "end covers the closing quote"
        );
    }

    #[test]
    fn range_of_field_ignores_the_same_value_under_a_different_key() {
        let text = r#"{
    "items": [
        { "name": "spacing", "value": "0.5" },
        { "name": "0.5" }
    ]
}"#;
        let range = range_of_field(text, "name", "0.5");

        assert_eq!(
            range.start,
            Position::new(3, 18),
            "should point at line 4's \"name\", not line 3's \"value\""
        );
        assert_eq!(range.end, Position::new(3, 23));
    }

    #[test]
    fn range_from_json_error_is_zero_based() {
        let err = serde_json::from_str::<serde_json::Value>("{\n  \"a\": 1,\n}").unwrap_err();

        let range = range_from_json_error(&err);

        assert_eq!(range.start, Position::new(2, 0));
        assert_eq!(range.start, range.end, "should be a zero-width range");
    }
}
