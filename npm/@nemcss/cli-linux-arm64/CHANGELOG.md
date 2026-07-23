# @nemcss/cli-linux-arm64

## 0.9.2

### Patch Changes

- f7f9108: Remove empty dead `bin` field from platform CLI packages

## 0.9.1

### Patch Changes

- c2b517a: Make watch mode more permissive. It was shutting down if the content list in nemcss config contained any glob for a folder not on disk yet.

## 0.9.0

### Minor Changes

- de138b7: Make token registration explicit in nemcss.config.json. All theme tokens now need an explicit prefix too. Init command has been simplified to only generate a config file and an empty tokens directory.

## 0.8.1

### Patch Changes

- 6621c21: Fix input CSS file not triggering a rebuild in watch mode.

## 0.8.0

### Minor Changes

- 6b67edc: Add new-token-file command to generate a token file via the CLI

## 0.7.2

### Patch Changes

- 2c5ae7c: We now normalize the content file paths before adding them to the glob. Previously, if a user wrote paths like "./index.html" in nemcss config content section, it caused issues with the CLI watcher. By normalizing the paths before adding them to the glob set, we now match them correctly.

## 0.7.1

### Patch Changes

- 84f4f5c: Add publishConfig for pnpm so that CLI executable files keep their executable permission when published via pnpm publish

## 0.6.0

### Minor Changes

- 28aabec: Add `nemcss schema` subcommand and generate JSON schema from Rust types
  - `nemcss schema` prints the JSON schema for `nemcss.config.json` to stdout
  - The schema is now derived directly from the Rust type definitions — it cannot drift from the actual config shape
  - `nemcss init` no longer emits a `$schema` field in the generated config
  - Unknown top-level fields in `nemcss.config.json` now produce a clear error instead of being silently ignored

  **Breaking**: `packages/nemcss/schemas/nemcss.config.schema.json` has been removed from the npm package. If your config has
  `"$schema": "./node_modules/nemcss/schemas/nemcss.config.schema.json"`, remove that line or replace it with the output of `nemcss
schema`.

## 0.5.0

### Minor Changes

- cd01f70: Add schema subcommand and versioned JSON schema URL in init
