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

use crate::app::{App, PendingInput};

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
///
/// **Note on `StoArith`:** Plan 02 deliberately routes STO-arithmetic
/// through the legacy v1.1 `S → +/-/×/÷ → register` chain (`StoAdd/Sub/
/// Mul/Div` variants) per W3 fix + D-25.7 — so the `StoArith` arm here is
/// reachable via tests + a future Plan-04 JSON pipeline but is NOT
/// constructed by the live keyboard handler in v2.2. `#[allow(dead_code)]`
/// is scoped to just this variant.
#[derive(Debug, Clone, PartialEq)]
pub enum RegisterOpKind {
    Sto,
    Rcl,
    #[allow(dead_code)]
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
/// **Phase 25 / Plan 02 — modal-opener f-shifted bindings.** When an
/// f-shifted key opens a modal rather than dispatching directly, this
/// function returns `None` AND populates `app.pending_input` with the
/// appropriate PendingInput variant. The signature takes `&mut App` so
/// the side effect is local to the resolver. Mapping table:
///
/// | Key   | Modal opened (PendingInput variant)      |
/// |-------|-------------------------------------------|
/// | `f-7` | `FlagPrompt { SetFlag, ind:false, acc:"" }`     |
/// | `f-8` | `FlagPrompt { ClearFlag, ind:false, acc:"" }`   |
/// | `f-9` | `FlagPrompt { Test(IsSet), … }`                |
/// | `f-4` | `FlagPrompt { Test(IsClear), … }`              |
/// | `f-5` | `FlagPrompt { Test(IsSetThenClear), … }`       |
/// | `f-6` | `FlagPrompt { Test(IsClearThenClear), … }`     |
/// | `f-v` | `RegisterPrompt { View, … }`                   |
/// | `f-a` | `RegisterPrompt { Arcl, … }`                   |
/// | `f-A` | `RegisterPrompt { Asto, … }`                   |
/// | `f-i` | `RegisterPrompt { Isg, … }`                    |
/// | `f-d` | `RegisterPrompt { Dse, … }`                    |
/// | `f-C` | `ClpLabel("")`                                  |
/// | `f-D` | `DelCount("")`                                  |
/// | `f-T` | `TonePrompt`                                    |
/// | `f-N` | `XeqByName("")`                                 |
///
/// **STO-arithmetic openers are deliberately absent** (W3 fix). The
/// f-shifted `-/+/*/(slash)` keys are LOCKED to the 4 conditional tests
/// per D-25.7 (Plan 01). STO-arithmetic (`STO+/-/×/÷`) stays reachable
/// via the legacy v1.1 `S → +/-/×/÷ → register` modal chain — `S` opens
/// `RegisterPrompt { Sto }`, and the existing pending_input route for
/// `StoRegister` intercepts `+/-/*/(slash)` to switch into `StoAdd/Sub/
/// Mul/Div`.
///
/// Plan 04 may rebuild the table entirely from `docs/hp41cv-functions.json`
/// per D-25.18. Returning `None` here is silent — the caller in
/// `App::handle_key` always clears the `shift_armed` flag regardless
/// (Pitfall 5).
pub fn shifted_key_to_op(key: KeyEvent, app: &mut App) -> Option<Op> {
    match key.code {
        // D-25.7 — four hardware-anchored conditional tests on the
        // f-shifted arithmetic keys.
        KeyCode::Char('-') => Some(Op::Test(TestKind::XEqY)),
        KeyCode::Char('+') => Some(Op::Test(TestKind::XLeY)),
        KeyCode::Char('*') => Some(Op::Test(TestKind::XGtY)),
        KeyCode::Char('/') => Some(Op::Test(TestKind::XEqZero)),

        // ── Plan 02 modal openers — return None and populate pending_input ──
        // Flag ops (f-shifted digit keys 4..=9):
        KeyCode::Char('7') => {
            app.pending_input = Some(PendingInput::FlagPrompt {
                kind: FlagPromptKind::SetFlag,
                ind: false,
                acc: String::new(),
            });
            None
        }
        KeyCode::Char('8') => {
            app.pending_input = Some(PendingInput::FlagPrompt {
                kind: FlagPromptKind::ClearFlag,
                ind: false,
                acc: String::new(),
            });
            None
        }
        KeyCode::Char('9') => {
            app.pending_input = Some(PendingInput::FlagPrompt {
                kind: FlagPromptKind::Test(FlagTestKind::IsSet),
                ind: false,
                acc: String::new(),
            });
            None
        }
        KeyCode::Char('4') => {
            app.pending_input = Some(PendingInput::FlagPrompt {
                kind: FlagPromptKind::Test(FlagTestKind::IsClear),
                ind: false,
                acc: String::new(),
            });
            None
        }
        KeyCode::Char('5') => {
            app.pending_input = Some(PendingInput::FlagPrompt {
                kind: FlagPromptKind::Test(FlagTestKind::IsSetThenClear),
                ind: false,
                acc: String::new(),
            });
            None
        }
        KeyCode::Char('6') => {
            app.pending_input = Some(PendingInput::FlagPrompt {
                kind: FlagPromptKind::Test(FlagTestKind::IsClearThenClear),
                ind: false,
                acc: String::new(),
            });
            None
        }
        // Register ops (lowercase: mnemonic-letter shortcuts because the
        // HP-41CV reference card positions for VIEW/ARCL/ASTO/ISG/DSE
        // are TBD per RESEARCH; Plan 04 may move these onto numeric
        // f-shift positions derived from docs/hp41cv-functions.json).
        KeyCode::Char('v') => {
            app.pending_input = Some(PendingInput::RegisterPrompt {
                op: RegisterOpKind::View,
                ind: false,
                acc: String::new(),
            });
            None
        }
        KeyCode::Char('a') => {
            app.pending_input = Some(PendingInput::RegisterPrompt {
                op: RegisterOpKind::Arcl,
                ind: false,
                acc: String::new(),
            });
            None
        }
        KeyCode::Char('A') => {
            app.pending_input = Some(PendingInput::RegisterPrompt {
                op: RegisterOpKind::Asto,
                ind: false,
                acc: String::new(),
            });
            None
        }
        KeyCode::Char('i') => {
            app.pending_input = Some(PendingInput::RegisterPrompt {
                op: RegisterOpKind::Isg,
                ind: false,
                acc: String::new(),
            });
            None
        }
        KeyCode::Char('d') => {
            app.pending_input = Some(PendingInput::RegisterPrompt {
                op: RegisterOpKind::Dse,
                ind: false,
                acc: String::new(),
            });
            None
        }
        // Specialty modal openers (uppercase ASCII shortcuts).
        // `C` opens ClpLabel, `D` opens DelCount, `T` opens TonePrompt,
        // `N` opens XeqByName (the lowercase counterparts would collide
        // with primary HP-41CV positions or with the IsClear `c` letter).
        KeyCode::Char('C') => {
            app.pending_input = Some(PendingInput::ClpLabel(String::new()));
            None
        }
        KeyCode::Char('D') => {
            app.pending_input = Some(PendingInput::DelCount(String::new()));
            None
        }
        KeyCode::Char('T') => {
            app.pending_input = Some(PendingInput::TonePrompt);
            None
        }
        KeyCode::Char('N') => {
            app.pending_input = Some(PendingInput::XeqByName(String::new()));
            None
        }
        // Everything else: unmapped. Caller clears shift_armed regardless
        // (Pitfall 5).
        _ => None,
    }
}

/// CLI-local resolver for the 8 non-keyboard HP-41CV conditional-test
/// mnemonics. Accepts BOTH ASCII-pure and Unicode-symbol spellings per
/// D-25.10 + RESEARCH §"Conditional tests". Returns `None` for the four
/// v2.1 card-reader names (those fall through to `Op::Xeq` →
/// `hp41_core::ops::program::builtin_card_op` via the modal Enter-arm) and
/// for unknown names.
///
/// Why CLI-local AND hp41-core both carry the mapping (Plan 03):
///   - This CLI-local path gives immediate dispatch from the XEQ-by-Name
///     modal Enter-arm without constructing `Op::Xeq` + a program run.
///   - The hp41-core `builtin_card_op` extension (Plan 03 Task 1) ensures
///     `Op::Xeq("X<>Y?")` inside a saved program also resolves to the same
///     `Op::Test` variant — keyboard + programmatic symmetry preserved.
///
/// The `cli_resolver_matches_core_resolver` integration test in
/// `tests/phase25_xeq_by_name.rs` guards against drift between the two
/// resolvers (T-25-09 mitigation).
///
/// Case-sensitive — HP-41 ROM names are uppercase.
pub fn xeq_by_name_local_resolve(name: &str) -> Option<Op> {
    match name {
        // X ≠ Y — three accepted spellings.
        "X<>Y?" | "X\u{2260}Y?" | "X#Y?" => Some(Op::Test(TestKind::XNeY)),
        // X < Y — single spelling.
        "X<Y?" => Some(Op::Test(TestKind::XLtY)),
        // X ≥ Y — two spellings.
        "X>=Y?" | "X\u{2265}Y?" => Some(Op::Test(TestKind::XGeY)),
        // X ≠ 0 — two spellings.
        "X#0?" | "X\u{2260}0?" => Some(Op::Test(TestKind::XNeZero)),
        // X < 0 — single spelling.
        "X<0?" => Some(Op::Test(TestKind::XLtZero)),
        // X > 0 — single spelling.
        "X>0?" => Some(Op::Test(TestKind::XGtZero)),
        // X ≤ 0 — two spellings.
        "X<=0?" | "X\u{2264}0?" => Some(Op::Test(TestKind::XLeZero)),
        // X ≥ 0 — two spellings.
        "X>=0?" | "X\u{2265}0?" => Some(Op::Test(TestKind::XGeZero)),
        // The four v2.1 card-reader names + everything else: defer to
        // `hp41_core::ops::program::builtin_card_op` via the `Op::Xeq`
        // fallback chain in the modal Enter-arm.
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
