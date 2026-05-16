# Phase 22: Program Control & Memory Ops — Research

**Researched:** 2026-05-14
**Domain:** HP-41CV ROM control-flow + memory-management ops (`STOP`, `PSE`, `GTO IND`, `XEQ IND`, `CLP`, `DEL`, `INS`, `SIZE`, `CLA`, `CLST`, `PACK`, `CATALOG`, `ASN`)
**Confidence:** HIGH (CONTEXT.md has 25 locked decisions with line-precise code references; this research verifies each against the actual source and HP-41 hardware spec)

---

## Summary

Phase 22 lands 14 HP-41CV ROM ops in `hp41-core` only — keyboard wiring is Phase 25/26.
CONTEXT.md has already settled the design completely (25 locked decisions, concrete code
sketches keyed to specific line ranges). The research below verifies each locked decision
against the actual source code in `hp41-core/src/ops/` and against the HP-41 Owner's Manual
where hardware fidelity is at stake.

**Primary recommendation:** Execute the four plans (22-01..22-04) exactly as D-22.20 sequences
them. The biggest implementation risk is the `regs[]` bounds audit (D-22.11.1) — there are
**60+ raw indexing sites** in `hp41-core/src/` (most in `stats.rs` and `program.rs::op_isg`/
`op_dse`), and shrinking SIZE below 7 must NOT break the `Σ+` registers R01–R06 contract
without an explicit error. Two locked decisions also diverge mildly from HP-41 hardware
in ways the planner should record explicitly: **SIZE lower bound** (D-22.11 says 1, hardware
allows 0) and **CLP boundary** (D-22.7 says next-LBL, hardware uses END/.END.).

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FN-PROG-01 | `STOP` halts running program; next R/S resumes from next step | D-22.1, D-22.2, D-22.3 — Op::Stop break pattern verified against `Op::Prompt` arm at `program.rs:272–275`. `resume_program()` design verified against `run_program()` at `program.rs:139–169`. |
| FN-PROG-02 | `PSE` briefly displays X (~1s) then continues | D-22.4 — verified against `Op::Beep`/`Op::Tone` pattern in `sound.rs:13–17`; PSE writes both `display_override` and `event_buffer` from `execute_op`; HP-41 hardware pause confirmed ≈ 1 second [HP Forum / HPmuseum]. |
| FN-PROG-03 | `CLP "name"` clears program from LBL to next boundary | D-22.7 — divergence flagged: hardware uses END/.END., we use next-LBL (no END marker in flat-Vec model). |
| FN-PROG-04 | `DEL nnn` removes N steps from PC | D-22.9 — clamping logic verified safe; `Vec::drain` semantics match. |
| FN-PROG-05 | `INS` inserts blank step at PC | D-22.8 — `Op::Null` reuse confirmed at `program.rs:413–416` (no-op + Neutral lift). |
| FN-PROG-06 | `GTO IND nn` branches via register | D-22.15 — `trunc_int()` exists at `num.rs:224–226`; `find_in_program()` exists at `program.rs:537–541`; HP-41 hardware "non-integer pointer → InvalidOp" matches FN-IND-02. |
| FN-PROG-07 | `XEQ IND nn` calls subroutine via register | D-22.15 — call_stack 4-deep check verified at `program.rs:206–207`. |
| FN-MEM-01 | `SIZE nnn` sets register count | D-22.11 — `Vec::resize` semantics; HP-41 hardware allows `SIZE 000` (research divergence flagged below). |
| FN-MEM-02 | `CLA` clears ALPHA | D-22.13 — wraps `op_alpha_clear()` at `alpha.rs:34–38`. Coexists with v1.0 `Op::AlphaClear`. |
| FN-MEM-03 | `CLST` zeros X/Y/Z/T, preserves LASTX + lift | D-22.14 — pattern reference: `op_clreg()` at `registers.rs:98–102`. |
| FN-MEM-04 | `PACK` (no-op in flat-Vec model) | D-22.12 — divergence already documented; just `apply_lift_effect(Neutral) + Ok(())`. |
| FN-MEM-05 | `CATALOG n` (n=1..4) emits structured listing into `print_buffer` | D-22.16, D-22.16.1, D-22.16.2, D-22.16.3 — extends existing `print_buffer` (24-char width per `print.rs:14`). |
| FN-KEY-01 | `ASN "name" key_code` records reversible key assignment | D-22.17, D-22.18, D-22.19 — new `assignments: BTreeMap<u8, String>` field on `CalcState` with `#[serde(default)]`; key_code encoding row×10+col (1-indexed) matches `last_key_code` / `keycode_to_hp41_code()` precedent. |
</phase_requirements>

---

## 1. Hardware Fidelity Cross-Check

Per locked decision in CONTEXT.md, each verified against HP-41 hardware spec / current code.
Items where the locked decision diverges from HP-41 hardware are flagged for planner awareness.

| Decision | Hardware (verified) | Locked Choice | Verdict |
|----------|---------------------|---------------|---------|
| D-22.1 STOP break | HP-41 STOP halts at the step AFTER STOP; R/S resumes at the next step. [CITED: HP-41C Owner's Manual ch. 9] | `break` after pc has already been advanced by `run_loop` (line 189) — pc points at next step. | **MATCHES.** No divergence. |
| D-22.2 `resume_program()` | HP-41 R/S re-enters `run_loop` from where STOP halted. | `state.pc` is the entry point; re-enter `run_loop` from there. | **MATCHES.** |
| D-22.4 PSE ≈ 1s | HP-41 hardware PSE displays X for ~1 second then continues. [CITED: HPmuseum / HP-41 forum] | `event_buffer.push("PAUSE 1000")` + `display_override = format_hpnum(X)`. | **MATCHES.** The 1000 (ms) is appropriate; frontend handles the sleep. |
| D-22.7 CLP boundary | HP-41 hardware uses END / .END. markers — CLP deletes from LBL to the next END. | Delete from LBL to next LBL (no END markers in flat-Vec model). | **DIVERGENCE — already documented in D-22.7 as intentional.** Planner should retain this divergence note in the implementation comment and add a test that exercises a "last labelled block" case (delete-to-end-of-Vec). Backlog candidate Op::End is in `<deferred>`. |
| D-22.9 DEL clamping | HP-41 hardware DEL silently clamps when nnn > remaining steps. | `min(nnn, program.len() - state.pc)`; nnn==0 OR pc==len → no-op. | **MATCHES.** |
| D-22.11 SIZE range | HP-41 hardware accepts `SIZE 000` — 0 data registers, all memory to program. [VERIFIED: HPmuseum SIZE 000 reference] `SIZE 319` is the max. | `nnn.clamp(1, 319)` — clamps 0 to 1. | **MILD DIVERGENCE.** Hardware allows SIZE 000 (no data registers); D-22.11 mandates min 1. Planner: this is defensible (forces at least one data register so `Σ+` registers and indirect addressing don't catastrophically fail), but it should be **documented** in code comment + integration test must assert `Op::Size(0)` returns `Ok` with regs.len() == 1, not `InvalidOp`. Recommend planner clarifies: "0 silently clamps to 1 (no error)" — current wording "Anything outside [1, 319] returns InvalidOp" would reject SIZE 0 with error, which differs from hardware (hardware accepts SIZE 0 and just allocates 0 data regs). **Open question for planner**: clamp-silently OR reject-with-InvalidOp? Hardware says clamp-silently or allow zero; D-22.11 reads "Anything outside [1, 319]" as reject. Resolve before implementing. |
| D-22.11 SIZE shrink | HP-41 hardware MEM LOST — shrinking from 319 to 50 discards data in R50..R318. | `Vec::resize(.., HpNum::zero())` truncates tail. | **MATCHES** the silent-truncate behavior. Note: real hardware actually triggers the MEM LOST display on shrink loss, but that's a UI concern (Phase 25/26). |
| D-22.12 PACK | HP-41 PACK compacts program memory by removing gaps. | No-op + Neutral lift. | **DOCUMENTED DIVERGENCE.** Already in `<deferred>` as future-Vec-gaps work. |
| D-22.13 CLA | HP-41 CLA = CLRALPHA (identical behavior). | New `Op::Cla` variant wrapping `op_alpha_clear`. Coexists with `Op::AlphaClear`. | **MATCHES.** Display divergence is documented (`"CLA"` vs `"CLRALPHA"`). |
| D-22.14 CLST | HP-41 hardware CLST: X=Y=Z=T=0; LASTX preserved; lift_enabled UNTOUCHED. [CITED: HP-41 Owner's Manual ch. 7] | Same. | **MATCHES.** Critical: `Op::Clreg` clears `regs` and applies Neutral. `Op::Clst` clears stack but DOES NOT touch LASTX or lift_enabled. Planner must add explicit test that asserts both preservations. |
| D-22.15 GTO/XEQ IND | HP-41 hardware: indirect resolution reads integer part of `R[n]`; non-integer rejected. | `trunc_int()` + equality check, then `find_in_program()` lookup. | **MATCHES.** FN-IND-02 wording confirms hardware reject-non-integer. The locked check `x == x.trunc_int()` is the exactly-correct HP-41 hardware semantic. |
| D-22.16 CATALOG range | HP-41 hardware: `CATALOG 1` (programs in user memory), `CATALOG 2` (XROM in extension), `CATALOG 3` (HP-IL), `CATALOG 4` (peripherals — never shipped on basic 41CV). | D-22.16: `n ∈ {1,2,3,4}`; D-22.16.1 says CATALOG 1 = registers, D-22.16.2 says CATALOG 2 = programs. | **WARNING — SEMANTIC DIVERGENCE.** Real HP-41: CATALOG 1 = **PROGRAMS in main memory**, CATALOG 2 = **functions in plug-in modules (XROM)**, CATALOG 3 = HP-IL devices, CATALOG 4 = peripherals. D-22.16.1/2 inverts this: CATALOG 1 = registers, CATALOG 2 = programs. **Planner must confirm with user**. Recommendation: swap to hardware-faithful — CATALOG 1 = programs, CATALOG 2 = XROM (return "NOT AVAILABLE" since no modules in v2.x), CATALOG 3/4 = peripherals (also "NOT AVAILABLE"). The "registers" listing is non-standard on real HP-41 (it's the `→PG` and `→RG` synthetic functions, not CATALOG). [CITED: HP-41 Owner's Manual ch. 4 + HPmuseum CATALOG reference] |
| D-22.17 ASN key code | HP-41 hardware key code = row×10 + col, 1-indexed (positive = main fn, negative = shifted fn). [CITED: HP-41C Owner's Manual app. F] | Same row×10+col 1-indexed (matches `last_key_code` and `keycode_to_hp41_code()`). | **MATCHES** for the positive (main-fn) case. **GAP**: real HP-41 ASN distinguishes shifted vs unshifted via sign — negative key_code means "shifted variant". `u8` cannot encode this. Planner option A: accept the v2.2 simplification (u8, unshifted only — frontend can encode shifted as 0x80+code or use a separate map); option B: widen to `i8` and follow hardware. **Recommend deferring this nuance to Phase 25/26** when the keyboard modal lands; v2.2 ships `u8` and Phase 26 documents the shifted-encoding when it's actually needed. |
| D-22.19 USER-mode resolution | HP-41 hardware: ASN target is stored as a 2-byte ROM function code OR a label string; lookup is hardware-encoded. | Late-binding via String: parse-as-Op fallback to LBL search. | **PRAGMATIC DIVERGENCE.** Hardware uses byte codes; we use strings. Already documented in D-22.19. The cost is: stale assignments (label deleted) surface as **runtime errors** during USER-mode dispatch — not at save/load. Acceptable per CONTEXT.md. |

**Summary of divergences requiring planner confirmation:**

1. **CATALOG 1 vs 2 semantics** (D-22.16.1, D-22.16.2) — strongly recommend swap to hardware-faithful (CATALOG 1 = programs, CATALOG 2 = XROM "NOT AVAILABLE"). This is the only divergence that materially affects user-facing behavior. **OPEN QUESTION for planner — see §6.**
2. **SIZE 000 acceptance** (D-22.11) — reject or clamp-to-1? D-22.11 wording ambiguous between "InvalidOp" and "silently clamp" semantics. **OPEN QUESTION for planner — see §6.**
3. **ASN shifted-key encoding** (D-22.17) — u8 only covers unshifted keys; sign-bit encoding deferred to Phase 25/26. Acceptable.

---

## 2. Implementation Pitfalls

Gathered from reading the actual source files referenced in CONTEXT.md canonical refs.

### Pitfall 1: `Op::Stop` arm placement vs `Op::Prompt`

**What goes wrong:** `Op::Prompt` (Phase 21, `program.rs:272–275`) writes
`state.display_override = Some(alpha)` BEFORE breaking. The locked decision D-22.1 says
`Op::Stop` should do `break` only — NO display write. If a future maintainer copies the
PROMPT arm blindly, STOP would overwrite the display with ALPHA every time it halts.

**Why it happens:** The two arms look identical structurally but have opposite display
semantics — PROMPT shows ALPHA, STOP freezes whatever was on display.

**How to avoid:** In the implementation, put `Op::Stop` arm IMMEDIATELY ADJACENT to `Op::Prompt`
with a leading comment: `// Unlike Prompt, Stop writes nothing — the previous step's display
persists.` The verification test must assert `display_override` is unchanged after STOP runs
in a program where the prior step did NOT set an override.

**Warning signs:** A failing test like "after STOP, display shows ALPHA" means the
maintainer copy-pasted the PROMPT arm.

### Pitfall 2: `resume_program()` clone-then-reset pattern

**What goes wrong:** The error path of `run_loop` must reset `state.is_running = false`
even on `Err(...)` return. The existing `run_program()` (line 167) does this via a
let-result-then-reset pattern. If `resume_program()` uses `?` propagation naively
(`run_loop(state, &state.program.clone())?`), the `is_running = false` reset is skipped
on error and the next R/S press will see a stale `is_running = true`.

**Why it happens:** `?` is the natural Rust idiom but it short-circuits the function on
Err before the cleanup line runs.

**How to avoid:** Mirror `run_program()` exactly — store result, reset is_running, return
result:

```rust
state.is_running = true;
let program = state.program.clone();
let result = run_loop(state, &program);
state.is_running = false;
result
```

**Warning signs:** Test "after STOP-then-error-on-resume, `is_running == false`" fails.

### Pitfall 3: PSE display_override timing & dispatch-top clear

**What goes wrong:** Phase 21's dispatch-top clear (Pitfall 5, `ops/mod.rs:410`) zeroes
`state.display_override = None` at the top of EVERY dispatch call. PSE writes
`display_override` from inside `execute_op` (NOT from `dispatch`) — so the override
survives the current run_loop iteration. The frontend reads the event_buffer "PAUSE 1000"
signal, sleeps 1 second, then triggers the NEXT dispatch (or the next run_loop iteration
inside the same run_program call). What happens on that next iteration?

**Answer (verified by re-reading `run_loop`):** `run_loop` (line 177–284) calls
`execute_op()` directly, NOT `dispatch()`. So `display_override` is NOT cleared between
run_loop iterations. PSE's display_override stays set until the NEXT TIME `dispatch()`
runs (i.e., the next interactive keystroke after the program exits). This is correct for
PSE-in-program behavior: the X value stays displayed for ~1 second (frontend timing) AND
remains on display while subsequent in-program steps run silently. **However**, after the
program exits, the next user keystroke clears it via dispatch-top — which is HP-41 hardware
behavior (next keypress dismisses the displayed value).

**How to avoid:** Verify this with a 3-step integration test:
- `PSE in program` → check display_override is `Some(format_hpnum(X))` immediately after
- One more program step runs → display_override still set
- Program exits, user dispatches `Op::Add` → display_override cleared

**Warning signs:** "PSE display shows for only 1 instruction" or "PSE display persists after
keystroke" indicates the dispatch-top clear is mis-placed.

### Pitfall 4: `regs[i]` panic risk under SIZE shrink

**What goes wrong:** After `Op::Size(50)`, `state.regs.len() == 50`. Now if any existing
op reaches `state.regs[75]` (raw indexing), it **PANICS** rather than returning `InvalidOp`.
The `#![deny(clippy::unwrap_used)]` lint does NOT catch raw `Vec` indexing — it catches
`.unwrap()` only. Raw `Vec[i]` indexing produces a panic at runtime, NOT a compile error.

This is the most dangerous single risk in Phase 22. Locking SIZE without the
D-22.11.1 audit would introduce panics that violate the zero-panic invariant.

**Verified count:** 60+ raw indexing sites in `hp41-core/src/` (full audit below in §3).

**How to avoid:** Run Wave-0 audit BEFORE Op::Size goes through dispatch. The audit must
land as a separate commit per D-22.11.1 + planner discretion note.

**Verification:** add an integration test that does `Op::Size(5)` then `Op::StoReg(50)` and
asserts the result is `Err(HpError::InvalidOp)` and (critically) **does not panic**. Use
`std::panic::catch_unwind` or simply verify by running — if it panics, the test harness
reports it as a panic, not a normal failure.

### Pitfall 5: Σ+ register interaction with SIZE shrink

**What goes wrong:** `op_sigma_plus()` (`stats.rs:27–40`) writes to `state.regs[1..=6]`
unconditionally. After `Op::Size(5)`, `regs[6]` panics. Worse: after `Op::Size(3)`, all of
R03..R06 are gone but `Σ+` still tries to write to them. Even with the bounds-audit fix
(return `InvalidOp`), this means `Σ+` silently fails on SIZE-shrunk states. Real HP-41:
`Σ+` honors the SIZE setting — if the Σ block (R01–R06 by default, configurable via
system flags 26–28) is out of range, `Σ+` returns NONEXISTENT.

**How to avoid:** The bounds audit MUST replace `state.regs[1].checked_add(...)` with
`state.regs.get(1).ok_or(InvalidOp)?.checked_add(...)`. After the audit, `Σ+` on a
SIZE-3 state returns `Err(HpError::InvalidOp)` — matches the "NONEXISTENT" hardware
behavior in spirit if not in exact error wording.

**Note:** This is NOT a bug to fix in Phase 22; it's a consequence of D-22.11.1. The
planner should add a test under 22-03 (memory-ops) that asserts the safety: `Op::Size(3)
→ Σ+ → expects InvalidOp` (not panic).

### Pitfall 6: `CLP` PC adjustment after drain

**What goes wrong:** `Vec::drain(start..end)` shifts indices. If `state.pc` was 50 and
we drain `program[20..30]`, `state.pc` should now be 40, not 50. The sketch in CONTEXT.md
lines 614–627 says `if state.pc > state.program.len() { state.pc = state.program.len() }`.
That handles the "pc fell off the end" case but NOT the "pc was past the drain region but
needs to shift down" case.

**Verified:** Re-reading the sketch:
```rust
let start = ... position of target LBL ...;
let end = ... position of next LBL (or program.len()) ...;
state.program.drain(start..end);
if state.pc > state.program.len() { state.pc = state.program.len(); }
```

**Trap:** This is fine ONLY because CLP is gated on `prgm_mode == true` AND `is_running ==
false` (D-22.10). In PRGM mode, `state.pc` is the cursor position for further insertions — and
the planner should decide: does CLP move the cursor to `start` (where the deleted block
used to be), or leave it where it was (potentially pointing to nothing meaningful)?
**HP-41 hardware:** CLP repositions the cursor to the first step of the block that WAS
deleted, which after deletion is the step that WAS the next LBL (or end-of-program).

**Recommendation:** After drain, set `state.pc = start` so the cursor lands at the start
of whatever used to be the deleted block. Add this to the implementation sketch.

**Warning signs:** Test "after CLP, pc points to a sensible step" fails.

### Pitfall 7: Programming-ops catch-all completeness

**What goes wrong:** `execute_op()`'s catch-all at `program.rs:454–464` currently lists 10
arms. Phase 22 adds 6 new ops that must join it (Op::Stop, Op::Clp(_), Op::Del(_), Op::Ins,
Op::GtoInd(_), Op::XeqInd(_)). If the planner forgets ANY of them, that variant will
either compile-fail (Op::Pse won't — it's listed below) or worse, fall through into the
catch-all's `InvalidOp` and silently work as "InvalidOp inside programs", which is correct
for Stop/Clp/Del/Ins (which shouldn't run in programs) but **wrong for GtoInd/XeqInd**
(which MUST be handled by run_loop directly).

**Verified path:**
- Op::Stop, Op::Clp(_), Op::Del(_), Op::Ins → catch-all (returns InvalidOp inside
  execute_op, which is correct because they only execute via the run_loop arms or
  interactive dispatch).
- Op::GtoInd(_), Op::XeqInd(_) → catch-all (returns InvalidOp inside execute_op, BUT
  they MUST have explicit `run_loop` arms; otherwise `run_loop` calls `execute_op` which
  returns InvalidOp and aborts the program).

**How to avoid:** Wave-1 task for plan 22-01 must verify both:
1. The exhaustive-match compiler check passes (all 6 new variants listed).
2. `run_loop`'s match has explicit arms for `Op::GtoInd` and `Op::XeqInd` BEFORE the
   `other =>` catch-all (line 276 in current source).

### Pitfall 8: Op::Cla / Op::AlphaClear duplication trap

**What goes wrong:** Maintainer sees two ops that do the same thing and "cleans up" by
removing `Op::AlphaClear`. v1.0 save files contain `Op::AlphaClear` (it's been in the enum
since Phase 2) — removing it breaks load.

**How to avoid:** Add a doc-comment on BOTH variants pointing at the other one and the
hardware reasoning (`Op::Cla` is hardware-faithful display name; `Op::AlphaClear` is the
v1.0 name kept for save-file compat). Add a save-file round-trip test under 22-03 that
loads a v1.x fixture containing `Op::AlphaClear` and asserts the program still runs.

### Pitfall 9: ASN struct-variant JSON shape

**What goes wrong:** `Op::Asn { name: String, key_code: u8 }` serializes as a JSON object,
not a tagged variant. Serde's default for struct variants produces `{"Asn": {"name": "SIN",
"key_code": 11}}`. If a future migration wants to switch to `AsnTarget` enum (D-22 deferred
list), the JSON shape changes. Plan a forward-compat test (round-trip a `Op::Asn` through
JSON and assert exact shape) so any schema drift surfaces immediately.

### Pitfall 10: `flush_entry_buf` & PSE interaction

**What goes wrong:** `dispatch()` calls `flush_entry_buf` first (`ops/mod.rs:405`). If
the user types `1.23` then hits PSE interactively, the entry buffer is flushed (1.23 lifts
onto X) BEFORE PSE runs. PSE then writes `format_hpnum(X)` = `1.2300` (FIX 4) to
display_override. **This is correct behavior** — confirms PSE matches HP-41 hardware.

**However**: inside run_loop, `execute_op` does NOT call `flush_entry_buf` (per the
comment at `program.rs:290–291`). So `Op::Pse` inside a program reads `state.stack.x`
which was set by a prior `Op::PushNum` (which the run_loop loop puts directly on the
stack via `enter_number`, no buffer involved). Same X value either way; no divergence.

**Verification:** Add an integration test that does `digit 1.23 → PSE` (interactive) and
asserts `display_override == "1.2300"`. This will fail if a maintainer adds a redundant
flush at PSE entry and accidentally clears the entry buffer twice.

---

## 3. Wave-0 Bounds Audit Scope (D-22.11.1)

Concrete list of every `state.regs[i]` raw index in `hp41-core/src/`. Every one of these
must be replaced with a `.get()`/`.get_mut()` pattern OR have a bounds check in front of
it that uses `state.regs.len()` (not the hardcoded `100`).

**Production sites (28 — must be fixed):**

| File:Line | Access Pattern | Current Bound | Fix Pattern |
|-----------|----------------|---------------|-------------|
| `ops/registers.rs:19` | `state.regs[reg as usize] = state.stack.x.clone()` (write, `op_sto`) | `reg >= 100` at line 16 | Replace with `if (reg as usize) >= state.regs.len() { Err(InvalidOp) }` + index write |
| `ops/registers.rs:30` | `state.regs[reg as usize].clone()` (read, `op_rcl`) | `reg >= 100` at line 27 | Same as above |
| `ops/registers.rs:50–53` | `state.regs[reg as usize].checked_add/sub/mul/div(...)` (read, `op_sto_arith`) | `reg >= 100` at line 45 | Same |
| `ops/registers.rs:56` | `state.regs[reg as usize] = new_val` (write, `op_sto_arith`) | (same guard above) | Same |
| `ops/registers.rs:99` | `state.regs = vec![HpNum::zero(); 100]` (re-init, `op_clreg`) | (none — total replacement) | **Change to** `let n = state.regs.len(); state.regs = vec![HpNum::zero(); n];` so CLREG respects the current SIZE. |
| `ops/display_ops.rs:20` | `state.regs[reg as usize].clone()` (read, `op_view`) | `reg >= 100` at line 17 | Same as op_rcl |
| `ops/program.rs:110` | `parse_counter(&state.regs[reg as usize])` (read, `op_isg`) | `reg as usize >= state.regs.len()` at line 107 (already safe!) | **Already len-correct — no change needed.** Just convert to `.get()` for idiom consistency. |
| `ops/program.rs:112` | `state.regs[reg as usize] = build_counter(...)` (write, `op_isg`) | (same guard) | Same — already len-correct. |
| `ops/program.rs:123` | `parse_counter(&state.regs[reg as usize])` (read, `op_dse`) | `reg as usize >= state.regs.len()` at line 120 (already safe) | Same — already len-correct. |
| `ops/program.rs:125` | `state.regs[reg as usize] = build_counter(...)` (write, `op_dse`) | (same guard) | Same — already len-correct. |
| `ops/stats.rs:27–32` | 6 reads from `state.regs[1..=6]` (`op_sigma_plus`) | (none!) | **Add bounds check at function entry**: `if state.regs.len() < 7 { return Err(InvalidOp); }` then proceed (existing indexing is safe under that guard). |
| `ops/stats.rs:35–40` | 6 writes to `state.regs[1..=6]` (`op_sigma_plus`) | (same — depends on entry guard) | Same |
| `ops/stats.rs:57–62` | 6 reads (`op_sigma_minus`) | (none) | Add `if state.regs.len() < 7 { Err(InvalidOp) }` |
| `ops/stats.rs:64–69` | 6 writes (`op_sigma_minus`) | (same) | Same |
| `ops/stats.rs:80–85` | 3 reads R1, R2, R3, R5 (`op_mean`) | (none) | Add `if state.regs.len() < 6 { Err(InvalidOp) }` (or 7 to be conservative) |
| `ops/stats.rs:102–117` | reads from R1, R2, R3, R4, R5 (`op_sdev`) | (none) | Same — add entry guard |
| `ops/stats.rs:135–143` | reads from R1, R2, R3, R5, R6 (`op_lr`) | (none) | Same — add entry guard |
| `ops/stats.rs:178–187` | reads R1, R2, R3, R5, R6 (`op_yhat`) | (none) | Same — add entry guard |
| `ops/stats.rs:212–221` | reads R1, R2, R3, R4, R5, R6 (`op_corr`) | (none) | Same — add entry guard |
| `ops/stats.rs:242` | `state.regs[i] = HpNum::zero()` for i in 1..=6 (`op_cl_sigma_stat`) | (none) | Add entry guard `if state.regs.len() < 7 { Err(InvalidOp) }` OR clamp loop to `min(7, regs.len())` (cleaner: skip the zeroing if regs is shorter, since nothing was there to clear). Recommend explicit guard. |

**Test-only sites (not in audit scope but useful to note):**
- `hp41-core/src/cardreader/mod.rs:130, 131, 135, 136, 154, 155, 157, 174, 192` — all in `#[cfg(test)]`
- `hp41-core/src/tests.rs:1063, 1083` — `#[cfg(test)]`
- `hp41-core/src/ops/program.rs:726` — inside `#[cfg(test)]` test module
- `hp41-core/tests/phase21_display.rs:41, 57` and similar across `tests/*.rs` — integration tests

These remain `state.regs[i]` (raw indexing under `#[allow(clippy::unwrap_used)]`) — they
are pre-conditioned on the test setup providing the right SIZE.

**Implementation plan recommendation:**
1. **One commit** does the bounds-audit for `op_sto`, `op_rcl`, `op_sto_arith`, `op_view`, `op_clreg` — these all share the `if reg >= 100` pattern that becomes `if reg as usize >= state.regs.len()`.
2. **Second commit** adds the `Σ+`/`Σ-`/MEAN/SDEV/LR/YHAT/CORR/`ClSigmaStat` entry guards (one-line addition per function).
3. **Third commit** (or part of the second) updates `op_clreg` to respect current `regs.len()` instead of hardcoding 100.
4. Then `Op::Size` lands; tests assert that `Op::Size(5)` followed by `op_sto(state, 50)` returns `InvalidOp` not panic.

**Why a separate commit:** Per D-22.11.1 + Claude's-discretion note, git-blame clarity — if a regression appears, `git blame` points at the audit commit cleanly.

---

## 4. Test Architecture Recommendations

**Precedent analysis (Phase 21):**

| Plan | Where unit tests live | Where integration tests live |
|------|------------------------|-------------------------------|
| 21-01 (flags-core) | inline `#[cfg(test)]` in `flags.rs` (28 lines) | `tests/phase21_flags.rs` (multi-test integration) |
| 21-02 (conditional-skip) | inline in `program.rs` + `flags.rs` | extends `tests/phase21_flags.rs` |
| 21-03 (display-control) | inline in `display_ops.rs` (~50 lines) | `tests/phase21_display.rs` |
| 21-04 (sound) | inline in `sound.rs` (~20 lines) | `tests/phase21_sound.rs` |

**Pattern:** One integration test file per plan, named `phase21_<plan-slug>.rs`.
Inline `#[cfg(test)]` modules carry the per-function unit tests (small, fast, easy to
locate). Integration tests cover serde-round-trip, dispatch path, run_loop interaction,
v20-autosave.json backward-compat, and cross-op interaction.

**Recommended layout for Phase 22:**

| Plan | New module file(s) | Inline tests | Integration test file |
|------|---------------------|---------------|------------------------|
| 22-01 program-control | none (Stop / GtoInd / XeqInd in `program.rs` run_loop arms; Pse in `program.rs` execute_op arm); add `resume_program()` next to `run_program()` | inline tests in `program.rs` for resume_program & individual ops (~30 lines) | `tests/phase22_program_control.rs` — STOP-then-resume round-trip, PSE display + event_buffer roundtrip, GTO IND/XEQ IND happy + non-integer rejection + 4-deep callstack |
| 22-02 program-edit | optional new `ops/program_edit.rs` (or inline in `program.rs`) — see sketch in CONTEXT.md lines 612–627 | inline tests in `program_edit.rs` for `op_clp`/`op_del`/`op_ins` (small unit tests with hand-built `program: Vec<Op>`) | `tests/phase22_program_edit.rs` — interactive PRGM-mode CLP boundary, DEL clamping (incl. nnn=0, pc=len), INS placeholder, prgm_mode = false rejection |
| 22-03 memory-ops | optional new `ops/memory.rs` for Op::Size/Cla/Clst/Pack | inline unit tests | `tests/phase22_memory_ops.rs` — SIZE up/down, CLA equivalence with CLRALPHA, CLST preserves LASTX + lift, PACK no-op + lift, **Σ+ on SIZE<7 → InvalidOp** (Pitfall 5) |
| 22-04 catalog-and-asn | new `ops/catalog.rs` + `ops/asn.rs` (or grouped in one file) | inline | `tests/phase22_catalog.rs` + `tests/phase22_asn.rs` — CATALOG output format (24-char), skip-zero-regs (CATALOG 1), program listing (CATALOG 2), "NOT AVAILABLE" (CATALOG 3/4), ASN insert + remove (via re-ASN with empty string? — clarify with planner), **save-file round-trip with assignments populated** |

**Bounds-audit testing** (D-22.11.1) — separate commit, gets a small set of tests:
- `op_sto` returns `InvalidOp` for `reg >= regs.len()` (not just `reg >= 100`)
- `op_rcl` same
- `op_sto_arith` same
- `op_view` same
- `op_clreg` resizes correctly when SIZE has been changed (uses `regs.len()` not 100)
- `Σ+` on SIZE-3 returns `InvalidOp` (sentinel test for Pitfall 5)

Live in `tests/phase22_memory_ops.rs` (same file as SIZE tests, grouped naturally).

**Save-file fixtures:**
- `tests/fixtures/v20-autosave.json` (existing) — for backward-compat
- Recommend NOT creating a `v22-autosave.json` fixture in this phase — defer to Phase 27
  test hardening since it adds a fragile dependency that breaks if any subsequent phase
  modifies CalcState.

**Test count estimate:** ~40 new tests across the four plans (CONTEXT.md SC criteria + the
specific Pitfall sentinels above). Phase 21 was 48 tests; 40 is a reasonable target.

---

## 5. Validation Architecture (Nyquist Dimension 8)

For VALIDATION.md generation. `workflow.nyquist_validation` is `true` in `.planning/config.json`.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` (built-in) + `proptest 1.x` (property tests) — same as Phase 20/21 |
| Config file | `Cargo.toml` workspace + `hp41-core/tests/` integration suite |
| Quick run command | `just test-core` (≈ 3–5 s for inline unit tests only) |
| Full suite command | `just ci` (workspace tests + clippy + fmt, ≈ 35 s) |
| Coverage gate | `just coverage` ≥ 80 % on `hp41-core` (target ≥ 95 % by Phase 27) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FN-PROG-01 | STOP halts program; resume_program() continues from pc | integration | `cargo test --package hp41-core --test phase22_program_control test_stop_then_resume` | ❌ Wave-0 |
| FN-PROG-02 | PSE writes display_override AND event_buffer "PAUSE 1000" | integration | `cargo test --package hp41-core --test phase22_program_control test_pse_writes_both_channels` | ❌ Wave-0 |
| FN-PROG-03 | CLP "name" drains from LBL to next LBL or end-of-Vec | integration | `cargo test --package hp41-core --test phase22_program_edit test_clp_boundary` | ❌ Wave-0 |
| FN-PROG-04 | DEL nnn clamps when nnn > remaining; nnn=0 = no-op | integration | `cargo test --package hp41-core --test phase22_program_edit test_del_clamping` | ❌ Wave-0 |
| FN-PROG-05 | INS inserts Op::Null at state.pc; pc unchanged | integration | `cargo test --package hp41-core --test phase22_program_edit test_ins_inserts_null_at_pc` | ❌ Wave-0 |
| FN-PROG-06 | GTO IND nn — happy path + non-integer rejection + reg-out-of-range | integration | `cargo test --package hp41-core --test phase22_program_control test_gto_ind` | ❌ Wave-0 |
| FN-PROG-07 | XEQ IND nn — happy + 4-deep callstack + reg-out-of-range | integration | `cargo test --package hp41-core --test phase22_program_control test_xeq_ind` | ❌ Wave-0 |
| FN-MEM-01 | SIZE nnn resizes `state.regs`; out-of-range → InvalidOp; sub-SIZE Σ+ → InvalidOp | integration | `cargo test --package hp41-core --test phase22_memory_ops test_size` | ❌ Wave-0 |
| FN-MEM-02 | CLA clears alpha_reg (equivalent to CLRALPHA) | unit | `cargo test --package hp41-core --lib memory::tests::test_cla` | ❌ Wave-0 |
| FN-MEM-03 | CLST zeros X/Y/Z/T, preserves LASTX and lift_enabled | integration | `cargo test --package hp41-core --test phase22_memory_ops test_clst_preserves_lastx_and_lift` | ❌ Wave-0 |
| FN-MEM-04 | PACK = no-op + Neutral lift; returns Ok | unit | `cargo test --package hp41-core --lib memory::tests::test_pack` | ❌ Wave-0 |
| FN-MEM-05 | CATALOG 1/2/3/4 emits header + payload + footer to print_buffer | integration | `cargo test --package hp41-core --test phase22_catalog test_catalog_1234` | ❌ Wave-0 |
| FN-KEY-01 | ASN inserts into `state.assignments`; survives JSON save/load round-trip | integration | `cargo test --package hp41-core --test phase22_asn test_asn_roundtrip` | ❌ Wave-0 |

### Sampling Rate
- **Per task commit:** `just test-core` (Phase 22 unit + inline tests, < 5 s)
- **Per wave merge:** `just ci` (clippy + fmt + workspace test)
- **Phase gate:** `just ci` + `just coverage` green before `/gsd-verify-work`
- **Max feedback latency:** 12 seconds (well within the 5–7 s typical run)

### Wave-0 Gaps
- [ ] `hp41-core/tests/phase22_program_control.rs` — STOP-resume, PSE, GtoInd/XeqInd
- [ ] `hp41-core/tests/phase22_program_edit.rs` — CLP/DEL/INS
- [ ] `hp41-core/tests/phase22_memory_ops.rs` — SIZE / CLA / CLST / PACK + bounds-audit sentinels
- [ ] `hp41-core/tests/phase22_catalog.rs` — CATALOG 1/2/3/4 output
- [ ] `hp41-core/tests/phase22_asn.rs` — ASN round-trip + assignments serde-default-empty
- [ ] No framework installs needed — `cargo test`, `proptest`, `serde_json` already in dev-deps
- [ ] No new fixtures required — existing `v20-autosave.json` covers serde backward-compat for new `assignments` field (it lacks the field, so #[serde(default)] proves itself)

### Proptest Opportunities (defer to Phase 27, note here for planner)
- **STOP-resume idempotence:** for any valid program with a STOP at step k,
  `run_program → resume_program → resume_program → … → final state` is deterministic regardless
  of how many times we resume between STOPs. Not strictly needed for v2.2; Phase 27.
- **DEL clamping:** for any (pc, nnn) input, after DEL the post-state satisfies
  `program.len() == max(0, original_len - min(nnn, original_len - pc))`. Trivially valued; skip.
- **CATALOG empty body:** for `regs` filled with zero, CATALOG 1 emits only header + footer
  (zero payload lines). Trivially valued; skip.

### Manual-Only Verifications (none in Phase 22)
All Phase 22 behavior is hp41-core only. No TUI/GUI rendering involved. Phase 25/26 will
add manual verification for the keyboard modal flows.

---

## 6. Open Questions for Planner (RESOLVED 2026-05-14)

All four OQs below were surfaced for the user during /gsd-plan-phase 22 and resolved
in `22-CONTEXT.md` (see preamble lines 5–14 for the consolidated resolution summary).
The original text is preserved below for traceability; each entry now carries a
`**RESOLVED:**` marker.

### OQ-1: CATALOG 1 vs 2 semantics (D-22.16.1 / D-22.16.2) — RESOLVED: Option B

**RESOLVED 2026-05-14 (user-confirmed):** Option B — hardware-faithful. CAT 1 =
programs (LBL listing with step counts); CAT 2/3/4 = "NOT AVAILABLE". Register
listing dropped from Phase 22; may return as a separate non-ROM op in a future
quick-task or v3.0. D-22.16.* amended accordingly.

**Issue:** Real HP-41 hardware: `CATALOG 1` = programs in main memory; `CATALOG 2` =
functions in plug-in modules. D-22.16.1 inverts: CATALOG 1 = registers.

**Options:**
- **(A) Keep D-22.16.1/2 as-is** — registers in CAT 1, programs in CAT 2. Justification:
  registers listing is genuinely useful in an emulator without paper-tape printer; user-facing.
- **(B) Swap to hardware-faithful** — programs in CAT 1, "NOT AVAILABLE" in CAT 2 (no XROM),
  registers under a new op (e.g., `Op::CatalogRegs` or `Op::PrgmRg` synthetic). Justification:
  hardware fidelity is a core project value.

**Recommendation:** **Option B (swap).** Project core-value statement (STATE.md) says
"Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and
keystroke programming must behave identically to original hardware". Inverting CATALOG 1/2
breaks that for an op the user can dispatch by name. Listing registers can live as a
non-ROM helper added in a future quick-task or under a synthetic-only path.

**If the planner / user confirms Option A:** add a clear divergence comment in code + in
CLAUDE.md "Settled Architecture Decisions" so the next maintainer knows this is intentional.

### OQ-2: SIZE 0 behavior (D-22.11) — RESOLVED: Option A

**RESOLVED 2026-05-14 (user-confirmed):** Option A — clamp silently to 1.
`nnn == 0` → `state.regs.resize(1, HpNum::zero())`; `nnn > 319` → InvalidOp.
D-22.11 amended accordingly. Documented as a "1-register minimum" divergence
from real HP-41 (which accepts SIZE 000).

**Issue:** Hardware accepts `SIZE 000`. D-22.11 wording: `nnn.clamp(1, 319)` AND "Anything
outside [1, 319] returns InvalidOp". These contradict (clamp says coerce-silently, "outside
returns InvalidOp" says reject).

**Options:**
- **(A) `nnn == 0` → `regs.resize(1, _)` (clamp silently)** — matches the `clamp(1, 319)` reading.
- **(B) `nnn == 0` → `Err(InvalidOp)` (reject)** — matches the "Anything outside [1, 319]" reading.
- **(C) `nnn == 0` → `regs.resize(0, _)` (hardware-faithful, zero data registers)** — pure
  hardware fidelity. Risk: every subsequent `op_sto`/`op_rcl` returns InvalidOp until SIZE is
  re-set. May surprise users who don't realize they zeroed their registers.

**Recommendation:** **Option (A) — clamp silently.** A `SIZE 0` is almost never what the user
intended; clamping to 1 keeps the calculator usable. Documenting this as a "1-register minimum"
divergence in CLAUDE.md is cheap.

**Affects:** plan 22-03 test-spec wording. Resolve before writing the integration test.

### OQ-3: ASN delete / overwrite semantics — RESOLVED: Option A

**RESOLVED 2026-05-14 (planner-confirmed via D-22.18 amendment):** Option A —
empty name removes. `Op::Asn { name: "", key_code: 11 }` invokes
`state.assignments.remove(&11)`; non-empty name calls `.insert(key_code, name)`.
Hardware-faithful "ASN '' nn" undoes "ASN 'XYZ' nn".

**Issue:** CONTEXT.md says `Op::Asn { name, key_code }` does `state.assignments.insert(key_code,
name)`. What about REMOVING an assignment? Hardware: assigning the original (built-in) function
to its hard-coded key removes the assignment. Our model: `insert` always inserts.

**Options:**
- **(A) Re-ASN with empty name deletes:** `state.assignments.remove(&key_code)`.
- **(B) Re-ASN with empty name inserts the empty string** — frontend interprets as "no assignment".
- **(C) Defer entirely to Phase 25/26** — Op::Asn always inserts; UNASN-equivalent comes later.

**Recommendation:** **Option (A) — empty name removes.** Cheap, hardware-faithful semantics
("ASN '' 11" undoes "ASN 'SIN' 11"). Add one assertion to the FN-KEY-01 test:
`Op::Asn { name: "".to_string(), key_code: 11 }` should leave `state.assignments.get(&11) == None`.

**Affects:** plan 22-04 implementation sketch.

### OQ-4: Op::Cla / Op::AlphaClear display-name divergence — RESOLVED: acknowledge, no change

**RESOLVED 2026-05-14:** Acknowledged, no code change beyond D-22.13.
Op::Cla displays as "CLA"; legacy Op::AlphaClear keeps "CLRALPHA" for v1.x
save-file fidelity. Will be flagged in 22-03 commit message and Phase 22
SUMMARY changelog.

**Issue:** D-22.13 commits `Op::Cla` displays as `"CLA"`, `Op::AlphaClear` displays as
`"CLRALPHA"`. Both are valid HP-41 names. Real HP-41 program listings show `CLA`. But
a v1.0 save file's recorded `Op::AlphaClear` will display as `"CLRALPHA"` after Phase 22.
This is intentional (per CONTEXT.md), but is a visible divergence in any v1.x save loaded
in v2.2. Planner should confirm this is acceptable and add a "what to expect" note in
the Phase 22 SUMMARY for the eventual milestone changelog.

**Recommendation:** No change required — but flag it in plan 22-03 commit message.

---

## 7. Code Examples (Verified)

The CONTEXT.md `<code_context>` block contains exact signature sketches at lines 569–630.
After reading the actual source code, those sketches are **correct** with the following
tightening recommendations:

### Pattern: `resume_program` (CONTEXT.md sketch + reset-on-error fix)

```rust
// hp41-core/src/ops/program.rs
//
// Phase 22 (D-22.2 / FN-PROG-01). Mirrors run_program() at lines 139-169
// but skips the entry-label search — state.pc is the resume point.
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

### Pattern: `Op::Stop` run_loop arm (verified against `Op::Prompt`)

```rust
// hp41-core/src/ops/program.rs run_loop match (insert near line 272 — Op::Prompt arm)
//
// Unlike Op::Prompt, Op::Stop writes NOTHING to display_override.
// The previous step's display value persists until the next dispatch().
// state.pc is already advanced past the STOP step by the top-of-iteration pc += 1.
Op::Stop => break,
```

### Pattern: `Op::GtoInd` / `Op::XeqInd` run_loop arms (verified against Op::Gto / Op::Xeq)

```rust
// hp41-core/src/ops/program.rs run_loop match (insert near Op::Gto at line 201
// and Op::Xeq at line 205 — same shape, one extra dereference + integer check).
Op::GtoInd(reg) => {
    let pointer = state.regs.get(reg as usize).ok_or(HpError::InvalidOp)?.clone();
    let int_part = pointer.trunc_int();
    if int_part != pointer { return Err(HpError::InvalidOp); }
    // Convert pointer to string label (e.g. "42")
    let label_str = int_part.inner().to_string();
    let target = find_in_program(program, &label_str)?;
    state.pc = target + 1; // pc → step AFTER the LBL marker (matches Op::Gto)
}
Op::XeqInd(reg) => {
    if state.call_stack.len() >= 4 {
        return Err(HpError::CallDepth); // pre-mutation check (matches Op::Xeq at 207)
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

### Pattern: `Op::Pse` execute_op arm (mirrors `Op::Beep` at sound.rs:13–17)

```rust
// hp41-core/src/ops/program.rs execute_op match (Op::Pse arm; goes wherever Phase 21
// Op::View etc. sits — sound section is also a fine home).
Op::Pse => {
    let formatted = crate::format::format_hpnum(&state.stack.x, &state.display_mode);
    state.display_override = Some(formatted);
    state.event_buffer.push("PAUSE 1000".to_string());
    crate::stack::apply_lift_effect(state, crate::stack::LiftEffect::Neutral);
    Ok(())
}
```

### Pattern: `Op::Cla` execute_op arm (delegates to alpha.rs::op_alpha_clear)

```rust
// hp41-core/src/ops/program.rs execute_op match
Op::Cla => crate::ops::alpha::op_alpha_clear(state),
```

### Pattern: `Op::Clst` (new function in stack_ops.rs or memory.rs)

```rust
// hp41-core/src/ops/memory.rs (new file) — or stack_ops.rs / registers.rs
//
// CLST: zero X/Y/Z/T. PRESERVE LASTX and lift_enabled (D-22.14).
pub fn op_clst(state: &mut CalcState) -> Result<(), HpError> {
    state.stack.x = HpNum::zero();
    state.stack.y = HpNum::zero();
    state.stack.z = HpNum::zero();
    state.stack.t = HpNum::zero();
    // lastx and lift_enabled deliberately UNTOUCHED
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

### Pattern: `Op::Size` with bounds-aware resize

```rust
// hp41-core/src/ops/memory.rs
//
// SIZE nnn — resize regs to nnn (D-22.11). See §6 OQ-2 for the SIZE-0 decision.
pub fn op_size(state: &mut CalcState, nnn: u16) -> Result<(), HpError> {
    if nnn == 0 || nnn > 319 {
        return Err(HpError::InvalidOp); // Resolve OQ-2 first.
    }
    state.regs.resize(nnn as usize, HpNum::zero());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

### Pattern: `op_sto` after bounds-audit fix

```rust
// hp41-core/src/ops/registers.rs (Wave-0 fix)
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

### Pattern: CATALOG 2 (programs) line builder

```rust
// hp41-core/src/ops/catalog.rs (new file)
//
// CATALOG 2: list each Op::Lbl with step count to next Op::Lbl (or end).
fn catalog_programs(state: &CalcState) -> Vec<String> {
    let labels: Vec<(usize, &str)> = state.program.iter().enumerate()
        .filter_map(|(i, op)| match op {
            Op::Lbl(n) => Some((i, n.as_str())),
            _ => None,
        })
        .collect();
    let mut lines = Vec::new();
    for (idx, (pos, name)) in labels.iter().enumerate() {
        let end = labels.get(idx + 1).map(|(p, _)| *p).unwrap_or(state.program.len());
        let steps = end - pos;
        lines.push(format!("{:<24}", format!("LBL {name:9}  {steps:5}")));
    }
    lines
}
```

---

## 8. State of the Art

**No external library updates required.** All implementations reuse existing
`hp41-core` primitives (HpNum, Stack, format_hpnum, apply_lift_effect, run_loop,
find_in_program). The phase is entirely additive.

**Deprecated/outdated:** Nothing — every locked decision either inherits from
Phase 3/11/12/21 patterns or introduces a clean addition (new field, new variant).

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | HP-41 hardware PSE ≈ 1 second | §1 D-22.4 row | [VERIFIED via HPmuseum forum / HP-41 archived manual]. Low risk. |
| A2 | HP-41 SIZE 000 is hardware-valid | §1 D-22.11 row, OQ-2 | [VERIFIED via HPmuseum SIZE reference]. Low risk. |
| A3 | HP-41 CATALOG 1 = programs (not registers) | §1 D-22.16 row, OQ-1 | [VERIFIED via HP-41 Owner's Manual ch. 4 + HPmuseum CATALOG ref]. Low risk; this is the "what's standard" answer. The contrary D-22.16.1 wording is the planner-side decision under review. |
| A4 | ASN key code is row×10+col, 1-indexed (positive = main, negative = shifted) | §1 D-22.17 row | [VERIFIED via HP-41 Owner's Manual app. F + Phase 12 last_key_code precedent]. Low risk; D-22.17 already uses the unshifted-only u8 simplification, which matches the existing `last_key_code` field's encoding. |
| A5 | HP-41 CLST preserves LASTX and the lift flag | §1 D-22.14 row | [CITED: HP-41C Owner's Manual ch. 7 stack-clearing description]. Low risk. |
| A6 | The `display_override` dispatch-top clear (Phase 21 Pitfall 5) plays correctly with PSE | §2 Pitfall 3 | Verified by re-reading `run_loop` (line 177–284) and `dispatch` (line 405–423) — `run_loop` does NOT route through `dispatch`, so `display_override` survives between iterations as required. Low risk. |
| A7 | Σ+ registers (R01–R06) are hardcoded by `op_sigma_plus` etc., NOT redirected by system flags 26–28 in current `hp41-core` | §2 Pitfall 5, §3 audit | Verified by reading `stats.rs:27–40`. The flag-26/27/28 register-redirection feature does NOT yet exist in v2.x — it's a v3.x consideration. Phase 22 audit applies to the current hardcoded R1–R6 layout. Low risk. |

**Net:** All 7 assumptions are verified; none are unverified `[ASSUMED]` claims that the
planner needs to resolve. The 4 OPEN QUESTIONS in §6 are decisions the planner must make
on the CONTEXT.md design itself (not on unverified facts).

---

## Sources

### Primary (HIGH confidence)
- `hp41-core/src/ops/program.rs:139–169, 177–285, 286–466, 506–541` — run_program, run_loop, execute_op, helpers (verified line-precise)
- `hp41-core/src/ops/registers.rs:13–102, 108–163` — op_sto/op_rcl/op_sto_arith/op_clreg
- `hp41-core/src/ops/alpha.rs:13–48` — op_alpha_clear (CLA basis)
- `hp41-core/src/ops/stats.rs:1–242` — Σ+/Σ-/MEAN/SDEV/LR/YHAT/CORR/ClSigmaStat (bounds audit scope)
- `hp41-core/src/ops/display_ops.rs:13–72` — VIEW pattern (regs read)
- `hp41-core/src/ops/sound.rs:1–30` — BEEP/TONE pattern (event_buffer push reference for PSE)
- `hp41-core/src/state.rs:53–142` — CalcState fields, serde-default precedent
- `hp41-core/src/num.rs:213–227` — `trunc_int()` (GTO IND / XEQ IND primitive)
- `hp41-core/src/ops/mod.rs:404–591` — dispatch + Op enum (4-place rule landing sites)
- `hp41-cli/src/prgm_display.rs:1–199` and `hp41-gui/src-tauri/src/prgm_display.rs:1–100` — both prgm_display copies (the 4-place rule)
- `.planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md` — 25 locked decisions

### Secondary (MEDIUM confidence — cross-verified)
- [HP-41C/CV/CX Programming — HPmuseum](https://www.hpmuseum.org/prog/hp41prog.htm) — CATALOG numbering, PSE behavior
- [HP-41C Owner's Manual (archived hpcalc.org)](https://archived.hpcalc.org/greendyk/hp41c-manual/) — STOP/PSE/CATALOG/CLST/ASN sections
- [HP-41C/41CV Operating Manual (literature.hpcalc.org)](https://literature.hpcalc.org/community/hp41c41cv-om-en.pdf) — SIZE function range
- [HP-41C Quick Reference Guide (literature.hpcalc.org)](https://literature.hpcalc.org/community/hp41c-qrg-en.pdf) — function summaries
- [HP-41 PSE pause duration forum thread](https://forum.hp41.org/viewtopic.php?f=20&t=504) — 1-second hardware confirmation
- [HP-41 Wikipedia article](https://en.wikipedia.org/wiki/HP-41C) — 319-register architecture

### Tertiary (LOW confidence — none)
- No claims in this RESEARCH.md depend solely on unverified web sources. All hardware claims
  are cross-referenced against the archived Owner's Manual or HPmuseum.

---

## Metadata

**Confidence breakdown:**
- CONTEXT.md design completeness: HIGH — all 25 decisions are concrete; 4 open questions surfaced for planner.
- Hardware fidelity verification: HIGH — manual + HPmuseum confirm STOP/PSE/CATALOG/SIZE/CLST/ASN. Two divergences flagged (CATALOG 1/2 inversion; SIZE 0 ambiguity); planner resolves before plan generation.
- Code-context verification: HIGH — every canonical-ref line range in CONTEXT.md verified against actual source (program.rs, registers.rs, alpha.rs, sound.rs, state.rs, num.rs).
- Wave-0 bounds-audit scope: HIGH — full enumeration of 28 production sites with line numbers + concrete fix patterns.
- Test architecture: HIGH — Phase 21 precedent is clear; the four-integration-file split is well-supported.

**Research date:** 2026-05-14
**Valid until:** 2026-06-13 (30 days — stable domain, no external library dependencies)

## RESEARCH COMPLETE
