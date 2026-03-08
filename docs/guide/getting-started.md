# Getting Started

The quickest way to get started is with the `nemcss` CLI. If you're using Vite or PostCSS, steps 1–4 are the same. Only the build step differs. See [Vite](/integrations/vite) or [PostCSS](/integrations/postcss) for integration-specific setup.

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

This generates the following files:

**`nemcss.config.json`**

```json
{
  "$schema": "./node_modules/nemcss/schemas/nemcss.config.schema.json",
  "content": [
    "src/**/*.html",
    "src/**/*.jsx",
    "src/**/*.tsx",
    "src/**/*.vue",
    "src/**/*.svelte",
    "src/**/*.astro"
  ],
  "tokensDir": "design-tokens",
  "theme": {
    "colors": {
      "prefix": "color",
      "source": "design-tokens/colors.json"
    },
    "spacings": {
      "prefix": "spacing",
      "source": "design-tokens/spacings.json",
      "utilities": [
        { "prefix": "p", "property": "padding" },
        { "prefix": "m", "property": "margin" }
      ]
    }
  },
  "semantic": {
    "text": {
      "property": "color",
      "tokens": {
        "default": "{colors.black}",
        "muted": "{colors.white}"
      }
    },
    "bg": {
      "property": "background-color",
      "tokens": {
        "page": "{colors.white}",
        "surface": "{colors.black}"
      }
    }
  }
}
```

**`design-tokens/colors.json`**

```json
{
  "title": "Color Tokens",
  "description": "Example design token file for colors",
  "items": [
    { "name": "white", "value": "hsl(0, 0%, 100%)" },
    { "name": "black", "value": "hsl(0, 0%, 0%)" }
  ]
}
```

**`design-tokens/spacings.json`**

```json
{
  "title": "Spacing Tokens",
  "description": "Example design token file for spacings",
  "items": [
    { "name": "0", "value": "0" },
    { "name": "xxs", "value": "0.125rem" },
    { "name": "xs", "value": "0.25rem" },
    { "name": "sm", "value": "0.5rem" },
    { "name": "md", "value": "1rem" },
    { "name": "lg", "value": "1.5rem" },
    { "name": "xl", "value": "2rem" },
    { "name": "xxl", "value": "3rem" }
  ]
}
```

The config gives you a starting point: color tokens with no utilities (custom properties only), spacing tokens with padding and margin utilities, and a semantic layer with `text` and `bg` groups scoped to your colors. Adjust to fit your project.

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

NemCSS uses two directives that it replaces at build time:

| Directive            | What it generates                                                               |
| -------------------- | ------------------------------------------------------------------------------- |
| `@nemcss base;`      | `:root { }` block with all CSS custom properties                                |
| `@nemcss utilities;` | Utility classes and responsive variants (only those used in your content files) |

Add one or both to your CSS entry file. Any other CSS in the file is preserved.

```css
/* other CSS can appear before */

@nemcss base;
@nemcss utilities;

/* or after */
body {
  background-color: var(--color-black);
}
```

You can easily scope the directives to separate cascade layers:

```css
@layer global, utilities;

@layer global {
  @nemcss base;
}

@layer utilities {
  @nemcss utilities;
}
```

A few things to keep in mind:

- Place the directive in your CSS entry file, or high enough in your CSS so the generated custom properties are available to the rest of your styles
- Use it once per file
- With the CLI, the directive must be in the file passed via `-i`. With the Vite and PostCSS plugins it can be in any CSS file processed by the pipeline
- The CLI will error if the directive is missing. The Vite and PostCSS plugins will silently skip the file

## Step 4: Build your CSS

```sh
nemcss build -i src/styles.css -o dist/styles.css
```

The output file will have each directive replaced with its generated CSS. Only the utility classes found in your content files are included.

For watch mode during development:

```sh
nemcss watch -i src/styles.css -o dist/styles.css
```

## Next steps

- [Configuration reference](/guide/configuration): learn what you can configure
- [Examples](/examples/vanilla): full walkthroughs for Vanilla HTML, Astro, and React
