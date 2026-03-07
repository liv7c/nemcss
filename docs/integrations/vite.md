# Vite

The `@nemcss/vite` plugin integrates NemCSS into your Vite build. It supports HMR so your CSS updates without a full page reload when you change tokens, config, or content files.

## Installation

::: code-group
```sh [npm]
npm install -D @nemcss/vite
```
```sh [pnpm]
pnpm add -D @nemcss/vite
```
```sh [yarn]
yarn add -D @nemcss/vite
```
```sh [bun]
bun add -D @nemcss/vite
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

Then add the plugin to your Vite config:

```js
// vite.config.js
import { defineConfig } from 'vite'
import { nemcss } from '@nemcss/vite'

export default defineConfig({
  plugins: [nemcss()],
})
```

Then add `@nemcss base;` to your CSS input file:

```css
@nemcss base;
```

`@nemcss base;` is replaced at build time with the CSS custom properties and utility classes generated from your design tokens.

## Options

| Option | Type | Default | Description |
| --- | --- | --- | --- |
| `configPath` | `string` | `"nemcss.config.json"` | Path to the nemcss config file, relative to the Vite root. |
| `hmr` | `boolean` | `true` | Reload CSS on token, content, or config file changes without a full page reload. |

```js
nemcss({
  configPath: 'config/nemcss.config.json',
  hmr: true,
})
```

## HMR

When `hmr` is enabled, the plugin watches the `tokensDir` directory and all `content` glob base directories defined in `nemcss.config.json`. Any change triggers a rebuild and invalidates the CSS module without a full page reload.

See the full [configuration reference](/guide/configuration).
