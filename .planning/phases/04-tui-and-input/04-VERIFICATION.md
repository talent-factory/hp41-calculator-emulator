---
phase: 04-tui-and-input
verified: 2026-05-07T12:00:00Z
status: human_needed
score: 3/4 must-haves verified
overrides_applied: 0
gaps:
  - truth: "User can perform any documented calculator operation using only the physical keyboard — discoverable key labels are visible in the TUI"
    status: partial
    reason: "EEX ('e' key) is documented in KEY_REF_TABLE and accepted by handle_key, but rust_decimal::Decimal::from_str does not parse scientific-notation strings (e.g. '1.5e3'), so pressing 'e' then a non-digit op silently returns HpError::InvalidOp and discards entered digits. EEX entry is non-functional. Additionally, entry_buf accepts structurally invalid sequences (multiple '.', multiple 'e', leading 'e') with no guard, all of which fail flush_entry_buf. These are code-review findings CR-001 and CR-002."
    artifacts:
      - path: "hp41-cli/src/app.rs"
        issue: "handle_key appends 'e' to entry_buf unconditionally; no guard for empty buf, no guard for duplicate '.'/e; leads to parse failure on next op"
      - path: "hp41-core/src/ops/mod.rs"
        issue: "flush_entry_buf uses Decimal::from_str which does not parse scientific notation; Decimal::from_scientific is not called as fallback"
    missing:
      - "In flush_entry_buf: add .or_else(|_| Decimal::from_scientific(&s)) fallback"
      - "In handle_key entry_buf append: guard 'e' so it requires non-empty buf and no existing 'e'; guard '.' so it requires no existing '.' and no existing 'e'"
human_verification:
  - test: "Verify two-column TUI renders correctly at startup"
    expected: "Left column shows T/Z/Y/X/L labels, all 0.0000; Display panel shows '0.0000'; Annunciator bar shows [USER][PRGM][ALPHA][SHIFT][RAD][DEG][GRAD] with [DEG] bold; Right panel shows key reference table"
    why_human: "Terminal rendering requires human visual inspection; cannot be verified via grep or cargo test"
  - test: "Verify annunciator updates in same frame as mode change"
    expected: "Pressing 'd' toggles angle mode; DEG annunciator dims, RAD becomes bold, all in the same rendered frame with no flicker"
    why_human: "Frame-synchrony is a visual/interaction property; no automated test covers it"
  - test: "Verify panic hook restores terminal (SC-4)"
    expected: "An injected panic in hp41-core causes ratatui::restore() to fire before the panic message prints; terminal is left in normal mode"
    why_human: "Panic path requires a simulated panic in a running process; not testable with cargo test"
---

# Phase 4: TUI & Input — Verification Report

**Phase Goal:** Users interact with the emulator entirely via keyboard in a persistent ratatui terminal panel that shows the 4-level stack, LASTX, 12-character HP-41 display, and all annunciators at all times.
**Verified:** 2026-05-07
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | TUI renders a persistent panel showing X/Y/Z/T, LASTX, 12-char display, and annunciators (USER, PRGM, ALPHA, SHIFT, RAD/DEG/GRAD) without any user action | ? HUMAN NEEDED | `render_ui()` in ui.rs is fully implemented (192 lines) and wired to App::draw(); structure verified in code; visual correctness requires human |
| 2 | Annunciator state updates immediately when calculator mode changes (same-frame) | ? HUMAN NEEDED | `render_annunciators()` reads `st.prgm_mode`, `st.alpha_mode`, `st.angle_mode` directly from CalcState; ratatui redraws before polling for next event (draw-first loop); same-frame semantics require human observation |
| 3 | User can perform any documented calculator operation using only the physical keyboard — discoverable key labels visible in TUI | PARTIAL (WARNING) | 25 Op bindings implemented in key_to_op(); KEY_REF_TABLE has 33 entries shown in right panel; HOWEVER EEX ('e') is documented but non-functional: Decimal::from_str rejects scientific-notation strings (CR-001); entry_buf also accepts invalid multi-dot/multi-e sequences (CR-002) |
| 4 | Any unhandled panic in hp41-core is caught at CLI boundary — terminal restored to normal | ? HUMAN NEEDED | `ratatui::init()` is used (installs panic hook); `ratatui::restore()` is called explicitly after run(); code path is correct per plan; panic-path correctness requires a live injected panic to confirm |

**Score:** 3/4 truths verified at code level (1 partial, 3 need human)

The partial truth (SC-3) is documented code-review finding CR-001/CR-002. It does not block basic calculator use — integer and decimal entry works correctly — but EEX entry fails silently.

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-cli/Cargo.toml` | ratatui 0.30, crossterm 0.29, clap 4.x | VERIFIED | Lines 12-14 confirm all three dependencies at correct versions |
| `hp41-cli/src/main.rs` | Entry point with ratatui::init(), restore(), clap, 5 mod declarations | VERIFIED | 51 lines; `Cli::parse()` before `ratatui::init()`; all 5 mod declarations; `App::new().run(terminal)` wired |
| `hp41-cli/src/app.rs` | App struct + run() poll loop + handle_key() + call_dispatch() | VERIFIED | 147 lines; struct App with CalcState; poll(16ms) loop; KeyEventKind::Press filter as first check in handle_key; digit-direct-append; call_dispatch() wrapping hp41_core::ops::dispatch |
| `hp41-cli/src/ui.rs` | Full two-column widget layout replacing stub | VERIFIED | 192 lines; render_ui(), render_left_panel(), render_right_panel(), render_stack(), render_display(), render_annunciators(), render_status() all present; Block::bordered().title_top() API used throughout |
| `hp41-cli/src/keys.rs` | Full key_to_op() + KEY_REF_TABLE with 33 entries | VERIFIED | 97 lines; 25 Op bindings returned; KEY_REF_TABLE const with 33 entries confirmed by unit test |
| `hp41-cli/src/prgm_display.rs` | format_step() + exhaustive op_display_name() | VERIFIED | 92 lines; format_step() reads state.program.get(state.pc); op_display_name() covers all 35 Op variants exhaustively |
| `hp41-cli/src/tests/keys_tests.rs` | Unit tests for key_to_op() | VERIFIED | 100 lines; 8 test functions covering all documented mappings; key_ref_table_has_33_entries assertion |
| `hp41-cli/src/tests/prgm_display_tests.rs` | Unit tests for format_step() | VERIFIED | 64 lines; 6 test functions including empty program, step ops, zero-padding, and pc-beyond-program |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `hp41-cli/src/main.rs` | `hp41-cli/src/app.rs` | `App::new().run(terminal)` | VERIFIED | Line 43: `let result = App::new().run(terminal);` |
| `hp41-cli/src/app.rs` | `hp41-core::ops::dispatch` | `call_dispatch(op)` → `hp41_core::ops::dispatch(&mut self.state, op)` | VERIFIED | Lines 141-145: call_dispatch wraps dispatch; all op paths route through it |
| `hp41-cli/src/app.rs` | `hp41-cli/src/keys.rs` | `keys::key_to_op(key, self)` | VERIFIED | Line 135: `if let Some(op) = keys::key_to_op(key, self)` |
| `hp41-cli/src/app.rs` | `hp41-core::run_program` | F5 → `hp41_core::run_program(&mut self.state, "A")` | VERIFIED | Lines 127-130: F5 dispatches run_program and maps error to self.message |
| `hp41-cli/src/ui.rs` | `hp41-core::format_hpnum` | `format_hpnum(&st.stack.x, &st.display_mode)` | VERIFIED | Lines 88, 129: format_hpnum used in render_stack and get_display_string |
| `hp41-cli/src/ui.rs` | `hp41-cli/src/prgm_display::format_step` | `prgm_display::format_step(st)` when prgm_mode | VERIFIED | Line 123: call site inside get_display_string prgm_mode branch |
| `hp41-cli/src/ui.rs` | `hp41-cli/src/keys::KEY_REF_TABLE` | `KEY_REF_TABLE.iter()` in render_right_panel | VERIFIED | Lines 19 (import) and 182 (usage): KEY_REF_TABLE iterated and rendered |
| `hp41-cli/src/prgm_display.rs` | `hp41-core::CalcState.program` | `state.program.get(state.pc)` | VERIFIED | Line 15-18: .get(step_num).map(op_display_name) |
| `hp41-core/src/ops/mod.rs` | `rust_decimal::Decimal::from_str` | `Decimal::from_str(&s)` in flush_entry_buf | PARTIAL — EEX BROKEN | Line 164: from_str only; from_scientific fallback absent; CR-001 |

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|--------------------|--------|
| `ui.rs` render_stack | `st.stack.{x,y,z,t,lastx}` | CalcState owned by App, mutated by dispatch() | Yes — dispatch writes stack registers | FLOWING |
| `ui.rs` render_display | `entry_buf` / `prgm_mode` / `alpha_mode` / `stack.x` | CalcState fields, directly read | Yes — real CalcState fields | FLOWING |
| `ui.rs` render_annunciators | `prgm_mode`, `alpha_mode`, `angle_mode` | CalcState fields | Yes — real CalcState fields | FLOWING |
| `ui.rs` render_right_panel | `KEY_REF_TABLE` | Const slice in keys.rs | Yes — 33 static entries | FLOWING |
| `ui.rs` render_status | `app.message` | Set by call_dispatch() error path | Yes — real HpError messages | FLOWING |

---

### Behavioral Spot-Checks

Step 7b is SKIPPED for the TUI-rendering behaviors (requires running terminal). Automated checks that can be performed:

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Zero build errors | `cargo build -p hp41-cli` (exit 0) | 0 error lines | PASS |
| All workspace tests pass | `cargo test --workspace` | 295 passed (15 suites) | PASS |
| hp41-cli unit tests pass | `cargo test -p hp41-cli` | 15 passed (1 suite) | PASS |
| KEY_REF_TABLE has 33 entries | test `key_ref_table_has_33_entries` | asserted in test suite | PASS |
| key_to_op returns correct Op for Enter | test `enter_maps_to_op_enter` | passes | PASS |
| format_step returns "000 END" for empty program | test `empty_program_shows_end` | passes | PASS |
| EEX entry commits number via from_scientific | manual / unit test | NOT TESTED — from_scientific fallback absent | FAIL |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| DISP-01 | 04-01, 04-02, 04-04, 04-05 | 12-char display with annunciators in TUI | VERIFIED (code) / HUMAN for visual | ui.rs render_display shows entry_buf / prgm step / alpha / format_hpnum(X); annunciator bar with all 7 indicators; human needed for visual confirmation |
| DISP-02 | 04-01, 04-02, 04-03, 04-04, 04-05 | Persistent stack panel (X/Y/Z/T, LASTX, annunciators, display) | VERIFIED (code) / HUMAN for visual | render_stack renders T/Z/Y/X/LASTX via format_hpnum; two-column layout always rendered; human needed for visual confirmation |
| INPUT-01 | 04-01, 04-03, 04-04, 04-05 | All calculator functions accessible from keyboard, key reference visible | PARTIAL | 25 Op bindings + KEY_REF_TABLE in right panel; EEX non-functional (CR-001); basic ops fully functional |

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `hp41-cli/src/app.rs` | 81 | `c == 'e'` appended to entry_buf with no validity guard | WARNING (CR-002) | User can produce "e", "1e2e3", "1.2.3" — all fail parse; digits silently lost |
| `hp41-core/src/ops/mod.rs` | 164 | `Decimal::from_str` without `from_scientific` fallback | WARNING (CR-001) | EEX entry always fails; any entry_buf containing 'e' returns InvalidOp |
| `hp41-cli/src/app.rs` | 68 | `'q'` quit check is unconditional before alpha_mode check | INFO (WR-001) | No Phase 4 impact; will break ALPHA-mode text entry in Phase 5 |
| `hp41-cli/src/prgm_display.rs` | 16 | format_step uses 0-based pc as step number | INFO (WR-002) | HP-41 hardware is 1-based ("001" for first instruction); current output is "000" |

No placeholder comments, empty return stubs, or disconnected props found. No `TODO`/`FIXME` in source files (Phase 5 placeholders are in comments, not code paths).

---

### Human Verification Required

#### 1. Two-Column TUI Visual Rendering (DISP-01 / DISP-02)

**Test:** Run `cargo run -p hp41-cli` in an 80×24+ terminal. Verify at startup without pressing any key.

**Expected:**
- Left column: T/Z/Y/X labels each on own line, all showing 0.0000; LASTX below showing 0.0000; Display panel with border and " Display " title showing "0.0000"; Annunciator bar showing [USER][PRGM][ALPHA][SHIFT][RAD][DEG][GRAD] with [DEG] bold; Status bar showing "Ready"
- Right column: bordered panel titled " Keys " with the 33-entry key reference table

**Why human:** Terminal rendering and visual layout cannot be verified programmatically.

#### 2. Same-Frame Annunciator Update (DISP-02 SC-2)

**Test:** With TUI running, press `d` three times (DEG → RAD → GRAD → DEG).

**Expected:** Each keystroke immediately updates the annunciator bar in the same frame — no flicker, no frame delay. Bold marker moves: [DEG] → [RAD] → [GRAD] → [DEG].

**Why human:** Frame-synchrony is a visual/temporal property not testable by grep or unit tests. Already verified in 04-04 human smoke test, but required for formal gate.

#### 3. Panic Hook Terminal Restoration (SC-4)

**Test:** Inject an intentional panic in a dev build and confirm terminal restores.

**Expected:** Terminal returns to normal shell mode after panic; no raw mode artifacts left.

**Why human:** Requires a live process with an injected panic; not feasible in unit test framework.

---

### Gaps Summary

**One WARNING gap (not a full blocker for basic operation):**

**EEX entry is non-functional (CR-001 + CR-002).** The 'e' key is correctly documented in KEY_REF_TABLE as "EEX (sci notation entry)" and is correctly accepted by handle_key into entry_buf. However, `rust_decimal::Decimal::from_str` — used exclusively in `flush_entry_buf` — does not parse scientific-notation strings such as "1.5e3". The `from_scientific` method exists in rust_decimal but is not called. Any entry_buf containing 'e' will return `HpError::InvalidOp` and silently discard all entered digits. Additionally, no guard prevents structurally invalid sequences like "1.2.3" or "1e2e3" from being built in entry_buf, all of which also fail parse.

This does not prevent arithmetic, trig, stack operations, PRGM mode, or any other documented operation — only scientific-notation number entry is broken. The 17-check human smoke test from Plan 04-04 did not include an EEX test case, so this went undetected.

**Three human-verification items remain** covering visual rendering, same-frame annunciator updates, and panic-hook restoration. These were all reported as passing in the Plan 04-04 smoke test (all 17 checks passed), but the formal verification gate requires explicit human sign-off.

---

_Verified: 2026-05-07_
_Verifier: Claude (gsd-verifier)_
