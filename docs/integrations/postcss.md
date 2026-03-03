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
import { nemcss } from '@nemcss/postcss'

export default {
  plugins: [nemcss()],
}
```

Then add `@nemcss base;` to your CSS input file:

```css
@nemcss base;
```

`@nemcss base;` is replaced at build time with the CSS custom properties and utility classes generated from your design tokens.

## Options

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
