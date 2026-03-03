# Introduction

NemCSS is a utility that generates CSS custom properties and utility classes based on your design tokens.

The idea is simple: you define your design tokens (colors, spacings, fonts, etc.) in JSON files, add a `@nemcss base;` directive to your CSS input file, and NemCSS replaces it at build time with the generated output. You can configure which utilities are generated and how they are named in a `nemcss.config.json` file.

NemCSS also ships a Language Server Protocol (LSP) implementation, giving you **autocomplete and hover documentation** for every generated utility class and custom property directly in your editor.

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
