# Phase 22: Program Control & Memory Ops — Pattern Map

**Mapped:** 2026-05-14
**Files analyzed:** 13 (10 modified / 5 new — incl. 5 new integration test files; some `program_edit.rs` is optional)
**Analogs found:** 13 / 13 (every file has a strong existing analog inside the workspace)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-core/src/ops/mod.rs` (Op enum + 13 new variants + 13 dispatch arms) | controller (dispatch) | request-response | self — Phase 21 additions at lines 570–589 (SfFlag, CfFlag, View, Beep, Tone) | exact (self-pattern continuation) |
| `hp41-core/src/ops/program.rs::resume_program()` (NEW fn) | service (interpreter entry) | event-driven (control flow) | `run_program()` at `program.rs:139–169` | exact (same shape, skip entry-label search) |
| `hp41-core/src/ops/program.rs` run_loop new arms (`Op::Stop`, `Op::GtoInd`, `Op::XeqInd`) | service (interpreter step) | event-driven | `Op::Prompt` arm `program.rs:272–275` (Stop); `Op::Gto` arm `:201–204` (GtoInd); `Op::Xeq` arm `:205–230` (XeqInd) | exact |
| `hp41-core/src/ops/program.rs` execute_op new arms (`Op::Pse`, `Op::Size`, `Op::Cla`, `Op::Clst`, `Op::Pack`, `Op::Catalog`, `Op::Asn`) + catch-all extension | service | request-response | `Op::AlphaClear` at `:387` (Cla wraps op_alpha_clear); `Op::Beep`/`Op::Tone` at `:452–453` (Pse pattern — execute_op delegation); programming-ops catch-all `:454–464` | exact |
| `hp41-core/src/state.rs` (add `assignments: BTreeMap<u8, String>` field) | model (state struct) | persistence | existing `key_assignments` at `state.rs:86–88`; existing `#[serde(default)]` fields at `:97–112, :119, :126, :134` | exact (sibling field) |
| `hp41-core/src/ops/registers.rs` (bounds-audit: op_sto, op_rcl, op_sto_arith, op_clreg) | service (data access) | CRUD | self — current raw-index pattern at `:19, 30, 50–56, 99` is the **BEFORE** to replace | exact (in-place rewrite) |
| `hp41-core/src/ops/stats.rs` (entry-guard `if regs.len() < 7 { Err(InvalidOp) }`) | service (data access) | CRUD | existing `op_sigma_plus` at `:22–48` for the BEFORE (no guard); `op_mean` at `:79–83` for the early-return-on-bad-input pattern | role-match |
| `hp41-core/src/ops/display_ops.rs` (bounds-audit `op_view`) | service (data access) | CRUD | same as `registers.rs` audit — `display_ops.rs:16–24` is in scope | exact |
| `hp41-core/src/ops/program_edit.rs` *(NEW file — optional; planner may inline in program.rs)* containing `op_clp`, `op_del`, `op_ins` | service (program-buffer mutation) | event-driven | `hp41-core/src/ops/sound.rs` (Phase 21, 2-fn single-purpose module — module header + 2 fns + inline test) | role-match (module shape) |
| `hp41-core/tests/phase22_program_control.rs` (NEW) | test (integration) | request-response | `hp41-core/tests/phase21_sound.rs` (whole file) | exact |
| `hp41-core/tests/phase22_program_edit.rs` (NEW) | test (integration) | request-response | `hp41-core/tests/phase21_flags.rs` (whole file) | exact |
| `hp41-core/tests/phase22_memory_ops.rs` (NEW) | test (integration) | request-response | `hp41-core/tests/phase21_flags.rs` + `phase21_sound.rs` (serde-default proof + dispatch round-trip) | exact |
| `hp41-core/tests/phase22_catalog.rs` (NEW) | test (integration) | request-response | `hp41-core/tests/phase21_sound.rs` (event_buffer write+drain pattern); `hp41-core/tests/print_tests.rs` (print_buffer assertions) | exact |
| `hp41-core/tests/phase22_asn.rs` (NEW) | test (integration) | persistence | `hp41-core/tests/phase21_flags.rs::test_serde_round_trip_with_flags_set` at `:39–46` | exact |
| `hp41-cli/src/prgm_display.rs::op_display_name` (add 13 arms) | view (program-listing display) | request-response | existing arms at `prgm_display.rs:106–177` | exact (same module continuation) |
| `hp41-gui/src-tauri/src/prgm_display.rs::op_display_name` (add same 13 arms) | view (program-listing display) | request-response | existing arms at `prgm_display.rs:47–197` (intentional duplicate of CLI copy per CLAUDE.md §SC-4) | exact |

---

## Pattern Assignments

### `hp41-core/src/ops/program.rs::resume_program()` (NEW)

**Analog:** `run_program()` at `hp41-core/src/ops/program.rs:139–169`.

**Reference excerpt** (`program.rs:139–169`):
```rust
/// Execute a recorded program starting at the given label.
/// D-06: sets is_running = true, resets to false even on error path.
pub fn run_program(state: &mut CalcState, entry_label: &str) -> Result<(), HpError> {
    // Clone program — borrow conflict guard
    let program = state.program.clone();

    // Linear scan for entry label (D-02).
    let start = match program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if l == entry_label))
    {
        Some(idx) => idx,
        None => {
            if let Some(op) = builtin_card_op(entry_label) {
                return crate::ops::dispatch(state, op);
            }
            return Err(HpError::InvalidOp);
        }
    };

    state.pc = start + 1;
    state.call_stack.clear();
    state.is_running = true;

    let result = run_loop(state, &program);

    state.is_running = false; // always reset, even on error
    result
}
```

**Pattern to apply** (`resume_program` is `run_program` minus the entry-label search; `state.pc` is the entry point):
```rust
pub fn resume_program(state: &mut CalcState) -> Result<(), HpError> {
    if state.pc >= state.program.len() {
        return Err(HpError::InvalidOp); // nothing to resume
    }
    let program = state.program.clone();
    state.is_running = true;
    let result = run_loop(state, &program);
    state.is_running = false; // ALWAYS reset, even on error (Pitfall 2)
    result
}
```

**Pitfall callouts:**
- **Pitfall 2 (RESEARCH §2):** Do NOT use `?` to propagate `run_loop` errors. Capture into `let result`, reset `is_running = false`, then return `result`. The naive `run_loop(...)?` short-circuits before the reset and leaves a stale `is_running == true`.
- Do NOT clear `call_stack` here (unlike `run_program`) — resume must preserve any pending XEQ frames so RTN behaves correctly.

---

### `hp41-core/src/ops/program.rs` run_loop new arms

**Insertion point:** inside the `match op { ... }` block at `program.rs:191–281`, near the existing programming-control arms (Op::Gto/Op::Xeq/Op::Prompt).

#### `Op::Stop` arm

**Analog:** `Op::Prompt` arm at `program.rs:272–275`:
```rust
// ── Phase 21: PROMPT — write ALPHA to display_override + break run_loop.
// Full STOP/resume semantics deferred to Phase 22 (RESEARCH A5).
Op::Prompt => {
    state.display_override = Some(state.alpha_reg.chars().take(24).collect::<String>());
    break;
}
```

**Pattern to apply** (D-22.1):
```rust
// ── Phase 22: STOP — break run_loop only. NO display write.
// Unlike Op::Prompt, the previous step's display value persists until the next
// dispatch(). state.pc is already advanced past the STOP step by the
// top-of-iteration `state.pc += 1` (line 189).
Op::Stop => break,
```

**Pitfall callouts:**
- **Pitfall 1 (RESEARCH §2):** Do NOT copy the `display_override = Some(...)` line from `Op::Prompt`. STOP freezes whatever is currently displayed; PROMPT replaces it with ALPHA. The arms must be adjacent in source with a leading comment to prevent future copy-paste regressions.

#### `Op::GtoInd(reg)` arm

**Analog:** `Op::Gto(label)` arm at `program.rs:201–204`:
```rust
Op::Gto(label) => {
    let target = find_in_program(program, &label)?;
    state.pc = target + 1;
}
```

**Pattern to apply** (D-22.15):
```rust
Op::GtoInd(reg) => {
    let pointer = state.regs.get(reg as usize).ok_or(HpError::InvalidOp)?.clone();
    let int_part = pointer.trunc_int();
    if int_part != pointer { return Err(HpError::InvalidOp); }
    let label_str = int_part.inner().to_string();
    let target = find_in_program(program, &label_str)?;
    state.pc = target + 1; // mirrors Op::Gto: pc → step AFTER the LBL marker
}
```

#### `Op::XeqInd(reg)` arm

**Analog:** `Op::Xeq(label)` arm at `program.rs:205–230`:
```rust
Op::Xeq(label) => {
    if state.call_stack.len() >= 4 {
        return Err(HpError::CallDepth); // D-13/D-14: error before mutation
    }
    match find_in_program(program, &label) {
        Ok(target) => {
            state.call_stack.push(state.pc);
            state.pc = target + 1;
        }
        Err(_) => {
            if let Some(card_op) = builtin_card_op(&label) {
                crate::ops::dispatch(state, card_op)?;
            } else {
                return Err(HpError::InvalidOp);
            }
        }
    }
}
```

**Pattern to apply** (D-22.15) — note the **pre-mutation 4-deep check** comes BEFORE the register read so we never partially mutate on error:
```rust
Op::XeqInd(reg) => {
    if state.call_stack.len() >= 4 {
        return Err(HpError::CallDepth); // pre-mutation check (matches Op::Xeq at :207)
    }
    let pointer = state.regs.get(reg as usize).ok_or(HpError::InvalidOp)?.clone();
    let int_part = pointer.trunc_int();
    if int_part != pointer { return Err(HpError::InvalidOp); }
    let label_str = int_part.inner().to_string();
    let target = find_in_program(program, &label_str)?;
    state.call_stack.push(state.pc);
    state.pc = target + 1;
}
```

**Pitfall callouts:**
- Indirect ops do NOT do the Card Reader builtin fallback (it's a label-search-by-name behavior; the integer pointer route resolves to a numeric label string only).
- Phase 24 will extract this inline check into `resolve_indirect(state, reg) -> Result<u8, HpError>`. Phase 22 ships inline.

---

### `hp41-core/src/ops/program.rs` execute_op new arms

**Insertion point:** inside the `match op { ... }` block at `program.rs:305–465`. Group additions near related Phase 21 sections.

#### `Op::Pse` arm

**Analog:** `Op::Beep` (sound — event_buffer push) at `program.rs:452` (and `sound.rs:13–17`):
```rust
// program.rs:452
Op::Beep => super::sound::op_beep(state),

// sound.rs:13–17
pub fn op_beep(state: &mut CalcState) -> Result<(), HpError> {
    state.event_buffer.push("BEEP".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Pattern to apply** (D-22.4) — combines event_buffer push (from sound) with display_override write (from `op_aview` at `display_ops.rs:28–33`):
```rust
// program.rs execute_op arm
Op::Pse => {
    let formatted = crate::format::format_hpnum(&state.stack.x, &state.display_mode);
    state.display_override = Some(formatted);
    state.event_buffer.push("PAUSE 1000".to_string());
    crate::stack::apply_lift_effect(state, crate::stack::LiftEffect::Neutral);
    Ok(())
}
```

**Pitfall callouts:**
- **Pitfall 3 (RESEARCH §2):** `run_loop` calls `execute_op` directly (NOT through `dispatch`), so the dispatch-top `display_override = None` clear at `mod.rs:410` does NOT run between iterations. PSE's display_override therefore survives subsequent program steps until the next interactive `dispatch()` call clears it. This is the correct HP-41 semantic (the X value stays visible while the program continues running silently). The frontend reads `event_buffer.contains("PAUSE 1000")` and sleeps ~1s before its next refresh.
- **Pitfall 10 (RESEARCH §2):** Interactive PSE flows through `dispatch()` → `flush_entry_buf()` first, so a pending `1.23` is lifted to X before PSE formats it. Do NOT add a second `flush_entry_buf` inside `op_pse` — that would double-clear the entry buffer.

#### `Op::Cla` arm

**Analog:** `Op::AlphaClear` at `program.rs:387`:
```rust
Op::AlphaClear => op_alpha_clear(state),
```

`op_alpha_clear` at `alpha.rs:34–38`:
```rust
pub fn op_alpha_clear(state: &mut CalcState) -> Result<(), HpError> {
    state.alpha_reg.clear();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Pattern to apply** (D-22.13) — `Op::Cla` is a thin alias that delegates to the same helper:
```rust
// program.rs execute_op match (and same arm in ops/mod.rs::dispatch)
Op::Cla => crate::ops::alpha::op_alpha_clear(state),
```

**Pitfall callouts:**
- **Pitfall 8 (RESEARCH §2):** Do NOT remove `Op::AlphaClear` — v1.0 save files contain it. Both `Op::Cla` and `Op::AlphaClear` must coexist; only the display name differs (`"CLA"` vs `"CLRALPHA"`).

#### `Op::Clst` arm

**Analog:** `op_clreg()` at `registers.rs:96–102` (whole-aggregate-zero pattern):
```rust
pub fn op_clreg(state: &mut CalcState) -> Result<(), HpError> {
    state.regs = vec![crate::num::HpNum::zero(); 100];
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Pattern to apply** (D-22.14) — same shape but targets the 4 stack levels; **preserve LASTX and lift_enabled**:
```rust
// recommended: hp41-core/src/ops/memory.rs (new file) or stack_ops.rs
pub fn op_clst(state: &mut CalcState) -> Result<(), HpError> {
    state.stack.x = HpNum::zero();
    state.stack.y = HpNum::zero();
    state.stack.z = HpNum::zero();
    state.stack.t = HpNum::zero();
    // lastx and lift_enabled deliberately UNTOUCHED (D-22.14)
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Pitfall callouts:**
- LASTX preservation must have an explicit integration test (RESEARCH §4 — `test_clst_preserves_lastx_and_lift`).
- `lift_enabled` is NOT modified by Neutral — verify by reading `apply_lift_effect` behavior in `stack.rs`.

#### `Op::Size(u16)` arm

**Analog:** `Op::FmtFix(n)` arm at `program.rs:356–363` (guard-then-mutate with `apply_lift_effect`):
```rust
Op::FmtFix(n) => {
    if n > 9 {
        return Err(HpError::InvalidOp);
    }
    state.display_mode = DisplayMode::Fix(n);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Pattern to apply** (D-22.11; **OQ-2 = Option A — clamp 0 to 1**, per CONTEXT.md line 9):
```rust
pub fn op_size(state: &mut CalcState, nnn: u16) -> Result<(), HpError> {
    if nnn > 319 {
        return Err(HpError::InvalidOp);
    }
    let target = nnn.max(1) as usize; // OQ-2: SIZE 0 → silently clamp to 1
    state.regs.resize(target, HpNum::zero());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

#### `Op::Pack` arm

**Analog:** `Op::Null` arm at `program.rs:413–416` (literal no-op with Neutral lift):
```rust
Op::Null => {
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Pattern to apply** (D-22.12) — identical shape:
```rust
Op::Pack => {
    // Flat-Vec program model has no gaps; PACK is a documented no-op
    // (real HP-41 compacts program memory). LiftEffect: Neutral.
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

#### `Op::Catalog(u8)` arm

**Analog:** `op_prx`/`op_pra`/`op_prstk` at `print.rs:13–60` (24-char width, push to print_buffer):
```rust
pub fn op_prx(state: &mut CalcState) -> Result<(), HpError> {
    let line = format!("{:>24}", format_hpnum(&state.stack.x, &state.display_mode));
    state.print_buffer.push(line);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Pattern to apply** (D-22.16 family; **OQ-1 = Option B — hardware-faithful**, per CONTEXT.md line 6 — CAT 1 = programs, CAT 2/3/4 = "NOT AVAILABLE"):
```rust
pub fn op_catalog(state: &mut CalcState, n: u8) -> Result<(), HpError> {
    if n == 0 || n >= 5 {
        return Err(HpError::InvalidOp);
    }
    state.print_buffer.push(format!("{:<24}", format!("-- CATALOG {n} --")));
    match n {
        1 => {
            // CATALOG 1: programs — iterate Lbl positions, emit "LBL name  steps"
            let labels: Vec<(usize, &str)> = state.program.iter().enumerate()
                .filter_map(|(i, op)| match op {
                    Op::Lbl(nm) => Some((i, nm.as_str())),
                    _ => None,
                })
                .collect();
            for (idx, (pos, name)) in labels.iter().enumerate() {
                let end = labels.get(idx + 1).map(|(p, _)| *p).unwrap_or(state.program.len());
                let steps = end - pos;
                state.print_buffer.push(format!("{:<24}", format!("LBL {name:9}  {steps:5}")));
            }
        }
        2 | 3 | 4 => {
            // CATALOG 2 = XROM, 3 = HP-IL, 4 = peripherals — none in this emulator.
            state.print_buffer.push(format!("{:<24}", "NOT AVAILABLE"));
        }
        _ => unreachable!(),
    }
    state.print_buffer.push(format!("{:<24}", "-- END --"));
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

#### `Op::Asn { name, key_code }` arm

**Analog:** `Op::SfFlag(n)` at `program.rs:440` (struct-variant dispatch indirection through a helper) AND `Op::FmtFix(n)` for the guard-then-mutate shape. There is no existing BTreeMap-write op in `hp41-core`; the closest precedent is the **Phase 5 `key_assignments: BTreeMap<char, String>`** field which is mutated by `hp41-cli/src/app.rs` (not by an op).

**Pattern to apply** (D-22.18; **OQ-3 = Option A — empty name removes**, per CONTEXT.md line 10):
```rust
pub fn op_asn(state: &mut CalcState, name: String, key_code: u8) -> Result<(), HpError> {
    if name.is_empty() {
        state.assignments.remove(&key_code); // OQ-3: empty name removes
    } else {
        state.assignments.insert(key_code, name);
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

In the `execute_op`/`dispatch` match arm:
```rust
Op::Asn { name, key_code } => op_asn(state, name, key_code),
```

#### Programming-ops catch-all extension

**Analog (BEFORE):** `program.rs:454–464`:
```rust
// Programming ops handled by run_loop directly — must not reach here
Op::Lbl(_)
| Op::Gto(_)
| Op::Xeq(_)
| Op::Rtn
| Op::PrgmMode
| Op::Test(_)
| Op::Isg(_)
| Op::Dse(_)
| Op::FlagTest { .. }
| Op::Prompt => Err(HpError::InvalidOp),
```

**Pattern (AFTER)** — add `Op::Stop`, `Op::Clp(_)`, `Op::Del(_)`, `Op::Ins`, `Op::GtoInd(_)`, `Op::XeqInd(_)` to this list (D-22.5 / D-22.10):
```rust
Op::Lbl(_)
| Op::Gto(_)
| Op::Xeq(_)
| Op::Rtn
| Op::PrgmMode
| Op::Test(_)
| Op::Isg(_)
| Op::Dse(_)
| Op::FlagTest { .. }
| Op::Prompt
| Op::Stop                                  // NEW Phase 22 (handled by run_loop directly)
| Op::Clp(_) | Op::Del(_) | Op::Ins         // NEW Phase 22 (interactive PRGM-mode primitives only)
| Op::GtoInd(_) | Op::XeqInd(_) => Err(HpError::InvalidOp),  // NEW Phase 22 (run_loop arms)
```

**Pitfall callouts:**
- **Pitfall 7 (RESEARCH §2):** `Op::Pse` does NOT join the catch-all (it executes inside execute_op — it's the entire point of PSE that it runs mid-program without breaking). `Op::Cla`/`Op::Clst`/`Op::Pack`/`Op::Size`/`Op::Catalog`/`Op::Asn` also do NOT join — they execute in both interactive and program contexts.

---

### `hp41-core/src/ops/program.rs` op_clp / op_del / op_ins (new functions)

**Module recommendation:** `hp41-core/src/ops/program_edit.rs` (new file) OR inline in `program.rs`. Planner picks; **module analog** is `hp41-core/src/ops/sound.rs` (small 2-fn single-purpose module — module header + 2 fns + inline test module).

**Reference excerpt** (`sound.rs:1–30`):
```rust
//! Phase 21 sound event operations: BEEP, TONE n.
//!
//! Both ops have LiftEffect::Neutral. Output is buffered into `state.event_buffer`;
//! the CLI / GUI drains the buffer after each dispatch (Phase 25/26 wiring).
//! This module preserves the hp41-core zero-I/O invariant (no println!/eprintln!).

use crate::error::HpError;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;

pub fn op_beep(state: &mut CalcState) -> Result<(), HpError> {
    state.event_buffer.push("BEEP".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

pub fn op_tone(state: &mut CalcState, n: u8) -> Result<(), HpError> {
    if n > 9 {
        return Err(HpError::InvalidOp);
    }
    state.event_buffer.push(format!("TONE {n}"));
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Pattern to apply** for `op_clp` (D-22.7) — gate on `prgm_mode`, drain `[start, end)`, then **reposition pc to start** (Pitfall 6):
```rust
pub fn op_clp(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    if !state.prgm_mode { return Err(HpError::InvalidOp); }
    let start = state.program.iter()
        .position(|op| matches!(op, Op::Lbl(n) if n == label))
        .ok_or(HpError::InvalidOp)?;
    let end = state.program.iter().skip(start + 1)
        .position(|op| matches!(op, Op::Lbl(_)))
        .map(|i| start + 1 + i)
        .unwrap_or(state.program.len());
    state.program.drain(start..end);
    state.pc = start.min(state.program.len()); // Pitfall 6: cursor → start of (now-deleted) block
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Pattern to apply** for `op_del` (D-22.9):
```rust
pub fn op_del(state: &mut CalcState, nnn: u8) -> Result<(), HpError> {
    if !state.prgm_mode { return Err(HpError::InvalidOp); }
    let n = (nnn as usize).min(state.program.len().saturating_sub(state.pc));
    if n == 0 {
        apply_lift_effect(state, LiftEffect::Neutral);
        return Ok(()); // no-op (nnn == 0 OR pc == len)
    }
    state.program.drain(state.pc..state.pc + n);
    // pc unchanged — next op naturally falls at the same index (which is the post-drain position).
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Pattern to apply** for `op_ins` (D-22.8):
```rust
pub fn op_ins(state: &mut CalcState) -> Result<(), HpError> {
    if !state.prgm_mode { return Err(HpError::InvalidOp); }
    state.program.insert(state.pc, Op::Null);
    // state.pc unchanged — still points at the freshly inserted Null
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Pitfall callouts:**
- **Pitfall 4 (RESEARCH §2 / §3 Pitfall 6):** CLP's `state.pc = start` is the hardware-faithful cursor reposition. Without it, pc may dangle past the new shorter program.
- All three ops MUST gate on `state.prgm_mode == true`. Interactive dispatch with `prgm_mode == false` returns InvalidOp. They are program-editing primitives, not recorded ops — they must NOT self-record (D-22.10).

---

### `hp41-core/src/state.rs` — new `assignments` field

**Analog:** existing `key_assignments` field at `state.rs:86–88` + `last_key_code` / `reg_m/n/o` / `flags` / `display_override` / `event_buffer` / `pending_card_op` for the `#[serde(default)]` pattern:

**Reference excerpt** (`state.rs:82–88`):
```rust
// ── Phase 5: USER mode & key assignments ─────────────────────────────────
/// USER mode active: when true, key_assignments are consulted before normal dispatch.
pub user_mode: bool,
/// User key assignments: maps key char → program label name.
/// BTreeMap for deterministic JSON serialization order (D-25, D-29).
pub key_assignments: BTreeMap<char, String>,
```

`#[serde(default)]` precedent (`state.rs:97–112`):
```rust
#[serde(default)]
pub last_key_code: u8,

#[serde(default)]
pub reg_m: HpNum,
```

**Pattern to apply** (D-22.17) — slot the new field adjacent to `key_assignments` for grep affinity:
```rust
/// User key assignments: maps key char → program label name.
/// BTreeMap for deterministic JSON serialization order (D-25, D-29).
pub key_assignments: BTreeMap<char, String>,
/// HP-41 ASN key assignments: maps hardware key code (row×10+col, 1-indexed)
/// → assigned target name. Phase 22 (FN-KEY-01). Coexists with key_assignments
/// (Phase 5, char-keyed) — Phase 25/26 reconciles the two maps.
/// `#[serde(default)]` keeps v1.0–v2.1 save files loadable (default → empty map).
#[serde(default)]
pub assignments: BTreeMap<u8, String>,
```

Update `CalcState::new()` at `state.rs:145–171` to add `assignments: BTreeMap::new(),` next to the existing `key_assignments: BTreeMap::new(),` line (`state.rs:160`).

---

### `hp41-core/src/ops/registers.rs` — bounds-audit (Wave-0)

**Analog (BEFORE):** existing raw-indexing pattern at `registers.rs:15–22`:
```rust
pub fn op_sto(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    if reg >= 100 {
        return Err(HpError::InvalidOp);
    }
    state.regs[reg as usize] = state.stack.x.clone();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Pattern to apply (AFTER)** — replace the hardcoded `100` bound with `state.regs.len()`:
```rust
pub fn op_sto(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let idx = reg as usize;
    if idx >= state.regs.len() {
        return Err(HpError::InvalidOp); // honors current SIZE (was hardcoded 100)
    }
    state.regs[idx] = state.stack.x.clone(); // safe — bounds-checked above
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Affected sites in `registers.rs` (Wave-0 audit, RESEARCH §3):**
| Site | Current Bound | Fix |
|------|---------------|-----|
| `:15–22` `op_sto` | `reg >= 100` | `idx >= state.regs.len()` |
| `:26–37` `op_rcl` | `reg >= 100` | same |
| `:44–58` `op_sto_arith` (4 read+1 write) | `reg >= 100` | same |
| `:98–102` `op_clreg` | hardcoded `vec![..; 100]` | `let n = state.regs.len(); state.regs = vec![HpNum::zero(); n];` so CLREG respects current SIZE |

**Same audit applies to** `hp41-core/src/ops/display_ops.rs::op_view` at `:16–24` (reads `state.regs[reg as usize]` with `reg >= 100` guard).

---

### `hp41-core/src/ops/stats.rs` — entry guards (Wave-0)

**Analog (BEFORE):** `op_sigma_plus` at `stats.rs:22–48` has **no** length guard on `state.regs[1..=6]`:
```rust
pub fn op_sigma_plus(state: &mut CalcState) -> Result<(), HpError> {
    let x = state.stack.x.clone();
    let y = state.stack.y.clone();
    let new_r1 = state.regs[1].checked_add(&x.checked_sq()?)?; // Σx² += x²
    // ... R2..R6 same shape ...
}
```

**Pattern to apply (AFTER)** — single-line entry guard at the top of each Σ-family function (RESEARCH §3 — `op_sigma_plus`, `op_sigma_minus`, `op_mean`, `op_sdev`, `op_lr`, `op_yhat`, `op_corr`, `op_cl_sigma_stat`):
```rust
pub fn op_sigma_plus(state: &mut CalcState) -> Result<(), HpError> {
    if state.regs.len() < 7 {
        return Err(HpError::InvalidOp); // Σ block R01..R06 not addressable
    }
    let x = state.stack.x.clone();
    let y = state.stack.y.clone();
    // ... existing body unchanged ...
}
```

**Pitfall callouts:**
- **Pitfall 5 (RESEARCH §2):** after this audit, `Σ+` on a SIZE-3 state returns `Err(InvalidOp)` — matches HP-41 hardware "NONEXISTENT" behavior in spirit. Add sentinel test `Op::Size(3) → Op::SigmaPlus → expects InvalidOp` in `tests/phase22_memory_ops.rs`.
- The flag-26/27/28 register-redirection feature (real HP-41) is NOT in v2.x — Σ block is hardcoded R01–R06 (Assumption A7 in RESEARCH).

---

### `hp41-cli/src/prgm_display.rs::op_display_name` — add 13 arms

**Analog:** existing arms at `prgm_display.rs:28–177`. The exhaustive `match op { ... }` returns `String`.

**Reference excerpt** (selection showing all the styles needed):
```rust
// Phase 1: arithmetic (simple atom)
Op::Sin => "SIN".to_string(),
// Phase 2: registers (formatted with numeric suffix)
Op::StoReg(r) => format!("STO {r:02}"),
// Phase 3: programming (label suffix)
Op::Lbl(s) => format!("LBL {s}"),
Op::Gto(s) => format!("GTO {s}"),
Op::Rtn => "RTN".to_string(),
// Phase 21: Flags with struct-variant
Op::FlagTest { kind, flag } => {
    let mnemonic = match kind {
        FlagTestKind::IsSet => "FS?",
        FlagTestKind::IsClear => "FC?",
        FlagTestKind::IsSetThenClear => "FS?C",
        FlagTestKind::IsClearThenClear => "FC?C",
    };
    format!("{mnemonic} {flag:02}")
}
```

**Pattern to apply** — add the following 13 arms (planner picks final display text; suggested per CONTEXT.md "Claude's Discretion" line 345):
```rust
// Phase 22: Program control
Op::Stop => "STOP".to_string(),
Op::Pse => "PSE".to_string(),
Op::GtoInd(r) => format!("GTO IND {r:02}"),
Op::XeqInd(r) => format!("XEQ IND {r:02}"),
// Phase 22: Program editing
Op::Clp(name) => format!("CLP {name}"),
Op::Del(n) => format!("DEL {n:03}"),
Op::Ins => "INS".to_string(),
// Phase 22: Memory & stack
Op::Size(n) => format!("SIZE {n:03}"),
Op::Cla => "CLA".to_string(),    // D-22.13: NOT "CLRALPHA" (that's Op::AlphaClear)
Op::Clst => "CLST".to_string(),
Op::Pack => "PACK".to_string(),
// Phase 22: Catalog & assignments
Op::Catalog(n) => format!("CATALOG {n}"),
Op::Asn { name, key_code } => format!("ASN \"{name}\" {key_code:02}"),
```

**Pitfall callouts:**
- **Pitfall 8 (RESEARCH §2):** `Op::Cla` MUST emit `"CLA"`, `Op::AlphaClear` MUST stay `"CLRALPHA"` — they are two distinct variants by design (D-22.13). The duplication is the only correct way to honor both HP-41 listing convention and v1.0 save-file compat under the 4-place rule.

---

### `hp41-gui/src-tauri/src/prgm_display.rs::op_display_name` — same 13 arms

**Analog:** **CLI copy** (above). The GUI copy at `hp41-gui/src-tauri/src/prgm_display.rs:47–197` is structurally identical and **must receive the exact same 13 arms** per CLAUDE.md §SC-4 (intentional duplication; the GUI copy must not introduce calculator/math logic).

**Pattern to apply:** copy the 13 arms from the CLI edit verbatim. The two files share the same `Op` enum import (`use hp41_core::ops::{FlagTestKind, Op, StackReg, StoArithKind};`) and the same `String` return type.

---

### Integration test files (5 NEW)

**Analog:** `hp41-core/tests/phase21_flags.rs` (whole file, 235 lines) and `hp41-core/tests/phase21_sound.rs` (whole file, 130 lines). Both follow the same opening template.

**Reference excerpt** (`phase21_sound.rs:1–50`):
```rust
//! Integration tests for Phase 21 Plan 04 (Sound event channel: BEEP / TONE n).
//!
//! Covers FN-SOUND-01/02 plus the zero-I/O invariant regression sentinel.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{dispatch, Op};
use hp41_core::{CalcState, HpError};

#[test]
fn test_event_buffer_field_defaults_to_empty() {
    let s = CalcState::new();
    assert!(s.event_buffer.is_empty());
}

#[test]
fn test_load_v20_save_no_event_buffer_field() {
    let json = std::fs::read_to_string("tests/fixtures/v20-autosave.json").unwrap();
    let s: CalcState = serde_json::from_str(&json).unwrap();
    assert!(s.event_buffer.is_empty(),
        "v2.0 fixture must load with event_buffer empty");
}

#[test]
fn test_beep_pushes_event() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Beep).unwrap();
    assert_eq!(s.event_buffer, vec!["BEEP".to_string()]);
}
```

**Serde round-trip pattern** (`phase21_flags.rs:39–46`):
```rust
#[test]
fn test_serde_round_trip_with_flags_set() {
    let mut s = CalcState::new();
    s.flags = 0xDEAD_BEEFu64;
    let json = serde_json::to_string(&s).unwrap();
    let back: CalcState = serde_json::from_str(&json).unwrap();
    assert_eq!(back.flags, 0xDEAD_BEEFu64);
}
```

**Pattern to apply** for each Phase 22 integration file (header + one example test per FN-ID; full target list in RESEARCH §5):
```rust
//! Integration tests for Phase 22 Plan XX (<plan-name>).
//!
//! Covers FN-PROG-XX / FN-MEM-XX / FN-KEY-XX.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{dispatch, program::{run_program, resume_program}, Op};
use hp41_core::{CalcState, HpError, HpNum};

#[test]
fn test_stop_then_resume() {
    let mut s = CalcState::new();
    s.program = vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::from(42i32)),
        Op::Stop,
        Op::PushNum(HpNum::from(99i32)),
    ];
    run_program(&mut s, "A").unwrap();
    // After STOP: pc points past STOP; is_running == false; X == 42.
    assert_eq!(s.stack.x, HpNum::from(42i32));
    assert!(!s.is_running);
    assert!(s.pc < s.program.len(), "pc must point at next step");

    // Resume: continues from the saved pc through the final PushNum.
    resume_program(&mut s).unwrap();
    assert_eq!(s.stack.x, HpNum::from(99i32));
    assert!(!s.is_running);
}
```

**File-by-file content map:**
- `tests/phase22_program_control.rs` — STOP-then-resume, PSE writes both channels, GTO IND / XEQ IND happy + non-integer reject + 4-deep callstack, Pitfall 1 sentinel ("after STOP, display_override unchanged").
- `tests/phase22_program_edit.rs` — CLP boundary (last LBL → end-of-Vec), DEL clamping (nnn=0 no-op, nnn>remaining clamps), INS inserts Op::Null at pc with pc unchanged, prgm_mode=false → InvalidOp for all three.
- `tests/phase22_memory_ops.rs` — SIZE grow + shrink, SIZE 0 clamps to 1 (OQ-2), CLA equivalent to op_alpha_clear, CLST preserves LASTX + lift_enabled, PACK no-op + Neutral lift, **Pitfall 5 sentinel: `Op::Size(3)` then `Op::SigmaPlus` returns InvalidOp (NOT panic)**.
- `tests/phase22_catalog.rs` — CATALOG 1 = programs with non-empty header + body + footer (24-char width), CATALOG 2/3/4 = "NOT AVAILABLE" payload, CATALOG 0 + CATALOG 5 = InvalidOp.
- `tests/phase22_asn.rs` — ASN insert, ASN with empty name removes (OQ-3), serde round-trip (assignments survives JSON save/load), assignments field defaults to empty BTreeMap (serde-default sentinel against `v20-autosave.json`).

---

## Shared Patterns

### LiftEffect: Neutral (every Phase 22 op)

**Source:** `hp41-core/src/stack.rs::apply_lift_effect()` (called via `crate::stack::apply_lift_effect(state, LiftEffect::Neutral)`)
**Apply to:** every new op function — all 13 Phase 22 variants are Neutral (D-22.25).

```rust
apply_lift_effect(state, LiftEffect::Neutral);
Ok(())
```

### Error surface — InvalidOp / CallDepth only

**Source:** `hp41-core/src/error.rs::HpError` — Phase 22 introduces **zero new variants**.
**Apply to:** every error path. Use `HpError::InvalidOp` for: out-of-range register, non-integer indirect pointer, label-miss, prgm_mode-false-edit attempt, CATALOG n out of [1,4], SIZE n > 319, EOF resume. Use `HpError::CallDepth` for XEQ IND 4-deep guard.

```rust
return Err(HpError::InvalidOp);
// or for XeqInd / XEQ subroutine over-deep:
return Err(HpError::CallDepth);
```

### `#[serde(default)]` for new fields

**Source:** `hp41-core/src/state.rs:97–112, :119, :126, :134` — every field added after v1.0 carries `#[serde(default)]`.
**Apply to:** the new `assignments: BTreeMap<u8, String>` field. Verified by the `v20-autosave.json` regression test pattern (`phase21_flags.rs:27–37`):

```rust
#[test]
fn test_load_v20_save_no_assignments_field() {
    let json = std::fs::read_to_string("tests/fixtures/v20-autosave.json").unwrap();
    let s: CalcState = serde_json::from_str(&json).unwrap();
    assert!(s.assignments.is_empty(),
        "v2.0 fixture must load with assignments empty");
}
```

### Guard-then-mutate (atomicity)

**Source:** `op_sto_arith` at `registers.rs:44–58` ("compute first, write on success") and `op_tone` at `sound.rs:23–30` ("guard before push").
**Apply to:** all Phase 22 ops that mutate state behind an error gate — `op_size` (guard `nnn > 319` before `resize`), `op_catalog` (guard `n == 0 || n >= 5` before push), `op_gto_ind`/`op_xeq_ind` (guard non-integer before `pc = target + 1`).

### Bounds-audit replacement pattern

**Source:** the BEFORE pattern is the existing `state.regs[i]` raw indexing across ~28 production sites (RESEARCH §3).
**Apply to:** every site in the audit table (registers.rs / display_ops.rs / stats.rs). Two acceptable AFTER shapes:

```rust
// Shape 1 (read+write with idiomatic conversion):
let idx = reg as usize;
if idx >= state.regs.len() {
    return Err(HpError::InvalidOp);
}
state.regs[idx] = state.stack.x.clone();

// Shape 2 (read-only, .get + clone):
let val = state.regs.get(reg as usize).ok_or(HpError::InvalidOp)?.clone();

// Shape 3 (multi-read entry guard — stats family):
if state.regs.len() < 7 {
    return Err(HpError::InvalidOp);
}
// then existing state.regs[1..=6] indexing is safe under the guard
```

### Two-place arm landing (`dispatch` + `execute_op`)

**Source:** every Phase 11+ op shows the same 2-place pattern — `dispatch` arm (`mod.rs:404–590`) AND `execute_op` arm (`program.rs:305–465`), often delegating to the same helper function.
**Apply to:** Pse / Size / Cla / Clst / Pack / Catalog / Asn — appear in BOTH places (interactive AND program execution).
For Stop / Clp / Del / Ins / GtoInd / XeqInd — `dispatch` arm does the interactive work (mostly InvalidOp or PRGM-mode primitive), `execute_op` arm is in the programming-ops catch-all (returns InvalidOp), and run_loop has its own direct arm (Stop / GtoInd / XeqInd).

### Four-place rule (Op variant landing)

**Source:** CLAUDE.md §"Settled Architecture Decisions" — every new `Op` variant must appear in:
1. `hp41-core/src/ops/mod.rs::Op` enum (D-22.21: append at END to preserve discriminant order)
2. `hp41-core/src/ops/mod.rs::dispatch()` match
3. `hp41-core/src/ops/program.rs::execute_op()` match (or programming-ops catch-all)
4. **BOTH** `prgm_display.rs::op_display_name()` copies — CLI **AND** GUI

**Apply to:** all 13 new variants. The exhaustive-match compiler check is your safety net — adding a variant to the enum without landing it in all four match arms fails to compile.

---

## No Analog Found

None. Every Phase 22 file/change has a strong existing analog in the workspace. The cross-cutting **Wave-0 bounds audit** is novel as an audit *task* but the BEFORE/AFTER code shapes are already present in the codebase (raw `state.regs[i]` is the BEFORE; the `idx >= state.regs.len()` guard is structurally identical to the existing `reg >= 100` guard, with the constant swapped for the dynamic length).

The `assignments` field on `CalcState` is novel as a type-shape (`BTreeMap<u8, String>`), but the field-addition pattern is identical to `key_assignments` (Phase 5) and the `#[serde(default)]` precedent applies verbatim.

---

## Metadata

**Analog search scope:**
- `hp41-core/src/ops/` (entire directory — verified line-precise for the 6 modules above)
- `hp41-core/src/state.rs`, `hp41-core/src/error.rs`, `hp41-core/src/stack.rs` (referenced)
- `hp41-core/tests/` (integration test precedent — phase21_*.rs files)
- `hp41-cli/src/prgm_display.rs`, `hp41-gui/src-tauri/src/prgm_display.rs` (both 4-place rule copies)

**Files scanned:** 15 source files + 5 test files + 1 fixture directory listing
**Pattern extraction date:** 2026-05-14

---

## PATTERN MAPPING COMPLETE

**Phase:** 22 — Program Control & Memory Ops
**Files classified:** 13 (10 production touch-points + 5 new integration test files; some entries describe multi-arm additions to the same file)
**Analogs found:** 13 / 13

### Coverage
- Files with exact analog: 13
- Files with role-match analog: 0
- Files with no analog: 0

### Key Patterns Identified
- **`run_loop` arm vs `execute_op` arm asymmetry**: STOP / GtoInd / XeqInd handled in run_loop only (break / pc-manipulation); PSE handled in execute_op (no break — runs mid-program); Cla/Clst/Pack/Size/Catalog/Asn handled in both (interactive AND program).
- **`#[serde(default)]` + BTreeMap is the established new-field shape** since Phase 5 (`key_assignments`), Phase 12 (`last_key_code`, `reg_m/n/o`), Phase 21 (`flags`, `display_override`, `event_buffer`). Phase 22's `assignments` is exactly this pattern.
- **Programming-ops catch-all is the compile-time safety net**: adding new variants to `execute_op`'s catch-all guarantees that a variant landed without a `run_loop` arm errors at compile time AND fails-closed at runtime (returns InvalidOp rather than silently mis-executing).
- **Two-channel display+event pattern** (PSE): `display_override` for the visible string, `event_buffer` for the timing signal. Established by Phase 21 BEEP/TONE + VIEW/AVIEW; Phase 22 PSE combines both channels in a single op.
- **Four-place rule under the SC-4 invariant**: every new variant lands in enum + dispatch + execute_op + BOTH prgm_display copies. The GUI copy is intentionally duplicated (CLAUDE.md §SC-4).

### File Created
`/Users/daniel/GitRepository/hp41-calculator-emulator/.planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md`

### Ready for Planning
Pattern mapping complete. Each new variant / function / field has an exact existing analog with file:line excerpt. Planner can now write 22-01..22-04 PLAN.md files referencing this map's per-file pattern assignments and the 6 pitfall callouts (Pitfall 1 STOP-no-display, Pitfall 2 resume-reset-on-error, Pitfall 3 PSE display_override timing, Pitfall 4 CLP pc-adjustment, Pitfall 5 Σ+-on-shrunk-SIZE, Pitfall 7 catch-all completeness).
