# Changelog

This repository publishes several packages, each versioned independently with
[Changesets](https://github.com/changesets/changesets). Per-package changelogs are
the source of truth — see the one for the package you care about:

- [`nemcss`](packages/nemcss/CHANGELOG.md) — CLI and core npm package
- [`@nemcss/napi`](crates/napi/CHANGELOG.md) — native Node bindings (engine)
- [`@nemcss/vite`](packages/vite-plugin-nemcss/CHANGELOG.md) — Vite plugin
- [`@nemcss/postcss`](packages/postcss-plugin-nemcss/CHANGELOG.md) — PostCSS plugin
- [`nemcss-vscode`](editors/vscode/CHANGELOG.md) — VS Code extension

The platform CLI binaries under `npm/@nemcss/cli-*` are versioned in lockstep with
`nemcss`.

> To add an entry, run `pnpm changeset` — do **not** edit changelogs by hand.

---

## Archive

Entries below predate the move to Changesets (≤ v0.2.1) and are kept for history.

## [0.2.1] - 2026-03-10

### ⬆️ Dependencies

- *(lsp)* Bump tokio to 1.50.0 (runtime and sync bug fixes)

### 📚 Documentation

- Improve npm package README

## [0.2.0] - 2026-03-08

### 🚀 Features

- *(engine)* Split to_css into base_to_css and utilities_to_css
- *(engine)* Remove to_css method
- *(napi)* Use new base_to_css and utilities_to_css methods
- *(cli)* Use split base_to_css and utilities_to_css methods
- *(postcss)* Split base and utilities css generation
- *(vite-plugin)* Update plugin to support both base and utilities nemcss directives

### 🐛 Bug Fixes

- Address pre-release review issues
