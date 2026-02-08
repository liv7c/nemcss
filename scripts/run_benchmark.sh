#!/usr/bin/env bash

# Script to run benchmarks for a specific crate and save results
# Usage: ./scripts/run_benchmark.sh <crate_name> [description]

set -e

CRATE_NAME="${1}"
DESCRIPTION="${2:-benchmark}"

if [ -z "$CRATE_NAME" ]; then
    echo "Error: Crate name required"
    echo "Usage: $0 <crate_name> [description]"
    echo "Example: $0 engine baseline"
    exit 1
fi


# Create reports directory structure
REPORTS_DIR="benchmark-reports/${CRATE_NAME}"
mkdir -p "$REPORTS_DIR"

# Generate timestamp and filename
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
FILENAME="${REPORTS_DIR}/${TIMESTAMP}_${DESCRIPTION}.md"

# Get git metadata
GIT_COMMIT=$(git rev-parse --short HEAD 2> /dev/null || echo "unknown")
GIT_BRANCH=$(git branch --show-current 2> /dev/null || echo "unknown")
GIT_DIRTY=$(git diff --quiet 2> /dev/null || echo " (dirty)")

# Create report header in markdown
cat > "$FILENAME" << EOF
# Benchmark Report: ${CRATE_NAME}

**Date:** $(date +"%Y-%m-%d %H:%M:%S %Z")
**Git Branch:** \`${GIT_BRANCH}\`
**Git Commit:** \`${GIT_COMMIT}${GIT_DIRTY}\`
**Description:** ${DESCRIPTION}

## Environment

| Property | Value |
|----------|-------|
| Platform | $(uname -s) |
| Machine | $(uname -m) |
| OS Version | $(uname -r) |
| Rust Version | $(rustc --version) |

## Benchmark Results

EOF

# Run benchmark and filter output
echo "Running benchmarks for ${CRATE_NAME}..."

# Run and capture all output
BENCH_OUTPUT=$(cargo bench -p "$CRATE_NAME" 2>&1)

# Save raw benchmark output to a temp variable
RAW_BENCH=$(echo "$BENCH_OUTPUT" | sed -n '/Timer precision/,$p')

# Create summary table from benchmark output
cat >> "$FILENAME" << 'EOF'
| Benchmark | Fastest | Median | Mean | Slowest |
|-----------|---------|--------|------|---------|
EOF

# Parse benchmark lines and create table rows (skip parent benchmarks without times)
echo "$BENCH_OUTPUT" | awk '
/^[├╰]─ [a-z_]/ && $3 ~ /^[0-9]/ {
    # Extract benchmark name (column 2)
    name = $2
    # Extract times - correct columns:
    # $3 $4 = fastest (e.g., "552 µs")
    # $6 $7 = slowest
    # $9 $10 = median
    # $12 $13 = mean
    fastest = $3 " " $4
    slowest = $6 " " $7
    median = $9 " " $10
    mean = $12 " " $13
    printf "| %s | %s | %s | %s | %s |\n", name, fastest, median, mean, slowest
}' >> "$FILENAME"

# Add collapsible section with full raw output
cat >> "$FILENAME" << EOF

<details>
<summary>Full Benchmark Details (click to expand)</summary>

\`\`\`
$RAW_BENCH
\`\`\`

</details>
EOF

# Extract key metrics for summary (median time only)
echo "Extracting summary..."

# Dynamically extract all benchmarks and their median times
SUMMARY_ROWS=$(echo "$BENCH_OUTPUT" | awk '
/^[├╰]─ [a-z_]/ && $3 ~ /^[0-9]/ {
    name = $2
    median = $9 " " $10
    printf "| %s | %s |\n", name, median
}')

# Add summary section as markdown table
cat >> "$FILENAME" << EOF

## Summary

| Benchmark | Median Time |
|-----------|-------------|
EOF

# Append dynamic summary rows
echo "$SUMMARY_ROWS" >> "$FILENAME"

echo ""
echo "✔ Benchmark complete!"
echo ""
echo "📊 Summary:"
echo "$SUMMARY_ROWS" | sed 's/^|/  /'
echo ""
echo "📄 Full report: $FILENAME"
