# Design Tokens

Design tokens are the source of truth for your styles. NemCSS reads token files from your `tokensDir` directory and transforms them into CSS custom properties and utility classes.

## Token file format

Each token file is a JSON file with a `title`, an optional `description`, and an `items` array:

```json
{
  "title": "Colors",
  "description": "Brand color palette used across the design system.",
  "items": [
    { "name": "primary", "value": "#3b82f6" },
    { "name": "secondary", "value": "#64748b" }
  ]
}
```

| Field         | Required | Description                                               |
| ------------- | -------- | --------------------------------------------------------- |
| `title`       | yes      | Human-readable name for this token group.                 |
| `description` | no       | Optional description of what these tokens represent.      |
| `items`       | yes      | Array of token entries, each with a `name` and a `value`. |

Place these files in your `tokensDir` (defaults to `design-tokens/`). The filename determines the token category (e.g. `colors.json` is the `colors` category).

## Token value types

### Simple tokens

A simple token has a single string as its value:

```json
{ "name": "primary", "value": "#3b82f6" }
```

This generates:

```css
:root {
  --color-primary: #3b82f6;
}
```

### List tokens

A list token has an array of strings as its value. The values are joined with commas in the output. This is useful for things like font stacks or multi-value properties:

```json
{ "name": "base", "value": ["Satoshi", "Inter", "sans-serif"] }
```

This generates:

```css
:root {
  --font-base: Satoshi, Inter, sans-serif;
}
```

## Category names and prefixes

Custom property names follow the pattern `--{prefix}-{tokenName}`. The prefix comes from the token's **category name**, which is derived from the filename.

### How it works

**Step 1: filename to category name**

NemCSS strips the `.json` extension to get the category name:

```
colors.json       →  category: "colors"
brand-colors.json →  category: "brand-colors"
font-weights.json →  category: "font-weights"
```

**Step 2: category name to prefix**

For built-in category names, NemCSS applies a singularization to produce a clean prefix. For anything else, the category name is used as-is:

| Category name | Default prefix        |
| ------------- | --------------------- |
| `colors`      | `color`               |
| `spacings`    | `spacing`             |
| `fonts`       | `font`                |
| `shadows`     | `shadow`              |
| `borders`     | `border`              |
| `radii`       | `radius`              |
| `viewports`   | `viewport`            |
| anything else | same as category name |

So `colors.json` produces `--color-*`, but `brand-colors.json` produces `--brand-colors-*` by default.

### Custom prefix

To use a different prefix, set it explicitly in `nemcss.config.json` under the category name as the key:

```json
{
  "theme": {
    "brand-colors": {
      "source": "design-tokens/brand-colors.json",
      "prefix": "brand"
    }
  }
}
```

With a `primary` token in `brand-colors.json`, this produces `--brand-primary` instead of `--brand-colors-primary`.

The same applies to built-in categories if you want to rename their prefix:

```json
{
  "theme": {
    "colors": {
      "source": "design-tokens/colors.json",
      "prefix": "c"
    }
  }
}
```

This produces `--c-primary` instead of `--color-primary`.

## Utilities

Utilities are CSS classes generated from your tokens. Each utility has a `prefix` (the class name prefix) and a `property` (the CSS declaration to generate).

```json
{
  "theme": {
    "colors": {
      "source": "design-tokens/colors.json",
      "utilities": [
        { "prefix": "text", "property": "color" },
        { "prefix": "bg", "property": "background-color" }
      ]
    }
  }
}
```

For a `primary` token this generates:

```css
.text-primary {
  color: var(--color-primary);
}
.bg-primary {
  background-color: var(--color-primary);
}
```

### The `property` field

The `property` field accepts any valid CSS property. The possibilities are as wide as CSS itself:

```json
{ "prefix": "text", "property": "color" }
{ "prefix": "bg", "property": "background-color" }
{ "prefix": "border", "property": "border-color" }
{ "prefix": "outline", "property": "outline-color" }
{ "prefix": "fill", "property": "fill" }
{ "prefix": "stroke", "property": "stroke" }
```

You can also use a **CSS custom property** as the target. This lets you feed a token's value into another custom property, which is useful for theming.

```json
{ "prefix": "surface", "property": "--surface-color" }
```

For a `primary` token this generates:

```css
.surface-primary {
  --surface-color: var(--color-primary);
}
```

This way, components that reference `--surface-color` can be restyled just by changing which utility class you apply.

### Merging with defaults

Custom utilities are merged with the built-in defaults for that category, not replaced. To replace a default utility, reuse its prefix:

```json
{
  "theme": {
    "colors": {
      "source": "design-tokens/colors.json",
      "utilities": [{ "prefix": "text", "property": "color" }]
    }
  }
}
```

This replaces the default `text` utility while keeping any others (like `bg`) intact.
