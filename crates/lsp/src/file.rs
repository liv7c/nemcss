use tower_lsp::lsp_types::Url;

/// File extensions that always get custom property completions
/// regardless of the content globs in the config
pub const CSS_EXTENSIONS: &[&str] = &["css", "scss", "sass", "less"];

/// Returns `true` when the URI refers to a dedicated CSS-family file.
pub fn is_dedicated_css_file(url: &Url) -> bool {
    url.to_file_path()
        .ok()
        .and_then(|p| p.extension().and_then(|e| e.to_str()).map(str::to_owned))
        .map(|ext| CSS_EXTENSIONS.contains(&ext.as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dedicated_css_file_recognises_different_css_extensions() {
        let url = Url::parse("file:///project/styles/theme.less")
            .expect("failed to parse url for theme.less");
        assert!(is_dedicated_css_file(&url));
        let url = Url::parse("file:///project/styles/theme.css")
            .expect("failed to parse url for theme.css");
        assert!(is_dedicated_css_file(&url));
        let url = Url::parse("file:///project/styles/theme.scss")
            .expect("failed to parse url for theme.scss");
        assert!(is_dedicated_css_file(&url));
        let url = Url::parse("file:///project/styles/theme.sass")
            .expect("failed to parse url for theme.sass");
        assert!(is_dedicated_css_file(&url));
    }
}
