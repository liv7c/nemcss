use miette::Diagnostic;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashSet;

/// Renders a Rust diagnostic the way the CLI would (message, causes, help) so
/// the Vite overlay and PostCSS show the same hint.
fn to_js_error(err: impl Diagnostic) -> Error {
    let mut message = config::display_error_chain(&err);
    if let Some(help) = err.help() {
        message.push_str(&format!("\n\nhelp: {help}"));
    }

    Error::from_reason(message)
}

/// JsTokenUtility is the configuration of a utility class for a given token.
///
/// # Example
/// ```no_run
/// { prefix: "bg", property: "background-color" }
/// ```
#[napi(object)]
pub struct JsTokenUtility {
    pub prefix: String,
    pub property: String,
}

/// JsTokenEntry is the configuration of a token entry.
/// In Rust, we have an enum (Simple or List). We simplify it to an array of strings.
#[napi(object)]
pub struct JsTokenEntry {
    /// The name of the token entry.
    pub name: String,
    /// The value of the token entry.
    pub value: Vec<String>,
}

/// JsResolvedToken is the configuration of a resolved token.
/// It is used to generate both the CSS custom properties and the utilities for a given design token.
///
/// # Example
/// ```no_run
/// {
///     prefix: "bg",
///     tokens: [
///         { name: "color", value: ["red"] },
///         { name: "opacity", value: ["0.5"] }
///     ],
///     utilities: [
///         { prefix: "bg", property: "background-color" },
///         { prefix: "bg", property: "background-image" }
///     ]
/// }
/// ```
#[napi(object)]
pub struct JsResolvedToken {
    pub prefix: String,
    pub tokens: Vec<JsTokenEntry>,
    pub utilities: Vec<JsTokenUtility>,
}

/// Extracts the classes from a given content.
/// It is a wrapper around the `extractor` module.
/// Normally, `extractor::extract_classes` returns a HashSet.
/// For compatibility with the Node API, we convert it to a Vec.
#[napi]
pub fn extract_classes(content: String) -> Vec<String> {
    extractor::extract_classes(&content).into_iter().collect()
}

#[napi(object)]
pub struct GeneratedCss {
    pub base_css: String,
    pub utilities_css: String,
}

/// Generates the CSS from a given configuration file and outputs the CSS that will be used
/// to replace the `@nemcss base;` and `@nemcss utilities;` directives.
#[napi]
pub fn generate_css(
    config_path: String,
    used_classes: Option<Vec<String>>,
) -> Result<GeneratedCss> {
    let config = config::NemCssConfig::from_path(&config_path).map_err(to_js_error)?;

    let resolved = config.resolve_all_tokens().map_err(to_js_error)?;

    let used_set: Option<HashSet<String>> = used_classes.map(|v| v.into_iter().collect());
    let viewports = resolved
        .get("viewports")
        .or_else(|| resolved.get("viewport"));
    let semantic_tokens = config
        .resolve_semantic_groups(&resolved)
        .map_err(to_js_error)?;

    let resolved_modes = config
        .resolve_modes(&semantic_tokens, &resolved)
        .map_err(to_js_error)?;

    let generated = engine::generate_css(
        resolved.values(),
        semantic_tokens.values(),
        engine::GenerateCssOptions {
            modes: &resolved_modes,
            viewports,
            used_classes: used_set.as_ref(),
        },
    );

    Ok(GeneratedCss {
        base_css: generated.base_to_css(),
        utilities_css: generated.utilities_to_css(),
    })
}
