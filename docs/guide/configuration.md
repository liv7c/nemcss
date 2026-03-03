# Configuration

NemCSS is configured via a `nemcss.config.json` file at the root of your project. Run `nemcss init` to scaffold one automatically.

## Top-level fields

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `content` | `string[]` | (none) | Glob patterns for source files. NemCSS scans these to only generate used utility classes. |
| `tokensDir` | `string` | `"design-tokens"` | Path to the directory containing your token JSON files. |
| `theme` | `object` | (none) | Token category configuration. Each key is a category name (e.g. `colors`). |

## Built-in token types

NemCSS automatically discovers all `.json` files in `tokensDir`. For recognized filenames it applies default custom property prefixes and generates default utility classes. No `theme` config is needed to get started.

| Filename | Custom property prefix | Default utility classes |
| --- | --- | --- |
| `colors.json` / `color.json` | `color` | `.text-*` (color), `.bg-*` (background-color) |
| `spacings.json` / `spacing.json` | `spacing` | `.p-*`, `.pt-*`, `.pr-*`, `.pb-*`, `.pl-*`, `.px-*`, `.py-*`, `.m-*`, `.mt-*`, `.mr-*`, `.mb-*`, `.ml-*`, `.mx-*`, `.my-*` |
| `fonts.json` / `font.json` | `font` | `.font-*` (font-family) |
| `font-sizes.json` | `font-sizes` | `.text-*` (font-size) |
| `font-weights.json` | `font-weights` | `.font-*` (font-weight) |
| `shadows.json` / `shadow.json` | `shadow` | `.shadow-*` (box-shadow) |
| `borders.json` / `border.json` | `border` | `.border-*` (border) |
| `radii.json` / `radius.json` | `radius` | `.rounded-*` (border-radius) |
| `viewports.json` / `viewport.json` | `viewport` | (none) |

Any other `.json` file in `tokensDir` is discovered too. The filename becomes the prefix and no utility classes are generated unless you define them in `theme`.

## The `theme` block

Use `theme` to override defaults or add custom utilities for a token category.

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `source` | `string` | yes | Path to the token JSON file for this category. |
| `prefix` | `string` | no | Prefix for generated custom properties. `"brand"` → `--brand-...` |
| `utilities` | `object[]` | no | Utility classes to generate. Each entry has a `prefix` (class prefix) and `property` (CSS property). |

### Example: custom prefix and extra utility

```json
{
  "theme": {
    "colors": {
      "source": "design-tokens/colors.json",
      "prefix": "brand",
      "utilities": [{ "prefix": "highlight", "property": "background-color" }]
    }
  }
}
```

With a `primary` token in `colors.json`, this produces:

```css
:root {
  --brand-primary: #3b82f6;
}

.text-primary {
  color: var(--brand-primary);
}
.bg-primary {
  background-color: var(--brand-primary);
}
.highlight-primary {
  background-color: var(--brand-primary);
}
```

The default utilities (`.text-*`, `.bg-*`) are still generated. Custom entries are merged on top. To replace a default, reuse its prefix (e.g. `{ "prefix": "text", "property": "color" }`).
