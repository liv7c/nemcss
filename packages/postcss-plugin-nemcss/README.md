# @nemcss/postcss

> PostCSS plugin for [nemcss](../../README.md), a design-token-driven CSS custom properties and utility class generator

```sh
npm install -D @nemcss/postcss
```

## Setup

This plugin reads your `nemcss.config.json`. Run `npx nemcss init` to scaffold one, or see the [nemcss package](https://www.npmjs.com/package/nemcss) for configuration details. PostCSS does not include HMR; use [`@nemcss/vite`](https://www.npmjs.com/package/@nemcss/vite) if you need hot module replacement.

```js
// postcss.config.js
import { nemcss } from "@nemcss/postcss";

export default {
  plugins: [nemcss()],
};
```

```css
/* your CSS input file */
@nemcss base;
@nemcss utilities;
```

`@nemcss base;` is replaced with a `:root {}` block of CSS custom properties. `@nemcss utilities;` is replaced with the utility classes used in your content files. The `utilities` directive is optional.

## Options

| Option       | Type       | Default                | Description                                                                                               |
| ------------ | ---------- | ---------------------- | --------------------------------------------------------------------------------------------------------- |
| `configPath` | `string`   | `"nemcss.config.json"` | Path to the nemcss config file, relative to `cwd`.                                                        |
| `ignore`     | `string[]` | `[]`                   | Additional glob patterns to exclude from content scanning. `node_modules` and `dist` are always excluded. |

For the full configuration reference, see the [documentation](https://liv7c.github.io/nemcss).

## Editor support

The [NemCSS VS Code extension](https://marketplace.visualstudio.com/items?itemName=liv7c.nemcss) provides autocomplete and hover docs for your tokens and utility classes via the built-in LSP.
