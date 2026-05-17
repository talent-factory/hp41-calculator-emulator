// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Idempotency invariant tests for Plan 31-02 (T-31-W1-sticky-cancel mitigation).
//!
//! ## Purpose
//!
//! After a user cancels a long-running Math Pac I operation (INTG / SOLVE / DIFEQ)
//! via `request_cancel`, the `CalcState.cancel_requested` AtomicBool stays `true`.
//! Without a reset at workflow-opener time, the NEXT run of the same op would
//! immediately return `HpError::Canceled` on its first per-64-samples check —
//! the "sticky-cancel" bug (T-31-W1-sticky-cancel).
//!
//! Plan 31-02 Task 3 adds a `cancel_requested.store(false, Relaxed)` reset at the
//! top of each interactive dispatch arm (op_integ / op_solve / op_difeq) when the
//! user initiates the operation interactively (`!state.is_running`).
//!
//! These three tests assert: after pre-setting cancel_requested = true, invoking
//! the workflow opener MUST reset it to false before returning.
//!
//! ## Surgical hp41-core exception
//!
//! The 3×1-line changes are a documented exception to the Phase 28 math1 freeze,
//! analogous to v2.2 Phase 25-03's `builtin_card_op` 4→12 extension. NO new `pub`
//! functions, no new `Op::*` variants, no new `CalcState` fields — purely additive
//! Relaxed stores at existing function entry points.
//!
//! Coverage strategy (D-27.1): every test carries a `// Catches:` comment.
#![allow(clippy::unwrap_used)]

use std::sync::atomic::Ordering;

use hp41_core::ops::dispatch;
use hp41_core::ops::Op;
use hp41_core::state::CalcState;

// ── Test 1: op_integ dispatch arm resets cancel_requested ─────────────────────

// Catches: sticky-cancel bug after INTG is cancelled and re-initiated
// (T-31-W1-sticky-cancel / Plan 31-02 Task 3 guard).
#[test]
fn cancel_flag_resets_on_integ_open() {
    let mut state = CalcState::new();
    // Simulate: user had a previous INTG run that was cancelled.
    state.cancel_requested.store(true, Ordering::Relaxed);
    assert!(
        state.cancel_requested.load(Ordering::Relaxed),
        "pre-condition: cancel_requested must be true before invoking op_integ"
    );

    // Simulate: user initiates a new INTG interactively (is_running = false).
    // This calls the workflow-opener dispatch arm (op_integ with !state.is_running).
    // The dispatch arm must reset cancel_requested before opening the modal.
    state.is_running = false;
    let result = dispatch(&mut state, Op::Integ);

    assert!(
        result.is_ok(),
        "op_integ interactive dispatch must return Ok (opens modal)"
    );
    assert!(
        !state.cancel_requested.load(Ordering::Relaxed),
        "cancel_requested must be reset to false after op_integ workflow-opener entry"
    );
    assert!(
        state.modal_program.is_some(),
        "op_integ must open the INTG modal on interactive dispatch"
    );
}

// ── Test 2: op_solve dispatch arm resets cancel_requested ─────────────────────

// Catches: sticky-cancel bug after SOLVE is cancelled and re-initiated
// (T-31-W1-sticky-cancel / Plan 31-02 Task 3 guard — symmetric with INTG).
#[test]
fn cancel_flag_resets_on_solve_open() {
    let mut state = CalcState::new();
    // Simulate: user had a previous SOLVE run that was cancelled.
    state.cancel_requested.store(true, Ordering::Relaxed);
    assert!(
        state.cancel_requested.load(Ordering::Relaxed),
        "pre-condition: cancel_requested must be true before invoking op_solve"
    );

    // Simulate: user initiates a new SOLVE interactively (is_running = false).
    state.is_running = false;
    let result = dispatch(&mut state, Op::Solve);

    assert!(
        result.is_ok(),
        "op_solve interactive dispatch must return Ok (opens modal)"
    );
    assert!(
        !state.cancel_requested.load(Ordering::Relaxed),
        "cancel_requested must be reset to false after op_solve workflow-opener entry"
    );
    assert!(
        state.modal_program.is_some(),
        "op_solve must open the SOLVE modal on interactive dispatch"
    );
}

// ── Test 3: op_difeq dispatch arm resets cancel_requested ─────────────────────

// Catches: sticky-cancel bug after DIFEQ is cancelled and re-initiated
// (T-31-W1-sticky-cancel / Plan 31-02 Task 3 guard — symmetric with INTG/SOLVE).
#[test]
fn cancel_flag_resets_on_difeq_open() {
    let mut state = CalcState::new();
    // Simulate: user had a previous DIFEQ run that was cancelled.
    state.cancel_requested.store(true, Ordering::Relaxed);
    assert!(
        state.cancel_requested.load(Ordering::Relaxed),
        "pre-condition: cancel_requested must be true before invoking op_difeq"
    );

    // Simulate: user initiates a new DIFEQ interactively (is_running = false).
    state.is_running = false;
    let result = dispatch(&mut state, Op::Difeq);

    assert!(
        result.is_ok(),
        "op_difeq interactive dispatch must return Ok (opens modal)"
    );
    assert!(
        !state.cancel_requested.load(Ordering::Relaxed),
        "cancel_requested must be reset to false after op_difeq workflow-opener entry"
    );
    assert!(
        state.modal_program.is_some(),
        "op_difeq must open the DIFEQ modal on interactive dispatch"
    );
}
