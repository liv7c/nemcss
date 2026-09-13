/// Renders an error followed by each of its causes on its own line.
///
/// Errors in the config crate keeps their cause as a source instead of
/// inline it in the error message. This function is fow anything (LSP, Node bindings)
/// that might need to show the full context of an error.
pub fn display_error_chain(err: &dyn std::error::Error) -> String {
    use std::fmt::Write;

    let mut text = err.to_string();
    let mut source = err.source();

    while let Some(cause) = source {
        let _ = write!(text, "\n caused by: {cause}");
        source = cause.source();
    }

    text
}
