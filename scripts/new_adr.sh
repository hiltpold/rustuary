#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 1 ]; then
  echo "usage: $0 \"ADR title\"" >&2
  exit 1
fi

title="$*"
slug=$(echo "$title" | tr '[:upper:]' '[:lower:]' | tr -cs 'a-z0-9' '-' | sed 's/^-//;s/-$//')
next=$(find docs/adr -maxdepth 1 -name '[0-9][0-9][0-9][0-9]-*.md' | sed 's#.*/##' | cut -d- -f1 | sort | tail -n1)
if [ -z "$next" ]; then n=1; else n=$((10#$next + 1)); fi
num=$(printf '%04d' "$n")
file="docs/adr/${num}-${slug}.md"
cat > "$file" <<EOF
# ADR ${num}: ${title}

Date: $(date +%Y-%m-%d)

## Status

Proposed

## Context


## Decision


## Consequences


EOF

echo "$file"
