# CLI

The `nemcss` CLI provides standalone `build`, `watch`, and `init` commands. No build tool required.

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

| Command | Description |
| --- | --- |
| `nemcss init` | Scaffold `nemcss.config.json` and example token files in the current directory |
| `nemcss build -i <input> -o <output>` | One-shot build: scan content files and write CSS |
| `nemcss watch -i <input> -o <output>` | Watch mode: rebuild on token, content, or config changes |

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

## Configuration

A minimal `nemcss.config.json`:

```json
{
  "content": ["src/**/*.html", "src/**/*.tsx"],
  "tokensDir": "design-tokens"
}
```

See the full [configuration reference](/guide/configuration).
