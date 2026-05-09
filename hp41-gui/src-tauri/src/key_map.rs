//! Phase 14 IPC Layer — String key ID → Op resolver.
//!
//! Decisions: D-04..D-07 (key ID convention).
//! - Digit keys ("0"-"9", ".", "e") are NOT resolved here — they are special-cased in
//!   commands::dispatch_op and append directly to entry_buf (RESEARCH.md Pattern 6).
//! - Named ops use snake_case strings mirroring `hp41_cli::keys::key_to_op` semantically.
//! - Parameterized ops use compound key IDs: `"sto_NN"`, `"fix_N"`, `"sto_arith_<op>_<reg>"`.
//! - Unknown key IDs return `Err(GuiError)` (D-07: never silent discard).
//!
//! Wave 0 status: signature + RED tests. Body is `unimplemented!()` until Wave 1 (Plan 01).

use crate::types::GuiError;
use hp41_core::ops::Op;

/// Resolve a string key ID to an Op. Returns `Err(GuiError)` for unknown IDs.
///
/// This function does NOT handle digit keys (0-9, `.`, `e`) — those are caught earlier
/// in `commands::dispatch_op` and append to `entry_buf` directly.
pub fn resolve(_key_id: &str) -> Result<Op, GuiError> {
    unimplemented!("Wave 1 (Plan 01): match named ops + parameterized prefixes per RESEARCH.md Pattern 5")
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
}
