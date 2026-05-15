---
phase: 26-gui-integration-and-polish
plan: 01
subsystem: hp41-gui (Tauri + React frontend)
tags: [modal-architecture, key-wiring, IPC, CalcStateView, parity-D-25.6]
dependency_graph:
  requires:
    - phase-25 hp41-cli PendingInput hybrid struct-variants (Plan 25-02)
    - phase-20..24 hp41-core Op variants (Pi, polar conversions, flags, display,
      sound, program control, memory, catalog, ASN, ALPHA ops, *Ind family)
  provides:
    - frontend-owned PendingInput discriminated union (14 variants)
    - handleModalKey pure-function modal state machine
    - renderModalLcd LCD preview emitter (D-26.3)
    - CalcStateView projections: user_keymap, flags, display_override, event_buffer
    - extended key_map::resolve covering ~30 new bare ops + ~20 new parameterized prefixes
    - W3 audit infrastructure: key_defs_ids.ts + Rust-side mirror test
  affects:
    - Plan 26-02 (14-seg LCD): consumes `displayText` derivation via <Display14Seg text={...} />
    - Plan 26-03 (polish bundle): consumes user_keymap projection for USER-mode relabel
tech_stack:
  added:
    - vitest@4.1.6 (devDependency for pure-function modal tests)
  patterns:
    - "Frontend-owned UI state with backend round-trip on final dispatch (D-26.1)"
    - "Discriminated TypeScript union for one-of-N modal states (D-26.4)"
    - "More-specific-first prefix ordering in resolve_parameterized (Pitfall 3)"
    - "Drain-before-from_state for transient buffers (Pitfall 1, extended to event_buffer)"
key_files:
  created:
    - hp41-gui/src/pending_input.ts (modal state engine + LCD preview)
    - hp41-gui/src/pending_input.test.ts (42 Vitest unit tests)
    - hp41-gui/src/key_defs_ids.ts (W3 audit source-of-truth)
  modified:
    - hp41-gui/src-tauri/src/key_map.rs (+699 lines, -42)
    - hp41-gui/src-tauri/src/types.rs (+97 lines, -10)
    - hp41-gui/src-tauri/src/commands.rs (+11 lines, -5)
    - hp41-gui/src/App.tsx (+135 lines, -13)
    - hp41-gui/package.json (vitest devDep + "test" script)
decisions:
  - D-26.1 (frontend-owned modal state; no new IPC surface)
  - D-26.2 (IND-toggle via shift-0 reuses shiftActive bit)
  - D-26.3 (modal preview replaces LCD content during accumulation)
  - D-26.4 (single discriminated TS union; ALL 14 variants ship)
  - D-26.5 (handleClick intercepts modal openers BEFORE invokeForKey)
  - D-26.11 (CalcStateView gains 4 projections under ≤500 byte budget)
  - W3 (KEY_DEFS audit driven by exported constants + Rust-side mirror, no drift)
  - B1 revision (`direct` variant: 4 conditional-test prompts dispatch immediately)
  - B2 revision (`single_digit` variant merges Tone+Catalog with op+max discriminator)
  - B3 revision (DEL clamps at u8::MAX with explicit GuiError + "DEL ERR" preview)
metrics:
  duration: ~75 min (3 atomic commits + measurement + cleanup commit)
  completed_date: 2026-05-15
  commits: 4
  tests_added:
    rust: 6 (test_new_v22_named_op_resolvers, test_new_v22_parameterized_prefixes,
          test_del_clamps_at_u8_max, test_more_specific_prefix_wins,
          test_keyboard_skin_ids_resolve_or_are_modal_openers, plus 5 new types::tests)
    typescript: 42 (Vitest — handleModalKey transitions, renderModalLcd previews,
                    W3 audit shape invariants)
  budget_actual:
    calcstateview_empty: 337 bytes (vs 500-byte FN-GUI-05 budget; ~33% headroom)
    calcstateview_realistic_load: 401 bytes (5 ASN entries + 3 set flags; ~20% headroom)
---

# Phase 26 Plan 01: Modal Architecture & Key Wiring Summary

**One-liner:** Wires every Phase 20-24 HP-41CV ROM op into hp41-gui's
`key_map.rs` resolver (~80 new variants) and ships the React-side
`PendingInput` discriminated union + `handleModalKey` state machine that
intercepts the 13 prompt-id modals, closing the CLI ↔ GUI parity gap
established by D-25.6.

## Outcome

Phase 26 Plan 01 delivers the modal infrastructure that Plans 26-02 (14-seg
LCD) and 26-03 (polish bundle) depend on. Every HP-41CV ROM op now resolves
in the GUI either as a real `Op::*` variant (bare or parameterized prefix)
or by opening a React modal that accumulates input and dispatches the final
parameterized id at end-of-accumulation. The CLI Phase 25 user-observable
behavior is mirrored bit-for-bit: same logical states, same one-shot SHIFT
lifetime, same Esc-clears-both semantics, same IND-toggle via shift-0, same
hardware-faithful tuple-match decision for direct vs. IND dispatch.

## Bare-Op Resolvers Added (count = 31 across 5 sections)

**Phase 20 — math + stack (10):**
`pi`, `polar_to_rect`, `rect_to_polar`, `rnd`, `frc`, `mod_op`, `abs`, `fact`,
`sign`, `r_up`

**Phase 21 — display control + sound (6):**
`aview`, `prompt`, `aon`, `aoff`, `cld`, `beep`

**Phase 22 — program control + memory (6):**
`stop`, `pse`, `ins`, `cla`, `clst`, `pack`

**Phase 23 — ALPHA-register operations (4):**
`atox`, `xtoa`, `arot`, `posa`

**Phase 25 D-25.7 — keyboard-bound conditional tests (4):**
`x_eq_y`, `x_le_y`, `x_gt_y`, `x_eq_0` (the only 4 conditional tests on the
physical HP-41CV keyboard; the other 8 route through XEQ-by-Name modal).

**(1 already shipped) v2.1:** `xge_y`

## Parameterized Prefixes Added

**IND-bearing prefixes (must strip BEFORE non-IND counterpart):**
- `sto_ind_NN` → `Op::StoInd(NN)`
- `rcl_ind_NN` → `Op::RclInd(NN)`
- `isg_ind_NN` / `dse_ind_NN` → `Op::IsgInd(NN)` / `Op::DseInd(NN)`
- `sf_ind_NN` / `cf_ind_NN` → `Op::SfFlagInd(NN)` / `Op::CfFlagInd(NN)`
- `fs_ind_NN` / `fc_ind_NN` / `fs_c_ind_NN` / `fc_c_ind_NN` → `Op::FlagTestInd { kind, ind_reg }`
- `view_ind_NN` / `arcl_ind_NN` / `asto_ind_NN` → `Op::ViewInd/ArclInd/AstoInd`
- `sto_arith_<op>_ind_NN` → `Op::StoArithInd(NN, kind)`

**Non-IND prefixes:**
- `sf_NN`, `cf_NN`, `fs_NN`, `fc_NN`, `fs_c_NN`, `fc_c_NN` → flag ops + `Op::FlagTest`
- `view_NN`, `arcl_NN`, `asto_NN`, `tone_N`, `catalog_N`, `size_NNN`
- `gto_ind_NN`, `xeq_ind_NN` → indirect program control
- `clp_LABEL`, `asn_NN_NAME` → label-bearing ops (resolve_asn helper)

**Example resolutions** (all under test):
| Input | Op |
|-------|-----|
| `sto_05` | `Op::StoReg(5)` |
| `sto_ind_05` | `Op::StoInd(5)` |
| `sf_c_ind_12` | `Op::FlagTestInd { kind: IsSetThenClear, ind_reg: 12 }` |
| `clp_MYPRG` | `Op::Clp("MYPRG")` |
| `asn_22_TEST` | `Op::Asn { name: "TEST", key_code: 22 }` |
| `sto_arith_plus_ind_07` | `Op::StoArithInd(7, StoArithKind::Add)` |

## BLOCKER B3 — DEL u8 Clamp Resolution

`hp41-core::ops::Op::Del` takes a `u8` field (line 397 of `hp41-core/src/ops/mod.rs`).
HP-41CV hardware natively accepts DEL 000-999; the u8 cap is a documented
v2.2 divergence (widening deferred to v3.x per the phase boundary).

**Two-layer surfacing of the divergence:**

1. **Backend (key_map.rs):** `del_<NNN>` parses the accumulator as u16. If
   `n > 255`, returns `Err(GuiError { message: "DEL value must be 0-255
   (hp41-core Op::Del field is u8 — Phase 26 divergence from HP-41 hardware
   0-999, deferred to v3.x). Got: <N>" })`. The frontend surfaces this as a
   2-second toast — never silent (D-07 invariant).

2. **Frontend (renderModalLcd):** the DEL modal preview renders `"DEL ERR"`
   instead of the numeric preview when the accumulator has 3 digits AND
   parses to > 255 — surfaces the divergence to the user BEFORE dispatch.

Locked by:
- `test_del_clamps_at_u8_max` (Rust) — asserts boundary `del_255` → Ok, `del_256` → Err with "0-255" in message
- `del modal value 256 renders DEL ERR in LCD preview` (Vitest) — verifies the frontend preview path

## D-26.11 Budget Audit

**Measured actual JSON sizes** (recorded as inline test comments + this summary
for Phase 27 trend tracking):

| Scenario | Bytes | Budget | Headroom |
|----------|------:|------:|---------:|
| Empty `CalcState` (no ASN, no flags) | **337** | 500 | ~33% |
| Realistic load: 5 ASN entries + 3 set flags | **401** | 500 | ~20% |

The 4 new projections (`user_keymap`, `flags`, `display_override`,
`event_buffer`) add ~60-100 bytes over the v2.1 baseline (~270-300 bytes
empirically). Plan 26-03 will consume `user_keymap` for the USER-mode
per-key relabel without requiring further IPC schema changes.

## KEY_DEFS Audit (W3)

**Approach:** instead of a hand-curated whitelist, this plan introduces
`hp41-gui/src/key_defs_ids.ts` exporting two const-tuples (`KEY_DEFS_PRIMARY_IDS`,
`KEY_DEFS_SHIFTED_IDS`) plus `KEY_DEFS_HANDLED_OUTSIDE_RESOLVE` for the
sst/bst/r_s/shift/`""`/clx_or_a/e exceptions. The Rust-side mirror in
`key_map.rs::tests::test_keyboard_skin_ids_resolve_or_are_modal_openers`
hand-lists the same ids with a sentinel comment.

**Drift detection:** when a key is added to `Keyboard.tsx` `KEY_DEFS`, BOTH
the TS source-of-truth AND the Rust-side mirror must be updated. The Rust
test asserts every id either resolves to an `Op`, hits the
`planned for a future phase` stub-error arm, or is in
`HANDLED_OUTSIDE_RESOLVE`. A regression that adds a Keyboard.tsx id but
forgets the TS / Rust constants fails `just gui-check` immediately.

**KEY_DEFS additions/fixes during audit:** NONE. The Keyboard.tsx 39-entry
grid already covers every keyboard-bound v2.2 ROM op via either the primary
or shifted slot (Phase 19 shipped this layout in v2.1; Phase 26 adds no new
visible keys — modal openers reuse existing shifted slots like
`shifted: { id: 'sf_prompt', label: 'SF' }`).

## FlagTestKind Family Audit (W1)

The full `FlagTestKind` enum has 4 variants (`IsSet`, `IsClear`,
`IsSetThenClear`, `IsClearThenClear`). Crossing with the SF/CF set+clear
ops gives 6 logical keyboard-bound variants (SF, CF, FS?, FC?, FS?C, FC?C).

**Keyboard-bound (3, via Keyboard.tsx row 5 shifted slots):**
- `sf_prompt` (SF — sets flag)
- `cf_prompt` (CF — clears flag)
- `fs_prompt` (FS? — IsSet test)

**XEQ-by-Name-reachable only (3):**
- FC? (IsClear) — no keyboard slot
- FS?C (IsSetThenClear) — no keyboard slot
- FC?C (IsClearThenClear) — no keyboard slot

This matches the Phase 25 D-25.7 / D-25.9 pattern (only the 4 conditional
tests on the physical HP-41CV row 5-8 left edge are keyboard-bound; the
other 8 conditionals + the 3 *?C flag tests route through XEQ-by-Name).
`test_modal_prompt_ids_are_stubs_for_now` id list stays unchanged — no new
prompt ids to add to the test.

## SHIFT-during-modal Cross-Layer Flow (W2)

The IND-toggle path (D-26.2) is hardware-faithful per HP-41C/CV Quick
Reference Guide p.14:

1. User opens a register or flag modal (e.g. clicks SHIFT+STO → SHIFT
   toggles `shiftActive=true` then resets on `consumesShift`, opening
   `RegisterPrompt { Sto }` with `acc=""`, `ind=false`).
2. User clicks SHIFT again — `handleClick` early-returns through the
   `key.id === 'shift'` branch, toggling `shiftActive=true` WITHOUT
   invoking `handleModalKey` (cross-layer flow handled in App.tsx, not
   pending_input.ts).
3. User clicks "0" — this time the click DOES route through
   `handleModalKey('0', pending, shiftActive=true)`. The function detects
   `shiftActive && key === '0'`, toggles `pending.ind`, and returns
   `consumesShift=true`. `handleClick` applies `setShiftActive(false)` on
   the consumed bit.
4. Modal preview updates to `"STO IND __"`.
5. User clicks "0", "5" — end-of-2-digit dispatch fires `sto_ind_05`
   (NOT `sto_05`) per the tuple-match decision in handleModalKey.

The SHIFT bit is REUSED — no separate `shiftPending` state field. Bit-for-bit
parity with CLI per D-25.6 / D-26.2.

Locked by:
- Vitest `IND-toggle via shift-0 sets ind=true AND consumesShift=true`
- Vitest `IND-toggle does NOT append 0 to acc (W2 verification)`
- Vitest `register modal with ind=true dispatches sto_ind_NN`

## Planner Discretion Decisions

- **`regPrefix` / `flagPrefix` / `flagDisplayLabel` helpers** — adopted
  switch-statement pure functions (exhaustive over the TS union; tsc
  enforces). Alternative considered: lookup objects. Switches are more
  greppable when chasing CLI ↔ GUI prefix drift.
- **`xeq_name.dispatchPrefix: 'xeq' | 'gto' | 'lbl'`** — added a discriminator
  to the `xeq_name` variant rather than introducing parallel `gto_name` /
  `lbl_name` variants. Three modals share the same accumulator + Enter
  semantics; one variant + a 3-value field is cleaner than three variants
  with identical shape.
- **`makeKeyCodeMagic` / `parseKeyCodeMagic` helpers** — the `assign_key`
  modal needs a keycode from the caller (handleClick has the clicked
  KeyDef in scope; handleModalKey does not). Rather than passing a
  separate `keyCode` argument that's `null` for every other variant, the
  caller encodes the keycode into the key-string via a magic prefix.
  Mechanically simple and inspectable in tests.
- **TS-side handleModalKey returns ModalKeyResult struct** (B6) — the
  3-tuple of `(nextPending, dispatchId, consumesShift)` carries cleanly
  through React state updates without ambiguity. Alternative
  considered: separate handler functions per modal kind. Single function
  with switch is more compact and the exhaustiveness check from the
  discriminated union catches missed cases at compile time.

## Deviations from Plan

### Auto-fixed Issues

None. The plan executed exactly as written modulo two minor adjustments
documented inline:

1. **renderModalLcd `flagDisplayLabel` helper** — initial draft used a
   `.replace('Query', '?').replace('Clear', 'C')` chain which produced
   `"Fs?"` (incorrect case). Replaced with an explicit switch returning
   `"FS?"` / `"FC?"` / `"FS?C"` / `"FC?C"`. Caught during inline review
   before commit.

2. **vitest dev-dep installation** — the plan specified Vitest tests but
   the repo didn't yet have vitest installed. Added `vitest@4.1.6` as a
   devDependency and added a `"test": "vitest run"` script in
   `hp41-gui/package.json`. No other testing-library dependencies
   needed — handleModalKey + renderModalLcd are pure functions tested in
   isolation (no React rendering).

### Authentication Gates

None encountered.

## SC-4 Invariant Verification

```bash
$ grep -rEn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/
# (no output — zero matches)
```

No calculator/math logic added to `hp41-gui/src-tauri/`. Pure dispatch
routing and DTO projection.

## All Gates Green

| Gate | Status | Detail |
|------|--------|--------|
| `cargo test -p hp41-gui-tauri --lib` | ✅ | 58 passed |
| `cargo test -p hp41-gui-tauri` (all targets) | ✅ | 61 passed (4 suites) |
| `cargo clippy --all-targets -D warnings` | ✅ | No issues found |
| `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` | ✅ | clean |
| `npm run test` (Vitest) | ✅ | 42 / 42 passed |
| `npx tsc --noEmit` | ✅ | clean |
| `just gui-check` | ✅ | clean |
| SC-4 invariant grep | ✅ | zero matches |

## Cross-References

- **Plan 26-02 (14-seg LCD)** consumes the `displayText` derivation
  (`pendingInput ? renderModalLcd(...) : calcState.display_str`) which is
  already wired in App.tsx. Plan 26-02 swaps the inner `<div className="display">`
  content from `{displayText}` to `<Display14Seg text={displayText} />`.
- **Plan 26-03 (polish bundle)** reads `calcState.user_keymap` to pass to
  `<Keyboard userKeymap={...} />` for the USER-mode per-key relabel. The
  TS interface mirror already exposes the field per BLOCKER B5.

## Self-Check: PASSED

**Files created (verified):**
- `hp41-gui/src/pending_input.ts` ✓ FOUND
- `hp41-gui/src/pending_input.test.ts` ✓ FOUND
- `hp41-gui/src/key_defs_ids.ts` ✓ FOUND

**Commits (verified via `git log --oneline -5`):**
- `eecb728 feat(26-01): wire v2.2 ROM ops into hp41-gui key_map resolver` ✓ FOUND
- `322bd0c feat(26-01): extend CalcStateView with D-26.11 projections + event_buffer drain` ✓ FOUND
- `46eb3d5 feat(26-01): wire PendingInput modal infrastructure in hp41-gui frontend` ✓ FOUND
- Plus a small follow-up commit recording measured byte counts as inline test comments.
