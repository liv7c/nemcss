# Changelog

## 0.4.0

### Minor Changes

- 289bada: Improve error handling and DX across the LSP and plugins. The LSP no longer breaks on a bad token file or config, and shows diagnostics in the editor. The Vite and PostCSS now fail the build with a clear message and help text instead of silently producing broken CSS.

### Patch Changes

- Updated dependencies [289bada]
  - @nemcss/napi@0.7.0

## 0.3.8

### Patch Changes

- Updated dependencies [36a8496]
  - @nemcss/napi@0.6.1

## 0.3.7

### Patch Changes

- 498dda5: Attach messages to plugin to trigger CSS rebuild on key file changes (such as config file or token files)

## 0.3.6

### Patch Changes

- 9acca2b: Add a CommonJS build output and default export to support plugin loaders loading plugins by name

## 0.3.5

### Patch Changes

- 6976e2b: Fix a regression introduced by the upgrade to pnpm v11. With the new version, we need to specify a `files` property in the package json to include the dist files.

## 0.3.4

### Patch Changes

- Updated dependencies [1c7a7be]
  - @nemcss/napi@0.6.0

## 0.3.3

### Patch Changes

- Updated dependencies [de138b7]
  - @nemcss/napi@0.5.0

## 0.3.2

### Patch Changes

- Updated dependencies [a1a35cd]
  - @nemcss/napi@0.4.1

## 0.3.1

### Patch Changes

- Updated dependencies [157e502]
  - @nemcss/napi@0.4.0

This package is versioned and released together with `nemcss`.
See [packages/nemcss/CHANGELOG.md](../nemcss/CHANGELOG.md) for the full changelog.
