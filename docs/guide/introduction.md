# Introduction

NemCSS connects your design token JSON files to your CSS. Define your tokens as simple name/value pairs, then configure in `nemcss.config.json` the custom property prefix and the utility classes to derive from each token. Only the utilities your project actually uses end up in the final CSS.

NemCSS also ships a LSP to smooth out your developer experience. You get autocomplete and hover documentation for all generated utility classes and custom properties directly in your editor. Add a new token or update your config and your IDE picks it up instantly.

## How it works

1. You define design tokens in JSON files (e.g. `design-tokens/colors.json`)
2. You add `@nemcss base;` somewhere in your CSS input file
3. At build time, NemCSS reads your tokens and your `nemcss.config.json`, scans your content files for used utility classes, and replaces `@nemcss base;` with the generated CSS custom properties and utility classes

## Packages

| Package | Description |
| --- | --- |
| [`nemcss`](https://www.npmjs.com/package/nemcss) | CLI with standalone `build`, `watch`, and `init` commands |
| [`@nemcss/vite`](https://www.npmjs.com/package/@nemcss/vite) | Vite plugin that integrates NemCSS into your Vite build with HMR support |
| [`@nemcss/postcss`](https://www.npmjs.com/package/@nemcss/postcss) | PostCSS plugin that integrates NemCSS into any PostCSS-based build |
| [VS Code extension](https://marketplace.visualstudio.com/items?itemName=liv7c.nemcss) | LSP-powered editor support with autocomplete and hover docs |
