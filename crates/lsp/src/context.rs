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

/// Represents information about the cursor's position within a class context.
/// The "class context" is being used instead of "class name" because
/// the class can take multiple forms, based on the framework being used.
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

    let before_cursor = &line[span_start..col];

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

/// Detects the context of a class inside a multiline string.
/// Handles cases where the class content is spread across multiple lines.
///
///
/// # Returns
/// Returns `Some(ClassContext)` with the partial token and optional
/// responsive prefix, or `None` if the cursor is not in a class context.
///
/// # Limitations
/// The limitation of this function is that it only scans up to 15 lines above
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

    // Go up to 15 lines above and below the current line to search for class context
    const MAX_SCAN_LINES: usize = 15;
    let start_line = line_idx.saturating_sub(MAX_SCAN_LINES);
    let end_line = (line_idx + MAX_SCAN_LINES).min(rope.len_lines().saturating_sub(1));

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

    detect_class_context(&combined, combined_col)
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
}
