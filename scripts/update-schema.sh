#!/usr/bin/env bash

# make script fail directly if any command fails
set -euo pipefail
cd "$(dirname "$0")/.."

mkdir -p editors/vscode/schemas docs/public/schema

cargo run -q -p cli --bin nemcss -- schema \
  | tee editors/vscode/schemas/nemcss.config.schema.json \
  > docs/public/schema/nemcss.config.schema.json