---
phase: 12
status: issues_found
files_reviewed: 11
depth: standard
findings:
  critical: 1
  warning: 0
  info: 0
  total: 1
reviewed: 2026-05-09
---

# Phase 12 Code Review — Synthetic Programming

**Files reviewed (11):** `hp41-cli/src/app.rs`, `hp41-cli/src/help_data.rs`, `hp41-cli/src/keys.rs`, `hp41-cli/src/prgm_display.rs`, `hp41-cli/src/tests/keys_tests.rs`, `hp41-cli/src/ui.rs`, `hp41-core/src/ops/mod.rs`, `hp41-core/src/ops/program.rs`, `hp41-core/src/ops/registers.rs`, `hp41-core/src/state.rs`, `hp41-core/tests/synthetic_tests.rs`

---

## Critical

### CR-01 — `Vec::insert` panic when `state.pc > state.program.len()` after ISG/DSE skip-at-end

**Confidence: 88%**
**File:** `hp41-cli/src/app.rs` — HexModal `Some(_)` arm

**Root cause:** `Vec::insert(i, val)` panics when `i > self.len()`. `state.pc` can exceed `program.len()` after `run_program()` terminates via an ISG or DSE skip that fires on the last step.

**Mechanism:** In `run_loop` (`ops/program.rs`), the loop pre-increments `pc` on every iteration AND ISG/DSE skip adds a second increment. If the last step is ISG/DSE and its skip condition fires, `state.pc` becomes `program.len() + 1` when `run_program()` returns.

**Trigger sequence:**
1. Program has ISG or DSE as its last step.
2. Skip condition fires on execution.
3. `run_program()` returns with `state.pc = program.len() + 1`.
4. User enters PRGM mode, presses `X`, types two valid hex digits (e.g. `CF`).
5. `state.program.insert(state.pc, Op::SyntheticByte(byte))` panics.

**Normal case safe:** `Vec::insert(len, val)` is valid (equivalent to `push`). Only the skip-past-end ISG/DSE path produces `pc = len + 1`.

**Fix — clamp insert position before inserting:**

```rust
Some(_) => {
    let insert_pos = self.state.pc.min(self.state.program.len());
    self.state.program.insert(insert_pos, Op::SyntheticByte(byte));
    self.state.pc = insert_pos + 1;
    self.message = None;
}
```

---

## Passing Items

**Security invariant (T-12-W2-02):** `synthetic_byte_to_op(byte)` always called before `program.insert()`. `None` branch leaves program unchanged and sets `"INVALID"` message. Byte is already constrained by the lookup — no injection possible.

**No infinite recursion:** `synthetic_byte_to_op()` never returns `Some(Op::SyntheticByte(_))` — confirmed across all 21 match arms. Single-level `dispatch(state, op)` call in `SyntheticByte` arm is safe.

**dispatch() / execute_op() parity:** All 9 Phase 12 Op variants appear in both `dispatch()` and `execute_op()`. No variant is missing from either arm.

**`#[serde(default)]` on all new CalcState fields:** `last_key_code`, `reg_m`, `reg_n`, `reg_o` all carry `#[serde(default)]`. Backward compatibility with v1.0 save files preserved.

**HexModal PRGM-mode gate:** `'X'` interceptor correctly gated on `self.state.prgm_mode`. Tests cover both branches.

**Two-char hex accumulation:** Correctly accumulates 2 hex chars before validating. Non-hexdigit keys keep modal open. Esc cancels with no side effects.

**`#![deny(clippy::unwrap_used)]` compliance:** No `.unwrap()` in `hp41-core` production code.

**LiftEffect correctness:** GetKey → Enable; Null → Neutral; StoM/N/O → Neutral; RclM/N/O → Enable.

**`prgm_display::op_display_name` exhaustiveness:** All 9 new Op variants handled correctly.

**`help_data.rs`:** `=== Synthetic Programming ===` category with 7 entries present.
