# CLI

The `nemcss` CLI provides standalone `build`, `watch`, `init`, `new-token-file` and `schema` commands. No build tool required.

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

| Command                               | Description                                                                                                    |
| ------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `nemcss init`                         | Create a minimal `nemcss.config.json` and an empty `design-tokens/` folder in the current directory            |
| `nemcss build -i <input> -o <output>` | One-shot build: scan content files and write CSS                                                               |
| `nemcss watch -i <input> -o <output>` | Watch mode: rebuild on token, content, or config changes                                                       |
| `nemcss schema`                       | Print the JSON schema for `nemcss.config.json` to stdout                                                       |
| `nemcss new-token-file <NAME>`        | Generates a new token file and registers it in nemcss.config.json. Run it with `--help` to see all the options |

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
in `nemcss.config.json`. Requires an existing config. Run `nemcss init` first before trying this command.
Aliased as `nemcss ntf`.

Run it with explicit values with names:

```sh
nemcss new-token-file spacing --unit px --values "8,16,24,32" --names "sm,md,lg,xl"
```

You can also generate uniform scales (0.5, 1, 1.5):

```sh
nemcss new-token-file spacing --unit rem --step 0.5 --count 12
```

Or with arbitrary values (kept as such in the generated token file):

```sh
nemcss new-token-file font-size --unit rem --values "1,clamp(1.5rem, 1rem + 2vw, 2.5rem)" --names "md,fluid"
```

You can also generate a placeholder file you can edit manually:

```sh
nemcss new-token-file max-width
```

Here's a recap of all the command options:

| Flag                           | Description                                                                                                                   |
| ------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| `--values`                     | Comma-separated values. Numbers get `--unit` appended (`0` stays bare); anything else is kept verbatim and requires `--names` |
| `--step`, `--count`, `--start` | Generate a uniform scale instead; `--start` defaults to `--step`. Mutually exclusive with `--values`                          |
| `--names`                      | Token names, one per value. Numeric values name themselves when omitted                                                       |
| `--unit`                       | CSS unit appended to numeric values (`px`, `rem`, …)                                                                          |
| `--prefix`                     | Custom-property prefix registered in the config (defaults to the file name)                                                   |
| `--force`                      | Overwrite an existing token file or `theme` entry                                                                             |
| `--interactive`                | Run the command in interactive mode                                                                                           |

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
