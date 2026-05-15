---
phase: 26-gui-integration-and-polish
plan: 04
subsystem: hp41-gui (React frontend only)
gap_closure: true
tags: [gap-closure, modal-routing, frontend-projections, integration-tests]
dependency_graph:
  requires:
    - phase-26 plan-01 PendingInput discriminated union + handleModalKey + MODAL_OPENERS
    - phase-26 plan-02 Display14Seg + displayText derivation
    - phase-26 plan-03 helpOpen state + USER-mode relabel scaffold + KeyDef.keyCode literals
  provides:
    - CR-01 fix: handleClick assign_key branch uses canonical key.keyCode + undefined-keyCode toast
    - CR-02 fix: handleKey short-circuits on helpOpen before resolveKeyId / dispatchKeyId
    - CR-03 fix: handleClick translates on-screen 'enter'/'clx_or_a' → 'Enter'/'Backspace' at the click-router boundary
    - CR-04a fix: displayText derivation prefers display_override over display_str when no modal is open
    - CR-04b fix: new useEffect surfaces calcState.event_buffer entries as toasts
    - CR-05 fix: pending_input.ts single_digit arm enforces op-specific lower bound (Catalog ≥ 1, Tone ≥ 0); MODAL_OPENERS.catalog raised to max:4
    - 13-test integration suite at hp41-gui/src/App.test.tsx with mocked Tauri invoke
    - data-key-id / data-displaytext test-locator attributes (inert React passthrough)
  affects:
    - Phase 26 must-haves count: 7/12 → 12/12 (subject to /gsd-verify-phase 26 --re-verify)
    - FN-POLISH-03 (USER mode relabel): restored from dead-on-arrival to end-to-end working
    - FN-POLISH-02 (`?` overlay search): restored from corrupts-state to owns-focus
    - FN-GUI-05 (display_override + event_buffer): restored from hollow-prop to visible-effect
    - FN-GUI-01 (CATALOG 4 reachability): restored from unreachable to dispatchable
    - FN-GUI-03 (click-only modal Enter submit): restored from broken-on-click-path to working
tech_stack:
  added: []
  patterns:
    - "vi.mock('@tauri-apps/api/core', ...) — first occurrence in the repo; module-scoped mockInvoke + beforeEach mockReset/mockResolvedValue baseline"
    - "Nested act() blocks: fireEvent → setTimeout(0) so React 19 useEffect cleanups commit between interactions"
    - "Inert React data-* attributes as test locators (data-key-id / data-displaytext)"
    - "Click-router translation at the App.tsx handleClick boundary (NOT inside pending_input.ts) — cleaner test surface per plan-checker verdict"
key_files:
  created:
    - hp41-gui/src/App.test.tsx (366 lines, 13 integration tests across 6 describe blocks)
    - hp41-gui/src/test_setup.ts (sets IS_REACT_ACT_ENVIRONMENT=true for React 19)
  modified:
    - hp41-gui/src/App.tsx (Tasks 1+2+3: ~70 line delta — CR-01/02/03/04 + data-displaytext)
    - hp41-gui/src/pending_input.ts (Task 1 CR-05: minDigit guard inside single_digit arm)
    - hp41-gui/src/pending_input.test.ts (Task 1: +8 CR-05 tests)
    - hp41-gui/src/Keyboard.tsx (Task 3: data-key-id test locator on both <g> branches)
    - hp41-gui/vite.config.ts (Task 3: test.setupFiles → ./src/test_setup.ts)
decisions:
  - "CR-01: reject undefined keyCode (variant 'top'/'shift', CHS, xge_y, clx_or_a) with toast 'This key cannot be assigned' — D-07 invariant preserved, no silent discards"
  - "CR-03: translate at click-router boundary in App.tsx (NOT widen predicates in pending_input.ts) — plan-checker verdict, cleaner test surface"
  - "CR-04 BEEP/TONE: surface as toasts (NOT Web Audio API) — D-26.6 v3.x deferral; visible but not audible"
  - "test-only data-key-id / data-displaytext attributes adopted per plan-checker W3 recommendation — robust locators with one line of HTML cost"
  - "act() environment via test_setup.ts: required to flush React 19 useEffect cleanups between fireEvent calls; without this, stale window keydown listeners pile up and multi-dispatch a single keystroke"
metrics:
  duration: ~15 min (3 atomic fix commits + measurement + SUMMARY)
  completed_date: 2026-05-15
  commits: 3 (Tasks 1, 2, 3 — SUMMARY commit follows from this plan's orchestrator)
  tests_added:
    typescript: 21 (+8 CR-05 unit-shape in pending_input.test.ts + 13 integration in App.test.tsx)
  vitest_total_after_plan: 142  # 121 baseline + 8 CR-05 + 13 integration
  blocker_closure:
    cr_01_asn_keycode: closed
    cr_02_help_overlay_keystroke_leak: closed
    cr_03_on_screen_enter_translation: closed
    cr_04a_display_override_consumer: closed
    cr_04b_event_buffer_consumer: closed
    cr_05_catalog_bounds: closed
---

# Phase 26 Plan 04: Gap-Closure Bundle Summary

**One-liner:** Closes the 5 BLOCKER gaps from 26-VERIFICATION.md (CR-01..CR-05)
with surgical edits to App.tsx and pending_input.ts, and ships the
@testing-library/react integration test suite the verifier flagged as missing.
Lifts Phase 26 from 7/12 to 12/12 must-haves verified (pending re-verify).

## Outcome

Plan 26-04 is a gap-closure pass against the verifier's 5 BLOCKER findings.
Every defect was a wiring break BETWEEN correctly-built layers — each layer
was individually unit-tested but the click → modal → dispatch → render
pipeline was never exercised end-to-end. This plan fixes the wiring AND
ships the integration suite that would have caught all 5 defects at CI
time, so a future regression of the same class fails before users see it.

What's now possible that wasn't before:

- **USER mode relabel works end-to-end.** Pre-fix the ASN flow stored
  assignments at layout-computed `row*10+(col+1)` coordinates that no
  `KeyDef.keyCode` advertised — `userKeymap.find` always missed. Now the
  flow uses `key.keyCode` (the hardcoded CLI-canonical literal per
  Keyboard.tsx W9 doc); ASN'd labels surface on the keycap when USER mode
  toggles.
- **`?` overlay search input owns focus.** Pre-fix every keystroke in the
  search box also leaked to `resolveKeyId` → `dispatchKeyId`; typing 's'
  fired `Op::Sqrt`, 'q' fired `Op::Sin`, Backspace fired `Op::Clx` — every
  search keystroke corrupted calculator state. Now `handleKey` short-
  circuits when `helpOpen=true`.
- **On-screen ENTER / ← confirm modals.** Pre-fix the click-router sent
  lowercase `'enter'` / `'clx_or_a'` to `handleModalKey`, but
  `pending_input.ts` only matched `'Enter'` / `'Backspace'`. Click-only
  ASN / CLP / XEQ / GTO / LBL flows were dead. Now the translation
  happens at the `App.tsx::handleClick` boundary; click and physical
  keyboard paths converge on the same modal alphabet.
- **AVIEW / PROMPT / VIEW are visible.** Pre-fix the backend projection
  set `display_override` and `commands.rs` shipped it over IPC, but the
  React render path read only `display_str`. Now the precedence is
  modal preview > `display_override` > `display_str`.
- **BEEP / TONE surface feedback.** Pre-fix `event_buffer` was drained on
  the backend but never consumed in React. Now a `useEffect` iterates the
  array and toasts each entry. v3.x Web Audio API plugs in here without
  changing the projection contract.
- **CAT 4 (XFNS) is reachable.** Pre-fix `MODAL_OPENERS.catalog`
  capped at `max: 3` (CAT 4 unreachable); `single_digit` arm had no
  lower bound (CAT 0 dispatched-then-rejected backend). Now Catalog
  accepts 1..=4 inclusive, matching `hp41-core::op_catalog`.

## BLOCKER Closure Table

| BLOCKER | File | Line range | Fix | Test that locks it |
|---------|------|------------|-----|--------------------|
| CR-01 (ASN keycode) | hp41-gui/src/App.tsx | ~315-330 in handleClick | `key.keyCode` instead of `key.row * 10 + (key.col + 1)`; undefined-keyCode → toast | App.test.tsx A2 (`asn_25_TEST` NOT `asn_23_TEST`), A3 (CHS toast) |
| CR-02 (helpOpen leak) | hp41-gui/src/App.tsx | ~427-429 in handleKey | `if (helpOpen) return;` before resolveKeyId | App.test.tsx B1 (no dispatches during search), B2 (sqrt fires after Esc) |
| CR-03 (on-screen ENTER) | hp41-gui/src/App.tsx | ~291-340 in handleClick | `'enter' → 'Enter'`, `'clx_or_a' → 'Backspace'` translation at click-router | App.test.tsx C1 (asn_25_TEST on click ENTER), C2 (← pops xeq_name) |
| CR-04a (display_override) | hp41-gui/src/App.tsx | ~528-530 displayText | `pendingInput ? renderModalLcd(pi) : (display_override ?? display_str)` | App.test.tsx D1 (HELLO renders), D2 (3.1416 fallback) |
| CR-04b (event_buffer) | hp41-gui/src/App.tsx | ~480-491 new useEffect | iterate event_buffer; showToast(line) per entry | App.test.tsx D3 (BEEP surfaces in `.toast`) |
| CR-05 (Catalog bounds) | hp41-gui/src/pending_input.ts | 294-318 single_digit arm | op-specific `minDigit = pending.op === 'Catalog' ? 1 : 0`; reject `digit < minDigit` | pending_input.test.ts (8 new tests); App.test.tsx E1/E2 |
| CR-05 (Catalog max) | hp41-gui/src/App.tsx | 158 MODAL_OPENERS.catalog | `max: 4` (was `max: 3`) | App.test.tsx E1 (catalog_4 dispatches) |

## Integration Test Inventory

`hp41-gui/src/App.test.tsx` ships 13 tests in 6 describe blocks, all driving
the full `<App />` render via `@testing-library/react` against a mocked
`@tauri-apps/api/core`. The mockInvoke pattern is the first occurrence in
the repo; future Phase 27 Playwright E2E (FN-QUAL-05) layers on top.

| Group | Test | Asserts |
|-------|------|---------|
| A (CR-01) | A1 | SHIFT+XEQ opens ASN; click SIN advances to `assign_label`; LCD `ASN _`; no dispatch fired |
| A (CR-01) | A2 | ASN + SIN (kc=25) + 'TEST' + click ENTER → `dispatch_op({keyId:'asn_25_TEST'})`, NOT `asn_23_TEST` |
| A (CR-01) | A3 | Click CHS inside ASN → toast 'cannot...assign'; no `asn_*` dispatch; modal stays open |
| B (CR-02) | B1 | helpOpen=true → no `sqrt`/`sin`/`rdn`/`tan`/`clx`/digit dispatches during keystroke sequence |
| B (CR-02) | B2 | Esc closes overlay → 's' resumes dispatching `sqrt` |
| C (CR-03) | C1 | assign_label acc=TEST + click ENTER → `asn_25_TEST` dispatches |
| C (CR-03) | C2 | xeq_name acc=ABC + click ← → LCD `XEQ AB_`; no clx/alpha_clear dispatch |
| D (CR-04) | D1 | display_override=HELLO → displayText='HELLO' |
| D (CR-04) | D2 | display_override=null + display_str=3.1416 → displayText='3.1416' |
| D (CR-04) | D3 | event_buffer=['BEEP'] → `.toast` contains 'BEEP' |
| E (CR-05) | E1 | SHIFT+ENTER opens Catalog; press '4' → `catalog_4` dispatches |
| E (CR-05) | E2 | Catalog rejects '0' (lower-bound) and '5' (upper-bound); no dispatch; modal stays open |
| F (CR-01+CR-03) | F1 | Full ASN click flow → toggle USER → STO keycap shows 'TEST' (relabel works end-to-end) |

## Source-Grep Verification

```bash
$ rg -c "key.keyCode" hp41-gui/src/App.tsx
2  # both inside handleClick (lines 318, 326)

$ rg -c "key.row \* 10" hp41-gui/src/App.tsx
0  # the buggy formula is eliminated

$ rg -c "if (helpOpen) return" hp41-gui/src/App.tsx -F
1  # new short-circuit in handleKey (around line 428)

$ rg -c "display_override \?\?" hp41-gui/src/App.tsx
1  # displayText derivation (around line 529)

$ rg -E "useEffect[^)]*\{[^}]*event_buffer" hp41-gui/src/App.tsx
# match around line 480 — new useEffect consumer

$ rg -cE "effectiveId === 'enter'|effectiveId === 'clx_or_a'" hp41-gui/src/App.tsx
2  # CR-03 translation branches

$ rg -c "max: 4" hp41-gui/src/App.tsx
1  # MODAL_OPENERS.catalog (line 158)

$ rg -c "max: 3" hp41-gui/src/App.tsx
0  # old ceiling gone

$ rg -c "minDigit" hp41-gui/src/pending_input.ts
2  # const declaration + < minDigit guard inside single_digit arm
```

All ten verification checks from PLAN.md `<verification>` pass.

## All Gates Green

| Gate | Status | Detail |
|------|--------|--------|
| `cd hp41-gui && npx tsc --noEmit` | OK | clean (zero TypeScript errors) |
| `cd hp41-gui && npx vitest run` | OK | **142 / 142 passed** (was 121 baseline; +8 CR-05 + +13 integration) |
| `just gui-check` | OK | cargo check on hp41-gui/src-tauri clean |
| `just gui-ci` | OK | npm install → tsc --noEmit → cargo test (58 + 0 + 3 + 0 = 61 passed) → cargo build --release (clean) |
| SC-4 invariant grep | OK | `rg -rE "fn op_(add\|sub\|...)" hp41-gui/src-tauri/src/` returns 0 matches |
| `cargo test -p hp41-gui-tauri` | OK | 61 / 61 passing (zero backend changes from Phase 26 baseline) |
| Build artifact size delta | n/a | Only pure-function fix + new useEffect; no new dependencies; bundle delta < 5 KB |

## SC-4 Invariant Preservation

Plan 26-04 is FRONTEND-ONLY. No file under `hp41-gui/src-tauri/` was modified.
The SC-4 invariant (no calculator logic in the GUI backend) is preserved by
exclusion:

```bash
$ rg -rEn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/
# (no output — zero matches)
```

Likewise zero changes to `hp41-core/` (frozen since Plan 25-01) and `hp41-cli/`
(Phase 25 shipped). All fixes live inside `hp41-gui/src/`.

## No-Deviation Note

No new `Op` variants. No new `CalcState` fields. No new IPC commands. No new
permission TOMLs. Save-file backward compatibility unchanged. The fix surface
is the minimum needed to close the 5 BLOCKERs: 2 source files touched (App.tsx,
pending_input.ts) plus 2 test files (App.test.tsx new, pending_input.test.ts
extended) plus 2 supporting one-liners (Keyboard.tsx data-key-id locator,
vite.config.ts test.setupFiles).

The 8 WARNING / INFO findings from 26-REVIEW.md (WR-02 isPrintableChar
widening for `<>?`, IN-01..04 informationals) remain explicitly out of scope
per PLAN frontmatter — those are Phase 27 territory.

## Re-Verification Trigger

Phase 26 verification should now re-run with
`/gsd-verify-phase 26 --re-verify` and lift the score from 7/12 to 12/12. The
7 `human_verification` items in 26-VERIFICATION.md (visual LCD sanity, modal
preview rendering, help overlay search behavior, USER mode relabel,
`p`/`P` physical-key remap, AVIEW visible effect, BEEP/TONE feedback) are
still pending manual sanity check by the developer — but the three that
previously WOULD HAVE FAILED (help overlay search, USER mode relabel, AVIEW
visible) should now succeed end-to-end against the production code.

## Deviations from Plan

### Auto-fixed Issues

1. **[Rule 3 — Blocking] React 19 act() environment setup.** The first run
   of App.test.tsx flooded stderr with `The current testing environment is
   not configured to support act(...)` warnings AND test E2 leaked 16 stale
   dispatch_op calls per keystroke. Root cause: jsdom + React 19 + Vitest
   do not commit useEffect cleanups synchronously inside `fireEvent`; the
   window keydown listener swap (handleKey useEffect with `[handleKey]`
   deps) leaves stale listeners. Resolution: created
   `hp41-gui/src/test_setup.ts` setting
   `globalThis.IS_REACT_ACT_ENVIRONMENT = true`, registered via
   `vite.config.ts test.setupFiles`. Also rewrote `clickKey` / `pressKey`
   helpers to wrap each interaction in nested `act()` blocks
   (`fireEvent → setTimeout(0)`) so effects flush between interactions.
   Files touched: `hp41-gui/src/test_setup.ts` (new),
   `hp41-gui/vite.config.ts` (+1 line `setupFiles: ['./src/test_setup.ts']`).
   No deviation from the user-observable plan outcome — both files were
   plan-anticipated (mock infrastructure scope).

### Authentication Gates

None encountered. Plan 26-04 has zero external dependencies, no API keys,
no CLI logins.

## Cross-References

- **26-VERIFICATION.md gaps** — every BLOCKER closed maps 1:1 to a `gaps[]`
  entry by ID (CR-01..CR-05). The verifier's "no integration test exercises
  the end-to-end click → modal → dispatch → render flow" recommendation is
  closed by `hp41-gui/src/App.test.tsx`.
- **26-REVIEW.md** — 5 BLOCKER findings addressed; 8 WARNING/INFO findings
  deferred to Phase 27 per PLAN frontmatter.
- **26-01-SUMMARY.md / 26-02-SUMMARY.md / 26-03-SUMMARY.md** — the modal
  infrastructure (Plan 26-01), 14-seg LCD (Plan 26-02), and helpOpen + USER
  scaffold (Plan 26-03) that this plan repaired the wiring around.
- **Phase 27 FN-QUAL-05** — a future Playwright E2E smoke test would catch
  this class of regression at the booted-Tauri-app level. Plan 26-04 ships
  the Vitest+@testing-library/react integration suite that catches it at
  the render-DOM level (a strictly weaker but vastly faster sibling).

## Self-Check: PASSED

**Files created (verified):**
- `hp41-gui/src/App.test.tsx` ✓ FOUND (366 lines, 13 integration tests)
- `hp41-gui/src/test_setup.ts` ✓ FOUND

**Files modified (verified):**
- `hp41-gui/src/App.tsx` ✓ MODIFIED (Tasks 1+2+3)
- `hp41-gui/src/pending_input.ts` ✓ MODIFIED (Task 1)
- `hp41-gui/src/pending_input.test.ts` ✓ MODIFIED (Task 1)
- `hp41-gui/src/Keyboard.tsx` ✓ MODIFIED (Task 3 — data-key-id locators)
- `hp41-gui/vite.config.ts` ✓ MODIFIED (Task 3 — test.setupFiles)

**Commits (verified via `git log --oneline -5`):**
- `9688da6 fix(26-04): CR-05 — Catalog single_digit bounds (1..=4) + MODAL_OPENERS max:4` ✓ FOUND
- `200cb41 fix(26-04): CR-01/CR-02/CR-03/CR-04 — App.tsx wiring fixes (single edit pass)` ✓ FOUND
- `a9fd2b5 test(26-04): integration suite for CR-01..CR-05 with mocked Tauri invoke` ✓ FOUND
