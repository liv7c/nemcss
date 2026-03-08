# PostCSS

The `@nemcss/postcss` plugin integrates NemCSS into any PostCSS-based build.

## Installation

::: code-group

```sh [npm]
npm install -D @nemcss/postcss
```

```sh [pnpm]
pnpm add -D @nemcss/postcss
```

```sh [yarn]
yarn add -D @nemcss/postcss
```

```sh [bun]
bun add -D @nemcss/postcss
```

:::

## Setup

If you haven't already, run `nemcss init` in your project to generate `nemcss.config.json` and the `design-tokens/` folder:

::: code-group

```sh [npx]
npx nemcss init
```

```sh [pnpm dlx]
pnpm dlx nemcss init
```

```sh [yarn dlx]
yarn dlx nemcss init
```

:::

Then add the plugin to your PostCSS config:

```js
// postcss.config.js
import { nemcss } from "@nemcss/postcss";

export default {
  plugins: [nemcss()],
};
```

## Directives

The plugin looks for two directives in the input file and replaces them with generated CSS:

| Directive            | Output                                     |
| -------------------- | ------------------------------------------ |
| `@nemcss base;`      | `:root { --custom-properties }`            |
| `@nemcss utilities;` | Utility classes + responsive media queries |

The `@nemcss utilities` directive is optional. The plugin exits with an error if the `@nemcss base` directive is not found in the input file as it is the one responsible for injecting the custom properties derived fom the design tokens.

```css
/* Use both together */
@nemcss base;
@nemcss utilities;

/* Or scope to cascade layers */
@layer tokens, utilities;

@layer tokens {
  @nemcss base;
}

@layer utilities {
  @nemcss utilities;
}
```

## Plugin options

| Option | Type | Default | Description |
| --- | --- | --- | --- |
| `configPath` | `string` | `"nemcss.config.json"` | Path to the nemcss config file, relative to `cwd`. |
| `ignore` | `string[]` | `[]` | Additional glob patterns to exclude from content scanning. `node_modules` and `dist` are always excluded. |

```js
nemcss({
  configPath: 'config/nemcss.config.json',
  ignore: ['legacy/**'],
})
```

See the full [configuration reference](/guide/configuration).
