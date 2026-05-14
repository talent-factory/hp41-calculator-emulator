//! Integration tests for Phase 22 Plan 04 (ASN key-assignment op).
//!
//! Covers FN-KEY-01 + AMENDED D-22.18 / OQ-3 Option A:
//!   - assignments field defaults to empty (D-22.22 sentinel)
//!   - v2.0 save-file backward compat via #[serde(default)]
//!   - non-empty name inserts/overwrites
//!   - empty name removes (OQ-3 sentinel)
//!   - JSON round-trip preserves entries (deterministic BTreeMap order)
//!   - struct-variant JSON shape pinned (Pitfall 9)

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{dispatch, Op};
use hp41_core::CalcState;

#[test]
fn test_assignments_field_defaults_to_empty() {
    let s = CalcState::new();
    assert!(
        s.assignments.is_empty(),
        "fresh CalcState must have empty assignments map"
    );
}

#[test]
fn test_load_v20_save_no_assignments_field() {
    // v20-autosave.json lacks the `assignments` field. #[serde(default)]
    // must let it load with assignments = empty BTreeMap. This is the
    // D-22.22 backward-compat sentinel — any future field added to
    // CalcState that drops serde(default) will fail this test.
    let json = std::fs::read_to_string("tests/fixtures/v20-autosave.json").unwrap();
    let s: CalcState = serde_json::from_str(&json).unwrap();
    assert!(
        s.assignments.is_empty(),
        "v2.0 fixture must load with assignments empty"
    );
}

#[test]
fn test_asn_inserts() {
    let mut s = CalcState::new();
    dispatch(
        &mut s,
        Op::Asn {
            name: "SIN".to_string(),
            key_code: 11,
        },
    )
    .unwrap();
    assert_eq!(s.assignments.get(&11), Some(&"SIN".to_string()));
    assert_eq!(s.assignments.len(), 1);
}

#[test]
fn test_asn_overwrites() {
    let mut s = CalcState::new();
    dispatch(
        &mut s,
        Op::Asn {
            name: "SIN".to_string(),
            key_code: 11,
        },
    )
    .unwrap();
    dispatch(
        &mut s,
        Op::Asn {
            name: "COS".to_string(),
            key_code: 11,
        },
    )
    .unwrap();
    assert_eq!(s.assignments.get(&11), Some(&"COS".to_string()));
    assert_eq!(s.assignments.len(), 1, "still only one entry at key 11");
}

#[test]
fn test_asn_empty_name_removes() {
    // OQ-3 Option A sentinel: empty name removes assignment for key_code.
    let mut s = CalcState::new();
    dispatch(
        &mut s,
        Op::Asn {
            name: "SIN".to_string(),
            key_code: 11,
        },
    )
    .unwrap();
    assert_eq!(s.assignments.get(&11), Some(&"SIN".to_string()));

    dispatch(
        &mut s,
        Op::Asn {
            name: String::new(),
            key_code: 11,
        },
    )
    .unwrap();
    assert_eq!(s.assignments.get(&11), None);
    assert!(s.assignments.is_empty());
}

#[test]
fn test_asn_remove_nonexistent_is_noop() {
    // OQ-3: remove on an empty map is a silent no-op (Ok).
    let mut s = CalcState::new();
    let result = dispatch(
        &mut s,
        Op::Asn {
            name: String::new(),
            key_code: 99,
        },
    );
    assert!(result.is_ok(), "remove-nonexistent must succeed: {:?}", result);
    assert!(s.assignments.is_empty());
}

#[test]
fn test_asn_roundtrip_through_json() {
    // FN-KEY-01 SC#5: assignments must survive a JSON save/load round-trip
    // with all entries intact AND in deterministic order (BTreeMap guarantee).
    let mut s = CalcState::new();
    dispatch(
        &mut s,
        Op::Asn {
            name: "SIN".to_string(),
            key_code: 11,
        },
    )
    .unwrap();
    dispatch(
        &mut s,
        Op::Asn {
            name: "COS".to_string(),
            key_code: 12,
        },
    )
    .unwrap();
    dispatch(
        &mut s,
        Op::Asn {
            name: "MYPROG".to_string(),
            key_code: 21,
        },
    )
    .unwrap();

    let json = serde_json::to_string(&s).unwrap();
    let back: CalcState = serde_json::from_str(&json).unwrap();

    assert_eq!(back.assignments.len(), 3);
    assert_eq!(back.assignments.get(&11), Some(&"SIN".to_string()));
    assert_eq!(back.assignments.get(&12), Some(&"COS".to_string()));
    assert_eq!(back.assignments.get(&21), Some(&"MYPROG".to_string()));

    // BTreeMap iter order is sorted by key — re-serializing should produce
    // identical JSON.
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json, json2, "round-trip must be byte-identical (BTreeMap)");
}

#[test]
fn test_asn_json_struct_variant_shape() {
    // Pitfall 9 sentinel: pin the JSON shape of the Op::Asn struct-variant.
    // Serde's default for struct variants is the tagged-object form:
    // `{"Asn":{"name":"SIN","key_code":11}}`. If this changes (e.g., due to
    // a future serde-attribute addition), this test catches it immediately.
    let op = Op::Asn {
        name: "SIN".to_string(),
        key_code: 11,
    };
    let json = serde_json::to_string(&op).unwrap();
    assert_eq!(json, r#"{"Asn":{"name":"SIN","key_code":11}}"#);
}

#[test]
fn test_asn_lift_neutral() {
    // ASN is LiftEffect::Neutral — lift_enabled flag must be unchanged.
    let mut s = CalcState::new();
    s.stack.lift_enabled = true;
    dispatch(
        &mut s,
        Op::Asn {
            name: "SIN".to_string(),
            key_code: 11,
        },
    )
    .unwrap();
    assert!(s.stack.lift_enabled);

    let mut s2 = CalcState::new();
    s2.stack.lift_enabled = false;
    dispatch(
        &mut s2,
        Op::Asn {
            name: String::new(),
            key_code: 11,
        },
    )
    .unwrap();
    assert!(!s2.stack.lift_enabled);
}

#[test]
fn test_asn_multiple_keys_coexist() {
    let mut s = CalcState::new();
    for (kc, name) in [(11u8, "SIN"), (12, "COS"), (13, "TAN")] {
        dispatch(
            &mut s,
            Op::Asn {
                name: name.to_string(),
                key_code: kc,
            },
        )
        .unwrap();
    }
    assert_eq!(s.assignments.len(), 3);
    assert_eq!(s.assignments.get(&11), Some(&"SIN".to_string()));
    assert_eq!(s.assignments.get(&12), Some(&"COS".to_string()));
    assert_eq!(s.assignments.get(&13), Some(&"TAN".to_string()));
    // Removing one preserves the others.
    dispatch(
        &mut s,
        Op::Asn {
            name: String::new(),
            key_code: 12,
        },
    )
    .unwrap();
    assert_eq!(s.assignments.len(), 2);
    assert!(!s.assignments.contains_key(&12));
    assert!(s.assignments.contains_key(&11));
    assert!(s.assignments.contains_key(&13));
}
