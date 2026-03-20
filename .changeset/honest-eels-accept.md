---
"nemcss": minor
"@nemcss/cli-darwin-arm64": minor
"@nemcss/cli-darwin-x64": minor
"@nemcss/cli-linux-arm64": minor
"@nemcss/cli-linux-x64": minor
"@nemcss/cli-win32-x64": minor
---

Add `nemcss schema` subcommand and generate JSON schema from Rust types

- `nemcss schema` prints the JSON schema for `nemcss.config.json` to stdout
- The schema is now derived directly from the Rust type definitions — it cannot drift from the actual config shape
- `nemcss init` no longer emits a `$schema` field in the generated config
- Unknown top-level fields in `nemcss.config.json` now produce a clear error instead of being silently ignored

**Breaking**: `packages/nemcss/schemas/nemcss.config.schema.json` has been removed from the npm package. If your config has
`"$schema": "./node_modules/nemcss/schemas/nemcss.config.schema.json"`, remove that line or replace it with the output of `nemcss
schema`.
