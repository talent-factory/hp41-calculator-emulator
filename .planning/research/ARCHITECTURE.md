# Architecture Patterns

**Domain:** HP-41 Calculator Emulator — v3.0 Math 1 Pac Emulation (XROM-Module Framework)
**Researched:** 2026-05-16
**Focus:** How `Math 1` ROM functions (matrix, complex, polynomial root, INTEG, SOLVE, vector) integrate into the LIVE v1.0–v2.2 architecture without violating SC-4, the 4-exhaustive-match invariant, the CLI ↔ GUI parity invariant (D-25.6), or the JSON-canonical pipeline (D-25.16).

---

## Summary

The v3.0 architecture is **integration, not redesign**. Every load-bearing pattern was settled in v1.0 (CalcState single-source-of-truth, dispatch + LiftEffect, prgm_mode gate), reinforced in v1.1 / v2.0 (CalcStateView IPC, key_map.rs resolver, print_buffer drain pattern), and proven again in v2.2 (hybrid PendingInput struct-variants, JSON-canonical help pipeline, f-prefix one-shot model, builtin_card_op extension exception). Math 1 inherits all of them.

**The chosen Op-strategy is Option A — one new `Op` variant per Math 1 function**, with a single accommodation: every Math 1 op MUST carry its 2-digit XROM number in a const lookup table so future modules (v3.1 Stat 1 / v3.2 Time / v3.3 Advantage) can ship the same way without altering hp41-core enum dispatch. The 4-exhaustive-match invariant is preserved (rust compile errors continue to block missing arms in `dispatch()` / `execute_op()` / `hp41-cli/src/prgm_display.rs` / `hp41-gui/src-tauri/src/prgm_display.rs`). Option B (`Op::XromCall(u16)` table dispatch) is rejected because (a) it forfeits exhaustive-match safety net that has caught dozens of bugs across Phases 1–27, (b) it pushes type erasure into hp41-core which conflicts with `#![deny(clippy::unwrap_used)]`, and (c) every Math 1 op needs hand-written behavioral emulation anyway — there is no payload-only "data" op that benefits from table dispatch. See §"Op-Strategy Decision" below.

**The five new structural pieces** are minimal:
1. `hp41-core/src/ops/math1/` — new module hosting ~40 Math 1 op functions, structured per family (`matrix.rs`, `complex.rs`, `poly.rs`, `integ.rs`, `solve.rs`, `vector.rs`).
2. **Six new CalcState fields**, all `#[serde(default)]` (the v1.0+ backward-compat pattern proven through 5 milestones): `complex_mode: bool`, `matrix_dim: Option<(u8, u8)>`, `matrix_active_reg: Option<u8>`, `integ_state: Option<IntegState>`, `solve_state: Option<SolveState>`, `xrom_modules: u8` (bitfield, default `0b0000_0001` = Math 1 loaded).
3. `docs/hp41-math1-functions.json` — sibling file to `hp41cv-functions.json`, identical schema, **plus** an `xrom: { module: "Math 1", id: "01,01" }` object per entry. `scripts/docs-matrix/` extended to consume BOTH files.
4. `hp41-core/src/ops/xrom.rs` — small registry (~50 LOC): `xrom_id_of(op: &Op) -> Option<(u8, u8)>` + `module_loaded(state, module: XromModule) -> bool`. Used by `CATALOG 2`, by `Op::Xeq` fallback chain extension, and (in v3.1+) by indirect XEQ via XROM number.
5. **`run_program` is extended for re-entrancy** — a new `Op::Integ` / `Op::Solve` execution path that calls back into `run_loop` with a saved-and-restored caller frame. The 4-deep `call_stack` limit is preserved; INTEG/SOLVE consume ONE level for the user's function (matches Owner's Manual).

**What does NOT change:** the `Op` enum lives in the same place. `dispatch()` keeps its single-match shape. `flush_entry_buf` is unchanged. The GUI `key_map::resolve` keeps its current two-tier structure (bare-name match, then `resolve_parameterized`, then stub-error arm). The JSON pipeline keeps its single-source-of-truth semantics — Math 1 just gets its own JSON file. The card-reader-style frontend-drain pattern is reused unchanged for any I/O (no Math 1 op needs frontend I/O in v3.0 — all are pure compute).

**SC-4 remains intact.** All math logic lands in `hp41-core/src/ops/math1/`. The hp41-gui side gets only resolver entries (string-id → Op) — no `fn op_*` / `fn flush_entry_*` / `fn format_hpnum` duplication. The stricter SC-4 grep pattern (`op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)`) continues to match nothing in `hp41-gui/src-tauri/src/` after v3.0 ships.

**CLI ↔ GUI parity (D-25.6) remains intact.** The f-prefix one-shot model (`App.shift_armed` on CLI, `shiftActive` on GUI), the modal `PendingInput` hybrid struct-variants, and the XEQ-by-name modal-resolver pair (`xeq_by_name_local_resolve` + `builtin_card_op`) keep their bit-for-bit symmetry. Math 1 ops reach the user EXCLUSIVELY through XEQ-by-name (no dedicated keys); the existing modal infrastructure carries them with one targeted extension to the resolver: the same `Op::Xeq("M+")` fallback chain that today resolves to `builtin_card_op` is widened to ALSO try `xrom_resolve` after `builtin_card_op` returns `None`.

---

## Data Flow

### 1. Math 1 function invocation (XEQ-by-name path)

The user has NO dedicated keys for Math 1 — every function is reached via `XEQ "NAME"`. The existing v2.2 modal infrastructure carries this end-to-end:

```
CLI keyboard:                       GUI click:
  ┌─────────────────────┐           ┌─────────────────────┐
  │ user types "X"      │           │ user clicks "XEQ"   │
  │   (XEQ key)         │           │   (top-row)         │
  └──────────┬──────────┘           └──────────┬──────────┘
             ▼                                 ▼
  PendingInput::XeqPrompt(acc)       App.tsx opens XEQ modal
             │                                 │
  user types "M","+","\n"            user types "M","+","Enter"
             ▼                                 ▼
  ┌─────────────────────────────────────────────────┐
  │ Resolver chain (CLI + GUI identical) per D-25.6 │
  │                                                 │
  │  1. find_in_program(state.program, "M+")?       │  ← user LBL (always wins)
  │  2. xeq_by_name_local_resolve("M+")?            │  ← v2.2 8 conditional tests
  │  3. builtin_card_op("M+")?                      │  ← v2.1 4 card-reader names +
  │                                                 │     v2.2 8 conditionals (ASCII+Unicode)
  │  4. xrom_resolve("M+")?           ◄── NEW v3.0  │  ← Math 1 registry
  │  5. HpError::InvalidOp                          │
  └─────────────────────────────────────────────────┘
                       │
                       ▼ Some(Op::MatPlus)
             ┌──────────────────────┐
             │ dispatch(state, op)  │  ← unchanged hp41-core entry point
             └──────────┬───────────┘
                        ▼
             ┌──────────────────────────────────┐
             │ match Op::MatPlus =>             │
             │   math1::matrix::op_mat_plus()   │  ← new file
             │     ... reads regs[matrix_dim],  │
             │     ... writes regs[active_reg]  │
             │     apply_lift_effect(state,     │
             │       LiftEffect::Neutral)       │
             └──────────┬───────────────────────┘
                        ▼
              CalcStateView returned to frontend
              (existing IPC contract — unchanged)
```

**Each Math 1 op is a pure `(&mut CalcState) -> Result<(), HpError>` function.** No new IPC surface, no new commands, no new types crossing the Tauri boundary.

### 2. User-callback re-entrant path (INTEG / SOLVE)

INTEG and SOLVE call back into a user-defined LBL program. This is the only architecturally novel piece of v3.0 — everything else is "more of the same."

```
  XEQ "INTEG" called with user fn "F" on stack args (a, b, ε)
            │
            ▼
  ┌────────────────────────────────────────────────────────────┐
  │ run_loop reaches Op::Integ                                 │
  │   1. Pre-checks: state.call_stack.len() ≤ 3 (≥ 4 → Err)    │  ← preserves 4-deep cap
  │   2. Validate user fn label exists: find_in_program("F")?  │
  │   3. Snapshot caller state into IntegState:                │
  │        { user_label, a, b, eps, saved_pc, saved_x..t,      │
  │          rom_iter_state: RombergSamples::new() }           │
  │   4. state.integ_state = Some(IntegState { .. })           │  ← parked on CalcState
  │   5. Adaptive-quadrature outer loop (Romberg-style):       │
  │        - Pick next sample point x_k                        │
  │        - Push x_k onto stack via enter_number              │
  │        - state.call_stack.push(saved_pc)  ◄── 1 level used │
  │        - state.pc = label_target("F") + 1                  │
  │        - run_loop recurses (same fn, same MAX_STEPS guard) │  ← re-entrant
  │        - On RTN: caller frame popped, x_k result in X      │
  │        - Accumulate into rom_iter_state                    │
  │        - Convergence check (|delta| < eps) → exit loop     │
  │   6. Restore caller stack from snapshot                    │
  │   7. Push integral result into X (LiftEffect::Enable)      │
  │   8. state.integ_state = None                              │
  └────────────────────────────────────────────────────────────┘
```

**Re-entrancy invariants:**
- `state.is_running` STAYS `true` for the whole recursion — only one logical "run" is in flight.
- The 4-deep `call_stack` cap is enforced PRE-mutation per the v2.2 D-22.15 pattern (matches `Op::XeqInd` precedent at `hp41-core/src/ops/program.rs:479`).
- `MAX_STEPS` (1_000_000 per `run_program` call) remains the outer safety net; each user-function evaluation counts against it.
- `state.integ_state` / `state.solve_state` are `Option<…>` with `#[serde(default, skip)]` — transient, NEVER persisted (matches `print_buffer`, `display_override`, `event_buffer` precedent).
- Nested INTEG-inside-SOLVE-inside-INTEG is REJECTED at op entry with `HpError::InvalidOp` when `state.integ_state.is_some() || state.solve_state.is_some()` (Phase boundary — defer multi-level numeric nesting to v3.x backlog if ever needed). Documented divergence: the real HP-41 Math 1 ROM has the same restriction (one INTEG, one SOLVE).

---

## New Components

### Modified (workspace-level)

| File | Change | Reason |
|------|--------|--------|
| `Cargo.toml` (root) | No changes | hp41-gui stays nested-standalone; no new workspace member |
| `justfile` | Extend `docs-matrix` recipe to run twice (hp41cv + math1) OR pass both JSON paths to a single `docs-matrix` invocation | Math 1 JSON regeneration |

### New module: `hp41-core/src/ops/math1/`

Standard hp41-core sub-module layout, mirrors `ops/registers.rs` / `ops/math.rs` precedent. One file per Math 1 function family:

```
hp41-core/src/ops/math1/
  mod.rs           ← pub mod ... ; re-exports op_mat_plus, op_c_add, etc.
  matrix.rs        ← M+, M-, MAT*, INV, TRANS, DET, MAT/, MDIM, GETM, PUTM, ...
  complex.rs       ← CADD, CSUB, CMUL, CDIV, CABS, CARG, CCHS, CCONJ, CLN, CEXP, ...
  poly.rs          ← PROOT (quadratic, cubic, quartic + Bairstow for n>4)
  vector.rs        ← V+, V-, VDOT, VABS (magnitude), VxV (cross product if covered)
  integ.rs         ← INTEG (adaptive Romberg or Simpson w/ user-fn callback)
  solve.rs         ← SOLVE (secant w/ user-fn callback)
  xrom.rs          ← xrom_id_of(&Op) -> Option<(u8, u8)>; module_loaded()
```

**`hp41-core/src/lib.rs`** adds `pub use ops::math1::{IntegState, SolveState, XromModule};` (only if needed for tests; otherwise keep them internal).

### Modified: `hp41-core/src/state.rs`

Six new fields on `CalcState`, ALL with `#[serde(default)]`, three with `#[serde(skip)]` because they are transient. Backward compat with v1.0–v2.2 save files is preserved exactly the way v1.1 / v2.0 / v2.1 / v2.2 added theirs.

```rust
// ── v3.0: XROM module emulation ──────────────────────────────────────────
/// Bitfield of loaded XROM modules. Bit 0 = Math 1, bit 1 = Stat 1 (v3.1),
/// bit 2 = Time (v3.2), bit 3 = Advantage (v3.3). v3.0 ships with bit 0
/// hard-coded (Math 1 always loaded). Bits 4..7 reserved.
/// `#[serde(default)]` => v1.0–v2.2 save files load with 0; setup() in
/// hp41-gui / hp41-cli main() promotes 0 → 0b1 on the first dispatch after
/// load (single-write back-fill, NOT a save-file migration).
#[serde(default = "default_xrom_modules")]
pub xrom_modules: u8,

/// Complex-mode flag (Math 1 §3): when true, Math 1 complex ops read/write
/// Y+iX (1-stack-pair convention) per Owner's Manual; when false, they read
/// X+iY from regs[reg_pair_lo]+regs[reg_pair_hi] (2-register convention).
/// Default false (matches HP-41C/CV cold-start).
#[serde(default)]
pub complex_mode: bool,

/// Active matrix dimension (rows, cols). Set by `MDIM`; consumed by all
/// matrix ops. None = no matrix declared (matrix ops Err::InvalidOp).
#[serde(default)]
pub matrix_dim: Option<(u8, u8)>,

/// Active matrix base register pointer. Set by `MDIM`. Matrix lives in
/// regs[matrix_active_reg .. matrix_active_reg + rows*cols] (column-major
/// per Math 1 ROM convention — RESEARCH FEATURES §"Matrix storage").
#[serde(default)]
pub matrix_active_reg: Option<u8>,

/// In-flight INTEG state. Some(_) iff a user-callback recursion is active.
/// Transient — never persisted.
#[serde(default, skip)]
pub integ_state: Option<crate::ops::math1::integ::IntegState>,

/// In-flight SOLVE state. Some(_) iff a user-callback recursion is active.
/// Transient — never persisted.
#[serde(default, skip)]
pub solve_state: Option<crate::ops::math1::solve::SolveState>,

fn default_xrom_modules() -> u8 { 0b0000_0001 } // Math 1 loaded
```

### Modified: `hp41-core/src/ops/mod.rs`

The `Op` enum grows by ~40 variants. Three exhaustive matches in the same file (`dispatch()`) plus two more outside it (`execute_op()` in `program.rs`, `prgm_display.rs` ×2) gain the new arms. The compiler enforces all four — adding a new variant without all four arms is a compile error (the proven v1.0–v2.2 safety net).

```rust
pub enum Op {
    // ... existing v1.0–v2.2 variants ...

    // ── v3.0 Math 1: Matrix family ────────────────────────────────────────
    /// MDIM — declare matrix at regs[X] with dimensions Y (rows) × X-low (cols).
    /// Writes state.matrix_dim + state.matrix_active_reg. LiftEffect: Neutral.
    MDim,
    /// M+ — matrix add. Reads matrix at active_reg, adds matrix at regs[X-pointer].
    /// LiftEffect: Neutral. Phase 28 (FN-MATH1-MAT-01).
    MatPlus,
    MatMinus,
    MatMul,
    MatDiv,
    /// INV — matrix inverse in place. Domain error if det < eps. LiftEffect: Neutral.
    Inv,
    Trans,
    Det,
    Getm,
    Putm,

    // ── v3.0 Math 1: Complex family ───────────────────────────────────────
    CAdd, CSub, CMul, CDiv, CAbs, CArg, CChs, CConj, CLn, CExp,
    CSin, CCos, CTan,  // optional — gauge by FEATURES.md scope

    // ── v3.0 Math 1: Polynomial root ──────────────────────────────────────
    PRoot,

    // ── v3.0 Math 1: Vector family ────────────────────────────────────────
    VPlus, VMinus, VDot, VAbs,

    // ── v3.0 Math 1: Numeric callback ops ─────────────────────────────────
    /// INTEG — adaptive numeric integration of user LBL on the stack.
    /// Stack at entry: T=untouched, Z=eps, Y=a, X=b. LBL name in ALPHA.
    /// LiftEffect: Enable (result replaces X via lift-then-push idiom).
    /// REQUIRES `state.is_running == true` (i.e. inside run_program) OR
    /// CLI/GUI top-level XEQ path that constructs a transient run.
    Integ,
    /// SOLVE — secant-method root finder on user LBL.
    /// Stack at entry: Y=initial_guess_1, X=initial_guess_2. LBL in ALPHA.
    /// LiftEffect: Enable. Same re-entrancy rules as Integ.
    Solve,
}
```

Every variant gets:
1. An arm in `dispatch()` (e.g. `Op::MatPlus => math1::matrix::op_mat_plus(state)`).
2. An arm in `execute_op()` (`hp41-core/src/ops/program.rs:623`). Most arms delegate to the same function. INTEG/SOLVE are SPECIAL — they need run-loop access, so they live in `run_loop`'s `match op` block (around `hp41-core/src/ops/program.rs:453`), NOT in `execute_op`. The dispatch arm errors with `HpError::InvalidOp` if called outside `run_loop` (mirrors `Op::GtoInd` / `Op::XeqInd` precedent at line 839).
3. An arm in `hp41-cli/src/prgm_display.rs::op_display_name` for program listings.
4. An arm in `hp41-gui/src-tauri/src/prgm_display.rs::op_display_name` for program listings.

### New: `hp41-core/src/ops/math1/xrom.rs`

```rust
use crate::ops::Op;

/// XROM module identifier. Bit position in `state.xrom_modules`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XromModule {
    Math1,    // bit 0
    Stat1,    // bit 1 (v3.1)
    Time,     // bit 2 (v3.2)
    Advantage,// bit 3 (v3.3)
}

/// Return the (module-id, function-id) pair for a Math 1 / Stat 1 / etc op.
/// Used by CATALOG 2 and by the XEQ-by-name resolver fallback.
pub fn xrom_id_of(op: &Op) -> Option<(u8, u8)> {
    match op {
        // Module 01 = Math 1; function IDs per HP-41C/CV Math Pac manual.
        // (Exact numbers verified against owner's manual in Phase 28-research.)
        Op::MDim     => Some((1, 1)),
        Op::MatPlus  => Some((1, 2)),
        Op::MatMinus => Some((1, 3)),
        Op::MatMul   => Some((1, 4)),
        // ... etc.
        _ => None,
    }
}

/// Resolve an HP-41 mnemonic (case-sensitive) to its Op IF the owning module
/// is loaded in `state.xrom_modules`. v3.0: only Math 1.
pub fn xrom_resolve(name: &str, modules: u8) -> Option<Op> {
    if modules & 0b0000_0001 != 0 {
        if let Some(op) = math1_resolve(name) { return Some(op); }
    }
    // v3.1+: if modules & 0b10 != 0 { stat1_resolve(name) }
    None
}

fn math1_resolve(name: &str) -> Option<Op> {
    match name {
        "MDIM"   => Some(Op::MDim),
        "M+"     => Some(Op::MatPlus),
        "M-"     => Some(Op::MatMinus),
        "MAT*"   => Some(Op::MatMul),
        "MAT/"   => Some(Op::MatDiv),
        "INV"    => Some(Op::Inv),
        "TRANS"  => Some(Op::Trans),
        "DET"    => Some(Op::Det),
        "GETM"   => Some(Op::Getm),
        "PUTM"   => Some(Op::Putm),
        "CADD" | "C+" => Some(Op::CAdd),
        "CSUB" | "C-" => Some(Op::CSub),
        // ... etc, matching Owner's Manual spellings + Unicode variants
        // (mirrors v2.2 ASCII+Unicode dual-spelling precedent at keys.rs:347)
        "INTEG"  => Some(Op::Integ),
        "SOLVE"  => Some(Op::Solve),
        _ => None,
    }
}
```

### Modified: `hp41-core/src/ops/program.rs`

Three changes, all small and bounded:

1. **`builtin_card_op` fallback chain extension** (line ~511 in current code). The existing v2.2 dispatch path:
   ```rust
   if let Some(card_op) = builtin_card_op(&label) {
       crate::ops::dispatch(state, card_op)?;
   } else {
       return Err(HpError::InvalidOp);
   }
   ```
   becomes:
   ```rust
   if let Some(card_op) = builtin_card_op(&label) {
       crate::ops::dispatch(state, card_op)?;
   } else if let Some(xrom_op) = crate::ops::math1::xrom::xrom_resolve(
                                    &label, state.xrom_modules) {
       crate::ops::dispatch(state, xrom_op)?;
   } else {
       return Err(HpError::InvalidOp);
   }
   ```
   This extends `Op::Xeq("M+")` inside a saved program to resolve through the Math 1 registry, NOT just card-reader names. The same `Op::Xeq` programmatic-symmetry invariant from v2.2 Plan 25-03 carries over to v3.0. The `xeq_by_name_local_resolve` CLI fast-path (`hp41-cli/src/keys.rs:347`) gains the same fallback at its `_ => None` arm.

2. **`run_loop` match block gains INTEG / SOLVE arms** (around line 453). These cannot be in `execute_op` because they need program-vec access for the user-fn label lookup AND they manipulate `state.pc` / `state.call_stack`:
   ```rust
   Op::Integ => {
       math1::integ::run_integ(state, program)?;
       // run_integ internally recurses into run_loop; on return, x = integral.
   }
   Op::Solve => {
       math1::solve::run_solve(state, program)?;
   }
   ```

3. **`execute_op` gets stub arms for INTEG / SOLVE** that return `HpError::InvalidOp` — mirrors the `Op::Prompt` / `Op::Stop` precedent at line 600. INTEG/SOLVE are run-loop-only ops; reaching them via direct execute is a logic bug.

### Modified: `hp41-cli/src/keys.rs`

`xeq_by_name_local_resolve` (line 347) gains the same xrom fallback as the run_loop:
```rust
pub fn xeq_by_name_local_resolve(name: &str) -> Option<Op> {
    match name {
        // ... existing 8 conditional tests ...
        _ => {
            // v3.0 fallback chain: card-reader → xrom. Mirrors run_loop fallback
            // at hp41-core/src/ops/program.rs:511 for keyboard ↔ programmatic
            // parity per D-25.6.
            None  // unchanged; caller's modal Enter-arm tries builtin_card_op
                  // then xrom_resolve. See pending_input.rs::handle_xeq_enter.
        }
    }
}
```

The modal `Enter` handler in `hp41-cli/src/app.rs` (the XEQ-by-name modal) gets ONE new fallback line after the existing `builtin_card_op` check — keeps the resolver order consistent with the run_loop path.

### Modified: `hp41-gui/src-tauri/src/key_map.rs`

**No new bare-id arms.** Math 1 functions have NO dedicated keys — they reach Op only through the XEQ-by-name modal which already exists. The stub-error arm (`hp41-gui/src-tauri/src/key_map.rs:158`) is UNCHANGED in v3.0: it still lists `"asn"`, `"catalog"`, `"view"`, `"tone"`, and the 13 `*_prompt` ids that are intercepted by `App.tsx` modal handlers. **It does NOT shrink** because none of those ids were Math 1 to begin with.

The stub-error arm in v3.0 means "frontend modal infrastructure pending" — distinct from "v3.x module pac function". With Math 1 ROM ops reaching the user through XEQ-by-name only, none of them appear as direct clickable keys in `Keyboard.tsx`'s `KEY_DEFS`. The string-id resolver therefore needs no Math 1 entries at all. SC-4 trivially holds.

If v3.0 chooses to add the four "Math 1 quick keys" some hardware overlays support (uncommon), those would be added as `"mdim"`, `"m_plus"`, `"inv"`, `"trans"` etc. arms in `key_map::resolve` mapping to `Ok(Op::MDim)` / `Ok(Op::MatPlus)` / etc. — pure dispatch entries, no math logic. SC-4 still holds.

### Modified: `hp41-cli/src/prgm_display.rs` + `hp41-gui/src-tauri/src/prgm_display.rs`

Both `op_display_name` exhaustive matches gain ~40 arms returning the HP-41 mnemonic strings ("M+", "MAT*", "INV", "INTEG", "SOLVE", "C+", "PROOT", "V+", "VDOT", ...). These are PURE display formatters — they are precisely the kind of duplication CLAUDE.md's SC-4 spirit explicitly allows (the existing `fn op_display_name` is already duplicated in both copies per the CLAUDE.md note).

### New: `docs/hp41-math1-functions.json`

Sibling to `docs/hp41cv-functions.json`, identical schema **plus** an `xrom` object:

```json
{
  "op_variant": "MatPlus",
  "display_name": "M+",
  "category": "Math1-Matrix",
  "status": "implemented",
  "phase": "28",
  "key_path": "XEQ \"M+\"",
  "description": "Matrix add: result_matrix <- active_matrix + matrix at regs[X]",
  "xrom": { "module": "Math 1", "module_id": 1, "function_id": 2 },
  "divergences": []
}
```

`scripts/docs-matrix/src/main.rs` is extended to accept multiple JSON inputs and emit one matrix per module (or one combined matrix with a `Module` column added — TBD by Phase 28-03 plan):

```bash
docs-matrix --in docs/hp41cv-functions.json
           --in docs/hp41-math1-functions.json
           --out docs/hp41cv-function-matrix.md
           --out docs/hp41-math1-function-matrix.md
```

`hp41-cli/src/help_data.rs` gains a SECOND `OnceLock<Vec<HelpEntry>>` for Math 1, loaded from a second `include_str!` of `docs/hp41-math1-functions.json`. The `?` overlay merges both lists; the right-panel KEY_REF_TABLE derivation filters out entries with `key_path == null` (Math 1 entries all carry `"XEQ \"…\""` strings, NOT empty — they ARE keyboard-reachable, just through the modal). The bidirectional Op-enum ↔ JSON parity tests in `hp41-cli/tests/function_matrix_parity.rs` gain a Math 1 counterpart in `hp41-cli/tests/function_matrix_parity_math1.rs`.

### New: `hp41-core/tests/math1_*.rs`

Per-family test files mirroring the v2.2 plan-test split:
- `hp41-core/tests/math1_matrix.rs` — M+, M-, MAT*, INV, TRANS, DET, MDIM, GETM, PUTM
- `hp41-core/tests/math1_complex.rs` — CADD, CSUB, CMUL, CDIV, CABS, CARG, CCHS, CCONJ
- `hp41-core/tests/math1_poly.rs` — PROOT (quadratic exact, cubic Cardano, quartic Ferrari/numeric)
- `hp41-core/tests/math1_integ.rs` — INTEG: ∫sin from 0..π = 2.0 ± eps; nested-INTEG rejection; user-fn-throws propagation
- `hp41-core/tests/math1_solve.rs` — SOLVE: root of x²−2 ≈ √2; non-convergent rejection; user-fn-throws propagation
- `hp41-core/tests/math1_vector.rs` — V+, V-, VDOT, VABS
- `hp41-core/tests/math1_xrom_registry.rs` — `xrom_resolve` round-trip; `xrom_id_of` round-trip; bit-0-gated resolution (modules=0 returns None)
- `hp41-core/tests/numerical_accuracy.rs` — extended from 566 to ~700+ cases (per FEATURES.md plan); citations against Free42 / Owner's Manual maintained per D-27.7

---

## Op-Strategy Decision

### Option A (CHOSEN): One `Op` variant per Math 1 function

```rust
pub enum Op {
    // ... ~140 v2.2 variants ...
    MatPlus, MatMinus, MatMul, Inv, Trans, Det,
    CAdd, CSub, CMul, CDiv, CAbs, CArg,
    PRoot,
    VPlus, VMinus, VDot, VAbs,
    Integ, Solve,
    // ... ~40 new variants total ...
}
```

**Pros:**
- The 4-exhaustive-match invariant is preserved. Adding a Math 1 op without all four arms is a compile error — the same safety net that caught dozens of bugs in Phases 1–27.
- Zero new dispatch indirection. `dispatch()` keeps O(1) variant dispatch shape; `match` jump-table optimization remains intact (criterion benchmark `key_latency` continues to gate ≤ 50 ms).
- Op-enum is the documented contract surface for the JSON parity tests (`function_matrix_parity.rs`) — every JSON entry has a corresponding Op-enum variant, hard-checked at CI time. Option B would require a parallel `xrom_id_of` mapping AND a JSON pipeline that knows about ids-without-variants, doubling the source-of-truth.
- Save-file backward compat is trivial: every Op variant carries `#[derive(Serialize, Deserialize)]`. Loading a v3.0 save into a v3.1 build that has additional Stat 1 variants Just Works (serde handles the union).
- `#![deny(clippy::unwrap_used)]` remains enforceable end-to-end. Option B's table dispatch would either need fallible-then-unwrap or pervasive `Result<>` plumbing through what should be branch-free code paths.
- The 4-exhaustive-match cost is ~40 lines × 4 files = 160 lines of trivially-correct one-liners (`Op::MatPlus => math1::matrix::op_mat_plus(state)`). For the same ~40 lines, the compiler gives back four kinds of safety.

**Cons (and mitigations):**
- The `Op` enum grows from ~140 variants to ~180. Mitigation: variants are grouped with `// ── Phase 28: Math 1 Matrix ──` section dividers, mirroring the existing v2.0/v2.2 grouping pattern. Code-folding in IDE keeps it manageable.
- Future module shipments (v3.1 Stat 1, etc.) keep growing the enum. Mitigation: each module is a SEPARATE milestone with its own roadmap; the enum-grow pattern is the same exhaustive-match-preserving discipline used by Free42 and other behavioral-emulator references.

### Option B (REJECTED): `Op::XromCall(u16 xrom_id)` table dispatch

```rust
pub enum Op {
    // ... v2.2 variants ...
    XromCall(u16),  // packed (module:8 | function:8)
}

// dispatch:
Op::XromCall(id) => xrom_dispatch(state, id),

fn xrom_dispatch(state: &mut CalcState, id: u16) -> Result<(), HpError> {
    let (module, func) = ((id >> 8) as u8, (id & 0xFF) as u8);
    match (module, func) {
        (1, 1) => math1::matrix::op_mdim(state),
        (1, 2) => math1::matrix::op_mat_plus(state),
        // ... 40+ tuple-matches that the exhaustiveness check CAN'T see ...
        _ => Err(HpError::InvalidOp),
    }
}
```

**Why rejected:**
- The 4-exhaustive-match invariant breaks. The four files keep their `Op::XromCall(_) => …` catch-all; the REAL dispatch table moves into `xrom_dispatch` where the compiler cannot see "you forgot the MatMul arm." This is exactly the kind of run-time-only failure mode the v1.0 dispatch-design explicitly rejected.
- Save-file portability worsens, not improves. A v3.0 save with `Op::XromCall(0x0102)` is opaque without the registry; a v3.0 save with `Op::MatPlus` is self-describing in JSON (`{"MatPlus": null}`).
- Dynamic module loading is NOT a real v3.x requirement. The PROJECT.md scope freeze (line 156–164) names Math 1 / Stat 1 / Time / Advantage as the entire v3.x ambition — four modules, ship-once-per-milestone. Each ships as a code release; no end-user "module install" surface is in scope. The supposed flexibility benefit of Option B is unrealized.
- `Op::Xeq("M+")` programmatic-symmetry still requires a string → registry lookup at run-time (the v2.2 D-25.8 contract). Option B does NOT eliminate that lookup; it ADDS a second indirection on top.
- Tests would have to verify the `xrom_dispatch` table separately from the Op enum — doubling the test surface for no gain.

### Code precedent: `synthetic_byte_to_op` is NOT Option B

The closest precedent in hp41-core is `synthetic_byte_to_op(byte: u8) -> Option<Op>` (`hp41-core/src/ops/mod.rs:940`), which DOES table-dispatch on a u8. But that table is `byte → Op`, and the returned `Op` then goes through the regular exhaustive `dispatch()` match. It is a **resolver**, not a **dispatcher**. The Math 1 xrom registry is structured identically: `xrom_resolve(name, modules) -> Option<Op>`. **The xrom registry is the v3.0 analog of `synthetic_byte_to_op`** — a small resolver that returns a regular `Op` variant. Option B would have been the analog of replacing `synthetic_byte_to_op` with `Op::SyntheticByte(u8)` and table-dispatching inside `dispatch()` — which is precisely what `Op::SyntheticByte` does for the OLD code paths but with recursion back through the regular variant arm (note the explicit "CRITICAL INVARIANT" comment at `hp41-core/src/ops/mod.rs:933` forbidding `Op::SyntheticByte(_)` returns from the resolver to prevent infinite recursion).

**The v3.0 chosen pattern is the same shape as `synthetic_byte_to_op`, scaled up: a string-keyed resolver returning regular `Op::*` variants.**

---

## Module-Slot Data Model

**v3.0 ships single-module (Math 1 only).** `state.xrom_modules: u8` is a bitfield with bit 0 hard-set on cold-start (`fn default_xrom_modules() -> u8 { 0b0000_0001 }`). The CLI / GUI never expose a "load module" command in v3.0 — the bit is always on.

**v3.1 (Stat 1) and beyond reuse the same field.** Stat 1 sets bit 1; Time sets bit 2; Advantage sets bit 3. The `xrom_resolve` function in `hp41-core/src/ops/math1/xrom.rs` is extended (its `match name` arm grows; new `stat1_resolve` / `time_resolve` private helpers added) — no `CalcState` schema change after v3.0.

**Why a bitfield, not `[Option<ModuleSlot>; 4]`:**
- A real HP-41 has 4 ROM ports. The original Math 1 / Stat 1 / Time / Advantage Pacs are PHYSICAL CARTRIDGES that occupy one port each. Bit-per-port maps 1:1 to that semantic without modeling per-cartridge metadata.
- v3.0 does not need to expose port assignment to the user — there is no Math 1 conflicting with Math 1, no port shuffling. The bitfield is a degenerate `HashMap<port, module>` that wins zero flexibility but saves 24+ bytes of serialized state per save file.
- Future "user wants Math 1 unloaded" is a 1-bit toggle; no per-cartridge dimension to add. If true cartridge metadata (firmware version, ID) is ever needed (v4.x?), the bitfield migrates to `[Option<ModuleSlot>; 4]` via a single `#[serde(default)]` superseder field — same migration shape as `text_regs` superseding nothing.

**Math 1's own transient state** (`matrix_dim`, `matrix_active_reg`, `complex_mode`, `integ_state`, `solve_state`) lives DIRECTLY on `CalcState`, not in a nested `ModuleSlot::Math1 { … }`. Rationale:
- The HP-41 hardware shares the 100-register file across the ROM and the user's program — there's no module-private memory. The matrix lives in `state.regs[matrix_active_reg..]`, exactly where USER STO/RCL programs would put it. Stat 1's Σ-register sharing (with v1.0 SigmaPlus / SigmaMinus / Mean / Sdev) is exactly the same pattern — Stat 1 ROM ops would extend `state.regs` usage, not introduce a parallel register file.
- `serde(default)` per field keeps backward compat with v1.0–v2.2 saves trivially. A nested `ModuleSlot::Math1 { matrix_dim: …, … }` would need its own default impl + would require schema-version handling if Math 1 ever changes the slot's internal layout. Flat fields with `#[serde(default)]` win.

---

## CLI Integration

**No `key_to_op` extensions.** Math 1 functions have NO dedicated keys.

**One modal-resolver extension** in `hp41-cli/src/keys.rs::xeq_by_name_local_resolve` at the `_ => None` arm: a NEW fallback line calling `xrom_resolve(name, state.xrom_modules)` before returning `None`. The function signature MAY need to accept `&CalcState` (to read `xrom_modules`); alternatively, callers can pass the modules bitfield explicitly to keep the signature stub-friendly.

The `?` help overlay automatically picks up Math 1 entries through the JSON pipeline: `help_data.rs` loads BOTH `hp41cv-functions.json` AND `hp41-math1-functions.json` (one `OnceLock` each), the `help_overlay_rows` helper concatenates them, and the rendering code in `ui.rs::render_help_overlay` reads the combined slice. No hand-curated table to maintain.

The right-panel discoverability listing (the "JSON-derived KEY_REF_TABLE replacement" from v2.2 D-25.18) filters BOTH JSONs by `key_path != null` — Math 1 entries all carry `XEQ "NAME"` strings in `key_path`, so they appear in the right panel under their own category headers ("Math1-Matrix", "Math1-Complex", etc.).

**No new modal types.** Math 1 reuses the existing XEQ-by-name modal (`PendingInput::XeqPrompt(acc)`). The user types the function name into the modal; the existing Enter handler runs the full resolver chain (user-LBL → `xeq_by_name_local_resolve` → `builtin_card_op` → `xrom_resolve` → `Err`). No new `PendingInput` variants.

---

## GUI Integration

**Same modal-resolver extension** mirrors CLI: `hp41-gui/src/App.tsx`'s XEQ modal Enter handler invokes the new `dispatch_op` path with a synthesized key id like `"xeq_M+"` which the existing `resolve_parameterized` (`hp41-gui/src-tauri/src/key_map.rs:173`) already strips to call `Op::Xeq("M+".into())`. Hp41-core's run-time fallback chain (see "Modified hp41-core/src/ops/program.rs" above) does the rest — `xrom_resolve` fires on the Math 1 mnemonic and dispatch flows into `math1::matrix::op_mat_plus`.

**The stub-error arm in `key_map.rs::resolve` does NOT shrink.** It stays at exactly its current content (`"asn"`, `"catalog"`, `"view"`, `"tone"`, 13 `*_prompt` ids). Those ids are NOT Math 1 — they are v2.2-deferred modal-infrastructure entries. Math 1 has zero overlap with that list. The user-visible difference in v3.0: the `?` help overlay (Phase 26's `SKIN-05`) gains a new "Math1-Matrix" / "Math1-Complex" / etc. set of category sections at the bottom, populated from the new JSON file.

**`KEY_DEFS` in `hp41-gui/src/Keyboard.tsx` is UNCHANGED.** Math 1 has no dedicated keyboard keys.

**`Op::Integ` and `Op::Solve` cross the IPC boundary harmlessly.** Both are `Op` variants like any other; `serde(rename_all)` defaults handle the JSON shape. `CalcStateView` does NOT grow — `integ_state` / `solve_state` are `#[serde(skip)]` and would not appear in the view even if they were added (the view is hand-curated, not derived from CalcState).

---

## User-Callback Re-entrancy (INTEG / SOLVE)

**The critical risk area for v3.0.** The existing `run_loop` (`hp41-core/src/ops/program.rs:439`) was designed for v1.0's simple `Op::Xeq` 4-deep-subroutine semantic. INTEG / SOLVE add a SECOND kind of recursion — the ROM op itself calls back into user code, then resumes its own logic.

### Termination conditions

| Path | Condition | Mechanism |
|------|-----------|-----------|
| Normal convergence | INTEG: \|delta\| < eps for K consecutive iterations | math1::integ::run_integ outer-loop check |
| Normal convergence | SOLVE: \|delta_x\| < eps OR \|f(x)\| < eps | math1::solve::run_solve secant check |
| Non-convergence | INTEG: MAX_INTEG_ITERS iterations reached | math1::integ — return `HpError::NoConverge` |
| Non-convergence | SOLVE: MAX_SOLVE_ITERS iterations (typ. 30) reached | math1::solve — return `HpError::NoConverge` |
| User fn errors | User LBL hits Div-by-zero, Overflow, etc. | Error propagates from inner `run_loop` to `run_integ` to `run_loop` outer — preserves `state.integ_state = None` cleanup |
| Stack overflow | `state.call_stack.len() >= 4` at user-fn invocation | Pre-mutation check; returns `HpError::CallDepth` BEFORE the recursive call — mirrors v2.2 `Op::XeqInd` precedent |
| Hard timeout | Combined inner-loop steps exceed MAX_STEPS | `run_loop` outer-MAX_STEPS guard catches infinite loops in BOTH user fn AND INTEG/SOLVE's iteration count |
| Nested INTEG/SOLVE | `state.integ_state.is_some() || state.solve_state.is_some()` | Hard reject at op entry — `HpError::InvalidOp` (Owner's Manual divergence: real Math 1 also rejects this) |
| User RTN exits caller | User fn's top-level RTN encountered with caller frame on call_stack | Standard `Op::Rtn` pop unwinds back to INTEG/SOLVE iteration code — already works |

### Stack depth budget

The HP-41 hardware limits the subroutine call stack to **4 deep** (D-22.15). A direct `XEQ` consumes 1 level. INTEG / SOLVE each consume 1 level for the user-fn callback. **Therefore: a user program that calls INTEG from inside an XEQ-nested-XEQ-nested-XEQ would have 3 levels USED at INTEG entry, with 1 level available for the user fn — and the user fn ITSELF cannot XEQ another subroutine (would exceed 4-deep).** This matches HP-41 hardware behavior exactly. The error path is the existing `HpError::CallDepth` pre-mutation guard.

### Re-entrancy of `run_program` vs `run_loop`

**Only `run_loop` is re-entered.** `run_program` is NOT re-entered — it's an entry point that clones the program, sets `is_running = true`, calls `run_loop`, resets `is_running = false`. Re-entering `run_program` would re-clone (~30 KB) per integration sample (~1000 samples) = catastrophic. INTEG / SOLVE call `run_loop` DIRECTLY with the already-cloned program slice, reusing the outer `run_program` invocation's program clone.

`state.is_running` stays `true` for the WHOLE recursive evaluation, including the inner `run_loop` calls. `state.pc` IS swapped — INTEG/SOLVE push the current `pc` onto `state.call_stack` before jumping to the user LBL, exactly as `Op::Xeq` does. This means the existing `Op::Rtn` arm (line 454) "Just Works" for popping back into INTEG/SOLVE's iteration code — no special case needed.

### Pitfall: `state.entry_buf` and re-entrancy

The `flush_entry_buf` invariant (`hp41-core/src/ops/mod.rs:590`) says "MUST be called at the start of every `dispatch()` invocation." INTEG/SOLVE call `dispatch` recursively (the user fn runs through `run_loop` which calls `execute_op` which does NOT call `flush_entry_buf` — that's the Phase 3 design). **The recursive user-fn execution path avoids `dispatch()` entirely** (it uses `execute_op` directly), so `flush_entry_buf` is not called mid-recursion. This is correct: there's no digit entry to commit inside a program run (RESEARCH Pitfall 2 from v2.2 already documented this). **No new pitfall.**

### Pitfall: `state.display_override` and re-entrancy

If the user fn contains a `VIEW`/`AVIEW`/`PROMPT`, it writes to `state.display_override`. When INTEG/SOLVE resumes, that override is STILL THERE for the next interactive dispatch. **This is the HP-41 hardware-faithful behavior** — VIEW shows until next key. No special case needed; the existing dispatch-top clear at `hp41-core/src/ops/mod.rs:632` does the right thing on the next user key.

---

## JSON-Canonical Pipeline Extension

**Decision: separate JSON files per module, NOT a `module` field on `hp41cv-functions.json`.**

### Why separate files

| Aspect | Separate files (CHOSEN) | One file w/ `module` field |
|--------|-------------------------|----------------------------|
| Source-of-truth boundary | Each milestone owns its own JSON | All modules edit the same JSON — merge conflicts |
| File size | `hp41cv-functions.json` stays ~1400 lines; Math 1 starts a fresh ~600-line file | Single file grows to ~5000 lines by v3.3 |
| Schema evolution | Math 1 schema adds `xrom: { module, module_id, function_id }`; v2.2 JSON is UNCHANGED (no migration of 130 existing entries) | Every v2.2 entry needs an `xrom: null` field added — gratuitous 130-line churn |
| Test surface | Each JSON has its own parity test (`function_matrix_parity.rs` + `function_matrix_parity_math1.rs`) — failures are localized | Single test file mixes v2.2 ROM + v3.x module entries; failure messages less specific |
| Documentation output | `docs-matrix` emits two matrices (or one combined with a Module column) | Same — orthogonal to source-file count |
| `include_str!` budget | Two `&'static str` constants; lazy-parsed on first use | One larger constant; same parse cost overall |
| Git blame clarity | v3.0 commits touch only `hp41-math1-functions.json` | v3.0 commits touch `hp41cv-functions.json`, blurring milestone boundaries in `git log -- docs/` |
| v3.1+ scaling | Add `hp41-stat1-functions.json` next; no co-edit | One file accumulates all four modules' worth of entries |

### Schema extension for Math 1 JSON

```json
{
    "op_variant": "MatPlus",
    "display_name": "M+",
    "category": "Math1-Matrix",
    "status": "implemented",
    "phase": "28",
    "key_path": "XEQ \"M+\"",
    "description": "Matrix add: target_matrix <- active_matrix + matrix at regs[X]",
    "xrom": { "module": "Math 1", "module_id": 1, "function_id": 2 },
    "divergences": []
}
```

The `xrom` object is the only schema delta. `hp41cv-functions.json` entries have no `xrom` field (they are built-ins, not module ops); the Rust `HelpEntry` struct adds `xrom: Option<XromMeta>` with `#[serde(default)]` so both JSONs deserialize through the same struct.

### `scripts/docs-matrix/` extension

Two-input mode:
```rust
fn main() {
    // Parse --in <path> repeated; --out <path> repeated. Pair them positionally.
    // Each (input, output) pair generates one Markdown matrix.
}
```
Or, alternatively, one combined matrix with a `Module` column added (`Built-in / Math 1 / Stat 1`). The Phase 28-03 plan picks one. Either way, `just docs-matrix-check` (drift guard, Pitfall 8 from v2.2 RESEARCH) stays the CI gate.

---

## Persistence

**Six new `CalcState` fields, all `#[serde(default)]`:** `xrom_modules`, `complex_mode`, `matrix_dim`, `matrix_active_reg`, `integ_state` (skip), `solve_state` (skip).

**v1.0–v2.2 save files load into v3.0 cleanly:**
- `xrom_modules` defaults to `0b0000_0001` via `#[serde(default = "default_xrom_modules")]` — Math 1 is "loaded" the first time a v2.x save file is opened, exactly as if the user cold-started v3.0.
- `complex_mode` defaults to `false` — matches v2.x behavior (Math 1 ops never ran, complex mode was never relevant).
- `matrix_dim`, `matrix_active_reg` default to `None` — no matrix declared; any Math 1 matrix op called before `MDIM` returns `HpError::InvalidOp` (faithful HP-41 behavior).
- `integ_state`, `solve_state` are `#[serde(default, skip)]` — never persisted, never restored, always `None` on load.

**v3.0 save files load into v2.2 builds (forward-compat):** serde-json silently ignores unknown fields, so `xrom_modules`, `complex_mode`, `matrix_dim`, `matrix_active_reg` are dropped — the v2.2 build sees only its known fields. Round-trip v3.0 → v2.2 → v3.0 LOSES the Math 1 transient state (matrix declarations) but PRESERVES all v2.2 state. This matches the v2.2-into-v2.0 forward-compat shape; no special handling.

---

## Build Sequence

The current PROJECT.md line 22 already names `core → cli → docs → gui → tests` — same shape as v2.2 Phase 25 → 26 → 27. v3.0 follows it with one **prepended phase** for the framework infrastructure.

### Phase 28: XROM framework + Math 1 core ops (hp41-core only)

The "module-framework first" plan. Sub-divides cleanly:
- Plan 28-01: `xrom.rs` module + `xrom_resolve` + `xrom_id_of` + `xrom_modules` field on `CalcState` + bitfield default. Test: `xrom_registry.rs` round-trip.
- Plan 28-02: Matrix family — `MDIM`, `M+`, `M-`, `MAT*`, `MAT/`, `INV`, `TRANS`, `DET`, `GETM`, `PUTM`. Includes the `matrix_dim` / `matrix_active_reg` fields + `complex_mode` field (used by Phase 28-03 too). Test: `math1_matrix.rs`.
- Plan 28-03: Complex family — `CADD`, `CSUB`, `CMUL`, `CDIV`, `CABS`, `CARG`, `CCHS`, `CCONJ`. Test: `math1_complex.rs`.
- Plan 28-04: Polynomial root — `PROOT`. Test: `math1_poly.rs`.
- Plan 28-05: Vector family — `V+`, `V-`, `VDOT`, `VABS`. Test: `math1_vector.rs`.
- Plan 28-06: INTEG (user-callback). Adds `integ_state` field, `run_integ`, `Op::Integ`. Test: `math1_integ.rs` including non-convergent, nested, user-fn-errors.
- Plan 28-07: SOLVE (user-callback). Adds `solve_state` field, `run_solve`, `Op::Solve`. Test: `math1_solve.rs`.

### Phase 29: CLI integration

- Plan 29-01: `xeq_by_name_local_resolve` xrom fallback + `pending_input.rs::handle_xeq_enter` resolver-chain extension.
- Plan 29-02: `hp41-cli/src/help_data.rs` second `OnceLock` for Math 1 JSON + `help_overlay_rows` merge.
- Plan 29-03: `prgm_display.rs::op_display_name` ~40 new arms.
- Plan 29-04: Right-panel KEY_REF_TABLE derivation includes Math 1 entries.

### Phase 30: Documentation

- Plan 30-01: `docs/hp41-math1-functions.json` populated (~40 entries; status="implemented", phase="28", xrom={module, module_id, function_id}).
- Plan 30-02: `scripts/docs-matrix/` two-input mode (or combined-matrix Module column).
- Plan 30-03: `docs/hp41-math1-function-matrix.md` regenerated + `just docs-matrix-check` updated.
- Plan 30-04: README soft-claim update — "v3.0 ships HP-41 Math 1 Pac behavioral emulation; Stat 1 / Time / Advantage deferred to v3.1 / v3.2 / v3.3."

### Phase 31: GUI integration

- Plan 31-01: `hp41-gui/src-tauri/src/prgm_display.rs::op_display_name` ~40 new arms (mirror Phase 29-03).
- Plan 31-02: `hp41-gui/src/App.tsx` XEQ modal — verify the synthesized `"xeq_M+"` key id round-trips through `key_map::resolve_parameterized` to `Op::Xeq("M+")` and the hp41-core fallback chain handles it. **No new `KEY_DEFS` entries needed.**
- Plan 31-03: `?` overlay (v2.2 SKIN-05) renders new Math 1 categories from the second JSON file. Vitest test added.
- Plan 31-04: `key_map::resolve` ROM-bind regression test extended (every Math 1 entry resolvable via XEQ modal end-to-end). E2E smoke optionally extended with one Math 1 keystroke flow (e.g., `2 ENTER 3 XEQ "C+" → "5.0000"`).

### Phase 32: Test hardening

- Plan 32-01: `numerical_accuracy.rs` extended from 566 to ~700+ cases per FEATURES.md. Combined ≥ 98 % gate maintained; v1.x 503-baseline-floor 498 preserved.
- Plan 32-02: hp41-core coverage gate held at ≥ 95 % (no atomic raise this milestone). Math 1 files MUST meet the bar.
- Plan 32-03: E2E smoke green on Ubuntu; Vitest CI-gated; no MSRV bump.

**Why "framework prepended" not just "core first":** Phase 28 starts with Plan 28-01 (the `xrom.rs` registry + `xrom_modules` field) BEFORE any Math 1 op variant is added. This means the registry's `match name { ... }` block is empty initially — committed as a working skeleton that other plans fill in. The two-resolver-symmetry test (`xrom_resolve("M+") => Some(Op::MatPlus)` AND `xrom_id_of(&Op::MatPlus) => Some((1, 2))`) is a Plan-28-01 acceptance criterion, run with an INITIALLY EMPTY match (passing because the test data is empty too). Each subsequent plan adds N entries to both `xrom_resolve` AND `xrom_id_of` plus N variants to `Op` plus N rows to `hp41-math1-functions.json`. The four-source-of-truth (variant / resolver / id-table / JSON) parity is asserted by `xrom_registry.rs` integration tests — adding one without all four is a CI failure (same pattern as v2.2 `function_matrix_parity.rs`).

---

## Justfile Integration

Only the `docs-matrix` recipe needs an update — every other recipe (`build`, `test`, `coverage`, `ci`, `gui-dev`, `gui-build`, `gui-ci`) already covers v3.0 changes:

```just
# v3.0 update: regenerate both module matrices
docs-matrix:
    cargo run --manifest-path scripts/docs-matrix/Cargo.toml -- \
        --in  docs/hp41cv-functions.json \
        --out docs/hp41cv-function-matrix.md \
        --in  docs/hp41-math1-functions.json \
        --out docs/hp41-math1-function-matrix.md

docs-matrix-check:
    @cargo run --manifest-path scripts/docs-matrix/Cargo.toml -- \
        --in  docs/hp41cv-functions.json  --out /tmp/cv.md \
        --in  docs/hp41-math1-functions.json --out /tmp/math1.md
    @diff docs/hp41cv-function-matrix.md /tmp/cv.md
    @diff docs/hp41-math1-function-matrix.md /tmp/math1.md
```

---

## Key Architectural Decisions and Rationale

| Decision | Rationale |
|----------|-----------|
| **Option A: one Op variant per Math 1 function** | Preserves the 4-exhaustive-match invariant — the compile-time safety net that caught dozens of bugs across Phases 1–27. Option B (`Op::XromCall(u16)`) would forfeit it for zero realized benefit (no dynamic-load requirement in v3.x scope). |
| **Flat `state.xrom_modules: u8` bitfield, not nested ModuleSlot** | Matches HP-41 hardware semantic (4 ROM ports as bits). Trivial save-file backward compat via `#[serde(default = "default_xrom_modules")]`. v3.1–v3.3 reuse the field without schema change. |
| **Math 1 transient state flat on CalcState** | Mirrors v1.0 Σ-register / v2.2 text_regs precedent. No nested module-private state. Each new field carries `#[serde(default)]` per the established 5-milestone pattern. |
| **Separate `hp41-math1-functions.json`** | v2.2 JSON stays untouched; v3.0 commits localized; per-module parity tests have crisp failure messages; scales to v3.1+ without merge conflicts. |
| **`xrom_resolve` is a string-keyed resolver returning regular Op variants** | Same architectural shape as `synthetic_byte_to_op` (proven precedent at `hp41-core/src/ops/mod.rs:940`). Resolver returns a normal `Op::*`; dispatch flows through the regular exhaustive match. Zero new dispatch indirection. |
| **`Op::Xeq` programmatic-symmetry preserved via extended fallback chain** | `Op::Xeq("M+")` inside a saved program resolves through the SAME chain as the modal Enter-arm: user-LBL → `xeq_by_name_local_resolve` → `builtin_card_op` → `xrom_resolve`. Mirrors v2.2 D-25.8 "must_have truth #4" symmetry invariant exactly. |
| **INTEG / SOLVE re-enter `run_loop`, NOT `run_program`** | Re-entering `run_program` would re-clone the program Vec per sample (~30 KB × ~1000 = 30 MB). `run_loop` reuses the outer clone. `state.is_running` stays true; `state.call_stack` consumes 1 level per user-fn call (4-deep cap preserved). |
| **Nested INTEG/SOLVE rejected at op entry** | Matches Owner's Manual divergence (real Math 1 ROM rejects nesting). Avoids exponential blow-up of MAX_STEPS budget. Documented divergence; user-facing error message is `HpError::InvalidOp` with a clear "INTEG / SOLVE already in flight" log line. |
| **No new `key_map::resolve` bare-id arms for Math 1** | Math 1 has no dedicated keys; XEQ-modal access only. Reuses v2.2 XEQ modal infrastructure as-is. Stub-error arm (D-5) stays UNCHANGED in v3.0. SC-4 trivially holds (no `op_*` / `flush_entry_*` / `format_hpnum` ever touches hp41-gui). |
| **CLI ↔ GUI parity (D-25.6) preserved via shared hp41-core fallback** | The fallback chain extension lives in `hp41-core/src/ops/program.rs` — BOTH CLI and GUI XEQ paths go through it. No frontend logic duplication; the same Math 1 mnemonic resolves identically on both surfaces. |
| **Single Op-strategy across the project** | Maintains code-review predictability. New contributors learn one pattern (Op variant + dispatch arm + execute_op arm + display name × 2 + JSON entry); they don't have to learn "and a different pattern for module ops." |

---

## Constraints from hp41-core (must not be violated)

1. `hp41-core` keeps zero UI/CLI/Tauri deps — enforced at compile time. Math 1's `ops/math1/` module uses only `rust_decimal`, `serde`, and existing hp41-core deps.
2. `#![deny(clippy::unwrap_used)]` continues to apply. All Math 1 ops use `.expect("reason")` or `?`-propagation. Test files carry `#![allow(clippy::unwrap_used)]` at file scope per the Phase 1+ established pattern.
3. `CalcState` schema is append-only with `#[serde(default)]` per field. Six new fields in v3.0; zero existing fields modified.
4. **`Op` enum is append-only** — no v1.0–v2.2 variants are renamed, removed, or restructured. v3.0 adds ~40 new variants at the tail, grouped under `// ── v3.0 Math 1: ──` section comments.
5. The 4-exhaustive-match invariant holds for every new Op variant: `dispatch()` arm, `execute_op()` arm (or explicit `Err(InvalidOp)` for run-loop-only ops), CLI `prgm_display.rs::op_display_name` arm, GUI `prgm_display.rs::op_display_name` arm.
6. The CLI ↔ GUI parity invariant (D-25.6) holds: the f-prefix one-shot model is unchanged; the modal-resolver chains are identical on both surfaces (both call the same hp41-core `xrom_resolve`).
7. SC-4 invariant holds: stricter grep `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` continues to match nothing.
8. `state.is_running` stays a single-bit boolean — INTEG/SOLVE do NOT introduce a nested-counter. The `state.integ_state` / `state.solve_state` Option fields carry recursion presence; `is_running` carries "any top-level program execution active."
9. `MAX_STEPS = 1_000_000` is the outer safety net — re-entrant evaluations inside one `run_program` invocation share that budget. INTEG/SOLVE-internal iteration limits (~30 secant steps, ~10 Romberg levels) are additional inner caps that fire long before MAX_STEPS would.
10. No `tokio` / async anywhere in hp41-core. INTEG/SOLVE recursion is fully synchronous, single-threaded.

---

## Risk Areas (fragility heatmap, descending order)

1. **INTEG / SOLVE re-entrancy in `run_loop`** — the only architecturally novel piece. Risk: a regression in `state.call_stack` pre-mutation guard would either (a) accept a 5-deep call → SEGFAULT-equivalent in Rust (just `Vec` push, fine for memory but breaks HP-41 fidelity) or (b) reject valid 4-deep INTEG-from-XEQ calls (false positive). Mitigation: dedicated `math1_integ_callstack.rs` test exercising every level boundary 1–5. Phase 28-06 acceptance criterion.
2. **JSON ↔ Op-enum parity for ~40 new variants** — adding `Op::MatPlus` without the corresponding `hp41-math1-functions.json` row (or vice versa) is silent until CI runs the parity test. Risk: a contributor adds the Op variant + dispatch arm and forgets the JSON row, CI catches it in the parity test. Mitigation: Phase 30-02 ships the parity test BEFORE Phase 28's Op variants land — write the test against an empty match, then green-bar each plan as it adds entries.
3. **Fallback chain ordering in `xeq_by_name_local_resolve` + `builtin_card_op` + `xrom_resolve`** — incorrect order silently shadows entries. v2.2 already established the chain: user-LBL FIRST (always wins per D-22.18), then ROM mnemonics. Math 1 inserts at the tail. Risk: a Math 1 mnemonic colliding with a card-reader name (e.g. if a hypothetical Math 1 function were named "WDTA" — it isn't). Mitigation: collision test in `xrom_registry.rs` enumerates all `builtin_card_op` names and asserts no `xrom_resolve` overlap.
4. **`state.complex_mode` interaction with v1.0 stack ops** — when complex_mode is on, every stack push from a Math 1 complex op writes a PAIR (Y+iX). Risk: a user toggles complex_mode mid-program, the next `Op::Enter` lifts the stack as if scalar, breaking the pair. Mitigation: complex_mode reads NEVER cross the Op::Add / Op::Sub / etc boundary — those ops are SCALAR by definition and ignore complex_mode. Math 1 complex ops (Op::CAdd / etc.) are the ONLY readers. The flag is purely a Math 1-internal convention; v1.0 ops are oblivious.
5. **Matrix register-base validation** — `state.matrix_active_reg = Some(95)` with `state.matrix_dim = Some((3, 3))` would need regs[95..104], but regs only has 100 entries (regs[95..100] valid, regs[100..104] OOB). Risk: a matrix-op panics or silently wraps. Mitigation: every Math 1 matrix op pre-validates `matrix_active_reg + rows*cols <= state.regs.len()` and returns `HpError::InvalidOp` on OOB. The check lives in a `validate_matrix_window(state) -> Result<(usize, usize), HpError>` helper used by ALL matrix ops.
6. **`docs-matrix` two-input mode** — recipe must regenerate BOTH files atomically; partial regeneration leaves the matrices drifted. Mitigation: `just docs-matrix-check` (Pitfall 8 from v2.2 RESEARCH) catches drift in CI before any PR merges.
7. **Save-file forward-compat from v3.0 → v2.2** — explicitly NOT supported in CLAUDE.md but a reasonable user expectation. The v3.0-saved `matrix_active_reg = Some(50)` would be silently dropped on v2.2 load, and a v3.0-saved program containing `Op::MatPlus` would fail to deserialize ("unknown Op variant"). Mitigation: documented in v3.0 README ("v3.0 saves are not readable by v2.x; back up before upgrading"). Not a code change.

---

## Sources

Direct inspection of:
- `hp41-core/src/state.rs` (lines 49–196 — CalcState schema and serde precedent)
- `hp41-core/src/ops/mod.rs` (lines 103–578, 626–923 — Op enum, dispatch, prgm_mode gate)
- `hp41-core/src/ops/program.rs` (lines 370–615, 439–615 — run_program, run_loop, execute_op)
- `hp41-cli/src/keys.rs` (lines 320–370 — xeq_by_name_local_resolve)
- `hp41-cli/src/help_data.rs` (full file — JSON pipeline)
- `hp41-gui/src-tauri/src/key_map.rs` (lines 1–230 — resolve, resolve_parameterized, stub-error arm)
- `scripts/docs-matrix/src/main.rs` (full file — docs-matrix CLI)
- `docs/hp41cv-functions.json` (lines 1–100 — JSON schema)
- `.planning/PROJECT.md` (lines 1–234 — milestone history, settled decisions, scope locks)
- `CLAUDE.md` (full file — SC-4, D-25.6, D-22.15, JSON pipeline, exhaustive-match invariant)
- `.planning/milestones/v2.0-research/ARCHITECTURE.md` (full file — structural template; v2.0 IPC pattern)
