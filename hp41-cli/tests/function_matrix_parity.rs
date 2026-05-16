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

use hp41_cli::help_data::help_entries;

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
