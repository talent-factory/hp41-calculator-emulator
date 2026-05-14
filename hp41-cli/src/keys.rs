//! Key → Op mapping for the HP-41 TUI.
//!
//! key_to_op() is the sole translation layer between crossterm KeyEvents and
//! hp41-core Op variants. The `_app` parameter is kept for potential context-sensitivity
//! (USER mode state checks) without breaking call sites.
//!
//! Digit keys (0-9, '.', 'e'), quit key (Ctrl+C), mode-cycle keys ('d', 'f'),
//! and F5/F7/F8 are handled directly in app.handle_key() and MUST NOT appear here.

use crossterm::event::{KeyCode, KeyEvent};
use hp41_core::ops::{FlagTestKind, Op, StoArithKind, TestKind};

use crate::app::App;

// ── Phase 25 Plan 02: TUI-local discriminator enums for Hybrid PendingInput ──
//
// These enums collapse multiple parallel-state `Op::` variants into a single
// `PendingInput` group variant per D-25.11. They WRAP the hp41-core enums
// (`FlagTestKind`, `StoArithKind`) rather than redefining them per D-25.13 —
// the rule is "reuse hp41-core enums; do NOT define parallel TUI-local
// discriminator enums" for kinds that already exist in core.
//
// `FlagPromptKind` is the Phase 25 modal-driver for the 6 logical flag ops
// (SF / CF / FS? / FC? / FS?C / FC?C) × {direct, IND}. The Test arm reuses
// `hp41_core::ops::FlagTestKind` directly.
//
// `RegisterOpKind` is a new TUI-local enum because hp41-core has no single
// discriminator for the heterogeneous family `RCL / VIEW / ARCL / ASTO /
// ISG / DSE`. It wraps `hp41_core::ops::StoArithKind` for the STO-arith
// sub-family so we don't duplicate that enum either.

/// Discriminator for the `PendingInput::FlagPrompt` group variant.
///
/// Logical variants: SetFlag (SF), ClearFlag (CF), and four `Test(_)` arms
/// covering FS? / FC? / FS?C / FC?C via the reused
/// `hp41_core::ops::FlagTestKind` per D-25.13.
//
// `dead_code` is allowed at the type level because Task 1 lands the enum and
// the exhaustive `pending_prompt()` arm — Task 2 wires the modal openers and
// dispatch which exercise every variant. Removing this allow after Task 2 is
// part of that task's acceptance criteria.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum FlagPromptKind {
    SetFlag,
    ClearFlag,
    Test(FlagTestKind),
}

/// Discriminator for the `PendingInput::RegisterPrompt` group variant.
///
/// Logical variants: Sto / Rcl / StoArith(StoArithKind) (4 inner ops) / View
/// / Arcl / Asto / Isg / Dse. The `StoArith` arm reuses
/// `hp41_core::ops::StoArithKind` per D-25.13.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum RegisterOpKind {
    Sto,
    Rcl,
    StoArith(StoArithKind),
    View,
    Arcl,
    Asto,
    Isg,
    Dse,
}

/// Map a crossterm KeyEvent to an hp41-core Op for the **primary** HP-41CV
/// keyboard positions only (Phase 25 / D-25.1 / D-25.3).
///
/// Phase 25 (Plan 01, Task 2) is the **hard cut** from v1.x crossterm-style
/// direct-letter bindings to HP-41CV hardware-faithful primaries. The
/// previous letter map (C → COS, T → TAN, L → LN, G → LOG, E → e^x,
/// H → 10^x, I → 1/x, W → x², Y → y^x, q → SIN, a/c/k → ASIN/ACOS/ATAN,
/// s → √x, g → CLREG, z/Z/m/D/y/b/O/V → stats, h/j/J → HMS) is GONE per
/// D-25.3. Those ops are now reached either:
///   - via the f-prefix (`shifted_key_to_op` — Plan 01 wires the four
///     conditional tests; Plan 02 wires modal openers), or
///   - via the XEQ-by-Name modal (shipped v2.1; Plan 03 extends it for the
///     eight non-keyboard conditional tests per D-25.8/D-25.9), or
///   - via the FIX/SCI/ENG modal (`F`, preserved for Plan 01) which Plan 02
///     repositions onto its real f-shifted keyboard slot.
///
/// What we **keep** here:
///   - Truly universal control keys (Enter, Backspace) — same on every
///     HP calculator and the user's terminal.
///   - The four arithmetic primaries (+/-/*//) — top-row HP-41CV positions.
///   - `%` — HP-41 PctChange primary.
///   - Lower-case shortcut letters that happen to live on the user's
///     ASCII keyboard with no HP-41CV-letter collision: `n`→CHS, `r`→R↓,
///     `x`→X⟷Y, `l`→LASTX, `p`→PrgmMode, `u`→USER. These are convenience
///     mnemonics that survive D-25.3 because they correspond to HP-41CV
///     primary key labels (CHS is yellow-printed but reached via the chs
///     primary on row 8; R↓/X⟷Y/LASTX are primary positions; PRGM and
///     USER are top-row mode keys).
///
/// Returns None for keys handled elsewhere (digits, Ctrl+C quit, mode
/// cycles, F5/F7/F8) and for all unmapped keys (silently ignored by
/// app.handle_key — including every former v1.x letter binding).
pub fn key_to_op(key: KeyEvent, _app: &App) -> Option<Op> {
    match key.code {
        // ── Universal control keys ──────────────────────────────────────
        KeyCode::Enter => Some(Op::Enter),
        KeyCode::Backspace => Some(Op::Clx),

        // ── Arithmetic primaries (HP-41CV row 4–8 right column) ─────────
        KeyCode::Char('+') => Some(Op::Add),
        KeyCode::Char('-') => Some(Op::Sub),
        KeyCode::Char('*') => Some(Op::Mul),
        KeyCode::Char('/') => Some(Op::Div),
        KeyCode::Char('%') => Some(Op::PctChange),

        // ── HP-41CV primary positions with surviving ASCII shortcuts ────
        // CHS (row 8 chs key), R↓ (row 2), X⟷Y (row 2), LASTX (row 2-ish),
        // PRGM/USER (top-row mode keys).
        KeyCode::Char('n') => Some(Op::Chs),
        KeyCode::Char('r') => Some(Op::Rdn),
        KeyCode::Char('x') => Some(Op::XySwap),
        KeyCode::Char('l') => Some(Op::Lastx),
        KeyCode::Char('p') => Some(Op::PrgmMode),
        KeyCode::Char('u') => Some(Op::UserMode),

        // ── Modal openers handled BEFORE key_to_op in app.handle_key() ──
        // S → StoRegister, R → RclRegister, F → FmtDigits, P → PrintModal,
        // X → HexModal (PRGM mode). Returning None lets the fallthrough
        // be a no-op should those interceptors ever be reordered.
        KeyCode::Char('S')
        | KeyCode::Char('R')
        | KeyCode::Char('F')
        | KeyCode::Char('P')
        | KeyCode::Char('X') => None,

        // F1–F8 are TUI bindings handled directly in app.handle_key()
        // (R/S, SST, BST, USER F1–F4).
        KeyCode::F(_) => None,

        // All other keys — including every v1.x letter binding stripped
        // per D-25.3 (C, T, L, G, E, H, I, W, Y, q, a, c, k, s, g, z, Z,
        // m, D, y, b, O, V, h, j, J) — are silently unmapped.
        _ => None,
    }
}

/// Map a key pressed AFTER an armed f-prefix to its HP-41CV f-shifted Op.
///
/// Phase 25 / Plan 01 (D-25.7) wires the **four** hardware-anchored
/// conditional tests bound to the f-shifted arithmetic keys on the user's
/// physical HP-41CV:
///
/// | Key  | Op                         | Mnemonic |
/// |------|----------------------------|----------|
/// | `f-` | `Op::Test(TestKind::XEqY)` | X=Y      |
/// | `f+` | `Op::Test(TestKind::XLeY)` | X≤Y      |
/// | `f*` | `Op::Test(TestKind::XGtY)` | X>Y      |
/// | `f/` | `Op::Test(TestKind::XEqZero)` | X=0   |
///
/// These four are the **only** conditional tests on the physical HP-41CV
/// keyboard (D-25.7); the other eight (X≠Y, X<Y, X≥Y, X≠0, X<0, X>0,
/// X≤0, X≥0) are reached via the XEQ-by-Name modal per D-25.8/D-25.9
/// (Plan 03 wires the modal resolver).
///
/// Plan 02 extends this resolver with modal-opener f-shifted bindings
/// (SF/CF/VIEW/TONE/…) once those modals exist. Plan 04 may rebuild the
/// table entirely from `docs/hp41cv-functions.json` per D-25.18. Returning
/// `None` here is silent — the caller in `App::handle_key` always clears
/// the `shift_armed` flag regardless (Pitfall 5).
pub fn shifted_key_to_op(key: KeyEvent, _app: &App) -> Option<Op> {
    match key.code {
        // D-25.7 — four hardware-anchored conditional tests on the
        // f-shifted arithmetic keys.
        KeyCode::Char('-') => Some(Op::Test(TestKind::XEqY)),
        KeyCode::Char('+') => Some(Op::Test(TestKind::XLeY)),
        KeyCode::Char('*') => Some(Op::Test(TestKind::XGtY)),
        KeyCode::Char('/') => Some(Op::Test(TestKind::XEqZero)),
        // Modal-opener f-shifted bindings (SF/CF/VIEW/TONE/…) land in
        // Plan 02. Everything else is unmapped — the caller clears
        // shift_armed regardless (Pitfall 5).
        _ => None,
    }
}

/// Key-reference table for the TUI right panel (INPUT-01 discoverability).
/// Shown verbatim in ui.rs render_right_panel().
///
/// **Plan 25-01 note (D-25.18):** This hand-curated table is the **v1.x** key
/// reference. Plan 25-04 rebuilds it from `docs/hp41cv-functions.json` so the
/// JSON is the single source of truth for both CLI discoverability and the
/// GUI ?-overlay (FN-CLI-01 + FN-DOC-02). Until Plan 04 lands, this table is
/// left untouched but its content (especially the v1.x letter bindings on
/// the right side) is **stale** — Plan 01 strips those bindings from
/// `key_to_op()` itself in the next task. Do not extend this table by hand;
/// add entries to the JSON instead.
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
    ("q", "SIN (sine of X in current angle mode)"),
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
    (
        "%",
        "%CH (percent change: ((X\u{2212}Y)/Y)\u{00D7}100, Y preserved)",
    ),
    ("p", "PRGM toggle"),
    ("d", "cycle DEG/RAD/GRAD"),
    ("f", "cycle FIX/SCI/ENG (keeps digit count)"),
    ("F5", "R/S (run program A)"),
    ("F7", "SST (step forward)"),
    ("F8", "BST (step back)"),
    ("^C", "quit"),
    ("g", "CLREG (clear all storage registers R00-R99)"),
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
    (
        "F",
        "FIX/SCI/ENG n modal (set exact digit count 0\u{2013}9)",
    ),
    ("h", "HMS\u{2192} (H.MMSS to decimal hours)"),
    ("j", "HMS+  (add two H.MMSS values, base-60 carry)"),
    ("J", "HMS-  (subtract H.MMSS values, base-60 borrow)"),
    // Phase 12: synthetic programming
    (
        "X nn",
        "Insert synthetic hex byte at current PC (PRGM mode only)",
    ),
    // Card Reader comfort shortcuts (Ctrl+W/R/D/F)
    ("Ctrl+W", "WPRGM (write current program to card)"),
    ("Ctrl+R", "RDPRGM (read program from card)"),
    ("Ctrl+D", "WDTA (write data registers to card)"),
    ("Ctrl+F", "RDTA (read data registers from card)"),
];

/// Map a crossterm KeyCode to the HP-41 hardware key code (row×10 + col, 1-indexed).
/// Returns 0 for keys with no HP-41 hardware equivalent (function keys, Ctrl combos, etc.).
/// Called from `App::handle_key()` to update `CalcState.last_key_code` on every Press
/// event (D-01). Read by `Op::GetKey` to push the last key code to X (SYNT-01).
///
/// HP-41C keyboard layout: 8 rows × 5 columns. Key code = row × 10 + col.
/// Rows are numbered 1-8 top-to-bottom, columns 1-5 left-to-right.
/// Row 1: Σ+(11), 1/x(12), √x(13), LOG(14), LN(15)
/// Row 2: XEQ(21), STO(22), RCL(23), R↓(24), SIN(25)
/// Row 3: R/S(31), SST(32), GTO(33), COS(34), TAN(35)
/// Row 4: USER(41), f(42), g(43), ENTER(44), ÷(45)
/// Row 5: 7(51), 8(52), 9(53), ×(54)
/// Row 6: 4(61), 5(62), 6(63), −(64)
/// Row 7: 1(71), 2(72), 3(73), +(74)
/// Row 8: 0(81), .(82), EEX(83), R/S(84), ENTER(85) [rows from HP-41C Owner's Manual Appendix A]
///
/// [ASSUMED — rows 1-4 column assignments; rows 5-8 digit/arithmetic keys are certain.
///  See CONTEXT.md D-02 and RESEARCH.md A1.]
/// Returns `Some(code)` for keys that correspond to physical HP-41 calculator keys.
/// Returns `None` for TUI-only keys (F5/F7/F8) and unmapped keys.
///
/// Callers must only update `last_key_code` when `Some` is returned — `None` means
/// the keypress has no HP-41 hardware equivalent and must not corrupt GETKEY state.
pub fn keycode_to_hp41_code(code: crossterm::event::KeyCode) -> Option<u8> {
    use crossterm::event::KeyCode;
    Some(match code {
        // Row 8: 0(81), .(82), EEX(83), ENTER(84/85) — digit/arithmetic row (bottom)
        KeyCode::Char('0') => 81,
        KeyCode::Char('.') => 82,
        KeyCode::Char('e') => 83, // EEX
        KeyCode::Enter => 84,     // ENTER (row 8, col 4 in some HP-41C variants)
        // Row 7: 1(71), 2(72), 3(73), +(74)
        KeyCode::Char('1') => 71,
        KeyCode::Char('2') => 72,
        KeyCode::Char('3') => 73,
        KeyCode::Char('+') => 74,
        // Row 6: 4(61), 5(62), 6(63), −(64)
        KeyCode::Char('4') => 61,
        KeyCode::Char('5') => 62,
        KeyCode::Char('6') => 63,
        KeyCode::Char('-') => 64,
        // Row 5: 7(51), 8(52), 9(53), ×(54)
        KeyCode::Char('7') => 51,
        KeyCode::Char('8') => 52,
        KeyCode::Char('9') => 53,
        KeyCode::Char('*') => 54,
        // Row 4: USER(41), f(42), g(43), ENTER(44), ÷(45)
        // [ASSUMED — row 4 column assignments from HP-41C Owner's Manual]
        KeyCode::Char('u') | KeyCode::Char('U') => 41, // USER mode toggle
        KeyCode::Char('f') => 42,                      // f-key (format cycle)
        KeyCode::Char('g') => 43,                      // g-key (CLREG)
        KeyCode::Char('/') => 45,                      // ÷
        // Row 3: R/S(31), SST(32), GTO(33), COS(34), TAN(35)
        // [ASSUMED — row 3 column assignments]
        // F5/F7/F8 are TUI-only bindings with no physical HP-41 key equivalent.
        // They must not update last_key_code — caller checks for None.
        KeyCode::F(5) | KeyCode::F(7) | KeyCode::F(8) => return None,
        KeyCode::Char('C') => 34, // COS (uppercase, Shift+C)
        KeyCode::Char('T') => 35, // TAN (uppercase, Shift+T)
        // Row 2: XEQ(21), STO(22), RCL(23), R↓(24), SIN(25)
        // [ASSUMED — row 2 column assignments match Phase 8 TUI key assignments]
        KeyCode::Char('X') => 21, // XEQ
        KeyCode::Char('S') => 22, // STO modal opener
        KeyCode::Char('R') => 23, // RCL modal opener
        KeyCode::Char('r') => 24, // R↓ (lowercase r — roll down)
        KeyCode::Char('q') => 25, // SIN (Phase 8 reassignment to 'q')
        // Row 1: Σ+(11), 1/x(12), √x(13), LOG(14), LN(15) — top function row
        // [ASSUMED — row 1 column assignments]
        KeyCode::Char('z') => 11, // Σ+
        KeyCode::Char('I') => 12, // 1/x (uppercase I, Shift+I)
        KeyCode::Char('s') => 13, // √x (lowercase s)
        KeyCode::Char('G') => 14, // LOG (uppercase G, Shift+G)
        KeyCode::Char('L') => 15, // LN (uppercase L, Shift+L)
        // All other keys: no HP-41 hardware equivalent.
        _ => return None,
    })
}

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
            "UserMode dispatch must not error: {result:?}"
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

    #[test]
    fn test_q_dispatches_sin() {
        // 'q' maps to Op::Sin — verify the op produces the correct result: sin(30 DEG) = 0.5
        let mut state = CalcState::new(); // angle_mode is DEG by default
        state.stack.x = hp41_core::HpNum::from(30);
        let result = hp41_core::ops::dispatch(&mut state, Op::Sin);
        assert!(result.is_ok(), "Op::Sin must not error on valid input");
        assert_eq!(
            format!("{}", state.stack.x),
            "0.5000000000",
            "sin(30 DEG) must equal 0.5 (10 significant digits)"
        );
    }

    #[test]
    fn test_g_dispatches_clreg() {
        // 'g' maps to Op::Clreg — verify all storage registers are zeroed
        let mut state = CalcState::new();
        state.regs[5] = hp41_core::HpNum::from(42);
        let result = hp41_core::ops::dispatch(&mut state, Op::Clreg);
        assert!(result.is_ok(), "Op::Clreg must not error");
        assert!(
            state.regs.iter().all(|r| r.is_zero()),
            "CLREG must zero all storage registers"
        );
    }

    #[test]
    fn test_pct_keystroke_dispatches_pct_change() {
        // '%' maps to Op::PctChange — verify Y=100 base, X=125 new value → 25% change, Y preserved.
        // Compare HpNum values directly (PartialEq) rather than Display strings, which are
        // rust_decimal-scale-dependent and would break if HpNum::rounded() normalises trailing zeros.
        let mut state = CalcState::new();
        state.stack.y = hp41_core::HpNum::from(100);
        state.stack.x = hp41_core::HpNum::from(125);
        let result = hp41_core::ops::dispatch(&mut state, Op::PctChange);
        assert!(
            result.is_ok(),
            "Op::PctChange must not error on valid input"
        );
        assert_eq!(
            state.stack.x,
            hp41_core::HpNum::from(25),
            "%CH(100→125) must be 25"
        );
        assert_eq!(
            state.stack.y,
            hp41_core::HpNum::from(100),
            "Y must be preserved"
        );
    }

    #[test]
    fn test_key_ref_table_has_pct_entry() {
        let has_pct = KEY_REF_TABLE
            .iter()
            .any(|(k, desc)| *k == "%" && desc.contains("%CH"));
        assert!(has_pct, "KEY_REF_TABLE must contain a '%' → %CH entry");
    }
}
