//! Unit tests for keys::key_to_op() mapping table.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use hp41_core::ops::Op;
use hp41_core::CalcState;

use crate::app::App;
use crate::keys::key_to_op;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::NONE,
    }
}

fn make_app() -> App {
    App::new(CalcState::new(), std::path::PathBuf::from("/tmp/hp41_test.json"))
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
    assert_eq!(key_to_op(press(KeyCode::Char('p')), &app), Some(Op::PrgmMode));
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
    assert_eq!(key_to_op(press(KeyCode::Char('S')), &app), Some(Op::Sin));
    assert_eq!(key_to_op(press(KeyCode::Char('C')), &app), Some(Op::Cos));
    assert_eq!(key_to_op(press(KeyCode::Char('T')), &app), Some(Op::Tan));
    assert_eq!(key_to_op(press(KeyCode::Char('L')), &app), Some(Op::Ln));
    assert_eq!(key_to_op(press(KeyCode::Char('G')), &app), Some(Op::Log));
    assert_eq!(key_to_op(press(KeyCode::Char('E')), &app), Some(Op::Exp));
    assert_eq!(key_to_op(press(KeyCode::Char('H')), &app), Some(Op::TenPow));
    assert_eq!(key_to_op(press(KeyCode::Char('I')), &app), Some(Op::Recip));
    assert_eq!(key_to_op(press(KeyCode::Char('W')), &app), Some(Op::Sq));
    assert_eq!(key_to_op(press(KeyCode::Char('Y')), &app), Some(Op::YPow));
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
    assert_eq!(
        crate::keys::KEY_REF_TABLE.len(),
        33,
        "KEY_REF_TABLE must have exactly 33 entries"
    );
}
