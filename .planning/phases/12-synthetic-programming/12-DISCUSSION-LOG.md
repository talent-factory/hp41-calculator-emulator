# Phase 12: Synthetic Programming - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-08
**Phase:** 12-synthetic-programming
**Areas discussed:** GETKEY capture mechanics, Hidden registers M/N/O storage, Hex-byte safe subset scope, Hex-byte modal keyboard binding

---

## GETKEY Capture Mechanics

### Where should 'last key code' live?

| Option | Description | Selected |
|--------|-------------|----------|
| CalcState field | last_key_code: u8 on CalcState with #[serde(default)]. Persisted across save/load — consistent with all other calculator state. | ✓ |
| App field (ephemeral) | last_key_code: u8 on App. Not persisted. GETKEY injects it into CalcState before dispatch. | |

**User's choice:** CalcState field

---

### What HP-41 key encoding should GETKEY use?

| Option | Description | Selected |
|--------|-------------|----------|
| HP-41 row-column codes | key code = row×10 + col (e.g., ENTER=81, R/S=74). Faithful to hardware. Requires lookup table crossterm→HP-41 code. | ✓ |
| Arbitrary sequential codes | Own numbering, no lookup table. Easier but not hardware-faithful. | |

**User's choice:** HP-41 row-column codes

---

### What should GETKEY push when no key pressed yet?

| Option | Description | Selected |
|--------|-------------|----------|
| Push 0 | Hardware-faithful: HP-41 returns 0 when no key pressed. LiftEffect::Enable. | ✓ |
| Push -1 or error | Sentinel value, detectable in programs. Less hardware-faithful. | |
| You decide | Claude implements push 0 as hardware-faithful default. | |

**User's choice:** Push 0

---

## Hidden Registers M/N/O Storage

### Where should hidden registers M, N, O be stored?

| Option | Description | Selected |
|--------|-------------|----------|
| Separate named fields on CalcState | reg_m, reg_n, reg_o: HpNum with #[serde(default)]. Explicit, matches HP-41 docs. | ✓ |
| Extend regs Vec beyond R99 (indices 100/101/102) | Reuses existing Vec. Conceptually confusing — mixes numbered and hidden registers. | |
| Separate BTreeMap<char, HpNum> | Extensible but overkill for exactly 3 registers. | |

**User's choice:** Separate named fields on CalcState

---

### How should STO M / RCL M be entered via keyboard?

| Option | Description | Selected |
|--------|-------------|----------|
| Extend S/R modals to accept M/N/O | After 'S' or 'R', user types M/N/O → immediate dispatch. Consistent UX. | ✓ |
| Dedicated new key sequence | Separate prefix key. Less discoverable, more complex. | |

**User's choice:** Extend existing S/R modals

---

## Hex-Byte Safe Subset Scope

### What defines the 'curated safe subset'?

| Option | Description | Selected |
|--------|-------------|----------|
| Only codes mapping to implemented Ops | Lookup table of ~50 FOCAL codes that correspond to existing Op variants. Everything else rejected. | ✓ |
| Hand-curated allowlist with extras | Include some well-known harmless codes even if currently no-ops. | |
| You decide the table boundary | Claude defines the exact allowlist. | |

**User's choice:** Only codes that map to already-implemented Ops

---

### How should a rejected hex code be reported?

| Option | Description | Selected |
|--------|-------------|----------|
| "INVALID" in display + no insertion | app.message = Some("INVALID"), program unchanged, modal closes. | ✓ |
| Stay in modal, prompt re-entry | Keep modal open on invalid. Inconsistent with existing modal pattern. | |

**User's choice:** "INVALID" in display area + no insertion

---

### What Op variant represents an inserted synthetic byte?

| Option | Description | Selected |
|--------|-------------|----------|
| Op::SyntheticByte(u8) — raw hex code | Stores raw byte, re-looks up during execution. Preserves synthetic identity for display. | ✓ |
| Resolve to target Op at insertion time | Insert Op directly. Simpler but loses synthetic step identity. | |

**User's choice:** Op::SyntheticByte(u8)

---

## Hex-Byte Modal Keyboard Binding

### What key opens the hex-byte insertion modal?

| Option | Description | Selected |
|--------|-------------|----------|
| Uppercase 'X' (Shift+X) | 'x' lowercase is XY-swap; 'X' uppercase is free. Mnemonic: X for hex. | ✓ |
| F6 function key | F6 free, no mnemonic. | |
| Uppercase 'N' (Shift+N) | Free, mnemonic: iNsert byte. Less intuitive than X. | |

**User's choice:** Uppercase 'X'

---

### How should the hex-byte modal work step by step?

| Option | Description | Selected |
|--------|-------------|----------|
| 2-digit hex entry like STO register modal | X → HEX: _; first digit → HEX: n_; second digit → validate + insert or reject. Mirrors existing pattern exactly. | ✓ |
| Single-prompt with Enter to confirm | More complex state machine. Allows backspace. Inconsistent with existing modals. | |

**User's choice:** 2-digit hex entry (mirrors STO [nn] pattern)

---

### Where in the program is the synthetic byte inserted?

| Option | Description | Selected |
|--------|-------------|----------|
| At current pc position (insert before current step) | Consistent with HP-41 PRGM mode. Shifts existing steps down. pc advances after insertion. | ✓ |
| At end of program | Always append. Simpler but not HP-41 faithful. | |

**User's choice:** At current pc position

---

## Claude's Discretion

- Exact HP-41 key code lookup table contents (crossterm KeyCode → HP-41 row-column code). Claude defines from HP-41 documentation.
- Whether last_key_code update happens before or after flush_entry_buf() in handle_key(). Claude picks correct order.
- Exact hex subset lookup table (which FOCAL codes map to which Ops).
- Program listing display format for Op::SyntheticByte: "SYN nn" format — Claude picks readable format.

## Deferred Ideas

- SYNT-05: Full FOCAL byte-code table (~200 codes) — v2+
- SYNT-06: Interactive GETKEY (program pauses for key press) — requires event loop redesign, v2+
- Indirect addressing for M/N/O registers — v1.2+
- PRGM mode step navigation (browse/delete steps) — separate phase
