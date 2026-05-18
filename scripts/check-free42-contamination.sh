#!/usr/bin/env bash
# scripts/check-free42-contamination.sh
# CI gate: hp41-core/src/ops/math1/ must contain no distinctive Free42
# identifiers (Intel BID library, decNumber, Free42 internals, copyright
# markers) outside the allowlisted disclaim header on each math1 file.
#
# Phase 32 Plan 32-03: D-32.7 (12-symbol policy) + D-32.8 (dual just-ci / ci.yml invocation).
# Pitfall 19: Free42 GPL contamination via copy-paste from the Free42 reference impl.
set -euo pipefail

MATH1_DIR="hp41-core/src/ops/math1"
DISCLAIM_LINE='Free42 source consulted only as sanity-check oracle'

# WR-01: explicit directory existence check — if MATH1_DIR is missing (refactor moved
# files, script invoked from wrong cwd, future module split), the grep pipeline would
# silently exit 0 ("no contamination") because a missing-path grep returns non-zero,
# causing the pipeline failure to evaluate as "no matches". That would neutralise the
# Pitfall 19 / D-32.7 license guard. Exit 2 on missing directory is unambiguous.
if [[ ! -d "$MATH1_DIR" ]]; then
    echo "FAIL: $MATH1_DIR does not exist — license guard cannot run." >&2
    exit 2
fi

# D-32.7: 12 distinctive symbols verified zero false-positives against current source.
# The bare string "Free42" is deliberately NOT in this pattern — 122 legitimate
# "Free42 v3.0.5: <value>" cross-check references exist across the codebase
# (per RESEARCH.md). The 12 symbols below are tight enough to never match those.
PATTERN='phloat|Phloat|bid128_|decNumber|decContext|vartype|arg_struct|prgm_lines|bcd_t|Thomas Okken|AGPL|GNU General Public License'

if matches=$(grep -rn -E "$PATTERN" "$MATH1_DIR" | grep -v "$DISCLAIM_LINE"); then
    echo "FAIL: Free42 contamination detected in $MATH1_DIR:"
    echo "$matches"
    exit 1
fi

echo "OK: no Free42 contamination detected in $MATH1_DIR/"
exit 0
