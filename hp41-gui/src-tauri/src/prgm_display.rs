//! PRGM mode step display for the HP-41 calculator GUI.
//!
//! format_step() renders the current program counter position as a human-readable
//! step string: "{pc:03} {op_name}" (D-14). Shown in the Display panel when
//! CalcState::prgm_mode is true.
//!
//! format_all_steps() pre-formats the entire program listing for the React frontend.
//! Copied from hp41-cli/src/prgm_display.rs per Phase 18 D-03.

use hp41_core::ops::{Op, StackReg, StoArithKind};
use hp41_core::CalcState;

/// Format the current program step.
/// Returns "{pc:03} {op_name}" where op_name is the HP-41 key label for the Op.
/// Returns "{pc:03} END" when pc is at or beyond the end of the program.
#[allow(dead_code)]
pub fn format_step(state: &CalcState) -> String {
    let step_num = state.pc;
    let op_name = state
        .program
        .get(step_num)
        .map(op_display_name)
        .unwrap_or_else(|| "END".to_string());
    format!("{step_num:03} {op_name}")
}

/// Format all program steps as pre-rendered strings for the React frontend.
/// Returns ["000 END"] for an empty program (always at least one row).
/// Index 0 = step 000; each string is "{index:03} {op_display_name}".
/// Note: do NOT use format_step() in a loop — it reads state.pc, not the index.
pub fn format_all_steps(state: &CalcState) -> Vec<String> {
    let mut steps: Vec<String> = state
        .program
        .iter()
        .enumerate()
        .map(|(i, op)| format!("{i:03} {}", op_display_name(op)))
        .collect();
    // Always append END so pc == program.len() has a matching row to highlight
    steps.push(format!("{:03} END", state.program.len()));
    steps
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
        Op::XySwap => "X\u{27F7}Y".to_string(),
        Op::Lastx => "LASTX".to_string(),
        Op::PushNum(n) => format!("{}", n.inner()),
        // Phase 2: unary math
        Op::Int => "INT".to_string(),
        Op::Recip => "1/x".to_string(),
        Op::Sqrt => "\u{221a}x".to_string(),
        Op::Sq => "x\u{00B2}".to_string(),
        Op::YPow => "Y^X".to_string(),
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
        Op::SyntheticByte(b) => format!("SYN {:02X}", b),
        // Card Reader
        Op::Wdta => "WDTA".to_string(),
        Op::Rdta => "RDTA".to_string(),
        Op::Wprgm => "WPRGM".to_string(),
        Op::Rdprgm => "RDPRGM".to_string(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_format_all_steps_empty_program() {
        let state = hp41_core::CalcState::new();
        let steps = format_all_steps(&state);
        assert_eq!(steps, vec!["000 END"]);
    }

    #[test]
    fn test_format_all_steps_nonempty() {
        use hp41_core::ops::Op;
        let mut state = hp41_core::CalcState::new();
        state.program = vec![Op::Add, Op::Enter];
        let steps = format_all_steps(&state);
        assert_eq!(
            steps[0], "000 + ",
            "step 0 must match op_display_name(Op::Add)"
        );
        assert_eq!(
            steps[1], "001 ENTER",
            "step 1 must match op_display_name(Op::Enter)"
        );
    }

    /// PR #5 review (pr-test-analyzer) — the whole point of commit 3372ec3
    /// (`format_all_steps always appends END so pc==program.len() highlights
    /// correctly`) is that the listing has one more row than the program.
    /// The trailing END row was not asserted by any test; add it here.
    #[test]
    fn test_format_all_steps_appends_end_row() {
        use hp41_core::ops::Op;
        let mut state = hp41_core::CalcState::new();
        state.program = vec![Op::Add, Op::Enter];
        let steps = format_all_steps(&state);
        assert_eq!(
            steps.len(),
            state.program.len() + 1,
            "format_all_steps must always append an END row so pc==program.len() highlights"
        );
        assert_eq!(
            steps[steps.len() - 1],
            "002 END",
            "trailing row must be the END marker at index program.len()"
        );
    }
}
