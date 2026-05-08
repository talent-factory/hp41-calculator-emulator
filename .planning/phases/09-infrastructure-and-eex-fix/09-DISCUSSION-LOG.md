# Phase 9: Infrastructure & EEX Fix - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-08
**Phase:** 9-Infrastructure & EEX Fix
**Areas discussed:** Exponent display format, Two-digit exponent cap, MSRV enforcement in CI

---

## Exponent Display Format

| Option | Description | Selected |
|--------|-------------|----------|
| `1.5E_ _` | Uppercase E + space + two underscores for exponent slots. Roadmap example. | ✓ |
| `1.5E` | Uppercase E only, no slot placeholders. Simpler to render. | |
| `1.5e__` | Lowercase e + two underscores, compact. | |

**User's choice:** `1.5E_ _` — HP-41 hardware feel with two explicit slot indicators

---

| Option | Description | Selected |
|--------|-------------|----------|
| `1.5E2_` | First slot filled, second underscore. Consistent with two-slot format. | ✓ |
| `1.5E 2` | Space + digit, right-justified. Matches HP-41 hardware right-to-left exponent entry. | |
| `1.5e2` | Raw entry_buf as-is. No transformation. | |

**User's choice:** `1.5E2_` — first digit fills left slot, second remains underscore until typed

---

| Option | Description | Selected |
|--------|-------------|----------|
| `1E_ _` | Consistent with `1.5E_ _` pattern. | ✓ |
| `1   _` | HP-41 hardware exact format with space padding. | |
| You decide | Either format fine — pick simplest to implement. | |

**User's choice:** `1E_ _` — consistent with the general format, simpler than hardware-exact padding

---

## Two-Digit Exponent Cap

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — cap at 2 digits | HP-41 hardware blocks 3rd digit. Enforces faithful behavior, prevents malformed entry_buf. | ✓ |
| No cap | Allow any digits; let rust_decimal clamp. Less faithful but simpler. | |

**User's choice:** Cap at 2 digits — HP-41 hardware fidelity

**Feedback modality:** Silent block — consistent with existing guards (no beep, no error message)

---

## MSRV Enforcement in CI

| Option | Description | Selected |
|--------|-------------|----------|
| Add MSRV CI job | Installs Rust 1.85 and runs `just ci`. Prevents drift from declared MSRV. | ✓ |
| Cargo.toml only | Just set rust-version field, CI stays on stable. Less enforcement. | |

**User's choice:** Add MSRV CI job — real enforcement, not just an aspirational declaration

---

## Claude's Discretion

- **Trailing-e normalization location:** Appending "00" in `flush_entry_buf()` before the parse chain (vs. in the display layer or guard layer) — chosen as the minimal correct fix at the right abstraction level
- **MSRV job placement:** Run as parallel sibling to `test` job (no `needs:` dependency) so it doesn't block existing CI signals
- **Exponent display helper:** Extract a `format_entry_buf_display()` function from `get_display_string()` for clarity

## Deferred Ideas

None — discussion stayed within phase scope.
