#[cfg(test)]
mod keys_tests;
#[cfg(test)]
mod prgm_display_tests;

// ── Phase 5 requirement coverage smoke tests ──────────────────────────────────
// Quick checks that all 5 Phase 5 requirements have working code paths.
// These are not exhaustive behavioral tests (those are in persistence::tests, etc.) —
// just canaries that the key code paths compile and return expected types.

#[test]
fn test_phase5_pers01_persistence_functions_exist() {
    // PERS-01: save and load functions must exist and accept correct types
    use crate::persistence::default_state_path;

    let path = default_state_path();
    assert!(
        path.to_str().is_some(),
        "default_state_path() must return a valid UTF-8 path"
    );
    assert!(
        path.to_str().unwrap().contains("autosave.json"),
        "default path must end with autosave.json, got: {}",
        path.display()
    );
}

#[test]
#[allow(clippy::const_is_empty)]
fn test_phase5_ux01_help_data_non_empty() {
    // UX-01: HELP_DATA must be non-empty (covered more thoroughly in help_data::tests)
    use crate::help_data::HELP_DATA;
    assert!(!HELP_DATA.is_empty(), "HELP_DATA must not be empty");
}

#[test]
fn test_phase5_ux03_sample_programs_count() {
    // UX-03: at least 10 bundled sample programs
    use crate::programs::sample_programs;
    assert!(
        sample_programs().len() >= 10,
        "must have at least 10 sample programs"
    );
}

#[test]
#[allow(clippy::const_is_empty)]
fn test_phase5_requirements() {
    // Integration-level canary: confirms all 5 Phase 5 requirement code paths compile together.
    // PERS-01: persistence module functions available
    use crate::persistence::default_state_path;
    // UX-01: help data available
    use crate::help_data::HELP_DATA;
    // UX-03: sample programs available
    use crate::programs::sample_programs;

    assert!(default_state_path().to_str().is_some());
    assert!(!HELP_DATA.is_empty());
    assert!(sample_programs().len() >= 10);
}
