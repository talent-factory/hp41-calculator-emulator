---
phase: 65-standalone-fidelity-fixes
plan: "02"
subsystem: cli-display
tags: [DISP-01, display_override, cli-gui-parity, comment-cleanup]
dependency_graph:
  requires: []
  provides: [DISP-01]
  affects: [hp41-cli/src/ui.rs, hp41-core/src/state.rs]
tech_stack:
  added: []
  patterns: [priority-chain-display, core-governed-dismissal]
key_files:
  created: []
  modified:
    - hp41-cli/src/ui.rs
    - hp41-core/src/state.rs
decisions:
  - "D-06: display_override inserted between prgm_mode and alpha_mode in get_display_string priority chain"
  - "D-05: no CLI-side clear; dismissal remains core-governed (CLD / next overwrite / entry start)"
  - "D-11: stale DISP-01-deferred comments replaced with Phase-65-resolved text"
metrics:
  duration: "12 min"
  completed: "2026-06-07"
  tasks_completed: 2
  tasks_total: 2
---

# Phase 65 Plan 02: DISP-01 CLI display_override + D-11 stale comment cleanup Summary

**One-liner:** CLI `get_display_string` now renders `state.display_override` (VIEW/AVIEW/PROMPT output) at the D-06 priority slot, matching GUI behavior; two stale "DISP-01 deferred to v4.4" comments in `state.rs` removed.

## What Was Built

### Task 1 — display_override branch in get_display_string

Inserted an `else if let Some(ref s) = st.display_override { s.clone() }` arm in
`hp41-cli/src/ui.rs::get_display_string` between the `prgm_mode` branch and the
`alpha_mode` branch. This yields the documented D-06 priority chain:

```
clock > stopwatch > entry_buf > prgm_mode > display_override > alpha_mode > X
```

The doc-comment priority line was updated to include `display_override`. No
CLI-side clear logic was added — dismissal is core-governed per D-05 (CLD, next
overwrite, or start of keyboard entry).

**Commit:** `ee8c882`

### Task 2 — Integration tests + D-11 stale comment cleanup

Added two unit tests in `hp41-cli/src/ui.rs`:

- `test_display_override_renders` — sets `app.state.display_override = Some("R01 1.000".into())`,
  asserts `get_display_string` returns `"R01 1.000"`.
- `test_display_override_none_falls_through` — leaves `display_override = None`, asserts a
  non-empty string is returned (X fallback path).

Resolved D-11 stale comments in `hp41-core/src/state.rs`:

- Line ~80 (`YieldState` doc): `"DISP-01 deferred"` → `"DISP-01 resolved in Phase 65"`
- Line ~531 (`pending_yield` field doc): `"DISP-01 stays deferred to v4.4"` → `"DISP-01 resolved in Phase 65"`

**Commit:** `6d1eeaf`

## Commits

| Task | Commit | Type | Message |
|------|--------|------|---------|
| 1 | `ee8c882` | feat | insert display_override branch in get_display_string (DISP-01) |
| 2 | `6d1eeaf` | test | add display_override integration tests + resolve D-11 stale comments |

## Verification

- `cargo build -p hp41-cli`: clean
- `cargo test -p hp41-cli`: 501 passed, 4 ignored
- `cargo test --workspace`: 3557 passed, 6 ignored
- `grep "deferred to v4.4" hp41-core/src/state.rs`: 0 matches

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — display_override is a live field written by op_view/op_aview/op_prompt in `display_ops.rs`.

## Threat Flags

None — frontend-only read of an existing core-owned string field; no new network, auth, or trust-boundary surface introduced.

## Self-Check: PASSED

- `hp41-cli/src/ui.rs` modified: FOUND
- `hp41-core/src/state.rs` modified: FOUND
- Commit `ee8c882` exists: FOUND
- Commit `6d1eeaf` exists: FOUND
- `deferred to v4.4` in state.rs: 0 matches (PASS)
- `cargo test --workspace`: 3557 passed (PASS)
