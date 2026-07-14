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

This creates a minimal `nemcss.config.json` and an empty `design-tokens/` directory:

```json
{
  "content": [
    "src/**/*.html",
    "src/**/*.jsx",
    "src/**/*.tsx",
    "src/**/*.vue",
    "src/**/*.svelte",
    "src/**/*.astro"
  ],
  "tokensDir": "design-tokens",
  "theme": {}
}
```

There are no tokens yet. The `theme` block is where each token file gets registered, and it fills up as you add them in the next step.

## Step 2: Add your first token files

The `new-token-file` command creates a token file and registers it in your config in one go. Add a few colors:

```sh
npx nemcss new-token-file colors --prefix color --values "hsl(0, 0%, 100%),hsl(0, 0%, 0%)" --names "white,black"
```

This writes `design-tokens/colors.json`:

```json
{
  "title": "Colors Tokens",
  "description": "Design tokens for colors",
  "items": [
    { "name": "white", "value": "hsl(0, 0%, 100%)" },
    { "name": "black", "value": "hsl(0, 0%, 0%)" }
  ]
}
```

and adds the matching entry to your config:

```json
{
  "theme": {
    "colors": {
      "prefix": "color",
      "source": "design-tokens/colors.json"
    }
  }
}
```

Now a spacing scale. Numeric values get the `--unit` appended:

```sh
npx nemcss new-token-file spacings --prefix spacing --unit rem --values "0.5,1,1.5" --names "sm,md,lg"
```

A few variations worth knowing:

- `nemcss new-token-file --interactive` walks you through the same options with prompts.
- `--step` and `--count` generate a uniform scale instead of explicit values, e.g. `--unit rem --step 0.5 --count 6`.
- `ntf` is a shorthand alias: `nemcss ntf radius --unit px --values "2,4,8" --names "sm,md,lg"`.

Every token file must be registered under `theme`. If a `.json` file sits in `design-tokens/` without a matching entry, the build stops with an error naming the file. Since `new-token-file` registers as it creates, you only need to think about this when adding files by hand. See the [configuration reference](/guide/configuration) for details.

### Add utility classes

So far the config only generates CSS custom properties. To also get utility classes, add a `utilities` array to a theme entry. Each utility pairs a class prefix with a CSS property:

```json
{
  "theme": {
    "spacings": {
      "prefix": "spacing",
      "source": "design-tokens/spacings.json",
      "utilities": [
        { "prefix": "p", "property": "padding" },
        { "prefix": "m", "property": "margin" }
      ]
    }
  }
}
```

With the spacing tokens above, this generates classes like `p-sm`, `p-md`, `m-lg`. Leaving `utilities` off a category (like `colors` here) is fine: you still get its custom properties.

### Add a semantic layer (optional)

Primitive tokens name what a value is (`black`), not what it is for. The optional `semantic` block maps intent names onto your primitives:

```json
{
  "semantic": {
    "text": {
      "property": "color",
      "tokens": {
        "default": "{colors.black}",
        "inverted": "{colors.white}"
      }
    }
  }
}
```

This generates `--text-default` and `--text-inverted` custom properties that reference your color tokens, plus `.text-default` and `.text-inverted` utility classes. See [the semantic block](/guide/configuration#the-semantic-block) for the full reference.

## Step 3: Point `content` at your source files

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

## Step 4: Add `@nemcss base;` to your CSS

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

## Step 5: Build your CSS

```sh
nemcss build -i src/styles.css -o dist/styles.css
```

The output file will have each directive replaced with its generated CSS. With the tokens from step 2 and a `p-md` class used somewhere in your content files, the output looks like this:

```css
:root {
  --color-white: hsl(0, 0%, 100%);
  --color-black: hsl(0, 0%, 0%);
  --spacing-sm: 0.5rem;
  --spacing-md: 1rem;
  --spacing-lg: 1.5rem;
}

.p-md {
  padding: var(--spacing-md);
}
```

For watch mode during development:

```sh
nemcss watch -i src/styles.css -o dist/styles.css
```

## Next steps

- [Configuration reference](/guide/configuration): learn what you can configure, including the semantic token layer
- [Examples](/examples/vanilla): full walkthroughs for Vanilla HTML, Astro, and React
