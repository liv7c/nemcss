# NemCSS

> A design-token-driven CSS utility and custom properties generator

<a href="https://github.com/liv7c/nemcss/actions"><img src="https://img.shields.io/github/actions/workflow/status/liv7c/nemcss/ci.yml?branch=main" alt="CI Status"></a>
<a href="https://github.com/liv7c/nemcss/blob/main/LICENSE"><img src="https://img.shields.io/github/license/liv7c/nemcss" alt="License"></a>
<a href="https://www.npmjs.com/package/nemcss"><img src="https://img.shields.io/npm/v/nemcss" alt="Latest Release"></a>

NemCSS is a small tool that takes your design tokens and generates CSS custom properties and utility classes from them, using your own naming conventions.

Define your tokens in JSON files. Declare your naming conventions and which utilities you wish to generate in one config file. Add `@nemcss base;` to your CSS, and your custom properties and utilities are available everywhere.

On top of your primitive tokens, a semantic layer lets you scope tokens to specific roles in your UI. Define which colors are for text, which are for backgrounds, directly from your config. You have a single source of truth, with explicit control over what gets generated and where.

NemCSS ships with an LSP for autocomplete and hover docs in your editor. It also has plugins for PostCSS and Vite.

## Documentation

Check out the full documentation with examples and guides at **[liv7c.github.io/nemcss](https://liv7c.github.io/nemcss/)**.

## Packages

| Package | Description |
| --- | --- |
| [`nemcss`](packages/nemcss) | CLI with standalone `build`, `watch`, and `init` commands |
| [`@nemcss/vite`](packages/vite-plugin-nemcss) | Vite plugin with HMR support |
| [`@nemcss/postcss`](packages/postcss-plugin-nemcss) | PostCSS plugin |
| [VS Code extension](editors/vscode) | Autocomplete and hover docs via LSP |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
