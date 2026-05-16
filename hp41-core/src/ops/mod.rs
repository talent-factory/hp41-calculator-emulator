use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::{CalcState, DisplayMode};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub mod alpha;
pub mod arithmetic;
pub mod cardreader_ops;
pub mod display_ops;
pub mod flags;
pub mod hms;
pub mod indirect;
pub mod math;
pub mod math1;
pub mod print;
pub mod program;
pub mod registers;
pub mod sound;
pub mod stack_ops;
pub mod stats;

use alpha::{op_alpha_append, op_alpha_backspace, op_alpha_clear, op_alpha_toggle};
use arithmetic::{op_add, op_div, op_mul, op_sub};
use cardreader_ops::{op_rdprgm, op_rdta, op_wdta, op_wprgm};
use math::{
    op_abs, op_acos, op_asin, op_atan, op_cos, op_exp, op_fact, op_frc, op_int, op_ln, op_log,
    op_mod, op_pct_change, op_pi, op_polar_to_rect, op_recip, op_rect_to_polar, op_rnd, op_set_deg,
    op_set_grad, op_set_rad, op_sign, op_sin, op_sq, op_sqrt, op_tan, op_tenpow, op_ypow,
};
use math1::complex::{
    op_a_pow_z, op_c_div, op_c_minus, op_c_plus, op_c_times, op_cinv, op_cos_z, op_exp_z,
    op_ln_z, op_log_z, op_magz, op_real, op_sin_z, op_tan_z, op_z_pow_1_n, op_z_pow_n,
    op_z_pow_w,
};
use math1::hyperbolics::{op_acosh, op_asinh, op_atanh, op_cosh, op_sinh, op_tanh};
use math1::matrix::{
    op_mat_det, op_mat_edit, op_mat_inv, op_mat_simeq, op_mat_size, op_mat_vcol, op_mat_vmat,
    op_matrix_workflow,
};
use math1::poly::{op_poly_workflow, op_roots};
use registers::{
    op_clreg, op_getkey, op_rcl, op_rcl_m, op_rcl_n, op_rcl_o, op_sto, op_sto_arith,
    op_sto_arith_stack, op_sto_m, op_sto_n, op_sto_o,
};
use stack_ops::{op_chs, op_clx, op_enter, op_lastx, op_r_up, op_rdn, op_xy_swap};

/// STO arithmetic operation kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StoArithKind {
    Add,
    Sub,
    Mul,
    Div,
}

/// Stack register target for STO arithmetic operations (STOA-03).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StackReg {
    Y,
    Z,
    T,
    LastX,
}

/// HP-41 flag-test kind — 4 total. Used in `Op::FlagTest { kind, flag }`.
///
/// Mirrors the `TestKind` / `StoArithKind` sub-enum precedent. The `?C` variants
/// (IsSetThenClear / IsClearThenClear) ALWAYS clear the flag as a side effect,
/// regardless of the test outcome (RESEARCH A4 — strict reading of FS?C / FC?C).
/// Phase 21 (FN-FLAG-02).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FlagTestKind {
    /// FS? — skip next step when flag is NOT set (condition: "is set"; skip-if-false).
    IsSet,
    /// FC? — skip next step when flag is NOT clear.
    IsClear,
    /// FS?C — skip if NOT set; ALWAYS clear the flag afterward (RESEARCH A4).
    IsSetThenClear,
    /// FC?C — skip if NOT clear; ALWAYS clear the flag afterward.
    IsClearThenClear,
}

/// HP-41 conditional test kind — 12 total. Used in Op::Test(TestKind).
/// D-07: single enum covers all HP-41 conditionals (symmetric with StoArithKind pattern).
/// D-08: exact variant names.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TestKind {
    XEqZero,
    XNeZero,
    XLtZero,
    XGtZero,
    XLeZero,
    XGeZero,
    XEqY,
    XNeY,
    XLtY,
    XGtY,
    XLeY,
    XGeY,
}

/// HP-41 calculator operations.
///
/// Phase 1: Add, Sub, Mul, Div, Enter, Clx, Chs, Rdn, XySwap, Lastx, PushNum
/// Phase 2: Recip, Sqrt, Sq, YPow, PctChange, Ln, Log, Exp, TenPow, Sin, Cos, Tan,
///          Asin, Acos, Atan, SetDeg, SetRad, SetGrad,
///          FmtFix, FmtSci, FmtEng, StoReg, RclReg, StoArith, Clreg,
///          AlphaToggle, AlphaAppend, AlphaClear
/// Phase 3: Lbl, Gto, Xeq, Rtn, PrgmMode, Test, Isg, Dse
/// Phase 6: SigmaPlus, SigmaMinus, Mean, Sdev, LR, Yhat, Corr, ClSigmaStat,
///          HmsToH, HToHms, HmsAdd, HmsSub
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Op {
    // ── Arithmetic (Phase 1) ──────────────────────────────────────────
    Add,
    Sub,
    Mul,
    Div,
    // ── Stack operations (Phase 1) ───────────────────────────────────
    Enter,
    Clx,
    Chs,
    Rdn,
    /// R↑ — roll stack up (mirror of Rdn): X←T, T←Z, Z←Y, Y←X.
    /// LiftEffect: Neutral. Does NOT update LASTX (Phase 20, D-19/D-20).
    Rup,
    XySwap,
    Lastx,
    /// PI — push the constant π (3.141592654, 10-digit rounded HP-41 hardware value).
    /// LiftEffect: Enable (Phase 20, D-08/D-10).
    Pi,
    /// Push a numeric literal onto the stack (e.g., from keyboard digit entry).
    PushNum(HpNum),
    // ── Unary math (Phase 2) ─────────────────────────────────────────
    /// INT — truncate X toward zero (integer part). LiftEffect: Enable.
    Int,
    /// RND — round X to the precision of the current display mode (FIX/SCI/ENG n).
    /// LiftEffect: Enable (via unary_result). (Phase 20, D-01/D-02/D-03)
    Rnd,
    /// FRC — fractional part of X (sign-preserving, complement of INT).
    /// LiftEffect: Enable. (Phase 20, D-15)
    Frc,
    /// ABS — absolute value of X. LiftEffect: Enable. (Phase 20, D-16)
    Abs,
    /// SIGN — sign of X: -1 / 0 / +1. LiftEffect: Enable.
    /// (Phase 20 always returns numeric; SIGN-on-ALPHA divergence is documented
    /// in Phase 25 docs per D-18.)
    Sign,
    /// FACT — factorial of integer X. Domain error for non-integer or negative X.
    /// OutOfRange for X > 69 (hardware spec, D-06).
    /// Overflow for X ≥ 28 (Decimal range cap, D-05). LiftEffect: Enable.
    Fact,
    /// 1/x — reciprocal. LiftEffect: Enable.
    Recip,
    /// √x — square root. LiftEffect: Enable.
    Sqrt,
    /// x² — square. LiftEffect: Enable.
    Sq,
    /// Y^X — Y raised to power X (binary). LiftEffect: Enable.
    YPow,
    /// MOD — Y mod X with HP-41 trunc-toward-zero convention (D-14):
    /// result = Y − X · trunc(Y/X). Sign follows Y (matches HP-41C Owner's
    /// Manual + Free42 source). Examples: `7 MOD -3 = 1`, `-7 MOD 3 = -1`.
    /// Domain error if X = 0. LiftEffect: Enable (via binary_result).
    /// (Phase 20, D-14, FN-MATH-06)
    Mod,
    /// %CH — percent change ((X−Y)/Y)×100. Stack effect: unary (Y preserved, LASTX←X). LiftEffect: Enable.
    PctChange,
    /// LN — natural logarithm. LiftEffect: Enable.
    Ln,
    /// LOG — log base 10. LiftEffect: Enable.
    Log,
    /// e^x — natural exponential. LiftEffect: Enable.
    Exp,
    /// 10^x — base-10 exponential. LiftEffect: Enable.
    TenPow,
    // ── Trig (Phase 2) ───────────────────────────────────────────────
    /// SIN — sine in current angle_mode. LiftEffect: Enable.
    Sin,
    /// COS — cosine in current angle_mode. LiftEffect: Enable.
    Cos,
    /// TAN — tangent in current angle_mode. LiftEffect: Enable.
    Tan,
    /// ASIN — arcsine, result in current angle_mode. LiftEffect: Enable.
    Asin,
    /// ACOS — arccosine, result in current angle_mode. LiftEffect: Enable.
    Acos,
    /// ATAN — arctangent, result in current angle_mode. LiftEffect: Enable.
    Atan,
    /// P→R — convert polar (Y = magnitude, X = angle in current angle_mode)
    /// to rectangular (Y = x-coord, X = y-coord). LiftEffect: Enable.
    /// Direct stack assignment; LASTX ← consumed X. (Phase 20, D-11/D-12/D-13)
    PolarToRect,
    /// R→P — convert rectangular (Y = x-coord, X = y-coord) to polar
    /// (Y = magnitude, X = angle in current angle_mode). LiftEffect: Enable.
    /// Direct stack assignment; LASTX ← consumed X. (Phase 20, D-11/D-12/D-13)
    RectToPolar,
    // ── Angle mode (Phase 2) ─────────────────────────────────────────
    /// Set angle mode to DEG. LiftEffect: Neutral.
    SetDeg,
    /// Set angle mode to RAD. LiftEffect: Neutral.
    SetRad,
    /// Set angle mode to GRAD. LiftEffect: Neutral.
    SetGrad,
    // ── Display mode (Phase 2) ───────────────────────────────────────
    /// FIX n — fixed decimal display (n = 0–9). LiftEffect: Neutral.
    FmtFix(u8),
    /// SCI n — scientific notation display (n = 0–9). LiftEffect: Neutral.
    FmtSci(u8),
    /// ENG n — engineering notation display (n = 0–9). LiftEffect: Neutral.
    FmtEng(u8),
    // ── Storage registers (Phase 2) ──────────────────────────────────
    /// STO n — store X into register n (0–99). LiftEffect: Neutral.
    StoReg(u8),
    /// RCL n — recall register n into X (0–99). LiftEffect: Enable.
    RclReg(u8),
    /// STO+/−/×/÷ n — arithmetic on register n using X. LiftEffect: Neutral.
    StoArith {
        reg: u8,
        kind: StoArithKind,
    },
    /// STO+/−/×/÷ stack-reg — arithmetic on a stack register using X. LiftEffect: Neutral.
    StoArithStack {
        kind: StoArithKind,
        stack_reg: StackReg,
    },
    /// CLREG — clear all storage registers to zero. LiftEffect: Neutral.
    Clreg,
    // ── ALPHA mode (Phase 2) ─────────────────────────────────────────
    /// ALPHA — toggle alpha_mode flag. LiftEffect: Neutral.
    AlphaToggle,
    /// Append char to alpha_reg (max 24). LiftEffect: Neutral.
    AlphaAppend(char),
    /// CLRALPHA — clear alpha_reg. LiftEffect: Neutral.
    AlphaClear,
    // ── Programming (Phase 3) ────────────────────────────────────────────
    /// LBL "name" — program label marker. No-op during execution. LiftEffect: Neutral.
    /// D-01: labels are Op::Lbl markers stored in the flat program Vec.
    Lbl(String),
    /// GTO "name" — unconditional branch to label. LiftEffect: Neutral.
    Gto(String),
    /// XEQ "name" — subroutine call (max 4 deep, D-14). LiftEffect: Neutral.
    Xeq(String),
    /// RTN — return from subroutine; terminates run if call_stack is empty. LiftEffect: Neutral.
    Rtn,
    /// PRGM — toggle prgm_mode recording flag (D-03). LiftEffect: Neutral.
    PrgmMode,
    /// Conditional test — skip next step if condition is false (D-09). LiftEffect: Neutral.
    Test(TestKind),
    /// ISG n — increment register n by step, skip next if new_current > final (D-11). LiftEffect: Neutral.
    Isg(u8),
    /// DSE n — decrement register n by step, skip next if new_current <= final (D-11). LiftEffect: Neutral.
    Dse(u8),
    // ── USER mode (Phase 5) ──────────────────────────────────────────────
    /// USER mode toggle: flip state.user_mode. LiftEffect: Neutral.
    UserMode,
    // ── ALPHA backspace (Phase 5) ────────────────────────────────────────
    /// ALPHA backspace: remove last char from alpha_reg (HP-41 ← key). LiftEffect: Neutral.
    AlphaBackspace,
    // ── Science & Engineering (Phase 6) ─────────────────────────────────
    /// Σ+ — accumulate X and Y into Σ registers R01–R06; push count n into X. LiftEffect: Enable.
    SigmaPlus,
    /// Σ− — remove X and Y from Σ registers R01–R06; push count n into X. LiftEffect: Enable.
    SigmaMinus,
    /// MEAN — push x̄ to X and ȳ to Y from Σ registers. LiftEffect: Enable.
    Mean,
    /// SDEV — push sample σx to X and σy to Y (n-1 denominator). LiftEffect: Enable.
    Sdev,
    /// L.R. — linear regression: push slope m to Y and intercept b to X. LiftEffect: Enable.
    LR,
    /// YHAT — ŷ prediction: read x from X, push ŷ into X. LiftEffect: Enable.
    Yhat,
    /// CORR — correlation coefficient r in X. LiftEffect: Enable.
    Corr,
    /// CLΣSTAT — zero Σ registers R01–R06. LiftEffect: Neutral.
    ClSigmaStat,
    /// HMS→ — convert H.MMSS to decimal hours in X. LiftEffect: Enable.
    HmsToH,
    /// →HMS — convert decimal hours in X to H.MMSS. LiftEffect: Enable.
    HToHms,
    /// HMS+ — add two H.MMSS values (Y + X), result in X with stack drop. LiftEffect: Enable.
    HmsAdd,
    /// HMS− — subtract H.MMSS values (Y − X), result in X with stack drop. LiftEffect: Enable.
    HmsSub,
    // ── Print operations (Phase 11) ─────────────────────────────────────────────
    /// PRX — print X register in current display format, right-aligned to 24 chars. LiftEffect: Neutral.
    PRX,
    /// PRA — print ALPHA register, left-aligned to 24 chars. LiftEffect: Neutral.
    PRA,
    /// PRSTK — print full stack T/Z/Y/X/LASTX/ALPHA, 6 lines, 24 chars each. LiftEffect: Neutral.
    PRSTK,
    // ── Synthetic Programming (Phase 12) ────────────────────────────────────
    /// GETKEY — push last key code (HP-41 row×10+col) to X. LiftEffect: Enable.
    GetKey,
    /// NULL — true no-op; does not modify any state. LiftEffect: Neutral.
    Null,
    /// STO M — store X into hidden register M. LiftEffect: Neutral.
    StoM,
    /// STO N — store X into hidden register N. LiftEffect: Neutral.
    StoN,
    /// STO O — store X into hidden register O. LiftEffect: Neutral.
    StoO,
    /// RCL M — recall hidden register M into X. LiftEffect: Enable.
    RclM,
    /// RCL N — recall hidden register N into X. LiftEffect: Enable.
    RclN,
    /// RCL O — recall hidden register O into X. LiftEffect: Enable.
    RclO,
    /// SyntheticByte(u8) — synthetic op inserted via hex modal. At execution time,
    /// dispatches to the corresponding Op via `synthetic_byte_to_op()` lookup.
    /// LiftEffect: varies (delegates to the mapped op).
    SyntheticByte(u8),
    // ── Card Reader ─────────────────────────────────────────────────────────
    /// WDTA — write data registers R00..R(SIZE-1) to the card named in the
    /// ALPHA register. Stages a `CardOpRequest::WriteData` for the frontend
    /// to drain. Empty ALPHA → `HpError::AlphaData`. LiftEffect: Neutral.
    Wdta,
    /// RDTA — read a data card named in ALPHA and replace data registers.
    /// Stages a `CardOpRequest::ReadData`. LiftEffect: Neutral.
    Rdta,
    /// WPRGM — write current program to the `.raw` card named in ALPHA.
    /// Stages a `CardOpRequest::WriteProgram`. LiftEffect: Neutral.
    Wprgm,
    /// RDPRGM — read a `.raw` card named in ALPHA and insert its ops
    /// (replace if program empty, else insert after PC). LiftEffect: Neutral.
    Rdprgm,
    // ── Phase 21: Flags ──────────────────────────────────────────────────────
    /// SF n — set flag n (0..=55). LiftEffect: Neutral.
    SfFlag(u8),
    /// CF n — clear flag n (0..=55). LiftEffect: Neutral.
    CfFlag(u8),
    /// FS? / FC? / FS?C / FC?C n — conditional flag test (run_loop skips next
    /// step on false; `?C` variants also always-clear). LiftEffect: Neutral.
    /// Interactive dispatch is a no-op (mirrors `Op::Test` precedent — no next
    /// program step at the keyboard). Phase 21 (FN-FLAG-02).
    FlagTest {
        kind: FlagTestKind,
        flag: u8,
    },
    // ── Phase 21: Display Control ────────────────────────────────────────────
    /// VIEW n — show formatted value of register n (0..=99). LiftEffect: Neutral.
    /// Phase 21 (FN-DISP-01).
    View(u8),
    /// AVIEW — show ALPHA register on display (24-char truncate). LiftEffect: Neutral.
    /// Phase 21 (FN-DISP-02).
    AView,
    /// PROMPT — show ALPHA on display; inside `run_loop`, also `break` execution.
    /// LiftEffect: Neutral. Phase 21 (FN-DISP-03). Full STOP/resume deferred to Phase 22.
    Prompt,
    /// AON — enable ALPHA auto-display (set system flag 48, HP-42S compat).
    /// LiftEffect: Neutral. Phase 21 (FN-DISP-04).
    Aon,
    /// AOFF — disable ALPHA auto-display (clear system flag 48).
    /// LiftEffect: Neutral. Phase 21 (FN-DISP-04).
    Aoff,
    /// CLD — explicit clear of `display_override`. LiftEffect: Neutral.
    /// Phase 21 (FN-DISP-05).
    Cld,
    // ── Phase 21: Sound ──────────────────────────────────────────────────────
    /// BEEP — push `BEEP` event to event_buffer. LiftEffect: Neutral.
    /// Phase 21 (FN-SOUND-01).
    Beep,
    /// TONE n — push `TONE n` event to event_buffer (n=0..=9).
    /// LiftEffect: Neutral. Phase 21 (FN-SOUND-02).
    Tone(u8),
    // ── Phase 22: Program control (D-22.1, D-22.4, D-22.15, D-22.22) ────────
    /// STOP — halt program execution. Inside `run_loop`, breaks the loop without
    /// writing to `display_override` (unlike `Op::Prompt`). Interactive dispatch
    /// is a Neutral no-op (D-22.5). LiftEffect: Neutral. Phase 22 (FN-PROG-01).
    Stop,
    /// PSE — pause: write formatted X to `display_override` AND push "PAUSE 1000"
    /// into `state.event_buffer`. Does NOT break run_loop — execution continues.
    /// Frontend reads the event marker and inserts a ~1s delay before refresh.
    /// LiftEffect: Neutral. Phase 22 (FN-PROG-02, D-22.4).
    Pse,
    /// GTO IND nn — indirect branch through register nn. The pointer is the
    /// truncated integer part of `state.regs[nn]`; non-integer values reject
    /// with `HpError::InvalidOp`. Phase 22 ships an inline resolver — Phase 24
    /// will extract the shared `resolve_indirect()` helper. Programming-only:
    /// interactive dispatch returns `InvalidOp`. LiftEffect: Neutral.
    /// Phase 22 (FN-PROG-06, D-22.15).
    GtoInd(u8),
    /// XEQ IND nn — indirect subroutine call through register nn. Pre-mutation
    /// 4-deep call_stack guard (returns `HpError::CallDepth` before any state
    /// change). Pointer extracted by the same integer-truncate-then-equality-
    /// check pattern as `Op::GtoInd`. No card-reader builtin fallback (the
    /// label is a numeric string only). Programming-only: interactive dispatch
    /// returns `InvalidOp`. LiftEffect: Neutral. Phase 22 (FN-PROG-07, D-22.15).
    XeqInd(u8),
    // ── Phase 22: Program editing (D-22.7, D-22.8, D-22.9, D-22.10) ─────────
    /// CLP "name" — clear program from `Op::Lbl(name)` to the next `Op::Lbl(_)`
    /// (or end-of-Vec if no further LBL). After deletion, `state.pc` repositions
    /// to the start of the deleted block (clamped to `state.program.len()` —
    /// Pitfall 6). Missing label → `HpError::InvalidOp`. Documented divergence
    /// from HP-41 hardware: real device uses END/.END. markers; we use next-LBL
    /// because the flat-Vec model has no explicit END marker. Gated on
    /// `state.prgm_mode == true` (D-22.10). LiftEffect: Neutral.
    /// Phase 22 (FN-PROG-03, D-22.7).
    Clp(String),
    /// DEL nnn — delete `nnn` program steps starting at `state.pc`. `nnn` is
    /// silently clamped to `min(nnn, program.len() - state.pc)`; `nnn == 0` OR
    /// `state.pc == program.len()` → no-op. `state.pc` is UNCHANGED (the drain
    /// shifts the tail down so pc naturally points at the next step). Gated
    /// on `state.prgm_mode == true` (D-22.10). LiftEffect: Neutral.
    /// Phase 22 (FN-PROG-04, D-22.9).
    Del(u8),
    /// INS — insert `Op::Null` (no-op placeholder, Phase 12) at `state.pc`.
    /// `state.pc` is UNCHANGED — the cursor still points at the freshly
    /// inserted Null. Gated on `state.prgm_mode == true` (D-22.10).
    /// LiftEffect: Neutral. Phase 22 (FN-PROG-05, D-22.8).
    Ins,
    // ── Phase 22: Memory & stack management (D-22.11..13) ────────────────────
    /// SIZE nnn — resize `state.regs` to nnn ∈ [1, 319]. AMENDED D-22.11 /
    /// OQ-2: `nnn == 0` silently clamps to 1 (documented divergence from real
    /// HP-41 which accepts `SIZE 000`); `nnn > 319` returns `HpError::InvalidOp`.
    /// Shrinking truncates the tail (hardware-faithful "MEM LOST"); growing
    /// zero-fills new slots; overlapping range preserves values.
    /// `u16` because 319 > u8::MAX. LiftEffect: Neutral. Phase 22 (FN-MEM-01).
    Size(u16),
    /// CLA — clear ALPHA register. Hardware-faithful HP-41 display name
    /// (program listings show "CLA", not "CLRALPHA"). Delegates to
    /// `op_alpha_clear` — same body as legacy `Op::AlphaClear`. The two
    /// variants COEXIST: `Op::Cla` is the hardware-faithful display alias
    /// and `Op::AlphaClear` (display "CLRALPHA") stays in the enum for
    /// v1.0 save-file backward compat (Pitfall 8 — do NOT consolidate).
    /// LiftEffect: Neutral. Phase 22 (FN-MEM-02, D-22.13).
    Cla,
    /// CLST — clear stack: zero X, Y, Z, T. PRESERVES `state.stack.lastx`
    /// AND `state.stack.lift_enabled` (D-22.14 invariant). The preservation
    /// of LASTX is the critical divergence from `Op::Clreg` (which only
    /// clears regs, not the stack). The preservation of `lift_enabled`
    /// follows from `LiftEffect::Neutral` — `apply_lift_effect(Neutral)`
    /// is a no-op for `lift_enabled`. Verified by sentinel test
    /// `test_clst_preserves_lastx_and_lift_enabled` in
    /// `hp41-core/tests/phase22_memory_ops.rs`.
    /// LiftEffect: Neutral. Phase 22 (FN-MEM-03, D-22.14).
    Clst,
    /// PACK — documented no-op + Neutral lift. Real HP-41 PACK compacts
    /// program memory by removing gaps from in-place edits; our flat-Vec
    /// program model has no gaps to compact, so PACK is a no-op. This is
    /// a deliberate divergence flagged in D-22.12 — implementing it
    /// meaningfully would require introducing gaps into `state.program`
    /// (deferred backlog candidate).
    /// LiftEffect: Neutral. Phase 22 (FN-MEM-04, D-22.12).
    Pack,
    /// CATALOG n — hardware-faithful HP-41 CATALOG (AMENDED D-22.16 / OQ-1
    /// Option B). `n == 0` OR `n >= 5` → `HpError::InvalidOp`. Output is
    /// written to `state.print_buffer` (Phase 11 drain channel) with 24-char
    /// width: header `-- CATALOG n --`, payload, footer `-- END --`.
    ///
    /// CAT 1: programs — iterate `state.program`, emit `LBL <name>  <steps>`
    ///   line per `Op::Lbl`. Step count = distance to next LBL or
    ///   `program.len()` for the last labelled block.
    /// CAT 2 (XROM modules), CAT 3 (HP-IL), CAT 4 (peripherals) — none
    ///   in this emulator → single "NOT AVAILABLE" payload line.
    ///
    /// LiftEffect: Neutral. Phase 22 (FN-MEM-05, D-22.16 AMENDED).
    Catalog(u8),
    /// ASN "name" key_code — record a key assignment (AMENDED D-22.18 /
    /// OQ-3 Option A, FN-KEY-01).
    ///
    /// If `name` is empty: removes the assignment for `key_code`
    /// (`state.assignments.remove(&key_code)` — silent no-op if absent).
    /// Otherwise: inserts/overwrites (`state.assignments.insert(key_code, name)`).
    ///
    /// `key_code` uses HP-41 row×10+col encoding (1-indexed; same as
    /// `last_key_code` and `keycode_to_hp41_code`). Any u8 is accepted —
    /// frontend (CLI / GUI keyboard layer in Phase 25/26) restricts to the
    /// hardware key-code domain.
    ///
    /// Hardware-faithful semantics: `ASN "" 11` undoes `ASN "SIN" 11`.
    ///
    /// Late-binding (D-22.19): hp41-core stores the assignment as a String;
    /// resolution (parse-as-Op vs LBL search) happens at USER-mode dispatch
    /// in Phase 25/26.
    ///
    /// LiftEffect: Neutral. Phase 22 (FN-KEY-01, D-22.18 AMENDED).
    Asn {
        name: String,
        key_code: u8,
    },
    // ── Phase 23: ALPHA-register operations (D-23.12) ────────────────────────
    /// ARCL nn — append register-N's formatted value to the ALPHA register.
    /// Reads from `state.text_regs[reg]` if present (packed-text shadow from
    /// a prior ASTO), else formats `state.regs[reg]` via `format_hpnum`
    /// (respects the current FIX/SCI/ENG display mode — SC#1). 24-char
    /// silent-discard cap. Out-of-range `reg` → `HpError::InvalidOp` BEFORE
    /// any text_regs lookup (W-2 strengthening of D-23.3; T-23-01).
    /// LiftEffect: Neutral. Phase 23 (FN-ALPHA-01, D-23.3).
    Arcl(u8),
    /// ASTO nn — pack the first 6 chars of the ALPHA register into
    /// `state.text_regs[reg]` and zero the numeric slot `regs[reg]` (no-drift
    /// invariant, paired with D-23.4 sidecar-clearing in op_sto / op_sto_arith
    /// / op_clreg). The ALPHA register itself is NOT modified. Out-of-range
    /// `reg` → `HpError::InvalidOp` BEFORE the sidecar write (atomicity).
    /// LiftEffect: Neutral. Phase 23 (FN-ALPHA-02, D-23.2).
    Asto(u8),
    /// ATOX — pop the first ALPHA char and push its Unicode codepoint into X
    /// (capped at 255 via `.min(255)` — D-23.10 8-bit cap). Empty ALPHA pushes
    /// 0. The lift is Enable (mirrors `op_pi`'s lift-then-push idiom): X→Y,
    /// Y→Z, Z→T BEFORE the new code is written into X. ALPHA mutation uses
    /// `chars()` (multibyte-safe per Phase 2 invariant). HP-41 hardware glyphs
    /// 128..=255 are NOT preserved by the round-trip — documented divergence.
    /// LiftEffect: Enable. Phase 23 (FN-ALPHA-03, D-23.10).
    Atox,
    /// XTOA — convert X mod 256 to a character and append it to ALPHA. Codes
    /// 0..=127 append as ASCII; 128..=255 append as `'?'` (HP-41 upper-ASCII
    /// glyphs are not in our String/UTF-8 model — D-23.11 documented divergence).
    /// 24-char ALPHA cap silently discards the append on overflow (Phase 2
    /// invariant, mirrors `op_alpha_append`). X is NOT consumed.
    /// LiftEffect: Neutral. Phase 23 (FN-ALPHA-04, D-23.11).
    Xtoa,
    /// AROT — rotate ALPHA by X chars (positive = left rotation, negative =
    /// right rotation via `rem_euclid` — D-23.8). Empty ALPHA is a no-op
    /// preserving X. |N| > len is normalised by `rem_euclid(len)`. Non-integer
    /// X is silently truncated toward zero (faithful HP-41CV per D-23.9 —
    /// stricter than POSA which rejects). X is NOT consumed.
    /// LiftEffect: Neutral. Phase 23 (FN-ALPHA-05, D-23.8 / D-23.9).
    Arot,
    /// POSA — single-char POSA (D-23.7). X must be an integer ASCII codepoint
    /// in 0..=127 — non-integer X or out-of-range X returns `HpError::InvalidOp`
    /// (stricter than AROT's silent-truncate per D-23.7 vs D-23.9). The result
    /// REPLACES X: position of the first matching char in ALPHA (0-indexed),
    /// or `-1` if not found (SC#5 explicit wording — other HP-41 sources
    /// return haystack length; we pick -1). Multi-char POSA is deferred to
    /// v3.x per D-23.6 (requires typed-stack `x_text` shadow channel).
    /// LiftEffect: Disable. Phase 23 (FN-ALPHA-06, D-23.7).
    Posa,
    // -- Phase 24: Indirect Addressing (FN-IND-01, FN-IND-02) -------------
    // Every variant delegates to its direct-form counterpart via
    // `crate::ops::indirect::resolve_indirect`. Family naming pattern: `<Name>Ind(u8)`.
    // Inherits sidecar (D-23.4), atomicity (D-22.x), and lift-effect from
    // the delegated direct op (D-24.4 -- no replication).
    /// STO IND nn -- store X into the register pointed to by `state.regs[nn]`'s
    /// integer part. LiftEffect: Neutral (inherited from `op_sto`). Inherits
    /// the D-23.4 text_regs sidecar clear and D-22.11.1 bounds via delegation.
    /// Phase 24 (FN-IND-01).
    StoInd(u8),
    /// RCL IND nn -- recall `regs[regs[nn].int_part]` into X.
    /// LiftEffect: Enable (inherited from `op_rcl`). Phase 24 (FN-IND-01).
    RclInd(u8),
    /// STO+/-/x// IND nn -- arithmetic via the indirect register address.
    /// LiftEffect: Neutral (inherited from `op_sto_arith`). Kind reuse mirrors
    /// the `Op::StoArith` shape exactly (tuple-variant family pattern, D-24.7).
    /// Phase 24 (FN-IND-01).
    StoArithInd(u8, StoArithKind),
    /// ISG IND nn -- increment register at the resolved indirect address and
    /// skip next step on counter exit. LiftEffect: Neutral (inherited from
    /// `op_isg`). Skip semantics live in `run_loop`, not `dispatch`. Phase 24
    /// (FN-IND-01).
    IsgInd(u8),
    /// DSE IND nn -- decrement register at the resolved indirect address and
    /// skip next step on counter exit. LiftEffect: Neutral (inherited from
    /// `op_dse`). Skip semantics live in `run_loop`. Phase 24 (FN-IND-01).
    DseInd(u8),
    /// SF IND nn -- set the flag whose number is `regs[nn]`'s integer part.
    /// LiftEffect: Neutral (inherited from `op_sf`). Inherits the `< 56`
    /// flag-bounds check via delegation. Phase 24 (FN-IND-01).
    SfFlagInd(u8),
    /// CF IND nn -- clear the flag whose number is `regs[nn]`'s integer part.
    /// LiftEffect: Neutral (inherited from `op_cf`). Phase 24 (FN-IND-01).
    CfFlagInd(u8),
    /// FS? / FC? / FS?C / FC?C IND nn -- conditional flag test on the flag
    /// whose number is `regs[ind_reg]`'s integer part. Interactive dispatch
    /// is a Neutral no-op (mirrors `Op::FlagTest` precedent -- no next program
    /// step at the keyboard). Skip / always-clear semantics live in `run_loop`.
    /// STRUCT variant per D-24.6, mirroring `Op::FlagTest { kind, flag }`.
    /// LiftEffect: Neutral. Phase 24 (FN-IND-01).
    FlagTestInd {
        kind: FlagTestKind,
        ind_reg: u8,
    },
    /// ARCL IND nn -- append the formatted value of `regs[regs[nn]]` to the
    /// ALPHA register. LiftEffect: Neutral (inherited from `op_arcl`).
    /// Inherits the text_regs sidecar read path. Phase 24 (FN-IND-01).
    ArclInd(u8),
    /// ASTO IND nn -- pack the first 6 ALPHA chars into `regs[regs[nn]]`'s
    /// packed-text shadow and zero the numeric slot. LiftEffect: Neutral
    /// (inherited from `op_asto`). Phase 24 (FN-IND-01).
    AstoInd(u8),
    /// VIEW IND nn -- display the VALUE of `regs[regs[nn]]` (NOT the pointer
    /// register). LiftEffect: Neutral (inherited from `op_view`). R9
    /// mitigation: `op_view_ind` delegates to `op_view(state, resolved_addr)`,
    /// so the display shows the resolved register's contents. Phase 24
    /// (FN-IND-01).
    ViewInd(u8),
    // ── Phase 28: Hyperbolics (Plan 28-02) ────────────────────────────────────
    /// SINH — hyperbolic sine. Angle-mode-independent. LiftEffect: Enable.
    /// XROM Math Pac I (HP 00041-90034). No domain restriction; Overflow for extreme magnitudes.
    Sinh,
    /// COSH — hyperbolic cosine. Angle-mode-independent. LiftEffect: Enable.
    /// XROM Math Pac I (HP 00041-90034). No domain restriction; Overflow for extreme magnitudes.
    Cosh,
    /// TANH — hyperbolic tangent. Angle-mode-independent. LiftEffect: Enable.
    /// XROM Math Pac I (HP 00041-90034). Saturates to ±1 for large |X| (not an error).
    Tanh,
    /// ASINH — inverse hyperbolic sine. Angle-mode-independent. LiftEffect: Enable.
    /// XROM Math Pac I (HP 00041-90034). No domain restriction.
    Asinh,
    /// ACOSH — inverse hyperbolic cosine. Angle-mode-independent. LiftEffect: Enable.
    /// XROM Math Pac I (HP 00041-90034). Domain: X >= 1.0; X < 1.0 → HpError::Domain.
    Acosh,
    /// ATANH — inverse hyperbolic tangent. Angle-mode-independent. LiftEffect: Enable.
    /// XROM Math Pac I (HP 00041-90034). Domain: |X| < 1.0; |X| >= 1.0 → HpError::Domain.
    Atanh,
    // ── Phase 28: Complex Stack Arithmetic (Plan 28-03) ────────────────────
    /// C+ — complex addition: ζ' = ζ + τ where ζ=X+iY, τ=Z+iT.
    /// T-replicate: new Z and T get old T. LiftEffect: Enable.
    /// Sets complex_mode = true (D-28.2 auto-on). CMPLX-02 / HP 00041-90034.
    CPlus,
    /// C- — complex subtraction: ζ' = ζ - τ.
    /// T-replicate: new Z and T get old T. LiftEffect: Enable.
    /// Sets complex_mode = true (D-28.2 auto-on). CMPLX-03 / HP 00041-90034.
    CMinus,
    /// C× — complex multiplication: ζ' = ζ · τ = (XZ-YT) + i(XT+YZ).
    /// T-replicate: new Z and T get old T. LiftEffect: Enable.
    /// Sets complex_mode = true (D-28.2 auto-on). CMPLX-04 / HP 00041-90034.
    CTimes,
    /// C÷ — complex division: ζ' = ζ / τ = ((XZ+YT) + i(YZ-XT)) / (Z²+T²).
    /// Zero-divisor guard BEFORE any mutation: Z=0 AND T=0 → HpError::DivideByZero.
    /// T-replicate: new Z and T get old T. LiftEffect: Enable.
    /// Sets complex_mode = true (D-28.2 auto-on). CMPLX-05 / HP 00041-90034.
    CDiv,
    /// REAL — deactivate complex mode (CMPLX-18 / D-28.3).
    /// Sets complex_mode = false. Stack untouched. LiftEffect: Neutral.
    /// UX extension — NOT in Math Pac I OM 1979; documented divergence per D-28.3.
    Real,
    // ── Phase 28: Complex Functions (Plan 28-04) ────────────────────────────
    /// MAGZ — complex magnitude |ζ| = sqrt(X²+Y²). Writes to X; Y unchanged.
    /// LiftEffect: Disable. Sets complex_mode = true. CMPLX-06 / HP 00041-90034 ~p.25.
    Magz,
    /// CINV — complex inverse 1/(X+iY). DivideByZero on (0,0) (pre-mutation guard).
    /// LiftEffect: Disable. Sets complex_mode = true. CMPLX-07 / HP 00041-90034 ~p.25.
    Cinv,
    /// Z↑N — complex integer-exponent power: ζ^N via repeated multiply. N=X, base=Y+iZ.
    /// LiftEffect: Disable. Sets complex_mode = true. CMPLX-14 / HP 00041-90034 ~p.26.
    ZpowN,
    /// Z↑1/N — complex N-th root: r^(1/N)·cis(θ/N). N=X, base=Y+iZ. (0,0)→(0,0).
    /// LiftEffect: Disable. Sets complex_mode = true. CMPLX-15 / HP 00041-90034 ~p.26.
    Zpow1N,
    /// E↑Z — complex exponential: e^X·(cos(Y)+i·sin(Y)).
    /// LiftEffect: Disable. Sets complex_mode = true. CMPLX-10 / HP 00041-90034 ~p.25.
    ExpZ,
    /// LNZ — complex natural log: ln|ζ| + i·arg(ζ). Domain on (0,0) (CMPLX-11).
    /// LiftEffect: Disable. Sets complex_mode = true. CMPLX-11 / HP 00041-90034 ~p.26.
    LnZ,
    /// SINZ — complex sine: sin(X)·cosh(Y) + i·cos(X)·sinh(Y).
    /// LiftEffect: Disable. Sets complex_mode = true. CMPLX-08 / HP 00041-90034 ~p.26.
    SinZ,
    /// COSZ — complex cosine: cos(X)·cosh(Y) - i·sin(X)·sinh(Y).
    /// LiftEffect: Disable. Sets complex_mode = true. CMPLX-09 / HP 00041-90034 ~p.26.
    CosZ,
    /// TANZ — complex tangent: sin(z)/cos(z). Domain at cos(z)=0 singularity (CMPLX-13).
    /// LiftEffect: Disable. Sets complex_mode = true. CMPLX-13 / HP 00041-90034 ~p.26.
    TanZ,
    /// A↑Z — complex power a^z = exp(z·ln(a)). a=τ, z=ζ. Domain on a=(0,0) (CMPLX-16).
    /// Binary: T-replicate. LiftEffect: Enable. CMPLX-16 / HP 00041-90034 ~p.26.
    ApowZ,
    /// LOGZ — complex log base 10: LNZ/ln(10). Domain on (0,0). CMPLX-12.
    /// LiftEffect: Disable. Sets complex_mode = true. CMPLX-12 / HP 00041-90034 ~p.26.
    LogZ,
    /// Z↑W — complex power z^w = exp(w·LnZ). z=ζ, w=τ. Domain on (0,0)^w with Re(w)≤0 (CMPLX-17).
    /// Binary: T-replicate. LiftEffect: Enable. CMPLX-17 / HP 00041-90034 ~p.26.
    ZpowW,
    // ── Phase 28: POLY / ROOTS (Plan 28-05) ────────────────────────────────────
    /// POLY — polynomial root-finder master entry. Opens modal workflow (DegreePrompt →
    /// CoefficientPrompt → Ready). Sets state.modal_program + state.modal_prompt.
    /// LiftEffect: Neutral. POLY-01 / HP 00041-90034 Chapter 7.
    PolyWorkflow,
    /// ROOTS — polynomial root executor. Reads coefficients from R00..R(degree).
    /// Outputs roots to state.print_buffer in U=u/V=v/U=u/-V=-v format (POLY-04).
    /// LiftEffect: Neutral. POLY-02 / HP 00041-90034 Chapter 7.
    Roots,
    // ── Phase 28: MATRIX (Plan 28-06) ────────────────────────────────────────
    /// MATRIX — master matrix workflow entry: opens ORDER=? modal (MatrixWorkflow).
    /// Sets modal_program = Matrix(OrderPrompt); matrix_active_reg = 15.
    /// LiftEffect: Neutral. MAT-01 / HP 00041-90034 Chapter 3.
    MatrixWorkflow,
    /// SIZE — returns matrix order N from R14 to X.
    /// LiftEffect: Enable. MAT-02 / HP 00041-90034 Chapter 3.
    MatSize,
    /// VMAT — displays all matrix elements in column-major order via print_buffer.
    /// Format: "A{r},{c}={val}" per element. LiftEffect: Neutral. MAT-03.
    MatVmat,
    /// EDIT — opens matrix edit mode (ROW↑COL=? prompt).
    /// LiftEffect: Neutral. MAT-04 / HP 00041-90034 Chapter 3.
    MatEdit,
    /// DET — LU determinant with partial pivoting; result in X.
    /// LiftEffect: Enable. MAT-05 / HP 00041-90034 Chapter 3, p. 14.
    MatDet,
    /// INV — Gauss-Jordan inversion in place; singular → "NO SOLUTION" modal_prompt.
    /// Singularity threshold: INV_EPSILON = 1e-10 (ADR-003, Plan 28-01).
    /// LiftEffect: Neutral. MAT-06/MAT-07 / HP 00041-90034 Chapter 3, p. 23.
    MatInv,
    /// SIMEQ — solves [A|b]; solution at R(N+1)..R(2N); sets flag 5 on success.
    /// Singular → "NO SOLUTION" modal_prompt. LiftEffect: Neutral.
    /// MAT-08/MAT-10/MAT-11 / HP 00041-90034 Chapter 3, p. 28.
    MatSimeq,
    /// VCOL — displays B-vector elements R(N+1)..R(2N) via print_buffer.
    /// Format: "B{n}={val}" per element. LiftEffect: Neutral. MAT-09.
    MatVcol,
}

/// Flush the number entry buffer to the stack.
///
/// If entry_buf is non-empty, parse it as a Decimal and push onto the stack
/// via enter_number (respecting lift_enabled). Then set lift_enabled = true.
///
/// This MUST be called at the start of every dispatch() invocation so that
/// pending digit entry is committed before any operation consumes the stack.
///
/// Returns Err(HpError::InvalidOp) only if entry_buf contains unparseable content
/// (defensive guard; well-formed CLI input produces valid Decimal strings).
pub fn flush_entry_buf(state: &mut CalcState) -> Result<(), HpError> {
    if state.entry_buf.is_empty() {
        return Ok(());
    }
    // Clone entry_buf for parsing — do NOT clear yet. Clearing happens only on success
    // so that a parse error preserves the user's in-progress input (WR-02).
    let mut s = state.entry_buf.clone();
    // D-09 (Phase 9): trailing 'e' with no exponent digits is HP-41 hardware-faithful
    // shorthand for "exponent 00". Normalize by appending "00" so from_scientific accepts it.
    // Also handles trailing "e-" (CHS pressed with no exponent digits yet) → "e-00" = exponent 0.
    // We check both 'e'/'E' variants for safety even though entry_buf is always lowercase.
    if s.ends_with("e-") || s.ends_with("E-") || s.ends_with('e') || s.ends_with('E') {
        s.push_str("00");
    }
    let d = Decimal::from_str(&s)
        .or_else(|_| Decimal::from_scientific(&s))
        .map_err(|_| HpError::InvalidOp)?;
    // Parse succeeded — now clear the entry buffer.
    state.entry_buf.clear();
    let n = HpNum::rounded(d);
    if state.prgm_mode {
        // Recording mode: PushNum goes to program Vec, not stack (D-03/D-04).
        // lift_enabled is NOT changed — recording does not affect execution state.
        state.program.push(Op::PushNum(n));
    } else {
        // Execute mode: existing behaviour unchanged.
        crate::stack::enter_number(state, n);
        crate::stack::apply_lift_effect(state, LiftEffect::Enable);
    }
    Ok(())
}

/// Dispatch an operation to its implementation function.
///
/// This is the single entry point for all calculator operations in hp41-core.
/// Callers (hp41-cli, tests) call dispatch(state, op) and handle the Result.
pub fn dispatch(state: &mut CalcState, op: Op) -> Result<(), HpError> {
    flush_entry_buf(state)?; // commit any pending digit entry before executing op
                             // ── Phase 21 Pitfall-5: clear stale display override before op runs.
                             // VIEW/AVIEW/PROMPT write AFTER this line and so survive their own
                             // dispatch; the NEXT op's dispatch clears the override again — matches
                             // HP-41 hardware "VIEW shows until next key" behavior.
    state.display_override = None;
    // ── Phase 3: PRGM mode recording gate (D-03) ────────────────────────────
    if state.prgm_mode {
        // PrgmMode op while recording = exit recording immediately (toggle, Pitfall 6).
        // This op is NOT recorded — it executes immediately to restore normal dispatch.
        if matches!(op, Op::PrgmMode) {
            state.prgm_mode = false;
            apply_lift_effect(state, LiftEffect::Neutral);
            return Ok(());
        }
        // Phase 22 (D-22.10): CLP / DEL / INS are PRGM-mode editing primitives
        // that mutate state.program directly. They MUST execute immediately
        // even while prgm_mode == true — recording them would self-corrupt
        // the program buffer. Fall through to the dispatch match below.
        if !matches!(op, Op::Clp(_) | Op::Del(_) | Op::Ins) {
            // All other ops: append to program Vec; do NOT execute. Stack unmodified.
            state.program.push(op);
            return Ok(());
        }
    }
    match op {
        // ── Phase 1 ops ──────────────────────────────────────────────────
        Op::Add => op_add(state),
        Op::Sub => op_sub(state),
        Op::Mul => op_mul(state),
        Op::Div => op_div(state),
        Op::Enter => op_enter(state),
        Op::Clx => op_clx(state),
        Op::Chs => op_chs(state),
        Op::Rdn => op_rdn(state),
        Op::Rup => op_r_up(state),
        Op::XySwap => op_xy_swap(state),
        Op::Lastx => op_lastx(state),
        Op::Pi => op_pi(state),
        Op::PushNum(v) => {
            crate::stack::enter_number(state, v);
            Ok(())
        }
        // ── Phase 2 math/trig/angle ops (Plan 04) ───────────────────────
        Op::Int => op_int(state),
        // ── Phase 20 unary math additions (D-15/D-16/D-17/D-04..D-07) ───
        Op::Rnd => op_rnd(state),
        Op::Frc => op_frc(state),
        Op::Abs => op_abs(state),
        Op::Sign => op_sign(state),
        Op::Fact => op_fact(state),
        Op::Recip => op_recip(state),
        Op::Sqrt => op_sqrt(state),
        Op::Sq => op_sq(state),
        Op::YPow => op_ypow(state),
        Op::Mod => op_mod(state),
        Op::PctChange => op_pct_change(state),
        Op::Ln => op_ln(state),
        Op::Log => op_log(state),
        Op::Exp => op_exp(state),
        Op::TenPow => op_tenpow(state),
        Op::Sin => op_sin(state),
        Op::Cos => op_cos(state),
        Op::Tan => op_tan(state),
        Op::Asin => op_asin(state),
        Op::Acos => op_acos(state),
        Op::Atan => op_atan(state),
        Op::PolarToRect => op_polar_to_rect(state),
        Op::RectToPolar => op_rect_to_polar(state),
        Op::SetDeg => op_set_deg(state),
        Op::SetRad => op_set_rad(state),
        Op::SetGrad => op_set_grad(state),
        Op::FmtFix(n) => {
            if n > 9 {
                return Err(HpError::InvalidOp);
            }
            state.display_mode = DisplayMode::Fix(n);
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::FmtSci(n) => {
            if n > 9 {
                return Err(HpError::InvalidOp);
            }
            state.display_mode = DisplayMode::Sci(n);
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::FmtEng(n) => {
            if n > 9 {
                return Err(HpError::InvalidOp);
            }
            state.display_mode = DisplayMode::Eng(n);
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::StoReg(r) => op_sto(state, r),
        Op::RclReg(r) => op_rcl(state, r),
        Op::StoArith { reg, kind } => op_sto_arith(state, reg, kind),
        Op::StoArithStack { kind, stack_reg } => op_sto_arith_stack(state, stack_reg, kind),
        Op::Clreg => op_clreg(state),
        Op::AlphaToggle => op_alpha_toggle(state),
        Op::AlphaAppend(ch) => op_alpha_append(state, ch),
        Op::AlphaClear => op_alpha_clear(state),
        Op::AlphaBackspace => op_alpha_backspace(state),
        Op::UserMode => {
            state.user_mode = !state.user_mode;
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        // ── Phase 3: Programming ops ─────────────────────────────────────────
        // Note: PrgmMode exit (prgm_mode=true + Op::PrgmMode) is handled by the gate above.
        // PrgmMode entry (prgm_mode=false) reaches here and sets prgm_mode=true.
        Op::Lbl(_) => program::op_lbl(state),
        Op::Gto(s) => program::op_gto(state, &s),
        Op::Xeq(s) => program::op_xeq(state, &s),
        Op::Rtn => program::op_rtn(state),
        Op::PrgmMode => program::op_prgm_mode(state),
        Op::Test(kind) => program::op_test(state, kind),
        Op::Isg(reg) => {
            // op_isg returns Result<bool>; dispatch() returns Result<()>.
            // Discard the bool skip signal — skip semantics only apply inside run_loop.
            program::op_isg(state, reg).map(|_| ())
        }
        Op::Dse(reg) => {
            // Same as Isg: discard the bool skip signal — skip only applies inside run_loop.
            program::op_dse(state, reg).map(|_| ())
        }
        // ── Phase 6: Science & Engineering ───────────────────────────────────────
        Op::SigmaPlus => stats::op_sigma_plus(state),
        Op::SigmaMinus => stats::op_sigma_minus(state),
        Op::Mean => stats::op_mean(state),
        Op::Sdev => stats::op_sdev(state),
        Op::LR => stats::op_lr(state),
        Op::Yhat => stats::op_yhat(state),
        Op::Corr => stats::op_corr(state),
        Op::ClSigmaStat => stats::op_cl_sigma_stat(state),
        Op::HmsToH => hms::op_hms_to_h(state),
        Op::HToHms => hms::op_h_to_hms(state),
        Op::HmsAdd => hms::op_hms_add(state),
        Op::HmsSub => hms::op_hms_sub(state),
        // ── Phase 11: Print operations ───────────────────────────────────────────────
        Op::PRX => print::op_prx(state),
        Op::PRA => print::op_pra(state),
        Op::PRSTK => print::op_prstk(state),
        // ── Phase 12: Synthetic Programming ─────────────────────────────────
        Op::GetKey => op_getkey(state),
        Op::Null => {
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::StoM => op_sto_m(state),
        Op::StoN => op_sto_n(state),
        Op::StoO => op_sto_o(state),
        Op::RclM => op_rcl_m(state),
        Op::RclN => op_rcl_n(state),
        Op::RclO => op_rcl_o(state),
        Op::SyntheticByte(b) => {
            if let Some(op) = synthetic_byte_to_op(b) {
                // Recursive dispatch — safe: synthetic_byte_to_op never returns
                // Some(Op::SyntheticByte(_)), so recursion depth is exactly 1.
                dispatch(state, op)
            } else {
                Err(HpError::InvalidOp)
            }
        }
        // ── Card Reader ───────────────────────────────────────────────────
        Op::Wdta => op_wdta(state),
        Op::Rdta => op_rdta(state),
        Op::Wprgm => op_wprgm(state),
        Op::Rdprgm => op_rdprgm(state),
        // ── Phase 21: Flags ───────────────────────────────────────────────
        Op::SfFlag(n) => flags::op_sf(state, n),
        Op::CfFlag(n) => flags::op_cf(state, n),
        // Interactive FlagTest is a no-op (mirrors Op::Test). The skip-next-step
        // and always-clear semantics live in run_loop. At the keyboard there is
        // no "next program step" to skip; pc and flags are untouched.
        Op::FlagTest { .. } => {
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        // ── Phase 21: Display Control ─────────────────────────────────────
        Op::View(r) => display_ops::op_view(state, r),
        Op::AView => display_ops::op_aview(state),
        Op::Prompt => display_ops::op_prompt(state),
        Op::Aon => display_ops::op_aon(state),
        Op::Aoff => display_ops::op_aoff(state),
        Op::Cld => display_ops::op_cld(state),
        // ── Phase 21: Sound ───────────────────────────────────────────────
        Op::Beep => sound::op_beep(state),
        Op::Tone(n) => sound::op_tone(state, n),
        // ── Phase 22: Program control ─────────────────────────────────────
        // Interactive Op::Stop (is_running == false) is a Neutral no-op per D-22.5.
        // The break-run_loop semantic only fires inside run_loop's match.
        Op::Stop => {
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        // PSE writes display_override + event_buffer "PAUSE 1000" then continues.
        // dispatch() has already called flush_entry_buf at the top, so any
        // in-progress digit entry was lifted to X before we format it
        // (Pitfall 10 — do NOT add a second flush_entry_buf here).
        Op::Pse => {
            let formatted = crate::format::format_hpnum(&state.stack.x, &state.display_mode);
            state.display_override = Some(formatted);
            state.event_buffer.push("PAUSE 1000".to_string());
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        // GTO IND / XEQ IND require the run_loop state machine to manipulate
        // pc and call_stack — interactive dispatch outside a running program
        // is undefined. Return InvalidOp; run_loop handles them directly.
        Op::GtoInd(_) | Op::XeqInd(_) => Err(HpError::InvalidOp),
        // ── Phase 22: Program editing (D-22.7, D-22.8, D-22.9, D-22.10) ──
        // CLP/DEL/INS are PRGM-mode editing primitives that mutate
        // state.program directly. They are NEVER recorded into the program
        // even when state.prgm_mode == true — the prgm_mode gate above
        // would otherwise append them to state.program. The helpers
        // themselves re-check prgm_mode (defense-in-depth, D-22.10).
        //
        // Important: the prgm_mode gate ABOVE this match block (line ~436)
        // only fires when state.prgm_mode == true. We reach this arm in two
        // cases:
        //   (a) prgm_mode == false (interactive, edit-primitives disallowed)
        //       → helper guard returns InvalidOp.
        //   (b) prgm_mode == true AND the gate's recording branch was
        //       bypassed by the special-case above for Clp/Del/Ins
        //       (added below). See the gate modification.
        Op::Clp(name) => program::op_clp(state, &name),
        Op::Del(n) => program::op_del(state, n),
        Op::Ins => program::op_ins(state),
        // ── Phase 22: Memory & stack management (D-22.11..13) ────────────
        // SIZE executes fine inside run_loop and interactively — it is a
        // regular dispatch op, not a control-flow primitive.
        Op::Size(n) => program::op_size(state, n),
        // D-22.13: Op::Cla delegates to op_alpha_clear (same body as
        // Op::AlphaClear). Two variants intentionally coexist: hardware-
        // faithful "CLA" display name (this variant) vs the v1.0-save
        // "CLRALPHA" legacy display (Op::AlphaClear). Pitfall 8: do NOT
        // remove Op::AlphaClear — v1.0 save files contain it.
        Op::Cla => op_alpha_clear(state),
        // D-22.14: CLST zeros X/Y/Z/T while preserving LASTX and
        // lift_enabled. Critical divergence from Clreg (which only
        // clears regs).
        Op::Clst => program::op_clst(state),
        // D-22.12: PACK is a documented no-op on the flat-Vec program
        // model (no gaps to compact). Neutral lift. Inline body matches
        // Op::Null / Op::AlphaToggle pattern.
        Op::Pack => {
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        // D-22.16 (AMENDED OQ-1 Option B): CATALOG n — hardware-faithful.
        // CAT 1 = programs (LBL listing); CAT 2/3/4 = "NOT AVAILABLE".
        // Output goes to state.print_buffer (Phase 11 drain pattern).
        Op::Catalog(n) => program::op_catalog(state, n),
        // D-22.18 (AMENDED OQ-3 Option A): ASN — empty `name` removes the
        // assignment for `key_code`; non-empty inserts/overwrites. hp41-core
        // stores as String; resolution at USER-mode dispatch (Phase 25/26)
        // per D-22.19. The owned String moves into op_asn (Op is consumed
        // by value here).
        Op::Asn { name, key_code } => program::op_asn(state, name, key_code),
        // ── Phase 23: ALPHA-register operations (D-23.12) ─────────────────
        // ARCL/ASTO both Neutral lift; both reuse format_hpnum / HpNum::zero
        // rather than re-deriving display or zero-value logic. ASTO writes
        // the packed-text shadow AND zeroes the numeric slot (no-drift).
        Op::Arcl(reg) => alpha::op_arcl(state, reg),
        Op::Asto(reg) => alpha::op_asto(state, reg),
        // Phase 23 plan 02 (FN-ALPHA-03..06): ATOX Enable, XTOA Neutral,
        // AROT Neutral, POSA Disable (D-23.16). All four touch only
        // `alpha_reg` and `stack.x` (POSA writes X; others read/preserve).
        Op::Atox => alpha::op_atox(state),
        Op::Xtoa => alpha::op_xtoa(state),
        Op::Arot => alpha::op_arot(state),
        Op::Posa => alpha::op_posa(state),
        // -- Phase 24: Indirect Addressing dispatch (FN-IND-01, FN-IND-02) -
        // Each arm resolves the indirect pointer via `resolve_indirect` and
        // delegates to its direct-form op. IsgInd/DseInd discard the bool
        // skip signal (mirrors `Op::Isg` / `Op::Dse` pattern above) -- skip
        // semantics live in run_loop only. FlagTestInd is a Neutral no-op
        // (mirrors `Op::FlagTest { .. }` above).
        Op::StoInd(reg) => indirect::op_sto_ind(state, reg),
        Op::RclInd(reg) => indirect::op_rcl_ind(state, reg),
        Op::StoArithInd(reg, kind) => indirect::op_sto_arith_ind(state, reg, kind),
        Op::IsgInd(reg) => indirect::op_isg_ind(state, reg).map(|_| ()),
        Op::DseInd(reg) => indirect::op_dse_ind(state, reg).map(|_| ()),
        Op::SfFlagInd(reg) => indirect::op_sf_flag_ind(state, reg),
        Op::CfFlagInd(reg) => indirect::op_cf_flag_ind(state, reg),
        Op::FlagTestInd { .. } => {
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::ArclInd(reg) => indirect::op_arcl_ind(state, reg),
        Op::AstoInd(reg) => indirect::op_asto_ind(state, reg),
        Op::ViewInd(reg) => indirect::op_view_ind(state, reg),
        // ── Phase 28: Hyperbolics (Plan 28-02) ────────────────────────────────────
        Op::Sinh => op_sinh(state),
        Op::Cosh => op_cosh(state),
        Op::Tanh => op_tanh(state),
        Op::Asinh => op_asinh(state),
        Op::Acosh => op_acosh(state),
        Op::Atanh => op_atanh(state),
        // ── Phase 28: Complex Stack Arithmetic (Plan 28-03) ───────────────────────
        Op::CPlus => op_c_plus(state),
        Op::CMinus => op_c_minus(state),
        Op::CTimes => op_c_times(state),
        Op::CDiv => op_c_div(state),
        Op::Real => op_real(state),
        // ── Phase 28: Complex Functions (Plan 28-04) ─────────────────────────────
        Op::Magz => op_magz(state),
        Op::Cinv => op_cinv(state),
        Op::ZpowN => op_z_pow_n(state),
        Op::Zpow1N => op_z_pow_1_n(state),
        Op::ExpZ => op_exp_z(state),
        Op::LnZ => op_ln_z(state),
        Op::SinZ => op_sin_z(state),
        Op::CosZ => op_cos_z(state),
        Op::TanZ => op_tan_z(state),
        Op::ApowZ => op_a_pow_z(state),
        Op::LogZ => op_log_z(state),
        Op::ZpowW => op_z_pow_w(state),
        // ── Phase 28: POLY / ROOTS (Plan 28-05) ─────────────────────────────
        Op::PolyWorkflow => op_poly_workflow(state),
        Op::Roots => op_roots(state),
        // ── Phase 28: MATRIX (Plan 28-06) ────────────────────────────────────
        Op::MatrixWorkflow => op_matrix_workflow(state),
        Op::MatSize => op_mat_size(state),
        Op::MatVmat => op_mat_vmat(state),
        Op::MatEdit => op_mat_edit(state),
        Op::MatDet => op_mat_det(state),
        Op::MatInv => op_mat_inv(state),
        Op::MatSimeq => op_mat_simeq(state),
        Op::MatVcol => op_mat_vcol(state),
    }
}

// ── Phase 12: Synthetic Byte Subset (D-11, D-12) ─────────────────────────────
//
// Maps HP-41 NUT/FOCAL byte codes to already-implemented Op variants.
// This is a CONSERVATIVE initial table — covers ~15 well-known single-byte
// codes. Codes outside this table are rejected at the hex modal entry point
// (D-13: app.message = "INVALID"). Expandable in v2+ as part of SYNT-05.
//
// CRITICAL INVARIANT: this function MUST NOT return Some(Op::SyntheticByte(_))
// — that would cause infinite recursion in dispatch() / execute_op().
//
// [ASSUMED] — exact NUT byte codes from secondary sources. Cross-verify
// against HP-41 FOCAL reference if precision is needed for a specific code.

/// Map an HP-41 byte code to the corresponding Op, if it is in the safe subset.
/// Returns `None` for codes outside the curated subset.
pub fn synthetic_byte_to_op(byte: u8) -> Option<Op> {
    match byte {
        // Arithmetic (HP-41 single-byte FOCAL codes — [ASSUMED])
        0x40 => Some(Op::Add),
        0x41 => Some(Op::Sub),
        0x42 => Some(Op::Mul),
        0x43 => Some(Op::Div),
        // Stack ops
        0x4F => Some(Op::Chs),
        0x73 => Some(Op::Clx),
        0x74 => Some(Op::Rdn),
        0x71 => Some(Op::XySwap),
        // Math
        0x52 => Some(Op::Sqrt),
        0x53 => Some(Op::Sq),
        0x57 => Some(Op::Log),
        0x67 => Some(Op::Ln),
        0x60 => Some(Op::Recip),
        // Trig
        0x59 => Some(Op::Sin),
        0x5A => Some(Op::Cos),
        0x5B => Some(Op::Tan),
        // Synthetic primitives — primary purpose of the hex modal
        0xCF => Some(Op::Null),
        0xCE => Some(Op::GetKey),
        // Hidden register access — synthetic byte path mirrors the new Op variants
        0xB0 => Some(Op::StoM),
        0xB1 => Some(Op::StoN),
        0xB2 => Some(Op::StoO),
        0x90 => Some(Op::RclM),
        0x91 => Some(Op::RclN),
        0x92 => Some(Op::RclO),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod flush_eex_tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn make_state_with_entry(s: &str) -> CalcState {
        let mut state = CalcState::new();
        state.entry_buf = s.to_string();
        state
    }

    #[test]
    fn test_flush_scientific_lowercase_e() {
        let mut state = make_state_with_entry("1.5e3");
        flush_entry_buf(&mut state).unwrap();
        assert_eq!(state.stack.x.0, Decimal::from(1500));
    }

    #[test]
    fn test_flush_scientific_uppercase_e() {
        let mut state = make_state_with_entry("2.5E-2");
        flush_entry_buf(&mut state).unwrap();
        assert_eq!(state.stack.x.0, Decimal::from_str("0.025").unwrap());
    }

    #[test]
    fn test_flush_plain_decimal_still_works() {
        let mut state = make_state_with_entry("1500");
        flush_entry_buf(&mut state).unwrap();
        assert_eq!(state.stack.x.0, Decimal::from(1500));
    }

    #[test]
    fn test_flush_invalid_returns_err() {
        let mut state = make_state_with_entry("notanumber");
        assert!(flush_entry_buf(&mut state).is_err());
    }

    #[test]
    fn test_flush_trailing_e_without_exponent_commits_zero_exponent() {
        // HP-41 hardware: trailing 'e' with no exponent digits commits as exponent 00.
        // "1.5e" + ENTER pushes 1.5 to the stack (exponent treated as 00), not a parse error.
        // Per D-09 (Phase 9 CONTEXT): flush_entry_buf normalizes by appending "00" before parsing.
        let mut state = make_state_with_entry("1.5e");
        let result = flush_entry_buf(&mut state);
        assert!(
            result.is_ok(),
            "trailing 'e' with no exponent must commit as exponent 00, not Err"
        );
        assert_eq!(
            state.stack.x.0,
            Decimal::from_str("1.5").unwrap(),
            "1.5e must commit as 1.5 (exponent 00)"
        );
        assert!(
            state.entry_buf.is_empty(),
            "entry_buf must be cleared after successful commit"
        );
    }

    #[test]
    fn test_flush_implicit_one_with_trailing_e_commits_one() {
        // HP-41 hardware: empty-buffer EEX inserts implicit "1" mantissa (D-07 in app.rs).
        // After app.rs sets entry_buf = "1e", flush must commit as 1.0 (1 * 10^0).
        let mut state = make_state_with_entry("1e");
        let result = flush_entry_buf(&mut state);
        assert!(result.is_ok(), "1e must commit successfully");
        assert_eq!(
            state.stack.x.0,
            Decimal::from(1),
            "1e must commit as 1 (1 * 10^0)"
        );
    }

    #[test]
    fn test_flush_trailing_e_minus_parses_as_one() {
        // HP-41 hardware: "1e-" + ENTER commits as 1.0 (exponent -00 = 0).
        // flush_entry_buf normalizes "1e-" → "1e-00" before parsing.
        let mut state = make_state_with_entry("1e-");
        flush_entry_buf(&mut state).unwrap();
        assert!(state.entry_buf.is_empty());
        // "1e-00" == 1.0
        use crate::num::HpNum;
        assert_eq!(state.stack.x, HpNum::from(1i32));
    }

    #[test]
    fn test_flush_entry_buf_negative_exponent() {
        // "1e-2" is a complete negative exponent — parses directly as 0.01.
        let mut state = make_state_with_entry("1e-2");
        flush_entry_buf(&mut state).unwrap();
        assert!(state.entry_buf.is_empty());
        // 1e-2 == 0.01
        assert_eq!(state.stack.x.0, Decimal::from_str("0.01").unwrap());
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::state::CalcState;

    #[test]
    fn test_user_mode_toggle() {
        let mut state = CalcState::new();
        assert!(!state.user_mode);
        dispatch(&mut state, Op::UserMode).unwrap();
        assert!(state.user_mode, "UserMode must flip to true");
        dispatch(&mut state, Op::UserMode).unwrap();
        assert!(!state.user_mode, "second UserMode must flip back to false");
    }

    #[test]
    fn test_op_serde_round_trip() {
        // Verify Op::Add serializes as JSON string, not a complex structure
        let json = serde_json::to_string(&Op::Add).unwrap();
        let back: Op = serde_json::from_str(&json).unwrap();
        assert_eq!(Op::Add, back);
    }

    #[test]
    fn test_pushnum_serde_round_trip() {
        use crate::num::HpNum;
        let op = Op::PushNum(HpNum::from(42i32));
        let json = serde_json::to_string(&op).unwrap();
        let back: Op = serde_json::from_str(&json).unwrap();
        assert_eq!(op, back);
    }
}
