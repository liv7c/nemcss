/**
 * Post-version hook: syncs Cargo.toml workspace version and platform CLI
 * package versions to match the nemcss npm package version.
 *
 * Run via: pnpm version (after changeset version)
 */

import { readFileSync, writeFileSync } from 'node:fs';

const nemcssPkg = JSON.parse(readFileSync('packages/nemcss/package.json', 'utf8'));
const version = nemcssPkg.version;

// 1. Patch Cargo.toml workspace version
const cargoToml = readFileSync('Cargo.toml', 'utf8');
const updatedCargo = cargoToml.replace(
  /^version = ".*"/m,
  `version = "${version}"`,
);
writeFileSync('Cargo.toml', updatedCargo);
console.log(`Cargo.toml workspace version → ${version}`);

// 2. Sync platform CLI package versions
const platforms = [
  'npm/@nemcss/cli-darwin-arm64',
  'npm/@nemcss/cli-darwin-x64',
  'npm/@nemcss/cli-linux-x64',
  'npm/@nemcss/cli-linux-arm64',
  'npm/@nemcss/cli-win32-x64',
];

for (const dir of platforms) {
  const path = `${dir}/package.json`;
  const pkg = JSON.parse(readFileSync(path, 'utf8'));
  pkg.version = version;
  writeFileSync(path, JSON.stringify(pkg, null, 2) + '\n');
}
console.log(`Platform CLI packages → ${version}`);
