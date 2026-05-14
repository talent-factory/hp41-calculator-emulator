//! Unit tests for keys::key_to_op() and keycode_to_hp41_code() mapping tables.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use hp41_core::ops::Op;
use hp41_core::CalcState;

use crate::app::App;
use crate::keys::{key_to_op, keycode_to_hp41_code};

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::NONE,
    }
}

fn make_app() -> App {
    App::new(
        CalcState::new(),
        std::path::PathBuf::from("/tmp/hp41_test.json"),
        None,
    )
}

#[test]
fn enter_maps_to_op_enter() {
    let app = make_app();
    assert_eq!(key_to_op(press(KeyCode::Enter), &app), Some(Op::Enter));
}

#[test]
fn backspace_maps_to_op_clx() {
    let app = make_app();
    assert_eq!(key_to_op(press(KeyCode::Backspace), &app), Some(Op::Clx));
}

#[test]
fn arithmetic_keys() {
    let app = make_app();
    assert_eq!(key_to_op(press(KeyCode::Char('+')), &app), Some(Op::Add));
    assert_eq!(key_to_op(press(KeyCode::Char('-')), &app), Some(Op::Sub));
    assert_eq!(key_to_op(press(KeyCode::Char('*')), &app), Some(Op::Mul));
    assert_eq!(key_to_op(press(KeyCode::Char('/')), &app), Some(Op::Div));
    assert_eq!(
        key_to_op(press(KeyCode::Char('%')), &app),
        Some(Op::PctChange)
    );
}

#[test]
fn stack_ops_lowercase() {
    let app = make_app();
    // Phase 25 (D-25.3): the lowercase stack-ops bindings survive the hard
    // cut because they correspond to real HP-41CV primary positions
    // (CHS row 8, R↓ row 2, X⟷Y row 2, LASTX row 2, PRGM top row). They
    // are explicitly kept in key_to_op per Plan 01 / Task 2.
    assert_eq!(key_to_op(press(KeyCode::Char('n')), &app), Some(Op::Chs));
    assert_eq!(key_to_op(press(KeyCode::Char('r')), &app), Some(Op::Rdn));
    assert_eq!(key_to_op(press(KeyCode::Char('x')), &app), Some(Op::XySwap));
    assert_eq!(key_to_op(press(KeyCode::Char('l')), &app), Some(Op::Lastx));
    assert_eq!(
        key_to_op(press(KeyCode::Char('p')), &app),
        Some(Op::PrgmMode)
    );
    // Phase 25 (D-25.3): 's' → Op::Sqrt is REMOVED. The √x op is now reached
    // via the XEQ-by-Name modal ("SQRT") or its f-shifted keyboard position
    // (Plan 02). Plain 's' is unmapped.
    assert_eq!(
        key_to_op(press(KeyCode::Char('s')), &app),
        None,
        "'s' v1.x→Sqrt binding REMOVED per D-25.3"
    );
}

#[test]
fn inverse_trig_lowercase_removed_d25_3() {
    let app = make_app();
    // Phase 25 (D-25.3): the lowercase inverse-trig direct map (a→ASIN,
    // c→ACOS, k→ATAN) is GONE. These ops are now reached via the
    // XEQ-by-Name modal (Plan 03) or their HP-41CV f-shifted keyboard
    // positions (Plan 02 / 04).
    assert_eq!(key_to_op(press(KeyCode::Char('a')), &app), None);
    assert_eq!(key_to_op(press(KeyCode::Char('c')), &app), None);
    assert_eq!(key_to_op(press(KeyCode::Char('k')), &app), None);
}

#[test]
fn trig_math_uppercase_shift_removed_d25_3() {
    let app = make_app();
    // Phase 5: 'S' is now intercepted in app.handle_key() BEFORE key_to_op() — it triggers
    // the STO [nn] modal. key_to_op must return None for 'S' and 'R' (D-10 routing).
    assert_eq!(
        key_to_op(press(KeyCode::Char('S')), &app),
        None,
        "'S' must return None — STO modal is intercepted upstream"
    );
    assert_eq!(
        key_to_op(press(KeyCode::Char('R')), &app),
        None,
        "'R' must return None — RCL modal is intercepted upstream"
    );
    // Phase 25 (D-25.3): every v1.x uppercase letter direct-binding is GONE.
    // The previously-mapped ops are now reached via XEQ-by-Name (Plan 03) or
    // their real HP-41CV f-shifted keyboard positions (Plan 02 / 04).
    let removed = ['C', 'T', 'L', 'G', 'E', 'H', 'I', 'W', 'Y'];
    for c in removed {
        assert_eq!(
            key_to_op(press(KeyCode::Char(c)), &app),
            None,
            "v1.x binding for {:?} must be removed per D-25.3",
            c
        );
    }
    // Phase 5: 'u' maps to Op::UserMode — primary HP-41CV USER-mode toggle.
    assert_eq!(
        key_to_op(press(KeyCode::Char('u')), &app),
        Some(Op::UserMode)
    );
}

#[test]
fn f_keys_handled_in_app_return_none() {
    let app = make_app();
    assert_eq!(key_to_op(press(KeyCode::F(5)), &app), None);
    assert_eq!(key_to_op(press(KeyCode::F(7)), &app), None);
    assert_eq!(key_to_op(press(KeyCode::F(8)), &app), None);
    assert_eq!(key_to_op(press(KeyCode::F(1)), &app), None);
    assert_eq!(key_to_op(press(KeyCode::F(4)), &app), None);
}

#[test]
fn unmapped_keys_return_none() {
    let app = make_app();
    // digit keys are handled by app.handle_key directly, not key_to_op
    assert_eq!(key_to_op(press(KeyCode::Char('1')), &app), None);
    assert_eq!(key_to_op(press(KeyCode::Char('.')), &app), None);
    assert_eq!(key_to_op(press(KeyCode::F(10)), &app), None);
    assert_eq!(key_to_op(press(KeyCode::Esc), &app), None);
}

#[test]
fn key_ref_table_has_60_entries() {
    // 55 baseline entries through Phase 12, plus '%' (%CH) and the four
    // Card Reader shortcuts (Ctrl+W/R/D/F) — 60 total.
    assert_eq!(crate::keys::KEY_REF_TABLE.len(), 60);
}

// Phase 12: F5/F7/F8 must return None from keycode_to_hp41_code so the caller
// skips the last_key_code write and GETKEY capture is not corrupted.
#[test]
fn f5_f7_f8_return_none_keycode_for_getkey() {
    assert_eq!(
        keycode_to_hp41_code(KeyCode::F(5)),
        None,
        "F5 (R/S TUI trigger) must return None — no HP-41 physical key equivalent"
    );
    assert_eq!(
        keycode_to_hp41_code(KeyCode::F(7)),
        None,
        "F7 (SST TUI trigger) must return None"
    );
    assert_eq!(
        keycode_to_hp41_code(KeyCode::F(8)),
        None,
        "F8 (BST TUI trigger) must return None"
    );
}

#[test]
fn digit_5_returns_hp41_keycode_62() {
    assert_eq!(
        keycode_to_hp41_code(KeyCode::Char('5')),
        Some(62),
        "'5' must map to HP-41 code 62 (row 6, col 2)"
    );
}

#[test]
fn unmapped_keys_return_none_from_keycode_fn() {
    assert_eq!(keycode_to_hp41_code(KeyCode::Esc), None);
    assert_eq!(keycode_to_hp41_code(KeyCode::F(1)), None);
    assert_eq!(keycode_to_hp41_code(KeyCode::Up), None);
}

// Phase 25 (D-25.3): the Phase 8 q→SIN and g→CLREG direct bindings are
// REMOVED. SIN is reached via the SIN f-shifted keyboard position (Plan 02 /
// 04). CLREG is reached via XEQ-by-Name modal "CLRG" (Plan 03). Plain `q`
// and `g` are unmapped.
#[test]
fn q_unmapped_after_d25_3() {
    let app = make_app();
    assert_eq!(
        key_to_op(press(KeyCode::Char('q')), &app),
        None,
        "'q' v1.x→SIN binding REMOVED per D-25.3 (Plan 25-01 Task 2)"
    );
}

#[test]
fn g_unmapped_after_d25_3() {
    let app = make_app();
    assert_eq!(
        key_to_op(press(KeyCode::Char('g')), &app),
        None,
        "'g' v1.x→CLREG binding REMOVED per D-25.3 (Plan 25-01 Task 2)"
    );
}
