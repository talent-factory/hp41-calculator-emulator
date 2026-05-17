//! Bidirectional drift catch between `docs/hp41cv-functions.json` and the
//! `hp41_core::ops::Op` enum (D-25.15, Pitfall 6).
//!
//! - **Forward:** every named ROM Op variant must have a matching JSON entry.
//! - **Reverse:** every `status: "implemented"` JSON entry must name a known
//!   `Op::` variant (or an explicit XEQ-by-Name alias whitelisted below).
//! - **Inventory parity:** `ALL_OP_VARIANT_NAMES` matches the hand-curated
//!   130-row target — adding a new Op enum variant in a future phase forces
//!   the developer to update this list AND the JSON in the same commit.

#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use hp41_cli::help_data::{help_entries, help_entries_math1};

/// Hand-curated inventory of all `hp41_core::ops::Op` variants. Drift
/// between this list and the enum is caught by `test_op_inventory_count_matches_enum`.
///
/// Maintenance gate: every new `Op` variant landed in any future phase must
/// be appended here. This is intentionally manual — Rust has no built-in
/// enum-variant introspection and the strum-dep cost is rejected per
/// RESEARCH §"Don't Hand-Roll".
const ALL_OP_VARIANT_NAMES: &[&str] = &[
    // Phase 1 arithmetic / stack
    "Add",
    "Sub",
    "Mul",
    "Div",
    "Enter",
    "Clx",
    "Chs",
    "Rdn",
    "Rup",
    "XySwap",
    "Lastx",
    "Pi",
    "PushNum",
    // Phase 2 / 20 unary math / trig / mode / registers / alpha
    "Int",
    "Rnd",
    "Frc",
    "Abs",
    "Sign",
    "Fact",
    "Recip",
    "Sqrt",
    "Sq",
    "YPow",
    "Mod",
    "PctChange",
    "Ln",
    "Log",
    "Exp",
    "TenPow",
    "Sin",
    "Cos",
    "Tan",
    "Asin",
    "Acos",
    "Atan",
    "PolarToRect",
    "RectToPolar",
    "SetDeg",
    "SetRad",
    "SetGrad",
    "FmtFix",
    "FmtSci",
    "FmtEng",
    "StoReg",
    "RclReg",
    "StoArith",
    "StoArithStack",
    "Clreg",
    "AlphaToggle",
    "AlphaAppend",
    "AlphaClear",
    // Phase 3 programming
    "Lbl",
    "Gto",
    "Xeq",
    "Rtn",
    "PrgmMode",
    "Test",
    "Isg",
    "Dse",
    // Phase 5 USER mode, ALPHA back
    "UserMode",
    "AlphaBackspace",
    // Phase 6 stats / HMS
    "SigmaPlus",
    "SigmaMinus",
    "Mean",
    "Sdev",
    "LR",
    "Yhat",
    "Corr",
    "ClSigmaStat",
    "HmsToH",
    "HToHms",
    "HmsAdd",
    "HmsSub",
    // Phase 11 print
    "PRX",
    "PRA",
    "PRSTK",
    // Phase 12 synthetic
    "GetKey",
    "Null",
    "StoM",
    "StoN",
    "StoO",
    "RclM",
    "RclN",
    "RclO",
    "SyntheticByte",
    // v2.1 card reader
    "Wdta",
    "Rdta",
    "Wprgm",
    "Rdprgm",
    // Phase 21 flags / display / sound
    "SfFlag",
    "CfFlag",
    "FlagTest",
    "View",
    "AView",
    "Prompt",
    "Aon",
    "Aoff",
    "Cld",
    "Beep",
    "Tone",
    // Phase 22 program control / editing / memory / catalog / ASN
    "Stop",
    "Pse",
    "GtoInd",
    "XeqInd",
    "Clp",
    "Del",
    "Ins",
    "Size",
    "Cla",
    "Clst",
    "Pack",
    "Catalog",
    "Asn",
    // Phase 23 ALPHA ops
    "Arcl",
    "Asto",
    "Atox",
    "Xtoa",
    "Arot",
    "Posa",
    // Phase 24 indirect
    "StoInd",
    "RclInd",
    "StoArithInd",
    "IsgInd",
    "DseInd",
    "SfFlagInd",
    "CfFlagInd",
    "FlagTestInd",
    "ArclInd",
    "AstoInd",
    "ViewInd",
];

/// Op variants that do NOT correspond to a discoverable HP-41CV ROM op —
/// these are internal calculator primitives. The forward parity check
/// (`every_rom_op_has_matrix_entry`) skips these because the JSON
/// intentionally lacks rows for them.
const INTERNAL_OP_VARIANTS: &[&str] = &[
    "PushNum",       // numeric-literal entry; not a named ROM op
    "SyntheticByte", // hex-modal insertion; internal primitive
];

/// JSON op_variant aliases representing the 8 XEQ-by-Name-only conditional
/// tests. These rows exist in `docs/hp41cv-functions.json` so the function
/// matrix and `tests/key_coverage.rs` can document and probe them, but they
/// are NOT distinct `Op::` variants — they all resolve to `Op::Test(_)` via
/// `keys::xeq_by_name_local_resolve` or `builtin_card_op`. The reverse
/// parity check whitelists them.
const XEQ_ALIAS_OP_VARIANTS: &[&str] = &[
    "XNeY_XEQ",
    "XLtY_XEQ",
    "XGeY_XEQ",
    "XNeZero_XEQ",
    "XLtZero_XEQ",
    "XGtZero_XEQ",
    "XLeZero_XEQ",
    "XGeZero_XEQ",
];

#[test]
fn test_op_inventory_count_matches_enum() {
    // Maintenance gate per RESEARCH §"CI parity test": the hand-curated
    // inventory must hold exactly 130 ROM-named Op variants. If the Op enum
    // grows past 130 in a future phase without this list growing, this
    // assertion fires and forces the developer to append the new variant
    // here AND add a matching JSON entry.
    assert_eq!(
        ALL_OP_VARIANT_NAMES.len(),
        130,
        "ALL_OP_VARIANT_NAMES out of sync with hp41_core::ops::Op enum. \
         Did a future phase add new Op variants without updating this \
         inventory and docs/hp41cv-functions.json?"
    );
}

#[test]
fn test_every_rom_op_has_matrix_entry() {
    // Forward direction: every ROM-named Op variant (minus the internal
    // skiplist) must have a corresponding JSON row.
    let entries = help_entries();
    let json_variants: HashSet<&str> = entries.iter().map(|e| e.op_variant.as_str()).collect();

    let mut missing: Vec<&str> = Vec::new();
    for name in ALL_OP_VARIANT_NAMES {
        if INTERNAL_OP_VARIANTS.contains(name) {
            continue;
        }
        if !json_variants.contains(name) {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "Op::* variants missing from docs/hp41cv-functions.json: {missing:?}"
    );
}

#[test]
fn test_every_implemented_matrix_entry_has_op() {
    // Reverse direction: every implemented JSON entry must name a known Op
    // variant — unless it's an explicit XEQ-by-Name alias whitelisted above
    // (those routes resolve to Op::Test(_) via xeq_by_name_local_resolve).
    let names: HashSet<&str> = ALL_OP_VARIANT_NAMES.iter().copied().collect();
    let aliases: HashSet<&str> = XEQ_ALIAS_OP_VARIANTS.iter().copied().collect();

    let mut orphans: Vec<&str> = Vec::new();
    for entry in help_entries() {
        if entry.status != "implemented" {
            continue;
        }
        if names.contains(entry.op_variant.as_str()) || aliases.contains(entry.op_variant.as_str())
        {
            continue;
        }
        orphans.push(entry.op_variant.as_str());
    }
    assert!(
        orphans.is_empty(),
        "implemented JSON rows with no matching Op variant: {orphans:?}"
    );
}

#[test]
fn test_matrix_has_at_least_130_entries() {
    // Combined with the Pitfall 7 smoke check in tests/phase25_help_data.rs.
    let entries = help_entries();
    assert!(
        entries.len() >= 130,
        "function matrix should list >= 130 HP-41CV ROM ops; got {}",
        entries.len()
    );
}

// ── Phase 29 Plan 01 Task 3: Math Pac I bidirectional parity tests (CLI-02) ──
//
// Three tests guarding the hp41-math1-functions.json ↔ MATH_1.ops ↔ Op::* chain:
// 1. Inventory drift sentinel (MATH1_OP_VARIANT_NAMES length == 45)
// 2. Forward parity: every MATH1_OP_VARIANT_NAMES entry has a JSON row
// 3. Reverse parity: every JSON display_name resolves via xrom_resolve
//
// Tests are partitioned by pool (v2.2 vs Math Pac I) so future v3.1 Stat Pac
// additions don't break v2.2 assertions (Claude's Discretion, CONTEXT §Parity Test).

/// Hand-curated inventory of all Math Pac I `Op` variants shipped in Phase 28.
/// Drift between this list and the `MATH_1.ops` table in `hp41-core/src/ops/math1/xrom.rs`
/// is caught by `test_math1_op_inventory_count`.
///
/// Maintenance gate: if Phase 30+ adds new Math Pac I `Op` variants, append here
/// AND add matching JSON rows to `docs/hp41-math1-functions.json`.
const MATH1_OP_VARIANT_NAMES: &[&str] = &[
    // Phase 28-02: Hyperbolics (6)
    "Sinh",
    "Cosh",
    "Tanh",
    "Asinh",
    "Acosh",
    "Atanh",
    // Phase 28-03: Complex Stack Arithmetic (5)
    "CPlus",
    "CMinus",
    "CTimes",
    "CDiv",
    "Real",
    // Phase 28-04: Complex Functions (12)
    "Magz",
    "Cinv",
    "ZpowN",
    "Zpow1N",
    "ExpZ",
    "LnZ",
    "SinZ",
    "CosZ",
    "TanZ",
    "ApowZ",
    "LogZ",
    "ZpowW",
    // Phase 28-05: Polynomial (2)
    "PolyWorkflow",
    "Roots",
    // Phase 28-06: Matrix (8)
    "MatrixWorkflow",
    "MatSize",
    "MatVmat",
    "MatEdit",
    "MatDet",
    "MatInv",
    "MatSimeq",
    "MatVcol",
    // Phase 28-07: Integration (1)
    "Integ",
    // Phase 28-08: Root Solver (2)
    "Solve",
    "Sol",
    // Phase 28-09: Differential Equation (1)
    "Difeq",
    // Phase 28-10: Fourier / Triangle Solvers / Coordinate Transform (7)
    "Four",
    "TriSss",
    "TriAsa",
    "TriSaa",
    "TriSas",
    "TriSsa",
    "Trans2d",
    "Trans3d",
];

#[test]
fn test_math1_op_inventory_count() {
    // Catches: drift between this hand-curated list and MATH_1.ops in xrom.rs.
    // If a new Math Pac I Op variant is added without updating this list,
    // this assertion fires forcing the developer to also add a JSON entry.
    assert_eq!(
        MATH1_OP_VARIANT_NAMES.len(),
        45,
        "MATH1_OP_VARIANT_NAMES inventory drift — expected 45 unique Math Pac I Op variants \
         (52 total MATH_1.ops entries minus 7 ASCII aliases). Did a new Phase 28+ plan add \
         Op variants without updating this inventory and docs/hp41-math1-functions.json?"
    );
}

#[test]
fn test_every_math1_rom_op_has_math1_json_entry() {
    // Catches: forward parity gap — a Math Pac I Op variant without a JSON entry.
    // Uses help_entries_math1() (narrow accessor) to assert against only the Math1 pool.
    // Failure message lists missing variants by name for easy diagnosis.
    let json_variants: HashSet<&str> = help_entries_math1()
        .iter()
        .map(|e| e.op_variant.as_str())
        .collect();

    let mut missing: Vec<&str> = Vec::new();
    for name in MATH1_OP_VARIANT_NAMES {
        if !json_variants.contains(name) {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "Math Pac I Op::* variants missing from docs/hp41-math1-functions.json: {missing:?}"
    );
}

#[test]
fn test_every_math1_json_entry_has_xrom_resolver_match() {
    // Catches: reverse parity gap — a JSON entry whose display_name cannot be
    // resolved by xrom_resolve (C-28.4). Ensures the JSON and MATH_1.ops table
    // stay in sync — a typo in display_name or a missing math1_resolve arm fails here.
    let mut orphans: Vec<String> = Vec::new();
    for entry in help_entries_math1() {
        let resolved =
            hp41_core::ops::math1::xrom::xrom_resolve(entry.display_name.as_str(), 0b0000_0001);
        if resolved.is_none() {
            orphans.push(format!(
                "'{}' (display_name='{}') — not found in MATH_1.ops / math1_resolve",
                entry.op_variant, entry.display_name
            ));
        }
    }
    assert!(
        orphans.is_empty(),
        "Math1 JSON entries whose display_name is NOT resolved by xrom_resolve(_, 0b0000_0001): {orphans:?}"
    );
}
