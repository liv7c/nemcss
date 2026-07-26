# NemCSS Configuration

NemCSS is configured via a `nemcss.config.json` file at the root of your project. Run `nemcss init` to scaffold one automatically.

## Top-level fields

| Field       | Type       | Default           | Description                                                                                                                           |
| ----------- | ---------- | ----------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| `content`   | `string[]` | `[]`              | Glob patterns for source files. NemCSS scans these to only generate used utility classes. If empty, no utility classes are generated. |
| `tokensDir` | `string`   | `"design-tokens"` | Path to the directory containing your token JSON files.                                                                               |
| `theme`     | `object`   | (none)            | Token category configuration. Each key is a category name (e.g. `colors`).                                                            |
| `semantic`  | `object`   | (none)            | Semantic token groups. Optional. See [the semantic block](#the-semantic-block).                                                       |
| `modes`     | `object`   | (none)            | Named modes that override semantic tokens. Optional. See [the modes block](#the-modes-block).                                          |

## The `theme` block

For each token category, you decide which utility classes to generate. A utility is defined by a `prefix` (the class name prefix) and a `property` (the CSS property). One utility class is generated per token in the category. If you don't define any utilities, none are generated. Custom properties are always generated regardless.

| Field       | Type       | Required | Description                                                                                                               |
| ----------- | ---------- | -------- | ------------------------------------------------------------------------------------------------------------------------- |
| `source`    | `string`   | yes      | Path to the token JSON file for this category, relative to the project root.                                              |
| `prefix`    | `string`   | yes      | Base name for generated custom properties. `"sp"` → `--sp-xxs`, `--sp-xs`, etc. |
| `utilities` | `object[]` | no       | Utility classes to generate. Each entry has a `prefix` (class prefix) and `property` (CSS property).                      |

### Naming

The key you use in `theme` (e.g. `"spacings"`) is the category name. It is what you reference in semantic token values (`{spacings.xxs}`). The `prefix` field controls the base name for the generated CSS custom properties. A category `"spacings"` with `"prefix": "sp"` produces `--sp-xxs`, `--sp-xs`, and so on.

**Every token file must be registered under `theme`.** If a .json file sits in your tokensDir without a matching entry, the build stops with an error naming the file. The easiest way to add an entry is `nemcss new-token-file`, which creates the file and registers it in one step.

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

Each entry defines a group with an optional CSS `property` and a `tokens` map. Token values reference your primitive tokens using the `{category.tokenName}` syntax.

| Field      | Type     | Required | Description                                                                                                                                                                       |
| ---------- | -------- | -------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `property` | `string` | no       | The CSS property for generated utility classes (e.g. `color`, `background-color`). **Omit it to generate only the CSS custom properties for this group with no utility classes.** |
| `tokens`   | `object` | yes      | A map of semantic token names to primitive token references (`{category.tokenName}`).                                                                                             |

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

.text-primary {
  color: var(--text-primary);
}
.text-secondary {
  color: var(--text-secondary);
}
.text-muted {
  color: var(--text-muted);
}
```

All your primitive color custom properties remain available. The semantic layer adds a second, intent-driven layer on top.

### Generating only custom properties (no utility classes)

If you only want the semantic CSS variables and don't need utility classes, omit the `property` field. The group's custom properties are still generated; no utility classes are.

```json
{
  "semantic": {
    "text": {
      "tokens": {
        "primary": "{colors.blue-600}",
        "secondary": "{colors.slate-500}",
        "muted": "{colors.slate-400}"
      }
    }
  }
}
```

This generates only the `:root` block:

```css
:root {
  --text-primary: var(--color-blue-600);
  --text-secondary: var(--color-slate-500);
  --text-muted: var(--color-slate-400);
}
```

## The `modes` block

The `modes` block is optional. Each key is a mode name, and each mode overrides semantic tokens under a media query or a selector. See [Modes](/guide/modes) for the guide.

| Field       | Type     | Required | Description                                                                                            |
| ----------- | -------- | -------- | ------------------------------------------------------------------------------------------------------ |
| `selector`  | `string` | no       | A CSS selector that activates the mode, e.g. `[data-theme="dark"]`. Cannot be combined with `media`.    |
| `media`     | `string` | no       | A media query that activates the mode, e.g. `(prefers-color-scheme: dark)`. Cannot be combined with `selector`. |
| `overrides` | `object` | no       | A map of semantic group name to a map of token name to primitive token reference.                       |

Every mode needs exactly one of `selector` or `media`. Declaring both is an error, and so is declaring neither.

### Example

```json
{
  "semantic": {
    "text": {
      "property": "color",
      "tokens": { "default": "{colors.slate-900}" }
    }
  },
  "modes": {
    "dark": {
      "media": "(prefers-color-scheme: dark)",
      "overrides": {
        "text": { "default": "{colors.slate-100}" }
      }
    }
  }
}
```

This generates:

```css
:root {
  --text-default: var(--color-slate-900);
}

@media (prefers-color-scheme: dark) {
  :root {
    --text-default: var(--color-slate-100);
  }
}
```

With `selector` instead of `media`, the same overrides are written under that selector:

```css
[data-theme='dark'] {
  --text-default: var(--color-slate-100);
}
```

### Overrides

Every group and token in `overrides` has to exist in your `semantic` block already, and every value has to be a `{category.tokenName}` reference. Raw CSS values are not accepted. Tokens you leave out keep their `:root` value.
