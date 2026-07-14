# Vanilla HTML

This guide walks through using NemCSS in a plain HTML project with no build framework. We'll use the `nemcss` CLI directly.

## Project structure

```
my-project/
  design-tokens/
    colors.json
    spacings.json
  src/
    styles.css
  dist/
    styles.css      (generated)
  index.html
  nemcss.config.json
```

## Step 1: Install nemcss

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

```sh [global]
npm install -g nemcss
```

:::

## Step 2: Initialize nemcss

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

```sh [global / local]
nemcss init
```

:::

This creates a minimal `nemcss.config.json` and an empty `design-tokens/` folder.

## Step 3: Add design tokens

Create a color and a spacing token file with `new-token-file`. It registers each one in your config as it creates it:

```sh
npx nemcss new-token-file colors --prefix color --values "hsl(0, 0%, 100%),hsl(0, 0%, 0%)" --names "white,black"
npx nemcss new-token-file spacings --prefix spacing --unit rem --values "0.5,1,1.5" --names "sm,md,lg"
```

Add a `padding` utility to the spacings entry, and a `text` semantic group so `text-default` and `text-muted` are available, by editing `nemcss.config.json`:

```json
{
  "theme": {
    "colors": {
      "prefix": "color",
      "source": "design-tokens/colors.json"
    },
    "spacings": {
      "prefix": "spacing",
      "source": "design-tokens/spacings.json",
      "utilities": [{ "prefix": "p", "property": "padding" }]
    }
  },
  "semantic": {
    "text": {
      "property": "color",
      "tokens": {
        "default": "{colors.black}",
        "muted": "{colors.white}"
      }
    }
  }
}
```

## Step 4: Configure content paths

Edit `nemcss.config.json` to point at your HTML files:

```json
{
  "content": ["*.html"],
  "tokensDir": "design-tokens"
}
```

## Step 5: Add the directives to your CSS

Create `src/styles.css` and add both directives:

```css
@nemcss base;
@nemcss utilities;
```

## Step 6: Link the output CSS in your HTML

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <title>My project</title>
    <link rel="stylesheet" href="dist/styles.css" />
  </head>
  <body>
    <h1 class="text-default">Hello NemCSS</h1>
    <p class="p-md text-muted">Styled with design tokens.</p>
  </body>
</html>
```

## Step 7: Build

```sh
npx nemcss build -i src/styles.css -o dist/styles.css
```

The `dist/styles.css` file will contain the generated custom properties and only the utility classes used in your HTML.

## Step 8: Watch during development

```sh
npx nemcss watch -i src/styles.css -o dist/styles.css
```

The output file rebuilds automatically when you change a token file, your config, or any content file.
