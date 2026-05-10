//! Phase 14 IPC Layer — String key ID → Op resolver.
//!
//! Decisions: D-04..D-07 (key ID convention).
//! - Digit keys ("0"-"9", ".", "e") are NOT resolved here — they are special-cased in
//!   commands::dispatch_op and append directly to entry_buf (RESEARCH.md Pattern 6).
//! - Named ops use snake_case strings mirroring `hp41_cli::keys::key_to_op` semantically.
//! - Parameterized ops use compound key IDs: `"sto_NN"`, `"fix_N"`, `"sto_arith_<op>_<reg>"`.
//! - Unknown key IDs return `Err(GuiError)` (D-07: never silent discard).

use crate::types::GuiError;
use hp41_core::ops::{Op, StackReg};
use hp41_core::StoArithKind;

/// Resolve a string key ID to an Op. Returns `Err(GuiError)` for unknown IDs.
///
/// This function does NOT handle digit keys (0-9, `.`, `e`) — those are caught earlier
/// in `commands::dispatch_op` and append to `entry_buf` directly.
pub fn resolve(key_id: &str) -> Result<Op, GuiError> {
    match key_id {
        // ── Stack ────────────────────────────────────────────────────────────
        "enter" => Ok(Op::Enter),
        "clx" => Ok(Op::Clx),
        "chs" => Ok(Op::Chs),
        "rdn" => Ok(Op::Rdn),
        "xy_swap" => Ok(Op::XySwap),
        "lastx" => Ok(Op::Lastx),
        // ── Arithmetic ───────────────────────────────────────────────────────
        "plus" => Ok(Op::Add),
        "minus" => Ok(Op::Sub),
        "mul" => Ok(Op::Mul),
        "div" => Ok(Op::Div),
        // ── Unary math ───────────────────────────────────────────────────────
        "sqrt" => Ok(Op::Sqrt),
        "sq" => Ok(Op::Sq),
        "ypow" => Ok(Op::YPow),
        "recip" => Ok(Op::Recip),
        "ln" => Ok(Op::Ln),
        "log" => Ok(Op::Log),
        "exp" => Ok(Op::Exp),
        "tenpow" => Ok(Op::TenPow),
        "int" => Ok(Op::Int),
        // ── Trig ─────────────────────────────────────────────────────────────
        "sin" => Ok(Op::Sin),
        "cos" => Ok(Op::Cos),
        "tan" => Ok(Op::Tan),
        "asin" => Ok(Op::Asin),
        "acos" => Ok(Op::Acos),
        "atan" => Ok(Op::Atan),
        // ── Angle mode ───────────────────────────────────────────────────────
        "set_deg" => Ok(Op::SetDeg),
        "set_rad" => Ok(Op::SetRad),
        "set_grad" => Ok(Op::SetGrad),
        // ── Registers (named + hidden M/N/O) ─────────────────────────────────
        "clreg" => Ok(Op::Clreg),
        "sto_m" => Ok(Op::StoM),
        "sto_n" => Ok(Op::StoN),
        "sto_o" => Ok(Op::StoO),
        "rcl_m" => Ok(Op::RclM),
        "rcl_n" => Ok(Op::RclN),
        "rcl_o" => Ok(Op::RclO),
        // ── ALPHA ────────────────────────────────────────────────────────────
        "alpha_toggle" => Ok(Op::AlphaToggle),
        "alpha_clear" => Ok(Op::AlphaClear),
        "alpha_backspace" => Ok(Op::AlphaBackspace),
        // ── Programming ──────────────────────────────────────────────────────
        "prgm_mode" => Ok(Op::PrgmMode),
        "rtn" => Ok(Op::Rtn),
        "null" => Ok(Op::Null),
        "getkey" => Ok(Op::GetKey),
        // ── User mode ────────────────────────────────────────────────────────
        "user_mode" => Ok(Op::UserMode),
        // ── Stats ────────────────────────────────────────────────────────────
        "sigma_plus" => Ok(Op::SigmaPlus),
        "sigma_minus" => Ok(Op::SigmaMinus),
        "mean" => Ok(Op::Mean),
        "sdev" => Ok(Op::Sdev),
        "lr" => Ok(Op::LR),
        "yhat" => Ok(Op::Yhat),
        "corr" => Ok(Op::Corr),
        "cl_sigma_stat" => Ok(Op::ClSigmaStat),
        // ── HMS ──────────────────────────────────────────────────────────────
        "hms_to_h" => Ok(Op::HmsToH),
        "h_to_hms" => Ok(Op::HToHms),
        "hms_add" => Ok(Op::HmsAdd),
        "hms_sub" => Ok(Op::HmsSub),
        // ── Print ────────────────────────────────────────────────────────────
        "prx" => Ok(Op::PRX),
        "pra" => Ok(Op::PRA),
        "prstk" => Ok(Op::PRSTK),
        // ── Parameterized & unknown ──────────────────────────────────────────
        _ => resolve_parameterized(key_id),
    }
}

/// Parse compound key IDs of the form `<prefix>_<arg>`. Returns `Err(GuiError)` if no
/// known prefix matches or if the argument fails to parse.
fn resolve_parameterized(key_id: &str) -> Result<Op, GuiError> {
    // Single-prefix u8 args
    if let Some(rest) = key_id.strip_prefix("sto_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::StoReg(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("rcl_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::RclReg(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("fix_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::FmtFix(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("sci_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::FmtSci(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("eng_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::FmtEng(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("isg_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::Isg(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("dse_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::Dse(n));
        }
    }

    // sto_arith_<op>_<reg> — multi-segment, parse with rsplit_once to peel reg from right.
    // (Pitfall 3 in RESEARCH.md: split_once would split at first '_' and break.)
    if let Some(rest) = key_id.strip_prefix("sto_arith_") {
        return resolve_sto_arith(rest, key_id);
    }

    // Label-bearing ops — the rest is the label string itself.
    if let Some(rest) = key_id.strip_prefix("gto_") {
        return Ok(Op::Gto(rest.to_string()));
    }
    if let Some(rest) = key_id.strip_prefix("xeq_") {
        return Ok(Op::Xeq(rest.to_string()));
    }
    if let Some(rest) = key_id.strip_prefix("lbl_") {
        return Ok(Op::Lbl(rest.to_string()));
    }

    // Single-character alpha append: "alpha_X" where X is exactly one char.
    if let Some(rest) = key_id.strip_prefix("alpha_") {
        let mut chars = rest.chars();
        if let (Some(ch), None) = (chars.next(), chars.next()) {
            return Ok(Op::AlphaAppend(ch));
        }
    }

    Err(GuiError {
        message: format!("unknown key: {key_id}"),
    })
}

/// Parse the body of `sto_arith_*` keys. `rest` is the substring after the
/// `"sto_arith_"` prefix; `original` is the full key_id (for error messages).
fn resolve_sto_arith(rest: &str, original: &str) -> Result<Op, GuiError> {
    let (kind_str, reg_str) = rest
        .rsplit_once('_')
        .ok_or_else(|| GuiError { message: format!("unknown key: {original}") })?;

    let kind = match kind_str {
        "plus" => StoArithKind::Add,
        "minus" => StoArithKind::Sub,
        "mul" => StoArithKind::Mul,
        "div" => StoArithKind::Div,
        _ => return Err(GuiError { message: format!("unknown key: {original}") }),
    };

    if let Ok(n) = reg_str.parse::<u8>() {
        return Ok(Op::StoArith { reg: n, kind });
    }

    let stack_reg = match reg_str {
        "y" => StackReg::Y,
        "z" => StackReg::Z,
        "t" => StackReg::T,
        "lastx" => StackReg::LastX,
        _ => return Err(GuiError { message: format!("unknown key: {original}") }),
    };

    Ok(Op::StoArithStack { kind, stack_reg })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use hp41_core::ops::StackReg;
    use hp41_core::StoArithKind;

    #[test]
    fn test_key_map_named_ops() {
        assert_eq!(resolve("plus").unwrap(), Op::Add);
        assert_eq!(resolve("minus").unwrap(), Op::Sub);
        assert_eq!(resolve("mul").unwrap(), Op::Mul);
        assert_eq!(resolve("div").unwrap(), Op::Div);
        assert_eq!(resolve("enter").unwrap(), Op::Enter);
        assert_eq!(resolve("sin").unwrap(), Op::Sin);
        assert_eq!(resolve("clreg").unwrap(), Op::Clreg);
        assert_eq!(resolve("prx").unwrap(), Op::PRX);
    }

    #[test]
    fn test_key_map_unknown_key() {
        // SC-2: unknown key returns GuiError (NOT a panic, NOT silent).
        let err = resolve("totally_unknown_xyz").unwrap_err();
        assert!(
            err.message.contains("unknown key"),
            "expected message to mention 'unknown key', got: {}",
            err.message
        );
        assert!(
            err.message.contains("totally_unknown_xyz"),
            "expected message to include the offending key id, got: {}",
            err.message
        );
    }

    #[test]
    fn test_key_map_compound_keys() {
        assert_eq!(resolve("sto_05").unwrap(), Op::StoReg(5));
        assert_eq!(resolve("rcl_12").unwrap(), Op::RclReg(12));
        assert_eq!(resolve("fix_4").unwrap(), Op::FmtFix(4));
        assert_eq!(resolve("sci_2").unwrap(), Op::FmtSci(2));
        assert_eq!(resolve("eng_3").unwrap(), Op::FmtEng(3));
        assert_eq!(
            resolve("sto_arith_plus_05").unwrap(),
            Op::StoArith { reg: 5, kind: StoArithKind::Add }
        );
        assert_eq!(
            resolve("sto_arith_minus_y").unwrap(),
            Op::StoArithStack { kind: StoArithKind::Sub, stack_reg: StackReg::Y }
        );
    }

    #[test]
    fn test_all_keyboard_skin_ids_are_valid() {
        // IDs from KEY_DEFS that are routed through key_map::resolve() (named ops only).
        // Digit keys ("0"-"9", ".", "e") are handled by handle_op() digit branch — not here.
        // Empty-string ids are visual-only and are never sent to dispatch_op.
        let named_ids = [
            "sigma_plus", "recip", "sqrt", "log", "ln",
            "sin", "cos", "tan", "rdn", "xy_swap",
            "enter", "div", "mul",
            "user_mode", "minus", "prgm_mode", "alpha_toggle",
            "chs", "plus",
            "lastx", "clreg", "clx",
        ];
        for id in named_ids {
            assert!(
                resolve(id).is_ok(),
                "key_map::resolve({id:?}) must succeed for a KEY_DEFS named id"
            );
        }
    }
}
