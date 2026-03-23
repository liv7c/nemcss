/**
 * Post-publish hook: creates git tags for new versions and pushes them.
 * Tag pushes trigger the build/publish workflows.
 *
 * Run via: pnpm release (called by changesets/action publish command)
 */

import { readFileSync } from 'node:fs';
import { execSync } from 'node:child_process';

// Fetch remote tags so we can check against them
execSync('git fetch --tags', { stdio: 'inherit' });

function tagExists(tag) {
  try {
    execSync(`git rev-parse refs/tags/${tag}`, { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

function createTag(tag) {
  console.log(`Creating tag: ${tag}`);
  execSync(`git tag ${tag}`, { stdio: 'inherit' });
}

// Read versions
const nemcssPkg = JSON.parse(readFileSync('packages/nemcss/package.json', 'utf8'));
const vscodePkg = JSON.parse(readFileSync('editors/vscode/package.json', 'utf8'));
const vitePkg = JSON.parse(readFileSync('packages/vite-plugin-nemcss/package.json', 'utf8'));
const postcssPkg = JSON.parse(readFileSync('packages/postcss-plugin-nemcss/package.json', 'utf8'));

const candidates = [
  { tag: `v${nemcssPkg.version}` },
  { tag: `editor-v${vscodePkg.version}` },
  { tag: `vite-v${vitePkg.version}` },
  { tag: `postcss-v${postcssPkg.version}` },
];

const tagsToCreate = [];

for (const { tag } of candidates) {
  if (!tagExists(tag)) {
    createTag(tag);
    tagsToCreate.push(tag);
  } else {
    console.log(`Tag ${tag} already exists, skipping`);
  }
}

if (tagsToCreate.length > 0) {
  console.log(`Pushing tags: ${tagsToCreate.join(', ')}`);
  execSync(`git push origin ${tagsToCreate.join(' ')}`, { stdio: 'inherit' });
} else {
  console.log('No new tags to push');
}
