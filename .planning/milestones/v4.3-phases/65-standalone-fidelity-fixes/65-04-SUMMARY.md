---
phase: 65-standalone-fidelity-fixes
plan: "04"
subsystem: display
tags: [disp-03, aon, flag-48, alpha, cli, gui, parity]
dependency_graph:
  requires: [65-02]
  provides: [DISP-03]
  affects: [hp41-cli/src/ui.rs, hp41-gui/src-tauri/src/types.rs]
tech_stack:
  added: []
  patterns:
    - "flag-bit access: state.flags & (1u64 << 48) != 0"
    - "format_alpha(&state.alpha_reg) reused for AON branch (same as alpha_mode branch)"
key_files:
  modified:
    - hp41-cli/src/ui.rs
    - hp41-gui/src-tauri/src/types.rs
decisions:
  - "65-04-D01: AON flag-48 branch placed AFTER alpha_mode and AFTER display_override — last else-if before X fallback (D-10 ordering); alpha_mode always takes priority over AON"
  - "65-04-D02: Raw state.flags u64 used in both frontends (not the projected Vec<u8>) — consistent with the CLI pattern and display_ops.rs flag_set implementation"
  - "65-04-D03: No IPC change, no App.tsx change — AON string flows through existing display_str field; frontend App.tsx chain unchanged"
metrics:
  duration_seconds: 233
  duration_display: "~4min"
  completed_date: "2026-06-07"
  tasks_completed: 3
  files_modified: 2
---

# Phase 65 Plan 04: DISP-03 AON Flag-48 Auto-Display Summary

## One-Liner

AON (flag 48) auto-display wired into both CLI and GUI: at rest, `format_alpha(alpha_reg)` replaces X in both frontends via identical `state.flags & (1u64 << 48) != 0` guards.

## What Was Built

DISP-03 wires the HP-41 AON (Alpha ON, flag 48) auto-display behavior into both frontends with full CLI-GUI parity (D-25.6):

**CLI `hp41-cli/src/ui.rs` — `get_display_string`:**
- Added `else if st.flags & (1u64 << 48) != 0 { format_alpha(&st.alpha_reg) }` as the last else-if before the X fallback
- Sits AFTER the `alpha_mode` branch (D-10 priority ordering)
- Updated priority comment: `clock > stopwatch > entry > prgm > display_override > alpha > (flag48 ? alpha : X)`
- 3 unit tests: AON shows alpha_reg, AOFF reverts to X, display_override wins over AON

**GUI `hp41-gui/src-tauri/src/types.rs` — `from_state` display_str chain:**
- Added `else if state.flags & (1u64 << 48) != 0 { format_alpha(&state.alpha_reg) }` before X fallback
- Uses raw `state.flags` u64 (NOT the projected `Vec<u8>`) per plan spec
- No IPC field added, no App.tsx change — AON string flows through existing `display_str`
- Updated stale comment at line ~152: "DISP-01 resolved in Phase 65" (was: "DISP-01 deferred")
- 3 unit tests: AON shows alpha_reg, AOFF reverts to X, alpha_mode wins over AON

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | CLI AON flag-48 branch in get_display_string + tests | 57fd618 | hp41-cli/src/ui.rs |
| 2 | GUI from_state AON flag-48 branch + stale comment | 3b66e0b | hp41-gui/src-tauri/src/types.rs |
| 3 | DISP-03 AON unit tests for GUI from_state | 90a7909 | hp41-gui/src-tauri/src/types.rs |

## Verification

- `cargo test -p hp41-cli`: 521 passed, 4 ignored — GREEN
- `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --lib types`: 18 passed — GREEN
- `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml`: clean compile

## Success Criteria

- [x] CLI shows ALPHA register at rest when flag 48 set, X otherwise
- [x] GUI display_str shows ALPHA register at rest when flag 48 set, X otherwise
- [x] display_override still takes precedence over AON (D-06)
- [x] AOFF (flag 48 cleared) reverts both frontends to X fallback
- [x] alpha_mode branch precedes flag-48 branch in both frontends (D-10)
- [x] No IPC field added, no App.tsx change
- [x] GUI crate builds clean (cargo check)
- [x] Both frontends use identical `state.flags & (1u64 << 48) != 0` expression (D-25.6)

## Deviations from Plan

None — plan executed exactly as written.

The only deviation was a Rule 1 auto-fix: the PATTERNS.md reference to `alpha_reg = [u8; 6]` is incorrect (it refers to the `HpValue::Alpha` register type, not `CalcState.alpha_reg` which is a `String`). Tests were corrected immediately on the first compile error.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. AON display is read-only access to an existing core-owned flag bit and ALPHA register string. T-65-05 (accept) and T-65-06 (mitigate — parity asserted by Task 3 tests) handled per threat register.

## Known Stubs

None — display chain is fully wired.

## Self-Check: PASSED

Files exist:
- hp41-cli/src/ui.rs — modified
- hp41-gui/src-tauri/src/types.rs — modified

Commits exist:
- 57fd618: feat(65-04): CLI AON flag-48 branch in get_display_string + tests
- 3b66e0b: feat(65-04): GUI from_state AON flag-48 branch in display_str chain
- 90a7909: test(65-04): DISP-03 AON flag-48 unit tests for GUI from_state
