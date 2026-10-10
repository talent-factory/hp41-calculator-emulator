# Phase 65: Standalone Fidelity Fixes - Context

**Gathered:** 2026-06-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Close four **independent** hardware-fidelity gaps between the emulator and a real HP-41CX. Three are small frontend display/entry fixes; one (MATH-01) escalated during discussion into a **core numeric-range extension** and is deliberately kept in this phase.

1. **DISP-01** (FGAP-02) — the CLI renders `state.display_override` (VIEW/AVIEW/PROMPT) on the main display, matching the GUI which already does this.
2. **DISP-02** (FGAP-05) — CHS during mantissa entry toggles the sign of the entry buffer **in place**, no flush, no stack lift.
3. **DISP-03** (FGAP-07) — AON (flag 48) auto-displays the ALPHA register after every operation; AOFF stops it. Wired on **both** CLI and GUI.
4. **MATH-01** (FGAP-03) — `FACT(27..=69)` returns the correct factorial in scientific notation (10 sig digits) instead of `Overflow`. **This requires extending `HpNum`'s representable range to the full HP-41 ±9.999999999E±99**, because `rust_decimal` caps at ~7.92E28 and FACT(28..69) (up to ~1.7E98) cannot be stored as a Decimal at all.

**Requirements:** DISP-01, DISP-02, DISP-03, MATH-01.

**In scope:** the four fixes above, including the `HpNum` range extension and its ADR (MATH-01). No new `Op` variants → the 4-way exhaustive-match invariant is NOT triggered; no new JSON pool entries.

**Out of scope:** any new user-facing capability; FACT input domain changes (X>69 stays OutOfRange, non-integer/negative stay Domain); the other FGAP/UNC items (CATALOG dump FGAP-06, SAVED/GETD block control FGAP-08, UNC-01/02/03 → Phase 66).
</domain>

<decisions>
## Implementation Decisions

### MATH-01 — FACT range / HpNum range extension (the substantive decision)
- **D-01 (range escalation, LOCKED):** Extend `HpNum` to represent the **full HP-41 range ±9.999999999E±99**. This is the *only* path where FACT(28..69) literally works — every op in the emulator currently overflows above the ~7.92E28 Decimal ceiling, so FACT is merely where the limitation is visible. The user explicitly accepted that this turns MATH-01 into an architectural change.
- **D-02 (kept in Phase 65, LOCKED):** The range extension stays **in this phase** alongside the three display fixes — NOT split into a separate phase. Consequence: Phase 65 is no longer "four small fixes." MATH-01 is a heavy plan touching `num.rs` arithmetic, serde, `format_hpnum`, and a Frozen-Invariant ADR; DISP-01/02/03 are light plans. Planner should sequence MATH-01 as the heavy plan with the three display fixes as light, parallelizable plans.
- **D-03 (representation — RESEARCH RESOLVES):** The internal representation is **deferred to the researcher**. Candidate approaches discussed: (a) keep `rust_decimal`'s exact 10-digit mantissa + a separate wider exponent field normalizing to a (mantissa, exp ±99) form — the HP-41's actual BCD+2-digit-exponent model, most faithful, more invasive; (b) f64 fallback only above the Decimal ceiling — smaller blast radius but introduces a second numeric path + base-2 rounding for large values. Researcher evaluates both against the codebase and proposes the approach in RESEARCH.md + an ADR.
  - **LOCKED constraints the chosen representation MUST satisfy:**
    1. Reach the full ±9.999999999E±99 range.
    2. **Not silently regress** the project's 10-significant-digit decimal fidelity (the `rust_decimal` ADR / Frozen Invariant). Any base-2 carve-out must be explicit and documented.
    3. Preserve **save-file backward compatibility** — v1.0–v4.2 saves load unchanged (`#[serde(default)]`/`#[serde(skip)]` discipline; the `HpNum` serde representation must round-trip old values).
  - An **ADR is REQUIRED** — it amends (does not silently abandon) the "Custom BCD evaluated and rejected / `rust_decimal` 1.42 with 10-sig-digit rounding" Frozen Invariant in CLAUDE.md.
- **D-04 (FACT behavior once range supports it):** `op_fact` returns the correct factorial for 27..=69 clipped to 10 sig digits. Input-domain guards are **unchanged**: X>69 → OutOfRange (D-06), non-integer X → Domain, negative X → Domain. Only the post-compute `Decimal::from_f64(...).ok_or(Overflow)` wall is what changes (it currently fails for n≥27). `math.rs` is OUTSIDE the frozen `math1/` tree, so editing `op_fact` is allowed.

### DISP-01 — CLI VIEW/AVIEW/PROMPT display
- **D-05 (mirror GUI, LOCKED):** CLI `get_display_string()` reads `state.display_override` exactly like the GUI — render it while `Some`, fall through otherwise. The core's existing clear semantics govern dismissal (CLD, the next VIEW/AVIEW overwrite, or starting number entry). **Rejected:** hardware "clear-on-next-keypress" — it would force the CLI to clear the override itself and diverge from the GUI, breaking D-25.6 parity for no fidelity gain in an event-driven TUI.
- **D-06 (priority placement, LOCKED):** Insert `display_override` into the CLI priority chain at: `clock > stopwatch > entry_buf > prgm > display_override > alpha > X`. (Clock/stopwatch/entry/prgm take over when active; the override sits above the plain ALPHA-mode and X fallbacks.) This is the CLI analogue of the GUI's `modal preview > pending_yield.text > display_override > display_str` chain.

### DISP-02 — CHS during number entry
- **D-07 (in-buffer toggle, LOCKED):** When `entry_buf` is non-empty AND contains no `e` (mantissa-only entry), CHS toggles a leading `-` in the entry buffer **in place** — no `flush_entry_buf()`, no dispatch, no stack lift. Mirror the existing EEX-CHS in-buffer handling at `hp41-cli/src/app.rs:691-707`.
- **D-08 (empty-buffer path unchanged, LOCKED):** With no active entry (`entry_buf` empty), CHS keeps its current behavior — dispatch `Op::Chs` to negate the X register (hardware-faithful). The fix is purely *additive*: the in-buffer toggle is gated on a non-empty, no-`e` buffer; the EEX case (`1e2`) keeps its existing exponent-sign handling.

### DISP-03 — AON auto-display
- **D-09 (both frontends, LOCKED):** Wire flag-48 reading into **both** the CLI (`get_display_string()`) and the GUI (`App.tsx`), per D-25.6 parity (the user chose parity consistently with DISP-01). The audit confirms neither frontend reads flag 48 today.
- **D-10 (precedence, LOCKED):** AON auto-display replaces the **X fallback at rest** and sits **below** the VIEW override: CLI `... > display_override > (flag48 ? alpha_reg : X)`; GUI `... ?? display_override ?? (flag48 ? alphaText : display_str)`. So a fresh VIEW still shows its value, entry/clock/prgm still take over while active, and at rest with flag 48 set the ALPHA register shows instead of X. AOFF (clear flag 48) reverts to the X fallback.

### Cross-cutting
- **D-11 (stale-comment cleanup):** `hp41-core/src/state.rs` has comments declaring DISP-01 "deferred to v4.4" (≈ lines 80 and 531). Phase 64's CONTEXT also deferred DISP-01 to v4.4. These are now **resolved in Phase 65** — update/remove those comments as part of the DISP-01 plan.
- **D-12 (parity discipline):** All four fixes honor D-25.6 (CLI↔GUI parity) where a frontend is touched, D-11 no-polling (GUI), panic-free core (`#![deny(clippy::unwrap_used)]`), and `just`-only task running. No async, no threads, no new runtime deps.

### Claude's Discretion / Planner
- Exact field/enum shape of the extended `HpNum` representation (follows research's recommendation + ADR).
- Test-fixture strategy for FACT(27..=69) accuracy (extend `tests/numerical_accuracy.rs` / `proptest_math.rs`; recalibrate the Phase-27 proptest magnitude wall that currently assumes X≤26).
- Whether the GUI AON path reads flag 48 from `CalcStateView.flags` (already projected — App.tsx:58) or needs a new projection field (prefer reusing the existing `flags` array — no IPC change).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Fidelity spec (the behavioral targets)
- `.planning/research/DIVERGENCE-AUDIT.md` — **FGAP-02** (DISP-01), **FGAP-03** (MATH-01 / FACT), **FGAP-05** (DISP-02 / CHS, see also UG-04), **FGAP-07** (DISP-03 / AON, see also UG-05), **UG-06** (FACT Decimal-ceiling overflow). Specifies the expected fix, affected files, and complexity per gap.
- `.planning/research/SUMMARY.md` — fidelity-audit overview placing these four in the "standalone" (non-yield-engine) group.

### MATH-01 — range extension (the heavy plan)
- `hp41-core/src/num.rs` — `HpNum(Decimal)` definition (line ~123), `HpNum::rounded` 10-sig rounding (line ~128), every `checked_*` arithmetic op that maps overflow → `HpError::Overflow`. The representation change lives here.
- `hp41-core/src/ops/math.rs` — `op_fact` (line ~454); the `Decimal::from_f64(acc).map(HpNum::rounded).ok_or(Overflow)` wall at ~480-482 is the failure point. (Outside frozen `math1/`.)
- `CLAUDE.md` → "BCD/f64" Frozen Invariant (`rust_decimal` 1.42, 10-sig rounding, custom BCD rejected) — the ADR amends this. Also "Zero new runtime deps since v3.0" — the extension must use no new deps.
- `docs/adr/` — write a new ADR (e.g. `v4.3-00X-hpnum-range-extension.md`) recording the representation choice and the Frozen-Invariant amendment.
- `hp41-core/tests/numerical_accuracy.rs`, `hp41-core/tests/proptest_math.rs` (+ `.proptest-regressions`) — FACT accuracy + the X≤26 magnitude-wall calibration to revisit.

### DISP-01 / DISP-03 — CLI display
- `hp41-cli/src/ui.rs` — `get_display_string()` (~line 132) priority chain; insertion points for `display_override` (D-06) and AON (D-10).
- `hp41-core/src/ops/display_ops.rs` — `op_view`/`op_aview` write `display_override` (~lines 23/32); `op_aon`/`op_aoff` set/clear flag 48 (~lines 53/62); `op_prompt` / CLD context.
- `hp41-core/src/state.rs` — `display_override: Option<String>` (line ~194); **stale "DISP-01 deferred to v4.4" comments at ~lines 80 and 531 to update (D-11)**.

### DISP-03 / DISP-01 — GUI parity
- `hp41-gui/src/App.tsx` — display-text precedence `modal preview > pending_yield.text > display_override > display_str` (line ~1416); `CalcStateView.flags` (line 58) + `display_override` (line 59) already projected. AON inserts at the `display_str` fallback (D-10).

### DISP-02 — CLI CHS
- `hp41-cli/src/app.rs` — the EEX-CHS in-buffer handling at **lines 691-707** is the pattern to mirror for the mantissa case (D-07); empty-buffer CHS → `Op::Chs` stays (D-08).

### Divergence record to update (Phase 66 owns final verification, but note here)
- `docs/hp41-*-divergences.md` / `README.md` (FACT divergence note ~line 170) — the FACT-overflow and DISP/CHS/AON divergences become "implemented (v4.3)". Coordinate with Phase 66's doc sweep.

### Frozen invariants (must honor)
- `CLAUDE.md` — BCD/`rust_decimal` ADR (amended by D-03 ADR), zero-new-deps, no async/no panics, save-file backward compat, 4-way exhaustive match (NOT triggered — no new Op), D-25.6 parity, D-11 no-polling, `just`-only.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`state.display_override: Option<String>`** — already written by `op_view`/`op_aview`/`op_prompt` and consumed by the GUI; DISP-01 just adds the CLI read.
- **GUI display precedence chain** (`App.tsx:1416`) — the exact model to mirror in the CLI (D-06) and extend for AON (D-10).
- **EEX-CHS in-buffer toggle** (`app.rs:691-707`) — the working pattern DISP-02 generalizes to the mantissa case.
- **`flag_set`/`flag_get` + flag 48** (`display_ops.rs:53-62`) — flag is already maintained; DISP-03 is purely a frontend read.
- **`op_fact` domain guards** (`math.rs:454-473`) — keep unchanged; only the post-compute conversion wall changes.

### Integration Points
- `hp41-core/src/num.rs` — extended `HpNum` representation + arithmetic + serde (MATH-01, heavy).
- `hp41-core/src/ops/math.rs` — `op_fact` returns large results via the new representation (MATH-01).
- `hp41-cli/src/ui.rs` — `get_display_string()` gains `display_override` + AON branches (DISP-01, DISP-03).
- `hp41-cli/src/app.rs` — CHS in-buffer toggle (DISP-02).
- `hp41-gui/src/App.tsx` — AON branch at the `display_str` fallback (DISP-03).
- `hp41-core/src/state.rs` — remove stale DISP-01-deferred comments (DISP-01).
- `docs/adr/` — new ADR for the range extension (MATH-01).
</code_context>

<specifics>
## Specific Ideas

- MATH-01 is really "give the whole emulator HP-41 range," surfaced via FACT — the user accepted this scope and chose to keep it in Phase 65.
- DISP-01 and DISP-03 both want strict CLI↔GUI parity (the user picked parity twice); implement them on both frontends with matching precedence chains.
- DISP-02 and DISP-01/03 are mechanical and well-specified by the audit; MATH-01 is where research effort concentrates (representation + ADR).
</specifics>

<deferred>
## Deferred Ideas

- **Splitting the range extension into its own phase** — explicitly considered and **rejected** by the user (D-02); recorded here in case the heavy plan proves larger than expected during planning and a re-split is reconsidered.
- **Other FGAP items** — CATALOG dump (FGAP-06), SAVED/GETD bbb.eee block control (FGAP-08) — not in this phase.
- **UNC-01/02/03 verification + divergence-doc finalization** — Phase 66.
</deferred>
