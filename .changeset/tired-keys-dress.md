---
"nemcss-vscode": patch
"@nemcss/cli-darwin-arm64": patch
"@nemcss/cli-darwin-x64": patch
"@nemcss/cli-linux-arm64": patch
"@nemcss/cli-linux-x64": patch
"@nemcss/cli-win32-x64": patch
"nemcss": patch
---

We now normalize the content file paths before adding them to the glob. Previously, if a user wrote paths like "./index.html" in nemcss config content section, it caused issues with the CLI watcher. By normalizing the paths before adding them to the glob set, we now match them correctly.
