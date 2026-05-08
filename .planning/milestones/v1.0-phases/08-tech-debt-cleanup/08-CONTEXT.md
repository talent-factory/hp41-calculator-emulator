# Phase 8: Tech Debt Cleanup - Context

**Gathered:** 2026-05-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Close four keyboard coverage gaps and one EEX entry bug identified by the v1.0 milestone audit. No new features — only fixing non-functional or misreported existing capabilities.

Deliverables:
1. EEX key ('e') functional: user can enter scientific notation numbers interactively
2. SIN accessible via keyboard (new binding: 'q')
3. CLREG accessible via keyboard (new binding: 'g')
4. AlphaClear accessible via keyboard (Delete key in ALPHA mode)
5. help_data.rs corrected: 'q' → SIN, 'S' → STO (modal), 'g' → CLREG documented

</domain>

<decisions>
## Implementation Decisions

### Key Assignments
- `'q'` → `Op::Sin` — add to `key_to_op()` in `hp41-cli/src/keys.rs`
- `'g'` → `Op::Clreg` — add to `key_to_op()` in `hp41-cli/src/keys.rs`
- `Delete` key in ALPHA mode → `Op::AlphaClear` — add to `handle_alpha_mode_key()` in `hp41-cli/src/app.rs`

### EEX Entry Fix
- In `flush_entry_buf()` (`hp41-core/src/ops/mod.rs`): add `.or_else(|_| Decimal::from_scientific(&s))` fallback after the primary `Decimal::from_str()` parse fails
- Add entry_buf guards in `handle_key()` (`hp41-cli/src/app.rs`): block 'e' if entry_buf is empty, block 'e' if 'e' already present, block '.' if '.' already present or 'e' already present
- No changes to the display format — EEX values display in standard format after entry

### STO Arithmetic (Deferred)
- `PendingInput::StoAdd/StoSub/StoMul/StoDiv` variants remain as-is (dead code) for now
- STO arithmetic keyboard modals deferred to v1.1 — multi-step modal flow is out of scope for a cleanup phase
- Remove the `#[allow(dead_code)]` comment and update the comment to say "deferred to v1.1"

### help_data.rs Corrections
- Fix `("S", "SIN", ...)` → `("q", "SIN", "Sine of X (in current angle mode)")`
- Add `("g", "CLREG", "Clear all storage registers R00-R99")`
- Keep `("S", "STO [nn] (modal register entry)", ...)` as-is (already documented in keys.rs KEY_REF_TABLE)
- The help_data.rs is the SINGLE SOURCE OF TRUTH — update it precisely

### Claude's Discretion
- Test coverage: each new key binding should have at least one unit test in `hp41-cli/src/tests/keys_tests.rs` or the inline test module
- The `#![deny(clippy::unwrap_used)]` crate-level deny attribute is active — any test code using `.unwrap()` must carry `#[allow(clippy::unwrap_used)]`

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `hp41-cli/src/keys.rs::key_to_op()` — match on `KeyCode::Char` to add 'q' and 'g' bindings
- `hp41-cli/src/app.rs::handle_alpha_mode_key()` — matches key events in ALPHA mode; add `KeyCode::Delete` branch calling `Op::AlphaClear`
- `hp41-core/src/ops/mod.rs::flush_entry_buf()` — line 213: primary parse point to add scientific notation fallback
- `hp41-cli/src/help_data.rs` — the KEY_CATEGORIES array to correct SIN entry and add CLREG

### Established Patterns
- All `key_to_op()` additions follow `KeyCode::Char('X') => Some(Op::Variant)` pattern
- entry_buf guards follow the existing pattern at app.rs:282: `if c.is_ascii_digit() || c == '.' || c == 'e'`
- Test modules carry `#[allow(clippy::unwrap_used)]` when using `.unwrap()` for conciseness

### Integration Points
- `Op::Sin`, `Op::Clreg`, `Op::AlphaClear` all exist in dispatch() already — no dispatch changes needed
- `Decimal::from_scientific()` is available via `rust_decimal::Decimal` — no new dependency needed

</code_context>

<specifics>
## Specific Ideas

- EEX fix: add the `from_scientific` fallback in `flush_entry_buf` — this is the minimal correct fix
- Key assignments: use the exact free keys identified (q, g, Delete-in-ALPHA) — no deviation
- help_data: fix the stale `("S", "SIN", ...)` entry immediately — it misleads users consulting `?`

</specifics>

<deferred>
## Deferred Ideas

- STO arithmetic keyboard modals (PendingInput::StoAdd/Sub/Mul/Div) — complex multi-step modal flow, deferred to v1.1
- EEX scientific notation display (showing mantissa+exponent in the TUI) — out of scope, deferred
- CLREG confirmation prompt — HP-41 hardware has none; keep it silent for HP-41 fidelity

</deferred>
