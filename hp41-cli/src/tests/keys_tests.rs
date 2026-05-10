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
}

#[test]
fn stack_ops_lowercase() {
    let app = make_app();
    assert_eq!(key_to_op(press(KeyCode::Char('n')), &app), Some(Op::Chs));
    assert_eq!(key_to_op(press(KeyCode::Char('r')), &app), Some(Op::Rdn));
    assert_eq!(key_to_op(press(KeyCode::Char('x')), &app), Some(Op::XySwap));
    assert_eq!(key_to_op(press(KeyCode::Char('l')), &app), Some(Op::Lastx));
    assert_eq!(key_to_op(press(KeyCode::Char('s')), &app), Some(Op::Sqrt));
    assert_eq!(
        key_to_op(press(KeyCode::Char('p')), &app),
        Some(Op::PrgmMode)
    );
}

#[test]
fn inverse_trig_lowercase() {
    let app = make_app();
    assert_eq!(key_to_op(press(KeyCode::Char('a')), &app), Some(Op::Asin));
    assert_eq!(key_to_op(press(KeyCode::Char('c')), &app), Some(Op::Acos));
    assert_eq!(key_to_op(press(KeyCode::Char('k')), &app), Some(Op::Atan));
}

#[test]
fn trig_math_uppercase_shift() {
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
    assert_eq!(key_to_op(press(KeyCode::Char('C')), &app), Some(Op::Cos));
    assert_eq!(key_to_op(press(KeyCode::Char('T')), &app), Some(Op::Tan));
    assert_eq!(key_to_op(press(KeyCode::Char('L')), &app), Some(Op::Ln));
    assert_eq!(key_to_op(press(KeyCode::Char('G')), &app), Some(Op::Log));
    assert_eq!(key_to_op(press(KeyCode::Char('E')), &app), Some(Op::Exp));
    assert_eq!(key_to_op(press(KeyCode::Char('H')), &app), Some(Op::TenPow));
    assert_eq!(key_to_op(press(KeyCode::Char('I')), &app), Some(Op::Recip));
    assert_eq!(key_to_op(press(KeyCode::Char('W')), &app), Some(Op::Sq));
    assert_eq!(key_to_op(press(KeyCode::Char('Y')), &app), Some(Op::YPow));
    // Phase 5: 'u' maps to Op::UserMode
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
fn key_ref_table_has_33_entries() {
    // Phase 5 added 7 new entries (u, ?, Ctrl+S, Ctrl+P, Ctrl+A, F1-F4, R modal);
    // Phase 6 added 12 new entries (z, Z, m, D, y, b, O, V, h, F, j, J).
    // Phase 8: quit entry "q/^C" replaced by "^C" (same count), added q->SIN and g->CLREG (+2).
    // Phase 12: added "X nn" hex modal entry (+1).
    // Total is now 55. Test name preserved for history; count updated to 55.
    assert_eq!(
        crate::keys::KEY_REF_TABLE.len(),
        55,
        "KEY_REF_TABLE must have exactly 55 entries (54 Phase 1-8 + 1 Phase 12: X nn hex modal)"
    );
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

// Phase 8: key_to_op() bindings for SIN and CLREG
#[test]
fn q_maps_to_sin() {
    let app = make_app();
    assert_eq!(
        key_to_op(press(KeyCode::Char('q')), &app),
        Some(Op::Sin),
        "'q' must map to Op::Sin after Phase 8 reassignment"
    );
}

#[test]
fn g_maps_to_clreg() {
    let app = make_app();
    assert_eq!(
        key_to_op(press(KeyCode::Char('g')), &app),
        Some(Op::Clreg),
        "'g' must map to Op::Clreg"
    );
}
