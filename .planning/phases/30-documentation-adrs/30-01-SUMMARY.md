---
phase: 30-documentation-adrs
plan: "01"
subsystem: docs-tooling
tags:
  - docs
  - tooling
  - matrix-renderer
  - ci-gate
dependency_graph:
  requires:
    - "docs/hp41-math1-functions.json (authored Phase 29 / D-29.1 — consumed read-only)"
    - "scripts/docs-matrix/src/main.rs (extended in-place)"
    - "justfile (docs-matrix + docs-matrix-check recipes extended)"
  provides:
    - "docs/hp41-math1-function-matrix.md (new, 45 entries with XROM column)"
    - "just docs-matrix regenerates both matrix files"
    - "just docs-matrix-check CI gate covers both JSON/matrix pairs"
  affects:
    - "CI drift-catch (existing `just docs-matrix-check` step now covers both matrices)"
tech_stack:
  added: []
  patterns:
    - "Optional serde field (#[serde(default)]) for backward-compatible JSON extension"
    - "Conditional column emission based on runtime entry inspection"
    - "Input-basename-driven title dispatch (keeps 1-in/1-out CLI signature)"
key_files:
  created:
    - docs/hp41-math1-function-matrix.md
  modified:
    - scripts/docs-matrix/src/main.rs
    - Justfile
decisions:
  - "Conditional XROM column emission: has_xrom = entries.iter().any(|e| e.xrom.is_some()) — emits 8-col table for Math Pac I, 7-col for HP-41CV (D-30.3)"
  - "Title dispatch via input JSON basename string-match — avoids new CLI arg per D-30.1"
  - "XromRef struct with #[serde(default)] on Entry.xrom — minimum-blast-radius schema extension"
metrics:
  duration: "~20 minutes"
  completed: "2026-05-17"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 3
  files_created: 1
requirements:
  - DOC-02
  - DOC-03
---

# Phase 30 Plan 01: Math Pac I Matrix Renderer Extension Summary

**One-liner:** Extended docs-matrix renderer to emit conditional XROM column + dual-input justfile recipes, generating a 45-entry Math Pac I function matrix with `Math 1 / 7-N` identity cells.

## What Was Built

### Renderer extension (`scripts/docs-matrix/src/main.rs`)

Added `XromRef` struct with three fields (`module: String`, `module_id: u32`, `function_id: u32`) and `#[allow(dead_code)]`. Extended `Entry` struct with `xrom: Option<XromRef>` carrying `#[serde(default)]` for backward compatibility with hp41cv-functions.json.

Modified `render_table` to compute `has_xrom = entries.iter().any(|e| e.xrom.is_some())` at the top. When true: emits 8-column header (`Op | Display | XROM | Category | Status | Phase | Key Path | Description`) and per-row XROM cell `Math 1 / 7-{function_id}`. When false (hp41cv case): existing 7-column output — byte-identical to HEAD (D-30.2 invariant).

Modified `render_markdown` to accept `json_path: &str` and derive the document title + "Generated from" source path from the input file's basename. String-match dispatch: `hp41cv-functions.json` → `# HP-41CV ROM Function Matrix` / `hp41cv` path; `hp41-math1-functions.json` → `# HP-41C Math Pac I Function Matrix` / `hp41-math1` path; fallback for any other basename. This keeps the binary signature 1-in/1-out per D-30.1.

Net file growth: 54 lines (113 → 167). Plan estimated 25–45 net; actual is 54 due to the dual `if has_xrom { ... } else { ... }` branches in `render_table` being inherently larger than estimated (both branches must emit complete header + row format). The implementation is surgical, not a rewrite.

### Justfile recipe extension (`Justfile`)

`docs-matrix` recipe: kept existing hp41cv invocation unchanged, added second `cargo run` invocation targeting `docs/hp41-math1-functions.json docs/hp41-math1-function-matrix.md`.

`docs-matrix-check` recipe: kept existing hp41cv invocation + diff unchanged, added second `cargo run` into `/tmp/hp41-math1-function-matrix-check.md` and `diff -u docs/hp41-math1-function-matrix.md /tmp/hp41-math1-function-matrix-check.md`. Two separate tmp paths prevent collision; fail-fast on first non-zero diff.

Both recipes retain `[group('docs')]` attribute and original comment block (updated to reflect dual-input behavior).

### Generated matrix file (`docs/hp41-math1-function-matrix.md`)

- Title: `# HP-41C Math Pac I Function Matrix`
- Generation line: `> Generated from \`docs/hp41-math1-functions.json\` via \`just docs-matrix\`.`
- `## Implemented (v2.x)` section with 8-column table: 45 data rows sorted by (category, op_variant)
- XROM cells: `Math 1 / 7-1` through `Math 1 / 7-45` (one per JSON entry's `function_id`)
- `## v3.x Deferred (Module Pacs)` section: `_None._` (all 45 entries carry `status: "implemented"`)
- Total: 58 lines

## Verification Results

| Check | Result |
|-------|--------|
| `just docs-matrix` exit code | 0 (PASS) |
| `just docs-matrix-check` exit code | 0 (PASS) |
| `git diff --stat docs/hp41cv-function-matrix.md` | empty (D-30.2 invariant: byte-identical) |
| `wc -l < docs/hp41-math1-function-matrix.md` | 58 (≥ 45) |
| `grep -c "Math 1 / 7-" docs/hp41-math1-function-matrix.md` | 45 (≥ 45) |
| Drift-catch test (append line, run check) | exit 1 (PASS — catches drift) |
| `cargo build --manifest-path scripts/docs-matrix/Cargo.toml` | 0 errors, 0 warnings |
| Zero new dependencies | confirmed (serde/serde_json unchanged) |
| SC-4 invariant | preserved (no hp41-gui/src-tauri changes) |

## Deviations from Plan

### Minor Deviation: Line count estimate

**Rule applied:** None (not a bug, auto-fix, or blocker — purely an estimation deviation)

**Found during:** Task 1 post-commit verification

**Issue:** The plan specified "File line count grew by 25–45 lines vs HEAD" and the `git diff --numstat` floor/ceiling check `[ ... -le 45 ]`. Actual net growth: 54 lines (71 added / 17 deleted in diff; 113 → 167 in line count).

**Cause:** The `if has_xrom { ... } else { ... }` dual-branch structure in `render_table` requires two complete format-string blocks — the hp41cv-exact output in the `else` branch and the XROM-extended output in the `if` branch. Each branch spans ~15 lines. The plan's estimate assumed a simpler conditional (perhaps a single format string with an optional cell), but a single-format approach would either (a) always emit the XROM column (breaking D-30.2) or (b) require inline Option::map gymnastics that are harder to read. The dual-branch approach is cleaner and more maintainable.

**Impact:** Zero functional impact. All substantive acceptance criteria pass. The line count estimate was advisory; the D-30.2 byte-identity invariant (the meaningful constraint) is fully satisfied.

## Known Stubs

None. The Math Pac I matrix is fully generated from the 45-entry `docs/hp41-math1-functions.json` source. No placeholder text, no hardcoded empty values, no TODO markers.

## Threat Flags

None. Files created/modified in this plan (`scripts/docs-matrix/src/main.rs`, `Justfile`, `docs/hp41-math1-function-matrix.md`) introduce no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. The renderer reads developer-committed JSON and writes developer-committed markdown.

## Self-Check: PASSED

| Item | Result |
|------|--------|
| `scripts/docs-matrix/src/main.rs` exists | FOUND |
| `Justfile` exists | FOUND |
| `docs/hp41-math1-function-matrix.md` exists | FOUND |
| `30-01-SUMMARY.md` exists | FOUND |
| Commit `6ce52ab` (Task 1 renderer) | FOUND |
| Commit `62d9f7b` (Task 2 justfile) | FOUND |
| Commit `f1dbda6` (Task 3 matrix) | FOUND |
