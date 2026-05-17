---
phase: 29-cli-integration
plan: 02
subsystem: hp41-cli
tags: [xrom, math-pac-i, help-data, key-ref, overlay, accessor-migration, cli-03, cli-04]
dependency_graph:
  requires: [29-01-math-pac-i-json-pipeline]
  provides: [help_overlay_rows-math1, key_ref_entries-math1, cli-03-verified, cli-04-closed]
  affects: [hp41-cli/src/help_data.rs, hp41-cli/src/keys.rs, hp41-cli/tests/phase29_key_ref_includes_math1.rs, hp41-cli/tests/phase25_help_data.rs]
tech_stack:
  added: []
  patterns: [merged-accessor-migration, tdd-red-green, grep-audit-verification]
key_files:
  created:
    - hp41-cli/tests/phase29_key_ref_includes_math1.rs
  modified:
    - hp41-cli/src/help_data.rs
    - hp41-cli/src/keys.rs
    - hp41-cli/tests/phase25_help_data.rs
decisions:
  - "Fixed phase25_help_data::help_overlay_rows_contain_category_headers to use help_entries_all() for expected category count — Rule 1 bug (test compared merged output against narrow-pool count after migration)"
  - "CLI-03 verified as fully satisfied by Phase 28 ship — no new op_display_name arms needed; cargo check x2 is the gate"
metrics:
  duration: "6m"
  completed: "2026-05-17"
  tasks_completed: 2
  files_modified: 4
---

# Phase 29 Plan 02: Help Overlay + Key-Ref Migration to help_entries_all() Summary

**One-liner:** Single-line accessor migration in two production consumers (help_overlay_rows + key_ref_entries) to help_entries_all(), verified by 3 new Wave-0 tests and CLI-03 grep audit confirming Phase 28 ship completeness.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Wave-0 tests + migrate key_ref_entries() | 481faa4 | hp41-cli/src/keys.rs, hp41-cli/tests/phase29_key_ref_includes_math1.rs |
| 2 | Migrate help_overlay_rows() + verify CLI-03 | 33bac8f | hp41-cli/src/help_data.rs, hp41-cli/tests/phase25_help_data.rs |

## Success Criteria Verification

- **CLI-03:** `op_display_name` in BOTH `hp41-cli/src/prgm_display.rs` AND `hp41-gui/src-tauri/src/prgm_display.rs` covers all Math Pac I `Op` variants exhaustively — proven by:
  - `cargo check -p hp41-cli`: exit 0
  - `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml`: exit 0
  - Grep audit: 44 Math Pac I variant arms in CLI, 44 in GUI (identical count, D-25.6 parity)
  - No `_ =>` catch-all in either `op_display_name` (FN-CLI-04)

- **CLI-04:** `key_ref_entries()` now reads from `help_entries_all()` — Math Pac I rows surface in right-panel discoverability listing — proven by:
  - `phase29_key_ref_includes_math1::key_ref_entries_includes_math1_sinh`: PASS
  - `phase29_key_ref_includes_math1::key_ref_entries_includes_math1_matrix`: PASS
  - `phase29_key_ref_includes_math1::key_ref_entries_preserves_v22_entries`: PASS

- **`?` overlay migration:** `help_overlay_rows()` now reads from `help_entries_all()` — Math Pac I categories appear alongside v2.2 categories in the overlay (CLI-04 extension)

- **Lint:** `just lint` (cargo clippy --all-targets --all-features -- -D warnings) exits 0 — dead-code warnings from 29-01 (`MATH1_HELP_ENTRIES` etc.) resolved by these consumers

- **Tests:** `just test` — 288 passed, 0 failed (285 from 29-01 + 3 new Wave-0 tests)

## CLI-03 Verification Section

### Grep Audit Command

```
grep -c "Op::\(Sinh\|Cosh\|Tanh\|Asinh\|Acosh\|Atanh\|CPlus\|CMinus\|CTimes\|CDiv\|Real\|Magz\|Cinv\|ZpowN\|Zpow1N\|ExpZ\|LnZ\|SinZ\|CosZ\|TanZ\|LogZ\|ZpowW\|ApowZ\|PolyWorkflow\|Roots\|MatrixWorkflow\|MatSize\|MatVmat\|MatEdit\|MatDet\|MatInv\|MatSimeq\|MatVcol\|Integ\|Solve\|Sol\|Difeq\|Four\|TriSss\|TriAsa\|TriSaa\|TriSas\|TriSsa\|Trans2d\|Trans3d\)" hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs
```

### Audit Results

| File | Match Count | Notes |
|------|-------------|-------|
| `hp41-cli/src/prgm_display.rs` | 44 | 43 Op arms + 1 comment line; all Phase 28 variants present |
| `hp41-gui/src-tauri/src/prgm_display.rs` | 44 | Identical count — D-25.6 parity confirmed |

Specific spot-checks (all pass):

| Op Variant | CLI arm | GUI arm |
|------------|---------|---------|
| `Op::Sinh` | `Op::Sinh => "SINH".to_string()` (line 239) | `Op::Sinh => "SINH".to_string()` (line 260) |
| `Op::MatrixWorkflow` | `Op::MatrixWorkflow => "MATRIX".to_string()` (line 268) | `Op::MatrixWorkflow => "MATRIX".to_string()` (line 289) |
| `Op::Integ` | `Op::Integ => "INTG".to_string()` (line 277) | `Op::Integ => "INTG".to_string()` (line 298) |

### Compile-Time Gate

- `cargo check -p hp41-cli`: **exit 0** (FN-CLI-04: exhaustive match enforced by Rust compiler)
- `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml`: **exit 0**

CLI-03 is satisfied by the Phase 28 ship. Plan 29-02's role is **verification only** — no new `op_display_name` arms were added.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed stale expected-category count in phase25_help_data test**

- **Found during:** Task 2 — after migrating `help_overlay_rows()` to `help_entries_all()`, the existing `help_overlay_rows_contain_category_headers` test in `phase25_help_data.rs` compared the merged header count against `help_entries()` (v2.2 only) instead of `help_entries_all()`. The test failed because `help_overlay_rows()` now produces more category headers (v2.2 + Math1) than the narrow-pool count.
- **Issue:** The test at line 104 used `help_entries()` to compute `distinct_categories` while `help_overlay_rows()` now uses `help_entries_all()` as its source. The comparison was a mismatch (13 categories from v2.2 vs 19 from merged pool).
- **Fix:** Updated `phase25_help_data.rs` to import `help_entries_all` and derive `distinct_categories` from `help_entries_all()` instead of `help_entries()`. The test now correctly verifies that one header per category is produced over the merged pool.
- **Files modified:** `hp41-cli/tests/phase25_help_data.rs`
- **Commit:** 33bac8f (bundled with Task 2)

None of the architectural decisions (accessor shape, JSON pipeline, D-29.x decisions) were changed.

## Test Coverage

| Test File | Count | Description |
|-----------|-------|-------------|
| `phase29_key_ref_includes_math1.rs` | 3 (new) | Wave-0: Math Pac I rows surface in right-panel + v2.2 regression guard |
| `phase25_help_data.rs` | 7 (1 fixed) | help_overlay_rows category headers test updated for merged pool |

## Decisions Made

1. **Rule 1 bug fix applied inline to Task 2:** The `help_overlay_rows_contain_category_headers` test was comparing merged output against narrow-pool expected count. Fixed by updating the test to derive the expected count from `help_entries_all()`. This is a correctness fix — the test was testing the wrong invariant after the migration.

2. **CLI-03 is Phase 28 ship, Plan 02 verifies:** No new `op_display_name` arms were added. The `cargo check` x2 exit codes are the compile-time proof per FN-CLI-04.

## Known Stubs

None. Both migrations are complete. The `help_entries_all()` accessor returns live data from two JSON files. No placeholder content.

## Threat Flags

None. No new network endpoints, auth paths, file I/O changes, or schema changes. The change is purely intra-module accessor migration within `hp41-cli`.

## Self-Check: PASSED

Files created/modified:

- `hp41-cli/tests/phase29_key_ref_includes_math1.rs`: exists, 3 tests
- `hp41-cli/src/keys.rs`: `help_entries_all()` at line 403, old `help_entries()` call replaced
- `hp41-cli/src/help_data.rs`: `help_entries_all().collect()` in `help_overlay_rows` at line 156
- `hp41-cli/tests/phase25_help_data.rs`: `help_entries_all` imported, category count fixed

Commits:

- 481faa4: Task 1 — migrate key_ref_entries() + 3 new tests
- 33bac8f: Task 2 — migrate help_overlay_rows() + fix stale test

Quality gates:

- `just test`: 288 passed, 0 failed
- `just lint`: clean (no warnings)
- `cargo check -p hp41-cli`: exit 0
- `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml`: exit 0
