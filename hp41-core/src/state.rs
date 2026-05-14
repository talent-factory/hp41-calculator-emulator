// ADR-001: Numeric representation — rust_decimal 1.41 with HpNum newtype
//
// Decision date: Phase 1 (2026-05)
// Decision: Use `rust_decimal::Decimal` wrapped in `HpNum` newtype rather than:
//   (a) custom BCD nibble arithmetic, or
//   (b) f64 with manual rounding.
//
// Rationale:
//   - HP-41 hardware stores 10-digit BCD mantissa + 2-digit exponent (56-bit register).
//     Behavioral emulation does not require bit-identical storage — only identical outputs.
//   - `rust_decimal` is decimal-native (no binary float rounding artifacts like 0.1+0.2≠0.3).
//   - `round_sf_with_strategy(10, MidpointAwayFromZero)` enforces HP-41's 10-significant-digit
//     display precision with correct rounding direction (not Bankers rounding).
//   - A custom BCD struct would add ~500 LOC of nibble arithmetic with identical user-visible
//     behavior. The only scenario where this decision must be revisited is if Phase 7 QUAL-06
//     (≥98% numerical agreement, 500-case suite) reveals precision gaps that rust_decimal
//     cannot bridge — at which point a custom BCD struct replaces HpNum's inner type only.
//   - ISG/DSE counter fields (CCCCC.FFFDD) are extracted by string-splitting at the decimal
//     point regardless of representation — never via floor()/fmod() on f64.
//
// Consequences:
//   - All arithmetic in hp41-core flows through HpNum checked_* methods.
//   - Phase 2 adds `features = ["maths"]` to rust_decimal for ln/exp/pow.
//   - No f64 arithmetic on HP-41 register values anywhere in hp41-core.

use crate::num::HpNum;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Trigonometric angle mode — controls input/output units for SIN/COS/TAN/ASIN/ACOS/ATAN.
/// Default: Deg (HP-41 hardware cold-start default).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AngleMode {
    Deg,
    Rad,
    Grad,
}

/// Number display mode — controls how HpNum values are formatted for display.
/// u8 field = digit count (0–9).
/// Default: Fix(4) (HP-41 hardware cold-start default).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DisplayMode {
    Fix(u8),
    Sci(u8),
    Eng(u8),
}

/// The complete, mutable state of the HP-41 calculator.
///
/// All operations take `&mut CalcState`. No global mutable state anywhere.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcState {
    pub stack: Stack,
    /// Storage registers R00–R99 (0-indexed). All zero on startup.
    pub regs: Vec<HpNum>,
    /// ALPHA register — up to 24 characters.
    pub alpha_reg: String,
    /// true = keyboard routes chars to alpha_reg instead of entry_buf.
    pub alpha_mode: bool,
    /// Trig angle mode (DEG/RAD/GRAD). Default: DEG.
    pub angle_mode: AngleMode,
    /// Number display mode (FIX/SCI/ENG n). Default: FIX 4.
    pub display_mode: DisplayMode,
    /// Pending digit string for number entry. Empty when not in digit-entry state.
    pub entry_buf: String,
    // ── Phase 3: Programming Engine ──────────────────────────────────────────
    /// Keystroke program storage. Flat list — Op::Lbl markers delimit subroutines.
    /// D-01: single contiguous Vec<Op> matching HP-41 flat program memory.
    pub program: Vec<crate::ops::Op>,
    /// PRGM mode: when true dispatch() records ops to program instead of executing.
    /// D-03: gate is checked at the top of dispatch().
    pub prgm_mode: bool,
    /// Program counter — index of the next op to execute in `program`.
    /// D-05: set by run_program(); 0 at startup.
    pub pc: usize,
    /// Subroutine return stack. Max 4 entries (HP-41 hardware limit, D-14).
    pub call_stack: Vec<usize>,
    /// True while run_program() is active; guards against re-entrancy.
    /// D-06: reset to false even on error path.
    pub is_running: bool,
    // ── Phase 5: USER mode & key assignments ─────────────────────────────────
    /// USER mode active: when true, key_assignments are consulted before normal dispatch.
    /// D-25: default false.
    pub user_mode: bool,
    /// User key assignments: maps key char → program label name.
    /// BTreeMap for deterministic JSON serialization order (D-25, D-29).
    pub key_assignments: BTreeMap<char, String>,
    /// Buffer of formatted print lines from PRX/PRA/PRSTK.
    /// Drained by hp41-cli after each dispatch. Never persisted across sessions.
    /// #[serde(default, skip)] — default enables backward-compat deserialization of older
    /// save files that lack this field; skip prevents serialization of transient state.
    #[serde(default, skip)]
    pub print_buffer: Vec<String>,
    /// Last HP-41 row-column key code pressed (row×10+col, 1-indexed). 0 = none since startup.
    /// Updated by hp41-cli `handle_key()` on every Press event. Read by `Op::GetKey`.
    /// Default: 0. Persistent across save/load (#[serde(default)]).
    #[serde(default)]
    pub last_key_code: u8,

    /// Hidden register M — accessible via STO M / RCL M in programs.
    /// Not part of the numbered `regs: Vec<HpNum>`. Default: HpNum::zero().
    #[serde(default)]
    pub reg_m: HpNum,

    /// Hidden register N — accessible via STO N / RCL N in programs.
    #[serde(default)]
    pub reg_n: HpNum,

    /// Hidden register O — accessible via STO O / RCL O in programs.
    #[serde(default)]
    pub reg_o: HpNum,

    // ── Phase 21: Flags ──────────────────────────────────────────────────────
    /// HP-41 flags (user flags 0-29 + system flags 30-55) packed into a single u64.
    /// Bit n = flag n. Default: 0 (all clear). Use `ops::flags` helpers for safe access.
    /// `#[serde(default)]` — a v2.0 autosave.json without this field loads cleanly with flags == 0.
    /// Phase 21 (FN-FLAG-01).
    #[serde(default)]
    pub flags: u64,
    /// Pending card I/O request set by `Op::Wdta`/`Op::Rdta`/`Op::Wprgm`/`Op::Rdprgm`.
    /// The frontend (hp41-cli / hp41-gui) drains this after each `dispatch()` and
    /// performs the actual disk I/O — keeps hp41-core UI-agnostic. Mirrors the
    /// `print_buffer` drain pattern.
    #[serde(default, skip)]
    pub pending_card_op: Option<crate::cardreader::CardOpRequest>,
}

impl CalcState {
    pub fn new() -> Self {
        CalcState {
            stack: Stack::new(),
            regs: vec![HpNum::zero(); 100],
            alpha_reg: String::new(),
            alpha_mode: false,
            angle_mode: AngleMode::Deg,
            display_mode: DisplayMode::Fix(4),
            entry_buf: String::new(),
            program: Vec::new(),
            prgm_mode: false,
            pc: 0,
            call_stack: Vec::new(),
            is_running: false,
            user_mode: false,
            key_assignments: BTreeMap::new(),
            print_buffer: Vec::new(),
            last_key_code: 0,
            reg_m: HpNum::zero(),
            reg_n: HpNum::zero(),
            reg_o: HpNum::zero(),
            flags: 0,
            pending_card_op: None,
        }
    }
}

impl Default for CalcState {
    fn default() -> Self {
        Self::new()
    }
}

/// The HP-41 4-level RPN stack with LASTX and stack-lift flag.
///
/// Registers: X (visible), Y, Z, T (bottom). T is dropped (overwritten by Z) on lift;
/// T is duplicated (not consumed) on stack drop (binary result).
/// lift_enabled: true means the next number entry will lift the stack before writing X.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    /// X register — the currently visible value
    pub x: HpNum,
    /// Y register
    pub y: HpNum,
    /// Z register
    pub z: HpNum,
    /// T register — dropped (overwritten by Z) on lift; duplicated (not consumed) on stack drop
    pub t: HpNum,
    /// LASTX — captures X before it is consumed by a binary operation
    pub lastx: HpNum,
    /// Stack-lift flag: true = next number entry lifts; false = overwrites X
    pub lift_enabled: bool,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            x: HpNum::zero(),
            y: HpNum::zero(),
            z: HpNum::zero(),
            t: HpNum::zero(),
            lastx: HpNum::zero(),
            lift_enabled: false,
        }
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}
