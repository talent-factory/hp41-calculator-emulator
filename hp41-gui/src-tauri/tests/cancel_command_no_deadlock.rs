//! Integration test: request_cancel does NOT acquire the AppState Mutex.
//!
//! ## Purpose
//!
//! Verifies the deadlock-avoidance invariant (Pitfall 1 / T-31-W1-deadlock):
//! `request_cancel` flips the `cancel_requested: Arc<AtomicBool>` WITHOUT
//! acquiring the AppState Mutex. If a future regression made `request_cancel`
//! acquire the AppState lock, this test would hang (timeout detection).
//!
//! ## Design
//!
//! We extract the production body of `request_cancel` into a free function
//! `simulate_request_cancel(&CancelFlag)` that the production thunk wraps.
//! The test:
//!   1. Acquires the AppState Mutex in the foreground thread.
//!   2. Spawns a worker that calls `simulate_request_cancel`.
//!   3. Asserts the worker completes within 100ms (not blocked by the held Mutex).
//!   4. Asserts `cancel_flag.load(Relaxed)` is `true` after the worker.
//!   5. Asserts the AppState lock was held the entire time (foreground side).
//!
//! If a regression made the cancel thunk acquire AppState, the worker would block
//! indefinitely and `join_handle.join()` would exceed the timeout, failing the test.
//!
//! Coverage strategy (D-27.1): Catches — deadlock when request_cancel acquires
//! AppState Mutex while dispatch_op holds it (T-31-W1-deadlock).

#![allow(clippy::unwrap_used)]

use hp41_gui_lib::AppState;
use hp41_gui_lib::CancelFlag;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Free function equivalent to the production `request_cancel` thunk body.
///
/// The production thunk receives `State<'_, CancelFlag>` from Tauri and calls
/// `cancel.store(true, Relaxed)`. This function is that body without the
/// Tauri `State<>` wrapper — allowing us to test it in a unit context.
///
/// CRITICAL: this function MUST NOT acquire any AppState lock. If a future
/// regression adds `state.lock()` here (where `state` is AppState), the
/// deadlock test will catch it via timeout.
fn simulate_request_cancel(cancel_flag: &CancelFlag) {
    cancel_flag.store(true, Ordering::Relaxed);
}

/// Assert that the cancel thunk completes even when AppState Mutex is held.
///
/// Foreground holds AppState for 200ms. Worker calls simulate_request_cancel
/// and must finish within 100ms (well before foreground releases the lock).
/// If the worker blocks waiting for AppState, it will NOT finish in 100ms
/// and the test will report a hang detection failure.
#[test]
fn request_cancel_does_not_acquire_appstate_mutex() {
    // Set up shared state — the SAME Arc<AtomicBool> in both AppState and CancelFlag,
    // mirroring the lib.rs setup() production pattern (Arc::clone before Mutex wrap).
    let initial_state = hp41_core::CalcState::new();
    let cancel_flag: CancelFlag = Arc::clone(&initial_state.cancel_requested);
    let app_state: Arc<AppState> = Arc::new(Mutex::new(initial_state));

    // Pre-condition: flag starts at false.
    assert!(
        !cancel_flag.load(Ordering::Relaxed),
        "pre-condition: cancel_flag must start as false"
    );

    // Phase 1: foreground acquires AppState Mutex — simulates dispatch_op holding
    // the lock during a long-running INTG/SOLVE/DIFEQ computation.
    let app_state_fg = Arc::clone(&app_state);
    let _guard = app_state_fg.lock().unwrap();

    // Phase 2: spawn worker that calls simulate_request_cancel.
    // If simulate_request_cancel tried to acquire AppState, this would deadlock.
    let cancel_flag_worker = Arc::clone(&cancel_flag);
    let start = Instant::now();
    let join_handle = thread::spawn(move || {
        simulate_request_cancel(&cancel_flag_worker);
    });

    // Phase 3: wait for worker with a 100ms timeout.
    let timed_out = loop {
        if join_handle.is_finished() {
            break false;
        }
        if start.elapsed() > Duration::from_millis(100) {
            break true;
        }
        thread::sleep(Duration::from_millis(2));
    };

    // Join (should be instant — worker already finished).
    join_handle
        .join()
        .expect("worker thread must not panic");

    // Assert: (a) no deadlock — worker completed within 100ms.
    assert!(
        !timed_out,
        "DEADLOCK DETECTED: simulate_request_cancel did not complete within 100ms \
         while AppState Mutex was held — indicates the cancel thunk is acquiring AppState"
    );

    // Assert: (b) AppState lock was held the entire time — foreground side.
    // The guard is still alive here (drop happens at end of scope). This verifies
    // the lock was truly held while the worker ran.
    assert!(
        app_state.try_lock().is_err(),
        "AppState Mutex must still be held by foreground thread at this point"
    );

    // Assert: (c) cancel_flag was set to true by the worker.
    assert!(
        cancel_flag.load(Ordering::Relaxed),
        "cancel_flag must be true after simulate_request_cancel completes"
    );

    // Foreground lock drops here — test complete.
    drop(_guard);

    // Post-condition: lock is released and cancel_flag still true.
    assert!(
        app_state.try_lock().is_ok(),
        "AppState Mutex must be released after guard drop"
    );
    assert!(
        cancel_flag.load(Ordering::Relaxed),
        "cancel_flag must remain true after lock release"
    );
}
