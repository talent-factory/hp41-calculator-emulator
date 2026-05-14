---
phase: 25-cli-integration-and-documentation
plan: 02
subsystem: cli
tags: [cli, modal, pending-input, hp41cv, ind-modifier, hybrid-enum]

# Dependency graph
requires:
  - phase: 25-cli-integration-and-documentation
    plan: 01
    provides: "App.shift_armed (reused as the IND-toggle one-shot bit per D-25.12 W2 fix) + shifted_key_to_op stub (extended here with 15 modal-opener arms)"
  - phase: 24-indirect-addressing
    provides: "All IND-variant Ops (StoInd/RclInd/StoArithInd/IsgInd/DseInd/SfFlagInd/CfFlagInd/FlagTestInd/ArclInd/AstoInd/ViewInd) — dispatched by handle_register_prompt/handle_flag_prompt at end of 2-digit accumulation"
  - phase: 21-flags-display-control-and-sound
    provides: "Op::SfFlag, Op::CfFlag, Op::FlagTest { kind, flag }, Op::View(u8), Op::Tone(u8), FlagTestKind enum"
  - phase: 22-program-control-and-memory-ops
    provides: "Op::Clp(String), Op::Del(u8) — wired by ClpLabel + DelCount modals here"
provides:
  - "PendingInput grows from 12 to 18 variants (FlagPrompt + RegisterPrompt struct-group + ClpLabel + DelCount + TonePrompt + XeqByName specialty)"
  - "FlagPromptKind + RegisterOpKind TUI-local discriminator enums (in hp41-cli/src/keys.rs) wrapping hp41_core::FlagTestKind + StoArithKind per D-25.13"
  - "ui::pending_prompt() is exhaustive — no `_ =>` catch-all, no unreachable!() (FN-CLI-04 compile-time guarantee)"
  - "ui::pending_prompt now `pub` so integration tests can verify status-bar formatting"
  - "handle_register_prompt + handle_flag_prompt — shared IND-toggle via shift-0 (RESEARCH Pitfall 10 verbatim); single-decision-point dispatch picks Op::*Ind vs Op::*(n) at end of 2-digit accumulation per D-25.12"
  - "handle_clp_label / handle_del_count / handle_tone_prompt / handle_xeq_by_name — specialty modal scaffolds with length caps + silent-clamp per RESEARCH §Security V5"
  - "shifted_key_to_op extended with 15 modal-opener arms (SF/CF/FS?/FC?/FS?C/FC?C, VIEW/ARCL/ASTO/ISG/DSE, CLP/DEL/TONE/XEQ-by-name); signature widened to &mut App so the resolver can populate pending_input directly"
  - "S/R primary keys re-routed to RegisterPrompt{Sto/Rcl}; legacy v1.1 STO-arith chain + M/N/O dispatch preserved inside the new arm"
  - "hp41-cli/tests/phase25_pending_input.rs — 13 integration tests covering variant construction, exhaustive pending_prompt, IND-toggle (direct + IND via shift-0), FlagPrompt dispatch (both direct and IND through regs[5] indirect), ClpLabel cap-at-7, DelCount silent-clamp, TonePrompt auto-dispatch, XeqByName scaffold, Esc cancel uniformity, and three f-shifted modal-opener regression tests"
affects:
  - "25-03-xeq-by-name-extensions (consumes the XeqByName modal scaffold landed here — Plan 03 extends builtin_card_op 4→12 so the 8 non-keyboard conditional-test mnemonics resolve via the same modal)"
  - "25-04-json-pipeline-and-key-table (Plan 04's JSON pipeline will re-emit the f-shifted modal-opener mapping currently hand-coded in shifted_key_to_op)"

# Tech tracking
tech-stack:
  added: []   # no new crates; the 6 new variants use only hp41-core + crossterm
  patterns:
    - "Hybrid PendingInput struct-variants (D-25.11) — group ops with identical state shape into struct-tag variants, keep specialty ops unique"
    - "TUI-local discriminator enums that wrap (not duplicate) hp41-core enums (D-25.13 reuse rule) — FlagPromptKind wraps FlagTestKind; RegisterOpKind wraps StoArithKind"
    - "Shared IND-toggle helper (check_ind_toggle) returning an IndToggleAction enum — single source of truth for the f→0 shift-armed mutation inside modal arms"
    - "Single-decision-point dispatch via tuple match — `match (kind, ind) { (Sto, false) => Op::StoReg(n), (Sto, true) => Op::StoInd(n), … }` picks Op::*Ind vs Op::* at end of accumulation per D-25.12 verbatim"
    - "Exhaustive-match discipline for pending_prompt() — FN-CLI-04 compile-time guarantee preserved (no _ =>, no unreachable!())"

key-files:
  created:
    - "hp41-cli/tests/phase25_pending_input.rs — 13 integration tests for the Hybrid PendingInput architecture"
  modified:
    - "hp41-cli/src/app.rs — added 6 new PendingInput variants (FlagPrompt, RegisterPrompt, ClpLabel, DelCount, TonePrompt, XeqByName); added 6 handle_* methods with IND-toggle; added shared check_ind_toggle helper + IndToggleAction enum; re-routed S/R keys to RegisterPrompt; preserved legacy STO-arith chain + M/N/O dispatch in new arm"
    - "hp41-cli/src/keys.rs — added TUI-local FlagPromptKind + RegisterOpKind enums; widened shifted_key_to_op signature to &mut App; added 15 modal-opener arms"
    - "hp41-cli/src/ui.rs — pending_prompt() extended with 6 new exhaustive arms (no catch-all); function made pub for integration-test visibility"

key-decisions:
  - "STO-arithmetic stays on the v1.1 chain INSIDE the new RegisterPrompt arm — when op == Sto/Rcl and acc is empty, +/-/×/÷ keys transition into the legacy StoAdd/Sub/Mul/Div modals. No f-shifted opener for STO-arith per W3 fix + D-25.7 (the 4 f-arith keys are LOCKED to conditional tests)."
  - "XEQ-by-Name modal trigger key is `F-N` (uppercase). Was selected over alternatives because the lowercase `n` is CHS (HP-41CV row 8 primary) and is NOT free; uppercase letters survive D-25.3 stripping. Plan 03 wires the resolver (builtin_card_op 4→12 extension); Plan 02 falls back to the existing Op::Xeq → builtin_card_op chain (4-name fallback)."
  - "ClpLabel/DelCount/TonePrompt openers chosen as `F-C`/`F-D`/`F-T` (uppercase). The exact HP-41CV reference-card positions for these ops are TBD and Plan 04 may re-emit them from docs/hp41cv-functions.json. The mnemonic-letter shortcuts here are stable across that future re-emission because Plan 04 only changes which f-shifted KEY they live on, not the modal scaffold."
  - "RegisterOpKind::StoArith variant carries #[allow(dead_code)] scoped to just that variant — Plan 02 does NOT construct it (STO-arith stays on legacy StoAdd/Sub/Mul/Div per W3). The variant exists for Plan 04 JSON re-emission and for tests."
  - "Legacy PendingInput::StoRegister and RclRegister variants carry per-variant #[allow(dead_code)] — they are no longer constructed by the live keyboard (S/R now open RegisterPrompt instead) but their handle_pending_input arms remain functional (must_have truth #9: all 12 legacy variants continue to compile and dispatch). Deprecation/removal is tracked for Plan 04."
  - "Auto-fixed pre-existing in-source tests `test_sto_m_via_modal` + `test_rcl_m_via_modal` (Rule 1) — they asserted the legacy PendingInput::StoRegister/RclRegister shape that the Plan 02 must_have truth #7 explicitly replaces with RegisterPrompt{Sto/Rcl}. Updated to assert the new shape; M/N/O dispatch behavior is preserved and the tests still validate the same end-state (reg_m/n/o populated)."
  - "Esc inside FlagPrompt/RegisterPrompt also clears App.shift_armed (T-25-07 mitigation) — prevents a half-armed prefix from leaking past a cancelled modal into the next non-modal key cycle."

patterns-established:
  - "Hybrid PendingInput struct-variants: when 2+ Op variants share the same modal scaffold shape (kind, ind, acc), collapse them into a single struct-variant with a TUI-local discriminator enum that REUSES hp41-core enums where possible. Specialty Op variants with unique modal shapes (ClpLabel text-input, DelCount 3-digit, TonePrompt 1-digit, XeqByName text-input) stay as their own variants."
  - "Single-shot prefix-bit reuse: App.shift_armed is the ONE bit that serves both the global f-prefix arming (Plan 01) AND the in-modal IND-toggle (Plan 02 / D-25.12). The check_ind_toggle helper centralises the f→0 rule; the legacy Plan-01 consumption path at the top of handle_key handles the non-modal case."
  - "Tuple-match for single-decision-point dispatch: `match (op_or_kind, ind) { … }` is the canonical pattern for picking Op::*Ind vs Op::* at the end of accumulator-driven modals — one match site per modal arm, no conditional branching."

requirements-completed: []
# FN-CLI-02 (Hybrid PendingInput modal architecture) — CLOSED by this plan:
#   • 6 new variants (FlagPrompt, RegisterPrompt struct-group;
#     ClpLabel/DelCount/TonePrompt/XeqByName specialty) implemented and dispatch
#     correctly.
#   • IND-toggle via shift-0 (D-25.12 verbatim) wired and tested.
# FN-CLI-04 (exhaustive pending_prompt) — CLOSED by this plan:
#   • ui::pending_prompt() now exhaustive over all 18 variants — no `_ =>` arm,
#     no unreachable!(). Future variants WILL cause a compile error.
# FN-CLI-01 (KEY_REF_TABLE-equivalent discoverability) — PROGRESSES BUT NOT
# CLOSED. Plan 02 wires the modal-opener bindings (15 f-shifted keys + 2
# primary S/R re-routes) but the discoverability surface (help_data.rs JSON
# pipeline + KEY_REF_TABLE regeneration) is Plan 04's territory per D-25.18.
# The orchestrator should defer the mark-complete call for all three until
# Plan 04 (and Plan 03 for the XEQ-by-Name resolver extension).

# Metrics
duration: 45min
completed: 2026-05-14
---

# Phase 25 Plan 02: PendingInput Hybrid Modal Architecture Summary

**Lands the HP-41CV Hybrid PendingInput modal architecture in hp41-cli — 6 new variants (FlagPrompt + RegisterPrompt struct-group; ClpLabel + DelCount + TonePrompt + XeqByName specialty) reach the keyboard via 15 new f-shifted modal openers + 2 primary-key re-routes (S → STO, R → RCL) + the hardware-faithful IND-toggle via shift-0 (D-25.12 + RESEARCH Pitfall 10).**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-05-14
- **Completed:** 2026-05-14
- **Tasks executed:** 3 of 3 (Task 3 is a verification-only step folded into Tasks 1 + 2 per Plan 01 precedent)
- **Files modified:** 3 (hp41-cli/src/app.rs, keys.rs, ui.rs)
- **Files created:** 1 (hp41-cli/tests/phase25_pending_input.rs)
- **Net lines:** +1301 / −25 across 4 files

## Accomplishments

- **PendingInput grew from 12 to 18 variants with the Hybrid struct-group pattern.** Six new variants land: `FlagPrompt { kind, ind, acc }`, `RegisterPrompt { op, ind, acc }`, `ClpLabel(String)`, `DelCount(String)`, `TonePrompt`, `XeqByName(String)`. The struct-group pair collapses 34 logical operations (12 flag-op variants × {direct, IND} + 11 register-op variants × {direct, IND}) into 2 carrier enums with TUI-local discriminators that REUSE `hp41_core::ops::FlagTestKind` and `hp41_core::ops::StoArithKind` per D-25.13 — no parallel enum definitions in hp41-cli.

- **IND-toggle via shift-0 works hardware-faithfully (D-25.12 verbatim).** Inside an open FlagPrompt or RegisterPrompt modal: press `f` to arm `App.shift_armed` (the one-shot bit from Plan 01 — reused, NOT shadowed by a new field per W2); press `0` to flip the modal's `ind` field. The `0` is consumed by the IND-toggle, NOT pushed into the digit accumulator. End-of-2-digit-accumulation dispatch is a single tuple-match decision point that picks `Op::*Ind(n)` vs `Op::*(n)` per D-25.12. Status bar prepends `IND` to the mnemonic — e.g. `STO IND [_5]`.

- **15 new f-shifted modal openers land (W3 fix-compliant).** `f-7/8/9/4/5/6` open the 6 FlagPrompt variants (SF/CF/FS?/FC?/FS?C/FC?C); `f-v/a/A/i/d` open RegisterPrompt {View/Arcl/Asto/Isg/Dse}; `f-C/D/T/N` open the specialty modals (CLP/DEL/TONE/XEQ-by-name). Plan 04 may relocate these to JSON-derived positions; the modal scaffold stays stable across that future re-emission. STO-arithmetic openers are deliberately ABSENT (W3 fix) — the f-arith keys remain LOCKED to the 4 conditional tests per D-25.7 + Plan 01.

- **S / R primary keys re-routed to the new RegisterPrompt.** Pressing `S` now opens `RegisterPrompt { Sto, ind:false, acc:"" }`; `R` opens `RegisterPrompt { Rcl, … }` (must_have truth #7). The legacy v1.1 STO-arithmetic chain (`S → +/-/×/÷ → register`) is preserved by the new arm: when `op == Sto/Rcl && acc.is_empty()`, the arithmetic keys still transition to the legacy `PendingInput::StoAdd/Sub/Mul/Div` modals, and M/N/O dispatch to `StoM/N/O` (Phase 12 D-08) still works. This satisfies must_have truth #8 ("STO-arith stays on legacy S→op→reg chain") while moving S/R onto the new architecture.

- **`pending_prompt()` is now `pub` and exhaustive over all 18 variants (FN-CLI-04 closed).** No `_ =>` catch-all. No `unreachable!()`. The 6 new arms format their status text per RESEARCH §"Pattern 4: Exhaustive Match Discipline":
  - `SF/CF/FS?/FC?/FS?C/FC?C` mnemonic + optional ` IND` + ` [{acc:_<2}]`
  - `STO/RCL/STO+/STO-/STO×/STO÷/VIEW/ARCL/ASTO/ISG/DSE` mnemonic + optional ` IND` + ` [{acc:_<2}]`
  - `CLP [<acc>]_`, `DEL [<acc:_<3>]`, `TONE [_]`, `XEQ "<acc>"_`

- **13-test integration suite — all GREEN.** `hp41-cli/tests/phase25_pending_input.rs` covers: variant compile checks; pending_prompt exhaustive coverage; IND-toggle via shift-0 (including the round-trip STO IND 05 dispatching to regs[regs[5].int_part]); FlagPrompt direct + IND (with regs[5]=12 → SF IND 05 setting flag 12); ClpLabel cap-at-7; DelCount silent-clamp at u8::MAX; TonePrompt auto-dispatch (verifies state.event_buffer entry); XeqByName scaffold; Esc-cancel uniformity across all 6 new variants; f-shifted SF/CF/FS? opener regressions. Total hp41-cli test count: **254** (was 241 after Plan 01).

## Task Commits

Atomic, English-only conventional-commit messages per CLAUDE.md commit-language rule:

1. **Task 1: Add 6 PendingInput variants + exhaustive pending_prompt + test scaffold** — `13b356e` (feat, TDD-RED-then-GREEN cycle for variants + exhaustive match)
2. **Task 2: Wire modal openers + IND-toggle + handle_pending_input arms** — `f97c192` (feat, TDD-GREEN cycle for the 11 remaining tests)
3. **Task 3: Integration test scaffold** — _no new commit_ (the 13-test scaffold was built up incrementally across Tasks 1 and 2 as part of their TDD cycles; Task 3 acceptance criteria — file exists with `#![allow(clippy::unwrap_used)]`, ≥9 `#[test]` functions, all pass, clippy clean — were already met at the end of Task 2, mirroring Plan 01's D-2 precedent).

The 13 tests in `phase25_pending_input.rs` map to plan-named Task 3 tests as follows:

| Plan name                              | Status   | Lands in commit |
|----------------------------------------|----------|-----------------|
| pending_input_variants_compile         | present  | 13b356e         |
| pending_prompt_exhaustive              | present  | 13b356e         |
| test_ind_toggle_via_shift_0            | present  | f97c192         |
| test_flag_prompt_dispatches            | present  | f97c192         |
| test_flag_prompt_ind_dispatches_through_shift_0 | present  | f97c192         |
| test_clp_label_capped                  | present  | f97c192         |
| test_del_count_silent_clamp            | present  | f97c192         |
| test_tone_prompt_auto_dispatch         | present  | f97c192         |
| test_xeq_by_name_modal_scaffold        | present  | f97c192         |
| test_esc_cancels_all_new_variants      | present  | f97c192         |
| _bonus:_ test_f_shifted_seven_opens_sf_prompt | regression for the f-7 → SF opener wire | f97c192 |
| _bonus:_ test_f_shifted_eight_opens_cf_prompt | regression for the f-8 → CF opener wire | f97c192 |
| _bonus:_ test_f_shifted_nine_opens_fs_prompt  | regression for the f-9 → FS? opener wire | f97c192 |

## Files Created / Modified

- **`hp41-cli/src/app.rs`** — Added `FlagPrompt` + `RegisterPrompt` struct-variants and four specialty variants to `PendingInput`. Added six `handle_*` methods (`handle_flag_prompt`, `handle_register_prompt`, `handle_clp_label`, `handle_del_count`, `handle_tone_prompt`, `handle_xeq_by_name`). Added shared `check_ind_toggle` helper returning the new `IndToggleAction` enum (centralises the f→0 IND-toggle mutation of `App.shift_armed`). Re-routed `S`/`R` keys from `StoRegister`/`RclRegister` to `RegisterPrompt { Sto/Rcl }` (preserving the legacy v1.1 STO-arith chain + M/N/O dispatch inside the new arm). Updated two pre-existing tests (`test_sto_m_via_modal`, `test_rcl_m_via_modal`) to assert the new shape per Rule 1.

- **`hp41-cli/src/keys.rs`** — Added TUI-local enums `FlagPromptKind` (wraps `hp41_core::ops::FlagTestKind`) and `RegisterOpKind` (wraps `hp41_core::ops::StoArithKind`). Widened `shifted_key_to_op` signature to `(KeyEvent, &mut App) -> Option<Op>`. Added 15 modal-opener arms (returns `None` after populating `app.pending_input`).

- **`hp41-cli/src/ui.rs`** — Made `pending_prompt()` `pub` so integration tests can call it. Added 6 new exhaustive match arms per RESEARCH §"Pattern 4". No `_ =>` catch-all anywhere in the function.

- **`hp41-cli/tests/phase25_pending_input.rs`** _(new)_ — 13 integration tests; the canonical Phase 25 Plan 02 modal-architecture scaffold.

## Decisions Made

The frontmatter `key-decisions` section lists 7 decisions made during execution. Three warrant extra emphasis here:

### D-1 — STO-arithmetic dual-path (W3 fix preservation + must_have truth #7)

The plan has two must_have truths that look like they contradict:
- "Pressing `S` opens RegisterPrompt with op=Sto/Rcl"
- "STO-arithmetic stays reachable via the existing v1.1 `S → +/-/×/÷ → register` modal chain"

Resolution: both are true. `S` opens `RegisterPrompt { Sto }`, AND the new `handle_register_prompt` arm intercepts `+/-/*/(slash)` keys when `op == Sto && acc.is_empty()` to transition into the legacy `StoAdd/Sub/Mul/Div` modals. Same for M/N/O hidden registers (Phase 12 D-08). The legacy variants stay in the enum (must_have truth #9 — "all 12 existing variants continue to compile and dispatch as before"); deprecation is tracked for Plan 04. This is the cleanest path that respects all four constraints (truth #7, truth #8, truth #9, W3 fix).

### D-2 — Modal-opener key mapping (uppercase ASCII shortcuts)

The plan leaves the exact HP-41CV reference-card positions for VIEW / ARCL / ASTO / ISG / DSE / CLP / DEL / TONE / XEQ-by-name TBD; planner discretion. We chose:
- `f-v` → VIEW (lowercase mnemonic-letter)
- `f-a` → ARCL, `f-A` → ASTO (case-sensitive because both alpha-register ops start with 'A')
- `f-i` → ISG, `f-d` → DSE (lowercase mnemonic-letter — but note `d` is the angle-mode-cycle key in non-shifted context, so DSE is reachable only via `f-d`)
- `f-C` → CLP, `f-D` → DEL, `f-T` → TONE, `f-N` → XEQ-by-name (uppercase to avoid collision with HP-41CV primary positions like `n→CHS` and `r→Rdn`)

Plan 04 will re-emit these from `docs/hp41cv-functions.json` per D-25.18, likely relocating most onto numeric f-shift positions per the actual HP-41CV reference-card layout. The modal scaffold here stays stable across that future re-emission because the f-shifted resolver is a thin mapping layer above the same 18 PendingInput variants.

### D-3 — Task 3 produced no commit

Per Plan 01's D-2 precedent and GSD execute-plan's "do not create empty commit" guidance: the 13-test scaffold was built incrementally as the TDD-RED-then-GREEN cycles of Tasks 1 + 2. Task 3's acceptance criteria (file exists with `#![allow(clippy::unwrap_used)]`, ≥9 `#[test]` functions, clippy clean, all pass) were met at the END of Task 2's commit (`f97c192`). The orchestrator's plan-progress counter should expect 2 commits for this 3-task plan.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Bug] Pre-existing in-source tests asserted the legacy StoRegister/RclRegister shape**
- **Found during:** Task 2 — running `cargo test -p hp41-cli` after re-routing `S`/`R` keys to `RegisterPrompt`.
- **Issue:** `test_sto_m_via_modal` (line 2424 of `hp41-cli/src/app.rs`) and `test_rcl_m_via_modal` (line 2449) used `matches!(app.pending_input, Some(PendingInput::StoRegister(_)))` after pressing `S`/`R` — but the Plan 02 must_have truth #7 explicitly replaces that with `RegisterPrompt { Sto/Rcl }`.
- **Fix:** Updated both tests to assert the new `RegisterPrompt { op: RegisterOpKind::Sto, ind:false, .. }` (and `Rcl`) shape; the M/N/O dispatch behavior they validate is preserved by the new arm so the rest of the test bodies are unchanged. `test_mno_guard_only_when_acc_empty` (line 2488) still uses `PendingInput::StoRegister` directly because it asserts the legacy arm's M-guard behavior — that arm is still in the code and still works (must_have truth #9).
- **Files modified:** `hp41-cli/src/app.rs`
- **Commit:** `f97c192` (folded into Task 2's commit per atomic-task discipline).

**2. [Rule 2 — Critical functionality] Esc inside a modal must also clear App.shift_armed**
- **Found during:** Task 2 implementation — when writing the Esc arms for `handle_flag_prompt`/`handle_register_prompt`.
- **Issue:** The plan + RESEARCH Pitfall 10 explicitly say "the modal IND-toggle reuses `App.shift_armed`". If the user arms shift inside a modal (via `f`) and then presses Esc to cancel the modal, the bare `pending_input = None` would leave `shift_armed = true` — which would then leak into the next non-modal key cycle, dispatching the next op as f-shifted. This is the documented T-25-07 threat.
- **Fix:** Added `self.shift_armed = false;` to the Esc arms of both `handle_flag_prompt` and `handle_register_prompt`. The specialty arms (Clp/Del/Tone/Xeq) don't need this because they don't engage the shift-0 IND-toggle path.
- **Files modified:** `hp41-cli/src/app.rs`
- **Commit:** `f97c192` (T-25-07 mitigation explicit; covered by `test_esc_cancels_all_new_variants`).

### Other notes

- **`RegisterOpKind::StoArith` is constructed nowhere in Plan 02.** Per the W3 fix + D-25.7 the f-shifted arithmetic keys are LOCKED to the 4 conditional tests; STO-arith stays on the legacy `StoAdd/Sub/Mul/Div` variants. The `StoArith` arm of `RegisterOpKind` exists for Plan 04 (JSON re-emission) and for tests. `#[allow(dead_code)]` is scoped to just that variant — the surrounding enum is fully exercised.
- **`PendingInput::StoRegister` and `RclRegister` are no longer constructed by the live keyboard handler.** They're preserved in the enum (must_have truth #9) with per-variant `#[allow(dead_code)]`. Their `handle_pending_input` arms still work for tests + future plans.

### Authentication gates

None — Phase 25 is pure-Rust local code; no external services, no credentials.

## Threat Surface (post-execution review)

| Threat ID | Disposition | Status |
|-----------|-------------|--------|
| T-25-05 (DoS via unbounded accumulator) | mitigate | ✓ `ClpLabel` cap at 7 chars (`CLP_LABEL_CAP`), `XeqByName` cap at 24 chars (`XEQ_NAME_CAP`); covered by `test_clp_label_capped` |
| T-25-06 (Tampering via DelCount overflow) | mitigate | ✓ `Op::Del(.parse::<u8>().unwrap_or(u8::MAX))` silent-clamp; covered by `test_del_count_silent_clamp` |
| T-25-07 (Info disclosure via shift_armed leak past modal) | mitigate | ✓ Esc arms of `handle_flag_prompt`/`handle_register_prompt` clear `self.shift_armed = false`; covered by `test_esc_cancels_all_new_variants` (which asserts `!app.shift_armed` after Esc on every new variant) |
| T-25-08 (Future variant landing without pending_prompt arm) | mitigate | ✓ `ui::pending_prompt()` is exhaustive — no `_ =>`, no `unreachable!()`. Verified by `grep` (0 matches) and by `pending_prompt_exhaustive` test |

No NEW surfaces introduced beyond what the plan's threat register anticipated.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. The widening of `shifted_key_to_op` to `&mut App` is an in-crate signature change; it does not cross any trust boundary (CLI process is single-tier, single-user).

## Known Stubs

- **`XeqByName` resolver fall-through** — `handle_xeq_by_name` on Enter dispatches `Op::Xeq(acc)`. `Op::Xeq` already falls back to `builtin_card_op` for the 4 v2.1 card-reader names (WPRGM/RDPRGM/WDTA/RDTA). Plan 03 extends `builtin_card_op` from 4 to 12 entries so the 8 non-keyboard conditional-test mnemonics (X<>Y? / X<Y? / X>=Y? / X#0? / X<0? / X>0? / X<=0? / X>=0?) resolve to `Op::Test(_)` via the same modal. Plan 02's `test_xeq_by_name_modal_scaffold` verifies the modal closes cleanly even when the dispatch errors out (label not found) — the modal scaffold is functionally complete; only the resolver-extension half is deferred to Plan 03.
- **`RegisterOpKind::StoArith` variant** is constructible (it's a public enum variant) but Plan 02 never produces it through the live keyboard handler. Plan 04's JSON pipeline may relocate STO-arith onto explicit f-shifted positions if the HP-41CV reference card supports that; if so, the `RegisterOpKind::StoArith` arm becomes live. NOT a stub-error pattern (no message surfaced); the dual-path through `StoAdd/Sub/Mul/Div` legacy modals correctly satisfies the v1.1 STO-arith user flow.

## Self-Check: PASSED

Verifications performed:

- File `hp41-cli/tests/phase25_pending_input.rs` exists — confirmed via `ls`.
- Files `hp41-cli/src/app.rs`, `keys.rs`, `ui.rs` modified — confirmed via `git status` + `git diff --stat`.
- Both commits `13b356e` (Task 1) and `f97c192` (Task 2) exist on the worktree branch — confirmed via `git log --oneline`.
- `cargo test -p hp41-cli` — **254 passed**, 0 failed (was 241 after Plan 01; net +13 tests in `phase25_pending_input.rs`).
- `cargo test --workspace` — **1022 passed**, 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `cargo fmt --check` — clean.
- `cargo test -p hp41-cli --test phase25_keyboard` — **12 passed** (Plan 01 regression suite intact).
- `cargo test -p hp41-cli --test phase25_pending_input` — **13 passed**.
- `pending_prompt()` is exhaustive — `awk '/pub fn pending_prompt/,/^}/' hp41-cli/src/ui.rs | grep -c "_ =>"` returns 0; same for `unreachable!`.
- `shifted_key_to_op` signature widened to `&mut App` — `grep -n "pub fn shifted_key_to_op" hp41-cli/src/keys.rs` shows the new signature.
- `App.shift_armed` referenced 16× in `app.rs` (including the new modal-arm references) — `grep -c "self\.shift_armed" hp41-cli/src/app.rs` confirms ≥3 (FlagPrompt, RegisterPrompt, Plan-01 top-level + check_ind_toggle helper + Esc arms).
- NO `pub shift_pending: bool` field on `App` — `grep -c "pub shift_pending: bool" hp41-cli/src/app.rs` returns 0.

All claims in this SUMMARY have been verified before commit.
