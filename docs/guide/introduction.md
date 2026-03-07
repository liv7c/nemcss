# Introduction

NemCSS is a small tool that does one specific thing: take your design tokens and generate CSS custom properties and utility classes from them, using your own naming conventions.

Define your tokens in JSON files. One config file to declare your conventions and which utilities to generate. Add `@nemcss base;` to your CSS, and your custom properties and utilities are available everywhere. No magic defaults, no hidden utilities.

On top of your primitive tokens, a semantic layer lets you scope tokens to specific roles in your UI. Define which colors are for text, which are for backgrounds, directly from your config. A single source of truth, with explicit control over what gets generated and where.

## How it works

1. Define your design tokens in JSON files (e.g. `design-tokens/colors.json`)
2. Configure your conventions in `nemcss.config.json` — token prefixes, which utilities to generate, semantic aliases
3. Add `@nemcss base;` to your CSS input file
4. At build time, NemCSS scans your content files for used classes and replaces `@nemcss base;` with the generated custom properties and utility classes

## Packages

| Package | Description |
| --- | --- |
| [`nemcss`](https://www.npmjs.com/package/nemcss) | CLI with standalone `build`, `watch`, and `init` commands |
| [`@nemcss/vite`](https://www.npmjs.com/package/@nemcss/vite) | Vite plugin that integrates NemCSS into your Vite build with HMR support |
| [`@nemcss/postcss`](https://www.npmjs.com/package/@nemcss/postcss) | PostCSS plugin that integrates NemCSS into any PostCSS-based build |
| [VS Code extension](https://marketplace.visualstudio.com/items?itemName=liv7c.nemcss) | LSP-powered editor support with autocomplete and hover docs |
