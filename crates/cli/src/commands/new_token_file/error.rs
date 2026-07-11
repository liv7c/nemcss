use miette::Diagnostic;
use thiserror::Error;

/// Error type for the `new-token-file` command.
#[derive(Debug, Diagnostic, Error)]
pub enum NewTokenFileError {
    #[error(
        "name count does not match number of values provided: expected {expected} names, got {got} instead"
    )]
    #[diagnostic(code(nemcss::new_token_file::name_count_mismatch))]
    NameCountMismatch { expected: usize, got: usize },
    #[error(
        "value {value:?} is not a number, so it cannot automically be named automatically. Use the --names argument"
    )]
    #[diagnostic(code(nemcss::new_token_file::name_required_for_value))]
    NameRequiredForValue { value: String },
}
