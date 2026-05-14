# Phase 21: Flags, Display Control & Sound — Pattern Map

**Mapped:** 2026-05-14
**Files analyzed:** 11 new/modified files (in scope for Phase 21 core-only work)
**Analogs found:** 11 / 11 (every file has a direct prior-art analog inside `hp41-core`)

## Scope Boundary

Per RESEARCH.md §Architectural Responsibility Map and §Phase Requirements:

- **In scope for Phase 21 (this PATTERNS.md):** `hp41-core` only — state field additions, `Op` variants, new ops modules, `run_loop` arms, test fixtures, and the two `prgm_display.rs` copies (both copies updated together per the SC-4 spirit exception from CLAUDE.md).
- **Out of scope, deferred to Phase 25 / Phase 26:** `hp41-cli/src/{keys.rs,app.rs,help_data.rs,ui.rs}` and `hp41-gui/src-tauri/src/{key_map.rs,types.rs,commands.rs}` plus the React frontend. These are NOT touched in Phase 21 — confirmed at RESEARCH.md lines 31, 109–115, 205 ("Phase 21 does NOT modify key_map.rs"), and 583–589.

Files outside the core boundary that the prompt's "Anticipated new/modified files" list mentions (CLI keys/help/ui, GUI key_map/types/App.tsx/Keyboard.tsx) are listed in the §Deferred Files table at the end of this document — with the analog that Phase 25/26 will copy from — so the planner has a forward reference but does not bring them into Phase 21 plans.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-core/src/ops/flags.rs` | NEW-MODULE (utility + op layer) | mutator on `state.flags: u64`; no buffer | `hp41-core/src/ops/registers.rs::op_sto_arith` + bit helpers (greenfield) | role-match (parameterized op pattern); helpers are greenfield but mirror small free-fn style of `format.rs` |
| `hp41-core/src/ops/sound.rs` | NEW-MODULE (event-buffer op) | producer: pushes `String` to `state.event_buffer` | `hp41-core/src/ops/print.rs::op_prx` | exact (same buffer-channel discipline) |
| `hp41-core/src/ops/display_ops.rs` | NEW-MODULE (display-override + 6 ops) | mutator: writes `state.display_override = Some(_)` or `None` | `hp41-core/src/ops/print.rs::op_pra` (ALPHA-read pattern) + `op_clreg` (reset-field pattern) | role-match |
| `hp41-core/src/state.rs` | EXTEND-FIELD (×3 new fields) | data-class: 3 new fields w/ `#[serde]` attrs | existing `print_buffer` (line 93-94), `last_key_code` (97-99), `pending_card_op` (117-118) | exact (same idiom for serde-default / serde-skip) |
| `hp41-core/src/ops/mod.rs` | EXTEND-ENUM + EXTEND-MATCH | central dispatch hub | existing Phase 20 additions (lines 109-167 enum, 380-405 dispatch) + Phase 12 `StoArithStack` / `StackReg` for compound variants | exact |
| `hp41-core/src/ops/program.rs` | EXTEND-MATCH (`run_loop` skip + PROMPT break) + EXTEND-MATCH (`execute_op`) | interpreter loop with conditional `pc += 1` | `Op::Test(kind)` arm at lines 231-235 + `Op::Isg(reg)` arm at 236-244 | exact |
| `hp41-cli/src/prgm_display.rs` | EXTEND-MATCH (12 new arms) | display formatter (read-only) | existing Phase 20 arms at lines 47-51 (RND/FRC/ABS/SIGN/FACT) + `Op::FmtFix(n)` parameterized format at 76 | exact |
| `hp41-gui/src-tauri/src/prgm_display.rs` | EXTEND-MATCH (same 12 new arms, duplicate) | display formatter (read-only) | same file at lines 47-51, 96-98 (mirror of CLI copy) | exact — SC-4 spirit exception |
| `hp41-core/tests/phase21_flags.rs` | NEW-TEST (integration) | RED→GREEN integration tests | `hp41-core/tests/phase20_math.rs` | exact (same `dispatch + assert` shape, `push_x`/`push_y_then_x` helpers) |
| `hp41-core/tests/phase21_display.rs` | NEW-TEST (integration) | RED→GREEN integration tests | `hp41-core/tests/phase20_math.rs` | exact |
| `hp41-core/tests/phase21_sound.rs` | NEW-TEST (integration) | RED→GREEN integration tests | `hp41-core/tests/phase20_math.rs` | exact |

## Pattern Assignments

### `hp41-core/src/ops/flags.rs` (NEW-MODULE, parameterized mutator)

**Analog:** `hp41-core/src/ops/registers.rs::op_sto_arith` (lines 44-59) — closest match by shape (parameterized op with `if n > LIMIT { return Err(InvalidOp); }` guard, `apply_lift_effect(state, LiftEffect::Neutral)` tail, `Ok(())`).

**Imports pattern** (registers.rs lines 6-11) — mirror exactly:

```rust
//! Phase 21 flag operations: SF, CF.
//!
//! Both ops have LiftEffect::Neutral (they do not modify the stack).
//! Flag indices are 0..=55. Indices ≥ 56 return InvalidOp.

use crate::error::HpError;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;
```

**Bit-helper free functions** (greenfield — pattern is from RESEARCH.md §Code Example 1 + the small-free-fn style used in `hp41-core/src/format.rs`):

```rust
/// Get the state of a single flag (0..=55). Returns false for out-of-range
/// flags (defensive — callers should validate first via the op-layer guard).
#[inline]
pub fn flag_get(flags: u64, n: u8) -> bool {
    if n > 55 { return false; }
    (flags & (1u64 << n)) != 0
}

/// Set flag `n` (0..=55). Returns the modified flag word.
#[inline]
pub fn flag_set(flags: u64, n: u8) -> u64 {
    if n > 55 { return flags; }
    flags | (1u64 << n)
}

/// Clear flag `n`. Returns the modified flag word.
#[inline]
pub fn flag_clear(flags: u64, n: u8) -> u64 {
    if n > 55 { return flags; }
    flags & !(1u64 << n)
}
```

**Op layer — copy this exact shape from `op_sto_arith` (registers.rs:44-59):**

```rust
pub fn op_sf(state: &mut CalcState, n: u8) -> Result<(), HpError> {
    if n > 55 {
        return Err(HpError::InvalidOp);
    }
    state.flags = flag_set(state.flags, n);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

pub fn op_cf(state: &mut CalcState, n: u8) -> Result<(), HpError> {
    if n > 55 {
        return Err(HpError::InvalidOp);
    }
    state.flags = flag_clear(state.flags, n);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Difference vs. analog:** registers.rs guards `reg >= 100` and operates on `state.regs[reg as usize]` (Vec); flags.rs guards `n > 55` and operates on `state.flags` (u64 word).

**Reuse fraction:** **0.80** — guard + lift + Ok(()) tail are verbatim; only the field and bit math change.

---

### `hp41-core/src/ops/sound.rs` (NEW-MODULE, event-buffer producer)

**Analog:** `hp41-core/src/ops/print.rs::op_prx` (lines 11-18) — exact match for buffer-channel pattern. The only differences: the line content (no formatting from stack) and the target buffer name (`event_buffer` vs `print_buffer`).

**Imports pattern** (print.rs lines 6-9) — mirror:

```rust
//! Phase 21 sound event operations: BEEP, TONE n.
//!
//! Both ops have LiftEffect::Neutral. Output is buffered into state.event_buffer;
//! the CLI / GUI drains the buffer after each dispatch (Phase 25/26 wiring).

use crate::error::HpError;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;
```

**Core pattern — copy verbatim from `op_prx` (print.rs:13-18):**

```rust
/// BEEP — push the literal "BEEP" event line. LiftEffect::Neutral.
pub fn op_beep(state: &mut CalcState) -> Result<(), HpError> {
    state.event_buffer.push("BEEP".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// TONE n — push the literal "TONE n" event line. n is 0..=9; out-of-range InvalidOp.
pub fn op_tone(state: &mut CalcState, n: u8) -> Result<(), HpError> {
    if n > 9 {
        return Err(HpError::InvalidOp);
    }
    state.event_buffer.push(format!("TONE {n}"));
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Difference vs. analog:** `op_prx` builds the line from `format_hpnum(&state.stack.x, &state.display_mode)`; sound ops use literal strings (no stack read). Both end identically with `apply_lift_effect(state, LiftEffect::Neutral); Ok(())`.

**Reuse fraction:** **0.90** — buffer push + lift + Ok(()) are identical; only the string source changes.

---

### `hp41-core/src/ops/display_ops.rs` (NEW-MODULE, display-override channel)

**Analog (split):**
- For VIEW/AVIEW: `hp41-core/src/ops/print.rs::op_pra` (lines 22-30) — ALPHA-read + 24-char window + buffer write. Same chars-take-24 idiom; only the destination changes from `print_buffer.push(...)` to `display_override = Some(...)`.
- For CLD: `hp41-core/src/ops/registers.rs::op_clreg` (lines 98-102) — reset-a-field pattern.
- For AON/AOFF: `hp41-core/src/ops/flags.rs::op_sf` / `op_cf` (defined alongside in 21-01) targeting bit 48.

**Core pattern — VIEW (combines registers.rs read + print.rs format):**

```rust
//! Phase 21 display control: VIEW, AVIEW, PROMPT, AON, AOFF, CLD.

use crate::error::HpError;
use crate::format::format_hpnum;
use crate::ops::flags::{flag_clear, flag_set};
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;

/// VIEW nn — write a formatted register value into display_override. Stack untouched.
pub fn op_view(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    if reg >= 100 {
        return Err(HpError::InvalidOp);
    }
    let val = state.regs[reg as usize].clone();
    state.display_override = Some(format_hpnum(&val, &state.display_mode));
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// AVIEW — write ALPHA into display_override (truncated to 24 chars like PRA does).
pub fn op_aview(state: &mut CalcState) -> Result<(), HpError> {
    let alpha = state.alpha_reg.chars().take(24).collect::<String>();
    state.display_override = Some(alpha);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CLD — clear the display override only. Stack/ALPHA untouched. (analog: op_clreg shape)
pub fn op_cld(state: &mut CalcState) -> Result<(), HpError> {
    state.display_override = None;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// AON / AOFF — set/clear system flag 48 (HP-42S compat: "Alpha keyboard active").
pub fn op_aon(state: &mut CalcState) -> Result<(), HpError> {
    state.flags = flag_set(state.flags, 48);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
pub fn op_aoff(state: &mut CalcState) -> Result<(), HpError> {
    state.flags = flag_clear(state.flags, 48);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// PROMPT — write ALPHA to display_override. The PAUSE-program semantic is
/// handled by run_loop's Op::Prompt arm (it breaks the loop). The dispatch
/// path here is a no-op-but-set-override (interactive PROMPT just shows ALPHA).
pub fn op_prompt(state: &mut CalcState) -> Result<(), HpError> {
    let alpha = state.alpha_reg.chars().take(24).collect::<String>();
    state.display_override = Some(alpha);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Difference vs. analog:** print.rs writes to `print_buffer: Vec<String>`; display_ops.rs writes to `display_override: Option<String>` — `.push(...)` becomes `= Some(...)`. CLD mirrors `op_clreg`'s "reset to default" shape but resets `display_override` to `None` rather than zeroing `regs`.

**Reuse fraction:** **0.75** — chars-take-24 + lift + Ok(()) are reused verbatim; the write target is a `Option<String>` field instead of a `Vec<String>` buffer.

---

### `hp41-core/src/state.rs` (EXTEND-FIELD ×3)

**Analog:** the same file, lines 89-118 — the Phase 11 (`print_buffer`), Phase 12 (`last_key_code`, `reg_m/n/o`), and Card Reader quick task (`pending_card_op`) precedents. Three distinct serde idioms already coexist:

**Pattern A — `#[serde(default)]` (persistent, backward-compatible):**
```rust
// state.rs:97-99 — Phase 12 last_key_code
/// Last HP-41 row-column key code pressed ...
#[serde(default)]
pub last_key_code: u8,
```

**Pattern B — `#[serde(default, skip)]` (transient, never persisted):**
```rust
// state.rs:89-94 — Phase 11 print_buffer
/// Buffer of formatted print lines from PRX/PRA/PRSTK.
/// Drained by hp41-cli after each dispatch. Never persisted across sessions.
#[serde(default, skip)]
pub print_buffer: Vec<String>,

// state.rs:113-118 — Card Reader pending_card_op
#[serde(default, skip)]
pub pending_card_op: Option<crate::cardreader::CardOpRequest>,
```

**Phase 21 application (per RESEARCH.md §Pattern 3 lines 173-177):**

```rust
// Persistent: flag bits are part of the user's calculator state — survive save/load.
/// Phase 21: HP-41 flags (user flags 0–29, system flags 30–55) packed into a single u64.
/// Bit n = flag n. Default: 0 (all clear). Use ops::flags helpers for safe access.
#[serde(default)]
pub flags: u64,

// Transient: display override survives only until the next op clears it
/// Phase 21: display override channel. None = show normal display.
/// Cleared at top of dispatch() before each op (Pitfall 5).
#[serde(default, skip)]
pub display_override: Option<String>,

// Transient: event buffer mirrors print_buffer exactly.
/// Phase 21: BEEP/TONE event lines. Drained by frontend like print_buffer.
#[serde(default, skip)]
pub event_buffer: Vec<String>,
```

**And in `CalcState::new()` (state.rs:122-145) — add three lines, copying the existing init pattern:**
```rust
flags: 0,
display_override: None,
event_buffer: Vec::new(),
```

**Difference vs. analog:** None — these are direct copies of three existing precedents (one per pattern).

**Reuse fraction:** **0.95** — pure idiom copy.

---

### `hp41-core/src/ops/mod.rs` (EXTEND-ENUM + EXTEND-MATCH)

**Analog A — single-u8 parameterized variant** (`Op::StoReg(u8)`, mod.rs:183-184):
```rust
/// STO n — store X into register n (0–99). LiftEffect: Neutral.
StoReg(u8),
```
Pattern: doc-comment with LiftEffect tag + variant name + `(u8)`. Dispatch arm at line 431: `Op::StoReg(r) => op_sto(state, r),`.

**Analog B — struct-style compound variant** (`Op::StoArith`, mod.rs:188-191):
```rust
/// STO+/−/×/÷ n — arithmetic on register n using X. LiftEffect: Neutral.
StoArith {
    reg: u8,
    kind: StoArithKind,
},
```
Pattern: explicit named fields. Dispatch arm at mod.rs:433: `Op::StoArith { reg, kind } => op_sto_arith(state, reg, kind),`.

**Analog C — sub-enum** (`StoArithKind`, mod.rs:36-42 + `StackReg`, mod.rs:45-51) — companion enums declared in the same file before the `Op` enum.

**Phase 21 application — new enum + 12 variants:**

```rust
/// Phase 21: HP-41 flag-test kind — 4 total. Used in Op::FlagTest { kind, flag }.
/// Mirrors the StoArithKind / TestKind sub-enum pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FlagTestKind {
    /// FS? — skip next if flag is NOT set.
    IsSet,
    /// FC? — skip next if flag is NOT clear.
    IsClear,
    /// FS?C — skip next if not set; ALWAYS clear the flag afterward.
    IsSetThenClear,
    /// FC?C — skip next if not clear; ALWAYS clear the flag afterward.
    IsClearThenClear,
}
```

**12 new Op variants** (placed after Phase 12 / before Card Reader, ~mod.rs:262):

```rust
// ── Phase 21: Flags ─────────────────────────────────────────────────────
/// SF n — set flag n (0..=55). LiftEffect: Neutral.
SfFlag(u8),
/// CF n — clear flag n (0..=55). LiftEffect: Neutral.
CfFlag(u8),
/// FS?/FC?/FS?C/FC?C n — conditional flag test (run_loop skips next on false). LiftEffect: Neutral.
FlagTest { kind: FlagTestKind, flag: u8 },
// ── Phase 21: Display Control ───────────────────────────────────────────
/// VIEW nn — write register N's formatted value to display_override. LiftEffect: Neutral.
View(u8),
/// AVIEW — write ALPHA to display_override. LiftEffect: Neutral.
AView,
/// PROMPT — write ALPHA to display_override AND break run_loop. LiftEffect: Neutral.
Prompt,
/// AON / AOFF — set/clear system flag 48 (ALPHA auto-display). LiftEffect: Neutral.
Aon,
Aoff,
/// CLD — clear display_override. LiftEffect: Neutral.
Cld,
// ── Phase 21: Sound ─────────────────────────────────────────────────────
/// BEEP — push "BEEP" event to event_buffer. LiftEffect: Neutral.
Beep,
/// TONE n — push "TONE n" event to event_buffer (n = 0..=9). LiftEffect: Neutral.
Tone(u8),
```

**Dispatch arms** (insert after the Phase 12 block at mod.rs:481-505, before Card Reader):

```rust
// ── Phase 21: Flags ─────────────────────────────────────────────────
Op::SfFlag(n) => flags::op_sf(state, n),
Op::CfFlag(n) => flags::op_cf(state, n),
Op::FlagTest { .. } => {
    // Interactive: no-op (mirrors Op::Test). Skip semantic only inside run_loop.
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
// ── Phase 21: Display Control ───────────────────────────────────────
Op::View(r) => display_ops::op_view(state, r),
Op::AView => display_ops::op_aview(state),
Op::Prompt => display_ops::op_prompt(state),
Op::Aon => display_ops::op_aon(state),
Op::Aoff => display_ops::op_aoff(state),
Op::Cld => display_ops::op_cld(state),
// ── Phase 21: Sound ─────────────────────────────────────────────────
Op::Beep => sound::op_beep(state),
Op::Tone(n) => sound::op_tone(state, n),
```

**Module declaration** (insert at mod.rs:9-18 alphabetically, with the other `pub mod` lines):
```rust
pub mod display_ops;  // Phase 21
pub mod flags;        // Phase 21
pub mod sound;        // Phase 21
```

**Pitfall 5 — display_override reset at top of dispatch()** (mod.rs:346, just after `flush_entry_buf`):
```rust
pub fn dispatch(state: &mut CalcState, op: Op) -> Result<(), HpError> {
    flush_entry_buf(state)?;
    // Phase 21 D-Pitfall-5: clear stale display override BEFORE op runs.
    // Ops that WRITE the override (VIEW/AVIEW/PROMPT) do so AFTER this reset,
    // so they survive their own dispatch.
    state.display_override = None;
    // ── Phase 3: PRGM mode recording gate (D-03) ─────────...
```

**Difference vs. analog:** `Op::FlagTest { kind, flag }` mirrors `Op::StoArith { reg, kind }` exactly (same struct-variant shape, different sub-enum). The 8 nullary variants (`Op::AView`, `Op::Prompt`, `Op::Aon`, `Op::Aoff`, `Op::Cld`, `Op::Beep`) mirror `Op::PRX` / `Op::PRA` / `Op::Pi` exactly.

**Reuse fraction:** **0.85** — every new variant slots into an existing variant-shape; the enum + dispatch arm template is verbatim.

---

### `hp41-core/src/ops/program.rs` (EXTEND-MATCH `run_loop` + `execute_op`)

**Analog A — skip-next-step:** `Op::Test(kind)` at program.rs:231-235:
```rust
Op::Test(kind) => {
    if !evaluate_test(state, &kind) {
        state.pc += 1; // skip next step (D-09: skip-if-false)
    }
}
```

**Analog B — bool-returning helper variant:** `Op::Isg(reg)` at program.rs:236-244:
```rust
Op::Isg(reg) => {
    if op_isg(state, reg)? {
        state.pc += 1; // loop exit: skip next
    }
}
```

**Analog C — top-level RTN break (PROMPT's exit model):** `Op::Rtn` arm at program.rs:192-197:
```rust
Op::Rtn => {
    match state.call_stack.pop() {
        Some(return_pc) => state.pc = return_pc,
        None => break, // top-level RTN = normal termination
    }
}
```

**Phase 21 new `run_loop` arms** (insert before the `other =>` catch-all at program.rs:246):

```rust
// ── Phase 21: Flag tests (skip next step pattern, mirrors Op::Test) ──
Op::FlagTest { kind, flag } => {
    use crate::ops::flags::{flag_clear, flag_get};
    let is_set = flag_get(state.flags, flag);
    let should_skip = match kind {
        FlagTestKind::IsSet        => !is_set,      // FS?:  skip if NOT set
        FlagTestKind::IsClear      =>  is_set,      // FC?:  skip if NOT clear
        FlagTestKind::IsSetThenClear => {
            // FS?C: skip-if-clear, ALWAYS clear afterward (A4 in research)
            state.flags = flag_clear(state.flags, flag);
            !is_set
        }
        FlagTestKind::IsClearThenClear => {
            // FC?C: skip-if-set, ALWAYS clear afterward
            state.flags = flag_clear(state.flags, flag);
            is_set
        }
    };
    if should_skip {
        state.pc += 1; // skip next step (matches Op::Test idiom)
    }
}
// ── Phase 21: PROMPT — write ALPHA + exit run_loop (mirrors top-level RTN's `break`) ──
Op::Prompt => {
    state.display_override = Some(state.alpha_reg.chars().take(24).collect());
    break; // halt execution; resume via R/S — full STOP/resume in Phase 22
}
```

**Phase 21 new `execute_op` arms** (insert into the giant match at program.rs:275+, before the catch-all `Op::Lbl(_) | …` at line 410):

```rust
// ── Phase 21 (mirror dispatch() arms; Pitfall 2: NEVER add new variants ONLY
// to dispatch() — execute_op MUST list them or the catch-all swallows them silently)
Op::SfFlag(n) => super::flags::op_sf(state, n),
Op::CfFlag(n) => super::flags::op_cf(state, n),
Op::View(r) => super::display_ops::op_view(state, r),
Op::AView => super::display_ops::op_aview(state),
Op::Aon => super::display_ops::op_aon(state),
Op::Aoff => super::display_ops::op_aoff(state),
Op::Cld => super::display_ops::op_cld(state),
Op::Beep => super::sound::op_beep(state),
Op::Tone(n) => super::sound::op_tone(state, n),
```

**Add to the catch-all "programming-only" block** (program.rs:410-417) — `FlagTest` and `Prompt` must be listed here so execute_op rejects them (they belong only in run_loop):

```rust
Op::Lbl(_)
| Op::Gto(_)
| Op::Xeq(_)
| Op::Rtn
| Op::PrgmMode
| Op::Test(_)
| Op::Isg(_)
| Op::Dse(_)
| Op::FlagTest { .. }   // Phase 21 — handled by run_loop directly
| Op::Prompt            // Phase 21 — handled by run_loop directly
=> Err(HpError::InvalidOp),
```

**Difference vs. analog:** `FlagTest`'s pre-skip side-effect (clear-on-FS?C/FC?C) is novel — `Op::Test` is pure read; `FlagTest` writes back to `state.flags` before deciding the skip. Other arms are verbatim copies of `Op::Test` / `Op::Rtn` patterns.

**Reuse fraction:** **0.85** — skip mechanic is verbatim; break-on-PROMPT is verbatim copy of RTN's break; only the inner side-effects differ.

---

### `hp41-cli/src/prgm_display.rs` (EXTEND-MATCH — 12 new arms)

**Analog:** the existing Phase 20 additions at lines 47-51 (`Op::Rnd` → `"RND"`) and parameterized variants at lines 76-78 (`Op::FmtFix(n) => format!("FIX {n}")`).

**Imports update** (prgm_display.rs:7) — extend the use list with `FlagTestKind`:

```rust
use hp41_core::ops::{FlagTestKind, Op, StackReg, StoArithKind};
```

**Phase 21 arms** (insert near end of `op_display_name` match, before the Card Reader arms at line 150):

```rust
// Phase 21: Flags
Op::SfFlag(n) => format!("SF {n:02}"),
Op::CfFlag(n) => format!("CF {n:02}"),
Op::FlagTest { kind, flag } => {
    let mnemonic = match kind {
        FlagTestKind::IsSet => "FS?",
        FlagTestKind::IsClear => "FC?",
        FlagTestKind::IsSetThenClear => "FS?C",
        FlagTestKind::IsClearThenClear => "FC?C",
    };
    format!("{mnemonic} {flag:02}")
},
// Phase 21: Display Control
Op::View(r) => format!("VIEW {r:02}"),
Op::AView => "AVIEW".to_string(),
Op::Prompt => "PROMPT".to_string(),
Op::Aon => "AON".to_string(),
Op::Aoff => "AOFF".to_string(),
Op::Cld => "CLD".to_string(),
// Phase 21: Sound
Op::Beep => "BEEP".to_string(),
Op::Tone(n) => format!("TONE {n}"),
```

**Difference vs. analog:** None — direct copy of the existing format-string idiom. The compound `Op::StoArith { reg, kind }` arm at line 82-90 is the template for the `FlagTest { kind, flag }` arm.

**Reuse fraction:** **0.95** — pure format-string idiom copy.

---

### `hp41-gui/src-tauri/src/prgm_display.rs` (EXTEND-MATCH — IDENTICAL 12 arms)

**Analog:** the same file at lines 47-51 / 96-98 — this file is a **deliberate duplicate** of `hp41-cli/src/prgm_display.rs`. Per CLAUDE.md "SC-4 invariant": `op_display_name` is documented as the SC-4 spirit exception (display formatter, not calculator logic). Every new `Op` variant must be added to BOTH copies in lockstep.

**Action:** copy the exact 12 arms from `hp41-cli/src/prgm_display.rs` (the section immediately above) into the parallel arm block in `hp41-gui/src-tauri/src/prgm_display.rs::op_display_name`. The function signature, imports (`use hp41_core::ops::{Op, StackReg, StoArithKind};` → add `FlagTestKind`), and surrounding arms are already identical.

**Duplication-contract reminder (from CLAUDE.md):**
> `op_display_name` is duplicated in both `hp41-cli/src/prgm_display.rs` and `hp41-gui/src-tauri/src/prgm_display.rs` — every new Op variant must be added in both copies.

**Reuse fraction:** **1.00** — verbatim duplicate of the CLI arms above.

---

### `hp41-core/tests/phase21_flags.rs` / `phase21_display.rs` / `phase21_sound.rs` (NEW-TEST)

**Analog:** `hp41-core/tests/phase20_math.rs` (lines 1-100 shown above). Same shape: module-level `use hp41_core::ops::{dispatch, Op};`, `use hp41_core::{...CalcState, HpError, HpNum};`, helper fns `dec(s) -> Decimal` / `push_x(state, s)` / `push_y_then_x(state, y, x)`, and `#[allow(clippy::unwrap_used)]` at the top.

**Header pattern** (phase20_math.rs:1-20):
```rust
//! Integration tests for Phase 21 (Flags, Display Control & Sound).
//!
//! Covers FN-FLAG-01..02 / FN-DISP-01..05 / FN-SOUND-01..02 success criteria
//! plus per-op happy and error paths.
//!
//! Phase-21 4-place-rule enforcement (SC-5 analog) is compile-time, not runtime.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{dispatch, FlagTestKind, Op};
use hp41_core::{CalcState, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;
```

**Test pattern** (mirror phase20_math.rs:47-65 — Op::Pi test):
```rust
#[test]
fn test_sf_then_fs_q_skips_next_step() {
    let mut state = CalcState::new();
    // Build a tiny program: LBL "T" / SF 05 / FS? 05 / 1 / 2 / RTN
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::SfFlag(5),
        Op::FlagTest { kind: FlagTestKind::IsSet, flag: 5 },
        Op::PushNum(HpNum::from(Decimal::from_str("1").unwrap())),
        Op::PushNum(HpNum::from(Decimal::from_str("2").unwrap())),
        Op::Rtn,
    ];
    hp41_core::ops::program::run_program(&mut state, "T").unwrap();
    // FS? 5 on a set flag = TRUE → execute next step (the "1" push), then "2" push
    // Final stack: X = 2, Y = 1
    assert_eq!(state.stack.x.inner(), Decimal::from_str("2").unwrap());
    assert_eq!(state.stack.y.inner(), Decimal::from_str("1").unwrap());
}
```

**Difference vs. analog:** Phase 20 tests act on stack only (pure math); Phase 21 tests verify `state.flags`, `state.display_override`, and `state.event_buffer` mutations — plus the run_loop skip behavior (requires building a Vec<Op> program + calling `run_program`).

**Reuse fraction:** **0.80** — helper fns + header + assertion shape verbatim; only the body assertion targets change.

---

## Shared Patterns

### Pattern S-1: 4-place rule for new `Op` variants (cross-cutting)

**Source:** CLAUDE.md "v2.0 additions" line: *"Every new `Op` variant must appear in BOTH `dispatch()` in `ops/mod.rs` AND `execute_op()` in `ops/program.rs` AND the `prgm_display.rs` exhaustive match before any caller can compile."*

**Apply to:** all 12 new variants (`SfFlag`, `CfFlag`, `FlagTest`, `View`, `AView`, `Prompt`, `Aon`, `Aoff`, `Cld`, `Beep`, `Tone`). Each lands in exactly four places:

1. `hp41-core/src/ops/mod.rs` — `enum Op` declaration AND `dispatch()` match arm.
2. `hp41-core/src/ops/program.rs` — `execute_op()` match arm (or the catch-all "programming-only" block for `FlagTest` / `Prompt`).
3. `hp41-cli/src/prgm_display.rs` — `op_display_name()` arm.
4. `hp41-gui/src-tauri/src/prgm_display.rs` — `op_display_name()` arm (SC-4 spirit exception).

Compile-time enforcement: Rust exhaustive-match rejects code that misses any of these.

**Pitfall 2 sentinel** (from RESEARCH.md line 250): the catch-all block in program.rs:410-417 silently returns `InvalidOp` for any new variant not explicitly listed before it. Verification grep:

```bash
grep -A 12 "Op::Lbl(_)" hp41-core/src/ops/program.rs | grep -cE "SfFlag|CfFlag|FlagTest|View|AView|Prompt|Aon|Aoff|Cld|Beep|Tone"
```
must return either `0` (ops in normal arms) OR `2` (only `FlagTest` and `Prompt` listed in the catch-all because run_loop handles them).

### Pattern S-2: Buffer-channel for hp41-core I/O-free outputs

**Source:** `hp41-core/src/ops/print.rs:13-18` (op_prx) — push a `String` to a `Vec<String>` field on `CalcState`, return `Ok(())`. No `println!` / `eprintln!` anywhere.

**Apply to:** `sound.rs::op_beep`, `sound.rs::op_tone`. New buffer: `state.event_buffer: Vec<String>` (parallel to `state.print_buffer`).

**Invariant (CLAUDE.md):** `#![deny(clippy::unwrap_used)]` + zero `println!` in `hp41-core`. Phase 21 must preserve both.

### Pattern S-3: `#[serde(default)]` / `#[serde(default, skip)]` decision matrix

**Source:** `hp41-core/src/state.rs:89-118` (Phase 11 + Phase 12 + Card Reader precedents).

**Apply to:**
| New field | serde attr | Why |
|-----------|-----------|-----|
| `flags: u64` | `#[serde(default)]` | Persistent — flag state is user-visible calculator state, survives save/load; default 0 = all-clear |
| `display_override: Option<String>` | `#[serde(default, skip)]` | Transient — cleared at top of dispatch(); never persisted |
| `event_buffer: Vec<String>` | `#[serde(default, skip)]` | Transient — drained by frontend after each dispatch; mirrors print_buffer exactly |

### Pattern S-4: Range-guard before mutation (`if n > LIMIT { return Err(InvalidOp); }`)

**Source:** `hp41-core/src/ops/registers.rs:16-18` (op_sto: `if reg >= 100 { return Err(HpError::InvalidOp); }`) and mod.rs:407-410 (op_fmt_fix: `if n > 9 { return Err(InvalidOp); }`).

**Apply to:** `op_sf` / `op_cf` (`n > 55`), `op_view` (`reg >= 100`), `op_tone` (`n > 9`). Always return `HpError::InvalidOp` (no new error variant needed — confirmed RESEARCH.md line 622: "Phase 21 needs no new variant, reuses InvalidOp").

### Pattern S-5: `apply_lift_effect(state, LiftEffect::Neutral); Ok(())` tail

**Source:** every nullary / parameterized op in `hp41-core/src/ops/*.rs` follows this exact tail. Phase 21 ops are all `LiftEffect::Neutral` (per RESEARCH.md table — flags/display/sound never modify the stack).

**Apply to:** all 11 new op functions (`op_sf`, `op_cf`, `op_view`, `op_aview`, `op_prompt`, `op_aon`, `op_aoff`, `op_cld`, `op_beep`, `op_tone`). The `FlagTest` variant has no op-function — its semantics live in `run_loop`.

## No Analog Found

| File | Role | Reason |
|------|------|--------|
| (none) | — | Every Phase 21 file has a strong analog inside `hp41-core` or its mirror in `hp41-cli/hp41-gui/src/prgm_display.rs`. |

## Deferred Files (Phase 25 / Phase 26 — forward reference only, NOT in Phase 21 plans)

The prompt's "Anticipated new/modified files" list mentions CLI + GUI files. Per RESEARCH.md §Architectural Responsibility Map (line 31) and §Pattern 5 (line 205), these are explicitly **Phase 25 / 26 scope**, not Phase 21. Recorded here so the planner has a forward pointer but does NOT bring them into Phase 21 plans:

| Deferred file | Future phase | Closest existing analog | Notes |
|---------------|--------------|--------------------------|-------|
| `hp41-cli/src/keys.rs` | Phase 25 | existing `key_to_op` entries (digits, ENTER, +/-/×/÷) + `S`/`R` modal openers in `hp41-cli/src/app.rs:511+` | Will add bindings for SF/CF prompts, BEEP, VIEW |
| `hp41-cli/src/app.rs` (PendingInput modal flow) | Phase 25 | existing STO arithmetic modal at app.rs:511+ | New `PendingInput::SfPrompt(String)` / `CfPrompt(String)` variants |
| `hp41-cli/src/help_data.rs` | Phase 25 | existing `HELP_DATA` static table | Add `?` overlay rows for SF/CF/BEEP/VIEW |
| `hp41-cli/src/ui.rs` | Phase 25 | `pending_prompt()` exhaustive match at ui.rs:236-273; `format_entry_buf_display()` | Render `display_override` when present; render event lines |
| `hp41-gui/src-tauri/src/key_map.rs` | Phase 26 | existing stub-error arm at key_map.rs:101-104 | Un-stub `sf_prompt`/`cf_prompt`/`fs_prompt`/`fc_prompt`/`view`/`beep`; add real `resolve_parameterized` prefixes `sf_` / `cf_` / `fs_` / `fc_` / `view_` / `tone_` |
| `hp41-gui/src-tauri/src/types.rs` (CalcStateView) | Phase 26 | existing field additions at types.rs:25-38 (Phase 15 `y_str`/`z_str`/...; Phase 18 `program_steps`/`pc`) | Extend with `display_override: Option<String>` and `event_lines: Vec<String>`; relax JSON budget per FN-GUI-05 (≤500B) |
| `hp41-gui/src/App.tsx` + `Keyboard.tsx` | Phase 26 | existing `KEY_DEFS` placeholders for `sf_prompt`/`cf_prompt`/`fs_prompt`/`beep`/`view`/`aview` already at Keyboard.tsx:75-93 | Frontend wiring; surface display_override; toast pattern for stubs |

**Why deferred:** RESEARCH.md line 205 — "Phase 21 lands the Rust core for `beep`, `view` (and adds `aview`, `prompt`, `aon`, `aoff`, `cld`, `tone`, all 6 flag ops). The stub arm in `key_map.rs` will continue to error for those IDs until Phase 26 — but the core ops will exist and be dispatchable from tests/CLI. Phase 21 does NOT modify `key_map.rs`." This honors the same boundary Phase 20 honored (10 new ops landed in core, still stubbed in `key_map.rs` until Phase 26).

## Metadata

**Analog search scope:**
- `hp41-core/src/ops/{registers,print,program,mod,arithmetic,math}.rs`
- `hp41-core/src/state.rs`, `error.rs`, `format.rs`
- `hp41-core/tests/phase20_math.rs`
- `hp41-cli/src/prgm_display.rs`
- `hp41-gui/src-tauri/src/{prgm_display,types,key_map,commands}.rs`

**Files scanned:** 13 source files + RESEARCH.md + REQUIREMENTS.md
**Re-read budget honored:** every file read at most once, with offset/limit for the two largest (program.rs at 175-260; mod.rs at 1-150 and 150-300 + 320-520).
**Pattern extraction date:** 2026-05-14
