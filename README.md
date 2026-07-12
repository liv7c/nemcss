# NemCSS

NemCSS is a design-token-driven CSS utility and custom properties generator.

> **Pre-v1 notice**: NemCSS is under active development. The API and directives may change between minor versions until 1.0. Check the [CHANGELOG](./CHANGELOG.md) when upgrading.

<a href="https://github.com/liv7c/nemcss/actions"><img src="https://img.shields.io/github/actions/workflow/status/liv7c/nemcss/ci.yml?branch=main" alt="CI Status"></a>
<a href="https://github.com/liv7c/nemcss/blob/main/LICENSE"><img src="https://img.shields.io/github/license/liv7c/nemcss" alt="License"></a>
<a href="https://www.npmjs.com/package/nemcss"><img src="https://img.shields.io/npm/v/nemcss" alt="Latest Release"></a>

NemCSS is built around two ideas:

1. **Design tokens are the foundation.** You define your tokens (colors, spacings, radii, anything) in small JSON files. They are the single source of truth for everything NemCSS generates.
2. **Everything else is derived declaratively in `nemcss.config.json`.** From your tokens, NemCSS generates CSS custom properties using your own naming conventions. In the same config, you can derive a semantic layer (which colors are for text, which are for backgrounds) and the utility classes you want. Nothing gets generated unless you ask for it.

## Quick start

Install the CLI and scaffold a project:

```sh
npm install -D nemcss
npx nemcss init
```

This creates a `nemcss.config.json` and a `design-tokens` folder with example color and spacing tokens.

Need another token file? The `new-token-file` command creates it and registers it in your config in one go:

```sh
npx nemcss new-token-file radius --unit px --values "2,4,8" --names "sm,md,lg"
```

Add the directives to your CSS file:

```css
/* src/styles.css */
@nemcss base;
@nemcss utilities;
```

Then build:

```sh
npx nemcss build -i src/styles.css -o dist/styles.css
```

The `@nemcss base;` directive is replaced with the custom properties generated from your tokens, and `@nemcss utilities;` with the utility classes that are actually used in your content files:

```css
:root {
  --color-white: hsl(0, 0%, 100%);
  --color-black: hsl(0, 0%, 0%);
  --radius-sm: 2px;
  --radius-md: 4px;
  --radius-lg: 8px;
  --spacing-md: 1rem;
  /* ... */
  --text-default: var(--color-black);
  --bg-page: var(--color-white);
}

.p-md {
  padding: var(--spacing-md);
}
.text-default {
  color: var(--text-default);
}
.bg-page {
  background-color: var(--bg-page);
}
```

Use `nemcss watch` during development to rebuild on changes. There is also a Vite plugin and a PostCSS plugin if you prefer to integrate NemCSS into your existing build. Check out the [currently supported integrations](https://liv7c.github.io/nemcss/integrations/cli.html).

NemCSS also ships with an **LSP for autocomplete and hover docs** in your editor.

## Documentation

Check out the full documentation with examples and guides at **[liv7c.github.io/nemcss](https://liv7c.github.io/nemcss/)**.

## Packages

| Package                                             | Description                                                                 |
| --------------------------------------------------- | --------------------------------------------------------------------------- |
| [`nemcss`](packages/nemcss)                         | CLI with standalone `build`, `watch`, `init`, and `new-token-file` commands |
| [`@nemcss/vite`](packages/vite-plugin-nemcss)       | Vite plugin with HMR support                                                |
| [`@nemcss/postcss`](packages/postcss-plugin-nemcss) | PostCSS plugin                                                              |
| [VS Code extension](editors/vscode)                 | Autocomplete and hover docs via LSP                                         |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
