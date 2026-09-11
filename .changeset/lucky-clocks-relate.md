---
"@nemcss/cli-darwin-arm64": patch
"@nemcss/cli-darwin-x64": patch
"@nemcss/cli-linux-arm64": patch
"@nemcss/cli-linux-x64": patch
"@nemcss/cli-win32-x64": patch
"@nemcss/napi": patch
"nemcss": patch
---

Fix `new-token-file` generating invalid CSS for fractional scale values. Names like `0.5` are now written as `0_5` so the custom property (e.g. `--spacing-0_5`) and utility class (`p-0_5`) are valid identifiers. Token files with token names that would produce invalid CSS now fail the build too.
