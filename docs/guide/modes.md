# Modes

A mode is a named set of overrides for your semantic tokens. Dark mode is the usual reason to reach for this, but nothing about it is specific to dark.

Modes only change the value of semantic custom properties. Your utility classes already point at those properties, so they keep working and no extra class is generated.

## A dark mode

Modes override semantic tokens, so you need a `semantic` block first:

```json
{
  "semantic": {
    "text": {
      "property": "color",
      "tokens": { "default": "{colors.gray-900}" }
    }
  },
  "modes": {
    "dark": {
      "media": "(prefers-color-scheme: dark)",
      "overrides": {
        "text": { "default": "{colors.gray-100}" }
      }
    }
  }
}
```

This generates:

```css
:root {
  --text-default: var(--color-gray-900);
}

@media (prefers-color-scheme: dark) {
  :root {
    --text-default: var(--color-gray-100);
  }
}
```

Anything using `.text-default` or `var(--text-default)` picks up the new value when the query matches. Tokens you leave out keep their `:root` value.

## How a mode turns on

Every mode declares exactly one of `media` or `selector`. Both is an error, neither is an error, and there is no default.

| Field       | Type     | Description                                                                       |
| ----------- | -------- | --------------------------------------------------------------------------------- |
| `media`     | `string` | A media query that activates the mode. Cannot be combined with `selector`.        |
| `selector`  | `string` | A CSS selector that activates the mode. Cannot be combined with `media`.          |
| `overrides` | `object` | A map of semantic group name to a map of token name to primitive token reference. |

### `media`

For modes that follow the user's system settings and are not meant to be switched off. Any media feature works:

```json
{
  "modes": {
    "contrast": {
      "media": "(prefers-contrast: more)",
      "overrides": {
        "text": { "default": "{colors.black}" }
      }
    }
  }
}
```

### `selector`

For modes you switch yourself. Any selector works:

```json
{
  "modes": {
    "dark": {
      "selector": "[data-theme='dark']",
      "overrides": {
        "text": { "default": "{colors.gray-100}" }
      }
    }
  }
}
```

```css
[data-theme='dark'] {
  --text-default: var(--color-gray-100);
}
```

Put it on the root element to switch the page, or on a smaller element to scope the mode to part of it:

```html
<html data-theme="dark">
  <body>
    <p class="text-default">This uses the dark value.</p>
  </body>
</html>
```

Getting the attribute onto the page is your app's job. A cookie works well if you render on a server, since the attribute can go straight into the HTML and there is no flash of the wrong colors.

## What you can override

Every group and token in `overrides` has to exist in your `semantic` block already, and every value has to be a `{category.tokenName}` reference. Raw CSS values are not accepted. If something does not resolve, the build stops and names it.

## Where the CSS ends up

Each mode generates one block, written where you put `@nemcss base;`, right after `:root`. Modes are written in alphabetical order by name, which decides the outcome if two of them apply at once.

See [Configuration](/guide/configuration) for the full reference.
