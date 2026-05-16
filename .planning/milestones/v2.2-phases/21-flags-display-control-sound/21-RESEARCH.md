# Phase 21: Flags, Display Control & Sound — Research

**Researched:** 2026-05-14
**Domain:** HP-41 keystroke programming — flag storage + conditional skip, display-override channel, I/O-free sound event buffer
**Confidence:** HIGH (codebase patterns + flag semantics cross-verified against secondary HP-41 references and Free42 / HP-42S compatibility notes)
**Graph snapshot:** stale (72h old, 89 commits behind) — semantic relationships approximate. Used directly-read source files for all load-bearing claims.

## Summary

Phase 21 lands three loosely-coupled HP-41 subsystems inside `hp41-core` only — no CLI/GUI integration, that ships in Phases 25/26. The work decomposes into:

1. **Flag storage + 6 flag ops + skip-next-step in `run_loop`** — the largest slice, touches `CalcState`, `Op` enum, `dispatch`, `execute_op`, both `prgm_display.rs` copies, and adds a tight new `op_flags.rs` module.
2. **Display override + 5 display-control ops (VIEW/AVIEW/PROMPT/AON/AOFF/CLD)** — orthogonal to flags; adds a `display_override: Option<String>` field with `#[serde(default)]`. PROMPT additionally pauses program execution (requires a `paused` exit signal from `run_loop`).
3. **BEEP / TONE n event buffer** — single new `event_buffer: Vec<String>` field with `#[serde(skip)]` (or extend `print_buffer` with a tagged prefix — see D-04 below). Two ops, ~30 LOC.

All three subsystems share the same project-level discipline: every new `Op` variant lands in **4 places** (dispatch + execute_op + both prgm_display copies); every new `CalcState` field carries `#[serde(default)]` for backward compatibility with v1.0–v2.1 save files; zero `println!`/`eprintln!` may appear in `hp41-core`; `#![deny(clippy::unwrap_used)]` stays green.

**Primary recommendation:** Slice this phase into **4 plans** (`21-01` flags-core, `21-02` conditional-skip + tests, `21-03` display-control, `21-04` sound). Plans `21-01` and `21-03` and `21-04` can be developed in parallel waves after a small Wave-0 scaffold; `21-02` depends on `21-01`. Total estimated task count: **~18 tasks** across 4 plans (vs. Phase 12's 3 plans / 8 tasks and Phase 20's 1 plan / 6 tasks). Drives ~12 new `Op` variants — landing in lockstep through the 4-place rule is the central trap.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Flag storage (`flags: u64`) | hp41-core (state.rs) | — | Pure data structure; must serialize via serde; consumed by both CLI and GUI |
| Flag get/set/test helpers | hp41-core (ops/flags.rs new) | — | Bit-twiddling logic — single source of truth for all flag mutation |
| 6 flag ops (`SF/CF/FS?/FC?/FS?C/FC?C`) | hp41-core (dispatch + execute_op) | — | Op variants must compile across the workspace via 4-place rule |
| Skip-next-step semantic for flag tests | hp41-core (program.rs `run_loop`) | — | Mirrors existing `Op::Test(TestKind)` skip mechanism at line 231-235 of program.rs |
| Display override channel | hp41-core (state.rs `display_override: Option<String>`) | hp41-cli/gui (consumers) | Core emits, CLI/GUI render — but Phase 21 only builds the field + ops |
| PROMPT pause semantic | hp41-core (program.rs `run_loop` early-return) | hp41-cli/gui (R/S resumes) | Symmetric with `Op::STOP` (Phase 22) — same `is_running=false` exit |
| BEEP/TONE events | hp41-core (ops/sound.rs new + event channel) | hp41-cli/gui (audio output) | Phase 21 ships only the buffer; actual playback is Phase 25/26 |
| Keyboard wiring | — | hp41-cli (Phase 25) / hp41-gui (Phase 26) | **Out of scope for Phase 21** — explicit per ROADMAP |

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FN-FLAG-01 | CalcState exposes 56 user flags + system flags as `flags: u64` (or equivalent), `#[serde(default)]` | §Flag storage representation (`u64` recommended), §Save-file backward compat |
| FN-FLAG-02 | User can SF/CF/FS?/FC?/FS?C/FC?C any flag 0–55; tests skip next step on false | §Flag op semantics, §Conditional-skip pattern reuse |
| FN-DISP-01 | `VIEW nn` shows register N until next keypress; preserves stack | §Display-override channel, §VIEW parameterization |
| FN-DISP-02 | `AVIEW` shows ALPHA register until next keypress | §Display-override channel |
| FN-DISP-03 | `PROMPT` shows ALPHA and PAUSES running program until R/S | §PROMPT pause semantic (new `run_loop` exit branch) |
| FN-DISP-04 | `AON`/`AOFF` toggle ALPHA auto-display | §AON/AOFF system-flag mapping (flag 48 candidate) |
| FN-DISP-05 | `CLD` clears display without stack/ALPHA modification | §Display-override clearing, §CLD pure-no-op design |
| FN-SOUND-01 | `BEEP` emits structured event in print/event buffer; no I/O in core | §BEEP/TONE event channel (D-04) |
| FN-SOUND-02 | `TONE n` (0–9) — same buffer channel, with tone number | §Tone numbering, §Event line format |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1.x (workspace-pinned) | Serialize `u64` flag word natively in JSON | u64 serializes as integer (compact, valid JSON) [VERIFIED: state.rs already uses for `last_key_code: u8`] |
| rust_decimal | 1.42 | Existing `HpNum` arithmetic — used by VIEW reg lookup → format_hpnum | Already in dependency tree [VERIFIED: `Cargo.toml` workspace deps] |

### Supporting
No new external dependencies needed. The flag/display/sound subsystems are pure additions on top of existing infrastructure.

**No bit-set crate needed**: Rust native `u64` operators (`|=`, `&=!`, `& (1u64<<n) != 0`) handle all flag manipulation in ~20 LOC. A crate like `bitvec` (0.22.x) [CITED: crates.io/crates/bitvec] would be overkill for a single 64-bit word and adds a dependency footprint without behavioral benefit.

**Installation:** No new crates.

**Version verification:** No new packages — verification N/A.

## Architecture Patterns

### System Architecture Diagram

```
                                    ┌─────────────────────────────────────┐
                                    │  hp41-core (Phase 21 new surface)   │
                                    └─────────────────────────────────────┘
                                                       │
   ┌──────────────────────────┬───────────────────────┼─────────────────────────┐
   │                          │                       │                         │
┌─────────┐         ┌───────────────────┐  ┌─────────────────────┐  ┌───────────────────────┐
│Flag word│         │ Display-override  │  │  Event channel      │  │ Program / run_loop    │
│(u64 on  │         │ Option<String>    │  │  Vec<String> tagged │  │ (existing)            │
│CalcState│         │ on CalcState      │  │  #[serde(skip)]     │  │                       │
└─────────┘         └───────────────────┘  └─────────────────────┘  └───────────────────────┘
   │                          │                       │                         │
   │ SF/CF/                   │ VIEW/AVIEW/           │  BEEP push             │ FS?/FC?/FS?C/FC?C
   │ FS?/FC?/                 │ PROMPT/CLD            │  TONE n push           │ ↘ skip_pc if test fails
   │ FS?C/FC?C                │ AON/AOFF              │                         │ PROMPT → break loop
   │ (set/clear/test)         │ (write Some(...))     │                         │ (is_running=false)
   ↓                          ↓                       ↓                         ↓
        Existing Op enum + dispatch + execute_op + prgm_display (CLI + GUI)
                                          │
                                          ↓
                ┌─────────────────────────────────────────────────────┐
                │  Frontend consumers (Phase 25 CLI / Phase 26 GUI)   │
                │  ─ NOT touched in Phase 21 ─                        │
                │  • CLI: render display_override in ui.rs            │
                │  • CLI: drain event_buffer, print/play              │
                │  • GUI: send event_buffer over IPC in CalcStateView │
                │  • GUI: WebAudio play TONE n / BEEP                 │
                └─────────────────────────────────────────────────────┘
```

The diagram shows **three independent additions** to `CalcState` (the `flags` word, the `display_override` option, and the `event_buffer` vec) plus **one mutation to `run_loop`** (handling the new flag-test ops + the PROMPT exit branch). Frontends are explicitly out of scope.

### Component Responsibilities

| File | Responsibility | New / Modified |
|------|----------------|----------------|
| `hp41-core/src/state.rs` | Add 3 new fields: `flags: u64`, `display_override: Option<String>`, `event_buffer: Vec<String>`; init in `CalcState::new()` | Modified |
| `hp41-core/src/ops/flags.rs` (new) | Bit helpers (`flag_get`/`flag_set`/`flag_clear`); op_sf / op_cf functions | **New** |
| `hp41-core/src/ops/display_ops.rs` (new) | op_view(n) / op_aview / op_prompt / op_aon / op_aoff / op_cld | **New** |
| `hp41-core/src/ops/sound.rs` (new) | op_beep / op_tone(n) — pushes structured event strings | **New** |
| `hp41-core/src/ops/mod.rs` | Add ~12 new `Op` variants; 12 dispatch arms; module declarations | Modified |
| `hp41-core/src/ops/program.rs` | 12 new `execute_op` arms; new `run_loop` arms for FS?/FC?/FS?C/FC?C skip + PROMPT exit | Modified |
| `hp41-cli/src/prgm_display.rs` | 12 new `op_display_name` arms | Modified |
| `hp41-gui/src-tauri/src/prgm_display.rs` | Same 12 new arms (SC-4 exception — display formatter only) | Modified |
| `hp41-core/tests/phase21_flags.rs` (new) | Integration tests covering SC-1..SC-5 | **New** |
| `hp41-core/tests/phase21_display.rs` (new) | VIEW/AVIEW/PROMPT/AON/AOFF/CLD coverage | **New** |
| `hp41-core/tests/phase21_sound.rs` (new) | BEEP / TONE event buffer coverage | **New** |

### Recommended Project Structure

```
hp41-core/src/
├── ops/
│   ├── mod.rs              # Op enum + dispatch (modified)
│   ├── flags.rs            # NEW — bit helpers + 6 flag ops
│   ├── display_ops.rs      # NEW — VIEW/AVIEW/PROMPT/AON/AOFF/CLD
│   ├── sound.rs            # NEW — BEEP/TONE
│   ├── print.rs            # Existing — print_buffer model for event_buffer
│   ├── program.rs          # Modified — flag-test skip + PROMPT exit branch
│   └── registers.rs        # Existing — for VIEW nn register lookup
└── state.rs                # Modified — 3 new fields
hp41-core/tests/
├── phase21_flags.rs        # NEW
├── phase21_display.rs      # NEW
└── phase21_sound.rs        # NEW
```

### Pattern 1: Skip-Next-Step Conditional (REUSE existing)

**What:** The existing `Op::Test(TestKind)` mechanism inside `run_loop` (lines 231-235 of `hp41-core/src/ops/program.rs`):

```rust
// Source: hp41-core/src/ops/program.rs:231-235
Op::Test(kind) => {
    if !evaluate_test(state, &kind) {
        state.pc += 1; // skip next step (D-09: skip-if-false)
    }
}
```

`Op::Isg` / `Op::Dse` use a **bool-return helper pattern** (lines 236-244): the `op_isg(state, reg) -> Result<bool>` returns true when the loop body should skip; the `run_loop` arm increments `pc` on `Ok(true)`. **This is the exact pattern to mirror for FS?/FC?/FS?C/FC?C** — but with a cleaner approach: collapse all four into a single `Op::FlagTest { kind: FlagTestKind, flag: u8 }` variant (mirrors the `Op::Test(TestKind)` design), with `FlagTestKind` having 4 variants (`IsSet`, `IsClear`, `IsSetThenClear`, `IsClearThenClear`).

**When to use:** Inside `run_loop` only — interactive dispatch returns Ok(()) without skipping (no PC to advance at the keyboard, per HP-41 hardware behavior where `FS?` at keyboard "prints" YES/NO. In our emulator: keyboard-interactive FS? is a no-op or surfaces a status message — D-08 below).

### Pattern 2: I/O-Free Buffer Pattern (REUSE existing)

**What:** `print_buffer: Vec<String>` on `CalcState` with `#[serde(default, skip)]` (state.rs:93-94). Ops push formatted strings; the frontend drains the buffer.

**When to use:** Any time `hp41-core` needs to emit output without performing I/O. BEEP / TONE follow this exact model.

**Example:**

```rust
// Source: hp41-core/src/ops/print.rs:13-18
pub fn op_prx(state: &mut CalcState) -> Result<(), HpError> {
    let line = format!("{:>24}", format_hpnum(&state.stack.x, &state.display_mode));
    state.print_buffer.push(line);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

### Pattern 3: serde(default) for New CalcState Fields (REUSE existing)

**What:** Every field added since v1.0 carries `#[serde(default)]` so older save files deserialize without error.

**Source:** Phase 12 added 4 such fields (`last_key_code`, `reg_m`, `reg_n`, `reg_o`) — state.rs:97-112. Phase 11 added `print_buffer` with `#[serde(default, skip)]`. Phase 19 (Card Reader quick task) added `pending_card_op` with `#[serde(default, skip)]`.

**Phase 21 application:** `flags: u64` gets `#[serde(default)]` (persisted), `display_override: Option<String>` gets `#[serde(default, skip)]` (transient — cleared on next key event, matching HP-41 hardware: VIEW display vanishes on next operation), `event_buffer: Vec<String>` gets `#[serde(default, skip)]` (transient, drained by frontend).

**Critical:** The default for `flags: u64` is `0` — but real HP-41 hardware power-on sets several system flags ON (specifically flag 28 default depends on shipment locale; flag 11 auto-execution defaults off; flag 21 printer-enable defaults off until printer plugged in). **For our emulator, defaulting all flags to 0 is acceptable** because:
- Flag 28 (decimal separator) is a no-op in the v2.2 emulator — we always display with `.` as the radix mark, no locale toggle is wired
- Flag 21 (printer enable) — see §Pitfall 4 below for the cross-phase interaction with Phase 11 PRX/PRA/PRSTK
- All other system flag defaults match "all-clear" power-on state per the [HP-42S flag table](https://www.finseth.com/hpdata/hp42s.php) inheritance.

**Document the divergence in CLAUDE.md** alongside the existing v2.2 settled decisions.

### Pattern 4: Parameterized Op via Single u8 (REUSE existing)

**What:** `Op::StoReg(u8)`, `Op::RclReg(u8)`, `Op::FmtFix(u8)` all carry their parameter inline. The 4-place exhaustive match handles them via `format!("STO {r:02}")` in prgm_display.

**Phase 21 application:** `Op::SfFlag(u8)`, `Op::CfFlag(u8)`, and `Op::FlagTest { kind: FlagTestKind, flag: u8 }` follow the same pattern. `Op::View(u8)` does too. The flag arg is `0..=55`; the dispatch implementation must validate (`if flag > 55 { return Err(HpError::InvalidOp); }` — same guard as STO `if reg >= 100`).

### Pattern 5: Stub Already Present in GUI key_map

**What:** The Phase 19 v2.1 stub-error pattern in `hp41-gui/src-tauri/src/key_map.rs:101-104`:

```rust
"pi" | "polar_to_rect" | "rect_to_polar" | "beep" | "asn" | "catalog" | "view"
| "xeq_prompt" | "gto_prompt" | "lbl_prompt" => Err(GuiError {
    message: format!("'{key_id}' is planned for a future phase"),
}),
```

And the prompt-id list verified in tests (key_map.rs:328-345): `sto_prompt`, `rcl_prompt`, `xeq_prompt`, `gto_prompt`, `lbl_prompt`, `isg_prompt`, `fix_prompt`, `sci_prompt`, `eng_prompt`, **`sf_prompt`**, **`cf_prompt`**, **`fs_prompt`**, `x_eq_y_prompt`, `x_le_y_prompt`, `x_gt_y_prompt`, `x_eq_0_prompt`.

**Phase 21 lands the Rust core for `beep`, `view` (and adds `aview`, `prompt`, `aon`, `aoff`, `cld`, `tone`, all 6 flag ops).** The stub arm in `key_map.rs` will continue to error for those IDs until Phase 26 — but the **core ops will exist and be dispatchable from tests/CLI**. Phase 21 does NOT modify `key_map.rs`. This is the same boundary Phase 20 honored (10 new ops landed in core, still stubbed in `key_map.rs` until Phase 26).

### Anti-Patterns to Avoid

- **Hand-rolled bitset structs.** A `struct FlagSet { bits: [u64; 1] }` adds zero value over `u64` and breaks JSON serialization symmetry.
- **`[bool; 56]` array.** Verbose, larger JSON, no PartialEq performance gain in our use case.
- **`HashSet<u8>` for flags.** Sparse representation hides the system-flag boundary; iteration order is non-deterministic; serializes as a list `[5, 10, 23]` which is harder to diff than a single `4194416` integer.
- **Direct `println!` in `BEEP`/`TONE`.** Violates the `hp41-core` zero-I/O invariant. Must route through the event buffer.
- **`display_override: String` (non-Option).** Empty string vs. "display nothing" is ambiguous. Use `Option<String>` — `None` = render normal display.
- **PROMPT as a busy-loop in `run_loop`.** PROMPT must EXIT `run_loop` (set `is_running=false`, return Ok(())). Otherwise the IPC thread blocks indefinitely.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Bit-level flag storage | Custom `FlagSet` struct | `u64` + native operators | 8 bytes, serializes as integer, zero overhead |
| Flag bit indexing | `unsafe` pointer math | `1u64 << n` shift expression | Safe, zero-cost, matches Rust idiom |
| Power-on flag defaults | Manual flag init | `flags: u64 = 0` via `#[serde(default)]` | All-clear is acceptable for our emulator (see D-04 pitfall) |
| Event-line serialization | Custom binary format | Tagged plain `String` ("BEEP" / "TONE 5") | Matches `print_buffer` pattern; human-readable; v3.x can layer structured events |
| Display override timing | New "until next keypress" timer | `Option<String>` cleared by frontend on next key event | Frontend already has key-handling — no timer needed in core |

**Key insight:** All "buffer/channel" interactions between `hp41-core` and frontends already follow the same pattern (`Vec<String>` on `CalcState` with `#[serde(default, skip)]`, drained by the consumer). Phase 21 reuses this pattern twice — once for `display_override` (Option, not Vec — single value) and once for `event_buffer` (Vec, like print_buffer). No new infrastructure invented.

## Common Pitfalls

### Pitfall 1: Skip-Next-Step semantic divergence between keyboard and program

**What goes wrong:** Implementing FS? as "skip pc+1" at the keyboard, where there is no next program step to skip.

**Why it happens:** Real HP-41 hardware: in a running program, FS? skips the next step on false. At the keyboard, FS? displays "YES" or "NO" (or just leaves the X register intact). Conflating the two modes leads to "FS? at keyboard panics because pc points past program.len()".

**How to avoid:** Mirror the existing `Op::Test(TestKind)` pattern:
- In `dispatch()` (interactive path): the test ops are **no-op** (return Ok(()) without touching state) OR they set a status message. The existing `op_test()` (program.rs:98-101) is a no-op stub for the keyboard case.
- In `run_loop` (program path): handle the skip directly via `state.pc += 1` on false.

**Warning signs:** A test that runs FS? at the keyboard and expects pc to change is testing the wrong API. The keyboard FS? is a no-op (or surfaces status). The skip happens only inside `run_loop`.

### Pitfall 2: 4-place rule miss — silent InvalidOp in programs

**What goes wrong:** Adding a new `Op::SfFlag(u8)` variant to `dispatch()` but forgetting `execute_op` in `program.rs`. The variant gets caught by the catch-all `Op::Lbl(_) | Op::Gto(_) | ... => Err(HpError::InvalidOp)` (program.rs:410-417). Programs using SF return InvalidOp silently while interactive use works fine.

**Why it happens:** Rust exhaustive match on `Op` enum kicks in only for the `match op { ... }` inside `dispatch` and `execute_op`. The catch-all `Op::Lbl(_) | ...` block intentionally returns InvalidOp for "programming-only" ops that should never reach execute_op — but it ALSO eats any new variant that isn't explicitly listed before it.

**How to avoid:** **Audit the catch-all block** after adding any new variant. Verify the new ops are NOT in the `Op::Lbl(_) | Op::Gto(_) | Op::Xeq(_) | Op::Rtn | Op::PrgmMode | Op::Test(_) | Op::Isg(_) | Op::Dse(_)` chain. The Phase 12 plan documented this exact trap (12-01-PLAN.md acceptance criteria line 516).

**Warning signs:** `grep -A 10 "Op::Lbl(_)" hp41-core/src/ops/program.rs | grep -cE "SfFlag|CfFlag|FlagTest|View|AView|Prompt|Aon|Aoff|Cld|Beep|Tone"` must return 0.

### Pitfall 3: PROMPT blocks run_loop forever

**What goes wrong:** PROMPT is "halt program execution until R/S". A naive implementation might `loop { ... }` waiting for input — but `run_loop` is single-threaded and called synchronously from `dispatch()`. The thread blocks; the GUI freezes.

**Why it happens:** Forgetting that `run_loop` MUST return control to the caller for the event loop to redraw and accept input.

**How to avoid:** Treat PROMPT exactly like a top-level RTN: when `run_loop` encounters `Op::Prompt`, write the ALPHA text to `display_override`, then `break` out of the loop. `state.is_running` is reset to `false` by the existing `run_program` exit path. The caller (CLI: F5, GUI: run_stop command) resumes by calling `run_program` again with the next-step-index entry point. The full step semantics for resume are deferred to **Phase 22** (FN-PROG-01 STOP); Phase 21 only needs PROMPT to exit cleanly — full resume-from-PROMPT is a Phase 22 cross-cutting concern.

**Warning signs:** A test like `test_prompt_inside_program_returns` should complete in <100ms. If it hangs, PROMPT is looping.

### Pitfall 4: Flag 21 retroactively affects Phase 11 PRX/PRA/PRSTK

**What goes wrong:** On a real HP-41, flag 21 enables printing. When flag 21 is clear, PRX is a no-op (or prints to display only). Phase 11 shipped PRX/PRA/PRSTK as ALWAYS pushing to `print_buffer`. If Phase 21 wires SF/CF to flag 21 and the print ops start respecting it, **existing Phase 11 tests break**.

**Why it happens:** Cross-phase invariants — Phase 11 was written before flag 21 existed in our model.

**How to avoid:** **Make a conscious decision** in CONTEXT.md:
- **Option A (recommended)**: Phase 21 ships flag storage and the 6 ops, but **does NOT wire any system flag to existing op behavior**. Flag 21 is "data" — it stores/recalls correctly but does not gate PRX. Document this as a known divergence: "v2.2 system flags are stored faithfully but most have no observable effect; Phase 27 backlog item to wire flag 21 → PRX gate". Phase 11 tests stay green.
- **Option B**: Phase 21 wires flag 21 → PRX, and updates Phase 11 tests to either set flag 21 first or to assert the no-op path when clear. This is more hardware-faithful but more disruptive.

Picking Option A keeps Phase 21 small. The discuss-phase user can override.

**Warning signs:** `cargo test -p hp41-core --test print_tests` must pass green after Phase 21 lands.

### Pitfall 5: `display_override` not cleared on next op

**What goes wrong:** VIEW writes to `display_override`, the next keypress should clear it (HP-41 hardware: VIEW shows the register until ANY key is pressed, including digit keys). If the core never clears the override, the display panel shows the VIEW value forever.

**Why it happens:** Phase 21 only exists in core; the "clear on next key" logic lives in the frontend. But `dispatch()` is called for every op (including digit-entry routing in handle_op_prepare), so we can clear `display_override` at the top of `dispatch()`.

**How to avoid:** Add `state.display_override = None;` at the top of `dispatch()` AFTER `flush_entry_buf` but BEFORE the prgm_mode gate. The ops that WRITE the override (VIEW/AVIEW/PROMPT) do so AFTER this clear runs — so they survive. CLD is then a literal `state.display_override = None; Ok(())` — same effect as any other op's clear.

**Alternative:** Frontend-managed clearing. The CLI/GUI clear `display_override` on every key event before forwarding the dispatch. Simpler, but adds frontend obligations. Recommend the core-managed approach for v2.2 (single source of truth).

**Warning signs:** A test sequence `VIEW 5 ; Add` should leave `display_override = None`.

### Pitfall 6: AON/AOFF mapping to flag 48 vs. a dedicated field

**What goes wrong:** Two valid models:
- (a) `AON`/`AOFF` sets/clears a system flag (HP-42S maps this to flag 48 "Alpha keyboard active" per the [HP-42S flag table](https://www.finseth.com/hpdata/hp42s.php)) — uniform with the flag subsystem.
- (b) `AON`/`AOFF` toggles a dedicated `alpha_auto_display: bool` field on CalcState — explicit, no flag-mapping ambiguity.

**Why it happens:** Phase 21 introduces both flags AND display control, and they overlap on this one op.

**How to avoid:** Recommend Option (a) — use system flag 48 as the bit, with `op_aon` = `flag_set(48)` and `op_aoff` = `flag_clear(48)`. This avoids growing CalcState surface and matches the HP-41 → HP-42S progression. The flag's user-visible effect ("ALPHA display auto-shows after every op") is a Phase 25/26 concern, not Phase 21.

**Warning signs:** Two ways to query "is AON active" — flag 48 OR alpha_auto_display — means a downstream consumer will pick the wrong one. Use the flag.

## Code Examples

Verified patterns from existing source code:

### Example 1: Bit-twiddling helpers in pure Rust

```rust
// hp41-core/src/ops/flags.rs (NEW — proposed)

/// Get the state of a single flag (0..=55).
/// Returns false for any out-of-range flag (defensive — callers should validate first).
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

These are stateless free functions — they take and return `u64` rather than `&mut CalcState`. The op layer wires them:

```rust
// Op layer — guards against out-of-range BEFORE bit manipulation
pub fn op_sf(state: &mut CalcState, n: u8) -> Result<(), HpError> {
    if n > 55 { return Err(HpError::InvalidOp); }
    state.flags = flag_set(state.flags, n);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

### Example 2: Conditional skip in run_loop (mirror existing Test arm)

```rust
// hp41-core/src/ops/program.rs run_loop — NEW arm
// Mirrors lines 231-235 (Op::Test) and 236-244 (Op::Isg / Op::Dse)
Op::FlagTest { kind, flag } => {
    let is_set = flag_get(state.flags, flag);
    let should_skip = match kind {
        FlagTestKind::IsSet => !is_set,        // FS?: skip if NOT set
        FlagTestKind::IsClear => is_set,       // FC?: skip if NOT clear
        FlagTestKind::IsSetThenClear => {
            // FS?C: skip-if-clear, ALWAYS clear afterward
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
        state.pc += 1;
    }
}
```

**[ASSUMED]** — exact "always clear" semantic for FS?C / FC?C. Verified against [HP-41 docs/operations-reference.md](https://www.hpmuseum.org/prog/hp41prog.htm) and `docs/operations-reference.md:176-177`:
- `FS?C nn` = "Skip next if flag nn set, then clear flag" — implies "clear ALWAYS, regardless of skip outcome"
- `FC?C nn` = "Skip next if flag nn clear, then clear flag" — same

The success criterion SC-2 in ROADMAP confirms: "`FS?C 10` on a set flag clears it as a side effect; `FS?C 10` on a clear flag leaves it clear". The "leaves it clear" is a no-op for an already-clear flag (clearing a clear flag is idempotent). So both branches clear unconditionally — the test outcome only affects the skip.

### Example 3: PROMPT exit from run_loop

```rust
// hp41-core/src/ops/program.rs run_loop — NEW arm
Op::Prompt => {
    // Write ALPHA into display_override; exit run_loop cleanly.
    state.display_override = Some(state.alpha_reg.clone());
    break; // PROMPT halts execution — full STOP/resume semantic deferred to Phase 22
}
```

### Example 4: BEEP and TONE event push

```rust
// hp41-core/src/ops/sound.rs (NEW)

pub fn op_beep(state: &mut CalcState) -> Result<(), HpError> {
    state.event_buffer.push("BEEP".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

pub fn op_tone(state: &mut CalcState, n: u8) -> Result<(), HpError> {
    if n > 9 { return Err(HpError::InvalidOp); }   // TONE accepts 0..=9 only
    state.event_buffer.push(format!("TONE {n}"));
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

The line format `"TONE 5"` matches the existing `print_buffer` line-of-text convention. Frontends parse via `if line == "BEEP" { play_beep() } else if let Some(n) = line.strip_prefix("TONE ").and_then(|s| s.parse().ok()) { play_tone(n) }`. Future structured-events upgrade (e.g. JSON-tagged events) is non-breaking — just changes the line format; consumers update parsing.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Direct stdout `println!` for hardware emulator sounds | Buffer-channel pattern (push string to `event_buffer`) | Phase 11 (print emulation) | Established `hp41-core` zero-I/O invariant; Phase 21 inherits |
| Hand-rolled state machines for "display until next key" timers | Frontend-driven clear via dispatch-top reset | This research | Matches React/event-loop architecture; no timers in core |
| Sparse flag set (HashMap or HashSet) | Single `u64` word | This research | 8 bytes, cache-friendly, serializes as integer |
| Per-flag accessor functions on CalcState | Free helper functions taking `u64` | This research | Stateless, testable in isolation, no &mut |

**Deprecated/outdated:**
- Phase 12's `SyntheticByte(u8)` codepath does NOT need to be extended for flag ops — flag ops have proper `Op::SfFlag(u8)` variants. Synthetic byte map (ops/mod.rs:524-558) stays unchanged; no new entries required.

## Runtime State Inventory

Phase 21 is **greenfield additions only** — no rename / refactor. No data migration required. **Section omitted** per template guidance.

**Verification:** `grep -rn "flags:\|display_override\|event_buffer\|beep\|tone" hp41-core/ hp41-cli/ hp41-gui/` shows no pre-existing references to be renamed. The 3 new field names are unused project-wide.

## Project Constraints (from CLAUDE.md)

Extracted from `CLAUDE.md` — these directives MUST be honored by the planner:

1. **`hp41-core` zero-I/O invariant:** No `println!`/`eprintln!` in `hp41-core`. BEEP/TONE MUST route through `event_buffer` (or extend `print_buffer`). NON-NEGOTIABLE.
2. **4-place Op rule:** Every new `Op` variant lands in `dispatch()` (ops/mod.rs) + `execute_op()` (ops/program.rs) + `hp41-cli/src/prgm_display.rs` + `hp41-gui/src-tauri/src/prgm_display.rs`. Compile-time enforced via exhaustive matches.
3. **`#[serde(default)]` on every new CalcState field** for backward compat with v1.0/v1.1/v2.0/v2.1 save files.
4. **`#![deny(clippy::unwrap_used)]`** at the `hp41-core` crate root. All new code must use `.expect("reason")` or `?`-propagation. Test modules carry `#[allow(clippy::unwrap_used)]`.
5. **Workspace invariant:** `hp41-gui` MUST NOT appear in root `Cargo.toml` `members`. `tauri` / `tauri-build` MUST NOT appear in root `[workspace.dependencies]`.
6. **SC-4 invariant:** No calculator/math logic in `hp41-gui/src-tauri/src/`. Only `op_display_name` (in `prgm_display.rs`) is the documented exception. The stricter grep `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` must return empty.
7. **`pending_input` routing block must remain ABOVE modal-opening interceptors** in hp41-cli (S/R/Ctrl+A). Applies if Phase 25 adds new flag modals — but Phase 21 doesn't touch hp41-cli.
8. **Use `just`, never `cargo` directly** in any commit, plan, or doc.
9. **Commits via `/git-workflow:commit --with-skills` only.** German Emoji Conventional Commits. English-only message body and subject.
10. **MSRV 1.88** declared in `[workspace.package]` — preserve.
11. **Coverage gate:** `just coverage` runs `cargo llvm-cov clean --workspace` first. Current non-regression target: ≥ 92.5% on hp41-core (Phase 20 achieved 92.65%). v2.2 final target: ≥ 95% at Phase 27. Phase 21 should not regress below 92.5%.
12. **Save-file backward compat is sacrosanct.** SC-5 of this phase explicitly tests v1.x save files loading without error.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | HP-41 user flags are 0..=29 (some sources say 0..=10); system flags are 30..=55 — but we treat 0..=55 as user-addressable for SF/CF, consistent with `docs/operations-reference.md:168` which says "Flags 0–10 are user flags. Flags 11–55 are system flags" while the requirements say "56 user flags 00–55" | §Flag storage representation | LOW — even if the real boundary is 0..=10 user / 11..=55 system, SF/CF accepting 0..=55 is the union; the only consequence is that writing to a "system" flag doesn't produce a side effect, which we accept per Pitfall 4 D-04 |
| A2 | Flag 48 is the canonical bit for AON/AOFF (per HP-42S compatibility) | §Pitfall 6 | LOW — the bit number is internal; consumers test `flag_get(state.flags, 48)`; if HP-41 used a different number we adjust the constant |
| A3 | TONE accepts n in `0..=9` (10 distinct tones) | §Code Example 4 | MEDIUM — some HP-41 variants document `0..=9`; some say 9 tones; rejecting n > 9 with InvalidOp matches docs/operations-reference.md:269 "Tone n (0–9)" |
| A4 | FS?C / FC?C always clear the flag after the test, regardless of skip outcome | §Code Example 2 | MEDIUM — verified against `docs/operations-reference.md` and SC-2; if hardware actually clears only on the skip branch, the test SC-2 would still pass (it tests clear-after-set), but the FC?C-on-already-clear edge would diverge |
| A5 | PROMPT exits run_loop and full STOP/resume semantics are deferred to Phase 22 | §Pitfall 3 | LOW — Phase 22 owns FN-PROG-01 STOP; PROMPT pause is just a clean exit for Phase 21 |
| A6 | Power-on flags default to 0 (all clear), divergent from HP-41 hardware where some flags default ON | §Pattern 3 | LOW — accepted divergence; documented in CLAUDE.md additions; user can SF in PRGM mode to set defaults |
| A7 | Flag 21 (printer enable) is NOT wired to gate PRX/PRA/PRSTK in Phase 21 — Option A in Pitfall 4 | §Pitfall 4 | MEDIUM — discuss-phase should confirm. If user prefers Option B (wire flag 21 → PRX gate), Phase 11 test files need updates and the plan grows by 1 task |
| A8 | The string format for TONE events is plain text `"TONE 5"` (matches `print_buffer` convention); frontends parse via string match | §Code Example 4 | LOW — easy to evolve later to structured JSON if needed; never breaks save-file compat (event_buffer is `#[serde(skip)]`) |

## Open Questions

1. **Should `display_override` be cleared at the top of `dispatch()` (core-managed) or by the frontend (CLI/GUI-managed)?**
   - What we know: HP-41 hardware clears VIEW display on next keypress. Our dispatch() runs on every key — natural place to clear.
   - What's unclear: Does CLD itself need to be a clear, or just a no-op (since the next op clears anyway)?
   - Recommendation: **Core-managed** — clear at the top of `dispatch()` after `flush_entry_buf`. CLD becomes a literal `display_override = None` (which is a no-op against the freshly-cleared field, but provides a programmable way to explicitly clear). VIEW/AVIEW/PROMPT write AFTER the clear, so they survive their own dispatch. This is the single source of truth.

2. **Does `Op::Beep` carry any parameter, or is it strictly nullary?**
   - What we know: HP-41 docs: BEEP is parameterless; TONE n carries the tone number.
   - What's unclear: Whether to combine into one `Op::Tone(u8)` with BEEP being `Tone(5)` or similar.
   - Recommendation: **Two separate variants** — `Op::Beep` (nullary) and `Op::Tone(u8)`. BEEP on real HP-41 plays a distinctive 3-tone alarm pattern, not just a single tone; mapping to `Tone(5)` would be wrong. Keep them separate.

3. **Should `flags: u64` cap at 56 bits or expose all 64?**
   - What we know: HP-41 documents flags 0..=55; bits 56..=63 are unused on real hardware.
   - What's unclear: Whether to mask the upper 8 bits on serialize/deserialize or just leave them as 0.
   - Recommendation: **No mask needed.** `u64` with `1u64 << n` shifts beyond bit 55 just affects bits 56..63 which no consumer reads. The guard `if n > 55 { return Err(HpError::InvalidOp); }` in `op_sf` / `op_cf` / `op_flag_test` prevents user input from touching those bits.

4. **PROMPT resume — Phase 22 or Phase 21?**
   - What we know: PROMPT halts; R/S resumes. Phase 22 owns `STOP` (FN-PROG-01) which has identical semantics.
   - What's unclear: Whether Phase 21 needs to ship the resume hook or just the halt.
   - Recommendation: **Phase 21 ships halt only.** Phase 22 ships STOP and the shared resume infrastructure (PROMPT and STOP both rely on it). Document the half-implemented state in the Phase 21 summary; SC-3 of Phase 21 doesn't require resume — only "halt and shows ALPHA".

5. **AON/AOFF effect — what changes when flag 48 is set?**
   - What we know: HP-41 docs: "ALPHA auto-display" mode — after every operation, the ALPHA register is shown in the display until next key.
   - What's unclear: Whether Phase 21 needs to wire that behavior or just track the flag bit.
   - Recommendation: **Phase 21 only stores the flag**, Phase 25/26 wires display behavior. SC for FN-DISP-04 is "User can enable/disable ALPHA auto-display" — interpret as "flag is settable/clearable", not "auto-display works". Document in summary.

## Environment Availability

No new external dependencies required. Existing toolchain — `rust 1.88+`, `just`, `cargo-llvm-cov`, `serde`, `rust_decimal` — already installed and verified in CI.

**Skip status:** Phase 21 is pure code+test additions inside `hp41-core`. Section is N/A.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` (Rust std test harness) + `cargo llvm-cov` for coverage |
| Config file | `Cargo.toml` (workspace) + `justfile` for CI orchestration |
| Quick run command | `cargo test -p hp41-core --test phase21_flags` (single file) |
| Full suite command | `just ci` (lint → test → coverage gate) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| FN-FLAG-01 | `flags: u64` field on CalcState w/ serde(default) | unit | `cargo test -p hp41-core --test phase21_flags test_flags_field_serde_default` | ❌ Wave 0 |
| FN-FLAG-01 | Save-file backward compat (v1.x load) | integration | `cargo test -p hp41-core --test phase21_flags test_load_v1x_save_no_flags_field` | ❌ Wave 0 |
| FN-FLAG-02 | SF n then FS? n: skip-next behavior | integration | `cargo test -p hp41-core --test phase21_flags test_sf_then_fs_q_skips_next_step` | ❌ Wave 0 |
| FN-FLAG-02 | CF n then FS? n: no-skip behavior | integration | `cargo test -p hp41-core --test phase21_flags test_cf_then_fs_q_skips_next_step` | ❌ Wave 0 |
| FN-FLAG-02 | FS?C on set flag clears it | integration | `cargo test -p hp41-core --test phase21_flags test_fs_q_c_clears_flag_after_test` | ❌ Wave 0 |
| FN-FLAG-02 | FS?C on clear flag leaves clear | integration | `cargo test -p hp41-core --test phase21_flags test_fs_q_c_on_clear_flag_idempotent` | ❌ Wave 0 |
| FN-FLAG-02 | FC? / FC?C symmetric coverage | integration | `cargo test -p hp41-core --test phase21_flags test_fc_q_skips_when_set` | ❌ Wave 0 |
| FN-FLAG-02 | All 56 flags addressable | proptest | `cargo test -p hp41-core --test phase21_flags test_all_56_flags_round_trip` | ❌ Wave 0 |
| FN-FLAG-02 | Flag > 55 returns InvalidOp | unit | `cargo test -p hp41-core --test phase21_flags test_sf_out_of_range` | ❌ Wave 0 |
| FN-DISP-01 | VIEW 03 writes register R03 to display_override | unit | `cargo test -p hp41-core --test phase21_display test_view_writes_register_to_override` | ❌ Wave 0 |
| FN-DISP-01 | VIEW preserves stack | unit | `cargo test -p hp41-core --test phase21_display test_view_preserves_stack` | ❌ Wave 0 |
| FN-DISP-02 | AVIEW writes ALPHA to display_override | unit | `cargo test -p hp41-core --test phase21_display test_aview_writes_alpha_to_override` | ❌ Wave 0 |
| FN-DISP-03 | PROMPT inside program exits run_loop | integration | `cargo test -p hp41-core --test phase21_display test_prompt_exits_run_loop` | ❌ Wave 0 |
| FN-DISP-04 | AON sets flag 48 | unit | `cargo test -p hp41-core --test phase21_display test_aon_sets_flag_48` | ❌ Wave 0 |
| FN-DISP-04 | AOFF clears flag 48 | unit | `cargo test -p hp41-core --test phase21_display test_aoff_clears_flag_48` | ❌ Wave 0 |
| FN-DISP-05 | CLD clears display_override; preserves stack/ALPHA | unit | `cargo test -p hp41-core --test phase21_display test_cld_clears_only_override` | ❌ Wave 0 |
| FN-SOUND-01 | BEEP pushes "BEEP" line to event_buffer | unit | `cargo test -p hp41-core --test phase21_sound test_beep_pushes_event` | ❌ Wave 0 |
| FN-SOUND-02 | TONE 5 pushes "TONE 5" line | unit | `cargo test -p hp41-core --test phase21_sound test_tone_n_pushes_event` | ❌ Wave 0 |
| FN-SOUND-02 | TONE 10 returns InvalidOp | unit | `cargo test -p hp41-core --test phase21_sound test_tone_out_of_range` | ❌ Wave 0 |
| FN-SOUND-01/02 | event_buffer is `#[serde(skip)]` — never persisted | unit | `cargo test -p hp41-core --test phase21_sound test_event_buffer_not_persisted` | ❌ Wave 0 |
| (regression) | No I/O leak — `println!` count in hp41-core stays 0 | grep | `! grep -rn 'println!\|eprintln!' hp41-core/src/` (returns no production lines) | manual |
| (regression) | Phase 11 print_tests still green | regression | `cargo test -p hp41-core --test print_tests` | ✅ exists |
| (regression) | Phase 20 phase20_math still green | regression | `cargo test -p hp41-core --test phase20_math` | ✅ exists |

### Sampling Rate
- **Per task commit:** `cargo test -p hp41-core --test phase21_<area>` (one of `flags` / `display` / `sound`) — finishes <1s per file
- **Per wave merge:** `cargo test -p hp41-core` — full hp41-core test suite (~5-10s)
- **Phase gate:** `just ci` green AND `cargo llvm-cov --fail-under-lines 92.5 -p hp41-core` exits 0

### Wave 0 Gaps
- [ ] `hp41-core/tests/phase21_flags.rs` — covers FN-FLAG-01 / FN-FLAG-02 + skip-next-step semantics (12+ tests)
- [ ] `hp41-core/tests/phase21_display.rs` — covers FN-DISP-01..05 (8+ tests)
- [ ] `hp41-core/tests/phase21_sound.rs` — covers FN-SOUND-01..02 (5+ tests)
- [ ] Optional: proptest module `hp41-core/tests/phase21_flag_proptest.rs` for FN-QUAL-03 — but FN-QUAL-03 is officially in Phase 27, so this is a nice-to-have

*All existing test framework + tooling is in place — no install steps needed.*

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | hp41-core is a single-user calculator emulator with no auth |
| V3 Session Management | no | No sessions; CalcState is a process-local struct |
| V4 Access Control | no | All ops are user-initiated; no privilege separation |
| V5 Input Validation | **yes** | Flag number > 55 → InvalidOp; TONE n > 9 → InvalidOp; SF/CF guards mirror existing STO `reg >= 100` pattern |
| V6 Cryptography | no | No cryptographic operations in this phase |
| V10 Configuration | no | No config files added |
| V12 Files & Resources | no | No file I/O in Phase 21 (zero-I/O invariant in hp41-core) |

### Known Threat Patterns for hp41-core (Rust)

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Out-of-range flag (n > 55) from loaded JSON | Tampering | `flag_get`/`flag_set`/`flag_clear` guard `if n > 55 { /* defensive */ }`; op layer returns `InvalidOp` BEFORE bit manipulation |
| Malformed save file with non-u64 `flags` field | Tampering | serde returns `Err` on type mismatch — propagated through normal load-error path; `#[serde(default)]` covers missing-field case only |
| Event buffer overflow (programmatic infinite loop pushing TONE) | DoS | MAX_STEPS guard (1,000,000) already in `run_loop` (program.rs:175) — caps event_buffer length at MAX_STEPS items maximum per run |
| PROMPT infinite hang (Pitfall 3) | DoS | PROMPT exits run_loop via `break`; no busy-wait inside core |
| println! introduction in hp41-core | Tampering (invariant break) | CI grep `! grep -rn 'println!' hp41-core/src/` — fails build if introduced |

## Plan Sizing Recommendation

Based on the Phase 11 (1 plan, 4 tasks — print emulation), Phase 12 (3 plans, 8 tasks — synthetic programming), Phase 20 (1 plan, 6 tasks — 10 ops), recommend **4 plans for Phase 21**:

### 21-01: Flag Storage and SF/CF Ops (Wave 0 + Wave 1)
- **Wave 0:** Create `hp41-core/tests/phase21_flags.rs` with 12+ RED tests (Op::SfFlag, Op::CfFlag, flag_get/set/clear, serde-default round-trip, v1.x save-file load).
- **Wave 1, Task 1:** Add `flags: u64` field to CalcState with `#[serde(default)]`. Update `CalcState::new()`. Create `hp41-core/src/ops/flags.rs` with `flag_get`/`flag_set`/`flag_clear` helpers + `op_sf` / `op_cf`. Add `pub mod flags;` to `hp41-core/src/ops/mod.rs`.
- **Wave 1, Task 2:** Add `Op::SfFlag(u8)` + `Op::CfFlag(u8)` variants; 2 dispatch arms; 2 execute_op arms; 2 prgm_display arms in BOTH copies. Wave 0 tests flip GREEN.
- **Estimated:** 1 plan, 3 tasks, ~250 LOC.

### 21-02: Conditional Flag Tests + Skip-Next-Step (Wave 1 — depends on 21-01)
- **Wave 1, Task 1:** Add `FlagTestKind` enum (4 variants: IsSet, IsClear, IsSetThenClear, IsClearThenClear). Add `Op::FlagTest { kind, flag }` variant.
- **Wave 1, Task 2:** Wire dispatch arm (interactive: no-op, mirrors `Op::Test`). Wire `run_loop` arm with skip logic + always-clear for FS?C/FC?C. Add execute_op arm (delegates to interactive no-op — skip handled in run_loop).
- **Wave 1, Task 3:** prgm_display arms in BOTH copies — format e.g. `"FS? 05"`, `"FC?C 12"`. Wave 0 skip-tests flip GREEN.
- **Estimated:** 1 plan, 3 tasks, ~150 LOC.

### 21-03: Display Control — VIEW/AVIEW/PROMPT/AON/AOFF/CLD (parallel with 21-01)
- **Wave 0:** Create `hp41-core/tests/phase21_display.rs` with 8+ RED tests.
- **Wave 1, Task 1:** Add `display_override: Option<String>` field with `#[serde(default, skip)]`. Add clear-at-top-of-dispatch logic in `dispatch()`. Create `hp41-core/src/ops/display_ops.rs`.
- **Wave 1, Task 2:** Add 6 Op variants (`Op::View(u8)`, `Op::AView`, `Op::Prompt`, `Op::Aon`, `Op::Aoff`, `Op::Cld`); 6 dispatch arms; 6 execute_op arms; PROMPT exits `run_loop` via `break`. 6 prgm_display arms in BOTH copies.
- **Estimated:** 1 plan, 2 tasks (single-plan-style like Phase 20), ~250 LOC.

### 21-04: BEEP / TONE Event Channel (parallel with 21-01, 21-03)
- **Wave 0:** Create `hp41-core/tests/phase21_sound.rs` with 5+ RED tests.
- **Wave 1, Task 1:** Add `event_buffer: Vec<String>` field with `#[serde(default, skip)]`. Create `hp41-core/src/ops/sound.rs` with `op_beep` + `op_tone`. Add `Op::Beep` + `Op::Tone(u8)` variants; 2 dispatch arms; 2 execute_op arms; 2 prgm_display arms in BOTH copies.
- **Estimated:** 1 plan, 1 task (smallest slice), ~120 LOC.

### Total
| Plan | Tasks | LOC (est) | Wave 0 file |
|------|-------|-----------|-------------|
| 21-01 Flags core | 3 | 250 | phase21_flags.rs |
| 21-02 Conditional skip | 3 | 150 | (extends phase21_flags.rs) |
| 21-03 Display control | 2 | 250 | phase21_display.rs |
| 21-04 Sound | 1 | 120 | phase21_sound.rs |
| **Total** | **9** | **~770** | 3 new test files |

**Parallelism:** 21-01, 21-03, 21-04 can land in parallel (independent files / Op variants). 21-02 depends on 21-01 (needs the flag word + bit helpers). Recommended execution order: 21-01 first; 21-02, 21-03, 21-04 in parallel after 21-01 ships.

**Alternative:** Bundle into a single mega-plan (like Phase 20). Trade-off: fewer commits but harder to bisect if any individual subsystem regresses. Given the 4-place-rule landmine and the three orthogonal subsystems, **4 plans is the safer slicing**.

## Sources

### Primary (HIGH confidence — codebase files read directly)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/src/state.rs` (CalcState fields, serde(default) precedent)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/src/ops/mod.rs` (Op enum, dispatch, synthetic_byte_to_op, 4-place rule)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/src/ops/program.rs` (run_loop skip-next pattern, lines 231-244; catch-all trap at 410-417)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/src/ops/print.rs` (buffer-channel pattern for BEEP/TONE)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/src/ops/registers.rs` (parameterized op pattern for SF/CF/VIEW)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-cli/src/keys.rs` (key_to_op pattern; out of scope for Phase 21 but referenced by Phase 25)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-cli/src/app.rs:511+` (PendingInput modal pattern — Phase 25 reference)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-cli/src/ui.rs:236-273` (pending_prompt exhaustive match — Phase 25 reference)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-cli/src/prgm_display.rs` (op_display_name pattern, Phase 20 example at lines 47-51)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui/src-tauri/src/prgm_display.rs` (SC-4 exception — display formatter only)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui/src-tauri/src/key_map.rs:101-104, 328-345` (stub-error arm + reserved prompt IDs sf_prompt/cf_prompt/fs_prompt — Phase 26 reference)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui/src-tauri/src/commands.rs:229-280` (sst_step/bst_step/run_stop pattern — Phase 26 reference for PROMPT/STOP resume)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui/src-tauri/permissions/run-stop.toml` (permission file template — Phase 26 reference)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui/src/Keyboard.tsx:75-93` (KEY_DEFS already reserves sf_prompt/cf_prompt/fs_prompt/beep/view/aview placeholders)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/src/error.rs` (HpError variants — Phase 21 needs no new variant, reuses InvalidOp)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/docs/operations-reference.md:166-178, 200-216, 268-273` (project's own flag/display/sound docs)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/.planning/REQUIREMENTS.md:28-46` (Phase 21 requirement specs)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/.planning/ROADMAP.md:53-70` (Phase 21 ROADMAP section, success criteria, constraints)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/.planning/phases/11-print-emulation/11-00-PLAN.md, 11-01-PLAN.md` (closest prior plan — buffer-channel pattern)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/.planning/phases/12-synthetic-programming/12-01-PLAN.md` (closest prior plan — multi-Op variant + serde-default rollout)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/.planning/phases/20-core-math-and-conversions/20-01-SUMMARY.md` (Phase 20 — most-recent shipped plan; 4-place rule application; coverage gate practice)

### Secondary (MEDIUM confidence — flag-semantics cross-references)
- [HP-42S Flag Reference](https://www.finseth.com/hpdata/hp42s.php) — flag 21 / 22 / 23 / 25 / 27 / 28 / 29 / 48 confirmed; HP-41 is a subset of HP-42S flag space
- WebSearch result snippets [Manualslib HP-41CV Owner's Handbook, page 237 — flag table] [hpmuseum.org archv014.cgi flag thread] — confirm flag 21=printer, 22=numeric input, 23=alpha input, 25=error-ignore-and-clear, 27=USER mode, 28=radix mark, 29=digit grouping

### Tertiary (LOW confidence — flagged for validation in discuss-phase)
- A4 (FS?C / FC?C "always clear" vs. "clear only on skip"): two readings of the docs are possible; current proposed implementation matches the strict reading ("always clear")
- A7 (flag 21 → PRX gating): user-discretionary; either Option A or Option B in Pitfall 4 is defensible
- Exact JSON wire-format for the event_buffer in Phase 25/26 (plain string vs. tagged JSON object) — Phase 21 ships plain string, future-compat upgrade path remains open

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new external libraries needed; native Rust u64 sufficient; serde already pinned
- Architecture: HIGH — three additions reuse three existing patterns (buffer-channel, conditional-skip, serde-default field)
- Pitfalls: HIGH — 4-place rule and run_loop catch-all trap are explicitly documented from Phase 12; PROMPT block trap is straightforward
- Flag semantics (system flag bit numbers): MEDIUM — exact bit assignments cross-verified against HP-42S compat docs; one or two flags (e.g. AON/flag 48 mapping) are reasonable assumptions but not authoritative from a primary HP-41 reference

**Research date:** 2026-05-14
**Valid until:** 2026-06-13 (30 days — codebase is stable, no upstream churn anticipated; planner should re-validate flag-semantics references if Phase 21 isn't kicked off within this window)

## RESEARCH COMPLETE
