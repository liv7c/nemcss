use tower_lsp::lsp_types::Url;

/// Returns `true` when the URI refers to a dedicated CSS-family file.
pub fn is_dedicated_css_file(url: &Url) -> bool {
    url.to_file_path()
        .ok()
        .and_then(|p| p.extension().and_then(|e| e.to_str()).map(str::to_owned))
        .map(|ext| matches!(ext.as_str(), "css" | "scss" | "sass" | "lesss"))
        .unwrap_or(false)
}
