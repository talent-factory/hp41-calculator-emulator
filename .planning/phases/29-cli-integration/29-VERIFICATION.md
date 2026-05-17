---
phase: 29-cli-integration
verified: 2026-05-17T12:06:12Z
status: gaps_found
score: 4/5 must-haves verified
total_must_haves: 5
met: 4
unmet: 1
overrides_applied: 0
re_verification: false
gaps:
  - truth: "key_coverage.rs D-25.18 invariant holds for Math Pac I pool: every implemented Math Pac I JSON entry with non-null key_path dispatches via xeq_by_name_local_resolve to a known Op:: variant"
    status: failed
    reason: "key_coverage.rs::key_coverage_implemented_entries_dispatch still uses help_entries() (v2.2 narrow accessor) at line 120, never probing the 45 Math Pac I XEQ entries. BL-04 from 29-REVIEW.md is confirmed in codebase. The closure invariant from D-25.18 is broken for the entire Math1 pool."
    artifacts:
      - path: "hp41-cli/tests/key_coverage.rs"
        issue: "Line 120: `let entries = help_entries();` should be `let entries: Vec<&_> = help_entries_all().collect();`. The import at line 19 uses `help_data::help_entries` (not `help_entries_all`). The assertion floor at line 239 (`>= 50`) would also need raising to >= 95 to confirm the Math Pac I entries are probed."
    missing:
      - "Change line 19 import to include `help_entries_all` and line 120 to `let entries: Vec<&_> = help_entries_all().collect();`"
      - "Add a Math1-specific assertion sub-loop asserting every Math1 XEQ entry resolves via `xeq_by_name_local_resolve(&name, 0b0000_0001).is_some()`"
      - "Raise the assertion floor at line 239 from `>= 50` to `>= 95` (62 v2.2 + ~45 Math1 math entries with key_path)"

human_verification:
  - test: "Run `XEQ \"MATRIX\"` interactively in hp41-cli and step through the ORDER=? modal with R/S"
    expected: "Status bar shows `ORDER=?`; entering a digit and pressing F5 advances to `A1,1=?`; Esc shows `Cancelled`"
    why_human: "End-to-end TUI rendering with modal_prompt cannot be verified by grep or unit tests alone — requires a live ratatui terminal"
  - test: "Run `XEQ \"SOLVE\"` and verify FUNCTION NAME? auto-opens XEQ collection mode"
    expected: "Status bar shows `FUNCTION NAME?`; typing a label name and pressing Enter advances modal to `GUESS 1=?`"
    why_human: "Auto-open of CollectForModal + label submission involves App state machine timing that unit tests cover but visual rendering is only verifiable in a live terminal"
---

# Phase 29: CLI Integration Verification Report

**Phase Goal:** Every Math Pac I function reachable from `hp41-cli` via `XEQ`-by-name; ALPHA prompts (`ORDER=?`, `A1,1=?`, `FUNCTION NAME?`, `GUESS 1=?`) surface in `state.modal_prompt` and render in the TUI status bar; `?`-overlay lists Math Pac I entries in their own section.
**Verified:** 2026-05-17T12:06:12Z
**Status:** gaps_found
**Re-verification:** No — initial verification

**Note on ROADMAP goal wording:** The ROADMAP goal says prompts "surface in `state.print_buffer`". Decision D-28.4 (locked in 29-CONTEXT.md before Phase 29 execution) changed this to `state.modal_prompt` as the prompt channel, with `print_buffer` reserved for PRX/PRA/PRSTK output only. The implementation follows D-28.4 correctly; the ROADMAP goal text was not updated to match. This is an imprecision in the goal statement, not an implementation gap.

## Gate Results

| Gate | Result |
|------|--------|
| `just test` | PASS — all tests passed, 0 failures |
| `just lint` | PASS — clippy exits 0, no warnings |

## Observable Truths (ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| SC-1 | `xeq_by_name_local_resolve("SINH")` invokes `xrom_resolve` and returns `Op::Sinh`; all 3 call sites share resolver chain | VERIFIED | `hp41-cli/src/keys.rs:382` — `_ => hp41_core::ops::math1::xrom::xrom_resolve(name, xrom_modules)` is the explicit final fallback. Production call site `app.rs:1490` passes `self.state.xrom_modules`. `phase25_xeq_by_name.rs::cli_resolver_matches_core_resolver` extended with 10 positive + 10 negative Math Pac I cases. |
| SC-2 | `help_data.rs` loads a second JSON file via an additional `OnceLock`; `?` overlay groups Math Pac I entries in distinct categories | VERIFIED | `help_data.rs:105-135` — `MATH1_FUNCTIONS_JSON`, `MATH1_HELP_ENTRIES`, `help_entries_math1()`, `help_entries_all()` all present. `help_overlay_rows()` at line 156 uses `help_entries_all().collect()`. 45 JSON entries with 11 `Math1 *` categories confirmed by inspection. `phase29_help_data_math1.rs` 10-test suite passes. |
| SC-3 | `op_display_name` exhaustive match has ~40 new arms; no `_ =>` catch-all; program listings show authentic mnemonics | VERIFIED | `prgm_display.rs:239-292` (CLI) — 44 arms counted per 29-02-SUMMARY audit, zero `_ =>` catch-all. GUI copy is identical (44 arms). Compile-time enforcement: `cargo check -p hp41-cli` and `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` both exit 0. |
| SC-4 | `key_ref_entries()` (right-panel) derives Math Pac I entries from merged `help_entries_all()`, no parallel hand-curated table | VERIFIED | `keys.rs:406` — `for entry in crate::help_data::help_entries_all()`. `phase29_key_ref_includes_math1.rs` 3-test suite passes (SINH row present, MATRIX row present, v2.2 regression guard). `render_right_panel` at `ui.rs:450` calls `key_ref_entries()` directly. |
| SC-5 | Modal-prompt routing for all 7 workflows: XEQ triggers ModalProgram state machine; prompt text renders in TUI; user input flows through number-entry pipeline; ALPHA label integration | VERIFIED | `mod.rs:44-116` — `submit_modal`, `cancel_modal`, `submit_modal_with_label` wired. `app.rs:661-688` — F5 + Esc interceptors present with correct D-07 + §7.2 precedence ordering. `modal.rs:76` — `requires_alpha_label()` exists. `app.rs:1772` — `maybe_auto_open_collect_for_modal` wired. `ui.rs:230-234` — `pending_prompt` with widened signature renders `modal_prompt` in status bar. `phase29_modal_flow.rs` 5 integration tests pass. `phase29_pending_prompt_modal.rs` 3 unit tests pass. |

**Score:** 5/5 ROADMAP success criteria verified.

## Per-Requirement Verification Table

| Req | Description | Status | Evidence |
|-----|-------------|--------|----------|
| CLI-01 | `xeq_by_name_local_resolve` calls `xrom_resolve` after `builtin_card_op` | MET | `keys.rs:382` — explicit `_ => hp41_core::ops::math1::xrom::xrom_resolve(name, xrom_modules)` arm. Signature widened to accept `xrom_modules: u8`. Production call at `app.rs:1490` passes `self.state.xrom_modules`. Negative test (`0b0000_0000`) coverage in `phase25_xeq_by_name.rs`. |
| CLI-02 | Second JSON `OnceLock` in `help_data.rs`; `?` overlay shows Math Pac I entries | MET | `help_data.rs:105-135`: `MATH1_FUNCTIONS_JSON`, `MATH1_HELP_ENTRIES`, `help_entries_math1()`, `help_entries_all()`. 45 entries, all `module_id: 7`. `help_overlay_rows()` uses merged accessor. Bidirectional parity: `function_matrix_parity.rs` 3 new tests (7 total). |
| CLI-03 | `prgm_display.rs` has ~40 new `op_display_name` arms for Phase 28 Op variants | MET | 44 arms confirmed in CLI `prgm_display.rs:239-292`. GUI copy identical (44 arms). No `_ =>` catch-all. Compile-time enforcement via Rust exhaustive-match. |
| CLI-04 | `KEY_REF_TABLE` / right-panel derives Math Pac I entries from `help_entries_all()` | MET | `keys.rs:406` migrated to `help_entries_all()`. Old `help_entries()` call replaced. `render_right_panel` at `ui.rs:450` consumes `key_ref_entries()`. `phase29_key_ref_includes_math1.rs` 3 tests confirm SINH and MATRIX rows surface. |
| CLI-05 | Modal-prompt routing: MATRIX/SOLVE/POLY/INTG/DIFEQ/FOUR/TRANS trigger ModalProgram state machine; prompts render; input flows through number-entry; ALPHA labels integrate | MET | Full wiring verified: `submit_modal`/`cancel_modal`/`submit_modal_with_label` in `mod.rs`. R/S (F5) and Esc interceptors in `app.rs`. `maybe_auto_open_collect_for_modal` post-dispatch hook. `pending_prompt` widened signature. 5+3 Wave-0 tests pass. |

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `docs/hp41-math1-functions.json` | 45 entries, all `module_id: 7` | VERIFIED | 45 entries confirmed by `python3`; `grep -c "module_id": 7` = 45 |
| `hp41-cli/src/help_data.rs` | `XromEntry`, `MATH1_HELP_ENTRIES`, `help_entries_math1`, `help_entries_all` | VERIFIED | All 4 items present at lines 33-135 |
| `hp41-cli/src/keys.rs` | Widened signature, `xrom_resolve` as final fallback | VERIFIED | `xeq_by_name_local_resolve(name: &str, xrom_modules: u8)` at line 358; `xrom_resolve` at line 382 |
| `hp41-core/src/ops/math1/mod.rs` | `submit_modal`, `cancel_modal`, `submit_modal_with_label`, `ModalProgram` re-export | VERIFIED | All present at lines 44-116 |
| `hp41-core/src/ops/math1/modal.rs` | `requires_alpha_label()` method | VERIFIED | Line 76 |
| `hp41-cli/src/app.rs` | `XeqByNameMode`, struct-variant `XeqByName`, R/S + Esc interceptors, auto-open hook | VERIFIED | `XeqByNameMode` at line 45; interceptors at lines 661-688; `maybe_auto_open_collect_for_modal` at line 1772 |
| `hp41-cli/src/ui.rs` | Widened `pending_prompt`, `modal_prompt` surfaces in status bar | VERIFIED | `pending_prompt` at line 272; status bar call at lines 230-234 |
| `hp41-cli/tests/phase29_help_data_math1.rs` | 10-test smoke suite | VERIFIED | File exists, 10 tests |
| `hp41-cli/tests/phase29_key_ref_includes_math1.rs` | 3-test Wave-0 suite | VERIFIED | File exists, 3 tests |
| `hp41-cli/tests/phase29_modal_flow.rs` | 5 integration tests | VERIFIED | File exists, 5 tests |
| `hp41-cli/tests/phase29_pending_prompt_modal.rs` | 3 unit tests | VERIFIED | File exists, 3 tests |
| `hp41-cli/tests/key_coverage.rs` | D-25.18 closure invariant extended to Math Pac I | FAILED — BLOCKER | Still uses `help_entries()` narrow accessor at line 120; 45 Math Pac I XEQ entries never probed |

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `keys.rs::xeq_by_name_local_resolve` | `hp41-core::ops::math1::xrom::xrom_resolve` | `_ =>` final arm | WIRED | Line 382: `_ => hp41_core::ops::math1::xrom::xrom_resolve(name, xrom_modules)` |
| `help_data.rs::help_entries_math1` | `docs/hp41-math1-functions.json` | `include_str!` + `OnceLock` | WIRED | Line 105: `include_str!("../../docs/hp41-math1-functions.json")` |
| `help_data.rs::help_overlay_rows` | `help_entries_all()` | merged iterator | WIRED | Line 156: `let entries: Vec<&HelpEntry> = help_entries_all().collect()` |
| `keys.rs::key_ref_entries` | `help_entries_all()` | merged iterator | WIRED | Line 406: `for entry in crate::help_data::help_entries_all()` |
| `app.rs::handle_key (F5 interceptor)` | `hp41_core::ops::math1::submit_modal` | F5 when `modal_program.is_some()` | WIRED | Lines 661-673 |
| `app.rs::handle_key (Esc interceptor)` | `hp41_core::ops::math1::cancel_modal` | Esc when `modal_program.is_some() && !shift_armed` | WIRED | Lines 681-688 |
| `app.rs::call_dispatch (post-dispatch hook)` | `maybe_auto_open_collect_for_modal` | trailing call | WIRED | Lines 1717, 1757 |
| `app.rs::handle_xeq_by_name (Enter, CollectForModal)` | `submit_modal_with_label` | `XeqByNameMode::CollectForModal` branch | WIRED | Lines 1469-1477 |
| `ui.rs::pending_prompt` | `state.modal_prompt` | widened signature arg | WIRED | Lines 230-234 |

## Gaps Summary

One gap is confirmed by codebase inspection. It maps exactly to BL-04 in the code review.

**BL-04 confirmed:** `hp41-cli/tests/key_coverage.rs::key_coverage_implemented_entries_dispatch` iterates `help_entries()` (v2.2 narrow accessor, ~62 entries) at line 120, leaving the 45 Math Pac I entries with `key_path: "XEQ \"<NAME>\""` completely unprobed. The D-25.18 closure invariant — "every implemented JSON entry with non-null `key_path` dispatches to a known `Op::` variant" — is broken for the entire Math1 pool. A future JSON-authoring mistake (misspelled display_name) would pass all tests silently. The fix is a single-line change to the accessor plus an assertion floor bump.

**Assessment of code review Critical/Blocker findings vs. Phase 29 CLI requirements:**

- **CR-01** (`op_integ_run_loop` leaks `integ_state`): kernel correctness bug in Phase 28 code. Phase 29 scope is CLI plumbing (CLI-01 to CLI-05). CLI-05 requires modal routing works (R/S submits, Esc cancels, prompts render) — it does. The run_loop correctness is out of Phase 29 scope.
- **CR-02** (modal params disconnected from run_loop): same — kernel correctness from Phase 28. Not a CLI-05 routing failure; routing from keyboard to `submit_modal` works.
- **CR-03** (`FourInputStep::SamplePrompt` register collision): Phase 28 kernel bug, not a CLI-05 routing issue.
- **CR-04** (`op_integ` skips `ModeChoice` step): Phase 28 kernel incomplete implementation; modal opens at `FunctionNamePrompt` (CLI-05 tests verify this behavior and it passes). The skipped step is a Phase 28 correctness gap.
- **BL-01** (Trans `Init2dPrompt` drops y₀/θ): Phase 28 kernel bug.
- **BL-02** (Trans `ForwardPrompt`/`InversePrompt` do no transform): Phase 28 kernel bug; the doc overclaim is in `docs/verifying-math-pac-1.md` (Phase 29 documentation).
- **BL-03** (difeq.rs doc-comment contradicts register layout): documentation bug; does not affect CLI routing.
- **BL-04** (key_coverage.rs not migrated): THIS is the confirmed Phase 29 gap.

**WR-05** (dead unwrap_or in `pending_prompt` line 280): confirmed at `ui.rs:280` — `modal_prompt.unwrap_or("").to_string()` is indeed dead defensive code after the `is_some()` guard. Not a correctness failure but violates the `clippy::unwrap_used` convention. Clippy does not flag this because `unwrap_or` is not `unwrap`. This is a WARNING only.

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `hp41-cli/src/ui.rs` | 280 | `modal_prompt.unwrap_or("").to_string()` after `is_some()` guard | Warning | `unwrap_or` fallback `""` is dead code; intent is obscured; does not trigger clippy::unwrap_used |
| `hp41-cli/tests/key_coverage.rs` | 120 | `let entries = help_entries()` — narrow accessor misses Math Pac I | Blocker | D-25.18 closure invariant broken for 45 Math Pac I entries |

No `TBD`, `FIXME`, or `XXX` debt markers found in Phase 29 modified files.

## Human Verification Required

### 1. Interactive Modal Flow in TUI

**Test:** Launch `cargo run -p hp41-cli`, press `X` to open XEQ-by-name modal, type `MATRIX` and Enter. Then press `2` followed by F5 (R/S). Then type a matrix value and press F5 again.
**Expected:** Status bar shows `ORDER=?` after MATRIX dispatches; advances to `A1,1=?` after F5; continues through matrix entry. Pressing Esc shows `Cancelled`.
**Why human:** TUI rendering with ratatui cannot be verified programmatically; unit tests cover the App state machine but not visual output.

### 2. Interactive SOLVE Alpha Collection

**Test:** Launch `cargo run -p hp41-cli`, press `X`, type `SOLVE` and Enter. Verify status bar and XEQ-name-collection mode open. Type function name `F` and Enter. Press F5 twice (for Guess 1 and Guess 2).
**Expected:** Status bar shows `FUNCTION NAME?`; after `F` + Enter shows `GUESS 1=?`; after Guess 1 F5 shows `GUESS 2=?`; after Guess 2 F5 with a program containing label `F` that can be called, solver result appears in print panel.
**Why human:** Full Solve workflow end-to-end requires a user program in memory and live TUI interaction.

---

_Verified: 2026-05-17T12:06:12Z_
_Verifier: Claude (gsd-verifier)_
