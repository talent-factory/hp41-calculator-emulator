---
phase: 22-program-control-and-memory-ops
plan: 04-catalog-and-asn
subsystem: core
tags: [hp41, rust, catalog, asn, key-assignments, print-buffer, serde-default]

# Dependency graph
requires:
  - phase: 22-03-memory-ops
    provides: "Op enum + dispatch + execute_op + both prgm_display.rs copies pattern landed for 11 prior variants (Stop/Pse/GtoInd/XeqInd + Clp/Del/Ins + Size/Cla/Clst/Pack); 4-place rule and SC-4 invariant established; coverage baseline at 90.16% lines"
  - phase: 11-print-emulation
    provides: "state.print_buffer drain channel + 24-char width convention (op_prx/op_pra/op_prstk)"
  - phase: 05-persistence-and-ux
    provides: "BTreeMap<char, String> precedent (key_assignments) — Phase 22 adds a sibling BTreeMap<u8, String> for hardware-keyed assignments"
  - phase: 12-synthetic-programming
    provides: "#[serde(default)] precedent for last_key_code, reg_m/n/o; row×10+col 1-indexed key-code encoding"
  - phase: 21-flags-display-sound
    provides: "#[serde(default)] precedent for flags / display_override / event_buffer; serde-default backward-compat sentinel test pattern (phase21_flags.rs:27–37)"
provides:
  - "CalcState.assignments: BTreeMap<u8, String> field with #[serde(default)] — coexists with Phase 5 key_assignments (char-keyed); Phase 25/26 reconciles"
  - "Op::Catalog(u8) per AMENDED D-22.16 / OQ-1 Option B: CAT 1 = programs (LBL listing with step counts); CAT 2/3/4 = single 24-char NOT AVAILABLE payload; n==0 or n>=5 → InvalidOp"
  - "Op::Asn { name: String, key_code: u8 } per AMENDED D-22.18 / OQ-3 Option A: empty name removes; non-empty inserts/overwrites; late-binding resolution deferred to Phase 25/26 (D-22.19)"
  - "pub fn op_catalog, pub fn op_asn in hp41-core/src/ops/program.rs"
  - "prgm_display.rs (CLI + GUI) display strings: CATALOG n / ASN \"name\" nn"
  - "Phase 22 COMPLETE — 13 ROM ops + 1 new CalcState field landed across 4 plans"
affects:
  - phase: 25-cli-keyboard-wiring
    note: "Phase 25 will wire CLI CATALOG keystroke + 1-digit numeric prompt, and ASN 2-step modal (label prompt → key prompt). Stub-error pattern in v2.1 GUI key_map for `catalog`/`asn` ids can now resolve to real Op variants."
  - phase: 26-gui-key-map
    note: "Phase 26 GUI key_map can resolve the v2.1 stubbed `catalog` and `asn` ids to real Op::Catalog / Op::Asn variants and surface CATALOG output via the existing print_buffer drain channel."
  - phase: 27-test-hardening
    note: "Future proptest opportunity: ASN insert/remove invariant — for any sequence of (key_code, name) pairs, the final assignments map equals the high-water-mark of the operations after de-duplicating by key with empty-name removing. Low-value at v2.2; deferred."

# Tech tracking
tech-stack:
  added: []  # purely additive on hp41-core; no new dependencies
  patterns:
    - "Hardware-faithful structured-output pattern for CATALOG: header + payload + footer with uniform 24-char width discipline (print_buffer drain channel from Phase 11)"
    - "Long-label truncation idiom: `name.chars().take(9).collect::<String>()` keeps total LBL listing line width ≤24 chars without truncation marker"
    - "Struct-variant JSON shape pinning sentinel test (Pitfall 9): explicit equality check against `{\"Asn\":{\"name\":...,\"key_code\":...}}` catches accidental shape drift from future serde-attribute additions"
    - "Empty-string-as-tombstone semantic for map-mutation ops (OQ-3): `ASN \"\" key_code` removes the assignment rather than inserting the empty string — mirrors hardware-faithful HP-41 ASN behavior"
    - "Defensive `_ => Err(InvalidOp)` arm after a guarded match block: replaces `unreachable!()` for clippy-clean zero-panic compliance even when the arm is unreachable in practice"

key-files:
  created:
    - "hp41-core/tests/phase22_catalog.rs — 10 integration tests (210 lines)"
    - "hp41-core/tests/phase22_asn.rs — 10 integration tests (234 lines)"
  modified:
    - "hp41-core/src/state.rs — new field `assignments: BTreeMap<u8, String>` + initializer in CalcState::new()"
    - "hp41-core/src/ops/mod.rs — 2 new Op variants (Catalog/Asn) + 2 dispatch arms"
    - "hp41-core/src/ops/program.rs — 2 new pub helpers (op_catalog, op_asn) + 2 execute_op arms"
    - "hp41-cli/src/prgm_display.rs — 2 display arms (CATALOG n / ASN \"name\" nn)"
    - "hp41-gui/src-tauri/src/prgm_display.rs — same 2 display arms (SC-4 duplication)"

key-decisions:
  - "CATALOG 1 long-label truncation to 9 chars (vs full name with overflow): preserves uniform 24-char line width. LBL listing format = `LBL ` (4) + name :9 (9) + `  ` (2) + steps :5 (5) = 20 chars, padded to 24. Names ≤9 chars render unchanged; longer names render their first 9 chars (no truncation marker). Verified by `test_catalog_long_label_truncated_to_9_chars` with a 13-char label."
  - "Defensive `_ => Err(HpError::InvalidOp)` arm in op_catalog's inner match (replaces `unreachable!()`): clippy is happy, the arm is guarded unreachable by the outer `n == 0 || n >= 5` early-return, and the InvalidOp fallback is the same error the outer guard returns. Future-proof if the early-return is ever weakened."
  - "Inner match range pattern `2..=4` (vs OR pattern `2 | 3 | 4`): clippy's `manual_range_patterns` lint required the range form. Functionally identical."
  - "Op::Catalog dispatch + execute_op both call `program::op_catalog` (no inline body): matches the helper-pattern precedent established by Op::Size and Op::Clst (Phase 22-03). Keeps a single source of truth for the 25-line CATALOG body."
  - "Op::Asn `dispatch` arm consumes the variant by-value (`Op::Asn { name, key_code }`), so the owned String moves into op_asn — no `.clone()` needed. Verified by cargo check + clippy."
  - "ASN struct-variant JSON shape: `{\"Asn\":{\"name\":\"SIN\",\"key_code\":11}}` (serde's default tagged-object form for struct variants). Pinned by `test_asn_json_struct_variant_shape` so any future schema drift surfaces immediately (Pitfall 9 sentinel)."
  - "BTreeMap (not HashMap) for state.assignments: deterministic JSON serialization order. Matches Phase 5 `key_assignments` precedent (D-25, D-29) and ensures byte-identical save-file round-trips."
  - "`assignments` field placed adjacent to `key_assignments` in state.rs (lines 88 / 94) — same struct location for grep affinity and visual relation between the two coexisting maps."

patterns-established:
  - "Hardware-faithful structured-output to print_buffer: header / payload / footer pattern at uniform width. Reusable for any future emulator-listing op (e.g., REGISTERS, FLAGS-LIST)."
  - "Empty-string-as-tombstone for map ops: a single Op variant covers both insert and remove operations cleanly via the `is_empty()` discriminant. Saves the need for a separate Op::Unasn variant."
  - "Long-content truncation by `.chars().take(N).collect::<String>()` (vs char-byte truncation): UTF-8 safe; degrades gracefully for multi-byte names. Reusable for any 24-char-width listing op."

requirements-completed: [FN-MEM-05, FN-KEY-01]

# Metrics
duration: ~20 min
completed: 2026-05-14
---

# Phase 22 Plan 04: CATALOG & ASN Summary

**Final Phase 22 plan: CATALOG (hardware-faithful program listing) + ASN (empty-name-removes key assignments) + new `assignments: BTreeMap<u8, String>` field on CalcState — completing the v2.2 HP-41CV ROM op coverage for program control & memory operations. Phase 22 is now complete with 13 new Op variants and 1 new CalcState field across 4 plans.**

## Performance

- **Duration:** ~20 min (single uninterrupted execution wave; no checkpoints, no deviations)
- **Started:** 2026-05-14 (worktree-agent-ab49e95c76a827401)
- **Tasks:** 4 of 4 complete
- **Files modified:** 5 production files + 2 new integration test files
- **Lines added:** ~135 production + 444 test

## Accomplishments

- **CalcState.assignments field (D-22.17, FN-KEY-01):** new `BTreeMap<u8, String>` with `#[serde(default)]` placed adjacent to the Phase 5 `key_assignments` field (state.rs:94). Initialized to empty in `CalcState::new()`. The `#[serde(default)]` keeps every v1.0–v2.1 save file loadable: `v20-autosave.json` (no `assignments` field) deserializes with the map empty, verified by sentinel test `test_load_v20_save_no_assignments_field` (D-22.22).
- **Op::Catalog(u8) (D-22.16 AMENDED / OQ-1 Option B, FN-MEM-05):** hardware-faithful HP-41 CATALOG. `n == 0` OR `n >= 5` → `HpError::InvalidOp` with `print_buffer` untouched. For valid n: header `-- CATALOG n --` (24-char-padded), payload, footer `-- END --` (also 24-char-padded). CAT 1 enumerates `Op::Lbl` entries with step counts to the next LBL (or `program.len()` for the last labelled block); long names truncated to 9 chars to keep total line width at 24. CAT 2/3/4 emit a single 24-char `NOT AVAILABLE` payload line (no XROM/HP-IL/peripherals in this emulator). LiftEffect: Neutral.
- **Op::Asn { name: String, key_code: u8 } (D-22.18 AMENDED / OQ-3 Option A, FN-KEY-01):** struct-variant. If `name.is_empty()` → `state.assignments.remove(&key_code)` (silent no-op when absent). Otherwise → `state.assignments.insert(key_code, name)`. `key_code` uses HP-41 row×10+col 1-indexed encoding (matches `last_key_code` and `keycode_to_hp41_code`). Hardware-faithful: `ASN "" 11` undoes `ASN "SIN" 11`. Late-binding resolution (parse-as-Op vs LBL search) is deferred to Phase 25/26 per D-22.19 — hp41-core stores the assignment as a plain String. LiftEffect: Neutral.
- **JSON struct-variant shape pinned (Pitfall 9):** `Op::Asn { name: "SIN", key_code: 11 }` serializes as exactly `{"Asn":{"name":"SIN","key_code":11}}` — serde's default tagged-object form for struct variants. The sentinel test `test_asn_json_struct_variant_shape` pins this so any future schema drift (e.g., from a serde-attribute addition) surfaces immediately.
- **Save-file round-trip (FN-KEY-01 SC#5):** multi-entry `state.assignments` survives a JSON save/load round-trip with all entries intact AND in deterministic BTreeMap order. Re-serializing the round-tripped state produces byte-identical JSON — verified by `test_asn_roundtrip_through_json`.
- **4-place rule + SC-4 invariant honored:** both new variants land in (1) `ops/mod.rs::Op` enum, (2) `ops/mod.rs::dispatch()` match, (3) `ops/program.rs::execute_op()` match, (4) BOTH `prgm_display.rs` copies (CLI + GUI, intentional duplication per CLAUDE.md §SC-4). Compile-time exhaustive-match coverage intact.
- **Test coverage:** 20 new integration tests pass (10 per file); each FN-ID has at least one positive test plus dedicated sentinel tests for the locked decisions (OQ-1 CAT 1 enumeration / OQ-1 CAT 2-4 NOT AVAILABLE / OQ-3 empty-name-removes / Pitfall 9 JSON shape / D-22.22 serde-default-empty via v20 fixture). `cargo test --package hp41-core` reports 660/660 (was 640). `just ci` green; hp41-core coverage 92.32% lines / 90.13% regions (≥80% gate; effectively unchanged from the Phase 22-03 baseline of 92.28% regions).

## Task Commits

Each task committed atomically on `worktree-agent-ab49e95c76a827401`:

1. **Task 22-04-01: Add CalcState.assignments field with serde(default)** — `1a1ead4` (feat)
2. **Task 22-04-02: Op::Catalog(u8) hardware-faithful (CAT 1 enumerates, 2-4 NOT AVAILABLE)** — `73c70e0` (feat)
3. **Task 22-04-03: Op::Asn { name, key_code } with empty-name-removes** — `687adc4` (feat)
4. **Task 22-04-04: phase22_catalog + phase22_asn integration suites** — `820ec3d` (test)

Plan metadata (this SUMMARY): will be committed as `docs(22-04)` per the parallel-executor protocol.

## Files Created/Modified

### Created
- `hp41-core/tests/phase22_catalog.rs` (210 lines) — 10 integration tests covering FN-MEM-05 (CAT 1/2/3/4) + OQ-1 sentinels + invalid n rejection + 24-char width discipline + long-label truncation + LiftEffect::Neutral preservation.
- `hp41-core/tests/phase22_asn.rs` (234 lines) — 10 integration tests covering FN-KEY-01 (insert, overwrite, empty-name-removes, remove-nonexistent-noop, JSON round-trip) + D-22.22 serde-default-empty + Pitfall 9 JSON-shape pin + LiftEffect::Neutral preservation + multi-key coexistence.

### Modified
- `hp41-core/src/state.rs` — new field `assignments: BTreeMap<u8, String>` with `#[serde(default)]` placed adjacent to `key_assignments` (line 94); initialized to `BTreeMap::new()` in `CalcState::new()` (line 167).
- `hp41-core/src/ops/mod.rs` — 2 new Op variants appended at end of Op enum per D-22.22 (`Catalog(u8)` + `Asn { name: String, key_code: u8 }`); 2 new dispatch arms delegating to `program::op_catalog` and `program::op_asn`.
- `hp41-core/src/ops/program.rs` — 2 new public helpers (`pub fn op_catalog`, `pub fn op_asn`) placed adjacent to the other Phase 22 helpers (op_size, op_clst); 2 new execute_op arms (regular dispatch ops, NOT in the programming-ops catch-all — they execute fine in both interactive AND run_loop contexts).
- `hp41-cli/src/prgm_display.rs` — 2 new arms: `Op::Catalog(n) => format!("CATALOG {n}")`, `Op::Asn { name, key_code } => format!("ASN \"{name}\" {key_code:02}")`.
- `hp41-gui/src-tauri/src/prgm_display.rs` — same 2 arms, intentional duplication per CLAUDE.md SC-4 invariant.

## Decisions Made

- **CATALOG 1 long-label truncation to 9 chars** rather than overflow or "..." marker. Hardware HP-41 listings show labels in a fixed 9-char field; we follow the same convention. The 24-char total line width = `LBL ` (4) + name:9 + `  ` (2) + steps:5 = 20, padded to 24. Test `test_catalog_long_label_truncated_to_9_chars` asserts a 13-char label `VERYLONGLABEL` renders as the first 9 chars `VERYLONGL` on a 24-char-wide line.
- **`_ => return Err(HpError::InvalidOp)` arm in op_catalog (instead of `unreachable!()`)**: clippy is happy under the workspace `-D warnings` setting (avoids the `manual_unreachable_unsafe` discussion entirely), and the arm IS guarded by the outer `n == 0 || n >= 5` early-return so it never executes in well-formed input. Defensive fallback matches the outer guard's error, so the behavior is unchanged.
- **Inner match uses range pattern `2..=4`** (not `2 | 3 | 4`): clippy's `manual_range_patterns` lint required the range form under `-D warnings`. Functionally identical; one-line fix on first clippy failure.
- **op_catalog inlined logic vs separate per-CAT helpers**: 25 lines split into a 4-arm inner match plus header/footer push code; cleanly localized. Phase 24 may revisit if any of the NOT-AVAILABLE arms ever gain real bodies, but for v2.2 a single helper is the right granularity.
- **op_asn `dispatch` arm consumes Op by value** (`Op::Asn { name, key_code }`): the owned String moves into the helper call without a clone — verified by clippy. Same pattern as Op::Clp's `&name` borrow (which borrows because op_clp takes `&str`), but op_asn takes ownership.
- **No new HpError variants for CATALOG**: reusing `InvalidOp` for n out-of-range keeps the error surface stable.
- **`assignments` adjacent to `key_assignments`** in state.rs (lines 88 and 94): grep affinity — searching for "assignments" surfaces both fields together, making the future Phase 25/26 reconciliation easier to plan.
- **No `state.rs::CalcState::new()` default-trait derive**: the existing pattern is explicit construction in `new()`; followed.

## Deviations from Plan

None — plan executed exactly as written. All 4 tasks landed in order with no deviation rules triggered:
- No bugs found in the prior Phase 22 surface (22-01 / 22-02 / 22-03).
- No missing critical functionality discovered.
- No blocking issues; no architectural changes needed.
- No auth gates (purely core code).
- Two trivial clippy lints (manual_range_patterns → `2..=4`; the `unreachable!()` cleanup → `Err(InvalidOp)`) caught on first `cargo clippy` run and fixed in-place before committing Task 22-04-02. Standard "execute → lint → fix → commit" flow.

## Issues Encountered

None functional. The single ergonomic note: my first draft of the catch-all in op_catalog's inner match used `unreachable!()` per the plan's `<behavior>` sketch wording. clippy didn't actually complain about `unreachable!()` in this case, but the plan also offered the defensive `_ => Err(HpError::InvalidOp)` alternative. I chose the defensive form because it's future-proof: if the outer guard is ever weakened, the inner arm fails closed rather than panicking. Otherwise clippy required the `manual_range_patterns` fix (`2 | 3 | 4` → `2..=4`), which was a one-character change.

## User Setup Required

None — entirely additive on `hp41-core`. No new dependencies, no env vars, no service config. The new ops surface to end users via Phase 25 (CLI keyboard CATALOG modal + ASN 2-step modal) and Phase 26 (GUI key_map can now resolve the v2.1-stubbed `catalog` and `asn` ids).

## Next Plan Readiness

**Phase 22 is COMPLETE.** All 4 plans landed. 13 new Op variants in `hp41-core`, 1 new CalcState field, 4 integration test suites (66 new tests across phase22_program_control + phase22_program_edit + phase22_memory_ops + phase22_catalog + phase22_asn), Wave-0 bounds audit across 28 register-access sites in registers/display_ops/stats. Phase 22 milestone exit gate: `just ci` green, hp41-core coverage ≥90% (target ≥80%, well above), zero panics introduced.

**Phase 23 (ALPHA Operations) is unblocked.** Builds on the alpha_reg surface that Phase 22 plan 22-03 touched (CLA delegation). Phase 23 lands FN-ALPHA-01..06 (AS, ASTO, ARCL, ATOX, XTOA, AROT) onto the existing alpha_reg field.

**Phase 24 (Indirect Addressing) has a clean refactor target.** The 6-step inline indirect resolver in Op::GtoInd / Op::XeqInd arms (Plan 22-01) is intentionally duplicated. Phase 24's `resolve_indirect(state, reg) -> Result<u8, HpError>` helper extracts steps 1–4 (register read + integer truncate + non-integer reject + stringify); the run_loop arms then become 2 lines each.

**Phase 25/26 (CLI/GUI keyboard wiring)** can wire the v2.1-stubbed `catalog`, `asn`, `clp`, `del`, `ins`, `size_prompt`, `r_s` ids to real Op variants. The hp41-core surface is fully ready: keyboard prompts (CLI `PendingInput` variants + GUI modals) just need to construct the right Op and dispatch.

## Self-Check: PASSED

Files claimed created/modified verified present:
- `hp41-core/tests/phase22_catalog.rs` — FOUND (210 lines, 10 tests)
- `hp41-core/tests/phase22_asn.rs` — FOUND (234 lines, 10 tests)
- `hp41-core/src/state.rs` — FOUND (modified, contains `pub assignments: BTreeMap<u8, String>` with `#[serde(default)]`)
- `hp41-core/src/ops/mod.rs` — FOUND (modified, contains `Catalog(u8)` and `Asn { name: String, key_code: u8 }` variants + 2 dispatch arms)
- `hp41-core/src/ops/program.rs` — FOUND (modified, contains `pub fn op_catalog` + `pub fn op_asn` + 2 execute_op arms)
- `hp41-cli/src/prgm_display.rs` — FOUND ("CATALOG" + "ASN" arms present)
- `hp41-gui/src-tauri/src/prgm_display.rs` — FOUND (same 2 arms present)

Commit hashes verified present on `worktree-agent-ab49e95c76a827401`:
- `1a1ead4` — feat(22-04): CalcState.assignments field ✓
- `73c70e0` — feat(22-04): Op::Catalog(u8) ✓
- `687adc4` — feat(22-04): Op::Asn struct-variant ✓
- `820ec3d` — test(22-04): phase22_catalog + phase22_asn (20 tests) ✓

Quality gates verified green:
- `cargo check --workspace` — exit 0
- `cd hp41-gui/src-tauri && cargo check` — exit 0
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0
- `cargo test -p hp41-core --test phase22_catalog` — 10 passed, 0 failed
- `cargo test -p hp41-core --test phase22_asn` — 10 passed, 0 failed
- `cargo test -p hp41-core` — 660 passed, 0 failed (was 640 before this plan)
- `just ci` — exit 0 (workspace tests + clippy + fmt + coverage); hp41-core 92.32% lines / 90.13% regions
- Zero `.unwrap()` / `panic!()` introduced in production code
- Backward-compat: v20-autosave.json deserializes with `assignments` empty via `#[serde(default)]`
- Pitfall 9: ASN JSON struct-variant shape pinned `{"Asn":{"name":"SIN","key_code":11}}`

---
*Phase: 22-program-control-and-memory-ops*
*Plan: 04-catalog-and-asn*
*Completed: 2026-05-14*
*PHASE 22 COMPLETE*
