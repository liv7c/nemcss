#!/usr/bin/env node

const { spawnSync } = require("child_process");

const PLATFORMS = {
  "darwin-arm64": "@nemcss/cli-darwin-arm64",
  "darwin-x64": "@nemcss/cli-darwin-x64",
  "linux-x64": "@nemcss/cli-linux-x64",
  "linux-arm64": "@nemcss/cli-linux-arm64",
  "win32-x64": "@nemcss/cli-win32-x64",
};

const key = `${process.platform}-${process.arch}`;
const pkg = PLATFORMS[key];

if (!pkg) {
  console.error(`nemcss: unsupported platform: ${key}`);
  process.exit(1);
}

let binPath;
try {
  binPath = require.resolve(
    `${pkg}/bin/nemcss${process.platform === "win32" ? ".exe" : ""}`,
  );
} catch {
  console.error(
    `nemcss: platform package ${pkg} is not installed. Try reinstalling nemcss.`,
  );
  process.exit(1);
}

const result = spawnSync(binPath, process.argv.slice(2), { stdio: "inherit" });

if (result.error) {
  console.error(`nemcss: failed to run binary: ${result.error.message}`);
  process.exit(1);
}

if (result.signal) {
  process.kill(process.pid, result.signal);
}

process.exit(result.status ?? 1);
