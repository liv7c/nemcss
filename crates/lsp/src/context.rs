//! The goal of this module is to provide helpers to detect
//! the context of the current cursor position.
//! This will be useful to trigger the different completions and hovers
//! at the right time.
use extractor::{
    ASTRO_CLASS_LIST_REGEX, CLASS_ATTRIBUTE_REGEX, CLASS_UTILITY_REGEX, JSX_CLASS_EXPRESSION_REGEX,
    SVELTE_CLASS_BINDING_REGEX, VUE_CLASS_BINDING_REGEX,
};
use regex::Regex;

use ropey::Rope;

/// The maximum number of lines to scan up to for detecting a class context.
pub(crate) const MAX_SCAN_LINES: usize = 15;

/// Marker for the start of a `var(...)` expression.
const VAR_OPEN: &str = "var(";

/// Marker for the start of a token reference expression.
const TOKEN_REF_OPEN: &str = "{";

/// Represents information about the cursor's position within a class context.
/// The "class context" is being used instead of "class name" because
/// the class can take multiple forms, based on the framework being used.
#[derive(Debug, PartialEq)]
pub struct ClassContext {
    /// The partial class name typed so far.
    ///
    /// Examples:
    /// - `class="bg-pr|"`     → partial_token = "bg-pr"
    /// - `class="text-white |"` → partial_token = ""  (just typed a space)
    /// - `class="|"`          → partial_token = ""  (empty attribute)
    pub partial_token: String,
    /// If the user typed a responsive prefix like `sm:`, this holds `"sm"`.
    /// `None` for regular (non-responsive) positions.
    ///
    /// Example: `class="sm:bg-|"` → responsive_prefix = Some("sm")
    pub responsive_prefix: Option<String>,
}

/// Context when the cursor is inside of a `var(--...)` expression
/// It is going to capture whatever is inside the `var(...)` expression.
#[derive(Debug, PartialEq)]
pub struct VarContext {
    /// The partial custom property typed so far
    ///
    /// # Examples
    /// - 'var(--|)' -> partial = "--"
    /// - 'var(--color-|)' -> partial = "--color-"
    /// - 'var(--bg-primary|)' -> partial = "--bg-primary"
    pub partial_property: String,
}

/// Context when the cursor is inside a token reference `{...}` in a JSON value.
#[derive(Debug, PartialEq)]
pub struct TokenRefContext {
    /// The partial token reference typed so far (everything after `{`).
    ///
    /// # Examples
    /// - `"color": "{|"` → partial = ""
    /// - `"color": "{colors.pri|"` → partial = "colors.pri"
    pub partial: String,
}

/// Context when the cursor is at the left-hand side of a CSS custom property declaration.
/// This is used to trigger custom property name completion when a user is overriding a custom property (e.g. overriding a semantic alias like `--text-primary` in a selector)
#[derive(Debug, PartialEq)]
pub struct CssPropertyDeclarationContext {
    /// The partial custom property name typed so far
    ///
    /// # Examples
    /// - `--|` -> partial_name = "--"
    /// - `--bg-|` -> partial_name = "--bg-"
    pub partial_name: String,
}

/// Check whether the `col` (a byte offset) falls inside the value span of
/// a class-related regex match on `line`.
/// The column `col` is the character number. You can think of it as the cursor position.
/// Returns `Some((span_start, span_end))` when the cursor is inside a match.
/// Returns `None` otherwise.
pub fn find_class_span(line: &str, col: usize) -> Option<(usize, usize)> {
    let check = |regex: &Regex, group_name: &str| -> Option<(usize, usize)> {
        for cap in regex.captures_iter(line) {
            let Some(group) = cap.name(group_name) else {
                continue;
            };

            if group.start() <= col && col <= group.end() {
                return Some((group.start(), group.end()));
            }
        }
        None
    };

    None.or_else(|| check(&CLASS_ATTRIBUTE_REGEX, "className"))
        .or_else(|| check(&CLASS_UTILITY_REGEX, "content"))
        .or_else(|| check(&JSX_CLASS_EXPRESSION_REGEX, "content"))
        .or_else(|| {
            check(&VUE_CLASS_BINDING_REGEX, "double")
                .or_else(|| check(&VUE_CLASS_BINDING_REGEX, "single"))
        })
        .or_else(|| {
            check(&ASTRO_CLASS_LIST_REGEX, "quoted")
                .or_else(|| check(&ASTRO_CLASS_LIST_REGEX, "braced"))
        })
        .or_else(|| check(&SVELTE_CLASS_BINDING_REGEX, "className"))
}

/// Detect whether the cursor is inside a class context on the given line.
///
/// Returns `Some(ClassContext)` with the partial token and optional
/// responsive prefix, or `None` if the cursor is not in a class context.
pub fn detect_class_context(line: &str, col: usize) -> Option<ClassContext> {
    let (span_start, _span_end) = find_class_span(line, col)?;

    let before_cursor = &line.get(span_start..col)?;

    let is_delimiter = |c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '(' || c == ',';

    let partial_start = before_cursor
        .rfind(is_delimiter)
        .map(|i| i + 1)
        .unwrap_or(0);

    let partial_token = &before_cursor[partial_start..];

    let (responsive_prefix, final_token) = match partial_token.find(':') {
        Some(colon_pos) => (
            Some(partial_token[..colon_pos].to_string()),
            partial_token[colon_pos + 1..].to_string(),
        ),
        None => (None, partial_token.to_string()),
    };

    Some(ClassContext {
        partial_token: final_token,
        responsive_prefix,
    })
}

/// Builds a combined string from a window of lines around the current line.
/// The window is defined by `max_scan_lines` and the current line.
///
/// It is especially useful to support multiline class context detection.
pub fn build_multiline_window(
    rope: &Rope,
    line_idx: usize,
    col: usize,
    max_scan_lines: usize,
) -> (String, usize) {
    let start_line = line_idx.saturating_sub(max_scan_lines);
    let end_line = (line_idx + max_scan_lines).min(rope.len_lines().saturating_sub(1));

    let mut combined = String::new();
    let mut combined_col: usize = 0;
    for i in start_line..=end_line {
        if i == line_idx {
            combined_col = combined.len() + col;
        }
        for chunk in rope.line(i).chunks() {
            combined.push_str(chunk);
        }
    }

    (combined, combined_col)
}

/// Detects the context of a class inside a multiline string.
/// Handles cases where the class content is spread across multiple lines.
///
///
/// # Returns
/// Returns `Some(ClassContext)` with the partial token and optional
/// responsive prefix, or `None` if the cursor is not in a class context.
///
/// # Limitations
/// The limitation of this function is that it only scans up to [MAX_SCAN_LINES] lines above
/// and below the current line to find the class context.
pub fn detect_multiline_class_context(
    rope: &Rope,
    line_idx: usize,
    col: usize,
) -> Option<ClassContext> {
    let current_line = rope.line(line_idx).to_string();

    if let Some(ctx) = detect_class_context(&current_line, col) {
        return Some(ctx);
    }

    // Go up to MAX_SCAN_LINES lines above and below the current line to search for class context
    let (combined, combined_col) = build_multiline_window(rope, line_idx, col, MAX_SCAN_LINES);

    detect_class_context(&combined, combined_col)
}

/// Detects whether the cursor is inside a `var(--...)` expression.
///
/// Scans backwards to find the start of the `var(...)` expression.
///
/// # Returns
/// It returns `Some(VarContext)` with the partial property name typed so far, or `None` if the cursor is not inside a `var(...)` expression.
pub fn detect_var_context(line: &str, col: usize) -> Option<VarContext> {
    let before_cursor = &line.get(..col)?;

    let var_open = before_cursor.rfind(VAR_OPEN)?;
    let content_start = var_open + VAR_OPEN.len();

    let partial_property = &before_cursor.get(content_start..)?;

    // If the cursor is past the closing parenthesis, we assume it's not inside a var expression
    if partial_property.contains(')') {
        return None;
    }

    Some(VarContext {
        partial_property: partial_property.to_string(),
    })
}

/// Detects whether the cursor is at the left-hand side of a CSS custom property declaration.
pub fn detect_css_property_declaration_context(
    line: &str,
    col: usize,
) -> Option<CssPropertyDeclarationContext> {
    if detect_var_context(line, col).is_some() {
        return None;
    }

    let before_cursor = &line.get(..col)?;

    let after_boundary = match before_cursor.rfind(['{', ';']) {
        Some(pos) => {
            let txt_before_cursor = before_cursor.get(pos + 1..).unwrap_or("");
            if txt_before_cursor.contains(':') {
                return None;
            }
            txt_before_cursor
        }
        None => before_cursor,
    };

    let trimmed = after_boundary.trim_start();
    if !trimmed.starts_with("--") {
        return None;
    }

    Some(CssPropertyDeclarationContext {
        partial_name: trimmed.trim_end().to_string(),
    })
}

/// Extracts the token at the cursor position.
///
/// It scans in **both** directions to extract the full token.
///
/// # Arguments
/// * `line` - The full line content.
/// * `span_start` - The start index of the class content span
/// * `col` - The cursor byte offset
/// * `span_end` - The end index of the class content span
///
/// # Returns
/// Returns `Some(String)` with the extracted token, or `None` if the cursor is not inside a token.
pub fn extract_token_at_cursor(
    line: &str,
    span_start: usize,
    col: usize,
    span_end: usize,
) -> Option<String> {
    let content = &line.get(span_start..span_end)?;
    // column relative to the start of the class content span
    let rel_col = col.checked_sub(span_start)?;

    let is_boundary =
        |c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '(' || c == ',' || c == ')';

    // Check if the cursor is on a boundary char - in this case, we don't want to extract anything
    let cursor_char = content.get(rel_col..)?.chars().next();
    if cursor_char.is_none_or(is_boundary) {
        return None;
    }

    let start = content
        .get(..rel_col)?
        .rfind(is_boundary)
        .map(|i| i + 1)
        .unwrap_or(0);

    let end = content
        .get(rel_col..)?
        .find(is_boundary)
        .map(|i| i + rel_col)
        .unwrap_or(content.len());

    let token = content.get(start..end)?.trim();

    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

/// Extracts the full custom property name under the cursor inside a `var(...)` expression.
///
/// Unlike [`detect_var_context`], which returns the partial property name typed so far,
/// this function returns the full property name.
///
/// # Returns
/// Returns `Some(String)` with the extracted property name, or `None` if the cursor is not inside a `var(...)` expression.
pub fn extract_var_property(line: &str, col: usize) -> Option<String> {
    // If the cursor is not inside a `var(...)` expression, we don't want to extract anything
    detect_var_context(line, col)?;

    let var_open = line.get(..col)?.rfind(VAR_OPEN)? + VAR_OPEN.len();
    let after = line.get(var_open..)?;

    let end = after
        .find(|c: char| c == ',' || c == ')' || c.is_whitespace())
        .unwrap_or(after.len());

    if col > var_open.saturating_add(end) {
        return None;
    }

    let property = after.get(..end)?.trim();

    if property.starts_with("--") && property.len() > 2 {
        Some(property.to_string())
    } else {
        None
    }
}

/// Detects whether the cursor is inside a token reference `{...}` in a JSON value.
///
/// Scans backwards to find the `{` opener, then verifies the cursor is not
/// past a closing `}` and that the `{` appears after a JSON key-value separator.
///
/// # Returns
/// `Some(TokenRefContext)` with the partial token typed so far, or `None` if
/// the cursor is not inside a token reference.
///
/// # Limitations
/// Only scans the current line. A JSON value whose key is on the previous line will not be
/// detected.
pub fn detect_token_ref_context(line: &str, col: usize) -> Option<TokenRefContext> {
    let before_cursor = line.get(..col)?;

    let brace_pos = before_cursor.rfind(TOKEN_REF_OPEN)?;

    // Ensure the `{` is preceded by '"' (to distinguish from a JSON object)
    if before_cursor.get(brace_pos.saturating_sub(1)..brace_pos) != Some("\"") {
        return None;
    }

    // Ensure the `{` appears after a JSON key-value separator
    before_cursor.get(..brace_pos)?.rfind(": ")?;

    let after_open = before_cursor.get(brace_pos + TOKEN_REF_OPEN.len()..)?;

    // Cursor must not be past a closing `}`
    if after_open.contains('}') {
        return None;
    }

    Some(TokenRefContext {
        partial: after_open.to_string(),
    })
}

/// Extracts the partial token reference from a JSON value position.
/// Returns `Some(partial)` if the cursor is inside a token reference, `None` otherwise.
pub fn extract_token_ref_partial(line: &str, col: usize) -> Option<String> {
    detect_token_ref_context(line, col).map(|ctx| ctx.partial)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple helper to parse a cursor position from a string.
    /// You pass a class with a '|' marking the cursor position.
    /// It returns the string + the column number.
    ///
    /// Example: `parse_cursor(r#"<div class="bg-pr|">"#)`
    ///   → line = `<div class="bg-pr">`, col = 17
    fn parse_cursor(input: &str) -> (String, usize) {
        let col = input.find('|').expect("test input must contain '|'");
        let clean = format!("{}{}", &input[..col], &input[col + 1..]);
        (clean, col)
    }

    mod detect_class_context {
        use super::*;

        #[test]
        fn test_inside_class_attribute_empty() {
            let (line, col) = parse_cursor(r#"<div class="|">"#);
            let ctx = detect_class_context(&line, col).expect("should detect context");
            assert_eq!(ctx.partial_token, "");
            assert!(ctx.responsive_prefix.is_none());
        }

        #[test]
        fn test_inside_class_attribute_partial_token() {
            let (line, col) = parse_cursor(r#"<div class="bg-pr|">"#);
            let ctx = detect_class_context(&line, col).expect("should detect context");
            assert_eq!(ctx.partial_token, "bg-pr");
            assert!(ctx.responsive_prefix.is_none());
        }

        #[test]
        fn test_inside_class_attribute_after_space() {
            let (line, col) = parse_cursor(r#"<div class="text-white |">"#);
            let ctx = detect_class_context(&line, col).expect("should detect context");
            assert_eq!(ctx.partial_token, "");
            assert!(ctx.responsive_prefix.is_none());
        }

        #[test]
        fn test_inside_class_attribute_second_class() {
            let (line, col) = parse_cursor(r#"<div class="text-black p-| bg-white">"#);
            let ctx = detect_class_context(&line, col).expect("should detect context");
            assert_eq!(ctx.partial_token, "p-");
            assert!(ctx.responsive_prefix.is_none());
        }

        #[test]
        fn test_inside_classname_jsx() {
            let (line, col) = parse_cursor(r#"<div className="text-|">"#);
            let ctx = detect_class_context(&line, col).expect("should detect context");
            assert_eq!(ctx.partial_token, "text-");
            assert!(ctx.responsive_prefix.is_none());
        }

        #[test]
        fn test_inside_clsx_call() {
            let (line, col) = parse_cursor(r#"className={clsx("bg-|")}"#);
            let ctx = detect_class_context(&line, col).expect("should detect context");
            assert_eq!(ctx.partial_token, "bg-");
            assert!(ctx.responsive_prefix.is_none());
        }

        #[test]
        fn test_inside_cn_call() {
            let (line, col) = parse_cursor(r#"className={cn("text-pr|")}"#);
            let ctx = detect_class_context(&line, col).expect("should detect context");
            assert_eq!(ctx.partial_token, "text-pr");
            assert!(ctx.responsive_prefix.is_none());
        }

        #[test]
        fn test_inside_vue_class_binding() {
            let (line, col) = parse_cursor(r#":class="['text-|']""#);
            let ctx = detect_class_context(&line, col).expect("should detect context");
            assert_eq!(ctx.partial_token, "text-");
            assert!(ctx.responsive_prefix.is_none());
        }

        #[test]
        fn test_responsive_prefix() {
            let (line, col) = parse_cursor(r#"<div class="sm:bg-|">"#);
            let ctx = detect_class_context(&line, col).expect("should detect context");
            assert_eq!(ctx.partial_token, "bg-");
            assert_eq!(ctx.responsive_prefix, Some("sm".to_string()));
        }

        #[test]
        fn test_outside_any_attribute() {
            let (line, col) = parse_cursor(r#"<div |class="foo">"#);
            assert!(detect_class_context(&line, col).is_none());
        }

        #[test]
        fn test_inside_non_class_attribute() {
            let (line, col) = parse_cursor(r#"<div id="some-|">"#);
            assert!(detect_class_context(&line, col).is_none());
        }

        #[test]
        fn test_plain_javascript() {
            let (line, col) = parse_cursor(r#"let p|"#);
            assert!(detect_class_context(&line, col).is_none());
        }

        #[test]
        fn test_inside_script_tag() {
            let (line, col) = parse_cursor(r#"const name = "hello|";"#);
            assert!(detect_class_context(&line, col).is_none());
        }
    }

    mod find_class_span {
        use super::*;

        #[test]
        fn test_inside_class_attribute_empty() {
            let (line, col) = parse_cursor(r#"<div class="bg|">"#);
            let result = find_class_span(&line, col);
            assert_eq!(result, Some((12, 14)));
        }

        #[test]
        fn test_inside_classname_jsx() {
            let (line, col) = parse_cursor(r#"<div className="bg|">"#);
            let result = find_class_span(&line, col);
            assert_eq!(result, Some((16, 18)));
        }

        #[test]
        fn test_inside_clsx_call() {
            let (line, col) = parse_cursor(r#"className={clsx("bg|")}"#);
            let result = find_class_span(&line, col);
            assert_eq!(result, Some((16, 20)));
        }

        #[test]
        fn test_inside_vue_class_binding() {
            let (line, col) = parse_cursor(r#":class="['bg|']""#);
            let result = find_class_span(&line, col);
            assert_eq!(result, Some((8, 14)));
        }

        #[test]
        fn test_outside_any_attribute() {
            let (line, col) = parse_cursor(r#"<div |class="foo">"#);
            assert!(find_class_span(&line, col).is_none());
        }

        #[test]
        fn test_inside_non_class_attribute() {
            let (line, col) = parse_cursor(r#"<div id="some-|">"#);
            assert!(find_class_span(&line, col).is_none());
        }

        #[test]
        fn test_plain_javascript() {
            let (line, col) = parse_cursor(r#"let x = 42;|"#);
            assert!(find_class_span(&line, col).is_none());
        }
    }

    mod detect_multiline_class_context {
        use super::*;

        #[test]
        fn test_works_with_multiline_class_with_clsx() {
            let raw_content = r#"
                <div className={clsx(
                    "text-primary",
                    "bg-|"
                )}>
                    Foo bar
                </div>
            "#;

            let result = detect_multiline_class_context(&Rope::from(raw_content), 3, 24)
                .expect("expect to detect class context");
            assert_eq!(result.partial_token, "bg-");
            assert!(result.responsive_prefix.is_none());
        }

        #[test]
        fn test_works_with_multiline_responsive_class_with_cn() {
            let raw_content = r#"
                <div className={cn(
                    "text-primary",
                    "sm:bg-|"
                )}>
                    Foo bar
                </div>
            "#;

            let result = detect_multiline_class_context(&Rope::from(raw_content), 3, 27)
                .expect("expect to detect class context");
            assert_eq!(result.partial_token, "bg-");
            assert_eq!(result.responsive_prefix, Some("sm".to_string()));
        }

        #[test]
        fn test_returns_early_if_class_content_on_same_line() {
            let raw_content = r#"
                <div className={clsx('bg-|')}>
                    Foo bar
                </div>
            "#;

            let result = detect_multiline_class_context(&Rope::from(raw_content), 1, 41)
                .expect("expect to detect class context");
            assert_eq!(result.partial_token, "bg-");
            assert!(result.responsive_prefix.is_none());
        }

        #[test]
        fn test_returns_none_if_class_content_beyond_max_lines() {
            let raw_content = r#"
                <div className={clsx(
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "text-primary",
                    "bg-|"
                )}>
                    Foo bar
                </div>
            "#;

            let result = detect_multiline_class_context(&Rope::from(raw_content), 24, 24);
            assert!(result.is_none());
        }
    }

    mod extract_token_at_cursor {
        use super::*;

        #[test]
        fn test_extract_full_token_when_cursor_at_start_of_token() {
            let class_content = "bg-primary text-white";
            let result = extract_token_at_cursor(class_content, 0, 0, class_content.len());
            assert_eq!(result, Some("bg-primary".to_string()));
        }

        #[test]
        fn test_extract_full_token_when_cursor_at_end_of_token() {
            let class_content = "bg-primary text-white";
            let result = extract_token_at_cursor(class_content, 0, 9, class_content.len());
            assert_eq!(result, Some("bg-primary".to_string()));
        }

        #[test]
        fn test_extract_token_at_cursor_inside_class() {
            let class_content = r#"class="bg-secondary text-white""#;

            let result = extract_token_at_cursor(class_content, 7, 9, 31);
            assert_eq!(result, Some("bg-secondary".to_string()));

            let result = extract_token_at_cursor(class_content, 7, 29, 31);
            assert_eq!(result, Some("text-white".to_string()));
        }

        #[test]
        fn test_extract_token_at_cursor_outside_class() {
            let class_content = r#"class="bg-secondary text-white""#;

            let result = extract_token_at_cursor(class_content, 7, 19, 31);
            assert!(result.is_none());
        }
    }

    mod detect_var_context {
        use super::*;

        #[test]
        fn test_cursor_after_var_paren() {
            let (line, col) = parse_cursor(r#"var(|");"#);
            let ctx = detect_var_context(&line, col).expect("should detect context");
            assert_eq!(ctx.partial_property, "");
        }

        #[test]
        fn test_cursor_after_double_dashes() {
            let (line, col) = parse_cursor(r#"var(--|");"#);
            let ctx = detect_var_context(&line, col).expect("should detect context");
            assert_eq!(ctx.partial_property, "--");
        }

        #[test]
        fn test_cursor_mid_property_name() {
            let (line, col) = parse_cursor(r#"var(--bg-|);"#);
            let ctx = detect_var_context(&line, col).expect("should detect context");
            assert_eq!(ctx.partial_property, "--bg-");
        }

        #[test]
        fn test_cursor_after_property_name() {
            let (line, col) = parse_cursor(r#"var(--bg-primary|);"#);
            let ctx = detect_var_context(&line, col).expect("should detect context");
            assert_eq!(ctx.partial_property, "--bg-primary");
        }

        #[test]
        fn test_var_in_jsx_inline_style() {
            let (line, col) = parse_cursor(r#"<div style={{color: "var(--|)"}}/>"#);
            let ctx = detect_var_context(&line, col).expect("should detect context");
            assert_eq!(ctx.partial_property, "--");
        }

        #[test]
        fn test_var_in_template_literal() {
            let (line, col) = parse_cursor("const css = `color: var(--spacing-|);");
            let ctx = detect_var_context(&line, col).expect("should detect context");
            assert_eq!(ctx.partial_property, "--spacing-");
        }

        #[test]
        fn test_not_inside_var() {
            let (line, col) = parse_cursor(r#"const name = "hello|";"#);
            assert!(detect_var_context(&line, col).is_none());
        }

        #[test]
        fn test_cursor_outside_var() {
            let (line, col) = parse_cursor("color: var(--bg-primary)|;");
            assert!(detect_var_context(&line, col).is_none());
        }

        #[test]
        fn test_cursor_past_closing_paren() {
            let (line, col) = parse_cursor(r#"var(--bg-primary)|;"#);
            assert!(detect_var_context(&line, col).is_none());
        }

        #[test]
        fn test_var_with_fallback_cursor_before_comma() {
            let (line, col) = parse_cursor(r#"var(--bg-primary|, black);"#);
            let ctx = detect_var_context(&line, col).expect("should detect context");
            assert_eq!(ctx.partial_property, "--bg-primary");
        }
    }

    mod extract_var_property {
        use super::*;

        #[test]
        fn test_inside_var_expression() {
            let line = "color: var(--bg-primary);";
            let result = extract_var_property(line, 16).expect("should extract property");
            assert_eq!(result, "--bg-primary");
        }

        #[test]
        fn test_beginning_of_var_expression() {
            let line = "color: var(--bg-primary);";
            let result = extract_var_property(line, 12).expect("should extract property");
            assert_eq!(result, "--bg-primary");
        }

        #[test]
        fn test_cursor_at_end_of_var_expression() {
            let line = "color: var(--bg-primary);";
            let result = extract_var_property(line, 23).expect("should extract property");
            assert_eq!(result, "--bg-primary");
        }

        #[test]
        fn test_cursor_on_fallback_value() {
            let line = "color: var(--bg-primary, black);";
            let col = 28;
            assert!(extract_var_property(line, col).is_none());
        }

        #[test]
        fn test_returns_none_if_cursor_not_inside_var() {
            let line = "--bg-primary";
            let col = 4;
            assert!(extract_var_property(line, col).is_none());
        }
    }

    mod extract_token_ref_partial {
        use super::*;

        #[test]
        fn test_returns_none_when_no_brace() {
            let line = r#"  "color": "bar"#;
            assert!(extract_token_ref_partial(line, line.len()).is_none());
        }

        #[test]
        fn test_returns_none_when_inside_json_object_value() {
            let line = r#"  "color": {"nested": {"val"}}"#;
            assert!(extract_token_ref_partial(line, line.len()).is_none());
        }

        #[test]
        fn test_returns_none_when_brace_but_no_json_separator() {
            let line = "const foo = {colors.primary";
            assert!(extract_token_ref_partial(line, line.len()).is_none());
        }

        #[test]
        fn test_returns_none_when_past_closing_brace() {
            let line = r#"  "color": "{colors.primary}""#;
            assert!(extract_token_ref_partial(line, line.len()).is_none());
        }

        #[test]
        fn test_returns_empty_partial_when_just_opened() {
            let line = r#"  "color": "{"#;
            assert_eq!(
                extract_token_ref_partial(line, line.len()),
                Some(String::new())
            );
        }

        #[test]
        fn test_returns_partial_token() {
            let line = r#"  "color": "{colors.pri"#;
            assert_eq!(
                extract_token_ref_partial(line, line.len()),
                Some("colors.pri".to_string())
            );
        }
    }

    mod detect_css_property_declaration_context {
        use super::*;

        fn detect(input: &str) -> Option<CssPropertyDeclarationContext> {
            let (line, col) = parse_cursor(input);
            detect_css_property_declaration_context(&line, col)
        }

        #[test]
        fn test_detects_bare_double_dash_at_line_start() {
            let ctx = detect("--|").expect("should detect");
            assert_eq!(ctx.partial_name, "--");
        }

        #[test]
        fn test_does_not_detect_at_property_value_when_rule_is_inline() {
            assert!(detect("body { --text-primary: --|}").is_none());
        }

        #[test]
        fn test_detects_indented_double_dash() {
            let ctx = detect("    --|").expect("should detect");
            assert_eq!(ctx.partial_name, "--");
        }

        #[test]
        fn test_detects_partial_property_name() {
            let ctx = detect("--bg-|").expect("should detect");
            assert_eq!(ctx.partial_name, "--bg-");
        }

        #[test]
        fn test_detects_after_opening_brace_on_same_line() {
            let ctx = detect("{ --bg-|").expect("should detect");
            assert_eq!(ctx.partial_name, "--bg-");
        }

        #[test]
        fn test_does_not_detect_inside_var_expression() {
            assert!(detect("color: var(--bg-|);").is_none());
        }

        #[test]
        fn test_does_not_detect_at_property_value_position() {
            assert!(detect("color: --|;").is_none());
        }

        #[test]
        fn test_does_not_detect_at_standard_property_name() {
            assert!(detect("body { color|").is_none());
        }
    }
}
