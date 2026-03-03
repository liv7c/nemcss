# Getting Started

The quickest way to get started is with the `nemcss` CLI. If you're using Vite or PostCSS, steps 1–3 are the same. Only the build step differs. See [Vite](/integrations/vite) or [PostCSS](/integrations/postcss) for integration-specific setup.

## Step 0: Install nemcss

Install nemcss as a local dev dependency:

::: code-group
```sh [npm]
npm install -D nemcss
```
```sh [pnpm]
pnpm add -D nemcss
```
```sh [yarn]
yarn add -D nemcss
```
```sh [bun]
bun add -D nemcss
```
:::

Or install it globally to use `nemcss` anywhere on your machine:

::: code-group
```sh [npm]
npm install -g nemcss
```
```sh [pnpm]
pnpm add -g nemcss
```
```sh [yarn]
yarn global add nemcss
```
```sh [bun]
bun add -g nemcss
```
:::

## Step 1: Initialize nemcss in your project

Run the `init` command inside your project:

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

```sh [global]
nemcss init
```

:::

This generates:

- A `design-tokens/` folder with example `colors.json` and `spacings.json` files
- A `nemcss.config.json` file at the root of your project

## Step 2: Point `content` at your source files

Edit the `content` field in `nemcss.config.json` to match your project's source files. NemCSS scans these to only generate the utility classes your project actually uses.

```json
{
  "content": [
    "src/**/*.html",
    "src/**/*.tsx",
    "src/**/*.vue",
    "src/**/*.svelte",
    "src/**/*.astro"
  ]
}
```

## Step 3: Add `@nemcss base;` to your CSS

NemCSS looks for this directive in your CSS input file and replaces it at build time with the generated custom properties and utility classes.

```css
@nemcss base;
```

## Step 4: Build your CSS

```sh
nemcss build -i src/styles.css -o dist/styles.css
```

The output file will have `@nemcss base;` replaced with the generated CSS. Only the utility classes found in your content files are included.

For watch mode during development:

```sh
nemcss watch -i src/styles.css -o dist/styles.css
```

## Next steps

- [Configuration reference](/guide/configuration): learn what you can configure
- [Examples](/examples/vanilla): full walkthroughs for Vanilla HTML, Astro, and React
