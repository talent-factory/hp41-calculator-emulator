---
phase: 29-cli-integration
plan: 01
subsystem: hp41-cli + docs
tags: [xrom, math-pac-i, json-pipeline, help-data, resolver-chain, parity-tests]
dependency_graph:
  requires: [28-xrom-framework-math-pac-i-core-ops]
  provides: [xeq_by_name_local_resolve-math1-fallback, help_entries_math1, help_entries_all, hp41-math1-functions-json, math1-parity-tests]
  affects: [hp41-cli/src/keys.rs, hp41-cli/src/help_data.rs, hp41-cli/src/app.rs, docs/hp41-math1-functions.json]
tech_stack:
  added: []
  patterns: [OnceLock-json-pipeline, xrom-resolve-fallback, bidirectional-parity-tests]
key_files:
  created:
    - docs/hp41-math1-functions.json
    - hp41-cli/tests/phase29_help_data_math1.rs
  modified:
    - hp41-cli/src/help_data.rs
    - hp41-cli/src/keys.rs
    - hp41-cli/src/app.rs
    - hp41-cli/tests/function_matrix_parity.rs
    - hp41-cli/tests/phase25_xeq_by_name.rs
    - hp41-cli/tests/key_coverage.rs
decisions:
  - "Used 45 as the canonical Math Pac I Op variant count (plan said 47 but actual unique Op variants in MATH_1.ops dedup to 45: 52 total entries minus 7 ASCII alias entries)"
  - "Key paths use Unicode display names (C×, C÷, Z↑N, etc.) matching MATH_1.ops primary spellings; xrom_resolve accepts both Unicode and ASCII aliases"
  - "help_entries_all() returns impl Iterator<Item = &'static HelpEntry> chaining both pools"
metrics:
  duration: "10m"
  completed: "2026-05-17"
  tasks_completed: 3
  files_modified: 7
---

# Phase 29 Plan 01: Math Pac I CLI Pipeline + Resolver Extension Summary

**One-liner:** JSON canonical pipeline for 45 Math Pac I ops + xrom_resolve wired as final fallback in xeq_by_name_local_resolve, closing the third call site deferred by Phase 28.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Math Pac I JSON + help_entries_math1/all | e04808c | docs/hp41-math1-functions.json, help_data.rs, phase29_help_data_math1.rs |
| 2 | Widen xeq_by_name_local_resolve + ripple | 2d61ff3 | keys.rs, app.rs, phase25_xeq_by_name.rs, key_coverage.rs |
| 3 | Bidirectional Math1 parity tests | 6782e7e | function_matrix_parity.rs |

## Success Criteria Verification

- **CLI-01:** `xeq_by_name_local_resolve("SINH", 0b1) == Some(Op::Sinh)` and `xeq_by_name_local_resolve("SINH", 0b0) == None` — proven by `cli_resolver_matches_core_resolver` Phase 29 extension (10 positive + 10 negative cases)
- **CLI-02 (a):** second `OnceLock` loads `docs/hp41-math1-functions.json` via `help_entries_math1()`; merged accessor `help_entries_all()` chains both pools — proven by `phase29_help_data_math1` 10-test suite
- **CLI-02 (b):** bidirectional parity between `MATH_1.ops` ↔ JSON ↔ `Op::*` — proven by 3 new tests in `function_matrix_parity.rs` (total 7 tests, 4 existing + 3 new)
- **Compile-time exhaustiveness:** no `_ =>` arm in `xeq_by_name_local_resolve`; `xrom_resolve` is the explicit final fallthrough (FN-CLI-04 spirit)
- **Save-file backward compat:** no new `CalcState` fields; `HelpEntry::xrom` field is `#[serde(default)]`-guarded (v2.2 JSON parses unchanged)

## Deviations from Plan

### Auto-adjusted Count

**[Claude's Discretion] Corrected 47 → 45 unique Op variants**
- **Found during:** Task 1 planning / verification
- **Issue:** Plan said "47 unique op_variant rows" but MATH_1.ops has 52 entries with 7 ASCII aliases (C*, C/, Z^N, Z^1/N, E^Z, A^Z, Z^W) mapping to the same 7 Op variants as their Unicode primaries. Deduplication: 52 - 7 = 45 unique Op variants.
- **Fix:** Used 45 throughout (JSON entries, test assertions, MATH1_OP_VARIANT_NAMES length). Tests pass with 45; the plan's 47 count was a calculation error in the research phase.
- **Impact:** None — the physical MATH_1.ops table is authoritative; 45 is correct.

None of the plan's task scope, architectural choices, or D-29.x decisions were changed.

## Test Coverage

| Test File | Count | Description |
|-----------|-------|-------------|
| `phase29_help_data_math1.rs` | 10 | Smoke tests for help_entries_math1() and help_entries_all() |
| `function_matrix_parity.rs` | 7 (3 new) | Bidirectional JSON ↔ Op ↔ xrom_resolve parity |
| `phase25_xeq_by_name.rs` | 14 (ext.) | cli_resolver_matches_core_resolver extended with Math Pac I cases |

## Decisions Made

1. **Canonical Op count is 45** (not 47 as written in the plan): MATH_1.ops has 52 entries; 7 are ASCII aliases for Unicode-primary mnemonics. 52 - 7 = 45 unique Op variants.
2. **Unicode display names in JSON**: entries use primary Unicode names (C×, C÷, Z↑N, Z↑1/N, E↑Z, A↑Z, Z↑W) matching MATH_1.ops; `xrom_resolve` accepts both forms.
3. **`help_entries_all()` returns `impl Iterator`** (not `&'static [HelpEntry]`): chaining two static slices cannot produce a contiguous slice; the iterator approach is idiomatic and zero-allocation.

## Known Stubs

None. All 45 JSON entries have real descriptions and correct xrom blocks. The `help_entries_all()` accessor is wired to actual data. All test assertions operate on live data.

## Threat Flags

None. No new network endpoints, auth paths, file I/O changes at trust boundaries, or schema changes outside the additive `#[serde(default)] pub xrom: Option<XromEntry>` field.

## Self-Check: PASSED

- `docs/hp41-math1-functions.json`: exists, 45 entries, valid JSON
- `hp41-cli/src/help_data.rs`: XromEntry struct, MATH1_HELP_ENTRIES, help_entries_math1(), help_entries_all() — all present
- `hp41-cli/src/keys.rs`: signature widened, xrom_resolve wired as final fallback, no _ => None arm
- `hp41-cli/src/app.rs`: production call site updated to pass self.state.xrom_modules
- Commits e04808c, 2d61ff3, 6782e7e — all verified in git log
- `cargo test -p hp41-cli --tests`: 285 total tests passing, 0 failed
