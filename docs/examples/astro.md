# Astro

This guide walks through setting up NemCSS in an Astro project using `@nemcss/vite` via Astro's `vite` config option.

::: warning Vite plugin type compatibility
There is a known type compatibility issue between Vite plugins built against Vite 7 and Astro projects that still use Vite 6. You may see the following TypeScript error:

```
Type 'Plugin[]' is not assignable to type 'PluginOption'
```

If you hit this, add `// @ts-expect-error` above the plugin line in your Astro config (see Step 3), or use [`@nemcss/postcss`](/integrations/postcss) instead.

See [withastro/astro#14030](https://github.com/withastro/astro/issues/14030) for more details.
:::

## Step 1: Create an Astro project

::: code-group

```sh [npm]
npm create astro@latest
```

```sh [pnpm]
pnpm create astro@latest
```

```sh [yarn]
yarn create astro
```

:::

## Step 2: Install `@nemcss/vite`

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

## Step 3: Add the plugin to your Astro config

```js
// astro.config.mjs
import { defineConfig } from "astro/config";
import { nemcss } from "@nemcss/vite";

export default defineConfig({
  vite: {
    plugins: [
      // @ts-expect-error: Vite plugin type mismatch between Vite 6 (Astro) and Vite 7
      nemcss(),
    ],
  },
});
```

## Step 4: Initialize nemcss

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

This creates `nemcss.config.json` and a `design-tokens/` folder with example token files.

## Step 5: Configure content paths

Edit `nemcss.config.json` to point at your Astro components and pages:

```json
{
  "content": ["src/**/*.astro", "src/**/*.ts", "src/**/*.tsx"],
  "tokensDir": "design-tokens"
}
```

## Step 6: Add `@nemcss base;` to a global CSS file

Create `src/styles/global.css`:

```css
@nemcss base;
```

## Step 7: Import the global CSS in your layout

```astro
---
// src/layouts/Layout.astro
import '../styles/global.css'
---
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <title>My Astro site</title>
  </head>
  <body>
    <slot />
  </body>
</html>
```

## Step 8: Use the generated classes

```astro
---
// src/pages/index.astro
import Layout from '../layouts/Layout.astro'
---
<Layout>
  <h1 class="text-primary">Hello NemCSS</h1>
  <p class="p-md text-secondary">Styled with design tokens.</p>
</Layout>
```

## Step 9: Start the dev server

::: code-group

```sh [npm]
npm run dev
```

```sh [pnpm]
pnpm dev
```

```sh [yarn]
yarn dev
```

:::

HMR is enabled by default. Your CSS updates without a full page reload when you change a token file, your config, or any content file.
