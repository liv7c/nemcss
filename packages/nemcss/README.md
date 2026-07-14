# nemcss

> A design-token-driven CSS custom properties and utility class generator

```sh
# global
npm install -g nemcss

# local (per project)
npm install -D nemcss
```

When installed locally, run commands via `npx nemcss <command>` or add them as scripts in your `package.json`.

## Quick start

```sh
npx nemcss init
npx nemcss new-token-file colors --prefix color --values "hsl(0, 0%, 100%),hsl(0, 0%, 0%)" --names "white,black"
```

`init` creates `nemcss.config.json` and an empty `design-tokens/` folder. `new-token-file` creates a token file and registers it in the config in one step.

Add the directives to your CSS input file:

```css
/* your CSS input file */
@nemcss base;
@nemcss utilities;
```

`@nemcss base;` is replaced with a `:root {}` block of CSS custom properties. `@nemcss utilities;` is replaced with the utility classes used in your content files. The `utilities` directive is optional.

Then build:

```sh
npx nemcss build -i src/styles.css -o dist/styles.css
```

## CLI commands

| Command                                | Description                                                                 |
| --------------------------------------- | ---------------------------------------------------------------------------- |
| `nemcss init`                           | Create a minimal `nemcss.config.json` and an empty `design-tokens/` folder |
| `nemcss new-token-file <name>`          | Create a token file and register it in the config                          |
| `nemcss build -i <input> -o <output>`   | One-shot build: scan content files and write CSS                           |
| `nemcss watch -i <input> -o <output>`   | Watch mode: rebuild on token, content, or config changes                   |
| `nemcss schema`                         | Print the JSON schema for `nemcss.config.json` to stdout                   |

## Configuration

A minimal `nemcss.config.json`:

```json
{
  "content": ["src/**/*.html", "src/**/*.tsx"],
  "tokensDir": "design-tokens",
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

Every token file must be registered under `theme` with an explicit `prefix`, like `colors` above. `new-token-file` handles this for you.

For the full configuration reference, see the [documentation](https://liv7c.github.io/nemcss).

## Integrations

If you use Vite or PostCSS, you can use a plugin instead of the standalone CLI:

- [`@nemcss/vite`](https://www.npmjs.com/package/@nemcss/vite): Vite plugin with HMR support
- [`@nemcss/postcss`](https://www.npmjs.com/package/@nemcss/postcss): PostCSS plugin

See the [integrations documentation](https://liv7c.github.io/nemcss/integrations/cli.html) for setup guides.

## Editor support

The [NemCSS VS Code extension](https://marketplace.visualstudio.com/items?itemName=liv7c.nemcss) provides autocomplete and hover docs for your tokens and utility classes via the built-in LSP.
