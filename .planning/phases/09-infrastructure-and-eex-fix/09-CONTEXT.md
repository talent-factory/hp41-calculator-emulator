# Phase 9: Infrastructure & EEX Fix - Context

**Gathered:** 2026-05-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Two independent streams delivered together:

**Stream A — Infrastructure (INFRA-01):** Formally declare MSRV 1.85 in `Cargo.toml`, bump `rust_decimal` 1.41 → 1.42, and add an MSRV CI job that verifies `just ci` passes on Rust 1.85 toolchain.

**Stream B — EEX entry behavioral fix (INPUT-01/02/03):** Correct EEX entry to match HP-41 hardware behavior:
1. Trailing-e commit: `1.5e` + ENTER pushes `1.5` (exponent treated as 00), not a silent discard
2. Empty-buffer EEX: pressing EEX with nothing typed inserts implicit mantissa "1" and enters exponent mode
3. TUI exponent placeholder: display shows `1.5E_ _` / `1.5E2_` / `1E_ _` during partial exponent entry

No new operations or features. No display mode changes. No behavior changes to any other key.

</domain>

<decisions>
## Implementation Decisions

### Exponent Display Format (INPUT-03)
- **D-01:** When `entry_buf = "1.5e"` (trailing e, no exponent digits) → TUI display shows `1.5E_ _` (uppercase E, space, two underscore placeholders for the two exponent digit slots)
- **D-02:** When `entry_buf = "1.5e2"` (one exponent digit typed) → TUI display shows `1.5E2_` (first slot filled, second underscore for pending digit)
- **D-03:** When `entry_buf = "1.5e23"` (two exponent digits, fully entered) → TUI display shows `1.5E23` (no underscores — both slots filled)
- **D-04:** When `entry_buf = "1e"` (empty-buffer EEX → implicit "1") → TUI display shows `1E_ _`
- **Implementation:** Transform `entry_buf` in `get_display_string()` (`hp41-cli/src/ui.rs`). When `entry_buf` contains 'e': split at 'e', uppercase the mantissa part + 'E', then render 0/1/2 exponent digits already typed + underscore placeholders for the remaining slots up to 2.

### Two-Digit Exponent Cap (HP-41 hardware fidelity)
- **D-05:** Cap exponent entry at 2 digits — `handle_key()` silently blocks a 3rd digit after 'e' in `entry_buf`
- **D-06:** Silent block — no visual feedback or error message (consistent with how existing guards handle duplicate '.' and duplicate 'e' — they are silently ignored)
- **Guard logic:** Count digits in `entry_buf` after 'e'; if count ≥ 2, return without appending the new digit

### EEX Entry Guards (INPUT-01 and INPUT-02)
- **D-07:** Remove the guard that blocks EEX when `entry_buf` is empty. Instead: insert "1" as the implicit mantissa, making `entry_buf = "1e"`. This matches HP-41 hardware behavior.
- **D-08:** Keep the guard that blocks EEX when `entry_buf` already contains 'e' (prevents double-EEX).
- **D-09:** In `flush_entry_buf()`: if string ends with 'e' (trailing-e, no exponent digits), normalize by appending "00" before parsing. This makes `flush_entry_buf("1.5e")` commit as 1.5 (exponent 00) instead of returning `Err`. The existing `from_scientific()` fallback then handles "1.5e00" correctly.
- **D-10:** Invert the test `test_flush_trailing_e_without_exponent_returns_err` (`hp41-core/src/ops/mod.rs:395`) — it currently asserts `Err` but must now assert `Ok(())` with X = 1.5.

### MSRV Enforcement (INFRA-01)
- **D-11:** Add `rust-version = "1.85"` field to the workspace `Cargo.toml` (root-level)
- **D-12:** Bump `rust_decimal = "1.41"` → `"1.42"` in workspace `Cargo.toml`
- **D-13:** Add an MSRV CI job to `.github/workflows/ci.yml` using `dtolnay/rust-toolchain@1.85` that runs `just ci`. This verifies the declared MSRV is real, not just aspirational.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap
- `.planning/ROADMAP.md` — Phase 9 goal, success criteria (§ Phase 9: Infrastructure & EEX Fix)
- `.planning/REQUIREMENTS.md` — INFRA-01, INPUT-01, INPUT-02, INPUT-03 (full acceptance criteria)

### Core Implementation Files
- `hp41-core/src/ops/mod.rs` — `flush_entry_buf()` (line 207) and `flush_eex_tests` test module (line 356). The trailing-e normalization goes here (D-09). The inverted test is at line 395 (D-10).
- `hp41-cli/src/app.rs` — `handle_key()` with EEX guards (lines 286–290). Guard changes for D-07 and 2-digit cap for D-05/D-06 go here.
- `hp41-cli/src/ui.rs` — `get_display_string()` (line 129). Exponent placeholder rendering logic (D-01 through D-04) goes here.

### Infrastructure Files
- `Cargo.toml` — workspace root manifest. Add `rust-version`, bump `rust_decimal` (D-11, D-12)
- `.github/workflows/ci.yml` — CI matrix. Add MSRV job (D-13)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `flush_entry_buf()` at `hp41-core/src/ops/mod.rs:207` — already has `.or_else(|_| Decimal::from_scientific(&s))` fallback; just needs trailing-'e' normalization prepended before the parse chain
- `get_display_string()` at `hp41-cli/src/ui.rs:129` — currently returns `st.entry_buf.clone()` raw for the entry case; needs a helper that detects 'e' in entry_buf and applies the E_ _ formatting
- `handle_key()` in `hp41-cli/src/app.rs` — digit/EEX/dot dispatch at line 266; the EEX guard block is at line 286–290 (remove the "is_empty()" condition, add the 2-digit exponent cap guard)

### Established Patterns
- All `handle_key()` guards are silent (no message, no beep) — D-06 follows this pattern exactly
- `#![deny(clippy::unwrap_used)]` is active in `hp41-core` — all new test code must carry `#[allow(clippy::unwrap_used)]`
- Existing CI jobs use `dtolnay/rust-toolchain@stable` — the MSRV job uses the same action with `@1.85` pinned version

### Integration Points
- `flush_entry_buf()` is called by `dispatch()` (`hp41-core/src/ops/mod.rs:234`) before every non-digit op — the trailing-e normalization happens transparently here
- `get_display_string()` is called by `render_display()` (`hp41-cli/src/ui.rs:115`) on every frame — the exponent formatting is a pure string transform with no state changes
- The `from_scientific()` fallback added in Phase 8 already handles "1.5e00" → 1.5 correctly; D-09 just ensures the string reaches it in parseable form

</code_context>

<specifics>
## Specific Ideas

- **Exponent placeholder rendering:** Split `entry_buf` at 'e', take the mantissa (everything before 'e'), take the exponent digits (everything after 'e', 0–2 chars), then format as `{mantissa}E{digit_or_underscore}{digit_or_underscore}`. Keep the mantissa part as-is (don't reformat numbers) — just append `E` and the slot indicators.
- **HP-41 hardware reference:** The `1.5E_ _` format (uppercase E, space between E and underscores) was chosen to match HP-41 hardware display conventions where the exponent appears as a 2-char right-aligned block to the right of `E`.
- **MSRV job placement in CI:** Add after the `coverage` job (or as a parallel sibling to `test`) so it doesn't block existing green signals. Use `needs: []` (no dependency) to run in parallel with other jobs.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 9-Infrastructure & EEX Fix*
*Context gathered: 2026-05-08*
