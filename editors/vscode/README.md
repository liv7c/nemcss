# NemCSS — VS Code Extension

VS Code extension for [NemCSS](https://github.com/liv7c/nemcss), a design-token-driven CSS utility generator.

Provides IDE support via the NemCSS Language Server for the utility classes and CSS custom properties generated from your `nemcss.config.json`.

## Features

- **Autocomplete** for NemCSS utility classes and custom properties
- **Hover** documentation showing the resolved value of a utility class or custom property
- Works across CSS, SCSS, Sass, Less, HTML, PHP, Blade, Twig, JavaScript, TypeScript, Vue, Svelte, and Astro files

## Requirements

The extension bundles the `nemcss-lsp` binary for the following platforms:

| Platform | Architecture |
|----------|-------------|
| macOS    | arm64, x64  |
| Linux    | x64         |
| Windows  | x64         |

No separate installation is required — the binary is included in the extension package.

## Extension Settings

| Setting | Type | Description |
|---------|------|-------------|
| `nemcss.lspPath` | `string` | Path to a custom `nemcss-lsp` binary. Useful if you want to use a locally compiled version. Defaults to the bundled binary. |

## Release Notes

### 0.0.1

Initial release.
