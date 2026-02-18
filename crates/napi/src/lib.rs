use napi_derive::napi;

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
