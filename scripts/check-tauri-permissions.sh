#!/usr/bin/env bash
# scripts/check-tauri-permissions.sh
# CI gate: every command registered in generate_handler! must have a matching
# hp41-gui/src-tauri/permissions/<kebab-case>.toml permission file.
#
# Phase 31 Plan 31-02 (Wave 0, Task 1): authored per 31-RESEARCH.md §"Open Questions Q1"
# Threat T-31-W1-permission-coverage: without this gate, a future command added to
# generate_handler! without a TOML would silently be reachable without declared permissions.
set -euo pipefail

HANDLER_FILE="hp41-gui/src-tauri/src/lib.rs"
PERMS_DIR="hp41-gui/src-tauri/permissions"

# Extract all command names from the generate_handler! macro block.
# Pattern: looks for `commands::<name>` references (snake_case).
commands=$(grep -oE 'commands::[a-z_]+' "$HANDLER_FILE" | sed 's/commands:://' | sort -u)

missing=0
for cmd in $commands; do
    kebab=$(echo "$cmd" | sed 's/_/-/g')
    if [[ ! -f "$PERMS_DIR/$kebab.toml" ]]; then
        echo "MISSING: $PERMS_DIR/$kebab.toml  (for command: $cmd)"
        missing=$((missing + 1))
    fi
done

if [[ $missing -eq 0 ]]; then
    echo "OK: all $(echo "$commands" | wc -w | tr -d ' ') commands have permission TOMLs"
fi

exit $missing
