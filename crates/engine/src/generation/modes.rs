use config::{ModeActivation, ResolvedMode};
use std::fmt::Write;

/// Generates one CSS block per resolved mode.
///
/// A mode activated by a selector, here `[data-mode="dark"]`, generates:
///
/// ```css
/// [data-mode="dark"] {
///   --text-default: var(--color-gray-100);
/// }
/// ```
///
/// A mode activated by a media query generates:
///
/// ```css
/// @media (prefers-color-scheme: dark) {
///   :root {
///     --text-default: var(--color-gray-100);
///   }
/// }
/// ```
pub fn generate_mode_blocks(modes: &[ResolvedMode]) -> Vec<String> {
    let mut blocks = Vec::new();

    for mode in modes {
        if mode.declarations.is_empty() {
            continue;
        }

        let mut block = String::new();

        match &mode.activation {
            ModeActivation::Selector(selector) => {
                let _ = writeln!(block, "{selector} {{");

                for (property, value) in &mode.declarations {
                    let _ = writeln!(block, "  {property}: {value};");
                }

                let _ = write!(block, "}}");
            }
            ModeActivation::Media(query) => {
                let _ = writeln!(block, "@media {query} {{");
                let _ = writeln!(block, "  :root {{");

                for (property, value) in &mode.declarations {
                    let _ = writeln!(block, "    {property}: {value};");
                }

                let _ = writeln!(block, "  }}");
                let _ = write!(block, "}}");
            }
        }

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
            activation: ModeActivation::Selector("[data-mode=\"dark\"]".to_string()),
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
    fn media_mode_generates_only_an_unguarded_media_block() {
        let mut mode = dark();
        mode.activation = ModeActivation::Media("(prefers-color-scheme: dark)".to_string());

        let blocks = generate_mode_blocks(&[mode]);

        assert_eq!(blocks.len(), 1);

        assert_eq!(
            blocks[0],
            "@media (prefers-color-scheme: dark) {\n  :root {\n    --text-default: var(--color-gray-100);\n    --text-muted: var(--color-gray-400);\n  }\n}"
        );
    }

    #[test]
    fn mode_without_declarations_generates_nothing() {
        let mut mode = dark();
        mode.declarations.clear();
        mode.activation = ModeActivation::Media("(prefers-color-scheme: dark)".to_string());

        let blocks = generate_mode_blocks(&[mode]);
        assert!(blocks.is_empty());
    }

    #[test]
    fn supports_multiple_modes() {
        let mut contrast = dark();
        contrast.activation = ModeActivation::Selector("[data-mode=\"contrast\"]".to_string());

        let blocks = generate_mode_blocks(&[contrast, dark()]);

        assert_eq!(blocks.len(), 2);
        assert!(blocks[0].starts_with("[data-mode=\"contrast\"]"));
        assert!(blocks[1].starts_with("[data-mode=\"dark\"]"));
    }
}
