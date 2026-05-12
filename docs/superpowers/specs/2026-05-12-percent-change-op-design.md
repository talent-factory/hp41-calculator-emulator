# Design — HP-41 %CH (Percent Change) Operation

**Date:** 2026-05-12
**Target milestone:** v2.1 Polish (or a standalone follow-up)
**Scope:** add the HP-41 %CH operation end-to-end: `hp41-core` engine + `hp41-cli` keystroke + `hp41-gui` keyboard binding. No SVG button. No base `%` op. No f-shift mechanism.

---

## 1. Operation semantics

**Op variant:** `Op::PctChange`

**Formula:** `result = ((X − Y) / Y) × 100`

Reading direction: `Y` is the *base/before*, `X` is the *new/after*. Entering `200 ENTER 230 %CH` yields `X = 15`, with `Y = 200` preserved for chained calculation.

**Stack effect — HP-41-faithful:**

| Register | After op |
|---|---|
| X | `result` |
| Y | unchanged (the defining feature of the % family) |
| Z | unchanged |
| T | unchanged |
| LASTX | old X |
| `lift_enabled` | `true` |

The stack effect is unary even though the math is binary — `%CH` reads `Y` as base but does NOT consume it. This is the precedent we are setting for the eventual base `%` op.

**Naming:** `PctChange` (not `PCH` / `PctCh`), matching the codebase style of human-readable variants (`YPow`, `Recip`, `SetDeg`, `Clreg`). Leaves room for a future `Op::Pct` without collision.

---

## 2. Implementation layout

### 2.1 `hp41-core/src/num.rs` — pure math method

```rust
/// %CH — percent change from self (base, Y) to new_val (the new value, X).
/// Computes ((new_val − self) / self) × 100.
/// Returns DivideByZero if self is zero; Overflow on intermediate or final overflow.
pub fn checked_pct_change(&self, new_val: &HpNum) -> Result<HpNum, HpError> {
    let delta = new_val.checked_sub(self)?;            // X − Y
    let ratio = delta.checked_div(self)?;              // DivideByZero if Y=0
    ratio.checked_mul(&HpNum::from(100i32))            // × 100
}
```

Convention: `self` = Y (left/base), param = X (right/new) — matches every other binary `HpNum::checked_*` method. As-shipped: the parameter is named `new_val` rather than `x` for symmetry with the rustdoc and to avoid confusion with the stack register `x`.

### 2.2 `hp41-core/src/ops/math.rs` — op fn

Place next to `op_ypow` (the existing binary math op).

```rust
/// %CH: ((X − Y) / Y) × 100, leaving Y on the stack.
/// HP-41 % family — reads Y as base and X as new value, but does NOT
/// consume Y. Stack effect is unary (LASTX←oldX, X←result, Y/Z/T fixed).
/// LiftEffect: Enable (via unary_result).
pub fn op_pct_change(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.y.checked_pct_change(&state.stack.x)?;
    unary_result(state, result);
    Ok(())
}
```

Reusing `unary_result()` rather than introducing a new helper: the stack mechanics are *identical* to a unary op. The two-line comment documents the subtle invariant (binary math, unary stack effect) so a future reader does not refactor it to `binary_result` and silently consume Y.

### 2.3 `hp41-core/src/ops/mod.rs` — Op variant + dispatch

- Add `PctChange` variant to the `Op` enum, in the Phase 2 math block next to `YPow`. Doc-comment: `/// %CH — percent change ((X−Y)/Y)×100; Y preserved. LiftEffect: Enable.`
- Add `op_pct_change` to the `math::{…}` `use` import.
- Add dispatch arm: `Op::PctChange => op_pct_change(state),` near `Op::YPow`.

### 2.4 `hp41-core/src/ops/program.rs` — `execute_op`

`execute_op` is an exhaustive match parallel to `dispatch` (it does NOT delegate). The CLAUDE.md v2.0 invariant requires every new `Op` variant to land in BOTH. Concrete changes:

- Add `op_pct_change` to the `use crate::ops::math::{…}` import block at the top of `execute_op`.
- Add `Op::PctChange => op_pct_change(state),` next to `Op::YPow` in the Phase 2 math block of the match.

### 2.5 `hp41-core/src/ops/mod.rs::synthetic_byte_to_op` — NOT in scope

`%CH` is deliberately *not* added to the synthetic-byte allow-list in this spec. The synthetic-byte path is a security-sensitive subset (Phase 12 invariant T-12-W2-02). A separate one-line follow-up can add `0x4D` (HP-41 NUT byte code for `%CH`, `[ASSUMED]`) if needed.

---

## 3. Error handling

All three error paths emerge from the existing `HpNum::checked_*` chain — no new `HpError` variants needed.

| Condition | Error | Source |
|---|---|---|
| `Y = 0` | `HpError::DivideByZero` | `checked_div` zero guard |
| `X − Y` overflows | `HpError::Overflow` | `checked_sub` |
| `(X − Y) / Y` overflows | `HpError::Overflow` | `checked_div` |
| `… × 100` overflows | `HpError::Overflow` | `checked_mul` |
| `Y < 0` | (no error) | sign emerges from arithmetic naturally |
| `X = Y` | result = `0` | (legitimate, not an error) |

**Invariant — atomicity on error:** on any `Err`, `dispatch()` propagates the error and the stack is **untouched** (no partial write), because `unary_result()` is never reached. This matches `op_ypow` / `op_div` behaviour and is what the CLI/GUI error overlays already expect.

**`?` ordering is deliberate.** `checked_sub` runs before `checked_div`, so an overflowing delta surfaces as `Overflow`, not as `DivideByZero` from a near-zero `Y`.

---

## 4. User-facing surface

### 4.1 CLI — `hp41-cli/src/keys.rs`

```rust
KeyCode::Char('%') => Some(Op::PctChange),
```

Crossterm delivers `Shift+5` as `KeyCode::Char('%')`, which is the same mechanism the existing `'S'`, `'L'`, `'+'` bindings rely on — no modifier check needed.

`KEY_REF_TABLE` entry:

```rust
("%", "%CH (percent change: ((X−Y)/Y)×100, Y preserved)"),
```

`keycode_to_hp41_code()`: **no change.** `%` has no physical HP-41C key (the hardware uses `f Σ+` for `%` and `f 1/x` for `%CH`); returning `None` matches the existing convention for TUI-only bindings and prevents `GetKey` from reading a misleading code.

### 4.2 CLI help overlay — `hp41-cli/src/help_data.rs`

Add a `%` row to `HELP_DATA` (single source of truth for the `?` overlay). Match the existing tone of nearby entries.

### 4.3 GUI key map — `hp41-gui/src-tauri/src/key_map.rs`

```rust
"pct_change" => Some(Op::PctChange),
```

SC-4 invariant preserved: this is a string-ID → `Op` lookup, no math leaks into the GUI crate.

### 4.4 GUI physical-keyboard listener — `hp41-gui/src/App.tsx`

The existing keyboard listener routes typed characters through `resolveKeyId`. Add `'%'` → `"pct_change"` there. The `busyRef` debounce already covers concurrent invokes; nothing else changes.

### 4.5 GUI SVG keyboard — `hp41-gui/src/Keyboard.tsx`

**No change.** Preserves the 44-key pixel-perfect HP-41C skin. `%CH` is reachable from the physical keyboard via the GUI's typed-key listener.

### 4.6 Program listing display — both `prgm_display.rs` copies

Add an arm to the exhaustive `op_display_name` match in BOTH copies:

- `hp41-cli/src/prgm_display.rs`
- `hp41-gui/src-tauri/src/prgm_display.rs`

```rust
Op::PctChange => "%CH",
```

This is required by the CLAUDE.md v2.0 invariant — every new `Op` variant lands in both copies before any caller can compile.

---

## 5. PRGM mode and serialization

- **Recording:** `dispatch()` already gates on `state.prgm_mode` before the match arm; `Op::PctChange` is appended to `state.program` like any other op. No extra wiring needed.
- **Playback:** `execute_op()` arm added per §2.4.
- **Persistence:** the `Op` enum derives `Serialize` / `Deserialize`. Adding a variant is forward-compatible with v1.x save files (they cannot contain `PctChange` and so cannot fail to deserialize); a v2.1 save containing `PctChange` would fail to load on an older binary, which is expected and matches every other variant added since v1.0.

---

## 6. Test plan

### 6.1 HpNum unit tests — `hp41-core/src/num.rs`

| Case | Y | X | Expected |
|---|---|---|---|
| +10% | 100 | 110 | 10 |
| −10% | 100 | 90 | −10 |
| +50% | 80 | 120 | 50 |
| 0% (equal) | 42 | 42 | 0 |
| Negative base | −100 | −80 | −20 |
| Sign across zero | 10 | −5 | −150 |
| Tiny delta (precision) | 1e9 | 1e9 + 1 | 1e−7 |
| Y = 0 | 0 | 42 | `Err(DivideByZero)` |
| Overflow at `×100` | 1 | 1e+27 | `Err(Overflow)` |

### 6.2 Op-level integration tests — `hp41-core/src/ops/math.rs`

- **`pct_change_preserves_y` (the defining test):** pre-state `Y=200, X=230, Z=z₀, T=t₀`; post-state `X=15 AND Y=200 AND Z=z₀ AND T=t₀`.
- **LASTX captures old X:** pre-state `X=230`; post-state `lastx == 230`.
- **lift_enabled is `true` after op:** a subsequent `enter_number` lifts (does not overwrite) the result.
- **Error path is atomic:** pre-state snapshot `(x,y,z,t,lastx)` with `Y=0, X=42`; after `dispatch` returns `Err(DivideByZero)`, the snapshot must equal the post-state.
- **LASTX round-trip:** sequence `200 ENTER 230 %CH LASTX` leaves `X=230` (verifies the new op wires LASTX correctly).
- **Repeat invocation chains correctly:** dispatching `Op::PctChange` twice in a row recomputes percent change against the still-present Y on the second call.

### 6.3 PRGM mode tests — `hp41-core/src/ops/program.rs` (or sibling)

- **Recording:** with `prgm_mode = true`, dispatching `Op::PctChange` appends to `state.program` and leaves the stack untouched.
- **Playback:** running `[LBL "T", PushNum(200), Enter, PushNum(230), PctChange, Rtn]` via `run_program` yields `X=15, Y=200`.

### 6.4 CLI keystroke tests — `hp41-cli/src/keys.rs` `#[cfg(test)]`

- `KEY_REF_TABLE` contains the `%` row.
- Dispatching `Op::PctChange` on a state seeded `Y=100, X=125` yields `X=25, Y=100`.

### 6.5 GUI key-map tests — `hp41-gui/src-tauri/src/key_map.rs` `#[cfg(test)]`

- `resolve("pct_change") == Some(Op::PctChange)`.
- Smoke through the Tauri command path: `handle_op(state, "pct_change")` with `Y=50, X=75` produces a `CalcStateView` with `x == "25"` and `y == "50"`.

### 6.6 Numerical-accuracy suite — `hp41-core/tests/numerical_accuracy.rs`

Add 3 `%CH` cases. As-shipped: the suite is now 503 cases (500 + 3 `%CH`); the gate was tightened proportionally from `passes >= 490` to `passes >= 493` to maintain the 98% policy strength at the new total. New cases must all pass deterministically.

### 6.7 Coverage

New surface is ~15 LoC in `num.rs` + ~10 LoC in `math.rs` + ~5 LoC across dispatch / execute_op / display matches. The test set above gives ≥95% line/region coverage on those lines — well above the 80% workspace gate.

---

## 7. Out of scope (deliberately deferred)

- **Base `%` operation.** Spec sets the conventions (Op naming, math.rs home, `unary_result` for "preserve Y" semantics) so base `%` lands cleanly later — but is not implemented here.
- **f-shift mechanism on the GUI.** The HP-41 hardware reaches `%` / `%CH` via the gold f-key. Modelling that is a v2.1 Polish design problem on its own.
- **SVG button for %CH on the GUI skin.** Preserves the 44-key pixel-perfect HP-41C layout.
- **`%CH` in the Phase 12 synthetic-byte allow-list.** Security-sensitive subset; trivially added later.

---

## 8. Files touched (summary)

**hp41-core:**
- `src/num.rs` — add `HpNum::checked_pct_change` + unit tests
- `src/ops/math.rs` — add `op_pct_change` + integration tests
- `src/ops/mod.rs` — add `Op::PctChange` variant + import + dispatch arm
- `src/ops/program.rs` — add `Op::PctChange` arm in `execute_op` + PRGM tests
- `tests/numerical_accuracy.rs` — add `%CH` cases

**hp41-cli:**
- `src/keys.rs` — add `%` binding + `KEY_REF_TABLE` entry + tests
- `src/help_data.rs` — add `%` row to `HELP_DATA`
- `src/prgm_display.rs` — add `Op::PctChange => "%CH"` arm

**hp41-gui:**
- `src-tauri/src/key_map.rs` — add `"pct_change"` arm + tests
- `src-tauri/src/prgm_display.rs` — add `Op::PctChange => "%CH"` arm
- `src/App.tsx` — add `'%'` → `"pct_change"` case in the keyboard listener
- `src/Keyboard.tsx` — *no change*

No changes required in `hp41-core/src/state.rs`, `stack.rs`, or `error.rs`.
