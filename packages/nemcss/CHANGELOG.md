# Changelog

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

## [0.4.0](https://github.com/liv7c/nemcss/compare/nemcss-v0.3.0...nemcss-v0.4.0) (2026-03-14)

### Features

- make adding utilities explicit with semantic layer ([#34](https://github.com/liv7c/nemcss/issues/34)) ([d59d899](https://github.com/liv7c/nemcss/commit/d59d899cc4af649e3fb22316d5e343757cecdce4))
- **npm:** add npm wrapper for nemcss CLI and scope plugin packages ([#30](https://github.com/liv7c/nemcss/issues/30)) ([28eb0f0](https://github.com/liv7c/nemcss/commit/28eb0f0bcc17307facc35521212e3e8d96314ac7))

## [0.3.0](https://github.com/liv7c/nemcss/compare/nemcss-v0.2.1...nemcss-v0.3.0) (2026-03-13)

### Features

- make adding utilities explicit with semantic layer ([#34](https://github.com/liv7c/nemcss/issues/34)) ([d59d899](https://github.com/liv7c/nemcss/commit/d59d899cc4af649e3fb22316d5e343757cecdce4))
- **npm:** add npm wrapper for nemcss CLI and scope plugin packages ([#30](https://github.com/liv7c/nemcss/issues/30)) ([28eb0f0](https://github.com/liv7c/nemcss/commit/28eb0f0bcc17307facc35521212e3e8d96314ac7))
