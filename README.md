# NemCSS

> A design-token-driven CSS utility generator

<a href="https://github.com/liv7c/nemcss/actions"><img src="https://img.shields.io/github/actions/workflow/status/liv7c/nemcss/ci.yml?branch=main" alt="CI Status"></a>
<a href="https://github.com/liv7c/nemcss/blob/main/LICENSE"><img src="https://img.shields.io/github/license/liv7c/nemcss" alt="License"></a>
<a href="https://www.npmjs.com/package/nemcss"><img src="https://img.shields.io/npm/v/nemcss" alt="Latest Release"></a>

NemCSS connects your design token JSON files to your CSS. Define your tokens as simple name/value pairs, then configure in `nemcss.config.json` the custom property prefix and the utility classes to derive from each token. Only the utilities your project actually uses end up in the final CSS. NemCSS also ships an LSP to smooth out your developer experience: you get autocomplete and hover documentation for all generated utility classes and custom properties directly in your editor.

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
