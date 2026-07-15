# Config File Autocomplete

Editing `nemcss.config.json` gets you completion, hover docs, and validation for the config shape itself: property names, types, and required fields. This is separate from the [LSP](/editor/) that completes generated classes inside your CSS and markup.

## VS Code with the nemcss extension

Install the [NemCSS extension](https://marketplace.visualstudio.com/items?itemName=liv7c.nemcss) and you get autocomplete in the config for free. Open any `nemcss.config.json` and you get completion, hover text, and red squiggles on invalid keys automatically.

## Neovim (`jsonls`)

Associate the schema manually. This is a workaround until nemcss is registered in the SchemaStore catalog; once that ships, `schemastore.nvim` users get it automatically.

```lua
require("lspconfig").jsonls.setup({
  settings = {
    json = {
      schemas = {
        {
          fileMatch = { "nemcss.config.json" },
          url = "https://liv7c.github.io/nemcss/schema/nemcss.config.schema.json",
        },
      },
    },
  },
})
```

## Any other editor

Add a `"$schema"` key to your config:

```json
{
  "$schema": "https://liv7c.github.io/nemcss/schema/nemcss.config.schema.json",
  ...
}
```

This is the fallback, not the recommendation. Most editors do it via filename association above, without touching the config file.
