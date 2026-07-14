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

const coreTag = `v${nemcssPkg.version}`;
const editorTag = `editor-v${vscodePkg.version}`;
const viteTag = `vite-v${vitePkg.version}`;
const postcssTag = `postcss-v${postcssPkg.version}`;

const tagsToCreate = [];

// Plugin tags are always pushed alongside core/editor tags:
// release-plugins.yml is the sole publisher for @nemcss/vite and
// @nemcss/postcss (npm trusted publishing allows one workflow per package).
for (const tag of [coreTag, editorTag, viteTag, postcssTag]) {
  if (!tagExists(tag)) {
    createTag(tag);
    tagsToCreate.push(tag);
  } else {
    console.log(`Tag ${tag} already exists, skipping`);
  }
}

// GitHub drops push events when more than 3 tags are pushed in a single
// `git push` command, silently skipping the tag-triggered release workflows.
// Push each tag separately so every one reliably fires its workflow.
if (tagsToCreate.length > 0) {
  for (const tag of tagsToCreate) {
    console.log(`Pushing tag: ${tag}`);
    execSync(`git push origin ${tag}`, { stdio: 'inherit' });
  }
} else {
  console.log('No new tags to push');
}
