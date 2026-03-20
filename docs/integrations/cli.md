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
