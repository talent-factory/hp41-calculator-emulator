//! Phase 14 IPC Layer — String key ID → Op resolver.
//!
//! Decisions: D-04..D-07 (key ID convention).
//! - Digit keys ("0"-"9", ".", "e") are NOT resolved here — they are special-cased in
//!   commands::dispatch_op and append directly to entry_buf (RESEARCH.md Pattern 6).
//! - Named ops use snake_case strings mirroring `hp41_cli::keys::key_to_op` semantically.
//! - Parameterized ops use compound key IDs: `"sto_NN"`, `"fix_N"`, `"sto_arith_<op>_<reg>"`.
//! - Unknown key IDs return `Err(GuiError)` (D-07: never silent discard).
//!
//! Phase 26 D-26.5: every HP-41CV ROM op variant added in Phases 20-24 resolves
//! through `resolve` (bare ops) or `resolve_parameterized` (compound ids). The
//! `*_prompt` ids + `asn`/`view`/`catalog`/`xeq_prompt`/`gto_prompt`/`lbl_prompt`
//! stay in the stub-error arm as defense-in-depth — the frontend intercepts them
//! in `handleClick` before they reach `dispatch_op`. A frontend regression that
//! skips the intercept surfaces a `GuiError` toast, never silent.

use crate::types::GuiError;
use hp41_core::ops::{FlagTestKind, Op, StackReg, TestKind};
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
        "r_up" => Ok(Op::Rup),
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
        "pct_change" => Ok(Op::PctChange),
        "recip" => Ok(Op::Recip),
        "ln" => Ok(Op::Ln),
        "log" => Ok(Op::Log),
        "exp" => Ok(Op::Exp),
        "tenpow" => Ok(Op::TenPow),
        "int" => Ok(Op::Int),
        // ── Phase 20: extended unary math + π + polar conversions ─────────────
        "pi" => Ok(Op::Pi),
        "rnd" => Ok(Op::Rnd),
        "frc" => Ok(Op::Frc),
        "abs" => Ok(Op::Abs),
        "sign" => Ok(Op::Sign),
        "fact" => Ok(Op::Fact),
        "mod_op" => Ok(Op::Mod),
        "polar_to_rect" => Ok(Op::PolarToRect),
        "rect_to_polar" => Ok(Op::RectToPolar),
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
        // ── Phase 23: ALPHA-register operations ──────────────────────────────
        "atox" => Ok(Op::Atox),
        "xtoa" => Ok(Op::Xtoa),
        "arot" => Ok(Op::Arot),
        "posa" => Ok(Op::Posa),
        // ── Programming ──────────────────────────────────────────────────────
        "prgm_mode" => Ok(Op::PrgmMode),
        "rtn" => Ok(Op::Rtn),
        "null" => Ok(Op::Null),
        "getkey" => Ok(Op::GetKey),
        // ── Phase 22: Program control ────────────────────────────────────────
        "stop" => Ok(Op::Stop),
        "pse" => Ok(Op::Pse),
        "ins" => Ok(Op::Ins),
        // ── Phase 22: Memory ─────────────────────────────────────────────────
        "cla" => Ok(Op::Cla),
        "clst" => Ok(Op::Clst),
        "pack" => Ok(Op::Pack),
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
        // ── Comparison (Test variants — keyboard-accessible without label arg) ───
        // Phase 19 v2.1 wired only `xge_y`. Phase 25 D-25.7 added the 4 keyboard-
        // bound conditional tests (the only conditionals on the physical keyboard);
        // the other 8 conditionals route through XEQ-by-Name in Phase 25 Plan 03's
        // `builtin_card_op` extension. Phase 26 surfaces the 4 GUI-bound ids here.
        "xge_y" => Ok(Op::Test(TestKind::XGeY)),
        "x_eq_y" => Ok(Op::Test(TestKind::XEqY)),
        "x_le_y" => Ok(Op::Test(TestKind::XLeY)),
        "x_gt_y" => Ok(Op::Test(TestKind::XGtY)),
        "x_eq_0" => Ok(Op::Test(TestKind::XEqZero)),
        // ── Phase 21: Display control + sound ────────────────────────────────
        "aview" => Ok(Op::AView),
        "prompt" => Ok(Op::Prompt),
        "aon" => Ok(Op::Aon),
        "aoff" => Ok(Op::Aoff),
        "cld" => Ok(Op::Cld),
        "beep" => Ok(Op::Beep),
        // ── Print ────────────────────────────────────────────────────────────
        "prx" => Ok(Op::PRX),
        "pra" => Ok(Op::PRA),
        "prstk" => Ok(Op::PRSTK),
        // ── Stub-error arm: ids that are clickable in the skin but require ──
        // ── frontend modal handling (Phase 26 D-26.5 — defense-in-depth) ──
        // The 13 `*_prompt` ids + `asn`/`view`/`catalog`/`xeq_prompt`/`gto_prompt`/
        // `lbl_prompt` are intercepted by `App.tsx::handleClick` BEFORE they
        // reach `dispatch_op`. They stay here so a frontend regression that
        // skips the intercept surfaces as a `GuiError` toast — never silent
        // (D-07 invariant). The Phase-25-shipped bare ROM ops (pi, polar_to_rect,
        // rect_to_polar, beep) MOVED to real `Ok(Op::*)` arms above.
        //
        // The three label-bearing prompt ids (xeq_prompt, gto_prompt, lbl_prompt)
        // MUST hit this arm BEFORE `resolve_parameterized` strips their prefixes
        // — otherwise the parameterized fallthrough would silently dispatch
        // `Op::Xeq("prompt")` / `Op::Gto("prompt")` / `Op::Lbl("prompt")`.
        "asn" | "catalog" | "view"
        | "xeq_prompt" | "gto_prompt" | "lbl_prompt"
        | "sto_prompt" | "rcl_prompt" | "isg_prompt"
        | "sf_prompt" | "cf_prompt" | "fs_prompt"
        | "fix_prompt" | "sci_prompt" | "eng_prompt"
        | "x_eq_y_prompt" | "x_le_y_prompt" | "x_gt_y_prompt" | "x_eq_0_prompt"
        | "tone" => Err(GuiError {
            message: format!("'{key_id}' is planned for a future phase"),
        }),
        // ── Parameterized & unknown ──────────────────────────────────────────
        _ => resolve_parameterized(key_id),
    }
}

/// Parse compound key IDs of the form `<prefix>_<arg>`. Returns `Err(GuiError)` if no
/// known prefix matches or if the argument fails to parse.
///
/// Phase 26: more-specific-first ordering — `strip_prefix("sto_ind_")` MUST appear
/// BEFORE `strip_prefix("sto_")`, otherwise the more-general arm wins and
/// `sto_ind_05` resolves as `Op::StoReg(<parse fails: "ind_05">)`. Same for
/// every IND-bearing prefix (rcl/isg/dse/sf/cf/fs/view/arcl/asto/sto_arith).
fn resolve_parameterized(key_id: &str) -> Result<Op, GuiError> {
    // ── IND-bearing single-prefix u8 args (MUST come before non-IND variants) ──
    if let Some(rest) = key_id.strip_prefix("sto_ind_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::StoInd(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("rcl_ind_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::RclInd(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("isg_ind_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::IsgInd(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("dse_ind_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::DseInd(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("sf_ind_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::SfFlagInd(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("cf_ind_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::CfFlagInd(n));
        }
    }
    // fs_c_ind_NN / fc_c_ind_NN — FlagTestInd with IsSetThenClear / IsClearThenClear.
    // MUST come before fs_ind_ / fc_ind_ (longer prefix wins).
    if let Some(rest) = key_id.strip_prefix("fs_c_ind_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::FlagTestInd {
                kind: FlagTestKind::IsSetThenClear,
                ind_reg: n,
            });
        }
    }
    if let Some(rest) = key_id.strip_prefix("fc_c_ind_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::FlagTestInd {
                kind: FlagTestKind::IsClearThenClear,
                ind_reg: n,
            });
        }
    }
    if let Some(rest) = key_id.strip_prefix("fs_ind_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::FlagTestInd {
                kind: FlagTestKind::IsSet,
                ind_reg: n,
            });
        }
    }
    if let Some(rest) = key_id.strip_prefix("fc_ind_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::FlagTestInd {
                kind: FlagTestKind::IsClear,
                ind_reg: n,
            });
        }
    }
    if let Some(rest) = key_id.strip_prefix("arcl_ind_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::ArclInd(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("asto_ind_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::AstoInd(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("view_ind_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::ViewInd(n));
        }
    }

    // ── sto_arith_<op>_ind_<reg> — multi-segment, IND variant (must come before sto_arith_) ──
    if let Some(rest) = key_id.strip_prefix("sto_arith_") {
        return resolve_sto_arith(rest, key_id);
    }

    // ── Non-IND single-prefix u8 args ─────────────────────────────────────────
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
    // Phase 21 flag prefixes — non-IND counterparts to the IND group above.
    if let Some(rest) = key_id.strip_prefix("sf_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::SfFlag(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("cf_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::CfFlag(n));
        }
    }
    // fs_c_NN / fc_c_NN — must come before fs_ / fc_ (longer prefix wins).
    if let Some(rest) = key_id.strip_prefix("fs_c_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::FlagTest {
                kind: FlagTestKind::IsSetThenClear,
                flag: n,
            });
        }
    }
    if let Some(rest) = key_id.strip_prefix("fc_c_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::FlagTest {
                kind: FlagTestKind::IsClearThenClear,
                flag: n,
            });
        }
    }
    if let Some(rest) = key_id.strip_prefix("fs_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::FlagTest {
                kind: FlagTestKind::IsSet,
                flag: n,
            });
        }
    }
    if let Some(rest) = key_id.strip_prefix("fc_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::FlagTest {
                kind: FlagTestKind::IsClear,
                flag: n,
            });
        }
    }
    // Phase 21 view (register) + Phase 23 arcl/asto.
    if let Some(rest) = key_id.strip_prefix("view_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::View(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("arcl_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::Arcl(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("asto_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::Asto(n));
        }
    }
    // Phase 21 tone (single 0..=9).
    if let Some(rest) = key_id.strip_prefix("tone_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::Tone(n));
        }
    }
    // Phase 22 catalog (1..=4 per HP-41CV; resolver accepts any u8, dispatch
    // validates the range and returns InvalidOp on n==0 or n>=5).
    if let Some(rest) = key_id.strip_prefix("catalog_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::Catalog(n));
        }
    }
    // Phase 22 size (u16 — HP-41CV accepts 0..=319).
    if let Some(rest) = key_id.strip_prefix("size_") {
        if let Ok(n) = rest.parse::<u16>() {
            return Ok(Op::Size(n));
        }
    }
    // Phase 22 DEL nnn — Phase 26 BLOCKER B3: hp41-core's Op::Del field is u8
    // (line 397 of hp41-core/src/ops/mod.rs). HP-41 hardware natively accepts
    // 000-999; the u8 cap is a documented v2.2 divergence (widening deferred
    // to v3.x per the phase boundary). Values 256..=999 must NOT silently
    // truncate — return an explicit GuiError that the frontend surfaces as
    // a "DEL ERR" toast. The frontend modal preview ALSO renders "DEL ERR"
    // before dispatch so the user sees the divergence before committing.
    if let Some(rest) = key_id.strip_prefix("del_") {
        if let Ok(n) = rest.parse::<u16>() {
            if n > 255 {
                return Err(GuiError {
                    message: format!(
                        "DEL value must be 0-255 (hp41-core Op::Del field is u8 — \
                         Phase 26 divergence from HP-41 hardware 0-999, deferred to v3.x). \
                         Got: {n}"
                    ),
                });
            }
            return Ok(Op::Del(n as u8));
        }
    }

    // Phase 22 GTO IND nn / XEQ IND nn — must come before the more-general
    // gto_/xeq_ label-bearing prefixes below.
    if let Some(rest) = key_id.strip_prefix("gto_ind_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::GtoInd(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("xeq_ind_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::XeqInd(n));
        }
    }

    // Phase 22 ASN — `asn_<KEY_CODE>_<NAME>` where KEY_CODE is u8 (row×10+col).
    // The label may contain underscores (HP-41 labels allow alphanumeric +
    // underscore-equivalents). Parse strategy: split at the FIRST underscore
    // after the numeric prefix; the remainder is the label verbatim.
    if let Some(rest) = key_id.strip_prefix("asn_") {
        return resolve_asn(rest, key_id);
    }

    // Label-bearing ops — the rest is the label string itself.
    // Phase 22 CLP "name" — sits BEFORE generic gto_/xeq_/lbl_ to honor the
    // hardware-faithful order.
    if let Some(rest) = key_id.strip_prefix("clp_") {
        return Ok(Op::Clp(rest.to_string()));
    }
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
///
/// Phase 26 extension: accepts both direct (`<op>_<reg>`) and IND
/// (`<op>_ind_<reg>`) forms. The IND form returns `Op::StoArithInd(reg, kind)`;
/// the direct form returns `Op::StoArith { reg, kind }` (numbered reg) or
/// `Op::StoArithStack { kind, stack_reg }` (Y/Z/T/LASTX target).
fn resolve_sto_arith(rest: &str, original: &str) -> Result<Op, GuiError> {
    // IND form: `<op>_ind_<reg>` (e.g. "plus_ind_05").
    if let Some(after_ind) = rest.strip_suffix_ind() {
        let (kind_str, reg_str) = after_ind;
        let kind = parse_sto_arith_kind(kind_str, original)?;
        if let Ok(n) = reg_str.parse::<u8>() {
            return Ok(Op::StoArithInd(n, kind));
        }
        return Err(GuiError {
            message: format!("unknown key: {original}"),
        });
    }

    // Direct form: `<op>_<reg>` (e.g. "plus_05" or "minus_y").
    let (kind_str, reg_str) = rest.rsplit_once('_').ok_or_else(|| GuiError {
        message: format!("unknown key: {original}"),
    })?;
    let kind = parse_sto_arith_kind(kind_str, original)?;

    if let Ok(n) = reg_str.parse::<u8>() {
        return Ok(Op::StoArith { reg: n, kind });
    }

    let stack_reg = match reg_str {
        "y" => StackReg::Y,
        "z" => StackReg::Z,
        "t" => StackReg::T,
        "lastx" => StackReg::LastX,
        _ => {
            return Err(GuiError {
                message: format!("unknown key: {original}"),
            })
        }
    };

    Ok(Op::StoArithStack { kind, stack_reg })
}

/// Helper trait to extract `(kind, reg)` from a `sto_arith_*_ind_NN` body.
/// Returns `Some((kind_str, reg_str))` if the body matches the IND pattern,
/// `None` otherwise.
trait StripSuffixInd {
    fn strip_suffix_ind(&self) -> Option<(&str, &str)>;
}

impl StripSuffixInd for str {
    fn strip_suffix_ind(&self) -> Option<(&str, &str)> {
        // Match `<kind>_ind_<reg>` where `<kind>` is a known sto-arith op name
        // and `<reg>` is the trailing numeric segment. The kind may itself contain
        // underscores in principle (none today, but defensively split at the
        // first `_ind_` substring).
        let (kind, rest) = self.split_once("_ind_")?;
        Some((kind, rest))
    }
}

fn parse_sto_arith_kind(kind_str: &str, original: &str) -> Result<StoArithKind, GuiError> {
    match kind_str {
        "plus" => Ok(StoArithKind::Add),
        "minus" => Ok(StoArithKind::Sub),
        "mul" => Ok(StoArithKind::Mul),
        "div" => Ok(StoArithKind::Div),
        _ => Err(GuiError {
            message: format!("unknown key: {original}"),
        }),
    }
}

/// Parse the body of `asn_*` keys: `<KEY_CODE>_<NAME>`. KEY_CODE is u8
/// (HP-41 row×10+col). NAME is the verbatim label (may contain underscores).
///
/// Example: `asn_22_TEST` → `Op::Asn { name: "TEST", key_code: 22 }`.
fn resolve_asn(rest: &str, original: &str) -> Result<Op, GuiError> {
    let (code_str, name) = rest.split_once('_').ok_or_else(|| GuiError {
        message: format!("unknown key: {original}"),
    })?;
    let key_code: u8 = code_str.parse().map_err(|_| GuiError {
        message: format!("unknown key: {original}"),
    })?;
    Ok(Op::Asn {
        name: name.to_string(),
        key_code,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use hp41_core::ops::{FlagTestKind, StackReg, TestKind};
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
            Op::StoArith {
                reg: 5,
                kind: StoArithKind::Add
            }
        );
        assert_eq!(
            resolve("sto_arith_minus_y").unwrap(),
            Op::StoArithStack {
                kind: StoArithKind::Sub,
                stack_reg: StackReg::Y
            }
        );
    }

    #[test]
    fn resolve_pct_change_id() {
        assert_eq!(resolve("pct_change").unwrap(), Op::PctChange);
    }

    #[test]
    fn test_new_named_op_resolvers() {
        // v2.1-shipped named ops (regression coverage).
        assert_eq!(resolve("sq").unwrap(), Op::Sq);
        assert_eq!(resolve("ypow").unwrap(), Op::YPow);
        assert_eq!(resolve("tenpow").unwrap(), Op::TenPow);
        assert_eq!(resolve("exp").unwrap(), Op::Exp);
        assert_eq!(resolve("xge_y").unwrap(), Op::Test(TestKind::XGeY));
    }

    /// Phase 26 — every bare HP-41CV ROM op that has a keyboard-reachable id
    /// must resolve directly (no stub, no panic).
    #[test]
    fn test_new_v22_named_op_resolvers() {
        // Phase 20: math + stack
        assert_eq!(resolve("pi").unwrap(), Op::Pi);
        assert_eq!(resolve("polar_to_rect").unwrap(), Op::PolarToRect);
        assert_eq!(resolve("rect_to_polar").unwrap(), Op::RectToPolar);
        assert_eq!(resolve("rnd").unwrap(), Op::Rnd);
        assert_eq!(resolve("frc").unwrap(), Op::Frc);
        assert_eq!(resolve("mod_op").unwrap(), Op::Mod);
        assert_eq!(resolve("abs").unwrap(), Op::Abs);
        assert_eq!(resolve("fact").unwrap(), Op::Fact);
        assert_eq!(resolve("sign").unwrap(), Op::Sign);
        assert_eq!(resolve("r_up").unwrap(), Op::Rup);

        // Phase 21: display + sound
        assert_eq!(resolve("aview").unwrap(), Op::AView);
        assert_eq!(resolve("prompt").unwrap(), Op::Prompt);
        assert_eq!(resolve("aon").unwrap(), Op::Aon);
        assert_eq!(resolve("aoff").unwrap(), Op::Aoff);
        assert_eq!(resolve("cld").unwrap(), Op::Cld);
        assert_eq!(resolve("beep").unwrap(), Op::Beep);

        // Phase 22: program control + memory
        assert_eq!(resolve("stop").unwrap(), Op::Stop);
        assert_eq!(resolve("pse").unwrap(), Op::Pse);
        assert_eq!(resolve("ins").unwrap(), Op::Ins);
        assert_eq!(resolve("cla").unwrap(), Op::Cla);
        assert_eq!(resolve("clst").unwrap(), Op::Clst);
        assert_eq!(resolve("pack").unwrap(), Op::Pack);

        // Phase 23: ALPHA-register operations
        assert_eq!(resolve("atox").unwrap(), Op::Atox);
        assert_eq!(resolve("xtoa").unwrap(), Op::Xtoa);
        assert_eq!(resolve("arot").unwrap(), Op::Arot);
        assert_eq!(resolve("posa").unwrap(), Op::Posa);

        // Phase 25 D-25.7: the 4 keyboard-bound conditional tests (the other 8
        // are XEQ-by-Name-only per the same decision).
        assert_eq!(resolve("x_eq_y").unwrap(), Op::Test(TestKind::XEqY));
        assert_eq!(resolve("x_le_y").unwrap(), Op::Test(TestKind::XLeY));
        assert_eq!(resolve("x_gt_y").unwrap(), Op::Test(TestKind::XGtY));
        assert_eq!(resolve("x_eq_0").unwrap(), Op::Test(TestKind::XEqZero));
    }

    /// Phase 26 — every parameterized prefix added by the v2.2 ROM ops must
    /// parse correctly, with more-specific (IND) prefixes winning over the
    /// less-specific direct forms.
    #[test]
    fn test_new_v22_parameterized_prefixes() {
        // Phase 24: indirect register ops
        assert_eq!(resolve("sto_ind_05").unwrap(), Op::StoInd(5));
        assert_eq!(resolve("rcl_ind_12").unwrap(), Op::RclInd(12));
        assert_eq!(resolve("isg_ind_07").unwrap(), Op::IsgInd(7));
        assert_eq!(resolve("dse_ind_99").unwrap(), Op::DseInd(99));

        // Phase 21 flag prefixes (direct + IND, all 4 FlagTestKind variants).
        assert_eq!(resolve("sf_12").unwrap(), Op::SfFlag(12));
        assert_eq!(resolve("sf_ind_12").unwrap(), Op::SfFlagInd(12));
        assert_eq!(resolve("cf_03").unwrap(), Op::CfFlag(3));
        assert_eq!(resolve("cf_ind_03").unwrap(), Op::CfFlagInd(3));
        assert_eq!(
            resolve("fs_05").unwrap(),
            Op::FlagTest {
                kind: FlagTestKind::IsSet,
                flag: 5
            }
        );
        assert_eq!(
            resolve("fc_05").unwrap(),
            Op::FlagTest {
                kind: FlagTestKind::IsClear,
                flag: 5
            }
        );
        assert_eq!(
            resolve("fs_c_05").unwrap(),
            Op::FlagTest {
                kind: FlagTestKind::IsSetThenClear,
                flag: 5
            }
        );
        assert_eq!(
            resolve("fc_c_05").unwrap(),
            Op::FlagTest {
                kind: FlagTestKind::IsClearThenClear,
                flag: 5
            }
        );
        // IND variants of all 4 FlagTestKind.
        assert_eq!(
            resolve("fs_ind_05").unwrap(),
            Op::FlagTestInd {
                kind: FlagTestKind::IsSet,
                ind_reg: 5
            }
        );
        assert_eq!(
            resolve("fc_ind_05").unwrap(),
            Op::FlagTestInd {
                kind: FlagTestKind::IsClear,
                ind_reg: 5
            }
        );
        assert_eq!(
            resolve("fs_c_ind_05").unwrap(),
            Op::FlagTestInd {
                kind: FlagTestKind::IsSetThenClear,
                ind_reg: 5
            }
        );
        assert_eq!(
            resolve("fc_c_ind_05").unwrap(),
            Op::FlagTestInd {
                kind: FlagTestKind::IsClearThenClear,
                ind_reg: 5
            }
        );

        // Phase 21 view + Phase 23 arcl/asto (direct + IND).
        assert_eq!(resolve("view_05").unwrap(), Op::View(5));
        assert_eq!(resolve("view_ind_05").unwrap(), Op::ViewInd(5));
        assert_eq!(resolve("arcl_05").unwrap(), Op::Arcl(5));
        assert_eq!(resolve("arcl_ind_05").unwrap(), Op::ArclInd(5));
        assert_eq!(resolve("asto_05").unwrap(), Op::Asto(5));
        assert_eq!(resolve("asto_ind_05").unwrap(), Op::AstoInd(5));

        // Phase 21 sound — tone_N.
        assert_eq!(resolve("tone_5").unwrap(), Op::Tone(5));
        assert_eq!(resolve("tone_0").unwrap(), Op::Tone(0));

        // Phase 22 catalog + size.
        assert_eq!(resolve("catalog_2").unwrap(), Op::Catalog(2));
        assert_eq!(resolve("size_64").unwrap(), Op::Size(64));

        // Phase 22 GTO IND / XEQ IND.
        assert_eq!(resolve("gto_ind_05").unwrap(), Op::GtoInd(5));
        assert_eq!(resolve("xeq_ind_05").unwrap(), Op::XeqInd(5));

        // Phase 22 CLP and ASN (label-bearing).
        assert_eq!(resolve("clp_MYPRG").unwrap(), Op::Clp("MYPRG".to_string()));
        assert_eq!(
            resolve("asn_22_TEST").unwrap(),
            Op::Asn {
                name: "TEST".to_string(),
                key_code: 22
            }
        );

        // Phase 24 STO-arith IND.
        assert_eq!(
            resolve("sto_arith_plus_ind_07").unwrap(),
            Op::StoArithInd(7, StoArithKind::Add)
        );
        assert_eq!(
            resolve("sto_arith_div_ind_99").unwrap(),
            Op::StoArithInd(99, StoArithKind::Div)
        );
    }

    /// Phase 26 BLOCKER B3 — DEL clamps at u8 max. Values 256..=999 produce
    /// an explicit GuiError surfacing the v2.2 divergence; values 0..=255
    /// resolve cleanly to `Op::Del(n as u8)`.
    #[test]
    fn test_del_clamps_at_u8_max() {
        assert_eq!(resolve("del_0").unwrap(), Op::Del(0));
        assert_eq!(resolve("del_010").unwrap(), Op::Del(10));
        assert_eq!(resolve("del_255").unwrap(), Op::Del(255));

        let err = resolve("del_256").unwrap_err();
        assert!(
            err.message.contains("0-255"),
            "expected '0-255' in clamp error, got: {}",
            err.message
        );
        let err = resolve("del_999").unwrap_err();
        assert!(
            err.message.contains("0-255"),
            "expected '0-255' in clamp error, got: {}",
            err.message
        );
    }

    /// Phase 26 — verify more-specific-first ordering for IND prefixes. A
    /// regression that swapped the order would resolve `sto_ind_05` as
    /// `Op::StoReg(<failed-parse-of-'ind_05'>)` → wrong arm wins → silent
    /// data corruption.
    #[test]
    fn test_more_specific_prefix_wins() {
        // sto_ind_NN must resolve to StoInd, NOT to a fallthrough or to a
        // mis-parsed StoReg.
        assert_eq!(resolve("sto_ind_05").unwrap(), Op::StoInd(5));
        // Same for rcl / isg / dse.
        assert_eq!(resolve("rcl_ind_05").unwrap(), Op::RclInd(5));
        assert_eq!(resolve("isg_ind_05").unwrap(), Op::IsgInd(5));
        assert_eq!(resolve("dse_ind_05").unwrap(), Op::DseInd(5));
        // sf_c_ind_NN must beat sf_ind_NN / sf_c_NN / sf_NN.
        assert_eq!(
            resolve("fs_c_ind_05").unwrap(),
            Op::FlagTestInd {
                kind: FlagTestKind::IsSetThenClear,
                ind_reg: 5
            }
        );
        // sto_arith_<op>_ind_NN must beat sto_arith_<op>_NN.
        assert_eq!(
            resolve("sto_arith_plus_ind_07").unwrap(),
            Op::StoArithInd(7, StoArithKind::Add)
        );
    }

    #[test]
    fn test_stub_error_for_v22_backlog_ops() {
        // These ids resolve to an explicit GuiError, not Ok(Op). A regression
        // that returns Ok(...) here would silently discard the user click.
        // Phase 26: `pi`, `polar_to_rect`, `rect_to_polar`, `beep` moved to
        // real `Ok(Op::*)` arms; only `asn`/`catalog`/`view` remain as bare
        // ids in the stub (they are modal-openers per D-26.5).
        let stub_ids = ["asn", "catalog", "view"];
        for id in stub_ids {
            let err = resolve(id).unwrap_err();
            assert!(
                err.message.contains("planned for a future phase"),
                "id {id:?} expected stub message, got: {}",
                err.message
            );
            assert!(
                err.message.contains(id),
                "id {id:?} expected message to contain id, got: {}",
                err.message
            );
        }
    }

    // Audit (Phase 26 W1 revision): the full FlagTestKind family is {IsSet,
    // IsClear, IsSetThenClear, IsClearThenClear} (4 variants). Only SF, CF,
    // and FS? have keyboard bindings (Keyboard.tsx row 5 cols 1-3 shifted ids:
    // `sf_prompt`, `cf_prompt`, `fs_prompt`). The other 3 FlagTest variants
    // (FC?, FS?C, FC?C) are XEQ-by-Name reachable only (Phase 25 D-25.7 /
    // D-25.9 pattern). No new *_prompt ids to add to this test.
    #[test]
    fn test_modal_prompt_ids_are_stubs_for_now() {
        // Modal-opener prompts stub until modal infrastructure lands.
        // Frontend MUST NOT send these to dispatch_op (App.tsx routes them
        // to in-progress modals or shows a not-yet-implemented toast). The
        // backend stub is defence-in-depth.
        let prompt_ids = [
            "sto_prompt",
            "rcl_prompt",
            "xeq_prompt",
            "gto_prompt",
            "lbl_prompt",
            "isg_prompt",
            "fix_prompt",
            "sci_prompt",
            "eng_prompt",
            "sf_prompt",
            "cf_prompt",
            "fs_prompt",
            "x_eq_y_prompt",
            "x_le_y_prompt",
            "x_gt_y_prompt",
            "x_eq_0_prompt",
        ];
        for id in prompt_ids {
            assert!(
                resolve(id).is_err(),
                "prompt id {id:?} must not resolve to an Op without its modal"
            );
        }
    }

    /// The three label-bearing prompt ids (xeq_prompt, gto_prompt, lbl_prompt)
    /// have an explicit stub arm in `resolve` that MUST sit before the
    /// `resolve_parameterized` fallthrough — otherwise they would be silently
    /// dispatched as `Op::Xeq("prompt")` / `Op::Gto("prompt")` / `Op::Lbl("prompt")`.
    /// Pinning the exact stub diagnostic locks the arm ordering: a regression
    /// that moves the stub below the prefix routes would fail this assertion
    /// rather than passing the looser `is_err()` check above (since the
    /// fallthrough also returns Err, just with a different message).
    #[test]
    fn test_label_bearing_prompts_hit_explicit_stub_arm() {
        for id in ["xeq_prompt", "gto_prompt", "lbl_prompt"] {
            let err = resolve(id).unwrap_err();
            assert!(
                err.message.contains("planned for a future phase"),
                "id {id:?} must hit the stub arm (which produces \
                 'planned for a future phase'), not the parameterized \
                 fallthrough. Got: {}",
                err.message
            );
        }
    }

    #[test]
    fn test_all_keyboard_skin_ids_are_valid() {
        // IDs from KEY_DEFS that are routed through key_map::resolve() (named ops only).
        // Digit keys ("0"-"9", ".", "e") are handled by handle_op() digit branch — not here.
        // Empty-string ids are visual-only and are never sent to dispatch_op.
        let named_ids = [
            "sigma_plus",
            "recip",
            "sqrt",
            "log",
            "ln",
            "sin",
            "cos",
            "tan",
            "rdn",
            "xy_swap",
            "enter",
            "div",
            "mul",
            "user_mode",
            "minus",
            "prgm_mode",
            "alpha_toggle",
            "chs",
            "plus",
            "lastx",
            "clreg",
            "clx",
        ];
        for id in named_ids {
            assert!(
                resolve(id).is_ok(),
                "key_map::resolve({id:?}) must succeed for a KEY_DEFS named id"
            );
        }
    }

    /// Phase 26 W3 — every primary + shifted id present in `Keyboard.tsx`
    /// `KEY_DEFS` must either:
    ///   (a) resolve successfully via `key_map::resolve` (real Op), OR
    ///   (b) hit the stub-error arm (modal-opener intercepted by the frontend
    ///       per D-26.5), OR
    ///   (c) be in the documented bare-handler set (`sst`/`bst`/`r_s`/
    ///       `shift`/`""`/`clx_or_a`) that App.tsx routes outside `dispatch_op`.
    ///
    /// This Rust-side mirror of the TS `key_defs_ids.ts` constants catches
    /// drift in either direction: when a key is added to Keyboard.tsx
    /// KEY_DEFS, this array must grow to match.
    ///
    /// CRITICAL: this array MUST match the union of `KEY_DEFS_PRIMARY_IDS` +
    /// `KEY_DEFS_SHIFTED_IDS` in `hp41-gui/src/key_defs_ids.ts`. Updating one
    /// without the other breaks the W3 audit contract.
    #[test]
    fn test_keyboard_skin_ids_resolve_or_are_modal_openers() {
        // Mirror of Keyboard.tsx TOP_ROW + MAIN_GRID primary + shifted ids.
        // Last audit: Phase 26 — verified against hp41-gui/src/Keyboard.tsx
        // lines 40-94.
        const PRIMARY_IDS: &[&str] = &[
            // TOP_ROW (4 entries; "" for ON is filtered separately).
            "user_mode",
            "prgm_mode",
            "alpha_toggle",
            // MAIN_GRID row 1
            "sigma_plus",
            "recip",
            "sqrt",
            "log",
            "ln",
            // MAIN_GRID row 2
            "xge_y",
            "rdn",
            "sin",
            "cos",
            "tan",
            // MAIN_GRID row 3
            "xeq_prompt",
            "sto_prompt",
            "rcl_prompt",
            "sst",
            // MAIN_GRID row 4
            "enter",
            "chs",
            "clx_or_a",
            // MAIN_GRID row 5
            "minus",
            // MAIN_GRID row 6
            "plus",
            // MAIN_GRID row 7
            "mul",
            // MAIN_GRID row 8
            "div",
            "r_s",
        ];
        const SHIFTED_IDS: &[&str] = &[
            // MAIN_GRID row 1
            "sigma_minus",
            "ypow",
            "sq",
            "tenpow",
            "exp",
            // MAIN_GRID row 2
            "cl_sigma_stat",
            "pct_change",
            "asin",
            "acos",
            "atan",
            // MAIN_GRID row 3
            "asn",
            "lbl_prompt",
            "gto_prompt",
            "bst",
            // MAIN_GRID row 4
            "catalog",
            "isg_prompt",
            "rtn",
            "clx_or_a",
            // MAIN_GRID row 5
            "x_eq_y_prompt",
            "sf_prompt",
            "cf_prompt",
            "fs_prompt",
            // MAIN_GRID row 6
            "x_le_y_prompt",
            "beep",
            "polar_to_rect",
            "rect_to_polar",
            // MAIN_GRID row 7
            "x_gt_y_prompt",
            "fix_prompt",
            "sci_prompt",
            "eng_prompt",
            // MAIN_GRID row 8
            "x_eq_0_prompt",
            "pi",
            "lastx",
            "view",
        ];
        const HANDLED_OUTSIDE_RESOLVE: &[&str] = &["sst", "bst", "r_s", "shift", "clx_or_a"];

        for id in PRIMARY_IDS.iter().chain(SHIFTED_IDS.iter()) {
            if HANDLED_OUTSIDE_RESOLVE.contains(id) {
                continue;
            }
            match resolve(id) {
                Ok(_) => { /* real op — OK */ }
                Err(err) => {
                    // Must be either the stub-error message OR an explicit
                    // documented divergence — but never "unknown key" (would
                    // indicate the id is not a recognised modal-opener and
                    // not a real op = drift).
                    assert!(
                        err.message.contains("planned for a future phase"),
                        "key_map::resolve({id:?}) must either Ok(...) or hit \
                         the modal-opener stub arm. Got error: {}. If this id \
                         is new in Keyboard.tsx, add it either to a real Op \
                         arm in `resolve` OR to the stub list.",
                        err.message
                    );
                }
            }
        }
    }
}
