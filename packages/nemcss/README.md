# nemcss

> A design-token-driven CSS custom properties and utility class generator

```sh
# global
npm install -g nemcss

# local (per project)
npm install -D nemcss
```

When installed locally, run commands via `npx nemcss <command>` or add them as scripts in your `package.json`.

`nemcss` reads your design tokens and a `nemcss.config.json` file, then generates CSS custom properties and utility classes. Add the following directives to your CSS input file:

```css
/* your CSS input file */
@nemcss base;
@nemcss utilities;
```

`@nemcss base;` is replaced with a `:root {}` block of CSS custom properties. `@nemcss utilities;` is replaced with the utility classes used in your content files. The `utilities` directive is optional.

## CLI commands

Run `nemcss init` to scaffold a `nemcss.config.json` and example token files, then use `build` or `watch` to generate your CSS.

| Command                               | Description                                                                    |
| ------------------------------------- | ------------------------------------------------------------------------------ |
| `nemcss init`                         | Scaffold `nemcss.config.json` and example token files in the current directory |
| `nemcss build -i <input> -o <output>` | One-shot build: scan content files and write CSS                               |
| `nemcss watch -i <input> -o <output>` | Watch mode: rebuild on token, content, or config changes                       |
| `nemcss schema`                        | Print the JSON schema for `nemcss.config.json` to stdout                       |

## Configuration

A minimal `nemcss.config.json`:

```json
{
  "content": ["src/**/*.html", "src/**/*.tsx"],
  "tokensDir": "design-tokens",
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

For the full configuration reference, see the [documentation](https://liv7c.github.io/nemcss).

## Integrations

If you use Vite or PostCSS, you can use a plugin instead of the standalone CLI:

- [`@nemcss/vite`](https://www.npmjs.com/package/@nemcss/vite): Vite plugin with HMR support
- [`@nemcss/postcss`](https://www.npmjs.com/package/@nemcss/postcss): PostCSS plugin

See the [integrations documentation](https://liv7c.github.io/nemcss/integrations/cli.html) for setup guides.

## Editor support

The [NemCSS VS Code extension](https://marketplace.visualstudio.com/items?itemName=liv7c.nemcss) provides autocomplete and hover docs for your tokens and utility classes via the built-in LSP.
