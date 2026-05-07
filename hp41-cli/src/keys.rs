//! Key → Op mapping for the HP-41 TUI.
//!
//! key_to_op() is the sole translation layer between crossterm KeyEvents and
//! hp41-core Op variants. The function signature receives &App for future
//! context-sensitivity (USER mode in Phase 5) without changing call sites.
//!
//! Digit keys (0-9, '.', 'e'), quit keys ('q', Ctrl+C), mode-cycle keys ('d', 'f'),
//! and F5/F7/F8 are handled directly in app.handle_key() and MUST NOT appear here.

use crossterm::event::{KeyCode, KeyEvent};
use hp41_core::ops::Op;

use crate::app::App;

/// Map a crossterm KeyEvent to an hp41-core Op.
/// Returns None for keys handled elsewhere (digits, quit, mode cycles, F5/F7/F8).
/// Returns None for unmapped keys (silently ignored by app.handle_key).
pub fn key_to_op(key: KeyEvent, _app: &App) -> Option<Op> {
    match key.code {
        // Stack operations
        KeyCode::Enter               => Some(Op::Enter),
        KeyCode::Backspace           => Some(Op::Clx),
        // Arithmetic
        KeyCode::Char('+')           => Some(Op::Add),
        KeyCode::Char('-')           => Some(Op::Sub),
        KeyCode::Char('*')           => Some(Op::Mul),
        KeyCode::Char('/')           => Some(Op::Div),
        // Stack ops (lowercase)
        KeyCode::Char('n')           => Some(Op::Chs),
        KeyCode::Char('r')           => Some(Op::Rdn),
        KeyCode::Char('x')           => Some(Op::XySwap),
        KeyCode::Char('l')           => Some(Op::Lastx),
        KeyCode::Char('s')           => Some(Op::Sqrt),
        KeyCode::Char('p')           => Some(Op::PrgmMode),
        // Inverse trig (lowercase — D-09 addition: a/c/k for ASIN/ACOS/ATAN)
        KeyCode::Char('a')           => Some(Op::Asin),
        KeyCode::Char('c')           => Some(Op::Acos),
        KeyCode::Char('k')           => Some(Op::Atan),
        // Trig / math: uppercase char = Shift+letter (D-09, crossterm convention).
        // Crossterm delivers Shift+s as KeyCode::Char('S'); no modifier check needed.
        KeyCode::Char('S')           => Some(Op::Sin),
        KeyCode::Char('C')           => Some(Op::Cos),
        KeyCode::Char('T')           => Some(Op::Tan),
        KeyCode::Char('L')           => Some(Op::Ln),
        KeyCode::Char('G')           => Some(Op::Log),
        KeyCode::Char('E')           => Some(Op::Exp),
        KeyCode::Char('H')           => Some(Op::TenPow),
        KeyCode::Char('I')           => Some(Op::Recip),
        KeyCode::Char('W')           => Some(Op::Sq),
        KeyCode::Char('Y')           => Some(Op::YPow),
        // F5/F7/F8 handled in app.handle_key — return None here.
        KeyCode::F(5) | KeyCode::F(7) | KeyCode::F(8) => None,
        // F1-F4: Phase 5 stubs (user-assignable keys).
        KeyCode::F(1) | KeyCode::F(2) | KeyCode::F(3) | KeyCode::F(4) => None,
        // All other keys (including digits 0-9, '.', 'e', 'd', 'f', 'q') — handled elsewhere.
        _ => None,
    }
}

/// Key-reference table for the TUI right panel (INPUT-01 discoverability).
/// Shown verbatim in ui.rs render_right_panel(). 33 entries.
/// STO/RCL/ALPHA ops deferred to Phase 5 (require address-entry dialog or mode routing).
pub const KEY_REF_TABLE: &[(&str, &str)] = &[
    ("0-9 .",  "digit entry"),
    ("e",      "EEX (sci notation entry)"),
    ("Enter",  "ENTER / lift stack"),
    ("Bksp",   "CLX (clear X)"),
    ("+",      "add"),
    ("-",      "subtract"),
    ("*",      "multiply"),
    ("/",      "divide"),
    ("n",      "CHS (change sign)"),
    ("r",      "R\u{2193} (roll down)"),
    ("x",      "X\u{27F7}Y (swap)"),
    ("l",      "LASTX"),
    ("s",      "\u{221a}x"),
    ("a",      "ASIN (arc sine)"),
    ("c",      "ACOS (arc cosine)"),
    ("k",      "ATAN (arc tangent)"),
    ("S",      "SIN  (Shift+s)"),
    ("C",      "COS  (Shift+c)"),
    ("T",      "TAN  (Shift+t)"),
    ("L",      "LN   (Shift+l)"),
    ("G",      "LOG  (Shift+g)"),
    ("E",      "e^x  (Shift+e)"),
    ("H",      "10^x (Shift+h)"),
    ("I",      "1/x  (Shift+i)"),
    ("W",      "x\u{00B2}   (Shift+w)"),
    ("Y",      "y^x  (Shift+y)"),
    ("p",      "PRGM toggle"),
    ("d",      "cycle DEG/RAD/GRAD"),
    ("f",      "cycle FIX/SCI/ENG"),
    ("F5",     "R/S (run program A)"),
    ("F7",     "SST (step forward)"),
    ("F8",     "BST (step back)"),
    ("q/^C",   "quit"),
];
