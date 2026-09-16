---
"@nemcss/napi": minor
"@nemcss/postcss": minor
"@nemcss/vite": minor
"nemcss-vscode": minor
---

Improve error handling and DX across the LSP and plugins. The LSP no longer breaks on a bad token file or config, and shows diagnostics in the editor. The Vite and PostCSS now fail the build with a clear message and help text instead of silently producing broken CSS.
