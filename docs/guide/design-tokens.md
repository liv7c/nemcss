# Design Tokens

Design token files are your primitives. Colors, spacings, fonts, whatever forms the foundation of your UI. NemCSS reads them and generates CSS custom properties from them. The token files stay simple and focused on one thing: your raw values. All the wiring happens in `nemcss.config.json`.


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

## Using tokens in utilities

Utilities are CSS classes generated from your tokens. They are defined in `nemcss.config.json`, not in the token file itself. One utility class is generated per token in the category.

Given a token file:

```json
{
  "title": "Spacings",
  "items": [
    { "name": "xs", "value": "0.25rem" },
    { "name": "sm", "value": "0.5rem" },
    { "name": "md", "value": "1rem" }
  ]
}
```

You define which utilities to generate in your config:

```json
{
  "theme": {
    "spacings": {
      "source": "design-tokens/spacings.json",
      "prefix": "spacing",
      "utilities": [
        { "prefix": "p", "property": "padding" },
        { "prefix": "m", "property": "margin" }
      ]
    }
  }
}
```

This generates:

```css
:root {
  --spacing-xs: 0.25rem;
  --spacing-sm: 0.5rem;
  --spacing-md: 1rem;
}

.p-xs { padding: var(--spacing-xs); }
.p-sm { padding: var(--spacing-sm); }
.p-md { padding: var(--spacing-md); }
.m-xs { margin: var(--spacing-xs); }
.m-sm { margin: var(--spacing-sm); }
.m-md { margin: var(--spacing-md); }
```

## Using tokens in semantic groups

Semantic groups let you scope a subset of your tokens to a specific role in your UI. Token values reference your primitive tokens using the `{category.tokenName}` syntax, where `category` is the key used in your `theme` config.

Given the same color tokens:

```json
{
  "title": "Colors",
  "items": [
    { "name": "primary", "value": "#2563eb" },
    { "name": "muted", "value": "#64748b" }
  ]
}
```

You can define a semantic group in your config:

```json
{
  "semantic": {
    "text": {
      "property": "color",
      "tokens": {
        "primary": "{colors.primary}",
        "muted": "{colors.muted}"
      }
    }
  }
}
```

This generates:

```css
:root {
  --text-primary: var(--color-primary);
  --text-muted: var(--color-muted);
}

.text-primary { color: var(--text-primary); }
.text-muted { color: var(--text-muted); }
```

Your primitive color custom properties remain available. The semantic group adds a second layer with explicit intent.

See [Configuration](/guide/configuration) for the full reference.
