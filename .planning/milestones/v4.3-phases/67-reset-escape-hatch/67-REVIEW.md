---
phase: 67-reset-escape-hatch
reviewed: 2026-06-10T10:30:00Z
depth: standard
files_reviewed: 17
files_reviewed_list:
  - hp41-core/src/state.rs
  - hp41-core/tests/phase_67_reset.rs
  - hp41-cli/src/app.rs
  - hp41-cli/src/ui.rs
  - hp41-cli/tests/phase67_reset_cli.rs
  - hp41-gui/src-tauri/src/commands.rs
  - hp41-gui/src-tauri/src/persistence.rs
  - hp41-gui/src-tauri/src/lib.rs
  - hp41-gui/src-tauri/capabilities/default.json
  - hp41-gui/src-tauri/permissions/reset-soft.toml
  - hp41-gui/src-tauri/permissions/reset-full.toml
  - hp41-gui/src/App.tsx
  - hp41-gui/src/App.test.tsx
  - hp41-gui/src/Keyboard.tsx
  - hp41-gui/src/App.css
  - hp41-gui/src/OnboardingWizard.tsx
  - hp41-gui/src/HelpOverlay.tsx
findings:
  critical: 2
  warning: 2
  info: 2
  total: 6
status: resolved
disposition:
  CR-01: deferred (out of scope — see below)
  CR-02: fixed (commit 11ee2b0)
  WR-01: deferred (escape-hatch must work while busy by design)
  WR-02: noted (no action)
  IN-01: noted (test polish, optional)
  IN-02: noted (test polish, optional)
---

# Phase 67: Code Review Report

## Orchestrator Disposition (2026-06-10, post-review)

- **CR-02 — FIXED (commit `11ee2b0`).** `soft_reset()` now clears `cancel_requested`
  in place (`store(false)`) and `memory_lost()` preserves the Arc identity across the
  factory reset instead of minting a new Arc. This keeps the GUI's long-lived
  `CancelFlag` clone connected so cancellation keeps working after any reset. Added
  RST-05 regression tests asserting `Arc::ptr_eq` holds across both reset tiers.
  `#[serde(skip)]` on the field means RST-03 JSON equivalence is unaffected.

- **CR-01 — DEFERRED (out of scope, not a Phase-67 regression).** The GUI physical-key
  `'r'`→RDPRGM mapping is **pre-existing and unchanged** by this phase. The locked phase
  design (67-CONTEXT.md) intentionally chose the **ON key** as the GUI reset surface and
  `Ctrl+R` *only* for the CLI ("a terminal has no ON button"). GUI users reach reset by
  tapping/clicking ON. D-25.6 parity is satisfied at the behavior level (both surfaces
  expose soft+full tiers). CLI/GUI keyboard maps diverging by design is a known, accepted
  state. Adding a GUI `Ctrl+R` reset binding is a future enhancement, tracked as a
  follow-up — not closed within Phase 67's scope fence.

- **WR-01 — DEFERRED.** Gating the long-press on `busyRef` would defeat the escape
  hatch's purpose (it must recover a stuck/busy app). The reset Tauri commands acquire
  the `AppState` lock between Phase-63 run-loop yields. Worth an on-device confirmation
  in a follow-up, but not a blocker.

- **WR-02 / IN-01 / IN-02 — NOTED.** Test-quality polish (the RST-01-i `pending_yield`
  assertion is tautological because `make_trapped_state()` never sets it; CLI RST-CLI-07
  negative-guard fragility). Optional; no action this phase.

---


**Reviewed:** 2026-06-10T10:30:00Z
**Depth:** standard
**Files Reviewed:** 17
**Status:** issues_found

## Summary

Phase 67 implements a two-tier "Reset Escape Hatch" feature: a soft reset that clears transient/trapping fields while preserving user data, and a full MEMORY LOST factory reset. The implementation is generally well-structured. The core `soft_reset()` logic is correct and comprehensive; the CLI escape hatch (Ctrl+R) is correctly placed above the `pending_input` routing block; and the Tauri mutex-while-persisting ordering invariant is correctly upheld in `commands.rs`.

Two critical defects were found:

1. **GUI physical-keyboard Ctrl+R** still routes to `xeq_RDPRGM` (card reader read) in `resolveKeyId()` — the GUI keyboard path was NOT updated when the CLI reassigned Ctrl+R to the reset escape hatch. This is a CLI/GUI parity gap (D-25.6 violation) and also means the GUI's physical-keyboard reset path is entirely absent.

2. **`soft_reset()` severs the `CancelFlag` Arc** — when `soft_reset()` calls `default_cancel_requested()` it creates a brand-new `Arc<AtomicBool>`, replacing `CalcState.cancel_requested`. The `CancelFlag` managed state in the Tauri app was cloned from the *original* Arc at startup and now points to a dead object. After any soft reset, `request_cancel` writes to the orphaned Arc and the solver loops read from the new one — cancellation of INTG/SOLVE/DIFEQ is permanently broken for the rest of that process's lifetime.

---

## Critical Issues

### CR-01: GUI physical-keyboard Ctrl+R still dispatches RDPRGM, not the reset escape hatch

**File:** `hp41-gui/src/App.tsx:178`

**Issue:** `resolveKeyId()` maps `case 'r'` (Ctrl+R / Cmd+R) to `'xeq_RDPRGM'`. The CLI comment in `hp41-cli/src/app.rs:743` explicitly documents that `Ctrl+R was previously Rdprgm but is now reserved for the reset escape-hatch (Phase 67-02). Rdprgm is reassigned to Ctrl+E`. The GUI was never updated. Consequences:

- A user pressing Ctrl+R on a physical keyboard in the GUI will fire a card-read operation, not the reset prompt — the escape hatch is unreachable via physical keyboard on the GUI.
- The CLI and GUI are out of parity on a documented, safety-critical keybinding (D-25.6 violation).
- The GUI has no physical-keyboard path to trigger either reset tier at all.

The GUI ON-key pointer handlers are the *only* reset path in the GUI; physical keyboard users (desktop, macOS menu-bar) have no rescue route when the app is stuck.

**Fix:** In `resolveKeyId()`, remap `case 'r'` to a sentinel that the `handleKey` callback can intercept for the reset flow, OR route it to a dedicated `'__reset_soft__'` id handled before `dispatchKeyId`. The `xeq_RDPRGM` should move to `case 'e'` to match the CLI reassignment:

```typescript
// In resolveKeyId(), App.tsx line ~176-182:
switch (e.key.toLowerCase()) {
  case 'w': return 'xeq_WPRGM';
  case 'r': return '__reset_soft__';   // Phase 67: escape hatch (was xeq_RDPRGM)
  case 'e': return 'xeq_RDPRGM';       // Phase 67: RDPRGM reassigned to Ctrl+E (mirrors CLI)
  case 'd': return 'xeq_WDTA';
  case 'f': return 'xeq_RDTA';
  case 's': return '__save_state__';
  default: return null;
}
```

Then in `handleKey`, intercept `'__reset_soft__'` before `dispatchKeyId` (analogous to `'__save_state__'`) and invoke `reset_soft`.

---

### CR-02: `soft_reset()` orphans the `CancelFlag` Arc — cancellation permanently broken after any soft reset

**File:** `hp41-core/src/state.rs` (soft_reset implementation) + `hp41-gui/src-tauri/src/lib.rs:159-162`

**Issue:** `soft_reset()` ends with:

```rust
self.cancel_requested = default_cancel_requested();
```

This replaces `CalcState.cancel_requested` with a brand-new `Arc<AtomicBool>`. However, the Tauri `CancelFlag` managed state is set up at startup (`lib.rs:159-162`) by cloning the Arc from the *initial* `CalcState`:

```rust
let cancel_flag: CancelFlag =
    std::sync::Arc::clone(&initial_state.cancel_requested);
app.manage(cancel_flag);
```

After `soft_reset()` runs, `CalcState.cancel_requested` points to a new Arc (reference count = 1, held only by `CalcState`). The managed `CancelFlag` still points to the original Arc (now orphaned — nothing reads it). The `request_cancel` Tauri command writes `cancel_flag.store(true, ...)` to the orphaned Arc. INTG/SOLVE/DIFEQ read `state.cancel_requested.load(...)` from the new Arc. The two Arcs are permanently disconnected — `request_cancel` can no longer cancel any long-running operation for the rest of the process lifetime.

This is a correctness/data-loss-adjacent bug: a user who performs a soft reset and then kicks off a 10,000-iteration INTG cannot cancel it.

**Fix (option A — preferred):** Do not replace the Arc in `soft_reset()`; instead just reset its *value* to `false`:

```rust
// Replace: self.cancel_requested = default_cancel_requested();
// With:
self.cancel_requested.store(false, std::sync::atomic::Ordering::SeqCst);
```

The CLI path uses the same `CalcState` struct and does not have a separate `CancelFlag` managed state, so this change is backward-compatible with the CLI.

**Fix (option B — alternative):** Keep the Arc replacement but update `CancelFlag` in the Tauri commands. Pass a `State<'_, CancelFlag>` parameter to `reset_soft` and re-point it after the swap. This is more invasive and requires a new Tauri permission.

Option A is strongly preferred.

---

## Warnings

### WR-01: `handleOnPointerDown` does not guard `busyRef` — long-press timer can fire while IPC is in-flight

**File:** `hp41-gui/src/App.tsx:422-430`

**Issue:** `handleOnPointerDown` starts the 600ms long-press timer unconditionally without checking `busyRef.current`:

```typescript
const handleOnPointerDown = useCallback(() => {
  if (busyRef.current) return;   // <-- this guard is ABSENT
  longPressFiredRef.current = false;
  longPressTimerRef.current = setTimeout(() => {
    longPressFiredRef.current = true;
    longPressTimerRef.current = null;
    setConfirmSheetOpen(true);
  }, LONG_PRESS_MS);
}, []);
```

The `busyRef` guard IS present in `handleOnPointerUp` (tap path, line 443) and in `handleConfirmFullReset` (line 464). But if a user presses and holds the ON key while a `dispatch_op` IPC call is in-flight (e.g. mid-INTG), the timer fires after 600ms and opens the confirm sheet while `busyRef.current = true`. The user then clicks Confirm, which hits the `busyRef` guard and silently does nothing — the MEMORY LOST is not performed. This is confusing UX but not data-destructive; the workaround is to release the IPC first.

More critically: if the user releases before 600ms (tap path), `handleOnPointerUp` correctly checks `busyRef` and bails. But `longPressFiredRef.current` is already `false` so neither branch fires `reset_soft`. The no-double-fire guard is correct, but the timer was needlessly started.

**Fix:** Add `if (busyRef.current) return;` as the first line of `handleOnPointerDown` (matches the pattern in every other pointer handler):

```typescript
const handleOnPointerDown = useCallback(() => {
  if (busyRef.current) return;  // add this guard
  longPressFiredRef.current = false;
  longPressTimerRef.current = setTimeout(() => {
    ...
  }, LONG_PRESS_MS);
}, []);
```

---

### WR-02: `handleOnPointerDown` has an empty dependency array but reads `busyRef` — stale-closure risk if the guard is added

**File:** `hp41-gui/src/App.tsx:422-430`

**Issue:** `handleOnPointerDown` is `useCallback(() => {...}, [])` with no dependencies. `busyRef` is a ref (stable identity, `useRef`) so the empty array is currently correct for the existing code. However this is coupled to WR-01: once the `busyRef.current` read is added (per WR-01 fix), the empty dependency array remains correct because `busyRef` is a ref object and its `.current` is read at call-time, not captured at creation-time. This is fine.

The real issue is a minor inconsistency: `handleOnPointerUp` and `handleOnPointerCancel` also have `[]` deps (correct for refs), while `handleConfirmFullReset` has `[showToast]` deps (also correct). The pattern is consistent. This WR exists to flag that the dependency analysis should be re-verified if the callbacks are extended.

**Fix:** No code change needed; verify deps when extending these callbacks. Lint rule `react-hooks/exhaustive-deps` should be enforced in the Vitest/ESLint config to catch future regressions automatically.

---

## Info

### IN-01: `make_trapped_state()` in tests does not set `pending_yield` — RST-01-i only partially exercised

**File:** `hp41-core/tests/phase_67_reset.rs:62-121`

**Issue:** The test helper `make_trapped_state()` sets `pending_interrupt`, `pending_interrupt_alarm_index`, `pending_interrupt_depth`, and `getkey_captured_code` but does NOT set `pending_yield`. The test `soft_reset_clears_phase63_64_transients` (RST-01-i, line 202) asserts `s.pending_yield.is_none()` after soft_reset(), but since `pending_yield` was never set to `Some(...)`, this assertion tests that a None field is still None — it cannot catch a regression where `soft_reset()` fails to clear a non-None `pending_yield`.

**Fix:** Add to `make_trapped_state()`:

```rust
s.pending_yield = Some(hp41_core::state::YieldState {
    kind: hp41_core::state::YieldKind::Pse,
    text: "PSE".to_string(),
    resume_ms: 1000,
});
```

---

### IN-02: CLI test `test_ctrl_r_no_longer_dispatches_rdprgm` sets `alpha_reg` but does not assert its content is unchanged — weak signal

**File:** `hp41-cli/tests/phase67_reset_cli.rs:371-392`

**Issue:** The test sets `app.state.alpha_reg = "MISSING".to_string()` and then checks `!msg.to_lowercase().contains("card")`. This is an indirect indicator that RDPRGM did not run. If a future refactor changes the error message format for card operations (e.g. to "Card file not found"), the test would silently pass even if RDPRGM ran. A direct assertion on `app.reset_prompt == ResetPrompt::AwaitingTier` is already present (line 381) which is the primary check — but the negative card-message check is the only guard that specifically verifies RDPRGM was NOT dispatched.

**Fix:** Add an explicit assertion that the alpha_reg is unchanged (soft reset was not triggered either):

```rust
assert_eq!(
    app.state.alpha_reg, "MISSING",
    "alpha_reg must be unchanged — Ctrl+R must not dispatch any op"
);
```

---

_Reviewed: 2026-06-10T10:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
