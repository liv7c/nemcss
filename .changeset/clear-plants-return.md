---
"@nemcss/postcss": patch
"@nemcss/vite": patch
---

Fix a regression introduced by the upgrade to pnpm v11. With the new version, we need to specify a `files` property in the package json to include the dist files.
