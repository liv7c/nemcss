# Contributing to NemCSS

Thank you for your interest in contributing!

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/) >= 10

## Setup

```sh
git clone https://github.com/liv7c/nemcss.git
cd nemcss
pnpm install
```

## Running tests

```sh
# Rust tests
cargo test --workspace

# Check formatting
cargo fmt --all --check

# Linting
cargo clippy --workspace --all-features --tests --benches -- -D warnings
```

## Submitting a pull request

1. Fork the repository and create a branch from `main`
2. Make your changes
3. Make sure tests and linting pass
4. If your change affects a published package, add a changeset: `pnpm changeset`
5. Open a pull request with a clear description of what changed and why

This project uses [conventional commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `chore:`, etc.).

## Releasing

See [internal/docs/release-flow.md](internal/docs/release-flow.md) for the full release process.
