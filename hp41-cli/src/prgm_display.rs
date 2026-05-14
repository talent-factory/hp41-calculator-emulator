//! PRGM mode step display for the HP-41 calculator TUI.
//!
//! format_step() renders the current program counter position as a human-readable
//! step string: "{pc:03} {op_name}" (D-14). Shown in the Display panel when
//! CalcState::prgm_mode is true.

use hp41_core::ops::{FlagTestKind, Op, StackReg, StoArithKind};
use hp41_core::CalcState;

/// Format the current program step.
/// Returns "{pc:03} {op_name}" where op_name is the HP-41 key label for the Op.
/// Returns "{pc:03} END" when pc is at or beyond the end of the program.
pub fn format_step(state: &CalcState) -> String {
    let step_num = state.pc;
    let op_name = state
        .program
        .get(step_num)
        .map(op_display_name)
        .unwrap_or_else(|| "END".to_string());
    format!("{step_num:03} {op_name}")
}

/// Map an Op variant to its HP-41 display name.
/// Uses String return type (not &'static str) because variants like Lbl(String),
/// PushNum(HpNum), FmtFix(u8) require dynamic content.
/// Covers all 35 Op variants exhaustively — no non-exhaustive patterns warning.
fn op_display_name(op: &Op) -> String {
    match op {
        // Phase 1: arithmetic
        Op::Add => "+ ".to_string(),
        Op::Sub => "- ".to_string(),
        Op::Mul => "\u{00D7} ".to_string(),
        Op::Div => "\u{00F7} ".to_string(),
        // Phase 1: stack
        Op::Enter => "ENTER".to_string(),
        Op::Clx => "CLX".to_string(),
        Op::Chs => "CHS".to_string(),
        Op::Rdn => "R\u{2193}".to_string(),
        Op::Rup => "R\u{2191}".to_string(),
        Op::XySwap => "X\u{27F7}Y".to_string(),
        Op::Lastx => "LASTX".to_string(),
        Op::Pi => "PI".to_string(),
        Op::PushNum(n) => format!("{}", n.inner()),
        // Phase 2: unary math
        Op::Int => "INT".to_string(),
        // Phase 20: additional unary math
        Op::Rnd => "RND".to_string(),
        Op::Frc => "FRC".to_string(),
        Op::Abs => "ABS".to_string(),
        Op::Sign => "SIGN".to_string(),
        Op::Fact => "FACT".to_string(),
        Op::Recip => "1/x".to_string(),
        Op::Sqrt => "\u{221a}x".to_string(),
        Op::Sq => "x\u{00B2}".to_string(),
        Op::YPow => "Y^X".to_string(),
        Op::Mod => "MOD".to_string(),
        Op::PctChange => "%CH".to_string(),
        Op::Ln => "LN".to_string(),
        Op::Log => "LOG".to_string(),
        Op::Exp => "e^x".to_string(),
        Op::TenPow => "10^x".to_string(),
        // Phase 2: trig
        Op::Sin => "SIN".to_string(),
        Op::Cos => "COS".to_string(),
        Op::Tan => "TAN".to_string(),
        Op::Asin => "ASIN".to_string(),
        Op::Acos => "ACOS".to_string(),
        Op::Atan => "ATAN".to_string(),
        Op::PolarToRect => "P\u{2192}R".to_string(),
        Op::RectToPolar => "R\u{2192}P".to_string(),
        // Phase 2: angle mode
        Op::SetDeg => "DEG".to_string(),
        Op::SetRad => "RAD".to_string(),
        Op::SetGrad => "GRAD".to_string(),
        // Phase 2: display mode
        Op::FmtFix(n) => format!("FIX {n}"),
        Op::FmtSci(n) => format!("SCI {n}"),
        Op::FmtEng(n) => format!("ENG {n}"),
        // Phase 2: registers
        Op::StoReg(r) => format!("STO {r:02}"),
        Op::RclReg(r) => format!("RCL {r:02}"),
        Op::StoArith { reg, kind } => {
            let op_sym = match kind {
                StoArithKind::Add => "+",
                StoArithKind::Sub => "-",
                StoArithKind::Mul => "\u{00D7}",
                StoArithKind::Div => "\u{00F7}",
            };
            format!("STO{op_sym} {reg:02}")
        }
        Op::StoArithStack { kind, stack_reg } => {
            let op_sym = match kind {
                StoArithKind::Add => "+",
                StoArithKind::Sub => "-",
                StoArithKind::Mul => "\u{00D7}",
                StoArithKind::Div => "\u{00F7}",
            };
            let reg_name = match stack_reg {
                StackReg::Y => "Y",
                StackReg::Z => "Z",
                StackReg::T => "T",
                StackReg::LastX => "L",
            };
            format!("STO{op_sym} {reg_name}")
        }
        Op::Clreg => "CLREG".to_string(),
        // Phase 2: alpha
        Op::AlphaToggle => "ALPHA".to_string(),
        Op::AlphaAppend(c) => format!("'{c}'"),
        Op::AlphaClear => "CLRALPHA".to_string(),
        // Phase 3: programming
        Op::Lbl(s) => format!("LBL {s}"),
        Op::Gto(s) => format!("GTO {s}"),
        Op::Xeq(s) => format!("XEQ {s}"),
        Op::Rtn => "RTN".to_string(),
        Op::PrgmMode => "PRGM".to_string(),
        Op::Test(_) => "TEST".to_string(),
        Op::Isg(r) => format!("ISG {r:02}"),
        Op::Dse(r) => format!("DSE {r:02}"),
        // Phase 5: new Op variants
        Op::UserMode => "USER".to_string(),
        Op::AlphaBackspace => "\u{2190}".to_string(),
        // Phase 6: Science & Engineering
        Op::SigmaPlus => "\u{03A3}+".to_string(),
        Op::SigmaMinus => "\u{03A3}-".to_string(),
        Op::Mean => "MEAN".to_string(),
        Op::Sdev => "SDEV".to_string(),
        Op::LR => "L.R.".to_string(),
        Op::Yhat => "\u{0177}".to_string(),
        Op::Corr => "CORR".to_string(),
        Op::ClSigmaStat => "CL\u{03A3}".to_string(),
        Op::HmsToH => "HMS\u{2192}".to_string(),
        Op::HToHms => "\u{2192}HMS".to_string(),
        Op::HmsAdd => "HMS+".to_string(),
        Op::HmsSub => "HMS-".to_string(),
        // Phase 11: Print operations
        Op::PRX => "PRX".to_string(),
        Op::PRA => "PRA".to_string(),
        Op::PRSTK => "PRSTK".to_string(),
        // Phase 12: Synthetic Programming
        Op::GetKey => "GETKEY".to_string(),
        Op::Null => "NULL".to_string(),
        Op::StoM => "STO M".to_string(),
        Op::StoN => "STO N".to_string(),
        Op::StoO => "STO O".to_string(),
        Op::RclM => "RCL M".to_string(),
        Op::RclN => "RCL N".to_string(),
        Op::RclO => "RCL O".to_string(),
        Op::SyntheticByte(b) => format!("SYN {b:02X}"),
        // Card Reader
        Op::Wdta => "WDTA".to_string(),
        Op::Rdta => "RDTA".to_string(),
        Op::Wprgm => "WPRGM".to_string(),
        Op::Rdprgm => "RDPRGM".to_string(),
        // Phase 21: Flags
        Op::SfFlag(n) => format!("SF {n:02}"),
        Op::CfFlag(n) => format!("CF {n:02}"),
        Op::FlagTest { kind, flag } => {
            let mnemonic = match kind {
                FlagTestKind::IsSet => "FS?",
                FlagTestKind::IsClear => "FC?",
                FlagTestKind::IsSetThenClear => "FS?C",
                FlagTestKind::IsClearThenClear => "FC?C",
            };
            format!("{mnemonic} {flag:02}")
        }
        // Phase 21: Display Control
        Op::View(r) => format!("VIEW {r:02}"),
        Op::AView => "AVIEW".to_string(),
        Op::Prompt => "PROMPT".to_string(),
        Op::Aon => "AON".to_string(),
        Op::Aoff => "AOFF".to_string(),
        Op::Cld => "CLD".to_string(),
        // Phase 21: Sound
        Op::Beep => "BEEP".to_string(),
        Op::Tone(n) => format!("TONE {n}"),
        // Phase 22: Program control
        Op::Stop => "STOP".to_string(),
        Op::Pse => "PSE".to_string(),
        Op::GtoInd(r) => format!("GTO IND {r:02}"),
        Op::XeqInd(r) => format!("XEQ IND {r:02}"),
        // Phase 22: Program editing
        Op::Clp(name) => format!("CLP {name}"),
        Op::Del(n) => format!("DEL {n:03}"),
        Op::Ins => "INS".to_string(),
        // Phase 22: Memory management (D-22.11, D-22.13)
        Op::Size(n) => format!("SIZE {n:03}"),
        // D-22.13: NOT "CLRALPHA" — that is Op::AlphaClear's display name.
        // Both variants coexist for hardware-faithful listing (CLA) vs v1.0
        // save-file compat (CLRALPHA). Pitfall 8: do NOT consolidate.
        Op::Cla => "CLA".to_string(),
        // D-22.14: CLST clears X/Y/Z/T (LASTX + lift_enabled preserved).
        Op::Clst => "CLST".to_string(),
        // D-22.12: PACK is a documented no-op (flat-Vec has no gaps).
        Op::Pack => "PACK".to_string(),
        // Phase 22: Catalog (D-22.16 AMENDED OQ-1 Option B)
        Op::Catalog(n) => format!("CATALOG {n}"),
        // Phase 22: ASN (D-22.18 AMENDED OQ-3 Option A)
        Op::Asn { name, key_code } => format!("ASN \"{name}\" {key_code:02}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_phase12_op_labels() {
        // UAT-2: program listing shows correct labels for all Phase 12 Op variants.
        assert_eq!(op_display_name(&Op::GetKey), "GETKEY");
        assert_eq!(op_display_name(&Op::Null), "NULL");
        assert_eq!(op_display_name(&Op::StoM), "STO M");
        assert_eq!(op_display_name(&Op::StoN), "STO N");
        assert_eq!(op_display_name(&Op::StoO), "STO O");
        assert_eq!(op_display_name(&Op::RclM), "RCL M");
        assert_eq!(op_display_name(&Op::RclN), "RCL N");
        assert_eq!(op_display_name(&Op::RclO), "RCL O");
        // SyntheticByte displays as "SYN <HEX>" — 0xCF = NULL, 0xCE = GETKEY
        assert_eq!(op_display_name(&Op::SyntheticByte(0xCF)), "SYN CF");
        assert_eq!(op_display_name(&Op::SyntheticByte(0xCE)), "SYN CE");
        assert_eq!(op_display_name(&Op::SyntheticByte(0xA0)), "SYN A0");
    }

    #[test]
    fn test_display_phase20_op_labels() {
        // Phase 20: 10 new Op variants must surface the documented HP-41
        // mnemonics in the program listing (D-22 4-place rule, fourth place).
        assert_eq!(op_display_name(&Op::Pi), "PI");
        assert_eq!(op_display_name(&Op::Rup), "R\u{2191}");
        assert_eq!(op_display_name(&Op::PolarToRect), "P\u{2192}R");
        assert_eq!(op_display_name(&Op::RectToPolar), "R\u{2192}P");
        assert_eq!(op_display_name(&Op::Rnd), "RND");
        assert_eq!(op_display_name(&Op::Frc), "FRC");
        assert_eq!(op_display_name(&Op::Mod), "MOD");
        assert_eq!(op_display_name(&Op::Abs), "ABS");
        assert_eq!(op_display_name(&Op::Fact), "FACT");
        assert_eq!(op_display_name(&Op::Sign), "SIGN");
    }
}
