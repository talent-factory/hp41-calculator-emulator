---
phase: 65
slug: standalone-fidelity-fixes
status: pending
created: 2026-06-07
source: 65-VERIFICATION.md (status human_needed — automated 4/4 passed)
---

# Phase 65 — Human Verification (UAT)

Automated verification passed **4/4 must-haves** (3588 workspace tests green; CR-01 silent-overflow fix confirmed). The items below are **live visual/interaction confirmations** layered on top of the automated unit + integration coverage — the behavior is tested, but the end-to-end TUI/GUI render after real keypresses can only be eyeballed.

Run the apps with `just` and confirm each:

| # | Requirement | What to check | How |
|---|-------------|---------------|-----|
| 1 | DISP-01 | After `VIEW`/`AVIEW`/`PROMPT`, the CLI main display line shows the override (register name + value / ALPHA string), matching the GUI; clears per core semantics (CLD / next VIEW / number entry). | Run the CLI (`hp41`), execute a VIEW, watch the display line refresh after the keypress. |
| 2 | DISP-02 | Typing a mantissa then `CHS` flips the sign **in place** (`3.14` → `-3.14` → `3.14`) with no stack lift, no overwrite, no flush; empty-buffer `CHS` still negates X; EEX (`1e2`) CHS still toggles the exponent sign. | CLI: type `3.14`, press CHS twice; type a value, ENTER, type another, CHS. |
| 3 | DISP-03 | With `AON` (flag 48) set, the ALPHA register auto-displays after **every** operation on **both** CLI and GUI; `AOFF` reverts to X. A fresh VIEW still wins over AON. | Set AON, run several ops, observe ALPHA shown at rest; AOFF; repeat in GUI. |
| 4 | MATH-01 | `FACT(27)`…`FACT(69)` show correct 10-sig scientific-notation values (e.g. FACT(69) ≈ 1.711224524E98); `FACT(70)` → OutOfRange; non-integer/negative → Domain. | Compare a few against the HP-41 Owner's Manual table for hardware-fidelity confidence. |

When satisfied, run `/gsd:verify-work 65` to mark these resolved (or report any issue for gap-closure).
