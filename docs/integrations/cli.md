# CLI

The `nemcss` CLI provides standalone `build`, `watch`, `init`, and `schema` commands. No build tool required.

## Installation

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

When installed locally, run commands via `npx nemcss <command>` or add them as scripts in your `package.json`.

## Commands

| Command                               | Description                                                                    |
| ------------------------------------- | ------------------------------------------------------------------------------ |
| `nemcss init`                         | Scaffold `nemcss.config.json` and example token files in the current directory |
| `nemcss build -i <input> -o <output>` | One-shot build: scan content files and write CSS                               |
| `nemcss watch -i <input> -o <output>` | Watch mode: rebuild on token, content, or config changes                       |
| `nemcss schema`                       | Print the JSON schema for `nemcss.config.json` to stdout                       |

### `init`

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

### `build`

```sh
nemcss build -i src/styles.css -o dist/styles.css
```

### `watch`

```sh
nemcss watch -i src/styles.css -o dist/styles.css
```

### `schema`

Prints the JSON schema for `nemcss.config.json` to stdout. Useful for piping into a file or validating the schema shape.

```sh
nemcss schema > nemcss.config.schema.json
```

### `new-token-file`

Generates a design token file in your `tokensDir` and registers it under `theme`
in `nemcss.config.json`. Requires an existing config — run `nemcss init` first.
Aliased as `nemcss ntf`.

```sh
# explicit values with names
nemcss new-token-file spacing --unit px --values "8,16,24,32" --names "sm,md,lg,xl"

# generated uniform scale (0.5rem, 1rem, … 6rem)
nemcss new-token-file spacing --unit rem --step 0.5 --count 12

# arbitrary CSS values are kept verbatim — commas inside () are safe
nemcss new-token-file font-size --unit rem --values "1,clamp(1.5rem, 1rem + 2vw, 2.5rem)" --names "md,fluid"

# empty placeholder file to fill in by hand
nemcss new-token-file max-width
```

| Flag                           | Description                                                                                                                   |
| ------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| `--values`                     | Comma-separated values. Numbers get `--unit` appended (`0` stays bare); anything else is kept verbatim and requires `--names` |
| `--step`, `--count`, `--start` | Generate a uniform scale instead; `--start` defaults to `--step`. Mutually exclusive with `--values`                          |
| `--names`                      | Token names, one per value. Numeric values name themselves when omitted                                                       |
| `--unit`                       | CSS unit appended to numeric values (`px`, `rem`, …)                                                                          |
| `--prefix`                     | Custom-property prefix registered in the config (defaults to the file name)                                                   |
| `--force`                      | Overwrite an existing token file or `theme` entry                                                                             |

## Directives

The CLI looks for two directives in the input file and replaces them with generated CSS:

| Directive            | Output                                     |
| -------------------- | ------------------------------------------ |
| `@nemcss base;`      | `:root { --custom-properties }`            |
| `@nemcss utilities;` | Utility classes + responsive media queries |

The `@nemcss utilities` directive is optional. The CLI exits with an error if the `@nemcss base` directive is not found in the input file as it is the one responsible for injecting the custom properties derived fom the design tokens.

```css
/* Use both together */
@nemcss base;
@nemcss utilities;

/* Or scope to cascade layers */
@layer tokens, utilities;

@layer tokens {
  @nemcss base;
}

@layer utilities {
  @nemcss utilities;
}
```

## Configuration

A minimal `nemcss.config.json`:

```json
{
  "content": ["src/**/*.html", "src/**/*.tsx"],
  "tokensDir": "design-tokens"
}
```

See the full [configuration reference](/guide/configuration).
