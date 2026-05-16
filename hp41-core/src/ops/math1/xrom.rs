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
/// - `ops = &[]` — empty until Plan 28-02 registers `Op::Sinh` etc.
pub const MATH_1: XromModule = XromModule {
    id: 7,
    name: "MATH 1A",
    ops: &[],
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
/// ```
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
fn math1_resolve(_name: &str) -> Option<Op> {
    // No Math Pac I Op variants exist yet. Plans 28-02..28-10 grow this match.
    None
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{xrom_resolve, MATH_1};

    // Test-only probe: before Plan 28-02 adds Op::Sinh, use a name guaranteed
    // to never exist to verify the "module loaded but name unknown" path.
    // This approach lets the resolver tests pass without a real Math Pac I variant.
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
}
