# nemcss

> A design-token-driven CSS utility generator

```sh
# global
npm install -g nemcss

# local (per project)
npm install -D nemcss
```

When installed locally, run commands via `npx nemcss <command>` or add them as scripts in your `package.json`.

`nemcss` reads your design tokens and a `nemcss.config.json` file, then generates CSS custom properties and utility classes. Add `@nemcss base;` to your CSS input file. `nemcss` replaces it at build time with the generated output.

## CLI commands

| Command                               | Description                                                                    |
| ------------------------------------- | ------------------------------------------------------------------------------ |
| `nemcss init`                         | Scaffold `nemcss.config.json` and example token files in the current directory |
| `nemcss build -i <input> -o <output>` | One-shot build: scan content files and write CSS                               |
| `nemcss watch -i <input> -o <output>` | Watch mode: rebuild on token, content, or config changes                       |

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

For the full configuration reference, see the [root README](../../README.md).
