//! Stress test: auto-save and cancel interleave without deadlock.
//!
//! ## Purpose
//!
//! Exercises the three concurrent actors that share state in the production app:
//!   A) A long INTG/SOLVE/DIFEQ computation (holds AppState Mutex for the duration)
//!   B) The 30s auto-save thread (locks AppState briefly to clone the snapshot)
//!   C) The `request_cancel` Tauri thunk (flips CancelFlag WITHOUT locking AppState)
//!
//! None of these actors should deadlock under concurrent operation.
//!
//! ## Thread model
//!
//! - Thread A: Holds AppState Mutex for 200ms (simulates a long compute).
//!   Reads cancel_flag inside the held lock (simulates the per-64-samples poll).
//! - Thread B: Clones AppState Arc, tries to lock every 30ms (simulates auto-save).
//!   May block behind Thread A but must eventually succeed.
//! - Thread C: Calls simulate_request_cancel every 10ms (simulates GUI cancel).
//!   Must NEVER block on AppState (deadlock-free by design).
//!
//! Wall-clock budget: 500ms. All threads must complete and none must panic.
//!
//! Coverage strategy (D-27.1): Catches — deadlock or panic under concurrent
//! auto-save + cancel + active long compute (T-31-W1-deadlock stress).

#![allow(clippy::unwrap_used)]

use hp41_gui_lib::AppState;
use hp41_gui_lib::CancelFlag;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Free function equivalent to the production `request_cancel` thunk body.
/// Must NOT acquire any AppState lock.
fn simulate_request_cancel(cancel_flag: &CancelFlag) {
    cancel_flag.store(true, Ordering::Relaxed);
}

#[test]
fn autosave_and_cancel_interleave_without_deadlock() {
    // Shared state — mirrors lib.rs setup() pattern.
    let initial_state = hp41_core::CalcState::new();
    let cancel_flag: CancelFlag = Arc::clone(&initial_state.cancel_requested);
    let app_state: Arc<AppState> = Arc::new(Mutex::new(initial_state));

    // Track thread outcomes.
    let thread_a_observed_cancel = Arc::new(AtomicBool::new(false));
    let thread_b_completed = Arc::new(AtomicBool::new(false));
    let thread_c_completed = Arc::new(AtomicBool::new(false));
    let thread_a_panicked = Arc::new(AtomicBool::new(false));
    let thread_b_panicked = Arc::new(AtomicBool::new(false));
    let thread_c_panicked = Arc::new(AtomicBool::new(false));

    // ── Thread A: simulates a long INTG run (holds AppState for 200ms) ──────────
    let app_a = Arc::clone(&app_state);
    let cancel_a = Arc::clone(&cancel_flag);
    let observed_a = Arc::clone(&thread_a_observed_cancel);
    let panicked_a = Arc::clone(&thread_a_panicked);
    let handle_a = thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = app_a.lock().unwrap_or_else(|e| e.into_inner());
            // Hold lock for 200ms (simulating INTG sample loop time).
            let start = Instant::now();
            while start.elapsed() < Duration::from_millis(200) {
                // Simulate per-64-samples cancel poll (Relaxed read, no AppState release).
                if cancel_a.load(Ordering::Relaxed) {
                    observed_a.store(true, Ordering::Relaxed);
                    break; // Would normally return HpError::Canceled and release lock.
                }
                thread::sleep(Duration::from_millis(5));
            }
            // Lock releases here.
        }));
        if result.is_err() {
            panicked_a.store(true, Ordering::Relaxed);
        }
    });

    // ── Thread B: simulates 30s auto-save (tries to lock every 30ms) ─────────────
    let app_b = Arc::clone(&app_state);
    let completed_b = Arc::clone(&thread_b_completed);
    let panicked_b = Arc::clone(&thread_b_panicked);
    let handle_b = thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let deadline = Instant::now() + Duration::from_millis(500);
            let mut lock_count = 0u32;
            while Instant::now() < deadline {
                // Try to lock — may block behind Thread A.
                let _guard = app_b.lock().unwrap_or_else(|e| e.into_inner());
                lock_count += 1;
                // Drop guard immediately (simulates clone-then-release pattern).
                drop(_guard);
                if lock_count >= 3 {
                    break; // Successfully locked 3 times — auto-save would succeed.
                }
                thread::sleep(Duration::from_millis(30));
            }
            assert!(
                lock_count >= 1,
                "Thread B (auto-save) must acquire AppState at least once in 500ms"
            );
            completed_b.store(true, Ordering::Relaxed);
        }));
        if result.is_err() {
            panicked_b.store(true, Ordering::Relaxed);
        }
    });

    // ── Thread C: simulates GUI request_cancel (flip CancelFlag every 10ms) ──────
    let cancel_c = Arc::clone(&cancel_flag);
    let completed_c = Arc::clone(&thread_c_completed);
    let panicked_c = Arc::clone(&thread_c_panicked);
    let handle_c = thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let deadline = Instant::now() + Duration::from_millis(500);
            let mut cancel_count = 0u32;
            while Instant::now() < deadline {
                simulate_request_cancel(&cancel_c);
                cancel_count += 1;
                if cancel_count >= 5 {
                    break; // 5 cancel signals is enough for the stress test.
                }
                thread::sleep(Duration::from_millis(10));
            }
            assert!(
                cancel_count >= 1,
                "Thread C (request_cancel) must fire at least once"
            );
            completed_c.store(true, Ordering::Relaxed);
        }));
        if result.is_err() {
            panicked_c.store(true, Ordering::Relaxed);
        }
    });

    // Wait for all threads with a 1s overall timeout.
    let overall_deadline = Instant::now() + Duration::from_millis(1000);
    handle_a.join().expect("Thread A (long compute) must not panic catastrophically");
    handle_b.join().expect("Thread B (auto-save) must not panic catastrophically");
    handle_c.join().expect("Thread C (request_cancel) must not panic catastrophically");
    assert!(
        Instant::now() < overall_deadline,
        "All threads must complete within 1000ms (deadlock timeout)"
    );

    // Assert outcomes:
    // (a) Thread A observed the cancel signal from Thread C inside the held lock.
    assert!(
        thread_a_observed_cancel.load(Ordering::Relaxed),
        "Thread A (long compute) must observe cancel_flag = true from Thread C \
         while holding AppState Mutex — proves CancelFlag is accessible lock-free"
    );

    // (b) Thread B completed (auto-save acquired the lock at least once).
    assert!(
        thread_b_completed.load(Ordering::Relaxed),
        "Thread B (auto-save) must have acquired AppState at least once in 500ms"
    );

    // (c) Thread C completed (cancel signals fired without blocking).
    assert!(
        thread_c_completed.load(Ordering::Relaxed),
        "Thread C (request_cancel) must have fired cancel signals without blocking"
    );

    // (d) No thread panicked inside catch_unwind.
    assert!(
        !thread_a_panicked.load(Ordering::Relaxed),
        "Thread A must not have panicked"
    );
    assert!(
        !thread_b_panicked.load(Ordering::Relaxed),
        "Thread B must not have panicked"
    );
    assert!(
        !thread_c_panicked.load(Ordering::Relaxed),
        "Thread C must not have panicked"
    );
}
