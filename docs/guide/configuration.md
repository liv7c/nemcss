# NemCSS Configuration

NemCSS is configured via a `nemcss.config.json` file at the root of your project. Run `nemcss init` to scaffold one automatically.

## Top-level fields

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `content` | `string[]` | `[]` | Glob patterns for source files. NemCSS scans these to only generate used utility classes. If empty, no utility classes are generated. |
| `tokensDir` | `string` | `"design-tokens"` | Path to the directory containing your token JSON files. |
| `theme` | `object` | (none) | Token category configuration. Each key is a category name (e.g. `colors`). |
| `semantic` | `object` | (none) | Semantic token groups. Optional. See [the semantic block](#the-semantic-block). |

## The `theme` block

For each token category, you decide which utility classes to generate. A utility is defined by a `prefix` (the class name prefix) and a `property` (the CSS property). One utility class is generated per token in the category. If you don't define any utilities, none are generated. Custom properties are always generated regardless.

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `source` | `string` | yes | Path to the token JSON file for this category, relative to the project root. |
| `prefix` | `string` | no | Base name for generated custom properties. `"sp"` → `--sp-xxs`, `--sp-xs`, etc. Defaults to the category name if not set. |
| `utilities` | `object[]` | no | Utility classes to generate. Each entry has a `prefix` (class prefix) and `property` (CSS property). |

### Naming

The key you use in `theme` (e.g. `"spacings"`) is the category name. It is what you reference in semantic token values (`{spacings.xxs}`). The category name is derived from the token file name without its extension (e.g. `spacings.json` → `"spacings"`). The `prefix` field controls the base name for the generated CSS custom properties. A category `"spacings"` with `"prefix": "sp"` produces `--sp-xxs`, `--sp-xs`, and so on. If `prefix` is not set, the category name is used.

::: tip Default prefix
For common category names, NemCSS applies a default singularization when `prefix` is not set: `colors` → `color`, `spacings` → `spacing`, `fonts` → `font`, `shadows` → `shadow`, `borders` → `border`, `radii` → `radius`. For anything else, the category name is used as-is.
:::

### Example

```json
{
  "theme": {
    "colors": {
      "source": "design-tokens/colors.json",
      "prefix": "color",
      "utilities": [
        { "prefix": "text", "property": "color" },
        { "prefix": "bg", "property": "background-color" }
      ]
    }
  }
}
```

With a `primary` token in `colors.json`, this produces:

```css
:root {
  --color-primary: #3b82f6;
}

.text-primary {
  color: var(--color-primary);
}
.bg-primary {
  background-color: var(--color-primary);
}
```

### The `property` field

The `property` field accepts any valid CSS property:

```json
{ "prefix": "text", "property": "color" }
{ "prefix": "bg", "property": "background-color" }
{ "prefix": "border", "property": "border-color" }
{ "prefix": "outline", "property": "outline-color" }
{ "prefix": "fill", "property": "fill" }
{ "prefix": "stroke", "property": "stroke" }
```

You can also use a CSS custom property as the target, useful for component-level theming:

```json
{ "prefix": "surface", "property": "--surface-color" }
```

For a `primary` token this generates:

```css
.surface-primary {
  --surface-color: var(--color-primary);
}
```

Components that reference `--surface-color` can be restyled just by changing which utility class you apply.

## The `semantic` block

The `semantic` block is optional. It lets you scope a subset of your primitive tokens to a specific role in your UI: text colors, background colors, surface colors, and so on.

Each entry defines a group with a CSS `property` and a `tokens` map. Token values reference your primitive tokens using the `{category.tokenName}` syntax.

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `property` | `string` | yes | The CSS property for generated utility classes (e.g. `color`, `background-color`). |
| `tokens` | `object` | yes | A map of semantic token names to primitive token references (`{category.tokenName}`). |

### Naming

The key you use in `semantic` (e.g. `"text"`) determines the base name for both the generated custom properties and utility classes. A group `"text"` with a token `"primary"` produces `--text-primary` and `.text-primary`.

### Example

```json
{
  "semantic": {
    "text": {
      "property": "color",
      "tokens": {
        "primary": "{colors.blue-600}",
        "secondary": "{colors.slate-500}",
        "muted": "{colors.slate-400}"
      }
    }
  }
}
```

This generates:

```css
:root {
  --text-primary: var(--color-blue-600);
  --text-secondary: var(--color-slate-500);
  --text-muted: var(--color-slate-400);
}

.text-primary { color: var(--text-primary); }
.text-secondary { color: var(--text-secondary); }
.text-muted { color: var(--text-muted); }
```

All your primitive color custom properties remain available. The semantic layer adds a second, intent-driven layer on top.
