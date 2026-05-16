---
phase: 25-cli-integration-and-documentation
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - hp41-cli/src/app.rs
  - hp41-cli/src/keys.rs
  - hp41-cli/src/ui.rs
  - hp41-cli/tests/phase25_keyboard.rs
autonomous: true
requirements:
  - FN-CLI-01
  - FN-TEST-01
user_setup: []
tags:
  - cli
  - keyboard
  - state-machine

must_haves:
  truths:
    - "Pressing 'f' in normal TUI mode arms a one-shot prefix; status bar annunciator shows SHIFT highlighted (and 'f→' indicator)"
    - "After 'f' is armed, pressing '-' dispatches Op::Test(TestKind::XEqY); '+' dispatches XLeY; '*' dispatches XGtY; '/' dispatches XEqZero"
    - "After arming, any single key (recognized or not) consumes the prefix — shift_armed returns to false at end of branch"
    - "Esc inside an armed state cancels the prefix without dispatching"
    - "In ALPHA mode, pressing 'f' types a literal 'F' character (D-25.5 divergence preserved) — prefix arming does NOT trigger"
    - "Inside an active modal (pending_input.is_some()), 'f' is consumed by the modal — global prefix arming does NOT trigger (Pitfall 4 — by design)"
    - "Every v1.x letter binding listed in Pitfall 3 + D-25.3 is removed from key_to_op() in hp41-cli/src/keys.rs"
    - "KEY_REF_TABLE in hp41-cli/src/keys.rs is left UNTOUCHED in Plan 01 (with a `#[deprecated]` or TODO comment cross-linking to Plan 04) — the canonical FN-CLI-01 'explicit table' is the JSON-derived `help_data.rs::help_entries()` per D-25.18 (added 2026-05-14 in CONTEXT.md); Plan 04 Task 3 finalizes the regeneration-or-deletion of the table from the JSON canonical source"
  artifacts:
    - path: "hp41-cli/src/app.rs"
      provides: "App.shift_armed: bool field + initialization + arming logic in handle_key()"
      contains: "shift_armed"
    - path: "hp41-cli/src/keys.rs"
      provides: "Stripped key_to_op() (HP-41CV primary positions only) + new shifted_key_to_op() (4 hardware-anchored conditional tests)"
      contains: "shifted_key_to_op"
    - path: "hp41-cli/src/ui.rs"
      provides: "Status-bar annunciator wired to app.shift_armed; optional 'f→' inline indicator"
      contains: "app.shift_armed"
    - path: "hp41-cli/tests/phase25_keyboard.rs"
      provides: "Integration tests for f-prefix arming + 4 conditional test dispatches + ALPHA override + Esc cancel + Pitfall 5 bleed-prevention"
      contains: "f_minus_dispatches_x_eq_y"
  key_links:
    - from: "hp41-cli/src/app.rs::handle_key"
      to: "hp41-cli/src/keys.rs::shifted_key_to_op"
      via: "shift_armed==true branch dispatches via shifted_key_to_op(key, self)"
      pattern: "shifted_key_to_op"
    - from: "hp41-cli/src/ui.rs::render_status (annunciator bar)"
      to: "App.shift_armed"
      via: "ann(\"SHIFT\", app.shift_armed) replaces hardcoded false at ui.rs:212"
      pattern: "ann\\(\"SHIFT\""
---

<objective>
Introduce a true HP-41CV one-shot f-prefix shift state machine to hp41-cli per D-25.1 (prefix supersedes v1.x letter-direct mapping) and D-25.2 (ONE yellow prefix key only — no `g`-prefix, hardware-correct for HP-41C/CV/CX), deprecate all v1.x crossterm letter-direct-dispatch bindings (D-25.3), and wire the 4 hardware-anchored conditional tests (X=Y, X≤Y, X>Y, X=0) to the f-shifted arithmetic keys (D-25.7).

Purpose: This plan is the foundation for every subsequent Phase 25 task. The prefix state machine is consumed by the modal architecture (Plan 02 — IND-toggle uses the same shift_armed bit) and by the docs/JSON pipeline (Plan 04 — key_path field of every JSON entry references the f-prefix). Per D-25.6, CLI parity with GUI v2.1 shiftActive is invariant: every behavior here must mirror hp41-gui/src/App.tsx (one-shot lifetime, ALPHA override, Esc cancel).

Output: Modified App struct with shift_armed: bool, rewritten keys.rs (HP-41CV primary positions only + new shifted_key_to_op), status-bar wired to shift_armed, and Wave-0 integration tests asserting all 4 f-arith conditional-test dispatches.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/25-cli-integration-and-documentation/25-CONTEXT.md
@.planning/phases/25-cli-integration-and-documentation/25-RESEARCH.md
@.planning/phases/25-cli-integration-and-documentation/25-PATTERNS.md
@CLAUDE.md

<interfaces>
<!-- Key types and contracts for this plan — sourced from the existing codebase. -->
<!-- Executor uses these directly; no codebase exploration needed. -->

From hp41-core/src/ops/mod.rs (Op + discriminator enums — DO NOT MODIFY in this plan):
- `pub enum Op { Add, Sub, Mul, Div, Enter, Clx, Chs, Rdn, Rup, XySwap, Lastx, ... Test(TestKind), ... }` — 130 total variants (post-Phase 24)
- `pub enum TestKind { XEqY, XNeY, XLtY, XGtY, XLeY, XGeY, XEqZero, XNeZero, XLtZero, XGtZero, XLeZero, XGeZero }` — Phase 3, 12 variants

From hp41-cli/src/app.rs (App struct — existing fields, EXTEND with shift_armed):
- `pub struct App { pub state: CalcState, pub message: Option<String>, pub exit: bool, pub pending_input: Option<PendingInput>, ... }` (lines 43–74)
- `fn handle_key(&mut self, key: KeyEvent)` — main dispatch (lines ~181+); has these guards in order:
  1. `if key.kind != KeyEventKind::Press { return }` — Windows double-event filter (line ~183)
  2. last_key_code tracking
  3. Ctrl+C / Ctrl+S / help / Ctrl+P
  4. pending_input route (line ~228) — Pitfall 4 sacred ordering
  5. modal openers (S/R/Ctrl+A)
  6. ALPHA mode block (line ~299–302)
  7. USER mode F1–F4
  8. digit/'.'/'e'/'n'
  9. F5/F7/F8
  10. key_to_op() fallthrough (line ~467 includes the v1.x `f` FmtDigits cycle — REMOVE per Pitfall 3)

From hp41-cli/src/keys.rs (lines 18–87 — current signature, PRESERVE):
- `pub fn key_to_op(key: KeyEvent, _app: &App) -> Option<Op>` — returns None for unmapped keys
- v1.x letter bindings to STRIP per D-25.3: `'C'`→Cos, `'T'`→Tan, `'L'`→Ln, `'G'`→Log, `'E'`→Exp, `'H'`→TenPow, `'I'`→Recip, `'W'`→Sq, `'Y'`→YPow, `'q'`→Sin, `'a'`→Asin, `'c'`→Acos, `'k'`→Atan, `'s'`→Sqrt, `'g'`→Clreg, `'z'`/`'Z'`→SigmaPlus/Minus, `'m'`→Mean, `'D'`→Sdev, `'y'`→Yhat, `'b'`→LR, `'O'`→Corr, `'V'`→ClSigmaStat, `'h'`→HmsToH, `'j'`→HmsAdd, `'J'`→HmsSub
- Lines KEEP (real HP-41 primary positions or universal): `KeyCode::Enter` → Op::Enter; `KeyCode::Backspace` → Op::Clx; `+/-/*//` → Add/Sub/Mul/Div; `'n'` → Op::Chs; `'r'` → Op::Rdn; `'x'` → Op::XySwap; `'l'` → Op::Lastx; `'p'` → Op::PrgmMode; `'u'` → Op::UserMode; `'%'` → Op::PctChange; `'S'/'R'/F1..F8` → None (handled elsewhere)

From hp41-cli/src/ui.rs (line 212 — single-line edit):
- Annunciator bar literal: `ann("SHIFT", false)` — replace with `ann("SHIFT", app.shift_armed)`

From hp41-gui/src/App.tsx (parity reference for D-25.6 — DO NOT modify in this plan, READ ONLY):
- `const [shiftActive, setShiftActive] = useState(false);` (line ~111)
- One-shot pattern: `if (consumesShift) setShiftActive(false);` in finally block (line ~206)

</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add App.shift_armed field + arming/consumption logic in handle_key()</name>
  <files>hp41-cli/src/app.rs</files>
  <read_first>
    - hp41-cli/src/app.rs (full file — App struct definition lines 43–74, App::new lines ~120–133, handle_key body to locate the pending_input route at line ~228, ALPHA-mode block at lines ~299–302, and the v1.x 'f' FmtDigits cycle block to REMOVE — RESEARCH cites lines ~466–475)
    - hp41-gui/src/App.tsx (lines 111–119, 161–206 — shiftActive parity reference per D-25.6)
    - .planning/phases/25-cli-integration-and-documentation/25-RESEARCH.md §"Pattern 1: One-Shot Prefix State Machine" + §"Common Pitfalls" 1–5
    - .planning/phases/25-cli-integration-and-documentation/25-PATTERNS.md (`hp41-cli/src/app.rs — App.shift_armed: bool + arming logic` section)
    - CLAUDE.md (settled Phase 4 trap: "ratatui::init() / Windows KeyEventKind::Release filter must run FIRST")
  </read_first>
  <behavior>
    - shift_armed defaults to false on App::new
    - Pressing 'f' (KeyCode::Char('f'), no Ctrl) sets shift_armed=true; clears app.message; returns without further dispatch
    - When shift_armed==true on the NEXT handle_key invocation: dispatch via keys::shifted_key_to_op(key, self) if Some(op); set shift_armed=false UNCONDITIONALLY at end of branch (Pitfall 5)
    - Esc while shift_armed==true clears the prefix without dispatching
    - In ALPHA mode (self.state.alpha_mode==true), 'f' bypasses the arming check and is routed to ALPHA-mode handling (literal F character) — Pitfall 2 (D-25.5)
    - Inside an active modal (self.pending_input.is_some()), 'f' is consumed by pending_input route — global prefix arming does NOT activate (Pitfall 4 — INTENDED behavior)
    - On Windows, KeyEventKind::Release events do NOT advance shift_armed state (Pitfall 1 — the existing line ~183 filter handles this)
    - The v1.x 'f' FmtDigits cycle block is REMOVED in the same commit (Pitfall 3) — FmtDigits modal now reachable only via the future f-shifted DISP key position (wired in Plan 02 / 04)
  </behavior>
  <action>
    Add field `pub shift_armed: bool` to the App struct in hp41-cli/src/app.rs (insert after `pub pending_input: Option<PendingInput>` near line 54). Initialize to `false` in App::new (near line 120–133). In `handle_key`, insert the f-prefix arming check AFTER the pending_input route (line ~228) AND AFTER the ALPHA-mode routing block (line ~299–302) — ordering is non-negotiable per Pitfall 2 + Pitfall 4. Logic shape:

    1. If `!self.shift_armed && key.code == KeyCode::Char('f') && !key.modifiers.contains(KeyModifiers::CONTROL)`: set `self.shift_armed = true`, set `self.message = None`, `return`.
    2. If `self.shift_armed`: handle Esc as cancel (set `shift_armed = false`, return). Otherwise call `keys::shifted_key_to_op(key, self)`; if `Some(op)` dispatch via `self.call_dispatch(op)`. ALWAYS set `self.shift_armed = false` at end of branch (Pitfall 5) regardless of outcome. Return.

    REMOVE the v1.x 'f' FmtDigits cycle block (RESEARCH cites lines ~466–475 in current app.rs) atomically in the SAME commit. The FmtDigits modal continues to be reachable via the `F` (uppercase) FmtDigits opener path, which Plan 02/04 will reposition; this plan only removes the v1.x 'f' direct-cycle binding per D-25.3 + Pitfall 3.

    Use `.expect("…")` not `.unwrap()`. No code blocks in this prose — refer to <interfaces> for the exact identifier and line context.
  </action>
  <verify>
    <automated>cargo test -p hp41-cli --test phase25_keyboard -- test_shift_armed_one_shot test_shift_armed_esc_cancel test_shift_armed_alpha_override test_shift_armed_pitfall5_bleed</automated>
  </verify>
  <acceptance_criteria>
    - `grep -n "pub shift_armed: bool" hp41-cli/src/app.rs` returns exactly 1 line in the App struct (not in PendingInput, not in a sub-struct)
    - `grep -n "shift_armed = true" hp41-cli/src/app.rs` matches inside `handle_key` AFTER both the pending_input route and the ALPHA-mode block (verify by line-number ordering against grep on `pending_input.is_some` and `alpha_mode`)
    - `grep -n "shift_armed = false" hp41-cli/src/app.rs` shows clears on BOTH the Esc-cancel path AND the end-of-shift-armed-branch (Pitfall 5)
    - The v1.x 'f' FmtDigits cycle block previously at app.rs ~466–475 is REMOVED: `grep -n "DisplayMode::Fix.*current_digits\|FmtDigits.*current_digits" hp41-cli/src/app.rs | grep -v "^#" | grep -c "" == 0` for that specific cycle-on-f pattern (the F-uppercase FmtDigits OPENER stays)
    - `cargo build -p hp41-cli` compiles with zero warnings
    - `cargo clippy -p hp41-cli -- -D warnings` passes
  </acceptance_criteria>
  <done>
    App carries `shift_armed: bool`; handle_key correctly arms on 'f' (outside ALPHA, outside modals) and consumes on next key with unconditional clear; v1.x 'f' FmtDigits cycle is gone; Wave-0 tests for arming/consumption/Esc/Pitfall-5 are GREEN.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Rewrite key_to_op() to HP-41CV primaries only + add shifted_key_to_op() with 4 f-arith conditional tests</name>
  <files>hp41-cli/src/keys.rs, hp41-cli/src/ui.rs</files>
  <read_first>
    - hp41-cli/src/keys.rs (full file — current key_to_op at lines 18–87 plus KEY_REF_TABLE at line 91+; strip all v1.x letter bindings per D-25.3)
    - hp41-cli/src/ui.rs (lines 200–273 — annunciator bar at line 212 to flip `false` → `app.shift_armed`, render_status at lines 223–234 for optional `f→` indicator)
    - hp41-gui/src/Keyboard.tsx (KEY_DEFS lines 23–94 — structural reference for primary/shifted/alphaChar layout per D-25.6)
    - .planning/phases/25-cli-integration-and-documentation/25-RESEARCH.md §"HP-41CV Keyboard Reference" + §"Conditional tests — keyboard vs XEQ-by-Name"
    - .planning/phases/25-cli-integration-and-documentation/25-PATTERNS.md (`hp41-cli/src/keys.rs (controller, request-response)` section)
  </read_first>
  <behavior>
    - key_to_op returns Op for ONLY: Enter→Enter, Backspace→Clx, +→Add, -→Sub, *→Mul, /→Div, %→PctChange, 'n'→Chs, 'r'→Rdn, 'x'→XySwap, 'l'→Lastx, 'p'→PrgmMode, 'u'→UserMode. Every other v1.x letter binding (C/T/L/G/E/H/I/W/Y/q/a/c/k/s/g/z/Z/m/D/y/b/O/V/h/j/J) is REMOVED.
    - shifted_key_to_op(key, _app) returns Some(Op::Test(TestKind::XEqY)) for `KeyCode::Char('-')`, Some(XLeY) for `'+'`, Some(XGtY) for `'*'`, Some(XEqZero) for `'/'`. Returns None for every other key (modal-opener f-shifted bindings — e.g. SF/CF/VIEW — are added in Plan 02).
    - Annunciator bar at ui.rs:212 reads `app.shift_armed` (no longer hardcoded `false`).
    - Optional improvement (recommended per RESEARCH Open Q 5): render_status prepends `"f→ "` to the status text when `app.shift_armed && app.pending_input.is_none() && !app.state.alpha_mode`.
  </behavior>
  <action>
    Rewrite hp41-cli/src/keys.rs::key_to_op to KEEP only the HP-41CV primary positions and universal control keys listed in <interfaces> (and in <behavior> above). DELETE every v1.x letter binding listed under "v1.x letter bindings to STRIP" in <interfaces>. Preserve the signature `pub fn key_to_op(key: KeyEvent, _app: &App) -> Option<Op>`.

    Add new function `pub fn shifted_key_to_op(key: KeyEvent, _app: &App) -> Option<Op>` adjacent to key_to_op. Body: match `key.code` arms for the 4 f-arith conditional tests per D-25.7 (KeyCode::Char('-') → Op::Test(TestKind::XEqY); '+' → XLeY; '*' → XGtY; '/' → XEqZero). Default arm `_ => None`. Import TestKind via `use hp41_core::ops::{Op, TestKind};` if not already present.

    Defer the rest of the f-shifted keyboard map (modal-opener f-shifted bindings — SF/CF/VIEW/TONE/etc.) to Plan 02 — Plan 01's `shifted_key_to_op` returns None for everything except the 4 conditional tests. KEY_REF_TABLE rewrite is also deferred to Plan 04 (where the JSON pipeline becomes the source of truth) — leave KEY_REF_TABLE untouched for now, but add a `#[deprecated(note = "v1.x letter conventions — rewritten in Plan 04 from docs/hp41cv-functions.json")]` attribute or a TODO comment cross-linking to Plan 04 above the table.

    In hp41-cli/src/ui.rs: change line 212 from `ann("SHIFT", false)` to `ann("SHIFT", app.shift_armed)`. In render_status (lines 223–234), if app.shift_armed is true AND there's no pending_input AND not in alpha mode, prepend `"f\u{2192} "` to the displayed text. Use string concatenation on the existing `text` binding before passing to Paragraph::new.

    Use `.expect("…")` not `.unwrap()`. No fenced code blocks here — see <interfaces> for the literal strings.
  </action>
  <verify>
    <automated>cargo test -p hp41-cli --test phase25_keyboard -- f_shifted_conditionals key_to_op_v1x_letters_removed && cargo clippy -p hp41-cli -- -D warnings</automated>
  </verify>
  <acceptance_criteria>
    - `grep -nE "KeyCode::Char\\('(C|T|L|G|E|H|I|W|Y|q|a|c|k|s|g|z|Z|m|D|y|b|O|V|h|j|J)'\\)" hp41-cli/src/keys.rs | grep -v '^[^:]*:[[:space:]]*//' | wc -l` returns 0 (all v1.x letter bindings stripped, comments allowed)
    - `grep -n "fn shifted_key_to_op" hp41-cli/src/keys.rs` returns exactly 1 line
    - `grep -n "Op::Test(TestKind::XEqY)" hp41-cli/src/keys.rs` matches inside `shifted_key_to_op`'s match arm for `'-'`
    - `grep -n "Op::Test(TestKind::XLeY)" hp41-cli/src/keys.rs` matches inside shifted_key_to_op for `'+'`
    - `grep -n "Op::Test(TestKind::XGtY)" hp41-cli/src/keys.rs` matches inside shifted_key_to_op for `'*'`
    - `grep -n "Op::Test(TestKind::XEqZero)" hp41-cli/src/keys.rs` matches inside shifted_key_to_op for `'/'`
    - `grep -n "ann(\"SHIFT\", app.shift_armed)" hp41-cli/src/ui.rs` returns exactly 1 match (was `ann("SHIFT", false)` before)
    - `cargo build -p hp41-cli` compiles with zero warnings; `cargo clippy -p hp41-cli -- -D warnings` passes
    - Integration tests `f_minus_dispatches_x_eq_y`, `f_plus_dispatches_x_le_y`, `f_star_dispatches_x_gt_y`, `f_slash_dispatches_x_eq_zero` all GREEN
  </acceptance_criteria>
  <done>
    keys.rs key_to_op is stripped to HP-41CV primaries; shifted_key_to_op dispatches the 4 f-arith conditional tests; ui.rs annunciator wired to shift_armed; tests covering all 4 keystroke dispatches GREEN.
  </done>
</task>

<task type="auto">
  <name>Task 3: Wave-0 integration test scaffold for f-prefix + 4 conditional tests + Pitfall 5 bleed-prevention</name>
  <files>hp41-cli/tests/phase25_keyboard.rs</files>
  <read_first>
    - hp41-cli/tests/card_io_tests.rs (lines 1–36 — integration-test scaffold pattern: `#![allow(clippy::unwrap_used)]`, imports, KeyEvent construction helper)
    - hp41-cli/src/app.rs (App::new signature + state_path / print_log constructor args)
    - .planning/phases/25-cli-integration-and-documentation/25-VALIDATION.md §"Wave 0 Requirements" + §"Per-Task Verification Map"
    - .planning/phases/25-cli-integration-and-documentation/25-RESEARCH.md §"Wave 0 Gaps"
  </read_first>
  <action>
    Create hp41-cli/tests/phase25_keyboard.rs with `#![allow(clippy::unwrap_used)]` at module head (test files exempt per CLAUDE.md). Imports: `crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers}`, `hp41_cli::app::App`, `hp41_core::ops::{Op, TestKind}`, `hp41_core::state::CalcState`, `hp41_core::num::HpNum`. Provide a helper `fn key(c: char) -> KeyEvent` that builds `KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty())` (set `kind: KeyEventKind::Press` explicitly via struct literal if `KeyEvent::new` defaults differ — RESEARCH Pitfall 1).

    Write these tests (each is a single `#[test]`):

    1. `test_shift_armed_one_shot` — construct App with stack `y=5, x=5`. Send `key('f')`. Assert `app.shift_armed == true`. Send `key('-')`. Assert `app.shift_armed == false` AND last dispatched op's effect (state.skip_next_step or evaluate_test outcome) corresponds to XEqY behavior.
    2. `test_shift_armed_esc_cancel` — send `key('f')`. Send `KeyEvent::new(KeyCode::Esc, KeyModifiers::empty())`. Assert `app.shift_armed == false` AND no Op was dispatched (stack unchanged).
    3. `test_shift_armed_alpha_override` — set `app.state.alpha_mode = true`. Send `key('f')`. Assert `app.shift_armed == false` (D-25.5 — prefix does NOT arm in ALPHA mode) AND alpha buffer contains 'F'.
    4. `test_shift_armed_pitfall5_bleed` — send `key('f')` (arm), then send `key(';')` (an unmapped key). Assert `app.shift_armed == false` AFTER the second key (Pitfall 5 — one-shot lifetime is "next key cycle", not "next CONSUMED key").
    5. `f_minus_dispatches_x_eq_y` — set stack `y=7, x=7`. Send `key('f')` then `key('-')`. Assert `app.state.skip_next_step == false` (X=Y test passes → no skip). Reset, set `y=7, x=8`. Repeat. Assert `app.state.skip_next_step == true` (X≠Y → skip).
    6. `f_plus_dispatches_x_le_y` — same pattern with `+` and TestKind::XLeY (test passes when X ≤ Y).
    7. `f_star_dispatches_x_gt_y` — same pattern with `*` and TestKind::XGtY.
    8. `f_slash_dispatches_x_eq_zero` — set `x=0`. Send `key('f')` then `key('/')`. Assert no skip. Reset, `x=5`. Repeat. Assert skip_next_step==true.
    9. `key_to_op_v1x_letters_removed` — assert `hp41_cli::keys::key_to_op(KeyEvent::new(KeyCode::Char('C'), KeyModifiers::empty()), &app)` returns None (was Some(Op::Cos) in v1.x). Repeat for at least 5 more removed letters (T, L, G, q, h).

    Use real App construction with a tempdir-backed state_path (mirror card_io_tests.rs scaffold). Drive handle_key via the public `App::handle_key` entry point (NOT raw key_to_op) so the prefix state machine is exercised end-to-end.

    Use `.expect("reason")`. No `.unwrap()` in non-test paths. Test module is `#[allow(clippy::unwrap_used)]` so `.unwrap()` IS permitted inside test bodies.
  </action>
  <verify>
    <automated>cargo test -p hp41-cli --test phase25_keyboard</automated>
  </verify>
  <acceptance_criteria>
    - File hp41-cli/tests/phase25_keyboard.rs exists with `#![allow(clippy::unwrap_used)]` at top
    - At least 9 `#[test]` functions present (`grep -c "^#\\[test\\]" hp41-cli/tests/phase25_keyboard.rs` ≥ 9)
    - All tests pass: `cargo test -p hp41-cli --test phase25_keyboard` exits 0
    - `cargo clippy -p hp41-cli --tests -- -D warnings` passes
  </acceptance_criteria>
  <done>
    Integration test file is the Wave-0 scaffold for FN-CLI-01 (4 keyboard dispatches verified) + FN-TEST-01 (partial — 4 of 12 conditional tests reachable via keyboard) + Pitfall 5 regression coverage; all tests GREEN.
  </done>
</task>

</tasks>

<verification>
- `cargo test -p hp41-cli --test phase25_keyboard` exits 0 with ≥9 tests GREEN.
- `cargo build -p hp41-cli` compiles with zero warnings.
- `cargo clippy -p hp41-cli -- -D warnings` passes.
- `just check` (workspace fmt+clippy+test) GREEN.
- Manual smoke: launch `just run-cli`, press `f`, verify SHIFT annunciator highlights AND `f→` shows in status bar; press `-` → display reflects X=Y test outcome; press `f` then any unrecognized key → annunciator clears.
</verification>

<success_criteria>
- App.shift_armed: bool exists and is correctly armed/consumed/cleared per D-25.4 + D-25.5 + Pitfall 5.
- keys::shifted_key_to_op dispatches all 4 hardware-anchored conditional tests per D-25.7.
- Every v1.x letter binding listed in D-25.3 is REMOVED from keys.rs::key_to_op (Pitfall 3 — v1.x 'f' FmtDigits cycle also removed from app.rs).
- Annunciator bar reads `app.shift_armed` (no longer hardcoded false); optional `f→` indicator shows in status bar when armed (RESEARCH Open Q 5 recommended yes).
- FN-CLI-01 progress: 13 primary HP-41CV positions still wired + 4 new f-shifted conditional tests landed (modal-opener f-shifted bindings come in Plan 02).
- FN-TEST-01 progress: 4 of 12 conditional tests now keyboard-reachable (X=Y / X≤Y / X>Y / X=0); remaining 8 land in Plan 03.
- All Wave-0 tests in hp41-cli/tests/phase25_keyboard.rs GREEN.
</success_criteria>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| crossterm → App.handle_key | KeyEvent stream from terminal; untrusted in theory but TUI input is single-user single-process |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-25-01 | Tampering | shift_armed bleed across handle_key invocations | mitigate | Pitfall 5 guard — unconditional `shift_armed = false` at end of branch regardless of `shifted_key_to_op` outcome; covered by `test_shift_armed_pitfall5_bleed` |
| T-25-02 | Denial of Service | Windows KeyEventKind::Release doubles every key, defeating one-shot | mitigate | Existing `if key.kind != KeyEventKind::Press { return }` filter at app.rs:~183 runs FIRST per Pitfall 1 — no changes needed |
| T-25-03 | Information Disclosure | ALPHA-mode user types 'f' expecting literal F but prefix consumes it | mitigate | Pitfall 2 — arming check placed AFTER the ALPHA-mode block at app.rs:~299; `test_shift_armed_alpha_override` regression test |
| T-25-04 | Denial of Service | Active modal silently swallowed by prefix-arming if ordering wrong | mitigate | Pitfall 4 — arming check placed AFTER pending_input route at app.rs:~228 (modal route is sacred per CR-02); INTENDED behavior, documented in test name |
</threat_model>

<output>
After completion, create `.planning/phases/25-cli-integration-and-documentation/25-01-SUMMARY.md` per execute-plan template — record final shift_armed location, the 4 dispatch verifications, the list of v1.x letter bindings removed (for Plan 04 docs cross-link), and any remaining keys.rs TODOs deferred to Plan 02 or Plan 04.
</output>
