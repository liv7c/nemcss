---
"@nemcss/cli-darwin-arm64": patch
"@nemcss/cli-darwin-x64": patch
"@nemcss/cli-linux-arm64": patch
"@nemcss/cli-linux-x64": patch
"@nemcss/cli-win32-x64": patch
"nemcss": patch
---

Make watch mode more permissive. It was shutting down if the content list in nemcss config contained any glob for a folder not on disk yet.
