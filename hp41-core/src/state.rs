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

/// The complete, mutable state of the HP-41 calculator.
///
/// All operations take `&mut CalcState`. No global mutable state anywhere.
#[derive(Debug, Clone)]
pub struct CalcState {
    pub stack: Stack,
    // Phase 2 additions: regs: [HpNum; 100], alpha: String, flags: CalcFlags
}

impl CalcState {
    pub fn new() -> Self {
        CalcState {
            stack: Stack::new(),
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
/// Registers: X (visible), Y, Z, T (bottom). T is duplicated (not dropped) on lift.
/// lift_enabled: true means the next number entry will lift the stack before writing X.
#[derive(Debug, Clone)]
pub struct Stack {
    /// X register — the currently visible value
    pub x: HpNum,
    /// Y register
    pub y: HpNum,
    /// Z register
    pub z: HpNum,
    /// T register (top/bottom depending on perspective; duplicated on lift)
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
