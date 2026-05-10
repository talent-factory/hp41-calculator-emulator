# Phase 14: IPC Layer - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-09
**Phase:** 14-IPC Layer
**Areas discussed:** CalcStateView fields, Key ID convention, Modal state ownership, Error response shape

---

## CalcStateView Fields

### Stack registers to expose

| Option | Description | Selected |
|--------|-------------|----------|
| X only | display_str + x_str cover Phase 15 needs; Y/Z/T/LASTX deferred | ✓ |
| All five (X/Y/Z/T/LASTX) | Future-proofs for Phase 15 stack panel; slightly more bytes | |
| display_str only | 12-char string only; no x_str | |

**User's choice:** X only

### Additional fields beyond X

| Option | Description | Selected |
|--------|-------------|----------|
| display_str + annunciators + print_lines | Covers Phase 15 needs exactly; ~200 bytes | ✓ |
| Add entry_buf and angle_mode | Useful for partial-entry rendering edge cases | |

**User's choice:** display_str + annunciators + print_lines

**Notes:** Final CalcStateView shape: `{ display_str, x_str, annunciators: { user, prgm, alpha, rad, grad }, print_lines }`.

---

## Key ID Convention

### Digit key format

| Option | Description | Selected |
|--------|-------------|----------|
| Bare digits: "0"–"9", ".", "e" | Mirrors CLI char dispatch; no prefix | ✓ |
| Prefixed: "digit_0"–"digit_9" | Explicit prefix; more verbose | |

**User's choice:** Bare digits

### Parameterized op format

| Option | Description | Selected |
|--------|-------------|----------|
| Compound key IDs: "sto_05", "fix_4" | Single string; frontend constructs after modal; no extra IPC fields | ✓ |
| key_id + arg field: {key: "sto", arg: "05"} | More structured; changes dispatch_op signature | |

**User's choice:** Compound key IDs

**Notes:** Full naming scheme: bare chars for digits, snake_case for named ops, `prefix_param` for parameterized. STO arithmetic: `"sto_arith_plus_05"`, `"sto_arith_minus_y"`.

---

## Modal State Ownership

| Option | Description | Selected |
|--------|-------------|----------|
| Frontend owns modal state | key_map.rs atomic only; frontend sequences S→op→reg; Phase 14 is stateless IPC | ✓ |
| Backend owns modal state | PendingModal enum in AppState; replicates CLI modal logic in Rust | |

**User's choice:** Frontend owns modal state

**Notes:** Phase 14 stays purely stateless. Modal UI logic deferred to Phase 15 React layer.

---

## Error Response Shape

### Return type

| Option | Description | Selected |
|--------|-------------|----------|
| Result<CalcStateView, GuiError> | Natural Tauri v2 pattern; frontend invoke().catch() handles errors | ✓ |
| Always CalcStateView with error field | No Promise rejection; non-idiomatic for Tauri | |

**User's choice:** Result<CalcStateView, GuiError>

### GuiError fields

| Option | Description | Selected |
|--------|-------------|----------|
| message: String only | Simple struct; covers unknown key + HpError forwarding | ✓ |
| message + kind enum | Frontend can handle errors by type; more structure | |

**User's choice:** message: String only

---

## Claude's Discretion

- `x_str` format: use `state.stack.x.to_string()` or the formatted display path — Claude decides what's most useful for Phase 15 rendering
- Capabilities permission identifiers exact syntax in `capabilities/default.json` (Tauri v2 plugin permission format)

## Deferred Ideas

- Y/Z/T/LASTX in CalcStateView — Phase 15 when stack panel is implemented
- TypeScript type generation — Phase 15 frontend consumption
- Stack panel rendering — Phase 15
- Physical keyboard wiring — Phase 15
