---
phase: 67-reset-escape-hatch
verified: 2026-06-10T17:30:00Z
status: passed
score: 6/6 must-haves verified
overrides_applied: 0
human_verification_resolved:
  - test: "Run `just ci` and `just gui-ci` post-CR-02-fix (commit 11ee2b0); both exit 0"
    resolution: "RESOLVED by orchestrator 2026-06-10. `just ci` exit 0 (coverage 95.27% regions ≥93% target; state.rs 99.03% lines / 99.26% regions; Free42 contamination none). `just gui-ci` exit 0 (GUI Rust tests pass; release build clean; vitest 348 passed / 12 files). Workspace core+cli 3714 passed; MSRV clippy clean; root cargo fmt --check clean."
  - test: "On-device: tap ON = soft reset; long-press ~600ms = portaled MEMORY LOST sheet (not clipped by transform:scale); Confirm = factory state; pointer-up after long-press no double-fire"
    resolution: "RESOLVED by user during the Plan 67-04 human-verify checkpoint (approved 2026-06-10; recorded in 67-04-SUMMARY as T-67-10 PASSED). The CR-02 fix is hp41-core-only (cancel_requested Arc internals) and does not touch App.tsx / Keyboard.tsx gesture or portal behavior, so the on-device approval remains valid post-fix."
---

# Phase 67: Reset Escape Hatch — Verification Report

**Phase Goal:** A user whose calculator is stuck in an input-blocking state — including a persisted trap that survives an app restart (shared autosave reloads it) — recovers in-app via a two-tier reset, without reinstalling. Soft reset (GUI/iOS ON tap, CLI Ctrl+R→s) clears working state + every input-trapping field while preserving stored user data; full reset / MEMORY LOST (GUI/iOS ON long-press+confirm, CLI Ctrl+R→f→y/n) restores factory state. Both run OUTSIDE the key→Op dispatch path and overwrite the autosave synchronously so recovery survives restart.

**Verified:** 2026-06-10T17:30:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths (Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| SC-1 | From any input-blocking state (incl. persisted trap) one action returns to input-accepting; a core test builds a trapped CalcState and asserts both soft_reset() and memory_lost() restore input acceptance | VERIFIED | `hp41-core/tests/phase_67_reset.rs`: RST-01-a through RST-01-n assert every trapping field cleared; RST-04-a/b assert trapped-state serde round-trip + recovery; make_trapped_state() at line 62 builds the trapped fixture |
| SC-2 | soft_reset() clears working state + every trapping field and preserves stored user data; memory_lost() == CalcState::new() | VERIFIED | `state.rs:701-771`: soft_reset() clears stack, entry_buf, alpha_reg, alpha_mode, display_override, prgm_mode, user_mode, modals, integ/solve/difeq states, matrix_dim, is_running, pc, call_stack, phase-63/64 transients, print/event buffers, clock modes, adv transients, cancel_requested (in-place); state.rs:781-792: memory_lost() = *self = CalcState::new() + Arc preserve; test RST-03 in phase_67_reset.rs asserts JSON equality with CalcState::new() |
| SC-3 | Reset overwrites the shared autosave synchronously (relaunch loads reset state — round-trip test) | VERIFIED | `persistence.rs:328-359`: test_reset_soft_overwrites_persisted_trap — builds trapped state, saves, reloads (trap persists), soft_resets, saves same path, reloads — asserts prgm_mode/user_mode/is_running cleared; `persistence.rs:369-395`: test_reset_full_overwrites_persisted_state — parallel test for memory_lost(). GUI commands.rs:696 acquires lock, resets, persists BEFORE releasing — mutex invariant upheld |
| SC-4 | ON key wired on GUI+iOS (tap=soft, long-press=full with portaled confirm); CLI Ctrl+R opens s/f/Esc prompt (f→MEMORY LOST y/n); invoked OUTSIDE key_map.resolve()/dispatch; reset is NOT an Op | VERIFIED | `App.tsx:422-472`: handleOnPointerDown/Up/Cancel bypass handleKeyClick (which returns early for empty id); invoke('reset_soft'/'reset_full') called directly. `Keyboard.tsx:454-483`: isOnKey branch routes pointer events to dedicated props. `app.rs:609-673`: Ctrl+R intercept at line 609, pending_input routing at line 678 — reset is above modal routing. No Reset/SoftReset variant in Op enum (grep returns 0 matches). CLI Ctrl+E now handles Rdprgm (app.rs:749-750). |
| SC-5 | ADR docs/adr/v4.3-007-reset-escape-hatch.md + a docs/hp41-*-divergences.md entry record the intentional divergence from hardware ON semantics | VERIFIED | `docs/adr/v4.3-007-reset-escape-hatch.md` exists (9.4 KB, Status: Accepted, 2026-06-10). `docs/hp41cv-divergences.md:207-263`: section D-CV-10 "Reset escape hatch diverges from hardware ON semantics — Implemented (v4.3)" documents all three divergence points with rationale and implementation references |
| SC-6 | just ci + just gui-ci are green; CLI↔GUI parity (D-25.6) holds | VERIFIED | Post-CR-02-fix gate run (orchestrator, 2026-06-10): `just ci` exit 0 (95.27% regions ≥93% target; state.rs 99.03% lines; Free42 none); `just gui-ci` exit 0 (release build clean; vitest 348 passed); workspace core+cli 3714 passed; MSRV clippy clean; root fmt clean. D-25.6 parity at the behavior level: both frontends expose soft+full tiers (CLI via Ctrl+R, GUI via ON key). GUI physical-keyboard Ctrl+R→xeq_RDPRGM (App.tsx:178) is the known CR-01 deviation — pre-existing, not a Phase-67 regression, explicitly deferred as out of scope. |

**Score:** 6/6 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-core/src/state.rs` | CalcState::soft_reset() + memory_lost() | VERIFIED | Lines 701–792; soft_reset clears 24+ fields, memory_lost = *self = new() + Arc identity preserved (CR-02 fix) |
| `hp41-core/tests/phase_67_reset.rs` | Table-driven clear/preserve + trapped-recovery + memory_lost==new tests | VERIFIED | 18.6 KB; RST-01 (14 subtests), RST-02 (preserve), RST-03 (JSON equality), RST-04 (round-trip), RST-05 (Arc identity, CR-02 regression) |
| `hp41-cli/src/app.rs` | ResetPrompt enum + Ctrl+R intercept above pending_input | VERIFIED | ResetPrompt enum at line 51; intercept at line 609; pending_input routing at line 678; clear ordering confirmed |
| `hp41-gui/src-tauri/src/commands.rs` | reset_soft + reset_full Tauri commands with persist-under-lock | VERIFIED | reset_soft at line ~695; reset_full at ~711; both lock AppState, call reset, persist, release — mutex-hold ordering confirmed |
| `hp41-gui/src-tauri/permissions/reset-soft.toml` | allow-reset-soft permission | VERIFIED | File exists (176B); referenced in capabilities/default.json:31 |
| `hp41-gui/src-tauri/permissions/reset-full.toml` | allow-reset-full permission | VERIFIED | File exists (176B); referenced in capabilities/default.json:32 |
| `hp41-gui/src-tauri/capabilities/default.json` | allow-reset-soft + allow-reset-full in permissions array | VERIFIED | Lines 31-32 confirmed |
| `hp41-gui/src/App.tsx` | ON-key tap/long-press handler outside resolve() + portaled confirm sheet | VERIFIED | Lines 405-477 (handlers); line 1729 (createPortal to document.body); confirmSheetOpen state gates rendering |
| `hp41-gui/src/Keyboard.tsx` | ON key isOnKey branch routing pointer events outside handleKeyClick | VERIFIED | Lines 454-483; `if (!key.id) return` guard at line 341; isOnKey branch at line 454 |
| `hp41-gui/src/App.test.tsx` | Vitest Q1-Q4: tap=soft, long-press=sheet, confirm=full, no-double-fire | VERIFIED | Lines 1199-1390+; Q1 (tap→reset_soft), Q2 (long-press→sheet), Q3a (confirm→reset_full), Q3b (cancel→no invoke), Q4 (no-double-fire) |
| `docs/adr/v4.3-007-reset-escape-hatch.md` | ADR for reset escape hatch | VERIFIED | File exists (9.4 KB), Status: Accepted, covers both divergences, all frontends, consequences |
| `docs/hp41cv-divergences.md` | D-CV-10 divergence entry | VERIFIED | Lines 207-263; IMPLEMENTED status; all three divergence points documented |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `hp41-core/src/state.rs::soft_reset` | cancel_requested (in-place store) | `self.cancel_requested.store(false, Relaxed)` | VERIFIED | Line 769-770; does NOT replace Arc (CR-02 fix); comment at line 765-768 explains rationale |
| `hp41-core/src/state.rs::memory_lost` | CalcState::new() + Arc preserve | `let cancel = Arc::clone(&self.cancel_requested); *self = new(); self.cancel_requested = cancel` | VERIFIED | Lines 787-791; Arc identity preserved (RST-05 tests assert ptr_eq) |
| `hp41-core/tests/phase_67_reset.rs::RST-05` | Arc::ptr_eq assertion | `std::sync::Arc::ptr_eq(&before, &s.cancel_requested)` | VERIFIED | Lines 455-485; both soft_reset_preserves_cancel_arc_identity and memory_lost_preserves_cancel_arc_identity |
| `hp41-cli/src/app.rs::handle_key` (Ctrl+R intercept) | pending_input routing | Ctrl+R at line 609, pending_input check at line 678 | VERIFIED | Reset intercept is 69 lines above pending_input block; code comment at 592-600 documents intentional D-07 exception |
| `hp41-cli/src/app.rs` reset handler | `persistence::save_state` | Synchronous call at lines 627-630 (soft) and 658-663 (full) | VERIFIED | Both tiers call `persistence::save_state(&self.state_path, &self.state)` immediately after reset |
| `hp41-gui/src-tauri/src/lib.rs` | `commands::reset_soft, commands::reset_full` | `generate_handler!` registration at lines 267-268 | VERIFIED | Both commands listed in generate_handler!; CancelFlag clone from initial state at line 160 |
| `App.tsx ON-key handler` | `invoke('reset_soft') / invoke('reset_full')` | Pointer handlers bypassing handleClick/resolve | VERIFIED | handleOnPointerUp invokes reset_soft (line 447); handleConfirmFullReset invokes reset_full (line 468); neither passes through resolveKeyId |
| `App.tsx confirm sheet` | document.body | createPortal | VERIFIED | Line 1729: `createPortal(<div...>, document.body)` |

---

### Data-Flow Trace (Level 4)

Not applicable — this phase adds methods on a model struct and CLI/GUI wiring code, not a rendering pipeline for fetched data. The reset path produces `CalcStateView` from `&CalcState` via `from_state()`, which is a well-tested existing path.

---

### Behavioral Spot-Checks

| Behavior | Check | Result | Status |
|----------|-------|--------|--------|
| soft_reset() leaves no trapping fields | 14 RST-01 subtests in phase_67_reset.rs | All assert trapping fields cleared after make_trapped_state() + soft_reset() | PASS (code) |
| memory_lost() == CalcState::new() | RST-03 in phase_67_reset.rs (JSON equality) | serde_json output of state after memory_lost() must equal output of CalcState::new() | PASS (code) |
| Cancel Arc identity preserved | RST-05 (soft + full) | Arc::ptr_eq holds across both reset tiers | PASS (code) |
| CLI Ctrl+R above pending_input | Line ordering in app.rs | Reset at 609, pending_input at 678 | PASS (grep) |
| Ctrl+E now handles Rdprgm (CLI) | app.rs:749-750 | `if key.code == KeyCode::Char('e') && CONTROL → Op::Rdprgm` | PASS (grep) |
| GUI reset commands registered | lib.rs generate_handler! | reset_soft and reset_full listed | PASS (grep) |
| Reset is not an Op | grep for SoftReset/MemoryLost in mod.rs, program.rs, prgm_display.rs (both) | 0 matches | PASS (grep) |

---

### Probe Execution

No probes declared in PLAN files. Phase 67 has no `scripts/*/tests/probe-*.sh` artifacts — skipped.

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|---------|
| RESET-01 | Plans 67-01 through 67-05 | Two-tier in-app reset escape hatch: soft (clear trapping fields, preserve data) + full (factory state). Outside dispatch path, synchronous autosave overwrite, ADR + divergence doc. | SATISFIED | `[x]` checked in REQUIREMENTS.md line 42; implementation verified across all 5 plans; traceability table line 88 still reads "Planned" (minor doc tracking gap, does not affect implementation) |

**Note:** The traceability table at REQUIREMENTS.md:88 reads `| RESET-01 | Phase 67 | Planned |` while the checkbox at line 42 reads `- [x] **RESET-01**`. The table status was not updated to "Complete" when Phase 67 closed. This is a documentation inconsistency only — the implementation is fully present.

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `hp41-gui/src/App.tsx` | 178 | `case 'r': return 'xeq_RDPRGM'` | INFO (deferred) | GUI physical-keyboard Ctrl+R still dispatches RDPRGM. CR-01 from 67-REVIEW.md; explicitly deferred as pre-existing, out of scope for Phase 67 per orchestrator disposition. GUI reset is reachable via ON-key pointer events. |
| `.planning/REQUIREMENTS.md` | 88 | `RESET-01 \| Phase 67 \| Planned` | INFO | Traceability table status not updated to "Complete" after phase closed; checkbox at line 42 correctly reads `[x]`. No code impact. |

No `TBD`, `FIXME`, or `XXX` markers found in phase-modified files. No stub patterns detected in implementation code.

---

### Human Verification — RESOLVED

#### 1. Post-CR-02 Quality Gate Confirmation — ✅ RESOLVED (orchestrator, 2026-06-10)

Both gates re-run on develop after the CR-02 fix (commit 11ee2b0):
- `just ci` → exit 0. Coverage TOTAL 95.27% regions (≥93% target ✓) / 93.07% lines (= pre-Phase-67 baseline, not a regression). `state.rs` itself 99.03% lines / 99.26% regions. RST-05 tests pass. Free42 contamination: none.
- `just gui-ci` → exit 0. GUI src-tauri Rust tests pass; release build clean; vitest 348 passed / 12 files.
- Workspace core+cli: 3714 passed. MSRV clippy (`cargo +1.88 clippy … -D warnings`): clean. Root `cargo fmt --check`: clean.

#### 2. iOS / On-Device Reset Behavior — ✅ RESOLVED (user, 2026-06-10)

Approved by the user during the Plan 67-04 human-verify checkpoint (recorded in 67-04-SUMMARY as T-67-10 PASSED): ON tap = soft reset; long-press = portaled MEMORY LOST sheet (not clipped by `transform:scale`); Confirm = factory state; no double-fire on pointer-up. The CR-02 fix is **hp41-core-only** (cancel_requested Arc internals) and does not touch `App.tsx` / `Keyboard.tsx` gesture or portal behavior, so the on-device approval remains valid post-fix.

---

### Gaps Summary

No gaps. All six success criteria are code-verified; the two human-gate items are resolved (gate run by orchestrator post-fix; on-device approved by user during 67-04). Phase goal achieved.

**CR-02 fix (commit 11ee2b0):** Confirmed in code. `soft_reset()` at `state.rs:769-770` calls `self.cancel_requested.store(false, Relaxed)` — does not replace the Arc. `memory_lost()` at `state.rs:787-791` clones and restores the Arc identity across the factory reset. RST-05 regression tests at `phase_67_reset.rs:455-485` assert `Arc::ptr_eq` holds for both tiers.

**CR-01 (GUI Ctrl+R → RDPRGM):** App.tsx:178 still maps `case 'r'` to `'xeq_RDPRGM'`. This is a pre-existing mapping, explicitly deferred by the orchestrator as out of scope. GUI reset is accessible via the ON-key pointer handlers which bypass `resolveKeyId` entirely. Not a Phase-67 blocker.

---

_Verified: 2026-06-10T17:30:00Z_
_Verifier: Claude (gsd-verifier)_
