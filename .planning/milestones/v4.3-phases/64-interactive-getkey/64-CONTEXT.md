# Phase 64: Interactive GETKEY - Context

**Gathered:** 2026-06-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Upgrade the existing `Op::GetKey` from **non-interactive** (reads a stale `state.last_key_code` and continues, `hp41-core/src/ops/registers.rs:145`) to **interactive**: when GETKEY executes inside a running program, **suspend execution, wait for the next keypress, push that key's HP-41 row×col code to X, and resume** from the exact step. This rides the Phase 63 yield/suspend engine, extended with a new **event-driven ("wait-for-key") yield kind** whose resume is triggered by a key event rather than a timer.

**In scope:** the GETKEY suspend point in `hp41-core`'s `run_loop`, the event-driven yield-channel extension, and the CLI + GUI frontend wiring to capture the keypress and resume. Requirement: **PRGM-03** (FGAP-04; subsumes FGAP-08). The `Op::GetKey` variant already exists → the 4-way exhaustive-match invariant is **NOT** triggered; no new JSON pool entry.

**Out of scope:** `GETKEYX` (timed variant) unless research finds it is intrinsic to faithful GETKEY (D-01); CATALOG interactive scroll (FGAP-06, v4.4); general DISP-01 `display_override` rendering (v4.4). No new user-authored Ops. No async, no threads, no `sleep` in `hp41-core` (yield-and-resume only).
</domain>

<decisions>
## Implementation Decisions

### Wait & no-key semantics (PRGM-03 core behavior)
- **D-01 (match HP-41 exactly — research resolves):** The emulator target is the **waiting** behavior (FGAP-04: pause execution and wait for the user to press a key). The exact wait semantics — **indefinite wait vs. a timeout**, and whether a faithful GETKEY includes the timed `GETKEYX`-style behavior — MUST be resolved by the researcher against the **HP-41 Extended Functions / CX Owner's Manual** before planning. Lock the *intent* (faithful interactive GETKEY); the researcher locks the *specifics*. The **no-key sentinel (0)** is returned only "when appropriate" (FGAP-08) — minimally when the wait is cancelled (see D-02); also on timeout IF research finds a timeout is HP-faithful.

### Key capture during the wait (Claude's discretion, locked consistent with D-01)
- **D-02:** **Faithful all-key capture.** While suspended on GETKEY, **every** key returns its row×col code and resumes the program — normal key functions (digits, ops, **R/S**, SST, …) are suspended during the wait, exactly like real hardware. R/S is captured as keycode **84** and does NOT stop the program. The escape from a GETKEY-blocked program is the **Phase 63 `request_cancel`** path (frontend cancel — e.g. ON / Esc / window cancel), which pushes the **no-key sentinel (0)** and ends/returns control. **Rejected:** R/S-retains-stop (intuitive but non-faithful — R/S must return 84); no-escape-at-all (leaves the user trapped with only window kill).

### Display while suspended
- **D-03:** **Display unchanged during the suspend** — show whatever was last on the 14-segment display; real HP-41 gives no special GETKEY prompt. Implemented via the Phase 63 yield channel carrying **no override text**; `display_override` stays untouched (Phase 63 D-04 carry-forward). Identical CLI ↔ GUI (D-25.6). **Rejected:** prompt indicator (less faithful; not needed).

### Input sourcing & GUI capture
- **D-04:** **CLI** reuses its existing `handle_key` row×col mapping (which already sets `last_key_code`). **GUI must now capture keys during GETKEY** — it currently NEVER sets `last_key_code` (only round-trips it in save/load; confirmed `hp41-gui/src-tauri/src/persistence.rs`). During the GETKEY suspend, **on-screen HP-41 key taps AND physical-keyboard keys** (via the same map) set the row×col code that resumes the program. Full CLI ↔ GUI parity (D-25.6).

### Yield-channel extension (Claude's discretion / planner)
- **D-05:** Extend the Phase 63 yield channel (`YieldState` / `YieldKind`, `hp41-core/src/state.rs`) with a new **event-driven "wait-for-key" kind** whose resume is triggered by a **key event**, not a `resume_ms` timer. `resume_program` carries the captured keycode into X (LiftEffect::Enable, as `op_getkey` already does). Exact field/enum shape is the planner's call; reuse Phase 63's `resume_program` re-entry verbatim. No new `Op`; no JSON/`builtin_card_op` change.

### Edge behavior (research)
- **D-06:** Researcher to confirm faithful behavior + re-entrancy safety for: **ALPHA-mode-during-GETKEY**; **nested yields** (GETKEY inside a Phase-63 interrupt-alarm handler, or during a PSE/VIEW/AVIEW yield); and GETKEY's interaction with the Phase 63 `pending_interrupt`/`pending_yield` states. Mirror the Phase 63 D-10 solver/modal isolation discipline — a GETKEY suspend must not corrupt an in-flight solver/modal or alarm-handler frame.

### Settled upstream — NOT re-decided here (carry forward from Phase 63)
- Yield-and-resume engine (63 D-01): `run_loop` breaks + records a yield; frontend resumes via `resume_program` re-entering `run_loop`. GETKEY is the **wait-for-input** member of this family (per `.planning/research/SUMMARY.md`).
- Dedicated typed yield channel + **`display_override` untouched** (63 D-04).
- D-11 no-polling (resume driven by command return values + a key event, never `get_state` polling); D-25.6 CLI↔GUI parity; D-07 never-silently-swallow (a cancelled/edge GETKEY surfaces sensibly); panic-free core; `request_cancel` exists.

### Claude's Discretion
- Exact shape/naming of the new wait-for-key `YieldKind` and how the captured keycode is threaded into `resume_program` → X (D-05).
- CLI test-fixture strategy for asserting the suspend/resume + keycode without real keyboard input (surface the captured keycode as data, mirror Phase 63's no-wall-clock-sleep test approach).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Run-loop yield engine (primary — this phase extends it)
- `.planning/phases/63-run-loop-yield-engine-interrupting-alarms-pse-view-aview/63-CONTEXT.md` — the yield/resume engine, yield channel (D-01/D-04), and the locked invariants GETKEY inherits.
- `.planning/phases/63-run-loop-yield-engine-interrupting-alarms-pse-view-aview/63-02-SUMMARY.md` — `run_loop` interrupt boundary + yield arms + `resume_program` (the mechanism to reuse).
- `.planning/phases/63-run-loop-yield-engine-interrupting-alarms-pse-view-aview/63-04-SUMMARY.md` + `63-06-SUMMARY.md` — GUI `run_program`/`resume_program` commands + the no-poll TS yield-and-resume driver (the GUI capture/resume path to extend).
- `.planning/research/ARCHITECTURE.md` — synchronous single-threaded `run_loop` yield mechanism; GETKEY is the "wait for input" yield-family member.

### GETKEY spec (the behavioral target)
- `.planning/research/DIVERGENCE-AUDIT.md` — **FGAP-04** (interactive GETKEY: pause, wait, push row×col, resume) and **FGAP-08** (return 0 no-key sentinel); UG-03 / SYNT-06 deferral history.
- `.planning/research/SUMMARY.md` — FGAP-04 in the `run_loop`-yield family (heavier "waiting for input" member).
- **HP-41 Extended Functions / CX Owner's Manual — GETKEY** — researcher MUST source the exact wait/timeout/GETKEYX semantics (D-01). External manual; no in-repo copy.

### Divergence record to update
- `docs/hp41-*-divergences.md` — flip the FGAP-04 / SYNT-06 "interactive GETKEY deferred" entry to "implemented (v4.3)", mirroring how Phase 63 flipped §D-40-04.

### Frozen invariants (must honor)
- `CLAUDE.md` — no async / no panics (`#![deny(clippy::unwrap_used)]`); 4-way exhaustive match (NOT triggered — `Op::GetKey` exists); save-file backward compat (`#[serde(default)]`/`#[serde(skip)]` on any new transient field); D-11 no-polling; D-25.6 parity; D-07 never-swallow; print-buffer discipline.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`Op::GetKey` + `op_getkey`** (`hp41-core/src/ops/registers.rs:143-151`): currently reads `state.last_key_code` and `enter_number` with `LiftEffect::Enable`. The interactive version keeps the push-to-X + lift semantics but sources the code from the suspend-and-wait instead of the stale field.
- **`last_key_code: u8`** (`hp41-core/src/state.rs:158`): row×10+col (1-indexed), 0 = none. Set by CLI `handle_key()` on every Press. **GUI never sets it** (gap — D-04).
- **Phase 63 yield engine** (`hp41-core/src/ops/program.rs` `run_loop`/`resume_program`; `state.rs` `YieldState`/`pending_yield`): break-record-resume machinery to extend with an event-driven kind (D-05).
- **CLI** `drain_pending_yields` (`hp41-cli/src/app.rs:308`) + `handle_key` row×col mapping; **GUI** `run_program`/`resume_program` commands (`commands.rs`) + App.tsx yield-and-resume driver + `request_cancel`.
- **`cardreader/raw.rs`** GetKey byte `0xCE` — unchanged (no encoding work).

### Integration Points
- `hp41-core/src/ops/program.rs` — GETKEY suspend point in `run_loop`; `resume_program` threads captured keycode → X.
- `hp41-core/src/state.rs` — new event-driven wait-for-key yield kind/field (`#[serde(default, skip)]`).
- `hp41-core/src/ops/registers.rs` — `op_getkey` reworked to trigger the suspend rather than read stale `last_key_code`.
- `hp41-cli/src/app.rs` — capture the next keypress during the suspend, resume with its row×col code; cancel → sentinel 0.
- `hp41-gui/src-tauri/src/commands.rs` + `hp41-gui/src/App.tsx` — GUI key capture (on-screen taps + physical) during suspend; resume via the event-driven path (no polling).
</code_context>

<specifics>
## Specific Ideas

- "Interactive GETKEY" is the **wait-for-input** member of the Phase 63 `run_loop`-yield family — reuse the engine, add an event-driven (key-triggered) resume instead of a timer.
- The escape from a blocked GETKEY is the Phase 63 `request_cancel` path → push the no-key sentinel (0); R/S is a normal captured key (84), not a stop.
- Flip the FGAP-04 / SYNT-06 divergence entry to "implemented (v4.3)", as Phase 63 did for §D-40-04.
</specifics>

<deferred>
## Deferred Ideas

- **`GETKEYX`** (timed/timeout GETKEY variant) — only in scope if research finds it is intrinsic to faithful GETKEY (D-01); otherwise its own later item.
- **CATALOG interactive scroll** (FGAP-06) — v4.4 (generator/iteration yield, sibling of this `run_loop`-yield family).
- **DISP-01** general CLI `display_override` rendering — v4.4 (Phase 63 routed around it; GETKEY keeps the display unchanged, D-03).
</deferred>
