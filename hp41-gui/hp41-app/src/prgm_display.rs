//! Shared PRGM mode step display for HP-41 application shells.
//!
//! format_step() renders the current program counter position as a human-readable
//! step string: "{pc:03} {op_name}" (D-14). Shown in the Display panel when
//! CalcState::prgm_mode is true.
//!
//! format_all_steps() pre-formats the entire program listing for the React frontend.
//! Copied from hp41-cli/src/prgm_display.rs per Phase 18 D-03.

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
/// Covers all Op variants exhaustively (v2.2 built-ins + Math Pac I + Stat 1 Pac + Time Module + Advantage Pac).
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
        Op::Clreg => "CLRG".to_string(),
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
        // Phase 23: ALPHA-register operations (D-23.12)
        Op::Arcl(reg) => format!("ARCL {reg:02}"),
        Op::Asto(reg) => format!("ASTO {reg:02}"),
        // Phase 23 plan 02 (FN-ALPHA-03..06): bare-string variants (mirror
        // of the hp41-cli copy — SC-4 invariant requires identical display
        // listing on both frontends).
        Op::Atox => "ATOX".to_string(),
        Op::Xtoa => "XTOA".to_string(),
        Op::Arot => "AROT".to_string(),
        Op::Posa => "POSA".to_string(),
        // Phase 24: Indirect Addressing (FN-IND-01) -- mirror across CLI + GUI
        // per SC-4 invariant. Space-separated MNEMONIC IND nn (Phase-22 form).
        Op::StoInd(r) => format!("STO IND {r:02}"),
        Op::RclInd(r) => format!("RCL IND {r:02}"),
        Op::IsgInd(r) => format!("ISG IND {r:02}"),
        Op::DseInd(r) => format!("DSE IND {r:02}"),
        Op::SfFlagInd(r) => format!("SF IND {r:02}"),
        Op::CfFlagInd(r) => format!("CF IND {r:02}"),
        Op::ArclInd(r) => format!("ARCL IND {r:02}"),
        Op::AstoInd(r) => format!("ASTO IND {r:02}"),
        Op::ViewInd(r) => format!("VIEW IND {r:02}"),
        Op::StoArithInd(reg, kind) => {
            let op_sym = match kind {
                StoArithKind::Add => "+",
                StoArithKind::Sub => "-",
                StoArithKind::Mul => "\u{00D7}",
                StoArithKind::Div => "\u{00F7}",
            };
            format!("STO{op_sym} IND {reg:02}")
        }
        Op::FlagTestInd { kind, ind_reg } => {
            let mnemonic = match kind {
                FlagTestKind::IsSet => "FS?",
                FlagTestKind::IsClear => "FC?",
                FlagTestKind::IsSetThenClear => "FS?C",
                FlagTestKind::IsClearThenClear => "FC?C",
            };
            format!("{mnemonic} IND {ind_reg:02}")
        }
        // ── Phase 28: Hyperbolics (Plan 28-02) ────────────────────────────────────
        Op::Sinh => "SINH".to_string(),
        Op::Cosh => "COSH".to_string(),
        Op::Tanh => "TANH".to_string(),
        Op::Asinh => "ASINH".to_string(),
        Op::Acosh => "ACOSH".to_string(),
        Op::Atanh => "ATANH".to_string(),
        // ── Phase 28: Complex Stack Arithmetic (Plan 28-03) ───────────────────────
        Op::CPlus => "C+".to_string(),
        Op::CMinus => "C-".to_string(),
        Op::CTimes => "C\u{00D7}".to_string(),
        Op::CDiv => "C\u{00F7}".to_string(),
        Op::Real => "REAL".to_string(),
        // ── Phase 28: Complex Functions (Plan 28-04) ─────────────────────────────
        Op::Magz => "MAGZ".to_string(),
        Op::Cinv => "CINV".to_string(),
        Op::ZpowN => "Z\u{2191}N".to_string(),
        Op::Zpow1N => "Z\u{2191}1/N".to_string(),
        Op::ExpZ => "E\u{2191}Z".to_string(),
        Op::LnZ => "LNZ".to_string(),
        Op::SinZ => "SINZ".to_string(),
        Op::CosZ => "COSZ".to_string(),
        Op::TanZ => "TANZ".to_string(),
        Op::ApowZ => "A\u{2191}Z".to_string(),
        Op::LogZ => "LOGZ".to_string(),
        Op::ZpowW => "Z\u{2191}W".to_string(),
        // ── Phase 28: POLY / ROOTS (Plan 28-05) ────────────────────────────────────
        Op::PolyWorkflow => "POLY".to_string(),
        Op::Roots => "ROOTS".to_string(),
        // ── Phase 28: MATRIX (Plan 28-06) ────────────────────────────────────────
        Op::MatrixWorkflow => "MATRIX".to_string(),
        Op::MatSize => "SIZE".to_string(),
        Op::MatVmat => "VMAT".to_string(),
        Op::MatEdit => "EDIT".to_string(),
        Op::MatDet => "DET".to_string(),
        Op::MatInv => "INV".to_string(),
        Op::MatSimeq => "SIMEQ".to_string(),
        Op::MatVcol => "VCOL".to_string(),
        // ── Phase 28: INTG (Plan 28-07) ────────────────────────────────────────
        Op::Integ => "INTG".to_string(),
        // ── Phase 28: SOLVE / SOL (Plan 28-08) ────────────────────────────────
        Op::Solve => "SOLVE".to_string(),
        Op::Sol => "SOL".to_string(),
        // ── Phase 28: DIFEQ (Plan 28-09) ──────────────────────────────────────
        Op::Difeq => "DIFEQ".to_string(),
        // ── Phase 28: FOUR / Triangle Solvers / TRANS (Plan 28-10) ────────────────
        Op::Four => "FOUR".to_string(),
        Op::TriSss => "SSS".to_string(),
        Op::TriAsa => "ASA".to_string(),
        Op::TriSaa => "SAA".to_string(),
        Op::TriSas => "SAS".to_string(),
        Op::TriSsa => "SSA".to_string(),
        Op::Trans2d => "TRANS".to_string(),
        Op::Trans3d => "T3D".to_string(),
        // ── Stat 1 Pac Univariate / Bivariate ──────────────────────────────────
        Op::SigmaBstat => "\u{03A3}BSTAT".to_string(),
        Op::SigmaBstg => "\u{03A3}BSTG".to_string(),
        Op::SigmaMmtug => "\u{03A3}MMTUG".to_string(),
        Op::SigmaMmtgd => "\u{03A3}MMTGD".to_string(),
        // ── Stat 1 Pac ANOVA Family ─────────────────────────────────────────────
        Op::SigmaAovone => "\u{03A3}AOVONE".to_string(),
        Op::SigmaAovtwo => "\u{03A3}AOVTWO".to_string(),
        Op::SigmaAnocov => "\u{03A3}ANOCOV".to_string(),
        // ── Stat 1 Pac Curve Fitting + Regression ───────────────────────────────
        Op::SigmaLin => "\u{03A3}LIN".to_string(),
        Op::SigmaExp => "\u{03A3}EXP".to_string(),
        Op::SigmaLogi => "\u{03A3}LOGI".to_string(),
        Op::SigmaPow => "\u{03A3}POW".to_string(),
        Op::SigmaMlrxy => "\u{03A3}MLRXY".to_string(),
        Op::SigmaMlrxyz => "\u{03A3}MLRXYZ".to_string(),
        Op::SigmaPolypWorkflow => "\u{03A3}POLYP".to_string(),
        Op::SigmaPolyc => "\u{03A3}POLYC".to_string(),
        // ── Stat 1 Pac Hypothesis Tests ─────────────────────────────────────────
        Op::SigmaPtst => "\u{03A3}PTST".to_string(),
        Op::SigmaTstat => "\u{03A3}TSTAT".to_string(),
        // ── Stat 1 Pac Nonparam / Chi-Sq Eval / Contingency ─────────────────────
        Op::SigmaXsqev => "\u{03A3}XSQEV".to_string(),
        Op::SigmaEfxsq => "\u{03A3}EFXSQ".to_string(),
        Op::SigmaCtkkk => "\u{03A3}CTKKK".to_string(),
        Op::SigmaCtkk => "\u{03A3}CTKK".to_string(),
        Op::SigmaSpear => "\u{03A3}SPEAR".to_string(),
        // ── Stat 1 Pac Distributions ────────────────────────────────────────────
        Op::SigmaNormdWorkflow => "\u{03A3}NORMD".to_string(),
        Op::SigmaChisqdWorkflow => "\u{03A3}CHISQD".to_string(),
        // ── Stat 1 Pac RNG ──────────────────────────────────────────────────────
        Op::Rand => "RAND".to_string(),
        Op::Seed => "SEED".to_string(),
        // ── Phase 38: Time Module (XROM 26) ───────────────────────────────────────
        Op::TimeAdate => "ADATE".to_string(),
        Op::TimeAlmcat => "ALMCAT".to_string(),
        Op::TimeAlmnow => "ALMNOW".to_string(),
        Op::TimeAtime => "ATIME".to_string(),
        Op::TimeAtime24 => "ATIME24".to_string(),
        Op::TimeClk12 => "CLK12".to_string(),
        Op::TimeClk24 => "CLK24".to_string(),
        Op::TimeClkt => "CLKT".to_string(),
        Op::TimeClktd => "CLKTD".to_string(),
        Op::TimeClock => "CLOCK".to_string(),
        Op::TimeCorrect => "CORRECT".to_string(),
        Op::TimeDate => "DATE".to_string(),
        Op::TimeDatePlus => "DATE+".to_string(),
        Op::TimeDdays => "DDAYS".to_string(),
        Op::TimeDmy => "DMY".to_string(),
        Op::TimeDow => "DOW".to_string(),
        Op::TimeMdy => "MDY".to_string(),
        Op::TimeRclaf => "RCLAF".to_string(),
        Op::TimeRclalm => "RCLALM".to_string(),
        Op::TimeRclsw => "RCLSW".to_string(),
        Op::TimeRunsw => "RUNSW".to_string(),
        Op::TimeSetaf => "SETAF".to_string(),
        Op::TimeSetdate => "SETDATE".to_string(),
        Op::TimeSetime => "SETIME".to_string(),
        Op::TimeSetsw => "SETSW".to_string(),
        Op::TimeStopsw => "STOPSW".to_string(),
        Op::TimeSw => "SW".to_string(),
        Op::TimeTplusx => "T+X".to_string(),
        Op::TimeTime => "TIME".to_string(),
        Op::TimeXyzalm => "XYZALM".to_string(),
        Op::TimeClalma => "CLALMA".to_string(),
        Op::TimeClalmx => "CLALMX".to_string(),
        Op::TimeClralms => "CLRALMS".to_string(),
        Op::TimeSwpt => "SWPT".to_string(),
        Op::TimeStpw => "STPW".to_string(),
        // ── Phase 43 (v3.3): Advantage Pac XROM 22 (ADV CONV + ADV MTRX) ──
        Op::AdvBinin => "BININ".to_string(),
        Op::AdvBinview => "BINVIEW".to_string(),
        Op::AdvOctin => "OCTIN".to_string(),
        Op::AdvHexin => "HEXIN".to_string(),
        Op::AdvHexview => "HEXVIEW".to_string(),
        Op::AdvCvtview => "CVTVIEW".to_string(),
        Op::AdvNot => "NOT".to_string(),
        Op::AdvAnd => "AND".to_string(),
        Op::AdvOr => "OR".to_string(),
        Op::AdvXor => "XOR".to_string(),
        Op::AdvRotxy => "ROTXY".to_string(),
        Op::AdvBitTest => "BIT?".to_string(),
        Op::AdvIPlus => "I+".to_string(),
        Op::AdvIMinus => "I-".to_string(),
        Op::AdvJPlus => "J+".to_string(),
        Op::AdvJMinus => "J-".to_string(),
        Op::AdvMr => "MR".to_string(),
        Op::AdvMs => "MS".to_string(),
        Op::AdvMrij => "MRIJ".to_string(),
        Op::AdvMsij => "MSIJ".to_string(),
        Op::AdvMsijr => "MSIJR".to_string(),
        Op::AdvMrcPlus => "MRC+".to_string(),
        Op::AdvMrcMinus => "MRC-".to_string(),
        Op::AdvMrrPlus => "MRR+".to_string(),
        Op::AdvMrrMinus => "MRR-".to_string(),
        Op::AdvMsrPlus => "MSR+".to_string(),
        Op::AdvMscPlus => "MSC+".to_string(),
        Op::AdvMswap => "MSWAP".to_string(),
        Op::AdvMnameQuery => "MNAME?".to_string(),
        Op::AdvDimQuery => "DIM?".to_string(),
        Op::AdvMatdim => "MATDIM".to_string(),
        Op::AdvMp => "MP".to_string(),
        Op::AdvPiv => "PIV".to_string(),
        Op::AdvRExchangeR => "R<>R".to_string(),
        Op::AdvRGtRQuery => "R>R?".to_string(),
        Op::AdvSum => "SUM".to_string(),
        Op::AdvSumab => "SUMAB".to_string(),
        Op::AdvMax => "MAX".to_string(),
        Op::AdvMaxab => "MAXAB".to_string(),
        Op::AdvMin => "MIN".to_string(),
        Op::AdvRmaxab => "RMAXAB".to_string(),
        Op::AdvRnrm => "RNRM".to_string(),
        Op::AdvRsum => "RSUM".to_string(),
        Op::AdvFnrm => "FNRM".to_string(),
        Op::AdvMdet => "MDET".to_string(),
        Op::AdvMinv => "MINV".to_string(),
        Op::AdvMsys => "MSYS".to_string(),
        Op::AdvMMulM => "M*M".to_string(),
        Op::AdvMatPlus => "MAT+".to_string(),
        Op::AdvMatMinus => "MAT-".to_string(),
        Op::AdvMatScalarMul => "MAT*C".to_string(),
        Op::AdvMatScalarDiv => "MAT/C".to_string(),
        Op::AdvTrnps => "TRNPS".to_string(),
        Op::AdvMmove => "MMOVE".to_string(),
        Op::AdvCExchangeC => "C<>C".to_string(),
        Op::AdvCmaxab => "CMAXAB".to_string(),
        Op::AdvCnrm => "CNRM".to_string(),
        Op::AdvCsum => "CSUM".to_string(),
        Op::AdvYcPlusC => "YC+C".to_string(),
        Op::AdvMatrx => "MATRX".to_string(),
        Op::AdvMtr => "MTR".to_string(),
        Op::AdvMedit => "MEDIT".to_string(),
        Op::AdvCmedit => "CMEDIT".to_string(),
        // ── Phase 43 (v3.3): Advantage Pac XROM 24 (ADV MATH + ADV TVM) ──
        Op::AdvExpZ => "E^Z".to_string(),
        Op::AdvLnZ => "LNZ".to_string(),
        Op::AdvLogZ => "LOGZ".to_string(),
        Op::AdvZPowN => "Z^N".to_string(),
        Op::AdvZPow1n => "Z^1/N".to_string(),
        Op::AdvZPowW => "Z^W".to_string(),
        Op::AdvZPow1w => "Z^1/W".to_string(),
        Op::AdvMagz => "|Z|".to_string(),
        Op::AdvSinZ => "SINZ".to_string(),
        Op::AdvCosZ => "COSZ".to_string(),
        Op::AdvTanZ => "TANZ".to_string(),
        Op::AdvAPowZ => "A^Z".to_string(),
        Op::AdvCPlus => "CADD".to_string(),
        Op::AdvCMinus => "CSUB".to_string(),
        Op::AdvCinv => "CINV".to_string(),
        Op::AdvCMul => "CMUL".to_string(),
        Op::AdvCDiv => "CDIV".to_string(),
        Op::AdvAip => "AIP".to_string(),
        Op::AdvPly => "PLY".to_string(),
        Op::AdvRts => "RTS".to_string(),
        Op::AdvFsolve => "FSOLVE".to_string(),
        Op::AdvFsolveRunLoop => "FSOLVE".to_string(),
        Op::AdvFintg => "FINTG".to_string(),
        Op::AdvFintgRunLoop => "FINTG".to_string(),
        Op::AdvFdifeq => "FDIFEQ".to_string(),
        Op::AdvFdifeqRunLoop => "FDIFEQ".to_string(),
        Op::AdvFroot => "FROOT".to_string(),
        Op::AdvCfit => "CFIT".to_string(),
        Op::AdvAs => "AS".to_string(),
        Op::AdvDs => "DS".to_string(),
        Op::AdvBfit => "BFIT".to_string(),
        Op::AdvFit => "FIT".to_string(),
        Op::AdvYQueryX => "Y?X".to_string(),
        Op::AdvSzQuery => "SZ?".to_string(),
        Op::AdvVPlus => "V+".to_string(),
        Op::AdvVMinus => "V-".to_string(),
        Op::AdvDot => "DOT".to_string(),
        Op::AdvCross => "CROSS".to_string(),
        Op::AdvVc => "VC".to_string(),
        Op::AdvVs => "VS".to_string(),
        Op::AdvVr => "VR".to_string(),
        Op::AdvVe => "VE".to_string(),
        Op::AdvVxy => "VXY".to_string(),
        Op::AdvUv => "UV".to_string(),
        Op::AdvVMag => "|V|".to_string(),
        Op::AdvVStar => "V*".to_string(),
        Op::AdvVd => "VD".to_string(),
        Op::AdvTr => "TR".to_string(),
        Op::AdvTvm => "TVM".to_string(),
        Op::AdvTvmN => "N".to_string(),
        Op::AdvTvmPv => "PV".to_string(),
        Op::AdvTvmPmt => "PMT".to_string(),
        Op::AdvTvmFv => "FV".to_string(),
        Op::AdvTvmStarI => "*I".to_string(),
        // ── Phase 51 (v4.0): X-MEM built-in ops ─────────────────────────────
        Op::EmDir => "EMDIR".to_string(),
        Op::EmRoom => "EMROOM".to_string(),
        Op::SaveP => "SAVEP".to_string(),
        Op::GetP => "GETP".to_string(),
        Op::SaveD => "SAVED".to_string(),
        Op::GetD => "GETD".to_string(),
        Op::EmReg => "EMREG".to_string(),
        Op::SaveRx => "SAVERX".to_string(),
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

    #[test]
    fn test_display_phase20_op_labels() {
        // Phase 20: byte-identical mnemonics with the hp41-cli copy (D-22, D-24).
        // The intentional duplication of `op_display_name` across hp41-cli and
        // hp41-gui/src-tauri is the documented SC-4 exception in CLAUDE.md.
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

    #[test]
    fn test_display_phase24_ind_op_labels() {
        // Phase 24: byte-identical mnemonics across hp41-cli and hp41-gui copies
        // (SC-4 mirror invariant). The 11 new Op::*Ind variants surface as
        // space-separated MNEMONIC IND nn (Phase-22 GTO IND precedent).
        assert_eq!(op_display_name(&Op::StoInd(5)), "STO IND 05");
        assert_eq!(op_display_name(&Op::RclInd(7)), "RCL IND 07");
        assert_eq!(op_display_name(&Op::IsgInd(5)), "ISG IND 05");
        assert_eq!(op_display_name(&Op::DseInd(5)), "DSE IND 05");
        assert_eq!(op_display_name(&Op::SfFlagInd(12)), "SF IND 12");
        assert_eq!(op_display_name(&Op::CfFlagInd(12)), "CF IND 12");
        assert_eq!(op_display_name(&Op::ArclInd(12)), "ARCL IND 12");
        assert_eq!(op_display_name(&Op::AstoInd(12)), "ASTO IND 12");
        assert_eq!(op_display_name(&Op::ViewInd(5)), "VIEW IND 05");
        // StoArithInd: all 4 StoArithKind sub-kinds
        assert_eq!(
            op_display_name(&Op::StoArithInd(12, StoArithKind::Add)),
            "STO+ IND 12"
        );
        assert_eq!(
            op_display_name(&Op::StoArithInd(12, StoArithKind::Sub)),
            "STO- IND 12"
        );
        assert_eq!(
            op_display_name(&Op::StoArithInd(12, StoArithKind::Mul)),
            "STO\u{00D7} IND 12"
        );
        assert_eq!(
            op_display_name(&Op::StoArithInd(12, StoArithKind::Div)),
            "STO\u{00F7} IND 12"
        );
        // FlagTestInd: all 4 FlagTestKind sub-kinds
        assert_eq!(
            op_display_name(&Op::FlagTestInd {
                kind: FlagTestKind::IsSet,
                ind_reg: 5
            }),
            "FS? IND 05"
        );
        assert_eq!(
            op_display_name(&Op::FlagTestInd {
                kind: FlagTestKind::IsClear,
                ind_reg: 5
            }),
            "FC? IND 05"
        );
        assert_eq!(
            op_display_name(&Op::FlagTestInd {
                kind: FlagTestKind::IsSetThenClear,
                ind_reg: 5
            }),
            "FS?C IND 05"
        );
        assert_eq!(
            op_display_name(&Op::FlagTestInd {
                kind: FlagTestKind::IsClearThenClear,
                ind_reg: 5
            }),
            "FC?C IND 05"
        );
    }
}
