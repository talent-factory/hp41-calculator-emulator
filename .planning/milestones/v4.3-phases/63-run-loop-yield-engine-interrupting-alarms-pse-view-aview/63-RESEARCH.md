# Phase 63: Run-Loop Yield Engine + Interrupting Alarms + PSE/VIEW-AVIEW — Research

**Researched:** 2026-06-06
**Domain:** Brownfield Rust — `hp41-core` synchronous clone-and-loop interpreter + CLI/GUI wiring
**Confidence:** HIGH (all anchors ground-verified against current `develop`)
**Nature:** Consolidation + ground-verification. Design is LOCKED (D-01..D-13, ADR v4.3-003). This file does NOT re-derive or re-open any decision.

---

## Summary

The planner is building **one synchronous yield/resume boundary** inside `run_loop` (`hp41-core/src/ops/program.rs`) and making four consumers ride it: interrupting control alarms (`>>label`, criteria 1–3), `PSE` (criterion 4), and `VIEW`/`AVIEW` (criterion 5). The unifying mechanic already exists in skeletal form: `Op::Prompt` (program.rs:660) writes a display value and `break`s out of `run_loop`; `Op::Xeq` (program.rs:538) pushes `state.pc` onto `call_stack` and `Op::Rtn` (program.rs:500) pops it back. The interrupt path **reuses both verbatim** — a synthetic XEQ frame jumps to the alarm label, and the existing `Rtn`/program-end pop restores the resume address (`pending_interrupt: Option<String>` is the only new control field; D-12). PSE/VIEW/AVIEW generalize the `Op::Prompt` break into a **typed yield channel** (D-04) that carries `(kind, formatted text, resume_ms)` so the frontend can render the value and auto-resume via `resume_program` — with **no blocking sleep in core** (D-01). `display_override` stays untouched (DISP-01 deferred to v4.4). No new `Op` variants → the 4-way exhaustive-match invariant is not triggered (D-12).

**Primary recommendation:** Follow ARCHITECTURE.md's Phase A→G order; build the alarm interrupt path (A–C + ack) first, then layer the PSE/VIEW/AVIEW yield consumers onto the same break boundary, then wire both frontends (CLI sleep-then-resume; GUI tick_time-scheduled resume) in the same phase for D-25.6 parity.

---

## Ground-Verified Anchors

All line numbers re-grepped against current `develop` (2026-06-06). The ARCHITECTURE.md anchors (also dated 2026-06-06) all still hold — **zero drift**.

| Symbol | File | Current line | Role |
|--------|------|-------------|------|
| `run_program` | `hp41-core/src/ops/program.rs` | **422** | Clone program, set `pc = start+1` (444), `call_stack.clear()` (445), `is_running = true` (446), `run_loop` (448), `is_running = false` on all paths (450). **D-12 insert point:** add `pending_interrupt = None` right after 445. |
| `resume_program` | `hp41-core/src/ops/program.rs` | **468** | Guard `pc >= program.len()` → `InvalidOp` (469); clone program; `is_running = true` (473); `run_loop`; `is_running = false`. **D-09 insert point:** clear `pending_interrupt = None` (and the yield channel) before entering `run_loop`. |
| `run_loop` | `hp41-core/src/ops/program.rs` | **485** | Per-iteration: `steps >= MAX_STEPS` guard (~488); `steps += 1`; `pc >= program.len()` break (~495); `op = program[pc].clone(); pc += 1` (496–497). **Insert points:** interrupt `take()` + Phase-C `check_alarms` between `steps += 1` and the `pc >= len` break (ARCHITECTURE.md ~189). |
| `MAX_STEPS` | `hp41-core/src/ops/program.rs` | **483** | `const MAX_STEPS: u64 = 1_000_000`. Phase-C cadence (D-05) is independent (~every 1000 steps). |
| `execute_op` | `hp41-core/src/ops/program.rs` | **730** (`execute_op_pub` 726) | Catch-all op executor called by `run_loop` `other =>` arm (710). VIEW/AVIEW/PSE currently live HERE (884/885/902), not in `run_loop`. |
| `find_in_program` | `hp41-core/src/ops/program.rs` | **1320** | Label→index resolver used by `Op::Xeq` (547) + `Op::Gto`. Synthetic interrupt frame reuses it to locate the alarm label. |
| `Op::Rtn` arm | `hp41-core/src/ops/program.rs` | **500–504** | `call_stack.pop()` → `pc = return_pc`, else `break`. **D-06 ack point:** after the pop that restores the *interrupted* program's resume address, ack+reschedule the alarm. |
| `Op::Stop` arm | `hp41-core/src/ops/program.rs` | **657** | `break`. Edge for D-09 (interrupt set just before STOP must be cleared on resume). |
| `Op::Prompt` arm | `hp41-core/src/ops/program.rs` | **660–663** | Writes `display_override = alpha[..24]` + `break`. **The template** for the yield-and-resume break (D-01). |
| `Op::Xeq` arm | `hp41-core/src/ops/program.rs` | **538–573** | `call_stack.len() >= 4` → `Err(CallDepth)` (539); else `push(pc)` (548), `pc = target+1` (549). Synthetic interrupt frame mirrors this; the cap at 539 is the D-07 4-level guard reference. |
| `Op::View` / `op_view` | `program.rs` **884** → `display_ops.rs` **17** | Runs in `execute_op`. `op_view` formats `format_hpnum(regs[reg], display_mode)` → `display_override` (display_ops.rs:23). **Yield engine must capture THIS formatted string** into the yield channel. |
| `Op::AView` / `op_aview` | `program.rs` **885** → `display_ops.rs` **30** | Runs in `execute_op`. `op_aview` writes `alpha_reg.chars().take(24)` → `display_override` (display_ops.rs:32). Yield channel captures this string. |
| `Op::Pse` arm | `hp41-core/src/ops/program.rs` | **902–908** | In `execute_op`: writes `display_override = format_hpnum(stack.x,…)` (904) + pushes `"PAUSE 1000"` event (905), `LiftEffect::Neutral` (906). **Does NOT break** — the PRGM-01 gap. Stringly-typed `"PAUSE 1000"` marker is to be replaced by the typed yield channel (D-04, do NOT overload it). |
| `check_alarms` | `hp41-core/src/ops/time/alarm.rs` | **448** | Scans `alarms`; for each newly past-due sets `past_due = true` (453) and calls `dispatch_alarm_event` (455). Phase-A change: gate `pending_interrupt` on `is_running == true && pending_interrupt.is_none()`. |
| `dispatch_alarm_event` | `hp41-core/src/ops/time/alarm.rs` | **493–512** | Message → `alarm:message:{msg}` + `print_buffer` (496–497); non-interrupting → `alarm:xeq:{label}` (508); interrupting → **`"alarm:interrupting:deferred"` stub at 506** (the dead-end Phase 63 replaces). Add solver/modal demotion guard here (D-10). |
| `acknowledge_alarm` | `hp41-core/src/ops/time/alarm.rs` | **474** | `repeat_secs > 0` → `trigger_unix += repeat_secs`, `past_due = false` (478–481); else `alarms.remove(index)` (483). D-06 calls this after the handler RTNs. |
| `parse_alarm_type` | `hp41-core/src/ops/time/alarm.rs` | **~78** | `>>` → `Control{interrupting:true}` (80–82); `>` → `Control{interrupting:false}` (85–87); else `Message`. **CORRECT per ADR v4.3-003 — do NOT touch (D-11).** |
| `call_stack` | `hp41-core/src/state.rs` | **79** (`Vec<usize>`) | Interrupt frame pushes here; 4-level cap is load-bearing (D-07). |
| `is_running` | `hp41-core/src/state.rs` | **82** (`bool`) | Gates idle (D-13) vs interrupt (D-05) path. Must be restored on all paths incl. errors (D-07). |
| `event_buffer` | `hp41-core/src/state.rs` | **152** (`#[serde(default, skip)]` at 151) | Carries `alarm:xeq` / `alarm:message` / new `alarm:missing:{label}`. |
| `display_override` | `hp41-core/src/state.rs` | **144** (`#[serde(default, skip)]` at 143) | **Left untouched** (D-04 / DISP-01 deferral). |
| transient serde fields | `hp41-core/src/state.rs` | 108/110, 143/144, 151/152, 217/218, 226, 232, 238, 245, 287, 338, 343, 348, 353, 386, 391, 396, 401 | Established `#[serde(default, skip)]` pattern — new fields follow it (D-12). |
| solver/modal state | `hp41-core/src/state.rs` | `modal_program` **219**, `integ_state` **233**, `solve_state` **239**, `difeq_state` **246** | D-10 demotion guard reads these. |
| `CalcState::new()` | `hp41-core/src/state.rs` | (init block) | Initialize `pending_interrupt = None`, `pending_interrupt_alarm_index = None`, yield channel `None` alongside existing transient inits. |
| `migrate_after_load()` | `hp41-core/src/state.rs` | (migration fn) | Review pass only — new transient fields default cleanly, no active migration needed (I-03). |
| CLI `drain_event_buffer` | `hp41-cli/src/app.rs` | **1778** | `alarm:message:` → `self.message` (1781); `alarm:xeq:` → `run_program` + `drain_pending_card_op` + `drain_and_show_print_output` (1783–1795); **`alarm:interrupting:…` silently ignored at 1797** (the stub to wire). |
| CLI `drain_and_show_print_output` | `hp41-cli/src/app.rs` | **1865** | I-07 print discipline — must run after any alarm-handler output. |
| GUI `handle_tick_time` | `hp41-gui/src-tauri/src/commands.rs` | **342** | `check_alarms` THEN drain `event_buffer` into `event_lines` (343–348). Idle-fire (D-13) detection cadence. |
| GUI `run_stop` / `handle_run_stop` | `hp41-gui/src-tauri/src/commands.rs` | **374 / 476** | Run/stop entry; drains `event_buffer` (479). Yield resume scheduling rides the existing `tick_time` loop. |
| GUI alarm event consumer (TS) | `hp41-gui/src/App.tsx` | **1166–1187** | `alarm:message:` → toast (1171); `alarm:xeq:` → `dispatch_op xeq_{label}` (1172–1179); **`alarm:interrupting:` silently ignored at 1180–1181** (the stub to wire). `display_override` precedence at 1268. `setInterval(tick_time,100ms)` at 528. |

---

## Implementation Guidance (Build Order)

Restated from ARCHITECTURE.md Phase A→G, folded with D-01..D-13, and adapted to the two consumer families: **alarms** (criteria 1–3) and **PSE/VIEW/AVIEW** (criteria 4–5).

### Phase A — Core state + `check_alarms` routing (alarms)
- Add to `CalcState` (state.rs), all `#[serde(default, skip)]`, all init `None` in `new()`:
  - `pending_interrupt: Option<String>` (D-12).
  - `pending_interrupt_alarm_index: Option<usize>` (D-06a — paired field preferred over scanning; disambiguates two alarms sharing a fire time).
  - the **yield channel** (D-04): recommended shape
    ```
    pub enum YieldKind { Pse, View, Aview }
    pub struct YieldState { pub kind: YieldKind, pub text: String, pub resume_ms: u64 }
    pub pending_yield: Option<YieldState>   // #[serde(default, skip)]
    ```
- `dispatch_alarm_event` (alarm.rs:503, the `interrupting` arm): replace the `"alarm:interrupting:deferred"` stub (506):
  - if `is_running == true` **and** `pending_interrupt.is_none()` **and** no solver/modal active (D-10 guard: `integ_state`/`solve_state`/`difeq_state` all `None` and `modal_program.is_none()`) → set `pending_interrupt = Some(label)`, record `pending_interrupt_alarm_index`.
  - else (idle D-13, or already-pending nesting, or solver/modal demotion D-10) → push `alarm:xeq:{label}` to `event_buffer`.
- DNT-05: do **not** touch the non-interrupting `alarm:xeq` arm (508).

### Phase B — `run_loop` interrupt check + synthetic XEQ frame
- Between `steps += 1` and the `pc >= program.len()` break, add a `pending_interrupt.take()` block:
  - if `call_stack.len() >= 4` → **suppress, leave `past_due`, do NOT push a 5th frame** (D-07); drop the `pending_interrupt` (already taken) and do NOT ack.
  - else `find_in_program(program, &label)`:
    - `Ok(target)` → `call_stack.push(state.pc)`, `pc = target + 1` (verbatim `Op::Xeq` mechanics).
    - `Err(_)` → push `alarm:missing:{label}` to `event_buffer` (D-08), do NOT ack.
- The alarm program now runs inside the same `run_loop`; its `Op::Rtn`/program-end pop restores the interrupted `pc` (no new resume mechanism).

### Phase C — periodic `check_alarms` inside `run_loop` (D-05)
- Call `check_alarms(state)` every ~1000 steps (e.g. `if steps % 1000 == 0`) so the GUI (which holds the Mutex for the whole `run_loop`) detects mid-program alarms at "next-boundary" granularity. Sub-millisecond at real speed; satisfies criterion 1.

### Phase D/F — Acknowledgment after RTN (D-06)
- When `Op::Rtn` (or program-end) pops back to the **interrupted program's** resume address, call `acknowledge_alarm(state, pending_interrupt_alarm_index)` then clear the index. This re-arms repeating alarms (ALARM-03) **only when the handler actually ran** — naturally skips cap-drop (D-07) and missing-label (D-08).
- `resume_program` (468): clear `pending_interrupt` + `pending_interrupt_alarm_index` + `pending_yield` before re-entering `run_loop` (D-09 — drop interrupt set just before STOP).
- `run_program` (after 445): clear `pending_interrupt` so a stale interrupt never fires into fresh execution.

### PSE/VIEW/AVIEW yield consumers (criteria 4–5) — onto the same break boundary
These currently run in `execute_op` (884/885/902) and return `Ok(())` without breaking. Move their break into `run_loop` (mirror the `Op::Prompt` arm at 660), OR add dedicated `run_loop` arms ahead of the `other =>` catch-all:
- `Op::Pse`: compute `format_hpnum(stack.x, display_mode)` (as today at 904), set `pending_yield = Some(YieldState{ Pse, text, resume_ms: 1000 })`, `LiftEffect::Neutral`, then **break** (replaces the `"PAUSE 1000"` event push — D-04). `pc` already points at the next step, so resume continues correctly.
- `Op::View(reg)`: capture `op_view`'s formatted string (display_ops.rs:23) into `pending_yield{ View, text, resume_ms }`, then break.
- `Op::AView`: capture `op_aview`'s `alpha[..24]` (display_ops.rs:32) into `pending_yield{ Aview, text, resume_ms }`, then break.
- **`display_override` is NOT written by these new paths** (D-04). The carried `text` is the sole display source for the yield.

**VIEW/AVIEW duration (Claude's discretion):** recommend **reuse the PSE 1000 ms** for all three. On real HP-41 a program-mode VIEW/AVIEW shows the value and (without a following PSE) the program runs on; the faithful "brief glance" for an emulator is the same ~1 s PSE cadence. A separate shorter "brief" constant is acceptable but adds a knob for imperceptible gain. **Recommendation: single `PSE_RESUME_MS = 1000`.**

### Auto-resume (frontend, both families)
- The break returns control with `pending_yield` (or, for an idle/error path, nothing). Frontend renders `pending_yield.text`, waits `resume_ms`, calls `resume_program`, which clears the channel and re-enters `run_loop` at the already-advanced `pc`.

---

## Frontend Wiring

Both frontends land in **this** phase (D-25.6 / I-08). Neither touches `CalcState` enums — they read `event_buffer` strings and the projected yield channel.

### CLI (`hp41-cli/src/app.rs`)
- **Yield rendering + resume:** after `run_program`/`resume_program` returns with `pending_yield` set, render `pending_yield.text` on the display, **sleep `resume_ms`** (CLI may sleep — the no-sleep rule is `hp41-core`-only), then call `resume_program`; loop until `pending_yield` is `None` and the program has stopped. This is the CLI's PSE/VIEW/AVIEW pause (criteria 4–5).
- **Interrupting alarms:** the `pending_interrupt` path executes entirely inside `run_program`'s `run_loop` (no CLI branch needed for the *running* case). The idle/demoted case still arrives as `alarm:xeq:{label}` and is handled by the existing branch at 1783.
- **Missing label (D-08):** in `drain_event_buffer` (1778), add an arm for `alarm:missing:{label}` → set `self.message = Some(format!("Alarm XEQ {label}: label not found"))` (status-line surface). Remove/replace the "silently ignored" comment at 1797.
- **Print discipline (I-07):** any alarm-handler output (PRX/PRA/PRSTK) staged during the interrupt must drain via `drain_and_show_print_output()` (1865) after the `run_program` call returns — the 1791–1792 pattern already does this for `alarm:xeq`; ensure the interrupt path's enclosing `run_program` call site (the R/S keystroke handler) also drains.

### GUI (`hp41-gui/src-tauri/src/commands.rs` + `hp41-gui/src/App.tsx`)
- **Yield rendering + resume:** project `pending_yield` through `CalcStateView` (a `kind`/`text`/`resume_ms` triple). The TS layer renders `text` on the main display, then **schedules `resume_program` via the existing `setInterval(tick_time)` cadence** (App.tsx:528) after `resume_ms` — Mutex is released between yields (D-01), so the GUI stays responsive (D-11: no `get_state` polling; resume is scheduled, not polled).
- **Interrupting alarms:** Phase-C `check_alarms` inside `run_loop` handles the *running* case server-side (no TS branch). Idle/demoted cases arrive as `alarm:xeq:{label}` and are dispatched at App.tsx:1172.
- **Missing label (D-08):** in the TS event consumer (1166–1187), replace the `alarm:interrupting:` silent-ignore (1180–1181) handling and add `alarm:missing:{label}` → `showToast("Alarm XEQ {label}: label not found")`.
- **iOS blind spot:** after touching `commands.rs` event handling, run `cargo check --target aarch64-apple-ios --manifest-path hp41-gui/src-tauri/Cargo.toml`.

---

## Validation Architecture

Test framework: Rust integration tests in `hp41-core/tests/` (cargo, via `just test` / `just test-core`) + Vitest for GUI TS. File-level `#[allow(clippy::unwrap_used)]` per existing convention (e.g. display_ops.rs:77).

**Test files (new):**
- `hp41-core/tests/phase_63_interrupting_alarms.rs` — alarm/run-loop engine tests (Scenarios 1–7 + edges).
- `hp41-core/tests/phase_63_yield_engine.rs` — PSE/VIEW/AVIEW yield tests.

**Deterministic-clock pattern (PITFALLS):** `trigger_unix = 0`, `time_offset_secs = 0`; assert on `CalcState` fields (`regs`, `call_stack`, `event_buffer`, `print_buffer`, `pending_yield.resume_ms`) — **never** on `SystemTime::now()`. Surface `resume_ms` as **data** and assert it; **never sleep** in core unit tests (the frontend owns the wait).

### Alarm scenarios (from PITFALLS — 7 + edges)

| # | Test name | One-line assertion | SC | Req |
|---|-----------|--------------------|----|-----|
| 1 | `interrupting_alarm_halts_running_program_and_resumes` | `regs[10]==1` (pre-interrupt), `regs[12]==99` (handler ran), `regs[11]==2` (resumed), `call_stack` empty, `is_running==false` | 1 | ALARM-02/03 |
| 2 | `interrupt_preserves_stack_x_y_z_t_and_lift_state` | `stack.lift_enabled` at resume == value at interrupt; `pc` resumes at correct step (not 0) | 1 | ALARM-02 |
| 3 | `interrupt_blocked_when_call_stack_at_4_level_cap` | alarm label NOT run; `call_stack` still 4 (unchanged); no `CallDepth` propagated; `is_running` unchanged; `past_due` left true | 2 | ALARM-03 |
| 4 | `interrupt_nesting_blocked_when_already_in_alarm_program` | second interrupt does NOT recurse; demoted to `event_buffer`; depth never > 4 | 2 | ALARM-03 |
| 5 | `interrupting_alarm_fires_when_no_program_running` | idle (`is_running==false`) → `alarm:xeq:{label}` queued; no resume logic; `call_stack` empty after | 3 | ALARM-02 |
| 6 | `non_interrupting_alarm_still_fires_as_event_not_inline` | `event_buffer` has `alarm:xeq:LABEL`; NOT run inline; `is_running` unchanged (DNT-05 regression) | — | ALARM-03 |
| 7 | `message_alarm_still_fires_to_event_buffer_not_executed` | `event_buffer` has `alarm:message:{text}`; `print_buffer` has text; no program execution | — | — |

### Yield (PSE/VIEW/AVIEW) tests

| Test name | One-line assertion | SC | Req |
|-----------|--------------------|----|-----|
| `pse_mid_run_breaks_and_records_resume_ms` | `Op::Pse` mid-program sets `pending_yield{kind:Pse, text:fmt(X), resume_ms:1000}` and `run_loop` breaks (no `display_override` write) | 4 | PRGM-01 |
| `pse_resume_continues_to_next_step` | after `resume_program`, execution continues at the step after PSE; `pending_yield` cleared | 4 | PRGM-01 |
| `view_mid_run_captures_formatted_register_into_yield` | `pending_yield.text == format_hpnum(regs[reg],…)`; breaks; resumes | 5 | PRGM-02 |
| `aview_mid_run_captures_alpha_into_yield` | `pending_yield.text == alpha[..24]`; breaks; resumes | 5 | PRGM-02 |

### Edge tests

| Test name | One-line assertion | Decision |
|-----------|--------------------|----------|
| `interrupt_demoted_when_solver_or_modal_active` | with `integ_state`/`solve_state`/`difeq_state` Some OR `modal_program` Some → `alarm:xeq:{label}` to `event_buffer`, no synthetic frame | D-10 |
| `missing_handler_label_surfaces_event` | unknown label → `event_buffer` has `alarm:missing:{label}`; no panic; no ack | D-08 |
| `pending_interrupt_cleared_on_resume_after_stop` | interrupt set before `Op::Stop`; `resume_program` clears it; resume does NOT redirect to handler | D-09 |
| `v4_3_interrupt_backward_compat` | a v4.2-era `autosave.json` deserializes; `pending_interrupt`/`pending_yield`/index default `None` | I-03 |
| `repeating_interrupting_alarm_reschedules_after_handler` | after handler RTN, `acknowledge_alarm` advanced `trigger_unix`; cap-drop/missing-label do NOT reschedule | D-06 |

**Coverage gate:** `hp41-core` ≥ 95% lines / ≥ 93% regions (`just coverage`). New code + new tests should hold or improve it. **`hp41-cli` is NOT coverage-gated** — the `drain_event_buffer` missing-label arm and CLI yield-resume loop need targeted CLI tests in `hp41-cli/src/` or manual verification as the backstop. GUI: add a Vitest for the yield render + an event-consumer test; an E2E alarm-fires-toast smoke is a "should-have" (CI gap noted in PITFALLS).

---

## Frozen-Invariant Checklist (pre-commit gate, this phase)

- **I-01 math1 freeze:** `git diff HEAD -- 'hp41-core/src/ops/math1/'` shows nothing (all work in `program.rs` / `state.rs` / `time/alarm.rs`).
- **I-02 LiftEffect:** interrupt preserves `stack.lift_enabled`; PSE/VIEW/AVIEW keep `LiftEffect::Neutral`; do NOT clear lift on the interrupt frame.
- **I-03 serde:** every new field `#[serde(default, skip)]`; v4.2 autosave deserializes; `migrate_after_load` review pass only.
- **I-04 no async / no panic:** synchronous only; `?`/`.expect("invariant: …")`, zero new `unwrap()` (`grep -rn 'unwrap()' hp41-core/src/ | grep -v '#\[allow'` → empty).
- **I-05 4-way match:** **no new `Op` variant** (D-12) → invariant not triggered. If that changes, all four sites in one commit.
- **I-06 zero new deps:** `git diff HEAD -- hp41-core/Cargo.toml` empty; reuse `Vec<usize>` / `usize` / `Option`.
- **I-07 no `println!` in core:** alarm-handler output via `print_buffer`; CLI drains via `drain_and_show_print_output()` (app.rs:1865). `grep -rn 'println!\|eprintln!' hp41-core/src/` → empty.
- **I-08 / D-25.6 parity:** CLI (app.rs:1797) + GUI (App.tsx:1180) interrupting/missing-label handling land in the SAME phase.
- **DNT-05:** non-interrupting `alarm:xeq` arm (alarm.rs:508) unchanged; Scenario 6 guards it.
- **D-11 prefix semantics:** do NOT touch `parse_alarm_type` (alarm.rs:80–87); ADR v4.3-003 confirms `>>`=interrupting is correct.

**Guard commands:**
```
just ci                                   # lint + test + coverage + license-audit
cargo +1.88 clippy --workspace --all-targets --all-features -- -D warnings   # MSRV-lint divergence
cd hp41-gui/src-tauri && cargo clippy --all-targets -- -D warnings           # GUI clippy (ungated by CI)
cargo check --target aarch64-apple-ios --manifest-path hp41-gui/src-tauri/Cargo.toml   # cfg(mobile) blind spot
```

**Docs to update (Phase G):** flip `docs/hp41-time-divergences.md` §D-40-04 to "implemented via pending-interrupt at run-loop boundary (v4.3)"; add ADR `docs/adr/v4.3-001-interrupt-alarm-pending-field.md`.

---

## Open Items for Planner

Genuinely Claude's-discretion (from CONTEXT) — nothing else is open:

1. **Yield-channel field shape/naming (D-04):** recommended `pending_yield: Option<YieldState{ kind: YieldKind, text: String, resume_ms: u64 }>`, `#[serde(default, skip)]`. Planner finalizes names.
2. **Alarm-index tracking (D-06a):** recommended paired field `pending_interrupt_alarm_index: Option<usize>` over scanning `state.alarms` (disambiguates same-fire-time alarms).
3. **VIEW/AVIEW duration:** recommended **reuse PSE 1000 ms** (`PSE_RESUME_MS = 1000`) for all three; a separate "brief" constant is acceptable but unnecessary.
4. **Test-fixture timing strategy:** surface `resume_ms` as data and assert it in core unit tests; never wall-clock sleep in `hp41-core` tests; use `trigger_unix = 0` / `time_offset_secs = 0`.

---

*No "⚠ Flag for human" items — every D-01..D-13 decision is internally consistent with the ground-verified code; all anchors hold.*
