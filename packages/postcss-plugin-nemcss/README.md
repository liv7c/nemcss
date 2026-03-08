# @nemcss/postcss

> PostCSS plugin for [nemcss](../../README.md), a design-token-driven CSS utility generator

```sh
npm install -D @nemcss/postcss
```

## Setup

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

For the full configuration reference, see the [root README](../../README.md).
