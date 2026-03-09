# NemCSS

NemCSS is a design-token-driven CSS utility and custom properties generator.

> **Pre-v1 notice**: NemCSS is under active development. The API and directives may change between minor versions until 1.0. Check the [CHANGELOG](./CHANGELOG.md) when upgrading.

<a href="https://github.com/liv7c/nemcss/actions"><img src="https://img.shields.io/github/actions/workflow/status/liv7c/nemcss/ci.yml?branch=main" alt="CI Status"></a>
<a href="https://github.com/liv7c/nemcss/blob/main/LICENSE"><img src="https://img.shields.io/github/license/liv7c/nemcss" alt="License"></a>
<a href="https://www.npmjs.com/package/nemcss"><img src="https://img.shields.io/npm/v/nemcss" alt="Latest Release"></a>

NemCSS is a small tool that takes your design tokens and generates CSS custom properties and utility classes from them, using your own naming conventions.

Here's an overview of how `nemcss` works:

1. You define your design tokens in JSON files (e.g. all your colors in a `design-tokens/colors.json`)
2. You configure your naming conventions and which utilities you wish to generate in a `nemcss.config.json` file.
   On top of your primitive tokens, a semantic layer lets you scope tokens to specific roles in your UI. Define which colors are for text, which are for backgrounds, directly from your config. You have a single source of truth, with explicit control over what gets generated and where.
3. You add a `@nemcss base;` to your CSS that gets replaced by all the generated custom properties at build time. You can also add a `@nemcss utilities;` to do the same for the generated utilities.
4. You can use the `nemcss` standalone CLI, or the Vite/PostCSS plugin to build the project. Check out the [currently supported integrations](https://liv7c.github.io/nemcss/integrations/cli.html).

NemCSS ships with an **LSP for autocomplete and hover docs** in your editor. It also has plugins for PostCSS and Vite.

## Documentation

Check out the full documentation with examples and guides at **[liv7c.github.io/nemcss](https://liv7c.github.io/nemcss/)**.

## Packages

| Package                                             | Description                                               |
| --------------------------------------------------- | --------------------------------------------------------- |
| [`nemcss`](packages/nemcss)                         | CLI with standalone `build`, `watch`, and `init` commands |
| [`@nemcss/vite`](packages/vite-plugin-nemcss)       | Vite plugin with HMR support                              |
| [`@nemcss/postcss`](packages/postcss-plugin-nemcss) | PostCSS plugin                                            |
| [VS Code extension](editors/vscode)                 | Autocomplete and hover docs via LSP                       |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
