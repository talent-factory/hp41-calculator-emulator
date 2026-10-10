# Phase 67: Reset Escape Hatch - Context

**Gathered:** 2026-06-10
**Status:** Ready for planning
**Source:** Approved design spec (`docs/superpowers/specs/2026-06-10-reset-escape-hatch-design.md`)

<domain>
## Phase Boundary

Deliver a two-tier, in-app **reset escape hatch** so a user whose calculator is
stuck in an input-blocking state — including a trap that survives an app restart
because the shared autosave reloads it — can recover without reinstalling.

- **Soft reset** — clears all transient working state + every input-trapping
  field, **preserves** stored user data (program, registers, flags, key
  assignments, X-MEM, modules, Time/Advantage state).
- **Full reset / MEMORY LOST** — restores factory state (`CalcState::new()`).

Both run **outside** the key→Op dispatch path (so they work when dispatch is
itself stuck) and overwrite the shared autosave **synchronously** (so recovery
survives relaunch).

Surfaces: `hp41-core` (engine), `hp41-cli` (TUI), `hp41-gui` (Tauri desktop +
iOS). This phase closes a **resilience** gap, not a hardware-fidelity gap — the
reset is an intentional, documented divergence from real HP-41 ON semantics.

**This is NOT:** the root-cause fix for the persisted-trap bug (separate
follow-up, needs a reproduction); an authentic ON+← physical chord; a power
on/off animation.
</domain>

<decisions>
## Implementation Decisions

All decisions below are **LOCKED** — sourced from the approved design spec.

### Core engine — `hp41-core/src/state.rs`
- Add **two methods** on `CalcState`: `soft_reset(&mut self)` and
  `memory_lost(&mut self)`. Neither is an `Op`; neither goes through `dispatch()`.
- `memory_lost()` is exactly equivalent to `CalcState::new()` (factory state).
- `soft_reset()` resets every **transient / input-trapping** field and
  preserves every **stored-data** field. Trapping fields to clear (non-exhaustive
  — the plan must enumerate the full current set by reading `state.rs`):
  - Stack (`x/y/z/t`), `last_x`, in-progress number entry / entry buffer
  - `display_override` → `None`
  - `prgm_mode` → `false`; `user_mode` → `false`
  - `modal_program` / `modal_prompt` → `None`; modal sub-states (`integ_state`,
    `solve_state`, siblings) → `None`
  - `matrix_dim` / `matrix_active_reg` → `None`
  - `is_running` → `false`; `pc` → `0`; `call_stack` → cleared
  - Phase 63/64 transient fields (`pending_interrupt`, `pending_yield`,
    GETKEY-capture) → `None`
  - `print_buffer` → cleared; `last_key_code` → `0`
- Stored-data fields to **preserve**: `program`; numbered `regs`; `reg_m` /
  `reg_n` / `reg_o`; text registers; `flags`; `key_assignments` / `assignments`;
  `xmem_files` / `xmem_active_file` / `xrom_modules`; `time_offset_secs`;
  `rand_seed`; Advantage state (`adv_*`, `adv_tvm_state`).
- **Frontends** additionally reset their own transient entry buffers that never
  live in `CalcState` (CLI `shift_armed`, GUI `shiftActive`, in-progress ALPHA
  entry, any "waiting for key" UI mode) as part of handling the reset.

### GUI IPC — `hp41-gui/src-tauri/src/commands.rs`
- Two new Tauri v2 commands returning `CalcStateView`, modeled on `save_state`
  (commands.rs:656): `reset_soft` and `reset_full`.
- Each: lock `AppState`, call `state.soft_reset()` / `state.memory_lost()`,
  then **immediately** `persistence::save_state(&state_path_for_app(&app),
  &snapshot)` to overwrite the autosave **while holding the mutex** (survives
  restart + avoids a race with the auto-save thread). **No new enum crosses IPC.**
- New permission TOMLs `permissions/reset-soft.toml`,
  `permissions/reset-full.toml` referenced from `capabilities/default.json`;
  run `cargo check` first to generate the permission registry (Tauri v2.11).

### GUI / iOS frontend — `hp41-gui/src/Keyboard.tsx` + `App.tsx`
- The `ON` key is currently a dead, unwired key (`id: ''`, Keyboard.tsx:167)
  filtered out of `key_map.resolve()` — wire it **outside** `resolve()` so it
  touches nothing in the (possibly stuck) core dispatch path.
- **Tap / click** → `reset_soft`. **Long-press (~600 ms, mouse-hold + touch)**
  → confirmation sheet ("MEMORY LOST — delete all programs, registers and
  files?") → on confirm → `reset_full`.
- Long-press **replaces** the authentic ON+← chord (no touch chording) —
  documented as an approximation; the confirmation sheet is the real safety.
- The confirm sheet is `createPortal`-ed to `document.body` so the
  `transform: scale()` ancestor does not become its containing block (iOS
  gotcha — `reference_ios_gui_layout_gotchas`).
- Long-press logic must **not** fire the soft-reset on the pointer-up that ends
  a long-press.

### CLI — `hp41-cli/src/app.rs`
- Intercept the reset binding at the **very top** of `handle_key` (app.rs:478),
  **above** the `pending_input` routing block. Deliberate, documented exception
  to D-07 never-discard ordering: the escape hatch must beat every stuck state,
  including a stuck `pending_input`.
- Trigger = **`Ctrl+R`** (single reset entry; a terminal has no ON button).
  Pressing it opens a status-bar prompt offering both tiers:
  `Reset:  [s] soft   [f] full (MEMORY LOST)   [Esc] cancel`
  - `s` → `state.soft_reset()` immediately, then `persistence::save_state`
    (persistence.rs:42).
  - `f` → second confirm `MEMORY LOST? [y/n]`; on `y`, `state.memory_lost()` +
    `save_state`.
  - `Esc` (or any other key) → cancel, no change.
- The plan **MUST** check `Ctrl+R` against existing CLI app-level bindings for
  conflicts before wiring it.

### Discoverability
- Surface a one-line hint (tap=soft / long-press=full) in the onboarding wizard
  and/or the `?` overlay. Plain UI copy only.
- Reset is **NOT** added to the six `docs/hp41*-functions.json` pools (not an
  Op, not XEQ-able).

### Documentation
- **ADR** `docs/adr/v4.x-reset-escape-hatch.md` — escape-hatch architecture +
  the intentional divergence from hardware ON semantics. (Use the next free
  `v4.3-NNN` number consistent with existing ADR naming.)
- **Divergence entry** in `docs/hp41-*-divergences.md` — soft reset clears
  working state where real ON preserves it; full reset maps to ON+← MEMORY LOST.

### Claude's Discretion
- Exact long-press threshold tuning, status-bar prompt rendering details, and
  precise React state shape for the long-press/confirm-sheet — within the locked
  behavioral contract above.
- ADR number suffix and exact divergence-doc id, consistent with existing
  conventions.
- Onboarding-hint vs `?`-overlay placement (either or both).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design contract (authoritative)
- `docs/superpowers/specs/2026-06-10-reset-escape-hatch-design.md` — the full
  approved design spec; every locked decision above derives from it.

### Core engine
- `hp41-core/src/state.rs` — `CalcState`, `CalcState::new()`,
  `migrate_after_load()`; the planner must enumerate the live field set here to
  build the soft-reset clear/preserve lists exactly.

### CLI
- `hp41-cli/src/app.rs` — `handle_key` (intercept point ~app.rs:478, above the
  `pending_input` routing block).
- `hp41-cli/src/keys.rs` — existing key bindings (check `Ctrl+R` for conflicts).

### GUI / iOS
- `hp41-gui/src-tauri/src/commands.rs` — `save_state` model (commands.rs:656),
  `state_path_for_app`, auto-save thread.
- `hp41-gui/src-tauri/src/persistence.rs` — `save_state` (persistence.rs:42).
- `hp41-gui/src-tauri/capabilities/default.json` + `permissions/*.toml` — Tauri
  v2.11 permission-registry pattern.
- `hp41-gui/src/Keyboard.tsx` — the `ON` dead key (id `''`, line ~167) +
  `key_map.resolve()` filtering (line ~126).
- `hp41-gui/src/App.tsx` — pointer-handler wiring + portaled confirm sheet.

### Project guardrails
- `CLAUDE.md` — Frozen Invariants (4-way match is N/A here since reset is not an
  `Op`; D-07 never-discard exception; D-25.6 CLI↔GUI parity; save-file serde
  back-compat; iOS scale/safe-area + portal gotchas).
- `docs/adr/` — ADR naming convention; `docs/hp41-*-divergences.md` — divergence
  entry format.
</canonical_refs>

<specifics>
## Specific Ideas

- CLI prompt copy (verbatim):
  `Reset:  [s] soft   [f] full (MEMORY LOST)   [Esc] cancel`
- Full-reset confirm copy (CLI): `MEMORY LOST? [y/n]`.
- GUI/iOS confirm-sheet copy: "MEMORY LOST — delete all programs, registers and
  files?".
- Long-press threshold: ~600 ms (mouse-hold and touch).
- Persistence ordering invariant: persist **inside** the GUI command while
  holding the mutex (so the later auto-save thread writes the already-reset
  state, not a stale one).
</specifics>

<deferred>
## Deferred Ideas

- Root-cause fix for the persisted-trap bug (separate task — needs a repro of
  the exact stuck key sequence).
- Authentic ON+← physical chord on desktop (long-press chosen for cross-frontend
  consistency; chord is a possible later nice-to-have).
- Animating a power on/off transition.
</deferred>

<scope_fence>
## Scope Fence

**In scope:** `soft_reset()` / `memory_lost()` core methods + tests; GUI
`reset_soft` / `reset_full` commands + permissions + autosave-overwrite;
GUI/iOS `ON`-key tap/long-press wiring + portaled confirm sheet; CLI `Ctrl+R`
intercept + two-tier status-bar prompt; discoverability hint; ADR + divergence
doc; green `just ci` + `just gui-ci` with D-25.6 parity preserved.

**Out of scope (do not implement):** the trap-bug root cause; ON+← chord;
power-transition animation; any new `Op` / XROM / function-JSON entry; touching
`math1/` frozen files; any change that makes reset traverse `dispatch()` or
`key_map.resolve()`.
</scope_fence>

---

*Phase: 67-reset-escape-hatch*
*Context gathered: 2026-06-10 from approved design spec (spec-as-PRD express path)*
