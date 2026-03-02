# NemCSS

> A design-token-driven CSS utility generator

<a href="https://github.com/liv7c/nemcss/actions"><img src="https://img.shields.io/github/actions/workflow/status/liv7c/nemcss/ci.yml?branch=main" alt="CI Status"></a>
<a href="https://github.com/liv7c/nemcss/blob/main/LICENSE"><img src="https://img.shields.io/github/license/liv7c/nemcss" alt="License"></a>
<a href="https://www.npmjs.com/package/nemcss"><img src="https://img.shields.io/npm/v/nemcss" alt="Latest Release"></a>

NemCSS is a utility that generates CSS custom properties and utility classes based on your design tokens. The idea is you define your design tokens (for fonts, spacings, and so on), you add a `@nemcss base;` in your CSS, and `nemcss` replaces it at build time with the generated output. You can configure which utilities are generated and how they are named in a `nemcss.config.json` file. NemCSS has both a PostCSS and a Vite plugin to integrate within existing build systems. You can also get both **autocomplete and hover docs** in your editor thanks to the `nemcss` LSP (Language Server Protocol).

## Installation

There are different ways to use `nemcss`:

| Use case                                   | Package                                                                                   |
| ------------------------------------------ | ----------------------------------------------------------------------------------------- |
| CLI (standalone build/watch, init command) | `npm install -g nemcss`                                                                   |
| Vite integration                           | `npm install -D @nemcss/vite`                                                             |
| PostCSS integration                        | `npm install -D @nemcss/postcss`                                                          |
| VS Code extension (LSP)                    | [Visit the marketplace](https://marketplace.visualstudio.com/items?itemName=liv7c.nemcss) |
| Neovim / Helix / other editors             | Download `nemcss-lsp` from [GitHub Releases](https://github.com/liv7c/nemcss/releases)    |

## Quick start

### Step 1: initialize nemcss in your project

The quickest way to get started with `nemcss` is to use the CLI, via `npx`, a global or a local installation. Then, you can run inside of your project:

```sh
# if nemcss is globally
$ nemcss init
# or using npx
$ npx nemcss init
```

This command generates a few files:

- It generates a `design-tokens` folder if one does not exist yet. This folder contains two files, `spacings.json` and `colors.json`, that contain example design tokens.
- It also creates a `nemcss.config.json` file at the root of your project.

### Step 2: update the `content` field to point to your content files

In `nemcss.config.json`, edit the `content` field to point to your source files. This field contains a classic list of glob patterns that `nemcss` uses to only generate the classes your project uses at build time.

```json
{
  "content": [
    "src/**/*.html",
    "design-system/**/*.jsx",
    "src/**/*.tsx",
    "src/**/*.vue",
    "src/**/*.svelte",
    "src/**/*.astro"
  ]
}
```

### Step 3: add the `@nemcss base;` directive to your CSS input file

At build time, `nemcss` looks for a `@nemcss base;` directive in your CSS input file and replaces this directive with the custom properties and utilities.
Make sure to add this directive somewhere in your CSS files:

```css
@nemcss base;
```

### Step 4: use the build command

You can build your CSS using the `build` command from the `nemcss` CLI. The only thing you need is to pass both an input CSS file and an output file:

```sh
$ nemcss build -i src/styles.css -o dist/styles.css
```

In the output file, `@nemcss base;` is replaced with the custom properties generated from your design tokens. Only the utility classes you use will get generated at build time.

> Using Vite or PostCSS? Steps 1 to 3 are the same. For build setup, see [`@nemcss/vite`](packages/vite-plugin-nemcss/README.md) or [`@nemcss/postcss`](packages/postcss-plugin-nemcss/README.md).

## Configuration reference

There are three possible fields at the top level of the `nemcss.config.json`:

| Field       | Type     | Default           | Description                                                                                            |
| ----------- | -------- | ----------------- | ------------------------------------------------------------------------------------------------------ |
| `content`   | string[] | -                 | Glob patterns for source files. nemcss scans these to only generate used utility classes.              |
| `tokensDir` | string   | `"design-tokens"` | Path to the directory containing your token JSON files.                                                |
| `theme`     | object   | -                 | Token category configuration. Each key is a category name (e.g. `colors`). See below for more details. |

`nemcss` automatically discovers all `.json` files in `tokensDir`. The token type is determined by the filename (without extension). For recognized filenames, `nemcss` applies a default custom property prefix and generates default utility classes. No `theme` config is needed to get started.

| Filename                           | Custom property prefix | Default utility classes                                                                                                    |
| ---------------------------------- | ---------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `colors.json` / `color.json`       | `color`                | `.text-*` (color), `.bg-*` (background-color)                                                                              |
| `spacings.json` / `spacing.json`   | `spacing`              | `.p-*`, `.pt-*`, `.pr-*`, `.pb-*`, `.pl-*`, `.px-*`, `.py-*`, `.m-*`, `.mt-*`, `.mr-*`, `.mb-*`, `.ml-*`, `.mx-*`, `.my-*` |
| `fonts.json` / `font.json`         | `font`                 | `.font-*` (font-family)                                                                                                    |
| `font-sizes.json`                  | `font-sizes`           | `.text-*` (font-size)                                                                                                      |
| `font-weights.json`                | `font-weights`         | `.font-*` (font-weight)                                                                                                    |
| `shadows.json` / `shadow.json`     | `shadow`               | `.shadow-*` (box-shadow)                                                                                                   |
| `borders.json` / `border.json`     | `border`               | `.border-*` (border)                                                                                                       |
| `radii.json` / `radius.json`       | `radius`               | `.rounded-*` (border-radius)                                                                                               |
| `viewports.json` / `viewport.json` | `viewport`             | —                                                                                                                          |

Any other `.json` file in `tokensDir` is also discovered. In those cases, the filename is used as the prefix and no utility classes are generated unless you define them in `theme`.

To override a default or add custom utilities, add an entry to the `theme` block. For each token category, you can define the `prefix` for generated custom properties and a list of utility classes with your own naming conventions.

| Field       | Type     | Required | Description                                                                                          |
| ----------- | -------- | -------- | ---------------------------------------------------------------------------------------------------- |
| `source`    | string   | yes      | Path to the token JSON file for this category.                                                       |
| `prefix`    | string   | no       | Prefix for generated custom properties. `color` → `--color-...`                                      |
| `utilities` | object[] | no       | Utility classes to generate. Each entry has a `prefix` (class prefix) and `property` (CSS property). |

For example, this config overrides the prefix and adds a custom utility for the `colors` token:

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

With a `primary` token in `colors.json`, this produces the following CSS (where `@nemcss base;` was):

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

The default utilities (`.text-primary`, `.bg-primary`) are still generated — custom utilities are merged on top, not replacing the defaults. To override a specific default, reuse its prefix (e.g. adding `{ "prefix": "text", "property": "color" }` would replace the default `text` entry).

> If you have the `nemcss` LSP installed (via the VS Code extension or the standalone `nemcss-lsp` binary), all generated utility classes and custom properties are available for autocomplete and hover documentation directly in your editor.

## Packages in this repo

| Package | Description |
| --- | --- |
| [`nemcss`](packages/nemcss) | CLI — standalone build, watch, and init commands |
| [`@nemcss/vite`](packages/vite-plugin-nemcss) | Vite plugin — integrates nemcss into your Vite build with HMR support |
| [`@nemcss/postcss`](packages/postcss-plugin-nemcss) | PostCSS plugin — integrates nemcss into any PostCSS-based build |
| [VS Code extension](editors/vscode) | LSP-powered editor support — autocomplete and hover docs for utility classes and custom properties |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)

