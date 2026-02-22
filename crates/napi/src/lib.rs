use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashSet;

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

/// Generates the CSS from a given configuration file and outputs the CSS that will be used
/// to replace the `@nemcss;` directives.
#[napi]
pub fn generate_css(config_path: String, used_classes: Option<Vec<String>>) -> Result<String> {
    let config = config::NemCssConfig::from_path(&config_path)
        .map_err(|e| Error::from_reason(format!("{e}")))?;

    let resolved = config
        .resolve_all_tokens()
        .map_err(|e| Error::from_reason(format!("{e}")))?;

    let used_set: Option<HashSet<String>> = used_classes.map(|v| v.into_iter().collect());
    let viewports = resolved
        .get("viewports")
        .or_else(|| resolved.get("viewport"));
    let generated = engine::generate_css(resolved.values(), viewports, used_set.as_ref());

    Ok(generated.to_css())
}
