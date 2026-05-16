// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! XROM resolver framework.
//!
//! `MATH_1` is the public registry for Math Pac I (module id 7, "MATH 1A").
//! Downstream plans (28-02..28-10) extend `math1_resolve()` as new `Op` variants are added.
//!
//! Resolver chain contract (C-28.4 / Pitfall 1):
//! `xrom_resolve` fires LAST — after `builtin_card_op`, before `Err(InvalidOp)`.
//! `tests/xrom_shadowing.rs` CI-gates this invariant on every `MATH_1.ops` entry.

use crate::ops::Op;

/// An XROM application-module descriptor.
///
/// `id` matches the HP-41C hardware XROM module ID (7 for Math Pac I).
/// `name` is the string the CATALOG 2 listing would display.
/// `ops` is the canonical mnemonic → Op mapping used both by `xrom_resolve`
/// and by `tests/xrom_shadowing.rs` to assert non-collision with v2.2 builtins.
/// Plans 28-02..28-10 append entries here via `MATH_1_OPS` once each `Op` variant
/// exists. Currently `&[]` (empty) because no Math Pac I `Op` variants exist yet.
pub struct XromModule {
    pub id: u8,
    pub name: &'static str,
    pub ops: &'static [(&'static str, Op)],
}

/// Math Pac I module registry.
///
/// - `id = 7` — real HP-41C hardware Math Pac I XROM module ID.
/// - `name = "MATH 1A"` — as displayed by CATALOG 2 on real hardware.
/// - `ops` — mnemonic → Op mapping; grows with each Plan 28-02..28-10.
///
/// Plan 28-02: 6 hyperbolic entries added (SINH, COSH, TANH, ASINH, ACOSH, ATANH).
/// Plan 28-03: 5 complex arithmetic entries added (C+, C-, C×, C÷, REAL).
///             ASCII aliases C* and C/ included for C× and C÷ respectively.
/// Plan 28-04: 12 complex function entries (MAGZ, CINV, Z↑N, Z↑1/N, E↑Z, LNZ,
///             SINZ, COSZ, TANZ, A↑Z, LOGZ, Z↑W) with ASCII aliases for Unicode ops.
/// Plan 28-05: 2 POLY/ROOTS entries (POLY modal opener, ROOTS executor).
/// Plan 28-06: 8 MATRIX entries (MATRIX, SIZE, VMAT, EDIT, DET, INV, SIMEQ, VCOL).
///             "INV" confirmed non-shadowing: builtin_card_op registers no "INV" string.
pub const MATH_1: XromModule = XromModule {
    id: 7,
    name: "MATH 1A",
    ops: &[
        // ── Plan 28-02: Hyperbolics ────────────────────────────────────────────
        ("SINH", Op::Sinh),
        ("COSH", Op::Cosh),
        ("TANH", Op::Tanh),
        ("ASINH", Op::Asinh),
        ("ACOSH", Op::Acosh),
        ("ATANH", Op::Atanh),
        // ── Plan 28-03: Complex Stack Arithmetic ──────────────────────────────
        ("C+", Op::CPlus),
        ("C-", Op::CMinus),
        ("C\u{00D7}", Op::CTimes),  // Unicode alias (primary)
        ("C*", Op::CTimes),          // ASCII alias for C×
        ("C\u{00F7}", Op::CDiv),     // Unicode alias (primary)
        ("C/", Op::CDiv),            // ASCII alias for C÷
        ("REAL", Op::Real),
        // ── Plan 28-04: Complex Functions ─────────────────────────────────────
        ("MAGZ", Op::Magz),
        ("CINV", Op::Cinv),
        ("Z\u{2191}N", Op::ZpowN),  // Unicode ↑ (primary)
        ("Z^N", Op::ZpowN),          // ASCII alias for Z↑N
        ("Z\u{2191}1/N", Op::Zpow1N), // Unicode ↑ (primary)
        ("Z^1/N", Op::Zpow1N),       // ASCII alias for Z↑1/N
        ("E\u{2191}Z", Op::ExpZ),   // Unicode ↑ (primary)
        ("E^Z", Op::ExpZ),           // ASCII alias for E↑Z
        ("LNZ", Op::LnZ),
        ("SINZ", Op::SinZ),
        ("COSZ", Op::CosZ),
        ("TANZ", Op::TanZ),
        ("A\u{2191}Z", Op::ApowZ),  // Unicode ↑ (primary)
        ("A^Z", Op::ApowZ),          // ASCII alias for A↑Z
        ("LOGZ", Op::LogZ),
        ("Z\u{2191}W", Op::ZpowW),  // Unicode ↑ (primary)
        ("Z^W", Op::ZpowW),          // ASCII alias for Z↑W
        // ── Plan 28-05: POLY / ROOTS ──────────────────────────────────────────
        ("POLY", Op::PolyWorkflow),
        ("ROOTS", Op::Roots),
        // ── Plan 28-06: MATRIX ────────────────────────────────────────────────
        ("MATRIX", Op::MatrixWorkflow),
        ("SIZE", Op::MatSize),
        ("VMAT", Op::MatVmat),
        ("EDIT", Op::MatEdit),
        ("DET", Op::MatDet),
        // "INV" non-shadowing: v2.2 Op::Inv (reciprocal) uses the "1/x" display
        // mnemonic in builtin_card_op, not "INV" — claiming "INV" for MATRIX is safe.
        // Confirmed: RESEARCH §"Resolver-Chain Conflict Map" lines 636-638.
        ("INV", Op::MatInv),
        ("SIMEQ", Op::MatSimeq),
        ("VCOL", Op::MatVcol),
        // ── Plan 28-07: INTG ──────────────────────────────────────────────────
        ("INTG", Op::Integ),
        // ── Plan 28-08: SOLVE / SOL ───────────────────────────────────────────
        ("SOLVE", Op::Solve),
        ("SOL", Op::Sol),
        // ── Plan 28-10: FOUR / Triangle Solvers / TRANS ───────────────────────
        // All 8 mnemonics confirmed non-shadowing per RESEARCH §"Resolver-Chain
        // Conflict Map" lines 619-633 (no v2.2 builtin uses SSS/ASA/SAA/SAS/SSA/FOUR/TRANS/T3D).
        // "T3D" chosen for Op::Trans3d to disambiguate from 2D TRANS (Plan 28-10 decision).
        ("FOUR", Op::Four),
        ("SSS", Op::TriSss),
        ("ASA", Op::TriAsa),
        ("SAA", Op::TriSaa),
        ("SAS", Op::TriSas),
        ("SSA", Op::TriSsa),
        ("TRANS", Op::Trans2d),
        ("T3D", Op::Trans3d),
    ],
};

/// Resolve an XEQ-by-name label against loaded XROM modules.
///
/// Returns `Some(Op)` if `name` matches a Math Pac I mnemonic AND bit 0 of
/// `modules` is set (Math 1 loaded). Returns `None` otherwise.
///
/// LAST-fires invariant: called AFTER `builtin_card_op`, BEFORE `Err(InvalidOp)`
/// at both insertion sites in `hp41-core/src/ops/program.rs` (C-28.4).
pub fn xrom_resolve(name: &str, modules: u8) -> Option<Op> {
    if modules & 0b0000_0001 != 0 {
        if let Some(op) = math1_resolve(name) {
            return Some(op);
        }
    }
    // Future v3.1+ modules go here:
    // if modules & 0b0000_0010 != 0 { stat1_resolve(name) }
    None
}

/// Math Pac I (bit 0) mnemonic resolver.
///
/// Currently returns `None` for all names (no Math Pac I `Op` variants exist yet).
/// Plans 28-02..28-10 extend this match block as new `Op` variants are added:
///
/// ```text
/// // Plan 28-02:
/// "SINH" => Some(Op::Sinh),
/// "COSH" => Some(Op::Cosh),
/// "TANH" => Some(Op::Tanh),
/// "ASINH" => Some(Op::Asinh),
/// "ACOSH" => Some(Op::Acosh),
/// "ATANH" => Some(Op::Atanh),
/// // Plan 28-03/04 adds complex ops ...
/// // Plan 28-05 adds POLY ...
/// // Plan 28-06 adds MATRIX / SIMEQ / DET / TRANS3D / DOT / CROSS ...
/// // Plan 28-07 adds INTG ...
/// // Plan 28-08 adds SOLVE ...
/// // Plan 28-09 adds DIFEQ ...
/// // Plan 28-10 adds FOUR / SSS / SAS / ASA / SSA / AAS / TRANS ...
/// ```
fn math1_resolve(name: &str) -> Option<Op> {
    match name {
        // ── Plan 28-02: Hyperbolics ────────────────────────────────────────────
        "SINH" => Some(Op::Sinh),
        "COSH" => Some(Op::Cosh),
        "TANH" => Some(Op::Tanh),
        "ASINH" => Some(Op::Asinh),
        "ACOSH" => Some(Op::Acosh),
        "ATANH" => Some(Op::Atanh),
        // ── Plan 28-03: Complex Stack Arithmetic ──────────────────────────────
        "C+" => Some(Op::CPlus),
        "C-" => Some(Op::CMinus),
        "C\u{00D7}" | "C*" => Some(Op::CTimes),  // Unicode × and ASCII * both accepted
        "C\u{00F7}" | "C/" => Some(Op::CDiv),     // Unicode ÷ and ASCII / both accepted
        "REAL" => Some(Op::Real),
        // ── Plan 28-04: Complex Functions ─────────────────────────────────────
        "MAGZ" => Some(Op::Magz),
        "CINV" => Some(Op::Cinv),
        "Z\u{2191}N" | "Z^N" => Some(Op::ZpowN),    // Unicode ↑ and ASCII ^ both accepted
        "Z\u{2191}1/N" | "Z^1/N" => Some(Op::Zpow1N),
        "E\u{2191}Z" | "E^Z" => Some(Op::ExpZ),     // Unicode ↑ and ASCII ^ both accepted
        "LNZ" => Some(Op::LnZ),
        "SINZ" => Some(Op::SinZ),
        "COSZ" => Some(Op::CosZ),
        "TANZ" => Some(Op::TanZ),
        "A\u{2191}Z" | "A^Z" => Some(Op::ApowZ),
        "LOGZ" => Some(Op::LogZ),
        "Z\u{2191}W" | "Z^W" => Some(Op::ZpowW),
        // ── Plan 28-05: POLY / ROOTS ──────────────────────────────────────────
        "POLY" => Some(Op::PolyWorkflow),
        "ROOTS" => Some(Op::Roots),
        // ── Plan 28-06: MATRIX ────────────────────────────────────────────────
        "MATRIX" => Some(Op::MatrixWorkflow),
        "SIZE" => Some(Op::MatSize),
        "VMAT" => Some(Op::MatVmat),
        "EDIT" => Some(Op::MatEdit),
        "DET" => Some(Op::MatDet),
        // "INV" is non-shadowing: builtin_card_op does not register "INV"
        // (Op::Inv reciprocal uses "1/x" display name, not "INV" string).
        // Confirmed via RESEARCH §"Resolver-Chain Conflict Map".
        "INV" => Some(Op::MatInv),
        "SIMEQ" => Some(Op::MatSimeq),
        "VCOL" => Some(Op::MatVcol),
        // ── Plan 28-07: INTG ──────────────────────────────────────────────────
        "INTG" => Some(Op::Integ),
        // ── Plan 28-08: SOLVE / SOL ───────────────────────────────────────────
        // "SOLVE" non-shadowing: RESEARCH §"Resolver-Chain Conflict Map" line 628
        // confirms neither "SOLVE" nor "SOL" appears in v2.2 xeq_by_name_local_resolve
        // or builtin_card_op. Safe to claim both for Math Pac I.
        "SOLVE" => Some(Op::Solve),
        "SOL" => Some(Op::Sol),
        // ── Plan 28-10: FOUR / Triangle Solvers / TRANS ───────────────────────
        // All 8 mnemonics confirmed non-shadowing per RESEARCH §"Resolver-Chain
        // Conflict Map" lines 619-633. "T3D" disambiguates from 2D "TRANS" (Plan 28-10).
        "FOUR" => Some(Op::Four),
        "SSS" => Some(Op::TriSss),
        "ASA" => Some(Op::TriAsa),
        "SAA" => Some(Op::TriSaa),
        "SAS" => Some(Op::TriSas),
        "SSA" => Some(Op::TriSsa),
        "TRANS" => Some(Op::Trans2d),
        "T3D" => Some(Op::Trans3d),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{xrom_resolve, MATH_1};
    use crate::ops::Op;

    const NONEXISTENT_NAME: &str = "__MATH1_PROBE_NONEXISTENT__";

    // Catches: bit-mask off-by-one (module loaded bit check)
    #[test]
    fn resolve_returns_none_for_unknown_name_with_module_loaded() {
        // MATH_1 is loaded (bit 0 set), but name is unknown
        let result = xrom_resolve(NONEXISTENT_NAME, 0b0000_0001);
        assert!(
            result.is_none(),
            "xrom_resolve should return None for unknown name even when module is loaded"
        );
    }

    // Catches: bit-mask off-by-one or module-not-loaded path skipped
    #[test]
    fn resolve_returns_none_when_module_not_loaded() {
        // Bit 0 is CLEAR — Math 1 not loaded
        let result = xrom_resolve(NONEXISTENT_NAME, 0b0000_0000);
        assert!(
            result.is_none(),
            "xrom_resolve should return None when Math 1 module is not loaded (bit 0 clear)"
        );
    }

    // Catches: module-bit isolation — bit 0 only, not any bit
    #[test]
    fn resolve_uses_bit_0_only_for_math1() {
        // Bit 1 set, bit 0 clear — Math 1 is NOT loaded
        let result = xrom_resolve("SINH", 0b0000_0010);
        assert!(
            result.is_none(),
            "xrom_resolve must check bit 0 specifically, not any set bit"
        );
    }

    // Catches: MATH_1 const field regression
    #[test]
    fn math1_const_id_and_name() {
        assert_eq!(MATH_1.id, 7, "MATH_1.id must be 7 (HP Math Pac I hardware module ID)");
        assert_eq!(
            MATH_1.name, "MATH 1A",
            "MATH_1.name must be 'MATH 1A' (HP-41C CATALOG 2 display string)"
        );
    }

    // ── Plan 28-02: Positive resolution tests (hyperbolic mnemonics) ─────────

    // Catches: math1_resolve not recognizing SINH when module is loaded
    #[test]
    fn resolve_sinh_with_module_loaded() {
        let result = xrom_resolve("SINH", 0b0000_0001);
        assert_eq!(result, Some(Op::Sinh), "xrom_resolve('SINH', bit0=1) must return Some(Op::Sinh)");
    }

    // Catches: module-not-loaded path not short-circuiting before math1_resolve
    #[test]
    fn resolve_sinh_module_not_loaded_returns_none() {
        let result = xrom_resolve("SINH", 0b0000_0000);
        assert!(result.is_none(), "xrom_resolve('SINH', bit0=0) must return None (module not loaded)");
    }

    // Catches: missing ASINH in math1_resolve match block
    #[test]
    fn resolve_asinh_with_module_loaded() {
        let result = xrom_resolve("ASINH", 0b0000_0001);
        assert_eq!(result, Some(Op::Asinh), "xrom_resolve('ASINH', bit0=1) must return Some(Op::Asinh)");
    }

    // Catches: MATH_1.ops slice not populated with correct count
    // Plan 28-02: 6 hyperbolic entries; Plan 28-03: +7 complex arith entries (C+, C-, C×, C*, C÷, C/, REAL)
    // Plan 28-04: +17 complex function entries (MAGZ, CINV, Z↑N, Z^N, Z↑1/N, Z^1/N, E↑Z, E^Z,
    //             LNZ, SINZ, COSZ, TANZ, A↑Z, A^Z, LOGZ, Z↑W, Z^W)
    // Plan 28-05: +2 POLY/ROOTS entries
    // Plan 28-06: +8 MATRIX entries (MATRIX, SIZE, VMAT, EDIT, DET, INV, SIMEQ, VCOL)
    // Plan 28-07: +1 INTG entry
    // Plan 28-08: +2 SOLVE/SOL entries
    // Plan 28-10: +8 FOUR/SSS/ASA/SAA/SAS/SSA/TRANS/T3D entries
    // Total: 6+7+17+2+8+1+2+8 = 51 entries
    #[test]
    fn math1_ops_has_correct_entry_count() {
        assert_eq!(
            MATH_1.ops.len(),
            51,
            "MATH_1.ops must have exactly 51 entries after Plan 28-10 (6 hyp + 7 complex-arith + 17 complex-fn + 2 poly + 8 matrix + 1 intg + 2 solve + 8 four/tri/trans)"
        );
    }

    // Catches: MATH_1.ops mnemonic strings not matching math1_resolve keys
    #[test]
    fn math1_ops_mnemonics_resolve_consistently() {
        // Every mnemonic in MATH_1.ops must resolve to the same Op via xrom_resolve
        for (name, expected_op) in MATH_1.ops {
            let resolved = xrom_resolve(name, 0b0000_0001);
            assert_eq!(
                resolved.as_ref(),
                Some(expected_op),
                "MATH_1.ops mnemonic {name:?} must resolve to {expected_op:?} via xrom_resolve"
            );
        }
    }
}
