# Phase 22: Program Control & Memory Ops — Context

**Gathered:** 2026-05-14
**Status:** Ready for planning
**Research-OQ resolutions (2026-05-14):**
- **OQ-1 → Option B (hardware-faithful CATALOG):** CAT 1 = programs;
  CAT 2/3/4 = "NOT AVAILABLE". Register-listing dropped. D-22.16,
  D-22.16.1, D-22.16.2, D-22.16.3 below are AMENDED.
- **OQ-2 → Option A (SIZE 0 clamps to 1):** D-22.11 AMENDED below.
- **OQ-3 → Option A (ASN empty name removes):** D-22.18 AMENDED below.
- **OQ-4:** Op::Cla shows "CLA"; legacy Op::AlphaClear keeps "CLRALPHA"
  for v1.x save-file fidelity. Acknowledged, no code change beyond
  D-22.13. Will be flagged in 22-03 commit message and Phase 22
  SUMMARY.

<domain>
## Phase Boundary

Land 14 new HP-41CV ROM ops in `hp41-core` covering program-flow control
(`STOP`, `PSE`, `GTO IND`, `XEQ IND`), program editing (`CLP`, `DEL`, `INS`),
memory & stack management (`SIZE`, `CLA`, `CLST`, `PACK`), and key
assignments (`CATALOG`, `ASN`). All hp41-core only — keyboard wiring,
modals, prompts, and help text are deferred to Phase 25 (CLI) and Phase 26
(GUI). The indirect-resolver helper that the rest of the IND family will
share lives in Phase 24; Phase 22's `GTO IND` / `XEQ IND` ship with a
minimal inline check that Phase 24 extracts.

**In scope:** new `Op` variants in `ops/mod.rs`, dispatch arms, `execute_op`
arms, both `prgm_display.rs` copies (`hp41-cli` + `hp41-gui`), the new
`assignments: BTreeMap<u8, String>` field on `CalcState`, the new
`resume_program()` entry point in `ops/program.rs`, in-module unit tests,
and a single integration test per plan covering each success criterion.

**Out of scope (Phase 25):** keyboard wiring in `hp41-cli/src/keys.rs`,
`KEY_REF_TABLE` entries, new `PendingInput` modal variants for
`AsnNamePrompt`/`AsnKeyPrompt`/`DelPrompt`/`ClpLabelPrompt`, R/S key
routing to `resume_program()`, `help_data.rs` updates,
`pending_prompt()` arms.

**Out of scope (Phase 26):** `key_map.rs::resolve` entries, `KEY_DEFS`
bindings in `Keyboard.tsx` for the new ops, modal routing for the
previously-stubbed prompt IDs, GUI USER-mode rewire that reconciles
`assignments` (new) with `key_assignments` (Phase 5).

**Out of scope (Phase 24):** the shared `resolve_indirect()` helper and the
other ~13 IND variants (`STO IND`, `RCL IND`, `SF IND`, `FS? IND`, etc.).
Phase 22's `GTO IND`/`XEQ IND` use an inline integer-part check; Phase 24
refactors the inline check into the shared helper.

**Out of scope (Phase 27):** extending the 500-case numerical-accuracy
suite for the new ops (none of them are math-precision-sensitive, but
flag-semantics proptest + indirect-addressing integration tests live in
Phase 27).

</domain>

<decisions>
## Implementation Decisions

### Interpreter control flow (STOP / PSE / R-S)

- **D-22.1 — STOP mechanism:** `Op::Stop` breaks `run_loop` mirroring the
  Phase 21 `Op::Prompt` pattern. The `run_loop` arm sets nothing besides
  the implicit `break` — state.pc already points at the step AFTER STOP
  (the loop advances pc before the `match` per program.rs:189). On
  function exit from `run_program`, the existing safety-reset sets
  `is_running = false`. STATE INVARIANT: after a STOP halt,
  `state.pc < state.program.len()` AND `state.is_running == false`.

- **D-22.2 — `resume_program()` entry point:** new public function
  `pub fn resume_program(state: &mut CalcState) -> Result<(), HpError>`
  in `hp41-core/src/ops/program.rs`. Sets `state.is_running = true`,
  re-enters `run_loop(state, &state.program.clone())` from current
  `state.pc`, then resets `is_running = false` on exit (same safety
  pattern as `run_program`). Returns `Err(HpError::InvalidOp)` if
  `state.pc >= state.program.len()` (nothing to resume).

- **D-22.3 — R-S routing:** R-S is a frontend (hp41-cli / hp41-gui)
  concern. The keyboard handler checks `state.is_running` to decide
  semantics: false + non-empty pc → call `resume_program(state)`. true
  → set a "stop next iteration" sentinel (deferred to Phase 25 — Phase
  22 lands `resume_program()` only). Interactive dispatch of `Op::Stop`
  (when `is_running == false`) is a no-op + Neutral lift. The v2.1
  `run_stop` Tauri command's current cosmetic `is_running` toggle is
  superseded — Phase 25/26 will wire it to `resume_program()`.

- **D-22.4 — PSE encoding:** `Op::Pse` writes the current X (formatted
  via `format_hpnum` honoring display_mode) to `state.display_override`
  AND pushes a structured event line `"PAUSE 1000"` into
  `state.event_buffer`. `run_loop` does NOT break — execution continues
  to the next step. Frontend reads the event_buffer (existing drain
  pattern), inserts a real ~1 s delay before processing the next
  display refresh. Mirrors `Op::Beep` / `Op::Tone` precedent exactly;
  hp41-core stays purely synchronous and clock-free. LiftEffect:
  Neutral.

- **D-22.5 — `Op::Stop` scope:** programs only. Interactive dispatch
  (`is_running == false`) returns Ok + Neutral lift. Inside `run_loop`,
  the `Op::Stop` arm breaks the loop (D-22.1). Symmetric with the
  v1.0 `Op::Pse` ban inside `execute_op()` (the catch-all arm at
  program.rs:455–465 already lists `Op::Prompt` — Phase 22 adds
  `Op::Stop | Op::Pse` to that catch-all only if they aren't
  handled directly by run_loop. Decision: handle Stop in run_loop's
  match (so the catch-all does NOT list it); Pse goes through
  `execute_op` because it doesn't break.

- **D-22.6 — Code location:** `resume_program()` lives next to
  `run_program()` in `hp41-core/src/ops/program.rs`. The `Op::Stop`
  arm sits in `run_loop`'s match next to `Op::Prompt`. The `Op::Pse`
  arm goes into `execute_op`'s body. New tests join the existing
  `program_tests` module in the same file. Zero new files for the
  interpreter-control-flow plan.

### Program editing (CLP / DEL / INS)

- **D-22.7 — CLP boundary:** `Op::Clp(String)`. Find the target
  `Op::Lbl(name)` by exact string match. Delete from that index up to
  (but not including) the next `Op::Lbl`. If the target is the LAST
  labelled block, delete to end-of-Vec. Missing label →
  `HpError::InvalidOp`. Documented divergence from HP-41 hardware
  (which uses END/.END. boundaries); we use next-LBL because the flat-
  Vec model has no explicit END marker. LiftEffect: Neutral.

- **D-22.8 — INS blank step:** `Op::Ins` inserts `Op::Null` at
  `state.pc` (using `state.program.insert(state.pc, Op::Null)`).
  Op::Null (Phase 12) is already a true no-op + Neutral lift,
  displayed as `"NULL"` in prgm_display. State.pc stays unchanged
  (still points at the newly inserted Null). Zero new variants for
  the placeholder. LiftEffect: Neutral.

- **D-22.9 — DEL bounds:** `Op::Del(u8)`. Clamp silently to
  `min(nnn, program.len() - state.pc)`. If clamped to 0 (nnn == 0
  OR state.pc == program.len()), it's a no-op. After deletion, if
  state.pc >= program.len() (deleted to the end), leave state.pc at
  len — the next dispatch detects end-of-program. LiftEffect: Neutral.

- **D-22.10 — Edit-op scope:** `Op::Clp` / `Op::Del` / `Op::Ins`
  execute ONLY when interactive (is_running == false) AND
  `state.prgm_mode == true`. Inside `run_loop` they join the
  programming-ops catch-all (program.rs:456–465) and return
  `HpError::InvalidOp`. Interactive with `prgm_mode == false` →
  `HpError::InvalidOp`. Inside dispatch when `prgm_mode == true`,
  these ops execute immediately — they do NOT self-record. They
  ARE the program-editing primitives. Mirrors `Op::PrgmMode` (which
  acts on prgm_mode rather than being recorded into it).

### Memory & stack management (SIZE / PACK / CLA / CLST)

- **D-22.11 — SIZE:** `Op::Size(u16)` (u16 because 319 > u8::MAX).
  Accepted range `[0, 319]`. `nnn` is **silently clamped to `1` when
  `0`** (decision OQ-2 = Option A, 2026-05-14 user-confirmed). `nnn >
  319` returns `HpError::InvalidOp`. Resize logic:
  `state.regs.resize(nnn.max(1).min(319) as usize, HpNum::zero())`
  followed by a `nnn > 319 → InvalidOp` early-return. Shrinking
  truncates the tail (hardware-faithful "MEM LOST"). Growing
  zero-fills. LiftEffect: Neutral. Documented divergence: `SIZE 000`
  on real HP-41 leaves 0 registers — we clamp to 1 so subsequent
  STO/RCL remain dispatchable.

- **D-22.11.1 — regs bounds audit:** `op_sto(state, r)` /
  `op_rcl(state, r)` / `op_clreg(state)` and any other reads of
  `state.regs[i]` currently assume `len() == 100`. They must use
  `state.regs.len()` for bounds checks. Out-of-bound register access
  returns `HpError::InvalidOp`. This audit is a Wave-0 prep task
  for the memory-ops plan (must land + pass existing tests BEFORE
  Op::Size is wired).

- **D-22.12 — PACK:** `Op::Pack`. No-op + `apply_lift_effect(state,
  LiftEffect::Neutral)`. Returns Ok(()). Documented divergence:
  HP-41 PACK compacts program memory by removing gaps; our flat-Vec
  model has no gaps.

- **D-22.13 — CLA:** new `Op::Cla` variant. Implementation calls
  existing `ops::alpha::op_alpha_clear(state)` internally. Display
  name: `"CLA"`. Op::AlphaClear stays untouched (kept for v1.0
  save-file compat, displays as `"CLRALPHA"`). LiftEffect: Neutral.
  Rationale: hardware-faithful program listings show `CLA`, not
  `CLRALPHA`. Per the 4-place Op-variant rule, prgm_display name is
  fixed per variant — we can't multi-name AlphaClear.

- **D-22.14 — CLST:** new `Op::Clst` variant. Zeros X, Y, Z, T (set
  to `HpNum::zero()`). PRESERVES `lastx` and `lift_enabled`.
  LiftEffect: Neutral.

### Indirect branching (Phase 22 inline, Phase 24 generalizes)

- **D-22.15 — `Op::GtoInd(u8)` / `Op::XeqInd(u8)`:** new variants in
  Phase 22. Inline resolver in the run_loop arms:
  1. Read pointer = `state.regs[reg].trunc_int()` (reuses Phase 20's
     existing `HpNum::trunc_int()`).
  2. If pointer != integer (i.e., the truncation discarded data) →
     `HpError::InvalidOp`. Use `x == x.trunc_int()`-style check
     (FN-IND-02 wording).
  3. Convert pointer to string label (e.g., `pointer.to_string()`)
     and reuse the existing `find_in_program()` label search.
  4. GtoInd: set state.pc = target + 1 (mirrors Op::Gto run_loop arm).
     XeqInd: push state.pc, set state.pc = target + 1 (mirrors
     Op::Xeq run_loop arm, including the 4-deep call-stack check).
  5. Out-of-range reg (reg >= state.regs.len()) → `HpError::InvalidOp`.
  - Phase 24 will EXTRACT this inline check into a shared
    `resolve_indirect(state, reg) -> Result<u8, HpError>` helper and
    refactor `Op::GtoInd`/`Op::XeqInd` (and the ~13 other IND variants)
    to call it.

### CATALOG output format

- **D-22.16 — `Op::Catalog(u8)`:** writes to `state.print_buffer` (one
  formatted line per push). Header: `"-- CATALOG n --"` (24-char width,
  right-padded with spaces if shorter). Footer: `"-- END --"` (same
  padding). LiftEffect: Neutral. Invalid n (n == 0 OR n >= 5) →
  `HpError::InvalidOp`. **OQ-1 resolution (2026-05-14, user-confirmed
  Option B = hardware-faithful):** CATALOG 1 = programs (real HP-41
  semantics). CATALOG 2 = "NOT AVAILABLE" (no XROM modules in our
  emulator). Register-listing as a CATALOG slot is dropped from Phase
  22; a future quick-task or v3.0 may add a separate non-ROM
  inspection op.

- **D-22.16.1 — CATALOG 1 (programs):** iterate `state.program`,
  collect each `Op::Lbl(name)` position. For each LBL, the step count
  = distance from the LBL's index to the NEXT LBL's index (or
  program.len() for the last LBL). Emit: `format!("LBL {:9}  {:5}",
  name, steps)`. 24-char width. (Was "registers" pre-OQ-1; swapped to
  hardware-faithful 2026-05-14.)

- **D-22.16.2 — CATALOG 2 (XROM modules):** single payload line
  `"NOT AVAILABLE         "` (24-char-padded) between header and
  footer. No-error path; signals "no XROM modules installed". (Was
  "programs" pre-OQ-1.)

- **D-22.16.3 — CATALOG 3 (HP-IL) / CATALOG 4 (extended memory):**
  single payload line `"NOT AVAILABLE         "` (24-char-padded)
  between header and footer. No-error path; signals "no such hardware"
  cleanly. Mirrors CAT 2's "NOT AVAILABLE" pattern.

### Key assignments (ASN)

- **D-22.17 — `assignments: BTreeMap<u8, String>` field:** new field
  on `CalcState` with `#[serde(default)]`. Maps HP-41 key code (row×10
  + col, 1-indexed; same encoding as Phase 12's `last_key_code` and
  Phase 19's `keycode_to_hp41_code`) to the assigned target name.
  Coexists with existing `key_assignments: BTreeMap<char, String>`
  (Phase 5, char-keyed, used by hp41-cli USER mode F1-F4 + a-b-c-d).
  Two maps, two purposes — Phase 25/26 will reconcile the wiring when
  CLI/GUI USER-mode routing is rewired. BTreeMap ensures
  deterministic JSON serialization order (mirrors D-25 / D-29 of
  Phase 5).

- **D-22.18 — `Op::Asn { name: String, key_code: u8 }`:** new struct-
  variant. dispatch() body: **if `name.is_empty()` →
  `state.assignments.remove(&key_code)`; else
  `state.assignments.insert(key_code, name)`** (decision OQ-3 =
  Option A, 2026-05-14 user-confirmed). Then
  `apply_lift_effect(state, LiftEffect::Neutral)`. The 2-step
  keyboard modal flow (alpha prompt → key prompt) is a hp41-cli /
  hp41-gui concern (Phase 25 introduces `PendingInput::AsnNamePrompt`
  → `AsnKeyPrompt`). The op variant is fully-formed by the caller
  when it reaches dispatch. Hardware-faithful "ASN '' 11" undoes
  "ASN 'SIN' 11".

- **D-22.19 — Resolution at USER-mode dispatch:** late-binding via
  String. When a key with an assignment is pressed in USER mode
  (Phase 25/26 wiring), the frontend resolves `name` by: (1) try
  parse-as-Op (uppercase match against built-in op names — "SIN",
  "COS", "+", "STO 05", "GTO ABC", etc.); (2) on miss, fall back to
  LBL search via `run_program(state, &name)`. The hp41-core layer
  knows nothing about resolution — it just stores the assignment.
  Save-file safety: stale assignments (label deleted, op renamed)
  surface as runtime errors during USER-mode dispatch, not at load
  time. Matches the existing Phase 5 `key_assignments` precedent
  where label strings are dereferenced at dispatch.

### Plan structure

- **D-22.20 — 4 plans by domain:**
  - **22-01-program-control-PLAN.md** — `Op::Stop`, `Op::Pse`,
    `resume_program()`, `Op::GtoInd(u8)`, `Op::XeqInd(u8)`. Covers
    FN-PROG-01, FN-PROG-02, FN-PROG-06, FN-PROG-07. Wave-0 prep:
    none.
  - **22-02-program-edit-PLAN.md** — `Op::Clp(String)`, `Op::Del(u8)`,
    `Op::Ins`. Covers FN-PROG-03, FN-PROG-04, FN-PROG-05. Depends
    on 22-01 only via file-overlap on `ops/mod.rs` / `ops/program.rs`
    / both `prgm_display.rs` copies — sequential.
  - **22-03-memory-ops-PLAN.md** — `Op::Size(u16)`, `Op::Cla`,
    `Op::Clst`, `Op::Pack`. Covers FN-MEM-01, FN-MEM-02, FN-MEM-03,
    FN-MEM-04. Wave-0 prep: `op_sto`/`op_rcl`/`op_clreg` bounds-
    audit (D-22.11.1). Sequential vs 22-01/22-02.
  - **22-04-catalog-and-asn-PLAN.md** — `Op::Catalog(u8)`,
    `Op::Asn { name, key_code }` (and the new `assignments` field on
    CalcState). Covers FN-MEM-05, FN-KEY-01. Sequential vs 22-03
    (state.rs touch).
  - File-overlap on `state.rs`, `ops/mod.rs`, `ops/program.rs`, both
    `prgm_display.rs` copies forces strict sequential execution
    (same constraint Phase 21 hit). The Wave numbers can still be
    assigned (e.g., everything Wave-1 with explicit `depends_on`
    chains) — `gsd-execute-phase` will serialize regardless.

### Cross-cutting (locked, not gray)

- **D-22.21 — 4-place Op-variant landing:** every new variant
  (`Stop`, `Pse`, `GtoInd`, `XeqInd`, `Clp`, `Del`, `Ins`, `Size`,
  `Cla`, `Clst`, `Pack`, `Catalog`, `Asn`) goes into
  `ops/mod.rs::Op` enum + `dispatch()` + `execute_op()` (in
  `ops/program.rs`) + BOTH `prgm_display.rs` copies (`hp41-cli` +
  `hp41-gui`). Programming-ops (Stop, Clp, Del, Ins, GtoInd, XeqInd)
  join the catch-all that returns `HpError::InvalidOp` from
  `execute_op` for cases that are handled directly by `run_loop`.

- **D-22.22 — Save-file compat:** new `CalcState.assignments` field
  carries `#[serde(default)]`. All new Op variants are additive in
  the `Op` enum — older save files don't contain them, so no
  migration needed. Phase 22's `Op::Stop` etc. land at the END of
  the `Op` enum (so existing enum-discriminant order is preserved
  for any tests that depend on it — though none should, per existing
  practice).

- **D-22.23 — Zero-panic policy:** `#![deny(clippy::unwrap_used)]`
  enforced. All new code uses `?`-propagation or `.expect("reason")`.
  Test modules carry `#[allow(clippy::unwrap_used)]`. Particular
  attention: `state.regs[reg]` raw indexing must be replaced with
  `state.regs.get(reg as usize).ok_or(HpError::InvalidOp)?` patterns
  everywhere new code touches regs by index.

- **D-22.24 — SC-4 invariant:** no `op_*` / `flush_entry_*` /
  `format_hpnum` added to `hp41-gui/src-tauri/`. Only
  `prgm_display.rs` exhaustive-match updates allowed there.

- **D-22.25 — LiftEffect summary:**
  - `Stop`: Neutral (run_loop break is structural, not stack-affecting)
  - `Pse`: Neutral
  - `GtoInd` / `XeqInd`: Neutral
  - `Clp` / `Del` / `Ins`: Neutral
  - `Size`: Neutral
  - `Cla` / `Clst` / `Pack`: Neutral
  - `Catalog`: Neutral
  - `Asn`: Neutral

  Phase 22 is overwhelmingly Neutral — these are control-flow,
  program-edit, and memory-management ops that do not push a new
  value onto the stack. Compare to Phase 20 (math) where every op
  was Enable.

### Claude's Discretion

- Exact display strings in `prgm_display.rs` for the new variants
  (`"STOP"`, `"PSE"`, `"CLA"`, `"CLST"`, `"PACK"`, `"SIZE nnn"`,
  `"DEL nnn"`, `"INS"`, `"CLP name"`, `"GTO IND nn"`, `"XEQ IND nn"`,
  `"CATALOG n"`, `"ASN \"name\" nn"`) — planner picks final text;
  match real HP-41 listing conventions.
- Test layout per plan (inline `#[cfg(test)]` mods vs centralized
  `tests/phase22_*.rs` integration suites) — planner decides;
  precedent is mixed (Phase 20: integration test; Phase 21: split
  across plans with per-plan integration suites).
- Exact wording of `"PAUSE 1000"` event_buffer line for PSE —
  planner picks; suggest something parseable by the frontend (e.g.,
  `"PAUSE 1000"` for ms or `"PSE"` as a marker).
- Whether `Op::Size` uses `u16` or just a u32 — `u16` is sufficient
  (max 319). Planner picks; `u16` is fine.
- Whether the bounds audit in D-22.11.1 lives as its own commit or
  is folded into the Op::Size implementation — planner decides;
  separate commit recommended for git-blame clarity.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-level anchors
- `.planning/PROJECT.md` — build sequence (core → cli → docs → gui →
  tests), shipped milestones, architectural invariants.
- `.planning/REQUIREMENTS.md` §36–66 (FN-PROG, FN-MEM, FN-KEY) — the
  13 requirements this phase delivers.
- `.planning/ROADMAP.md` §77–94 (Phase 22 details, success criteria,
  cross-cutting constraints).
- `.planning/STATE.md` §Key Decisions — settled v1.0–v2.1 architecture
  decisions (BCD/f64, stack-lift, LiftEffect-per-op, zero-panic
  policy, 4-place Op-variant rule).
- `CLAUDE.md` §"Settled Architecture Decisions" — full settled-decision
  catalog, including v2.0 IPC contract, v2.1 keyboard-authenticity
  patterns, the SC-4 invariant grep pattern, and the v1.1 print-
  buffer drain pattern that PSE will extend.

### Code references that constrain Phase 22

**Interpreter / run_loop:**
- `hp41-core/src/ops/program.rs:139–169` — `run_program()` entry point.
  `resume_program()` mirrors this almost exactly: clone program, set
  is_running = true, enter run_loop, reset is_running on exit.
- `hp41-core/src/ops/program.rs:177–285` — `run_loop()` body. Op::Stop
  arm goes next to Op::Prompt (line ~272). Op::GtoInd / Op::XeqInd
  arms go near Op::Gto (line ~201) and Op::Xeq (line ~205) — same
  shape, one extra `state.regs[reg]` dereference + integer check.
- `hp41-core/src/ops/program.rs:293–467` — `execute_op()`. Op::Pse
  goes here (writes display_override + event_buffer, returns Ok).
  Op::Stop and Op::GtoInd/XeqInd are run_loop-only; if reached in
  execute_op (interactive dispatch in program context), they join
  the catch-all that returns InvalidOp.
- `hp41-core/src/ops/program.rs:455–465` — programming-ops catch-all
  in execute_op. Op::Stop, Op::Clp(_), Op::Del(_), Op::Ins, Op::GtoInd
  /Op::XeqInd join this list. Op::Pse does NOT (it executes fine in
  programs — it's the entire point of PSE).
- `hp41-core/src/ops/program.rs:506–534` — `parse_counter()` and
  `build_counter()` helpers for ISG/DSE. Not directly used by Phase
  22, but instructive for the kind of inline string-split helpers
  Phase 24 will introduce for the indirect-resolver.
- `hp41-core/src/ops/program.rs:536–551` — `find_in_program()` and
  `find_label_in_state()` — REUSE these for GtoInd/XeqInd label
  search (after converting the integer pointer to a string).

**State / persistence:**
- `hp41-core/src/state.rs:53–142` — `CalcState` struct. Phase 22 adds
  ONE new field: `assignments: BTreeMap<u8, String>` with
  `#[serde(default)]`. Slot it next to `key_assignments` (line 88)
  for grep affinity.
- `hp41-core/src/state.rs:80–88` — existing `is_running`, `prgm_mode`,
  `pc`, `call_stack`, `key_assignments` fields. Phase 22 reads
  is_running (for the resume-program guard), reads prgm_mode (for
  CLP/DEL/INS scope), reads and writes pc (for resume + edit
  operations), reads call_stack (for XeqInd 4-deep check), and reads
  key_assignments only for awareness (D-22.17).

**Existing primitives Phase 22 reuses:**
- `hp41-core/src/ops/alpha.rs::op_alpha_clear()` (line 34) — Op::Cla
  delegates to this (D-22.13).
- `hp41-core/src/ops/registers.rs::op_clreg()` (line 98) — pattern
  reference for Op::Clst (zeroing pattern, but for stack instead of
  regs).
- `hp41-core/src/num.rs::HpNum::trunc_int()` — used by GtoInd/XeqInd
  integer-pointer extraction (D-22.15 step 1).
- `hp41-core/src/num.rs::HpNum::zero()` — used by Op::Clst (X/Y/Z/T
  zeroing) and Op::Size (Vec resize zero-fill).
- `hp41-core/src/format.rs::format_hpnum()` — used by CATALOG 1 and
  Op::Pse for display formatting.
- `hp41-core/src/stack.rs::apply_lift_effect()` — used by every new
  variant for the Neutral lift-effect declaration.

**Both prgm_display.rs copies (the 4-place rule):**
- `hp41-cli/src/prgm_display.rs::op_display_name()` — must add 13
  new arms. Reuse Phase 20/21 precedent for the per-op match style.
- `hp41-gui/src-tauri/src/prgm_display.rs::op_display_name()` — same
  13 arms. Duplication is intentional (per CLAUDE.md §SC-4 note).

**Error surface:**
- `hp41-core/src/error.rs::HpError` — Phase 22 uses existing variants
  ONLY: `InvalidOp` (most cases), `CallDepth` (XeqInd 4-deep). No
  new variants needed. Keeps the error surface stable for hp41-cli /
  hp41-gui display formatting.

### Prior-phase decisions that flow forward

- **Phase 3** (Programming Engine): the entire run_loop pattern,
  Op::Lbl/Op::Gto/Op::Xeq label semantics, call_stack 4-deep limit,
  parse_counter ISG/DSE string-splitting. Phase 22 inherits all of
  this and adds resume_program + 5 new run_loop arms.
- **Phase 11** (Print Emulation): `print_buffer: Vec<String>` is the
  channel for CATALOG output. Existing op_prx/op_pra/op_prstk show
  the 24-char width convention. The mandatory drain pattern (cli's
  `call_dispatch_and_drain` / `drain_and_show_print_output` and the
  v2.0 GUI's per-command drain) handles CATALOG output the same way.
- **Phase 12** (Synthetic Programming): Op::Null is the established
  "no-op placeholder" variant that Op::Ins reuses (D-22.8).
  `last_key_code` field uses the same row×10+col encoding that ASN
  key codes use (D-22.17).
- **Phase 21** (Flags, Display Control, Sound): the
  `display_override: Option<String>` and `event_buffer: Vec<String>`
  fields that Op::Pse writes to (D-22.4). The break-with-state
  pattern from Op::Prompt that Op::Stop replicates (D-22.1). The
  dispatch-top `display_override` clear (Pitfall 5) means Op::Pse's
  display_override write at run_loop time survives until the NEXT
  dispatch — which is exactly the desired 1-step PSE display window
  (the frontend's 1s delay happens between the run_loop iteration
  that wrote display_override and the iteration that clears it).
- **Phase 5** (Persistence & UX): the existing
  `key_assignments: BTreeMap<char, String>` and the USER-mode
  routing in `hp41-cli/src/app.rs:304–337`. Phase 22's new
  `assignments: BTreeMap<u8, String>` coexists; Phase 25/26 will
  reconcile.

### External reference (HP-41 hardware spec)

- HP-41C/CV Owner's Manual — for STOP/PSE timing (~1s pause),
  CATALOG paging behavior, ASN key-code encoding, CLP boundary
  conventions (END/.END.). Researcher should cross-reference if any
  of these decisions need verification, but the locked-in decisions
  above are sufficient for planning.
- HP-41CV Service Manual — for SIZE register-count semantics (1..319
  hardware range, MEM LOST behavior on shrink).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`run_program(state, label)`** (`program.rs:139`) — exact template
  for `resume_program(state)`. Just skip the entry_label search
  (state.pc already set) and call run_loop.
- **`run_loop` Op::Prompt arm** (`program.rs:272–276`) — exact
  template for the Op::Stop arm. Write nothing (state already has
  pc pointing at next step + display_override unchanged), then break.
- **`find_in_program(program, label)`** (`program.rs:537`) — reused by
  GtoInd/XeqInd label search (after stringifying the integer
  pointer).
- **`op_alpha_clear(state)`** (`alpha.rs:34`) — Op::Cla wraps this.
- **`apply_lift_effect(state, LiftEffect::Neutral)`** (`stack.rs`) —
  used by every Phase 22 op.
- **`HpNum::trunc_int()`** (`num.rs:213–226`) — used by GtoInd/XeqInd
  integer-pointer extraction. Same helper Phase 20's `Op::Fact`
  uses for the integer-check pattern.
- **`format_hpnum(value, display_mode)`** (`format.rs`) — CATALOG 1
  and PSE format X for display.
- **`state.event_buffer.push(line)`** (Phase 21 sound.rs pattern) —
  PSE pushes `"PAUSE 1000"` here.
- **`state.print_buffer.push(line)`** (Phase 11 print.rs pattern) —
  CATALOG writes to this.

### Established Patterns
- **Break-with-state in run_loop**: Op::Prompt (Phase 21) breaks the
  run_loop after writing `display_override`. Op::Stop replicates the
  break exactly (no display write — STOP's display behavior is
  "freeze current X" which is achieved by NOT clearing
  display_override, since the previous step's value persists until
  the next dispatch). state.pc is already advanced past the STOP
  step by the run_loop top-of-iteration `pc += 1` (line 189).
- **event_buffer for sub-second timing signals**: BEEP and TONE n
  use event_buffer to signal the frontend to play audio (it can't be
  done in core). PSE extends this pattern for "pause 1000ms".
  Single-channel buffer + frontend interpretation = clean separation.
- **print_buffer for bulk text output**: PRX/PRA/PRSTK push formatted
  lines. CATALOG extends this same pattern — header line, payload
  lines, footer line. 24-char width is the existing convention.
- **Programming-ops catch-all in execute_op**: Op::Lbl(_) | Op::Gto(_)
  | Op::Xeq(_) | Op::Rtn | Op::PrgmMode | Op::Test(_) | Op::Isg(_) |
  Op::Dse(_) | Op::FlagTest{..} | Op::Prompt return InvalidOp from
  execute_op (handled by run_loop directly). Phase 22 adds Op::Stop,
  Op::Clp(_), Op::Del(_), Op::Ins, Op::GtoInd(_), Op::XeqInd(_) to
  this catch-all. Op::Pse does NOT join (it executes in execute_op).
- **`#[serde(default)]` for new fields**: Phase 12's reg_m/n/o,
  Phase 21's flags/display_override/event_buffer all use this.
  Phase 22's new `assignments` follows the same pattern. Loading a
  v1.0–v2.1 save file → assignments deserializes to empty BTreeMap.
- **BTreeMap (not HashMap) for serializable maps**: deterministic
  JSON serialization order. Phase 5 (`key_assignments`) and the new
  Phase 22 (`assignments`) both use BTreeMap.

### Integration Points

**ops/mod.rs::Op enum** — add 13 new variants AT THE END (preserve
existing discriminant order):
```rust
Stop,
Pse,
Clp(String),
Del(u8),
Ins,
GtoInd(u8),
XeqInd(u8),
Size(u16),
Cla,
Clst,
Pack,
Catalog(u8),
Asn { name: String, key_code: u8 },
```

**ops/mod.rs::dispatch()** — add 13 match arms (each calls the new
op function or, for Op::Stop / Op::Clp(_) / Op::Del(_) / Op::Ins /
Op::GtoInd(_) / Op::XeqInd(_), goes through a guard checking
`is_running` and `prgm_mode` first — see D-22.5 / D-22.10).

**ops/program.rs::run_loop()** — add 3 new direct arms: Op::Stop,
Op::GtoInd(reg), Op::XeqInd(reg). Op::Pse falls through to
execute_op (handled there).

**ops/program.rs::execute_op()** — add Op::Pse, Op::Size, Op::Cla,
Op::Clst, Op::Pack, Op::Catalog, Op::Asn arms. Add Op::Stop,
Op::Clp(_), Op::Del(_), Op::Ins, Op::GtoInd(_), Op::XeqInd(_) to the
programming-ops catch-all.

**state.rs::CalcState** — add ONE new field, `assignments`, with
`#[serde(default)]`.

**hp41-cli/src/prgm_display.rs::op_display_name()** — add 13 arms.

**hp41-gui/src-tauri/src/prgm_display.rs::op_display_name()** — add
the same 13 arms (per the 4-place rule).

### Concrete signature sketches (planner-consumed)

```rust
// ops/program.rs
pub fn resume_program(state: &mut CalcState) -> Result<(), HpError> {
    if state.pc >= state.program.len() {
        return Err(HpError::InvalidOp);
    }
    let program = state.program.clone();
    state.is_running = true;
    let result = run_loop(state, &program);
    state.is_running = false;
    result
}

// ops/program.rs run_loop match arms (new):
Op::Stop => break,
Op::GtoInd(reg) => {
    let pointer = state.regs.get(reg as usize).ok_or(HpError::InvalidOp)?.clone();
    let int_part = pointer.trunc_int();
    if int_part != pointer { return Err(HpError::InvalidOp); }
    let label_str = int_part.inner().to_string();
    let target = find_in_program(program, &label_str)?;
    state.pc = target + 1;
}
Op::XeqInd(reg) => {
    if state.call_stack.len() >= 4 { return Err(HpError::CallDepth); }
    let pointer = state.regs.get(reg as usize).ok_or(HpError::InvalidOp)?.clone();
    let int_part = pointer.trunc_int();
    if int_part != pointer { return Err(HpError::InvalidOp); }
    let label_str = int_part.inner().to_string();
    let target = find_in_program(program, &label_str)?;
    state.call_stack.push(state.pc);
    state.pc = target + 1;
}

// ops/program.rs execute_op arm (new):
Op::Pse => {
    let formatted = crate::format::format_hpnum(&state.stack.x, state.display_mode);
    state.display_override = Some(formatted);
    state.event_buffer.push("PAUSE 1000".to_string());
    crate::stack::apply_lift_effect(state, crate::stack::LiftEffect::Neutral);
    Ok(())
}

// New op file recommended for Phase 22 grouping: hp41-core/src/ops/program_edit.rs
// (or inline in program.rs — planner picks).
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
    if state.pc > state.program.len() { state.pc = state.program.len(); }
    crate::stack::apply_lift_effect(state, crate::stack::LiftEffect::Neutral);
    Ok(())
}
// (Sketches for op_del, op_ins, op_size, op_cla, op_clst, op_pack, op_catalog,
// op_asn follow the same shape — planner fills in.)
```

</code_context>

<specifics>
## Specific Ideas

- **STOP and PSE work different break-patterns.** STOP = break run_loop
  (yields to user). PSE = no break, fires display_override + event_buffer
  marker, run_loop continues. The asymmetry is the entire point — STOP
  is a user-controlled rendezvous, PSE is a timed-display continuation.
- **The v2.1 `run_stop` Tauri command currently toggles is_running
  cosmetically — Phase 22 supersedes this.** Phase 22 lands
  `resume_program()` in core; Phase 25/26 will wire R/S key handling
  to call it (CLI keyboard + Tauri command). For Phase 22 itself, no
  CLI/GUI changes are needed — `resume_program()` just needs to exist
  and pass its own tests.
- **The `Op::Cla` + `Op::AlphaClear` duplication is intentional and
  documented** (D-22.13). Future agents may be tempted to delete
  `Op::AlphaClear` because `Op::Cla` does the same thing — DON'T.
  AlphaClear is in v1.0 save files; removing it breaks load. Phase 25
  CLI integration will choose which one the keyboard's "clear alpha"
  binding emits — likely Cla (hardware-faithful program listing).
- **The bounds-audit prep task (D-22.11.1) is a Wave-0 commit on its
  own**, before any Op::Size code lands. Pattern: replace every
  `state.regs[i]` or `state.regs[i as usize]` with
  `state.regs.get(i as usize).ok_or(HpError::InvalidOp)?`. Both reads
  and writes (the `state.regs[i] = HpNum::zero()` style in op_sto needs
  `if let Some(slot) = state.regs.get_mut(i as usize) { *slot = ...; }`
  or the indexed write with a prior bounds check).
- **CATALOG 1 skipping zero registers is a deliberate UX choice** —
  fresh state would otherwise dump 100 lines of `R{:02}  0.0000` into
  print_buffer. Frontends are free to display "no non-zero registers"
  if catalog body is empty (between header and footer).
- **Op::Stop and Op::GtoInd/XeqInd in execute_op MUST be in the
  programming-ops catch-all** — same compile-time exhaustive-match
  guard that protects against the "ran into a programming op outside
  run_loop" class of bugs. Op::Pse is the exception: it executes in
  both run_loop AND execute_op without break — same status as Op::PRX
  (executes everywhere).
- **The Op::Asn struct-variant `{ name, key_code }` is Phase 22's only
  struct-variant beyond Phase 21's FlagTest.** Save-file shape:
  `{"Asn": {"name": "SIN", "key_code": 11}}`. Planner should add at
  least one round-trip test confirming `assignments` survives a JSON
  save/load (FN-KEY-01 SC#5 wording).
- **No new HpError variants** — the entire phase reuses InvalidOp /
  CallDepth. This keeps the frontend error-display match exhaustive
  without churn.

</specifics>

<deferred>
## Deferred Ideas

- **Stop-from-keyboard while program is running** — the "stop next
  iteration" sentinel that the v2.1 `run_stop` Tauri command currently
  fakes via cosmetic toggle. Real implementation requires a way for
  the keyboard layer to signal run_loop to break. Two options:
  (a) cooperative — run_loop checks `state.stop_requested: bool` at
  the top of each iteration; (b) channel-based — but core is
  single-threaded, so the cooperative bool wins. Defer to Phase 25
  (CLI integration) where R/S key handling gets wired end-to-end.
  Phase 22 lands the foundation (`resume_program` + Op::Stop) only.
- **Real END / .END. marker** — would close the CLP boundary divergence
  cleanly (D-22.7) AND match HP-41 hardware program structure more
  faithfully. Adds: new Op::End variant; prgm_display update; CLP
  delete-to-End semantics; save-file migration. Probably belongs to
  a future Phase 22.5 or v3.0. Backlog candidate.
- **`pack` actually doing something** — if we ever introduce gaps in
  the program Vec (e.g., for in-place editing optimizations), PACK
  becomes meaningful. Currently moot. Backlog candidate.
- **CATALOG paging UX** — real HP-41 advances one entry per R/S press.
  Our bulk-print model differs. Could be revisited in Phase 26 GUI
  polish (the GUI could show a single line at a time with a paging
  UI overlaid on print_buffer drain).
- **AsnTarget enum** (D-22.18 deferred-alternative) — replacing the
  plain String with `AsnTarget { Op(Op), Label(String) }` for stronger
  static typing. Save-file complexity and migration cost make it not
  worth Phase 22 effort; revisit in v3.0 if dispatch-time resolution
  proves error-prone.
- **Reconciling `assignments` (u8-keyed) with `key_assignments`
  (char-keyed)** — Phase 25/26 work. Goal: one canonical hardware-
  faithful u8-keyed map; key_assignments becomes either a derived
  view or is migrated and removed. Out of scope for Phase 22.
- **SIZE memory model fidelity** — real HP-41 shares program and
  register memory; SIZE allocates between them. Our model has
  independent storage. Could be revisited in v3.0 Module Emulation
  milestone where memory-constrained programs need accurate behavior.

</deferred>

---

*Phase: 22-Program-Control-and-Memory-Ops*
*Context gathered: 2026-05-14*
