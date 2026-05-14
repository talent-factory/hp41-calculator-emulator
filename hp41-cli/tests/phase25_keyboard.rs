//! Phase 25 / Plan 01 — HP-41CV one-shot f-prefix state machine integration tests.
//!
//! These tests drive `App::handle_key` end-to-end so that the prefix-arm,
//! prefix-consume, ALPHA-override, and Pitfall-5-bleed paths are all
//! exercised through the same dispatcher the live TUI uses.
//!
//! Plan 01 Task 1 lands the arming/consumption logic and these tests.
//! Plan 01 Task 2 extends this file with f-arith dispatch tests and the
//! `v1.x letters removed` regression test.

#![allow(clippy::unwrap_used)]

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

use hp41_cli::app::App;
use hp41_core::state::CalcState;
use hp41_core::HpNum;

// ── Test scaffolding ─────────────────────────────────────────────────────────

/// Build a `KeyEvent::Press` for a plain character key (no modifiers).
///
/// Explicitly sets `kind: KeyEventKind::Press` so the very-first guard in
/// `handle_key` (`if key.kind != KeyEventKind::Press { return }` — Pitfall 1)
/// does not silently drop the test key.
fn key(c: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

/// Build a non-`Char` key event (Esc, Backspace, …) with Press semantics.
fn raw_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

/// Build a fresh App backed by a tempdir so persistence side effects are isolated.
/// The state file is not actually written by these tests (handle_key never auto-saves),
/// but routing the path through a tempdir keeps the tests hermetic.
fn make_app() -> (App, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("tempdir creation must succeed");
    let state_path = tmp.path().join("phase25-test-state.json");
    let app = App::new(CalcState::new(), state_path, None);
    (app, tmp)
}

// ── Task 1: arming + consumption + Esc + ALPHA-override + Pitfall 5 ──────────

/// Task 1 / behavior 1 + 2: pressing 'f' arms the prefix; pressing any op key
/// next consumes it and clears `shift_armed` back to false.
///
/// We use `-` as the next key so this also exercises the Task-2 dispatch path
/// (`f -` → X=Y). Before Task 2, `shifted_key_to_op` is a stub returning None
/// and the prefix is consumed silently — Task 1's invariant under test is just
/// that `shift_armed` returns to false (the core one-shot guarantee).
#[test]
fn test_shift_armed_one_shot() {
    let (mut app, _tmp) = make_app();
    assert!(!app.shift_armed, "shift_armed must default to false");

    app.handle_key(key('f'));
    assert!(
        app.shift_armed,
        "pressing 'f' in normal mode must arm the prefix"
    );

    app.handle_key(key('-'));
    assert!(
        !app.shift_armed,
        "shift_armed must clear after exactly one key cycle (one-shot, D-25.4)"
    );
}

/// Task 1 / behavior 3: Esc while armed cancels the prefix without dispatching.
#[test]
fn test_shift_armed_esc_cancel() {
    let (mut app, _tmp) = make_app();
    // Seed the stack with a recognizable value so we can later assert it was untouched.
    app.state.stack.x = HpNum::from(42);
    let x_before = app.state.stack.x.clone();

    app.handle_key(key('f'));
    assert!(app.shift_armed, "arming precondition");

    app.handle_key(raw_key(KeyCode::Esc));
    assert!(
        !app.shift_armed,
        "Esc must cancel the armed prefix (D-25.4)"
    );
    assert_eq!(
        app.state.stack.x, x_before,
        "Esc must NOT dispatch any op — stack X must be unchanged"
    );
}

/// Task 1 / behavior 4: in ALPHA mode, 'f' is a literal character — the
/// arming check is gated behind the ALPHA-mode `return` per Pitfall 2 (D-25.5).
#[test]
fn test_shift_armed_alpha_override() {
    let (mut app, _tmp) = make_app();
    app.state.alpha_mode = true;
    let alpha_before = app.state.alpha_reg.clone();

    app.handle_key(key('f'));

    assert!(
        !app.shift_armed,
        "f in ALPHA mode MUST NOT arm the prefix (D-25.5 — ALPHA overrides Prefix)"
    );
    // The current ALPHA-mode handler appends the raw char without uppercasing
    // (the hardware-faithful "ALPHA is always uppercase" rule is tracked as a
    // separate v3.x charset task). We accept either case so this regression
    // does NOT lock in the lowercase quirk — D-25.5 only requires that `f`
    // reaches the ALPHA buffer at all rather than arming the prefix.
    let appended = app.state.alpha_reg.trim_start_matches(&alpha_before[..]);
    assert!(
        appended.eq_ignore_ascii_case("f"),
        "f in ALPHA mode must append the literal letter (got alpha_reg={:?}, appended={:?})",
        app.state.alpha_reg,
        appended
    );
}

/// Task 1 / Pitfall 5: a prefix armed via `f` and then followed by an
/// UNMAPPED key (e.g. `;`) must clear at the end of THAT key cycle — not stick
/// until the next recognized key. The risk being guarded is that a naive impl
/// only clears `shift_armed = false` inside the `Some(op)` match arm, leaving
/// the prefix latched on a `shifted_key_to_op` miss.
#[test]
fn test_shift_armed_pitfall5_bleed() {
    let (mut app, _tmp) = make_app();

    app.handle_key(key('f'));
    assert!(app.shift_armed, "arming precondition");

    // `;` has no entry in shifted_key_to_op (Plan 01 wires only -/+/*/(slash)).
    app.handle_key(key(';'));
    assert!(
        !app.shift_armed,
        "Pitfall 5: shift_armed must clear after EVERY armed key cycle, \
         not only on recognized op keys (one-shot lifetime = 'next key cycle')"
    );
}

/// Task 1 / Pitfall 3 regression: pressing `f` once must arm the prefix and
/// NOT change `state.display_mode`. The v1.x FIX/SCI/ENG cycle was removed
/// atomically with the arming logic landing.
#[test]
fn test_f_does_not_cycle_display_mode() {
    let (mut app, _tmp) = make_app();
    let mode_before = app.state.display_mode;

    app.handle_key(key('f'));

    assert!(
        app.shift_armed,
        "f must arm the prefix (not fall through to v1.x FmtDigits cycle)"
    );
    assert_eq!(
        app.state.display_mode, mode_before,
        "Pitfall 3: the v1.x f-cycle binding was REMOVED — display_mode MUST be unchanged"
    );
}

/// Task 1 / Pitfall 4 (by design): if a modal is active, `f` is swallowed by
/// the modal — global prefix arming does NOT activate.
///
/// We use `PendingInput::PrintModal` since it has no transient state to seed.
/// The modal silently ignores unrecognized keys (it expects x/a/s) so we
/// simply check that `shift_armed` stays false.
#[test]
fn test_shift_armed_not_activated_inside_modal() {
    use hp41_cli::app::PendingInput;
    let (mut app, _tmp) = make_app();
    app.pending_input = Some(PendingInput::PrintModal);

    app.handle_key(key('f'));

    assert!(
        !app.shift_armed,
        "Pitfall 4 (by design): a modal must swallow `f`; global arming MUST stay off"
    );
    assert!(
        app.pending_input.is_some(),
        "the modal must still be open after the swallowed key"
    );
}
