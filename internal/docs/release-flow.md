# Release Flow

NemCSS uses [changesets](https://github.com/changesets/changesets) for versioning and release management.

## Overview

```
PR merged with changeset file
  → changesets/action creates or updates a "Version Packages" PR
  → that PR bumps package.json versions, updates changelogs,
    and syncs Cargo.toml + platform CLI versions (via scripts/version.mjs)

Merge "Version Packages" PR
  → scripts/release.mjs creates git tags (v*, editor-v*, vite-v*, postcss-v*)
  → v* tag  → release-core.yml  (builds CLI+LSP+NAPI, publishes npm, creates GitHub release)
  → editor-v* tag → release-editor.yml (builds LSP, publishes VSIX, creates GitHub release)
  → vite-v* / postcss-v* tags → release-plugins.yml (builds + publishes the plugin to npm)
```

## Adding a changeset to your PR

Every PR that affects a published package should include a changeset file. After making your code changes:

```sh
pnpm changeset
```

This prompts you to:
1. **Select affected packages.** Pick only the packages your change touches (e.g. `@nemcss/postcss`, `nemcss`, `nemcss-vscode`).
2. **Choose bump type.** `patch` (bug fix), `minor` (new feature), or `major` (breaking change).
3. **Write a summary.** A short description that will appear in the changelog.

This creates a `.changeset/<random-name>.md` file. Commit it alongside your code changes.

### Which packages to select

- **nemcss** is the CLI wrapper package (`packages/nemcss`). Bumping this also bumps the Rust workspace version in `Cargo.toml` and the 5 platform CLI packages.
- **@nemcss/napi** is the N-API bindings (`crates/napi`). Bumping this auto-bumps `@nemcss/vite` and `@nemcss/postcss` since they depend on it.
- **@nemcss/vite** / **@nemcss/postcss** are the framework plugins.
- **nemcss-vscode** is the VS Code extension. Bumping this creates a separate `editor-v*` tag and triggers the editor release workflow.

The 5 platform CLI packages (`@nemcss/cli-darwin-arm64`, etc.) appear in the prompt but should never be selected — their versions are synced automatically by `scripts/version.mjs`.

### Rust crate → package mapping

Since Rust crates aren't published to crates.io, select whichever packages ship the affected binary:

| Crate changed | Select these packages |
|---|---|
| `crates/engine`, `crates/config`, `crates/extractor` | `nemcss` + `@nemcss/napi` (+ `nemcss-vscode` if LSP uses that code path) |
| `crates/cli` | `nemcss` |
| `crates/lsp` | `nemcss-vscode` |
| `crates/napi` | `@nemcss/napi` |

### When no changeset is needed

- Documentation-only changes
- CI/tooling changes that don't affect published packages
- Changes to dev dependencies

Run `pnpm changeset --empty` to explicitly signal "no release needed" (silences the bot warning).

## What happens after your PR merges

1. The `release.yml` workflow runs on every push to `main`
2. `changesets/action` detects pending changeset files and creates (or updates) a **"Version Packages"** PR
3. That PR contains version bumps, changelog entries, and the synced Cargo.toml version

## Cutting a release

Merge the "Version Packages" PR. This triggers:

1. `scripts/release.mjs` runs and creates git tags for any new versions
2. Tag pushes trigger the build workflows:
   - `v0.5.0` → `release-core.yml`. Builds CLI + LSP + NAPI for all 5 platforms, publishes `nemcss`, `@nemcss/napi`, and the platform CLI packages to npm, creates a GitHub release with binaries.
   - `editor-v0.5.0` → `release-editor.yml`. Builds LSP, packages VSIX for all 5 platforms, publishes to VS Code Marketplace, creates a GitHub release with VSIX files.
   - `vite-v0.3.0` / `postcss-v0.3.0` → `release-plugins.yml`. Builds and publishes the plugin to npm. These tags are pushed whenever the plugin version changed, including alongside a core release — `release-plugins.yml` is the only workflow that publishes the plugins.

## Changeset bot (optional)

Install the [Changesets Bot GitHub App](https://github.com/apps/changeset-bot) on the repository. It comments on PRs indicating whether a changeset is included. This is optional. The release workflow works without it.

## Version alignment

The `scripts/version.mjs` hook ensures these stay in sync:
- `packages/nemcss/package.json` version = `Cargo.toml` workspace version = all 5 `npm/@nemcss/cli-*` versions

The VS Code extension (`editors/vscode`) is versioned independently.

## Troubleshooting

**"Version Packages" PR not appearing?**
Check that the `RELEASE_PLEASE_TOKEN` secret (a PAT with `contents: write` and `pull-requests: write`) is set. The default `GITHUB_TOKEN` can't trigger downstream workflows, so we use a PAT.

**Tags not triggering build workflows?**
The PAT used in `release.yml` must have permission to push tags. Tag pushes from `GITHUB_TOKEN` don't trigger `on: push: tags` workflows.

**Cargo.toml version out of sync?**
Run `node scripts/version.mjs` manually. It reads from `packages/nemcss/package.json` and patches everything.
