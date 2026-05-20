---
phase: 29-cli-integration
verified: 2026-05-17T15:30:00Z
status: verified
score: 5/5 must-haves verified
total_must_haves: 5
met: 5
unmet: 0
overrides_applied: 0
re_verification: true
previous_verification:
  verified: 2026-05-17T12:06:12Z
  previous_status: gaps_found
  previous_score: 4/5
  gaps_closed:
    - "key_coverage.rs D-25.18 invariant holds for Math Pac I pool"
  gaps_remaining: []
  regressions: []
  closure_evidence:
    - commit: ce19010
      subject: "test(29): BL-04 extend key_coverage closure to Math1 pool"
      files: ["hp41-cli/tests/key_coverage.rs"]
review_fix_context:
  report: ".planning/phases/29-cli-integration/29-REVIEW-FIX.md"
  status: all_fixed
  scope_count: 17
  commit_range: e071a88..6af997f
  note: "All 17 in-scope code-review findings closed across 15 commits. CR-01..04 and BL-01..03 were Phase 28 kernel correctness issues surfaced during Phase 29 review and are now in main; their closure is background context for this re-verification, not additional Phase 29 must-haves."

human_verification:
  - test: "Run `XEQ \"MATRIX\"` interactively in hp41-cli and step through the ORDER=? modal with R/S"
    expected: "Status bar shows `ORDER=?`; entering a digit and pressing F5 advances to `A1,1=?`; Esc shows `Cancelled`"
    why_human: "End-to-end TUI rendering with modal_prompt cannot be verified by grep or unit tests alone — requires a live ratatui terminal"
  - test: "Run `XEQ \"SOLVE\"` and verify FUNCTION NAME? auto-opens XEQ collection mode"
    expected: "Status bar shows `FUNCTION NAME?`; typing a label name and pressing Enter advances modal to `GUESS 1=?`"
    why_human: "Auto-open of CollectForModal + label submission involves App state machine timing that unit tests cover but visual rendering is only verifiable in a live terminal"
---

# Phase 29: CLI Integration Verification Report (Re-verification)

**Phase Goal:** Every Math Pac I function reachable from `hp41-cli` via `XEQ`-by-name; ALPHA prompts (`ORDER=?`, `A1,1=?`, `FUNCTION NAME?`, `GUESS 1=?`) surface in `state.modal_prompt` and render in the TUI status bar; `?`-overlay lists Math Pac I entries in their own section.
**Verified:** 2026-05-17T15:30:00Z
**Status:** verified
**Re-verification:** Yes — initial verification 2026-05-17T12:06:12Z reported `gaps_found` with one BL-04 gap on `key_coverage.rs`. Gap closed by commit `ce19010`. Surrounding 14 code-review fix commits (e071a88…6af997f) verified non-regressive against `just test` and `just lint`.

**Note on ROADMAP goal wording:** The ROADMAP goal says prompts "surface in `state.print_buffer`". Decision D-28.4 (locked in 29-CONTEXT.md before Phase 29 execution) changed this to `state.modal_prompt` as the prompt channel, with `print_buffer` reserved for PRX/PRA/PRSTK output only. The implementation follows D-28.4 correctly; the ROADMAP goal text was not updated to match. This is an imprecision in the goal statement, not an implementation gap.

**Note on out-of-scope review fixes:** Code review findings CR-01..04 and BL-01..03 surfaced during the Phase 29 review window were Phase 28 kernel correctness issues (integ leak on overflow, modal-param wiring to run_loop, FOUR sample register collision, INTG ModeChoice step, TRANS init-param capture, difeq doc-comment alignment). They are tracked in `29-REVIEW-FIX.md` and now land in main (commits 5da4183, e071a88, 856be2b, ce49e54, f6366b9, 745792b). They are NOT additional Phase 29 must-haves; the Phase 29 contract is CLI plumbing (CLI-01..CLI-05). This verification scope is unchanged from the initial pass.

## Gate Results

| Gate | Command | Exit | Result |
|------|---------|------|--------|
| `just test` | `cargo test --workspace --all-features` | 0 | PASS — all workspace test suites green; key suites observed: `key_coverage_implemented_entries_dispatch` (1 test, ok), `phase29_help_data_math1` (10), `phase29_key_ref_includes_math1` (3), `phase29_modal_flow` (5), `phase29_pending_prompt_modal` (3), `xrom_chain_order` (5), `xrom_shadowing` (2), `v3_save_compat` (2). Zero failures, zero ignored. |
| `just lint` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 0 | PASS — clippy exits 0 with no warnings. |

## Observable Truths (ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| SC-1 | `xeq_by_name_local_resolve("SINH")` invokes `xrom_resolve` and returns `Op::Sinh`; all 3 call sites share resolver chain | VERIFIED | `hp41-cli/src/keys.rs:382` — `_ => hp41_core::ops::math1::xrom::xrom_resolve(name, xrom_modules)` is the explicit final fallback inside `xeq_by_name_local_resolve` (signature at `hp41-cli/src/keys.rs:358`). `hp41-core/tests/xrom_chain_order.rs::math1_sinh_resolves_via_xrom` passes; `xrom_shadowing` 2-test suite passes (built-in names win over xrom names — C-28.4 ordering). |
| SC-2 | `help_data.rs` loads a second JSON file via an additional `OnceLock`; `?` overlay groups Math Pac I entries in distinct categories | VERIFIED | `hp41-cli/src/help_data.rs:105` — `const MATH1_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-math1-functions.json");`. `hp41-cli/src/help_data.rs:107` — `static MATH1_HELP_ENTRIES: OnceLock<Vec<HelpEntry>>`. `help_entries_math1` accessor at line 117, `help_entries_all` merged accessor at line 133. `help_overlay_rows` migrated to merged accessor at line 156. `docs/hp41-math1-functions.json` confirmed 45 entries, all `module_id: 7`. `phase29_help_data_math1.rs` 10-test suite passes. |
| SC-3 | `op_display_name` exhaustive match has ~40 new arms; no `_ =>` catch-all; program listings show authentic mnemonics | VERIFIED | `hp41-cli/src/prgm_display.rs` and `hp41-gui/src-tauri/src/prgm_display.rs` both contain 170 `Op::*` match arms each — identical counts confirm CLI/GUI parity. Zero `_ =>` catch-all. Compile-time enforcement: both `just test` and `just lint` exit 0 (would fail at compile time on a non-exhaustive match). |
| SC-4 | `key_ref_entries()` (right-panel) derives Math Pac I entries from merged `help_entries_all()`, no parallel hand-curated table | VERIFIED | `hp41-cli/src/keys.rs:406` — `for entry in crate::help_data::help_entries_all()` inside `key_ref_entries()` (line 404). `phase29_key_ref_includes_math1.rs` 3-test suite passes (SINH row present, MATRIX row present, v2.2 regression guard). SC-4 strict invariant grep (`fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)` in `hp41-gui/src-tauri/src/`) returns zero matches — Phase 29 touched only `hp41-cli/`, `docs/`, root files. |
| SC-5 | Modal-prompt routing for all 7 workflows: XEQ triggers ModalProgram state machine; prompt text renders in TUI; user input flows through number-entry pipeline; ALPHA label integration | VERIFIED | `hp41-core/src/ops/math1/mod.rs:54` (`submit_modal`), `:92` (`cancel_modal`), `:109` (`submit_modal_with_label`). `hp41-core/src/ops/math1/modal.rs:76` (`requires_alpha_label`). `hp41-cli/src/app.rs:665` F5 interceptor → `submit_modal`; `:692` Esc interceptor → `cancel_modal`; `:1478` `XeqByNameMode::CollectForModal` Enter arm → `submit_modal_with_label`; `:1724`/`:1764` auto-open hook; `:1779` `maybe_auto_open_collect_for_modal` body. `hp41-cli/src/ui.rs:230-234` widened `pending_prompt` call site reads `app.state.modal_prompt`. `phase29_modal_flow.rs` 5 integration tests + `phase29_pending_prompt_modal.rs` 3 unit tests pass. WR-06 fix (commit e91a161) resets `last_key_code = 0` on the modal F5 path per CLAUDE.md v1.1 GetKey invariant. |

**Score:** 5/5 ROADMAP success criteria verified.

## Per-Requirement Verification Table

| Req | Description | Status | Evidence |
|-----|-------------|--------|----------|
| CLI-01 | `xeq_by_name_local_resolve` calls `xrom_resolve` after `builtin_card_op` | MET | `hp41-cli/src/keys.rs:358` widened signature `pub fn xeq_by_name_local_resolve(name: &str, xrom_modules: u8) -> Option<Op>`; final fallback at `:382` explicitly arms `_ => hp41_core::ops::math1::xrom::xrom_resolve(name, xrom_modules)`. Negative test (`0b0000_0000` → None) coverage in `phase25_xeq_by_name.rs` + `xrom_chain_order.rs`. |
| CLI-02 | Second JSON `OnceLock` in `help_data.rs`; `?` overlay shows Math Pac I entries | MET | `hp41-cli/src/help_data.rs:105`/`:107`/`:117`/`:133` (constants, OnceLock, accessor, merged accessor); migrated `help_overlay_rows` at `:156` to merged accessor. 45 entries all `module_id: 7` in `docs/hp41-math1-functions.json`. Bidirectional Op-enum ↔ JSON parity asserted by `function_matrix_parity.rs`. `phase29_help_data_math1.rs` 10-test smoke suite passes. |
| CLI-03 | `prgm_display.rs` has ~40 new `op_display_name` arms for Phase 28 Op variants | MET | 170 arms each in CLI (`hp41-cli/src/prgm_display.rs`) and GUI (`hp41-gui/src-tauri/src/prgm_display.rs`) copies — bit-for-bit count parity confirms the CLAUDE.md v2.0 "every new `Op` variant must be added in both copies" invariant. Zero `_ =>` catch-all (compile-time exhaustive). |
| CLI-04 | `KEY_REF_TABLE` / right-panel derives Math Pac I entries from `help_entries_all()` | MET | `hp41-cli/src/keys.rs:404`-`:418` `key_ref_entries()` body uses `help_entries_all()` at line 406; deduplicates by `key_path` via `BTreeMap`. Consumed by `render_right_panel` in `ui.rs`. `phase29_key_ref_includes_math1.rs` confirms SINH + MATRIX rows surface. |
| CLI-05 | Modal-prompt routing: MATRIX/SOLVE/POLY/INTG/DIFEQ/FOUR/TRANS trigger ModalProgram state machine; prompts render; input flows through number-entry; ALPHA labels integrate | MET | Full wiring: `submit_modal`/`cancel_modal`/`submit_modal_with_label` in `math1/mod.rs:54/92/109`; F5 + Esc interceptors in `app.rs:665/692`; CollectForModal Enter arm in `app.rs:1478`; `maybe_auto_open_collect_for_modal` in `app.rs:1779`; widened `pending_prompt` in `ui.rs:272`; modal-active guards (WR-01/WR-02) prevent stack/alpha_reg pollution on degenerate call sites. 5 + 3 + 10 + 3 = 21 Phase 29 tests pass. |

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `docs/hp41-math1-functions.json` | 45 entries, all `module_id: 7` | VERIFIED | 45 entries confirmed via `python3 -c "import json; d = json.load(open('docs/hp41-math1-functions.json')); print(len(d))"` → 45; `grep -c '"module_id": 7'` → 45 |
| `hp41-cli/src/help_data.rs` | `XromEntry`, `MATH1_HELP_ENTRIES`, `help_entries_math1`, `help_entries_all` | VERIFIED | All 4 items present at lines 102-135 |
| `hp41-cli/src/keys.rs` | Widened signature, `xrom_resolve` as final fallback | VERIFIED | `xeq_by_name_local_resolve(name: &str, xrom_modules: u8)` at line 358; `xrom_resolve` fallback at line 382 |
| `hp41-core/src/ops/math1/mod.rs` | `submit_modal`, `cancel_modal`, `submit_modal_with_label`, `ModalProgram` re-export | VERIFIED | Lines 54, 92, 109 — all carry post-fix modal-active guards (WR-01/WR-02 fixes in commit 03a7625) |
| `hp41-core/src/ops/math1/modal.rs` | `requires_alpha_label()` method | VERIFIED | Line 76 |
| `hp41-cli/src/app.rs` | `XeqByNameMode`, struct-variant `XeqByName`, R/S + Esc interceptors, auto-open hook | VERIFIED | `XeqByNameMode` enum at line 45; F5 interceptor at line 665 (with WR-06 last_key_code reset); Esc cancel_modal at line 692; CollectForModal Enter arm at line 1478; auto-open hook calls at lines 1724/1764; `maybe_auto_open_collect_for_modal` at line 1779 |
| `hp41-cli/src/ui.rs` | Widened `pending_prompt`, `modal_prompt` surfaces in status bar | VERIFIED | `pending_prompt(pending: Option<&PendingInput>, modal_prompt: Option<&str>)` at line 272; status bar call site at lines 230-234; WR-05 `if let Some(mp)` replacement at line 284 (commit a9dcf09) |
| `hp41-cli/tests/phase29_help_data_math1.rs` | 10-test smoke suite | VERIFIED | File exists; `just test` shows 10 passed, 0 failed |
| `hp41-cli/tests/phase29_key_ref_includes_math1.rs` | 3-test Wave-0 suite | VERIFIED | File exists; 3 passed |
| `hp41-cli/tests/phase29_modal_flow.rs` | 5 integration tests | VERIFIED | File exists; 5 passed |
| `hp41-cli/tests/phase29_pending_prompt_modal.rs` | 3 unit tests | VERIFIED | File exists; 3 passed |
| `hp41-cli/tests/key_coverage.rs` | D-25.18 closure invariant extended to Math Pac I (BL-04 closure) | VERIFIED | **BL-04 closed (was FAILED in prior verification).** Line 19 import now reads `use hp41_cli::help_data::{help_entries_all, HelpEntry};`. Line 126 main loop: `let entries: Vec<&HelpEntry> = help_entries_all().collect();`. Line 244-249: probed-floor assertion raised from `>= 50` to `>= 95`. Lines 257-299: new Math1-specific sub-loop iterates `xrom.module_id == 7` entries, calls `xeq_by_name_local_resolve(name, 0b0000_0001)` per entry, asserts `resolved.is_some()`; floor at line 294 asserts `math1_probed >= 40`. `cargo test --test key_coverage` → 1 test, 0 failures. Closure commit: `ce19010` ("✅ test(29): BL-04 extend key_coverage closure to Math1 pool"). |

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `keys.rs::xeq_by_name_local_resolve` | `hp41-core::ops::math1::xrom::xrom_resolve` | `_ =>` final arm | WIRED | `hp41-cli/src/keys.rs:382` |
| `help_data.rs::help_entries_math1` | `docs/hp41-math1-functions.json` | `include_str!` + `OnceLock` | WIRED | `hp41-cli/src/help_data.rs:105` (include_str!), `:107` (OnceLock), `:117-122` (parse + cache) |
| `help_data.rs::help_overlay_rows` | `help_entries_all()` | merged iterator | WIRED | `hp41-cli/src/help_data.rs:156` |
| `keys.rs::key_ref_entries` | `help_entries_all()` | merged iterator | WIRED | `hp41-cli/src/keys.rs:406` |
| `app.rs::handle_key (F5 interceptor)` | `hp41_core::ops::math1::submit_modal` | F5 when `modal_program.is_some()` | WIRED | `hp41-cli/src/app.rs:665` |
| `app.rs::handle_key (Esc interceptor)` | `hp41_core::ops::math1::cancel_modal` | Esc when `modal_program.is_some() && !shift_armed` | WIRED | `hp41-cli/src/app.rs:692` |
| `app.rs::call_dispatch (post-dispatch hook)` | `maybe_auto_open_collect_for_modal` | trailing call | WIRED | `hp41-cli/src/app.rs:1724`, `:1764` |
| `app.rs::handle_xeq_by_name (Enter, CollectForModal)` | `submit_modal_with_label` | `XeqByNameMode::CollectForModal` branch | WIRED | `hp41-cli/src/app.rs:1478` |
| `ui.rs::pending_prompt` | `state.modal_prompt` | widened signature arg | WIRED | `hp41-cli/src/ui.rs:272` (signature), `:230-234` (call site) |
| `tests/key_coverage.rs` (Math1 sub-loop) | `xeq_by_name_local_resolve(name, 0b0000_0001)` | per-entry probe | WIRED | `hp41-cli/tests/key_coverage.rs:284` — BL-04 closure |

## Gaps Summary

**No gaps.** The prior verification's single gap (BL-04 — `key_coverage.rs` narrow accessor) is closed and verified:

1. Import widened at `hp41-cli/tests/key_coverage.rs:19` to include `help_entries_all`.
2. Main probe loop migrated at `hp41-cli/tests/key_coverage.rs:126` to iterate the merged pool.
3. Probed-entries floor raised from `>= 50` to `>= 95` at `hp41-cli/tests/key_coverage.rs:245`.
4. Math1-specific closure sub-loop added at `hp41-cli/tests/key_coverage.rs:257-299` — every `xrom.module_id == 7` entry with a non-null `key_path` is probed through `xeq_by_name_local_resolve(name, 0b0000_0001)` and must resolve to `Some(_)`. A second floor `math1_probed >= 40` (line 294) detects vacuous-pass regressions where the Math1 pool fails to load.

The test passes (`cargo test --test key_coverage` → 1/1) and gate runs (`just test` and `just lint`) both exit 0. Closure commit: `ce19010` (2026-05-17, part of the 15-commit `/gsd-code-review --fix` workflow batch e071a88…6af997f).

**Background context — code-review fixes shipped between verifications:** 14 additional fix commits surfaced during the same review window addressed Phase 28 kernel correctness gaps (CR-01..04, BL-01..03) and minor Phase 29 wiring polish (WR-01..09). These are tracked in `29-REVIEW-FIX.md` (`status: all_fixed`, 17/17 in-scope). They are NOT Phase 29 must-haves but are observed to be non-regressive: `just test` exits 0 with all suites green and `just lint` exits 0 with no warnings against the post-fix tree.

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none in Phase 29 modified files) | — | — | — | The prior verification's WR-05 dead `unwrap_or` warning at `hp41-cli/src/ui.rs:280` is resolved — commit `a9dcf09` replaced it with an explicit `if let Some(mp) = modal_prompt` pattern. Current line 284 carries the fix. |

**Debt-marker scan:** A single `TBD` marker exists at `hp41-cli/src/keys.rs:260` ("…HP-41CV reference card positions for VIEW/ARCL/ASTO/ISG/DSE are TBD per RESEARCH; Plan 04 may move these onto numeric f-shift positions…"). `git blame` attributes it to commit `f97c1921` (2026-05-14, Phase 25-02), pre-dating Phase 29 by three days. The marker is in a file Phase 29 modified (signature widening at line 358), but the marker line itself is unchanged Phase-25 documentation about a v2.2 keybinding choice and references `Plan 04` as the formal follow-up locus. Per the verifier debt-marker gate ("Any `TBD`, `FIXME`, or `XXX` marker in a file modified by this phase is a BLOCKER unless the same line references formal follow-up work"), the explicit `Plan 04 may…` clause is the referenced follow-up. Not a Phase 29 BLOCKER.

No `FIXME` or `XXX` markers found in Phase 29 modified files.

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Test suite green | `just test` | exit 0; ~30 test binaries, zero failures, zero ignored across hp41-core and hp41-cli | PASS |
| Lint green | `just lint` | exit 0; `cargo clippy --workspace --all-targets --all-features -- -D warnings` produces no output | PASS |
| BL-04 closure test | implicit in `just test` — `Running tests/key_coverage.rs` line confirms `key_coverage_implemented_entries_dispatch` ran and passed | 1 passed | PASS |
| Phase 29 test files all green | implicit in `just test` — `phase29_help_data_math1` (10), `phase29_key_ref_includes_math1` (3), `phase29_modal_flow` (5), `phase29_pending_prompt_modal` (3) | 21/21 passed | PASS |
| SC-4 strict invariant grep | `grep -rn "fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)" hp41-gui/src-tauri/src/` | zero matches | PASS |
| JSON entry count | `python3 -c "import json; print(len(json.load(open('docs/hp41-math1-functions.json'))))"` | `45 entries` | PASS |
| prgm_display arm parity | `grep -cE "^\s*(Op::[A-Z][A-Za-z0-9_]+\s*(\(.*\))?\s*=>)"` on CLI + GUI copies | 170 = 170 | PASS |

## Human Verification Required

The two human-verification items from the prior verification REMAIN OPEN — they exercise interactive TUI behavior that no automated test or grep can cover. They are unchanged by the BL-04 closure and the surrounding 14 fix commits.

### 1. Interactive MATRIX Modal Flow in TUI

**Test:** Launch `cargo run -p hp41-cli`, press `X` to open the XEQ-by-name modal, type `MATRIX` and Enter. Then press `2` followed by F5 (R/S). Then type matrix values and press F5 between entries.
**Expected:** Status bar shows `ORDER=?` after `MATRIX` dispatches; advances to `A1,1=?` after the first F5; continues through matrix entry. Pressing Esc shows `Cancelled`.
**Why human:** TUI rendering with ratatui cannot be verified programmatically; unit tests cover the App state machine but not visual output.

### 2. Interactive SOLVE Alpha-Label Collection

**Test:** Launch `cargo run -p hp41-cli`, press `X`, type `SOLVE` and Enter. Verify status bar reads `FUNCTION NAME?` and an XEQ-name-collection mode auto-opens. Type a label such as `F` and Enter. Press F5 twice (for Guess 1 and Guess 2) with a user program containing label `F` loaded.
**Expected:** Status bar shows `FUNCTION NAME?`; after `F` + Enter shows `GUESS 1=?`; after F5 shows `GUESS 2=?`; after second F5 the solver runs and the result appears in the print panel.
**Why human:** Full Solve workflow end-to-end requires a user program in memory and live TUI interaction. The auto-open of `XeqByNameMode::CollectForModal` involves App state-machine timing that unit tests cover (see `phase29_modal_flow.rs::auto_open_collect_for_modal_on_function_name_prompt`) but visual confirmation is only possible in a live terminal.

---

_Verified: 2026-05-17T15:30:00Z (re-verification)_
_Verifier: Claude (gsd-verifier)_
_Previous: 2026-05-17T12:06:12Z (gaps_found, 4/5)_
