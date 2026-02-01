//! # Extractor module
//!
//! This module contains the logic for extracting CSS classes from a given file. It heavily relies
//! on the a series of regular expressions to identify CSS classes and their usage.

use std::{collections::HashSet, sync::LazyLock};

use regex::Regex;

/// A lazy-loaded regex for matching CSS classes using the `class` attribute (or classname /
/// className).
/// Matches: class="foo bar", className="foo bar", classname="foo bar"
static CLASS_ATTRIBUTE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"class(name|Name)?=["'](?<className>[^"']+)["']"#)
        .expect("Failed to compile class attribute regex")
});

/// A regex for matching CSS classes using utilities like classnames(...), clsx(...), :class(...) or cn(...)
/// It uses a greedy match to capture everything that is in between the parentheses (as we could
/// have arrays, objects, etc.).
static CLASS_UTILITY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(clsx|classnames|cn)\((?<content>[\s\S]*?)\)"#)
        .expect("Failed to compile clsx regex")
});

/// A regex for matching JSX/Svelte class expressions: class={...}, or className={...}
static JSX_CLASS_EXPRESSION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"class(name|Name)?=\{(?<content>[\s\S]*?)\}"#)
        .expect("Failed to compile jsx class expression regex")
});

/// A regex to support Vue class binding syntax
static VUE_CLASS_BINDING_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#":class=(?:"(?<double>[\s\S]*?)"|'(?<single>[\s\S]*?)')"#)
        .expect("Failed to compile vue class binding regex")
});

/// A regex to support Astro class:list syntax
static ASTRO_CLASS_LIST_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"class:list=(?:["'](?<quoted>[^"']+)["']|\{(?<braced>[\s\S]+?)\})"#)
        .expect("Failed to compile astro class:list regex")
});

/// A regex to support Svelte class binding syntax
static SVELTE_CLASS_BINDING_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"class:(?<className>[\w-]+)=\{[^}]*\}"#)
        .expect("Failed to compile svelte class binding regex")
});

/// A regex for matching string literals
static STRING_LITERAL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"["'](?<string_literal>[^"']+)["']"#)
        .expect("Failed to compile string literal regex")
});

/// A regex for matching object keys
static OBJECT_KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\{?\s*["']?(?<object_key>[\w-]+)["']?:"#)
        .expect("Failed to compile object key regex")
});

/// Extract CSS classes from a given content string.
///
/// This function supports a variety of different syntaxes for declaring CSS classes, including:
/// - Regular class attributes (class="...")
/// - Utilities like classnames(...), clsx(...), :class(...) or cn(...) that wrap classes in
///   arrays, objects, etc.
/// - Class expressions in JSX/Svelte like class={...} or className={...}
/// - Conditional class directives like class:list="..." used in Astro
/// - Svelte class directives like class:active={isActive}
/// - Ternary expressions like {isActive ? 'active-state' : 'inactive-state'}
///
/// # Example
///
/// ```no_run
/// use std::collections::HashSet;
/// use scanner::extract_classes;
///
/// let content = r#"
///     <main class="container">
///         <h1 class="text-primary font-bold">Hello, world!</h1>
///         <div class="text-primary font-bold">
///             <span class="bg-secondary"></span>
///         </div>
///         <div class="text-primary"></div>
///     </main>
/// "#;
///
/// let classes = extract_classes(content);
/// assert!(classes.contains("container"));
/// assert!(classes.contains("text-primary"));
/// assert!(classes.contains("font-bold"));
/// assert!(classes.contains("bg-secondary"));
/// ```
pub fn extract_classes(content: &str) -> HashSet<String> {
    let mut classes = HashSet::new();

    // Look for class attributes
    for cap in CLASS_ATTRIBUTE_REGEX.captures_iter(content) {
        if let Some(class_name) = cap.name("className") {
            for class in class_name.as_str().split_whitespace() {
                classes.insert(class.to_string());
            }
        }
    }

    // Look for clsx(...) or classnames(...) calls
    for cap in CLASS_UTILITY_REGEX.captures_iter(content) {
        if let Some(css_content) = cap.name("content") {
            extract_classes_from_syntax(css_content.as_str(), &mut classes);
        }
    }

    // Look for :class="..." calls
    for cap in VUE_CLASS_BINDING_REGEX.captures_iter(content) {
        let css_content = cap.name("double").or_else(|| cap.name("single"));
        if let Some(content) = css_content {
            extract_classes_from_syntax(content.as_str(), &mut classes);
        }
    }

    // Look for class={...} or className={...} calls
    for cap in JSX_CLASS_EXPRESSION_REGEX.captures_iter(content) {
        let css_content = cap.name("content");
        if let Some(content) = css_content {
            extract_classes_from_syntax(content.as_str(), &mut classes);
        }
    }

    // Look for class:list="..." calls
    for cap in ASTRO_CLASS_LIST_REGEX.captures_iter(content) {
        if let Some(quoted) = cap.name("quoted") {
            for class in quoted.as_str().split_whitespace() {
                classes.insert(class.to_string());
            }
        }

        if let Some(braced) = cap.name("braced") {
            extract_classes_from_syntax(braced.as_str(), &mut classes);
        }
    }

    for cap in SVELTE_CLASS_BINDING_REGEX.captures_iter(content) {
        if let Some(class_name) = cap.name("className") {
            classes.insert(class_name.as_str().to_string());
        }
    }

    classes
}

/// Extract classes from a given syntax, such as a string literal, object key, or array.
fn extract_classes_from_syntax(content: &str, classes: &mut HashSet<String>) {
    for string_cap in STRING_LITERAL_REGEX.captures_iter(content) {
        if let Some(string_literal) = string_cap.name("string_literal") {
            for class in string_literal.as_str().split_whitespace() {
                classes.insert(class.to_string());
            }
        }
    }

    for object_key_cap in OBJECT_KEY_REGEX.captures_iter(content) {
        if let Some(object_key) = object_key_cap.name("object_key") {
            for class in object_key.as_str().split_whitespace() {
                classes.insert(class.to_string());
            }
        }
    }
}

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
    fn test_class_utility_regex() {
        let re = &CLASS_UTILITY_REGEX;
        assert!(re.is_match(r#"clsx("foo bar")"#));
        assert!(re.is_match(r#"clsx({foo: true, bar: true})"#));
        assert!(re.is_match(r#"classnames("foo bar")"#));
        assert!(re.is_match(r#"cn("foo bar")"#));
        assert!(re.is_match(r#"cn(["foo", "bar"])"#));
        assert!(re.is_match(r#"cn(["foo", "bar"])"#));

        let haystack = r#"<div className={clsx("foo bar")}></div>"#;
        let captures = re.captures(haystack).unwrap();
        assert_eq!(&captures["content"], r#""foo bar""#, "Got {captures:?}");

        let haystack = r#"<div className={classnames(["foo", "bar"])}></div>"#;
        let captures = re.captures(haystack).unwrap();
        assert_eq!(
            &captures["content"], r#"["foo", "bar"]"#,
            "Got {captures:?}"
        );

        let haystack = r#"<div className={cn(["foo", "bar"])}></div>"#;
        let captures = re.captures(haystack).unwrap();
        assert_eq!(
            &captures["content"], r#"["foo", "bar"]"#,
            "Got {captures:?}"
        );

        let haystack = r#"<div className={clsx({foo: true, bar: true})}></div>"#;
        let captures = re.captures(haystack).unwrap();
        assert_eq!(
            &captures["content"], r#"{foo: true, bar: true}"#,
            "Got {captures:?}"
        );
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

    #[test]
    fn test_object_key_regex_with_underscore_and_dash() {
        let re = &OBJECT_KEY_REGEX;
        let haystack = r#"{foo: true, 'text-primary': true, "card-box__text": false }"#;

        let keys: Vec<_> = re
            .captures_iter(haystack)
            .map(|cap| cap.name("object_key").unwrap().as_str())
            .collect();

        assert_eq!(keys, vec!["foo", "text-primary", "card-box__text"]);
    }

    #[test]
    fn test_extract_classes_with_html_doc() {
        let raw_html = r#"
            <main class="container">
                <h1>Hello, world!</h1>
                <div class="text-primary font-bold">
                    <span class="bg-secondary"></span>
                </div>
                <div class="text-primary"></div>
            </main>
        "#;

        let result = extract_classes(raw_html);
        assert!(
            result.contains("container"),
            "Expected to find 'container' class, got {result:?}"
        );
        assert!(
            result.contains("text-primary"),
            "Expected to find 'text-primary' class, got {result:?}"
        );
        assert!(
            result.contains("font-bold"),
            "Expected to find 'font-bold' class, got {result:?}"
        );
        assert!(
            result.contains("bg-secondary"),
            "Expected to find 'bg-secondary' class, got {result:?}"
        );
    }

    #[test]
    fn test_extract_classes_classname_jsx() {
        let raw_html = r#"
            <main className="container">
                <h1>Hello, world!</h1>
                <div className="text-primary font-bold">
                    <span className="bg-secondary"></span>
                </div>
                <div className="text-primary"></div>
            </main>
        "#;

        let result = extract_classes(raw_html);
        assert!(
            result.contains("container"),
            "Expected to find 'container' class, got {result:?}"
        );
        assert!(
            result.contains("text-primary"),
            "Expected to find 'text-primary' class, got {result:?}"
        );
        assert!(
            result.contains("font-bold"),
            "Expected to find 'font-bold' class, got {result:?}"
        );
        assert!(
            result.contains("bg-secondary"),
            "Expected to find 'bg-secondary' class, got {result:?}"
        );
    }

    #[test]
    fn test_extract_classes_clsx_jsx() {
        let raw_html = r#"
            <main className={clsx("container", { "text-primary": true })}>
                <h1>Hello, world!</h1>
                <div className={clsx({"text-primary": true, "bg-primary": false})}>
                    <span className={clsx(['font-mono', [['foo', 'bar']]])}></span>
                </div>
                <div className="text-neutral"></div>
            </main>
        "#;

        let result = extract_classes(raw_html);
        assert!(
            result.contains("container"),
            "Expected to find 'container' class, got {result:?}"
        );
        assert!(
            result.contains("text-primary"),
            "Expected to find 'text-primary' class, got {result:?}"
        );
        assert!(
            result.contains("bg-primary"),
            "Expected to find 'bg-primary' class, got {result:?}"
        );
        assert!(
            result.contains("font-mono"),
            "Expected to find 'font-mono' class, got {result:?}"
        );
        assert!(
            result.contains("foo"),
            "Expected to find 'foo' class, got {result:?}"
        );
        assert!(
            result.contains("bar"),
            "Expected to find 'bar' class, got {result:?}"
        );
        assert!(
            result.contains("text-neutral"),
            "Expected to find 'text-neutral' class, got {result:?}"
        );
    }

    #[test]
    fn test_extract_classes_vue_class_binding() {
        let raw_html = r#"
            <main :class="['container', { 'text-primary': true }]">
                <h1>Hello, world!</h1>
                <div :class="['text-primary', 'font-bold']">
                    <span :class="['bg-secondary']"></span>
                </div>
                <div :class="['text-primary']"></div>
                <div class="class-without-binding"></div>
            </main>
        "#;

        let result = extract_classes(raw_html);
        assert!(
            result.contains("container"),
            "Expected to find 'container' class, got {result:?}"
        );
        assert!(
            result.contains("text-primary"),
            "Expected to find 'text-primary' class, got {result:?}"
        );
        assert!(
            result.contains("font-bold"),
            "Expected to find 'font-bold' class, got {result:?}"
        );
        assert!(
            result.contains("bg-secondary"),
            "Expected to find 'bg-secondary' class, got {result:?}"
        );
        assert!(
            result.contains("class-without-binding"),
            "Expected to find 'normal-class' class, got {result:?}"
        );
    }

    #[test]
    fn test_extract_classes_astro_class_list() {
        let raw_html = r#"
            <main class:list="container text-primary">
                <h1 class="normal-class">Hello, world!</h1>
                <div class:list="text-primary font-bold">
                    <span class:list="bg-secondary"></span>
                </div>
                <div class:list="text-primary"></div>
                <div class:list={['box', { red: isRed }]}><slot /></div>
            </main>
        "#;

        let result = extract_classes(raw_html);
        assert!(
            result.contains("normal-class"),
            "Expected to find 'normal-class' class, got {result:?}"
        );
        assert!(
            result.contains("container"),
            "Expected to find 'container' class, got {result:?}"
        );
        assert!(
            result.contains("text-primary"),
            "Expected to find 'text-primary' class, got {result:?}"
        );
        assert!(
            result.contains("font-bold"),
            "Expected to find 'font-bold' class, got {result:?}"
        );
        assert!(
            result.contains("bg-secondary"),
            "Expected to find 'bg-secondary' class, got {result:?}"
        );

        assert!(
            result.contains("box"),
            "Expected to find 'box' class, got {result:?}"
        );
        assert!(
            result.contains("red"),
            "Expected to find 'red' class, got {result:?}"
        );
    }

    #[test]
    fn test_extract_classes_supports_multiple_classes() {
        let raw_html = r#"
            <main class="container text-primary">
                <h1 className={clsx({
                    'text-primary': true,
                    'font-bold': true,
                    'bg-secondary': false,
                })}>Hello</h1>
                <div className={cn([
                    "text-secondary",
                    'bg-red'
                ])}>Test</div>
            </main>
        "#;

        let result = extract_classes(raw_html);
        assert!(
            result.contains("container"),
            "Expected to find 'container' class, got {result:?}"
        );
        assert!(
            result.contains("text-primary"),
            "Expected to find 'text-primary' class, got {result:?}"
        );
        assert!(
            result.contains("font-bold"),
            "Expected to find 'font-bold' class, got {result:?}"
        );
        assert!(
            result.contains("bg-secondary"),
            "Expected to find 'bg-secondary' class, got {result:?}"
        );
        assert!(
            result.contains("text-secondary"),
            "Expected to find 'text-secondary' class, got {result:?}"
        );
        assert!(
            result.contains("bg-red"),
            "Expected to find 'bg-red' class, got {result:?}"
        );
    }

    #[test]
    fn test_extract_classes_supports_astro_class_list_multi_line() {
        let raw_html = r#"
            <main class:list={[
                'container',
                'text-primary',
                'font-bold',
                'bg-secondary',
            ]}>
                <h1 class="normal-class">Hello, world!</h1>
                <div class:list={['text-primary', 'font-bold']}>
                    <span class:list={['bg-secondary']}></span>
                </div>
                <div class:list={['text-primary']}></div>
                <div class:list={['box', { 
                    red: isRed 
                }]}><slot /></div>
            </main>
        "#;

        let result = extract_classes(raw_html);
        assert!(
            result.contains("normal-class"),
            "Expected to find 'normal-class' class, got {result:?}"
        );
        assert!(
            result.contains("container"),
            "Expected to find 'container' class, got {result:?}"
        );
        assert!(
            result.contains("text-primary"),
            "Expected to find 'text-primary' class, got {result:?}"
        );
        assert!(
            result.contains("font-bold"),
            "Expected to find 'font-bold' class, got {result:?}"
        );
        assert!(
            result.contains("bg-secondary"),
            "Expected to find 'bg-secondary' class, got {result:?}"
        );

        assert!(
            result.contains("box"),
            "Expected to find 'box' class, got {result:?}"
        );
        assert!(
            result.contains("red"),
            "Expected to find 'red' class, got {result:?}"
        );
    }

    #[test]
    fn test_extract_classes_supports_vue_class_binding_multi_line() {
        let raw_html = r#"
            <main :class="[
                'container',
                'text-primary',
                'font-bold',
                'bg-secondary',
            ]">
                <h1 class="normal-class">Hello, world!</h1>
                <div :class="['text-primary', 'font-bold']">
                    <span :class="['bg-secondary']"></span>
                </div>
                <div :class="['text-primary']"></div>
                <div :class="['box', {
                    red: isRed
                }]"><slot /></div>
            </main>
        "#;

        let result = extract_classes(raw_html);
        assert!(
            result.contains("normal-class"),
            "Expected to find 'normal-class' class, got {result:?}"
        );
        assert!(
            result.contains("container"),
            "Expected to find 'container' class, got {result:?}"
        );
        assert!(
            result.contains("text-primary"),
            "Expected to find 'text-primary' class, got {result:?}"
        );
        assert!(
            result.contains("font-bold"),
            "Expected to find 'font-bold' class, got {result:?}"
        );
        assert!(
            result.contains("bg-secondary"),
            "Expected to find 'bg-secondary' class, got {result:?}"
        );

        assert!(
            result.contains("box"),
            "Expected to find 'box' class, got {result:?}"
        );
        assert!(
            result.contains("red"),
            "Expected to find 'red' class, got {result:?}"
        );
    }

    #[test]
    fn test_extract_classes_supports_svelte_class_binding() {
        let raw_html = r#"
          <main class="container">
              <!-- Regular class attribute -->
              <h1 class="text-primary font-bold">Hello, world!</h1>

              <!-- Svelte class: directive (conditional classes) -->
              <div
                  class:active={isActive}
                  class:disabled={!isEnabled}
                  class="base-class">
                  Conditional
              </div>

              <!-- Multi-line class with expression -->
              <section
                  class="
                      bg-secondary
                      text-white
                      p-4
                  ">
                  Multi-line classes
              </section>

              <!-- Mixed: regular + conditional -->
              <button
                  class="btn"
                  class:btn-primary={variant === 'primary'}
                  class:btn-secondary={variant === 'secondary'}>
                  Button
              </button>

              <!-- Ternary expression -->
              <div class={isActive ? 'active-state' : 'inactive-state'}>
                  Dynamic
              </div>
          </main>
      "#;

        let result = extract_classes(raw_html);
        assert!(
            result.contains("container"),
            "Expected to find 'container' class, got {result:?}"
        );
        assert!(
            result.contains("text-primary"),
            "Expected to find 'text-primary' class, got {result:?}"
        );
        assert!(
            result.contains("font-bold"),
            "Expected to find 'font-bold' class, got {result:?}"
        );
        assert!(
            result.contains("base-class"),
            "Expected to find 'base-class' class, got {result:?}"
        );
        assert!(
            result.contains("active"),
            "Expected to find 'active' class, got {result:?}"
        );
        assert!(
            result.contains("disabled"),
            "Expected to find 'disabled' class, got {result:?}"
        );
        assert!(
            result.contains("bg-secondary"),
            "Expected to find 'bg-secondary' class, got {result:?}"
        );
        assert!(
            result.contains("text-white"),
            "Expected to find 'text-white' class, got {result:?}"
        );
        assert!(
            result.contains("p-4"),
            "Expected to find 'p-4' class, got {result:?}"
        );
        assert!(
            result.contains("btn"),
            "Expected to find 'btn' class, got {result:?}"
        );
        assert!(
            result.contains("btn-primary"),
            "Expected to find 'btn-primary' class, got {result:?}"
        );
        assert!(
            result.contains("btn-secondary"),
            "Expected to find 'btn-secondary' class, got {result:?}"
        );
        assert!(
            result.contains("active-state"),
            "Expected to find 'active-state' class, got {result:?}"
        );
        assert!(
            result.contains("inactive-state"),
            "Expected to find 'inactive-state' class, got {result:?}"
        );
    }

    #[test]
    fn test_extract_jsx_class_expression() {
        let raw_html = r#"
            <main className="container">
                <div className={isActive ? 'block' : 'hidden'}">
                    <span className="bg-secondary"></span>
                </div>
                <div className="text-primary"></div>
            </main>
        "#;

        let result = extract_classes(raw_html);
        assert!(
            result.contains("container"),
            "Expected to find 'container' class, got {result:?}"
        );
        assert!(
            result.contains("block"),
            "Expected to find 'block' class, got {result:?}"
        );
        assert!(
            result.contains("hidden"),
            "Expected to find 'hidden' class, got {result:?}"
        );
        assert!(
            result.contains("bg-secondary"),
            "Expected to find 'bg-secondary' class, got {result:?}"
        );
        assert!(
            result.contains("text-primary"),
            "Expected to find 'text-primary' class, got {result:?}"
        );
    }
}
