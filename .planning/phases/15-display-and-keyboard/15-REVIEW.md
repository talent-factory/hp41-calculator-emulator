---
status: warnings
phase: 15
reviewed_by: claude-sonnet-4-6
review_date: 2026-05-10
files_reviewed: 6
findings:
  critical: 0
  warning: 1
  info: 0
  total: 1
---

# Phase 15 — Display & Keyboard: Code Review

## Summary

Six files reviewed against CLAUDE.md project policies, PATTERNS.md, RESEARCH.md, and phase
decisions D-01 through D-12. No critical issues found. One warning issued for stale module-level
doc comments.

---

## Warnings

### W-01 — Stale "Phase 14" module doc comment (confidence: 85)

Both `commands.rs` and `types.rs` have `//! Phase 14 IPC Layer —` as their opening module doc
line. Both files were substantively modified in Phase 15 (new struct fields, new handle_op branch,
new tests). The stale header misleads future readers.

**File:** `hp41-gui/src-tauri/src/commands.rs`, line 1
**File:** `hp41-gui/src-tauri/src/types.rs`, line 1

**Fix:** Update the first line in each file:
- `commands.rs`: `//! Phase 14/15 IPC Layer — Tauri command handlers.`
- `types.rs`: `//! Phase 14/15 IPC Layer — DTO types sent from Tauri commands to the React frontend.`

---

## Confirmed Correct

### commands.rs

- Zero-panic policy: no `.unwrap()` in production paths. Both `state.lock()` calls use
  `unwrap_or_else(|e| e.into_inner())` for poisoned-lock recovery. The `.expect()` on line 74 is
  unreachable in practice (guarded by the `matches!` digit block above it).
- `eex_chs` branch appears before `key_map::resolve()`. No `Op::EexChs` variant exists; the
  placement is required and correct.
- Exponent digit guard (INPUT-02): correctly counts only `is_ascii_digit()` chars after 'e',
  so a leading '-' in the exponent is not counted toward the digit limit.
- `eex_chs` toggle logic: `find('e')` returns a valid byte index; `remove()`/`insert()` at
  `e_pos + 1` is always a valid char boundary because 'e' is ASCII.
- All four early-return branches drain `print_buffer` before constructing `CalcStateView`.
- Test module: `#[allow(clippy::unwrap_used)]` present; all four required tests exist.

### types.rs

- `CalcStateView` includes all Phase 15 fields: `y_str`, `z_str`, `t_str`, `lastx_str` (String),
  `in_eex_mode` (bool). Matches D-01 and D-02 exactly.
- `from_state()` uses `format_hpnum(&state.stack.{y,z,t,lastx}, &state.display_mode)` — correct.
- `in_eex_mode = state.entry_buf.contains('e')` matches D-02 exactly.
- Estimated JSON payload for `CalcState::new()`: ~236 bytes. The 300-byte assertion passes.

### App.tsx

- TypeScript `CalcStateView` interface uses snake_case field names matching Rust serde output.
- `invoke('dispatch_op', { keyId })`: Tauri v2 converts camelCase JS key to snake_case Rust
  parameter `key_id` automatically (verified in RESEARCH.md Pitfall 4).
- `e.repeat` guard precedes all other work — correct SC-4 fix.
- `useCallback([calcState])` + `useEffect([handleKey])` with cleanup: StrictMode-safe; 'n' key
  reads latest `in_eex_mode` via the dependency.
- `resolveKeyId` routes 'n' to 'eex_chs'/'chs' based on `state?.in_eex_mode` (D-06).
- Modal-trigger keys SRfFPX return null with no invoke (D-05).
- '-' maps to 'minus' (Op::Sub): consistent with hp41-cli keys.rs; EEX exponent negation uses
  'n' → 'eex_chs' per D-06.
- `<div id="keyboard-area" />` placeholder present for Phase 16.

### App.css

- All required CSS classes present and correctly structured.
- Annunciator opacity: inactive 0.35, active 1.0 — satisfies D-09.
- `white-space: pre` on `.display` preserves HP-41 leading-space formatting.
- `user-select: none` on `.annunciator` and `.stack-label`.

### index.css

- Tailwind `@import "tailwindcss"` removed; replaced with minimal box-sizing reset (D-10).

### vite.config.ts

- `tailwindcss()` plugin removed; only `react()` remains (D-10).
- Tauri v2 dev server configuration correct (`TAURI_DEV_HOST` env var).
