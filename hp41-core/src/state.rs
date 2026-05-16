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
    /// HP-41 ASN key assignments: maps hardware key code (row×10+col, 1-indexed)
    /// → assigned target name. Phase 22 (FN-KEY-01). Coexists with key_assignments
    /// (Phase 5, char-keyed) — Phase 25/26 reconciles the two maps.
    /// `#[serde(default)]` keeps v1.0–v2.1 save files loadable (default → empty map).
    #[serde(default)]
    pub assignments: BTreeMap<u8, String>,
    /// HP-41 packed-text register shadows: ASTO writes a 6-char string here,
    /// ARCL reads from here in preference to formatting the numeric `regs[reg]`.
    /// Numeric STO / op_sto_arith / op_clreg CLEAR the matching entry to keep
    /// the two representations from drifting (D-23.4). `op_sto_arith_stack`
    /// targets the Y/Z/T/LastX stack registers (not numbered regs) so it does
    /// not touch this map.
    /// `#[serde(default)]` keeps v1.x–v2.1 save files loadable (default → empty map).
    /// Phase 23 (FN-ALPHA-01, FN-ALPHA-02).
    #[serde(default)]
    pub text_regs: BTreeMap<u8, String>,
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

    // ── Phase 21: Display Control ────────────────────────────────────────────
    /// HP-41 display override channel: VIEW/AVIEW/PROMPT/CLD write to this.
    /// None = render normal display. Transient — cleared at the top of dispatch
    /// and never persisted (`#[serde(default, skip)]`). Phase 21 (FN-DISP-01..05).
    #[serde(default, skip)]
    pub display_override: Option<String>,

    // ── Phase 21: Sound ──────────────────────────────────────────────────────
    /// HP-41 sound event buffer: BEEP and TONE n push structured event lines here.
    /// Drained by hp41-cli / hp41-gui after each dispatch — frontend plays audio.
    /// Transient — never persisted (`#[serde(default, skip)]`).
    /// Phase 21 (FN-SOUND-01 / FN-SOUND-02).
    #[serde(default, skip)]
    pub event_buffer: Vec<String>,
    /// Pending card I/O request set by `Op::Wdta`/`Op::Rdta`/`Op::Wprgm`/`Op::Rdprgm`.
    /// The frontend (hp41-cli / hp41-gui) drains this after each `dispatch()` and
    /// performs the actual disk I/O — keeps hp41-core UI-agnostic. Mirrors the
    /// `print_buffer` drain pattern.
    #[serde(default, skip)]
    pub pending_card_op: Option<crate::cardreader::CardOpRequest>,

    // ── Phase 28 (v3.0): XROM framework + Math Pac I ────────────────────────

    /// Bitfield of loaded XROM modules. Bit 0 = Math 1 loaded.
    /// Default: 0b0000_0001 (Math 1 pre-loaded per v3.0 scope).
    /// Persistent across save/load. `#[serde(default = "default_xrom_modules")]`.
    #[serde(default = "default_xrom_modules")]
    pub xrom_modules: u8,

    /// Complex stack overlay mode (D-28.1 / D-28.2). When true, X+iY form
    /// the complex number ζ and Z+iT form τ. Auto-on at first complex op;
    /// explicit `XEQ "REAL"` (D-28.3) deactivates. Safe default: false.
    #[serde(default)]
    pub complex_mode: bool,

    /// Current matrix dimension (rows, cols) for MATRIX workflow (Plan 28-06).
    /// None = no matrix active. Persistent (matrix shape survives save/load).
    #[serde(default)]
    pub matrix_dim: Option<(u8, u8)>,

    /// Active matrix register index (for MATRIX element-edit mode, Plan 28-06).
    /// None = not editing a matrix register. Persistent.
    #[serde(default)]
    pub matrix_active_reg: Option<u8>,

    /// Active modal program (MATRIX/SOLVE/POLY/INTG/DIFEQ/FOUR/TRANS).
    /// Transient — set on modal-open, cleared on completion or cancel.
    /// Never persisted (`#[serde(default, skip)]`).
    #[serde(default, skip)]
    pub modal_program: Option<crate::ops::math1::modal::ModalProgram>,

    /// Modal prompt text for active workflow step.
    /// CLI renders in `pending_prompt()` (Phase 29 wiring).
    /// GUI renders as overlay banner above LCD (Phase 31 wiring).
    /// R/S key submits numeric input per D-28.5 (CLI/GUI wiring in Phase 29/31).
    /// Transient — never persisted (`#[serde(default, skip)]`).
    #[serde(default, skip)]
    pub modal_prompt: Option<String>,

    /// Mid-iteration state for INTG numerical integration (Plan 28-07).
    /// Transient — never persisted (`#[serde(default, skip)]`).
    /// Placeholder stub; Plan 28-07 fills fields.
    #[serde(default, skip)]
    pub integ_state: Option<crate::ops::math1::integ::IntegState>,

    /// Mid-iteration state for SOLVE root-finding (Plan 28-08).
    /// Transient — never persisted (`#[serde(default, skip)]`).
    /// Placeholder stub; Plan 28-08 fills fields.
    #[serde(default, skip)]
    pub solve_state: Option<crate::ops::math1::solve::SolveState>,

    /// Mid-iteration state for DIFEQ ODE solver (Plan 28-09).
    /// Transient — never persisted (`#[serde(default, skip)]`).
    /// RESEARCH Open Q2 recommendation (a): early commitment.
    /// Placeholder stub; Plan 28-09 fills fields.
    #[serde(default, skip)]
    pub difeq_state: Option<crate::ops::math1::difeq::DifeqState>,

    /// Cancellation flag for long-running solvers (INTG/SOLVE/DIFEQ).
    /// `Arc<AtomicBool>` so Phase 31 `request_cancel` Tauri command can set it
    /// from the GUI thread without locking the `AppState` Mutex (D-28.7).
    /// Per-64-samples check: `cancel_requested.load(Relaxed)` inside solver loops
    /// (D-28.8). Reset to `false` at every op_integ/op_solve/op_difeq entry.
    /// Transient — never persisted (`#[serde(default = "default_cancel_requested", skip)]`).
    #[serde(default = "default_cancel_requested", skip)]
    pub cancel_requested: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

// ── serde-default helpers ────────────────────────────────────────────────────

/// Default value for `xrom_modules`: bit 0 = Math 1 pre-loaded.
fn default_xrom_modules() -> u8 {
    0b0000_0001
}

/// Default value for `cancel_requested`: a new Arc<AtomicBool> initialized to false.
fn default_cancel_requested() -> std::sync::Arc<std::sync::atomic::AtomicBool> {
    std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false))
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
            assignments: BTreeMap::new(),
            text_regs: BTreeMap::new(),
            print_buffer: Vec::new(),
            last_key_code: 0,
            reg_m: HpNum::zero(),
            reg_n: HpNum::zero(),
            reg_o: HpNum::zero(),
            flags: 0,
            display_override: None,
            event_buffer: Vec::new(),
            pending_card_op: None,
            // Phase 28 (v3.0) fields
            xrom_modules: default_xrom_modules(),
            complex_mode: false,
            matrix_dim: None,
            matrix_active_reg: None,
            modal_program: None,
            modal_prompt: None,
            integ_state: None,
            solve_state: None,
            difeq_state: None,
            cancel_requested: default_cancel_requested(),
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    // Catches: default initializer regression for Phase 28 fields
    #[test]
    fn default_construction_phase28_fields() {
        let state = CalcState::default();
        assert_eq!(state.xrom_modules, 0b0000_0001, "Math 1 must be pre-loaded by default");
        assert!(!state.complex_mode, "complex_mode must default to false");
        assert_eq!(state.matrix_dim, None, "matrix_dim must default to None");
        assert_eq!(state.matrix_active_reg, None, "matrix_active_reg must default to None");
        assert!(state.modal_program.is_none(), "modal_program must default to None");
        assert!(state.modal_prompt.is_none(), "modal_prompt must default to None");
        assert!(state.integ_state.is_none(), "integ_state must default to None");
        assert!(state.solve_state.is_none(), "solve_state must default to None");
        assert!(state.difeq_state.is_none(), "difeq_state must default to None");
        assert!(
            !state.cancel_requested.load(Ordering::Relaxed),
            "cancel_requested must default to false"
        );
    }

    // Catches: serde(skip) on transient fields — they must NOT appear in serialized output;
    //          serde(default) on persistent fields — they must survive round-trip.
    #[test]
    fn serde_roundtrip() {
        let mut state = CalcState::new();
        // Set some transient fields to non-default values
        state.modal_prompt = Some("ORDER=?".to_string());
        state.integ_state = Some(crate::ops::math1::integ::IntegState::default());
        // Set persistent fields
        state.xrom_modules = 0b0000_0011; // Math 1 + hypothetical module 2
        state.complex_mode = true;
        state.matrix_dim = Some((3, 3));
        state.matrix_active_reg = Some(5);

        let json = serde_json::to_string(&state).unwrap();

        // Transient fields must NOT appear in serialized output
        assert!(!json.contains("modal_prompt"), "modal_prompt must be serde(skip)");
        assert!(!json.contains("integ_state"), "integ_state must be serde(skip)");
        assert!(!json.contains("cancel_requested"), "cancel_requested must be serde(skip)");

        // Persistent fields must appear in serialized output
        assert!(json.contains("xrom_modules"), "xrom_modules must be serialized");
        assert!(json.contains("complex_mode"), "complex_mode must be serialized");
        assert!(json.contains("matrix_dim"), "matrix_dim must be serialized");

        let restored: CalcState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.xrom_modules, 0b0000_0011);
        assert!(restored.complex_mode);
        assert_eq!(restored.matrix_dim, Some((3, 3)));
        assert_eq!(restored.matrix_active_reg, Some(5));

        // Transient fields reset to defaults after round-trip
        assert!(restored.modal_prompt.is_none(), "modal_prompt must reset to None after deserialization");
        assert!(restored.integ_state.is_none(), "integ_state must reset to None after deserialization");
        assert!(
            !restored.cancel_requested.load(Ordering::Relaxed),
            "cancel_requested must reset to false after deserialization"
        );
    }

    // Catches: v2.2 save-file backward-compat regression (Pitfall 12 mitigation)
    #[test]
    fn v22_save_loads_with_defaults() {
        // A minimal v2.2-shape JSON without any v3.0 fields
        let v22_json = r#"{
            "stack": {"x": "0", "y": "0", "z": "0", "t": "0", "lastx": "0", "lift_enabled": false},
            "regs": ["0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0"],
            "alpha_reg": "",
            "alpha_mode": false,
            "angle_mode": "Deg",
            "display_mode": {"Fix": 4},
            "entry_buf": "",
            "program": [],
            "prgm_mode": false,
            "pc": 0,
            "call_stack": [],
            "is_running": false,
            "user_mode": false,
            "key_assignments": {},
            "assignments": {},
            "text_regs": {},
            "last_key_code": 0,
            "reg_m": "0",
            "reg_n": "0",
            "reg_o": "0",
            "flags": 0,
            "pending_card_op": null
        }"#;

        let state: CalcState = serde_json::from_str(v22_json).unwrap();

        // Phase 28 fields must default cleanly
        assert_eq!(state.xrom_modules, 0b0000_0001, "v2.2 save must get default xrom_modules");
        assert!(!state.complex_mode, "v2.2 save must get default complex_mode");
        assert_eq!(state.matrix_dim, None, "v2.2 save must get default matrix_dim");
        assert_eq!(state.matrix_active_reg, None, "v2.2 save must get default matrix_active_reg");
    }

    // Catches: cancel_requested not being a real Arc (copy instead of shared reference)
    #[test]
    fn cancel_field_present() {
        let state = CalcState::new();
        let cloned_arc = std::sync::Arc::clone(&state.cancel_requested);
        // Set via the clone — must be visible through the original
        cloned_arc.store(true, Ordering::Relaxed);
        assert!(
            state.cancel_requested.load(Ordering::Relaxed),
            "cancel_requested must be a real Arc<AtomicBool> (not a copy)"
        );
    }
}
