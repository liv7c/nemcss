# Editor Support

The `nemcss` LSP (Language Server Protocol) gives you autocomplete and hover documentation for every generated utility class and CSS custom property, directly in your editor.

## VS Code

Install the extension from the VS Code marketplace:

- [NemCSS (VS Code Marketplace)](https://marketplace.visualstudio.com/items?itemName=liv7c.nemcss)

Once installed, the extension automatically starts the language server when it detects a `nemcss.config.json` in your workspace. No additional configuration needed.

**What you get:**
- Autocomplete for all generated utility classes (e.g. `.text-primary`, `.p-md`)
- Autocomplete for all generated CSS custom properties (e.g. `--color-primary`)
- Hover documentation showing the resolved value for each token

**Supported file types:**
CSS, SCSS, Sass, Less, HTML, JavaScript, TypeScript, JSX, TSX, Vue, Svelte, Astro

## Neovim and other editors

For any editor that supports the Language Server Protocol, you can use the standalone `nemcss-lsp` binary.

### Download

Download the binary for your platform from [GitHub Releases](https://github.com/liv7c/nemcss/releases):

| Platform | Binary |
| --- | --- |
| macOS (Apple Silicon) | `nemcss-lsp-darwin-arm64` |
| macOS (Intel) | `nemcss-lsp-darwin-x64` |
| Linux (x64) | `nemcss-lsp-linux-x64` |
| Linux (ARM64) | `nemcss-lsp-linux-arm64` |
| Windows (x64) | `nemcss-lsp-win32-x64.exe` |

### Configure your editor

Point your LSP client at the `nemcss-lsp` binary and enable it for CSS files. The server communicates over stdio.

**Neovim (with `nvim-lspconfig`):**

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

if not configs.nemcss then
  configs.nemcss = {
    default_config = {
      cmd = { '/path/to/nemcss-lsp' },
      filetypes = { 'css', 'html', 'javascriptreact', 'typescriptreact', 'vue', 'svelte', 'astro' },
      root_dir = lspconfig.util.root_pattern('nemcss.config.json'),
    },
  }
end

lspconfig.nemcss.setup({})
```
