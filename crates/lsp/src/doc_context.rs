/// The language in effect at a given position in a document.
/// For now, this is only used to determine whether we're in a CSS context or not to trigger
/// CSS completions at the right places.
#[derive(Debug, PartialEq, Clone)]
pub enum DocLang {
    /// Inside a CSS context: can be a CSS file, or a style tag in an HTML/Astro/Svelte file.
    Css,
    /// Anything else: HTML, script, template
    Other,
}

/// A language boundary in a document.
/// Boundaries are stored in ascending order. To find the language at a given position, we can do a binary search to find the last boundary before that position.
#[derive(Debug, PartialEq, Clone)]
pub struct DocLangBoundary {
    /// Byte offset in the dcoument where this language starts.
    /// We'll use Rope's byte offsets for this, so that we can easily compare with the offsets in the syntax tree.
    pub offset: usize,
    /// The language in effect starting from this position.
    pub lang: DocLang,
}

const OPENING_STYLE_TAG: &str = "<style";
const CLOSING_STYLE_TAG: &str = "</style>";

/// Parses the document and returns a list of language boundaries in ascending offset order.
pub fn get_doc_language_boundaries(text: &str) -> Vec<DocLangBoundary> {
    let mut boundaries = vec![DocLangBoundary {
        offset: 0,
        lang: DocLang::Other,
    }];

    let mut search_start = 0;

    // Goal of the loop is to find all <style> tags in the document, and add boundaries for the start and end of each style block.
    while let Some(remaining) = text.get(search_start..) {
        let Some(rel_open) = remaining.find(OPENING_STYLE_TAG) else {
            break;
        };

        let open_pos = search_start + rel_open;
        let after_keyword = open_pos + OPENING_STYLE_TAG.len();

        let is_valid_tag = text
            .get(after_keyword..)
            .map(|s| s.starts_with(|c: char| c.is_whitespace() || c == '>'))
            .unwrap_or(false);

        if !is_valid_tag {
            // skip past position and keep searching
            search_start = open_pos + 1;
            continue;
        }

        let Some(rel_bracket) = text.get(after_keyword..).and_then(|s| s.find('>')) else {
            // no closing bracket, invalid tag, stop processing
            break;
        };

        let css_start = after_keyword + rel_bracket + 1;
        boundaries.push(DocLangBoundary {
            offset: css_start,
            lang: DocLang::Css,
        });

        let Some(remaining_css) = text.get(css_start..) else {
            // no content after the opening tag, stop processing
            break;
        };

        let Some(rel_end) = remaining_css.find(CLOSING_STYLE_TAG) else {
            // no closing tag, stop processing
            break;
        };

        let css_end = css_start + rel_end;
        boundaries.push(DocLangBoundary {
            offset: css_end,
            lang: DocLang::Other,
        });

        search_start = css_end + CLOSING_STYLE_TAG.len();
    }

    boundaries
}

pub fn get_doc_language_at_offset(boundaries: &[DocLangBoundary], offset: usize) -> DocLang {
    let idx = boundaries.partition_point(|b| b.offset <= offset);
    idx.checked_sub(1)
        .map(|i| boundaries[i].lang.clone())
        .unwrap_or(DocLang::Other)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text_is_entirely_other() {
        let boundaries = get_doc_language_boundaries("hello style");
        assert_eq!(get_doc_language_at_offset(&boundaries, 0), DocLang::Other);
        assert_eq!(get_doc_language_at_offset(&boundaries, 11), DocLang::Other);
    }

    #[test]
    fn test_style_block_creates_css_region() {
        let html = "<html><style>body { color: red; }</style></html>";
        let boundaries = get_doc_language_boundaries(html);
        assert_eq!(get_doc_language_at_offset(&boundaries, 0), DocLang::Other);
        assert_eq!(get_doc_language_at_offset(&boundaries, 1), DocLang::Other);
        assert_eq!(get_doc_language_at_offset(&boundaries, 13), DocLang::Css);
        assert_eq!(get_doc_language_at_offset(&boundaries, 25), DocLang::Css);
        assert_eq!(get_doc_language_at_offset(&boundaries, 33), DocLang::Other);
    }

    #[test]
    fn test_style_block_with_lang_attribute() {
        let html = "<style lang=\"scss\">$color: red; body { color: $color; }</style>";
        let boundaries = get_doc_language_boundaries(html);

        let css_start = html
            .find("$color")
            .expect("expected to find $color in the string");
        assert_eq!(
            get_doc_language_at_offset(&boundaries, css_start),
            DocLang::Css
        );
    }

    #[test]
    fn test_multiple_style_blocks() {
        let html = "<html><style>#article { color: red; }</style><p>Between style blocks</p><style lang=\"scss\">$color: blue;</style></html>";
        let boundaries = get_doc_language_boundaries(html);

        let first_css = html
            .find("#article")
            .expect("expected to find #article in the string");
        let between = html
            .find("<p>")
            .expect("expected to find '<p>' in the string");
        let second_css = html
            .find("$color")
            .expect("expected to find $color in the string");

        assert_eq!(
            get_doc_language_at_offset(&boundaries, first_css),
            DocLang::Css
        );
        assert_eq!(
            get_doc_language_at_offset(&boundaries, between),
            DocLang::Other
        );
        assert_eq!(
            get_doc_language_at_offset(&boundaries, second_css),
            DocLang::Css
        );

        assert_eq!(get_doc_language_at_offset(&boundaries, 13), DocLang::Css);
        assert_eq!(get_doc_language_at_offset(&boundaries, 30), DocLang::Css);
    }

    #[test]
    fn test_get_doc_language_at_offset_returns_other_when_no_boundaries() {
        let boundaries = vec![];
        assert_eq!(get_doc_language_at_offset(&boundaries, 0), DocLang::Other);
        assert_eq!(get_doc_language_at_offset(&boundaries, 100), DocLang::Other);
    }

    #[test]
    fn test_get_language_at_position_uses_last_boundary_before_or_at_offset() {
        let boundaries = vec![
            DocLangBoundary {
                offset: 0,
                lang: DocLang::Other,
            },
            DocLangBoundary {
                offset: 10,
                lang: DocLang::Css,
            },
            DocLangBoundary {
                offset: 20,
                lang: DocLang::Other,
            },
        ];
        assert_eq!(get_doc_language_at_offset(&boundaries, 5), DocLang::Other);
        assert_eq!(get_doc_language_at_offset(&boundaries, 7), DocLang::Other);
        assert_eq!(get_doc_language_at_offset(&boundaries, 10), DocLang::Css);
        assert_eq!(get_doc_language_at_offset(&boundaries, 15), DocLang::Css);
        assert_eq!(get_doc_language_at_offset(&boundaries, 20), DocLang::Other);
        assert_eq!(get_doc_language_at_offset(&boundaries, 25), DocLang::Other);
        assert_eq!(get_doc_language_at_offset(&boundaries, 99), DocLang::Other);
    }
}
