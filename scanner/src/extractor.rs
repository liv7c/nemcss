//! # Extractor module
//!
//! This module contains the logic for extracting CSS classes from a given file. It heavily relies
//! on the a series of regular expressions to identify CSS classes and their usage.

use std::sync::LazyLock;

use regex::Regex;

/// A lazy-loaded regex for matching CSS classes using the `class` attribute (or classname /
/// className).
/// Matches: class="foo bar", className="foo bar", classname="foo bar", :class="foo bar",
/// :classname="foo bar" and :className="foo bar"
static CLASS_ATTRIBUTE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#":?class(name|Name)?=["'](?<className>[^"']+)["']"#)
        .expect("Failed to compile class attribute regex")
});

/// A regex for matching CSS classes using the classnames(...), clsx(...) or cn(...)
/// It uses a greedy match to capture everything that is in between the parentheses (as we could
/// have arrays, objects, etc.).
static CLSX_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(clsx|classnames|cn)\((?<clsx_content>.*)\)"#)
        .expect("Failed to compile clsx regex")
});

/// A regex for matching string literals
static STRING_LITERAL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"["'](?<string_literal>[^"']+)["']"#)
        .expect("Failed to compile string literal regex")
});

/// A regex for matching object keys
static OBJECT_KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\{*\s*["']?(?<object_key>[^"']+)["']?:"#)
        .expect("Failed to compile object key regex")
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_attribute_regex() {
        let re = &CLASS_ATTRIBUTE_REGEX;
        assert!(re.is_match(r#"class="foo bar""#));
        assert!(re.is_match(r#"className="foo bar""#));
        assert!(re.is_match(r#"classname="foo bar""#));
        assert!(re.is_match(r#":class="foo bar""#));
        assert!(re.is_match(r#":classname="foo bar""#));
        assert!(re.is_match(r#":className="foo bar""#));

        let haystack = r#"<div class="foo bar"></div>"#;
        let captures = re.captures(haystack).unwrap();
        assert_eq!(&captures["className"], "foo bar");
    }

    #[test]
    fn test_clsx_regex() {
        let re = &CLSX_REGEX;
        assert!(re.is_match(r#"clsx("foo bar")"#));
        assert!(re.is_match(r#"clsx({foo: true, bar: true})"#));
        assert!(re.is_match(r#"classnames("foo bar")"#));
        assert!(re.is_match(r#"cn("foo bar")"#));
        assert!(re.is_match(r#"cn(["foo", "bar"])"#));
        assert!(re.is_match(r#"cn(["foo", "bar"])"#));

        let haystack = r#"<div className={clsx("foo bar")}></div>"#;
        let captures = re.captures(haystack).unwrap();
        assert_eq!(&captures["clsx_content"], r#""foo bar""#);

        let haystack = r#"<div className={classnames(["foo", "bar"])}></div>"#;
        let captures = re.captures(haystack).unwrap();
        assert_eq!(&captures["clsx_content"], r#"["foo", "bar"]"#);

        let haystack = r#"<div className={cn(["foo", "bar"])}></div>"#;
        let captures = re.captures(haystack).unwrap();
        assert_eq!(&captures["clsx_content"], r#"["foo", "bar"]"#);

        let haystack = r#"<div className={clsx({foo: true, bar: true})}></div>"#;
        let captures = re.captures(haystack).unwrap();
        assert_eq!(&captures["clsx_content"], r#"{foo: true, bar: true}"#);
    }

    #[test]
    fn test_string_literal_regex() {
        let re = &STRING_LITERAL_REGEX;
        let haystack = r#"<div class="foo bar"></div>"#;
        let cap = re.captures(haystack).unwrap();
        assert_eq!(&cap["string_literal"], "foo bar");
    }

    #[test]
    fn test_object_key_regex() {
        let re = &OBJECT_KEY_REGEX;
        let haystack = r#"{foo: true, 'text-primary': true, "text-secondary": false }"#;

        let keys: Vec<_> = re
            .captures_iter(haystack)
            .map(|cap| cap.name("object_key").unwrap().as_str())
            .collect();

        assert_eq!(keys, vec!["foo", "text-primary", "text-secondary"]);
    }
}
