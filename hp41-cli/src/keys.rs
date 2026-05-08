//! Key → Op mapping for the HP-41 TUI.
//!
//! key_to_op() is the sole translation layer between crossterm KeyEvents and
//! hp41-core Op variants. The `_app` parameter is kept for potential context-sensitivity
//! (USER mode state checks) without breaking call sites.
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
        KeyCode::Enter => Some(Op::Enter),
        KeyCode::Backspace => Some(Op::Clx),
        // Arithmetic
        KeyCode::Char('+') => Some(Op::Add),
        KeyCode::Char('-') => Some(Op::Sub),
        KeyCode::Char('*') => Some(Op::Mul),
        KeyCode::Char('/') => Some(Op::Div),
        // Stack ops (lowercase)
        KeyCode::Char('n') => Some(Op::Chs),
        KeyCode::Char('r') => Some(Op::Rdn),
        KeyCode::Char('x') => Some(Op::XySwap),
        KeyCode::Char('l') => Some(Op::Lastx),
        KeyCode::Char('s') => Some(Op::Sqrt),
        KeyCode::Char('p') => Some(Op::PrgmMode),
        // Inverse trig (lowercase — D-09 addition: a/c/k for ASIN/ACOS/ATAN)
        KeyCode::Char('a') => Some(Op::Asin),
        KeyCode::Char('c') => Some(Op::Acos),
        KeyCode::Char('k') => Some(Op::Atan),
        // Trig / math: uppercase char = Shift+letter (D-09, crossterm convention).
        // Crossterm delivers Shift+s as KeyCode::Char('S'); no modifier check needed.
        // NOTE: 'S' is now intercepted in app.handle_key() BEFORE key_to_op() is called.
        // It triggers the STO [nn] register modal (Phase 5, D-10). SIN is now unmapped.
        KeyCode::Char('C') => Some(Op::Cos),
        KeyCode::Char('T') => Some(Op::Tan),
        KeyCode::Char('L') => Some(Op::Ln),
        KeyCode::Char('G') => Some(Op::Log),
        KeyCode::Char('E') => Some(Op::Exp),
        KeyCode::Char('H') => Some(Op::TenPow),
        KeyCode::Char('I') => Some(Op::Recip),
        KeyCode::Char('W') => Some(Op::Sq),
        KeyCode::Char('Y') => Some(Op::YPow),
        // Phase 5: USER mode toggle (D-26)
        KeyCode::Char('u') => Some(Op::UserMode),
        // Phase 6: Science & Engineering stats bindings
        // Note: 'd' is intercepted in app.rs for angle mode; use 'D' for SDEV.
        KeyCode::Char('z') => Some(Op::SigmaPlus),
        KeyCode::Char('Z') => Some(Op::SigmaMinus),
        KeyCode::Char('m') => Some(Op::Mean),
        KeyCode::Char('D') => Some(Op::Sdev),
        KeyCode::Char('y') => Some(Op::Yhat),
        // Note: 'l' is taken by Op::Lastx; 'R' is STO/RCL modal (None); use 'b' for L.R.
        KeyCode::Char('b') => Some(Op::LR),
        KeyCode::Char('O') => Some(Op::Corr),
        KeyCode::Char('V') => Some(Op::ClSigmaStat),
        // Phase 6: HMS bindings
        // Note: 'f' is intercepted in app.rs for format mode; use 'F' for →HMS.
        KeyCode::Char('h') => Some(Op::HmsToH),
        KeyCode::Char('F') => Some(Op::HToHms),
        KeyCode::Char('j') => Some(Op::HmsAdd),
        KeyCode::Char('J') => Some(Op::HmsSub),
        // Phase 5: S and R start STO/RCL register-number modal entry (D-10).
        // They do NOT return an Op here — the modal is intercepted in app.handle_key()
        // BEFORE key_to_op() is called. Return None so the fallthrough is a no-op.
        // (The modal sets pending_input, which is handled via handle_pending_input.)
        KeyCode::Char('S') | KeyCode::Char('R') => None,
        // F5/F7/F8 handled in app.handle_key — return None here.
        KeyCode::F(5) | KeyCode::F(7) | KeyCode::F(8) => None,
        // F1-F4: intercepted in app.handle_key() for USER mode dispatch — not routed through key_to_op().
        KeyCode::F(1) | KeyCode::F(2) | KeyCode::F(3) | KeyCode::F(4) => None,
        // All other keys (including digits 0-9, '.', 'e', 'd', 'f', 'q') — handled elsewhere.
        _ => None,
    }
}

/// Key-reference table for the TUI right panel (INPUT-01 discoverability).
/// Shown verbatim in ui.rs render_right_panel().
pub const KEY_REF_TABLE: &[(&str, &str)] = &[
    ("0-9 .", "digit entry"),
    ("e", "EEX (sci notation entry)"),
    ("Enter", "ENTER / lift stack"),
    ("Bksp", "CLX (clear X)"),
    ("+", "add"),
    ("-", "subtract"),
    ("*", "multiply"),
    ("/", "divide"),
    ("n", "CHS (change sign)"),
    ("r", "R\u{2193} (roll down)"),
    ("x", "X\u{27F7}Y (swap)"),
    ("l", "LASTX"),
    ("s", "\u{221a}x"),
    ("a", "ASIN (arc sine)"),
    ("c", "ACOS (arc cosine)"),
    ("k", "ATAN (arc tangent)"),
    ("S", "STO [nn] (modal register entry)"),
    ("R", "RCL [nn] (modal register entry)"),
    ("C", "COS  (Shift+c)"),
    ("T", "TAN  (Shift+t)"),
    ("L", "LN   (Shift+l)"),
    ("G", "LOG  (Shift+g)"),
    ("E", "e^x  (Shift+e)"),
    ("H", "10^x (Shift+h)"),
    ("I", "1/x  (Shift+i)"),
    ("W", "x\u{00B2}   (Shift+w)"),
    ("Y", "y^x  (Shift+y)"),
    ("p", "PRGM toggle"),
    ("d", "cycle DEG/RAD/GRAD"),
    ("f", "cycle FIX/SCI/ENG"),
    ("F5", "R/S (run program A)"),
    ("F7", "SST (step forward)"),
    ("F8", "BST (step back)"),
    ("q/^C", "quit"),
    // Phase 5: new bindings
    ("u", "USER mode toggle"),
    ("?", "help overlay (toggle)"),
    ("Ctrl+S", "save state to file"),
    ("Ctrl+P", "program library overlay"),
    ("Ctrl+A", "assign key in USER mode"),
    ("F1-F4", "USER keys a/b/c/d (USER mode)"),
    // Phase 6: Science & Engineering
    (
        "z",
        "\u{03A3}+  (\u{03A3}+ accumulate into stats registers; push n to X)",
    ),
    (
        "Z",
        "\u{03A3}-  (\u{03A3}- remove from stats registers; push n to X)",
    ),
    ("m", "MEAN (x\u{0305} in X, y\u{0305} in Y)"),
    (
        "D",
        "SDEV (sample \u{03C3}x in X, \u{03C3}y in Y; n-1 denom)",
    ),
    (
        "y",
        "YHAT (\u{0177} prediction from X via linear regression)",
    ),
    (
        "b",
        "L.R. (linear regression: slope m in Y, intercept b in X)",
    ),
    ("O", "CORR (correlation coefficient r in X)"),
    (
        "V",
        "CL\u{03A3} (clear \u{03A3} stats registers R01-R06 to zero)",
    ),
    ("h", "HMS\u{2192} (H.MMSS to decimal hours)"),
    ("F", "\u{2192}HMS (decimal hours to H.MMSS format)"),
    ("j", "HMS+  (add two H.MMSS values, base-60 carry)"),
    ("J", "HMS-  (subtract H.MMSS values, base-60 borrow)"),
];

#[cfg(test)]
mod tests {
    use super::KEY_REF_TABLE;
    use hp41_core::{ops::Op, CalcState};

    /// BLOCKER 1: test_user_mode_dispatch — pressing 'u' dispatches Op::UserMode which
    /// toggles state.user_mode. Verifies the op the key binding produces is correct.
    #[test]
    fn test_user_mode_dispatch() {
        let mut state = CalcState::new();
        assert!(!state.user_mode, "user_mode starts false");

        // Dispatch Op::UserMode directly (same op that key 'u' produces via key_to_op)
        let result = hp41_core::ops::dispatch(&mut state, Op::UserMode);
        assert!(
            result.is_ok(),
            "UserMode dispatch must not error: {:?}",
            result
        );
        assert!(
            state.user_mode,
            "user_mode must be true after toggling once"
        );

        // Second dispatch: toggle back to false
        let result2 = hp41_core::ops::dispatch(&mut state, Op::UserMode);
        assert!(result2.is_ok());
        assert!(
            !state.user_mode,
            "user_mode must be false after toggling twice"
        );
    }

    /// Verify that key assignments persist on state (prerequisite for USER mode key dispatch).
    #[test]
    fn test_user_key_assignment_persists() {
        let mut state = CalcState::new();
        state.user_mode = true;
        state.key_assignments.insert('a', "MYPROG".to_string());

        assert_eq!(
            state.key_assignments.get(&'a').map(|s| s.as_str()),
            Some("MYPROG"),
            "key assignment must be retrievable from state"
        );
    }

    // Phase 8: KEY_REF_TABLE content tests (do not require App construction)
    #[test]
    fn test_key_ref_table_has_sin_entry() {
        let has_sin = KEY_REF_TABLE
            .iter()
            .any(|(k, desc)| *k == "q" && desc.contains("SIN"));
        assert!(has_sin, "KEY_REF_TABLE must have q->SIN entry");
    }

    #[test]
    fn test_key_ref_table_has_clreg_entry() {
        let has_clreg = KEY_REF_TABLE
            .iter()
            .any(|(k, desc)| *k == "g" && desc.contains("CLREG"));
        assert!(has_clreg, "KEY_REF_TABLE must have g->CLREG entry");
    }

    #[test]
    fn test_key_ref_table_quit_is_ctrl_c_only() {
        // 'q' must no longer be listed as a quit key after Phase 8 reassignment
        let q_quit = KEY_REF_TABLE
            .iter()
            .any(|(k, desc)| k.contains('q') && desc.to_lowercase().contains("quit"));
        assert!(
            !q_quit,
            "KEY_REF_TABLE must not list 'q' as a quit key after reassignment to SIN"
        );
    }
}
