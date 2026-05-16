// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Wave-0 integration test: v2.2 save-file backward compatibility with v3.0 CalcState.
//!
//! Asserts that a synthetic v2.2-shape JSON (no v3.0 fields) deserializes into a
//! `CalcState` with correct v3.0 defaults. This is the canonical Pitfall 12 mitigation.
//!
//! Pitfall 12: if a new Phase 28 field lacks `#[serde(default)]`, loading a v2.2
//! save file panics with "missing field `xrom_modules`". This test catches that.

#![allow(clippy::unwrap_used)]

use hp41_core::CalcState;

/// A synthetic v2.2 save-file JSON containing all v2.2 fields and NONE of the v3.0 ones.
///
/// This is a canonical Pitfall-12 mitigation: if any v3.0 field lacks `#[serde(default)]`,
/// this deserialization will fail with a serde error ("missing field `xrom_modules`" or
/// similar), caught immediately by `unwrap()`.
const V22_JSON: &str = r#"{
    "stack": {
        "x": "0",
        "y": "0",
        "z": "0",
        "t": "0",
        "lastx": "0",
        "lift_enabled": false
    },
    "regs": [
        "0","0","0","0","0","0","0","0","0","0",
        "0","0","0","0","0","0","0","0","0","0",
        "0","0","0","0","0","0","0","0","0","0",
        "0","0","0","0","0","0","0","0","0","0",
        "0","0","0","0","0","0","0","0","0","0",
        "0","0","0","0","0","0","0","0","0","0",
        "0","0","0","0","0","0","0","0","0","0",
        "0","0","0","0","0","0","0","0","0","0",
        "0","0","0","0","0","0","0","0","0","0",
        "0","0","0","0","0","0","0","0","0","0"
    ],
    "alpha_reg": "",
    "alpha_mode": false,
    "angle_mode": "Deg",
    "display_mode": {"Fix": 4},
    "entry_buf": "",
    "program": [],
    "prgm_mode": false,
    "pc": 0,
    "call_stack": [],
    "is_running": false,
    "user_mode": false,
    "key_assignments": {},
    "assignments": {},
    "text_regs": {},
    "last_key_code": 0,
    "reg_m": "0",
    "reg_n": "0",
    "reg_o": "0",
    "flags": 0,
    "pending_card_op": null
}"#;

/// Loads a v2.2-shape JSON without any v3.0 fields and asserts all v3.0 fields
/// default correctly.
///
/// Catches: Pitfall 12 — missing `#[serde(default)]` on any new CalcState field
/// causes a deserialization failure ("missing field") when loading a v2.2 save file.
#[test]
fn loads_synthetic_v22_save_without_v3_fields() {
    let state: CalcState = serde_json::from_str(V22_JSON)
        .expect("v2.2-shape JSON must deserialize without error into v3.0 CalcState");

    // Phase 28 fields must be at their documented defaults
    assert_eq!(
        state.xrom_modules, 0b0000_0001,
        "v2.2 save must produce default xrom_modules = 0b0000_0001 (Math 1 pre-loaded)"
    );
    assert!(
        !state.complex_mode,
        "v2.2 save must produce default complex_mode = false"
    );
    assert_eq!(
        state.matrix_dim, None,
        "v2.2 save must produce default matrix_dim = None"
    );
    assert_eq!(
        state.matrix_active_reg, None,
        "v2.2 save must produce default matrix_active_reg = None"
    );
    assert!(
        state.modal_program.is_none(),
        "v2.2 save must produce default modal_program = None (serde skip)"
    );
    assert!(
        state.modal_prompt.is_none(),
        "v2.2 save must produce default modal_prompt = None (serde skip)"
    );
    assert!(
        state.integ_state.is_none(),
        "v2.2 save must produce default integ_state = None (serde skip)"
    );
    assert!(
        state.solve_state.is_none(),
        "v2.2 save must produce default solve_state = None (serde skip)"
    );
    assert!(
        state.difeq_state.is_none(),
        "v2.2 save must produce default difeq_state = None (serde skip)"
    );
}

/// Verifies that a v3.0 save (with xrom_modules, complex_mode etc.) round-trips correctly.
///
/// Catches: serde regression where a field is serialized but deserialized with wrong type
/// or value.
#[test]
fn v30_save_roundtrips_phase28_fields() {
    let mut state = CalcState::new();
    state.xrom_modules = 0b0000_0011; // Math 1 + hypothetical future module
    state.complex_mode = true;
    state.matrix_dim = Some((4, 3));
    state.matrix_active_reg = Some(7);

    let json = serde_json::to_string(&state).unwrap();
    let restored: CalcState = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.xrom_modules, 0b0000_0011);
    assert!(restored.complex_mode);
    assert_eq!(restored.matrix_dim, Some((4, 3)));
    assert_eq!(restored.matrix_active_reg, Some(7));
    // Transient fields reset to defaults
    assert!(restored.modal_program.is_none());
    assert!(restored.integ_state.is_none());
}
