use config::ResolvedMode;
use std::fmt::Write;

/// Applied to media-query blocks so an explicit `data-mode` attribute always beats the system preference.
/// It is kind of an edge case. Ideally, users would choose either the selector or the media as a way to switch between themes.
const MEDIA_FALLBACK_GUARD: &str = ":root:not([data-mode])";

/// Generates CSS blocks for all resolved modes.
///
/// For a mode "dark" with declaration ("--text-default", "var(--color-gray-100"), this generates:
///
/// ```css
/// [data-mode="dark"] {
/// --text-default: var(--color-gray-100);
/// }
/// ```
///
/// and when the mode declares a media query, addiotionally:
///
/// ```css
/// @media (prefers-color-scheme: dark) {
///  :root:not([data-mode]) {
///    --text-default: var(--color-gray-100);
///   }
/// }
/// ```
pub fn generate_mode_blocks(modes: &[ResolvedMode]) -> Vec<String> {
    let mut blocks = Vec::new();

    for mode in modes {
        if mode.declarations.is_empty() {
            continue;
        }

        if let Some(media) = &mode.media {
            let mut block = String::new();
            let _ = writeln!(block, "@media {media} {{");
            let _ = writeln!(block, "  {MEDIA_FALLBACK_GUARD} {{");

            for (property, value) in &mode.declarations {
                let _ = writeln!(block, "    {property}: {value};");
            }

            let _ = writeln!(block, "  }}");
            let _ = write!(block, "}}");
            blocks.push(block);
        }

        let mut block = String::new();
        let _ = writeln!(block, "{} {{", mode.selector);
        for (property, value) in &mode.declarations {
            let _ = writeln!(block, "  {property}: {value};");
        }

        let _ = write!(block, "}}");
        blocks.push(block);
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::ResolvedMode;

    fn dark() -> ResolvedMode {
        ResolvedMode {
            name: "dark".to_string(),
            selector: "[data-mode=\"dark\"]".to_string(),
            media: None,
            declarations: vec![
                (
                    "--text-default".to_string(),
                    "var(--color-gray-100)".to_string(),
                ),
                (
                    "--text-muted".to_string(),
                    "var(--color-gray-400)".to_string(),
                ),
            ],
        }
    }

    #[test]
    fn selector_only_mode_generates_one_block() {
        let blocks = generate_mode_blocks(&[dark()]);

        assert_eq!(blocks.len(), 1);

        assert_eq!(
            blocks[0],
            "[data-mode=\"dark\"] {\n  --text-default: var(--color-gray-100);\n  --text-muted: var(--color-gray-400);\n}"
        );
    }

    #[test]
    fn media_mode_generates_guarded_media_block_before_selector_block() {
        let mut mode = dark();
        mode.media = Some("(prefers-color-scheme: dark)".to_string());

        let blocks = generate_mode_blocks(&[mode]);

        assert_eq!(blocks.len(), 2);

        assert_eq!(
            blocks[0],
            "@media (prefers-color-scheme: dark) {\n  :root:not([data-mode]) {\n    --text-default: var(--color-gray-100);\n    --text-muted: var(--color-gray-400);\n  }\n}"
        );
        assert!(blocks[1].starts_with("[data-mode=\"dark\"]"));
    }

    #[test]
    fn mode_without_declarations_generates_nothing() {
        let mut mode = dark();
        mode.declarations.clear();
        mode.media = Some("(prefers-color-scheme: dark)".to_string());

        let blocks = generate_mode_blocks(&[mode]);
        assert!(blocks.is_empty());
    }

    #[test]
    fn supports_multiple_modes() {
        let mut contrast = dark();
        contrast.selector = "[data-mode=\"contrast\"]".to_string();

        let blocks = generate_mode_blocks(&[contrast, dark()]);

        assert_eq!(blocks.len(), 2);
        assert!(blocks[0].starts_with("[data-mode=\"contrast\"]"));
        assert!(blocks[1].starts_with("[data-mode=\"dark\"]"));
    }
}
