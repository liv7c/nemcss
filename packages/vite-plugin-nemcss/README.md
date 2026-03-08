# @nemcss/vite

> Vite plugin for [nemcss](../../README.md), a design-token-driven CSS utility generator

```sh
npm install -D @nemcss/vite
```

## Setup

```js
// vite.config.js
import { nemcss } from "@nemcss/vite";

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

| Option       | Type      | Default                | Description                                                                      |
| ------------ | --------- | ---------------------- | -------------------------------------------------------------------------------- |
| `configPath` | `string`  | `"nemcss.config.json"` | Path to the nemcss config file, relative to the Vite root.                       |
| `hmr`        | `boolean` | `true`                 | Reload CSS on token, content, or config file changes without a full page reload. |

## HMR

When `hmr` is enabled, the plugin watches the `tokensDir` directory and all `content` glob base directories defined in your `nemcss.config.json`. Any change triggers a rebuild and invalidates the CSS module without a full page reload.

For the full configuration reference, see the [root README](../../README.md).
