---
phase: 26-gui-integration-and-polish
verified: 2026-05-15T08:30:00Z
status: gaps_found
score: 7/12 must-haves verified (3 partial, 2 failed via review findings)
overrides_applied: 0
gaps:
  - truth: "ASN flow opens AssignKey modal -> press a key -> AssignLabel modal -> type text -> Enter dispatches asn_NN_NAME parameterized id"
    status: failed
    reason: "Two compounding defects make the ASN flow non-functional end-to-end. (1) CR-01: hp41-gui/src/App.tsx:297 computes the assign-target keycode as `key.row * 10 + (key.col + 1)` — layout coordinates — instead of `key.keyCode` (the canonical HP-41 literal from CLI). E.g. clicking SIN (row 2, col 2) stores at keycode 23 (`2*10+3`), but the canonical SIN code is 25; STO stores at 33, canonical is 22. The KEY_DEFS doc comment at Keyboard.tsx:36-52 was authored precisely to prevent this W9 violation. (2) CR-03: hp41-gui/src/pending_input.ts:372 checks `key === 'Enter'` (capitalized DOM-event spelling), but a click on the on-screen ENTER key resolves to id `'enter'` (lowercase). Clicking ENTER inside an open assign_label modal does nothing — the modal can only be confirmed via the physical keyboard. Together these defects break the truth on the click-only path AND would still store at a wrong keycode that the USER-mode relabel resolver never finds."
    artifacts:
      - path: "hp41-gui/src/App.tsx"
        issue: "Line 297 — `makeKeyCodeMagic(key.row * 10 + (key.col + 1))` computes layout-relative coord; must use `key.keyCode`. Reject undefined keyCode with a toast."
      - path: "hp41-gui/src/pending_input.ts"
        issue: "Lines 244, 326, 372 — `key === 'Enter'` excludes click-time `'enter'` id. Same for Backspace vs clx_or_a."
    missing:
      - "Replace `key.row * 10 + (key.col + 1)` with `key.keyCode` (the canonical hardware literal) in App.tsx:297"
      - "Short-circuit assign_key when key.keyCode is undefined (top row, SHIFT, CHS, XGE Y, CL X/A) with a toast"
      - "Translate effectiveId 'enter' → 'Enter' (and 'clx_or_a' → 'Backspace') at the App.tsx click-router boundary before calling handleModalKey, OR widen the modal predicates with an isEnter() helper"
      - "Vitest test asserting click-time on-screen ENTER inside an assign_label modal dispatches asn_<keyCode>_<NAME>"
      - "Vitest test asserting key.keyCode (not row*10+col) is used as the ASN target"

  - truth: "Pressing `?` opens a keyboard shortcut overlay populated from a TypeScript port of help_data.rs"
    status: partial
    reason: "CR-02: The overlay opens correctly, but its search input is unusable because window-level `handleKey` does NOT gate on `helpOpen`. Only Escape and '?' are short-circuited when helpOpen=true; every other physical key continues to flow through `resolveKeyId` → `dispatchKeyId`. Concrete consequences: typing `s`,`q`,`r`,`t` to search dispatches `Op::Sqrt`, `Op::Sin`, `Op::Rdn`, `Op::Tan` in the background; digit keys push to entry_buf; Backspace dispatches clx and destroys X. The overlay 'works' visually but cannot be used to search without corrupting calculator state, violating the must_have spirit ('opens a keyboard shortcut overlay')."
    artifacts:
      - path: "hp41-gui/src/App.tsx"
        issue: "handleKey at lines 346-410 has no `if (helpOpen) return;` short-circuit before resolveKeyId; only Escape and '?' are gated."
    missing:
      - "Insert `if (helpOpen) return;` immediately after the Tab branch in handleKey (around line 386) so the overlay input owns focus without leaking keystrokes to dispatch_op"
      - "Vitest integration test: open help overlay, type 's' in search input, assert no Op::Sqrt dispatch fires"

  - truth: "USER mode shows current key assignments overlaid on the skin (FN-POLISH-03)"
    status: failed
    reason: "Compound failure with CR-01. KEY_DEFS carries the correct hardcoded keyCode literals from CLI canonical mapping (verified: 32 main-grid entries have keyCode, e.g. SIN=25, STO=22, RCL=23) AND Keyboard.tsx:204-205 does `userKeymap.find(([code]) => code === keyCode)` correctly. BUT because the ASN click handler stores assignments at LAYOUT coordinates (key.row*10+(key.col+1) — see CR-01), `userKeymap` never contains a code that matches any KeyDef.keyCode. The USER-mode relabel resolves to null for every key the user could have ASN'd. The two halves of the feature are individually correct but never connect."
    artifacts:
      - path: "hp41-gui/src/App.tsx"
        issue: "Line 297 — assignments stored at layout coords break the USER-mode lookup contract documented in Keyboard.tsx:36-52"
    missing:
      - "Fix CR-01 (assign keycode = key.keyCode) — without this, USER mode is dead-on-arrival"
      - "Vitest end-to-end test: open ASN modal, click STO (keyCode 22), type 'TEST', confirm; then toggle USER mode and assert resolveUserLabel(22) returns 'TEST'"

  - truth: "FN-GUI-05: CalcStateView extended with display_override and event_buffer fields wired through to the UI"
    status: failed
    reason: "CR-04: The backend projection is correct — `display_override: Option<String>` and `event_buffer: Vec<String>` ship in CalcStateView (types.rs:47-57), drain/projection happens on every IPC response (commands.rs has 5 `event_buffer.drain` call sites), JSON budget honored at 401 bytes for realistic load. The TS interface mirror in App.tsx:36-39 includes both fields. BUT they are NEVER read in App.tsx. The `displayText` derivation at App.tsx:463-465 references only `pendingInput` and `display_str`, ignoring `display_override`. Phase 21 ROM ops AView/Prompt/View(n) — newly wired through key_map per Plan 26-01 — produce no visible effect: backend sets display_override, projection serializes it, frontend drops it. Same for event_buffer: Beep/Tone(n) push lines, drain happens, frontend never consumes — no audio, no toast, no visual feedback. Bare-op resolvers in key_map for `beep`, `aview`, `prompt` are functional but user-invisible — violating FN-GUI-05 (fields exist but not wired) AND undermining FN-GUI-03 (no `unknown key` toasts but no observable behavior either)."
    artifacts:
      - path: "hp41-gui/src/App.tsx"
        issue: "Line 463-465 displayText ignores display_override; no useEffect consumes event_buffer for toast/audio"
    missing:
      - "Wire displayText: `pendingInput ? renderModalLcd(pendingInput) : (calcState.display_override ?? calcState.display_str)`"
      - "Add useEffect dependent on calcState that consumes event_buffer into showToast (or a dedicated event log / audio feedback for Tone/Beep)"
      - "Vitest test: dispatch aview, assert displayText reflects display_override"
      - "Vitest test: dispatch tone_5, assert toast/event log surfaces a TONE 5 entry"

  - truth: "Every HP-41CV ROM op variant added in Phases 20-24 resolves successfully via key_map::resolve or key_map::resolve_parameterized — only v3.x module-Pac names remain in the stub-error arm"
    status: partial
    reason: "key_map.rs::resolve and ::resolve_parameterized correctly cover ~80 new bare-op resolvers + ~20 new parameterized prefixes with more-specific-first ordering for IND variants (verified: `strip_prefix(\"sto_ind_\")` precedes `strip_prefix(\"sto_\")` at line 175 vs subsequent fallthrough; same for rcl/isg/dse/sf/cf/fs/view/arcl/asto/sto_arith). Phase 20-24 bare ops (Pi, PolarToRect, RectToPolar, Rnd, Frc, Mod, Abs, Fact, Sign, RUp, AView, Prompt, AOn, AOff, Cld, Beep, Stop, Pse, Ins, Cla, Clst, Pack, Atox, Xtoa, Arot, Posa) and the 4 keyboard-bound conditional tests resolve. BUT: CR-05 — `catalog_N` parameterized prefix accepts `N=0` (which hp41-core rejects with InvalidOp) and `N=4` (XFNS, which hp41-core ACCEPTS). The frontend MODAL_OPENERS opens Catalog with max:3 — Catalog 4 is unreachable from the GUI, and Catalog 0 dispatches and surfaces a toast error. The truth is partially violated: Catalog 4 is a valid HP-41CV ROM op but no user click can reach it."
    artifacts:
      - path: "hp41-gui/src/App.tsx"
        issue: "Line 156 — `catalog: () => ({ kind: 'single_digit', op: 'Catalog', max: 3 })` rejects valid 4 and accepts invalid 0"
      - path: "hp41-gui/src/pending_input.ts"
        issue: "single_digit case has no lower-bound check; Catalog should be 1..=4 range"
    missing:
      - "Change MODAL_OPENERS.catalog to `{ kind: 'single_digit', op: 'Catalog', max: 4 }`"
      - "Add op-specific lower-bound check in pending_input.ts single_digit case: Catalog allows 1..=max, Tone allows 0..=max"
      - "Update pending_input.test.ts Catalog tests: reject 0 and 5, accept 1..=4"

deferred: []

human_verification:
  - test: "Visual sanity of 14-segment LCD"
    expected: "Boot `just gui-dev`. The display renders 14-segment glyphs with dim 'off' segments faintly visible behind lit text. Decimal points sit at lower-right of preceding digit. The dim/lit contrast looks LCD-like, not garish."
    why_human: "Phase 26-02 SUMMARY explicitly notes the user is expected to do a post-merge visual check; opacity 0.1 may be too dim or too bright depending on monitor calibration."

  - test: "Modal preview rendering through 14-seg"
    expected: "Open STO via SHIFT+STO, type 0, type 5. Display should show 'STO __' → 'STO _5' → 'STO 05' rendered with 14-segment glyphs, then dispatch and revert."
    why_human: "End-to-end user flow combining renderModalLcd output with Display14Seg rendering — visual coherence cannot be asserted programmatically."

  - test: "Help overlay search behavior"
    expected: "Press '?'. Overlay opens. Type 'sin' — list narrows to entries containing 'sin' (case-insensitive). Press Esc — overlay closes. Calculator state (X register, stack) is UNCHANGED by the search keystrokes."
    why_human: "CR-02 will FAIL this test today — every search keystroke mutates calculator state. The expected behavior is documented here so post-fix verification has a concrete acceptance criterion."

  - test: "USER mode relabel"
    expected: "Open ASN modal, click STO key, type 'TEST', press Enter. Toggle USER mode. The STO keycap should now display 'TEST' instead of 'STO'. Toggle USER off — 'STO' returns."
    why_human: "CR-01 will FAIL this test today (the ASN is stored at wrong keycode so the relabel never matches). Documented as acceptance criterion for post-fix verification."

  - test: "'p' / 'P' physical-keyboard remap"
    expected: "Press lowercase 'p' — PRGM annunciator toggles. Press SHIFT+'P' — X register prints to the print panel."
    why_human: "Direct physical-keyboard binding behavior; quickly verifiable in dev but not asserted by unit tests today."

  - test: "AVIEW / PROMPT visible effect"
    expected: "Programmatically enter PRGM mode, add steps `LBL 'A' / 'HELLO' / AVIEW / END`, exit PRGM, XEQ 'A'. Display should show 'HELLO' (the alpha-register content)."
    why_human: "CR-04 will FAIL this test today — display_override is set by backend but ignored by the React render. Documented as acceptance criterion."

  - test: "BEEP / TONE n audio or visual feedback"
    expected: "Click BEEP — some user-visible or audible feedback (toast, audio, brief LCD flash). Dispatch TONE 5 — same expectation."
    why_human: "CR-04 will FAIL today (event_buffer drained but never consumed in App.tsx). The user-observable contract for 'BEEP works' is intentionally loose pending v2.3 audio scope, but currently it is dead-silent."
---

# Phase 26: GUI Integration & Polish — Verification Report

**Phase Goal:** Every new v2.2 key ID resolves via `key_map.rs::resolve`; KEY_DEFS carries correct three-label bindings; previously-stubbed prompt IDs route to real React modal flows; 14-seg SVG LCD replaces the CSS-text display; `?` keyboard overlay; USER-mode key relabel; `p` remap to PRGM mode.

**Verified:** 2026-05-15T08:30:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (12 total)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Every v2.2 Op variant (Phases 20-24) resolves via key_map::resolve or resolve_parameterized | PARTIAL | ~80 new bare ops + 20 new prefixes wired with correct more-specific-first ordering. **CR-05 BLOCKER:** Catalog 4 unreachable, Catalog 0 dispatched-then-rejected. See gaps. |
| 2 | Clicking a prompt-id opens a frontend React modal (no GuiError toast for HP-41CV built-ins) | VERIFIED | MODAL_OPENERS table at App.tsx:133-162 maps all 13 D-26.5 prompt-ids + xeq/gto/lbl/asn/view/catalog/tone. handleClick intercept at line 307 fires `setPendingInput` before invokeForKey. Test contract `test_modal_prompt_ids_are_stubs_for_now` (defense-in-depth) continues to pass. |
| 3 | Inside open Flag/Register modal, SHIFT then 0 toggles `ind` (does NOT append '0' to acc) | VERIFIED | handleModalKey detects `shiftActive && key === '0'`, returns `consumesShift=true` with `pending.ind` toggled. Vitest `IND-toggle via shift-0 sets ind=true AND consumesShift=true` and `IND-toggle does NOT append 0 to acc` both pass (W2). |
| 4 | End-of-2-digit accumulation dispatches the correct `_ind_` infix when ind=true | VERIFIED | handleModalKey end-of-2-digit branch decides via tuple match on `pending.ind`. Vitest `register modal with ind=true dispatches sto_ind_NN` passes. |
| 5 | ASN flow opens AssignKey → AssignLabel → text → Enter dispatches `asn_NN_NAME` | FAILED | **CR-01 BLOCKER:** App.tsx:297 uses `key.row*10+(key.col+1)` for keycode instead of `key.keyCode`. **CR-03 BLOCKER:** on-screen ENTER (`'enter'`) is never accepted by `key === 'Enter'` in pending_input.ts. Click-only ASN path is non-functional AND stores at wrong code. |
| 6 | CalcStateView serializes to ≤500 bytes for empty + realistic load | VERIFIED | types.rs:170-209 budget tests pass. Empty: 337 bytes. Realistic load (5 ASN + 3 flags): 401 bytes. FN-GUI-05 honored. **Note WR-06:** ceiling may not cover worst-case AVIEW (24-byte alpha_reg) + 10 ASN entries — Phase 27 should add a stress-load test. |
| 7 | Esc inside open modal cancels modal AND clears shiftActive | VERIFIED | App.tsx:368-380 Esc precedence: helpOpen → pendingInput → shiftActive. Both flags clear when modal Esc fires. Mirror with hp41-cli Phase 25 W3 fix. |
| 8 | DEL prompt accepts 0..=255; 256+ produces "DEL ERR" preview and key_map returns GuiError | VERIFIED | key_map.rs::resolve_parameterized for `del_NNN` clamps at u8 with explicit GuiError message containing "0-255". `test_del_clamps_at_u8_max` passes. Frontend `renderModalLcd` emits "DEL ERR" for acc > 255. Two-layer divergence surface intact. |
| 9 | CalcStateView TS interface in App.tsx mirrors the 4 new Rust projections (user_keymap/flags/display_override/event_buffer) | PARTIAL | TS interface at App.tsx:36-39 has all 4 fields and tsc --noEmit passes. **CR-04 BLOCKER:** `display_override` and `event_buffer` are declared but NEVER READ in render code. The "TS mirror is correct" letter of B5 is satisfied; the spirit of FN-GUI-05 ("wired through to the UI") is violated. |
| 10 | 14-segment SVG LCD replaces CSS-text display | VERIFIED | Display14Seg.tsx exists with SEGMENT_PATHS (14), SEGMENT_MAP (49 glyphs), DECIMAL_DOT_PATH overlay. Wired into App.tsx:479 `<Display14Seg text={displayText} />`. `.display svg { display: block; width: 100%; height: 100% }` committed (W6). 22 Vitest tests green covering W4/W5/W6. Visual sanity check deferred to human (see human_verification). |
| 11 | Pressing `?` opens keyboard shortcut overlay from TS port of help_data.rs; USER mode shows ASN'd labels overlaid on skin | PARTIAL/FAILED | Overlay component (HelpOverlay.tsx) exists, opens on `?`, lists 62 keyboard-bound entries grouped by 11 categories, search input filters. **CR-02 BLOCKER:** search input keystrokes leak to window listener and dispatch ops in background — overlay is technically open but unusable. **USER-mode relabel half:** Keyboard.tsx:204-205 correctly looks up `userKeymap` by `key.keyCode`, but CR-01 stores assignments at the wrong keycode, so the lookup never matches. Both halves of truth 11 are broken end-to-end. |
| 12 | Pressing `p` opens PRGM mode (not PRX); SC-4 invariant grep returns nothing | VERIFIED | App.tsx MAP `'p': 'prgm_mode'` and `'P': 'prx'` confirmed. `resolveKeyId` silence list narrowed from `'SRfFPX'` to `'SRfFX'` so uppercase P reaches MAP. **SC-4 GREEN:** `grep -rEn "fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)" hp41-gui/src-tauri/src/` returns zero matches. |

**Score:** 7/12 truths fully verified; 3 partial; 2 failed. Multiple gaps overlap (CR-01 affects truths 5 and 11; CR-04 affects truth 9 and overlay user-observability of new ops in truth 1).

### Required Artifacts (all exist)

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-gui/src-tauri/src/key_map.rs` | Extended bare-op resolver + parameterized prefixes | VERIFIED | 42 KB, contains `Op::Pi`, IND prefixes ordered correctly, `resolve_sto_arith`, `resolve_asn` helpers. |
| `hp41-gui/src-tauri/src/types.rs` | CalcStateView with 4 new projections | VERIFIED | user_keymap/flags/display_override/event_buffer fields exist; from_state signature extended with event_lines; 6 test call sites updated. |
| `hp41-gui/src-tauri/src/commands.rs` | event_buffer drain on 5 helpers | VERIFIED | 5 `event_buffer.drain(..)` call sites confirmed at lines 217, 230, 269, 277, 286. |
| `hp41-gui/src/App.tsx` | PendingInput state + MODAL_OPENERS + Esc precedence + MAP swap + HelpOverlay wiring | EXISTS-with-defects | All structural elements present; CR-01/CR-02/CR-03/CR-04 are wiring/logic bugs inside the file. |
| `hp41-gui/src/pending_input.ts` | PendingInput discriminated union + handleModalKey + renderModalLcd | EXISTS-with-defects | 14 variants + struct-return handleModalKey + LCD preview emitter. CR-03 and CR-05 are bugs inside this file. |
| `hp41-gui/src/key_defs_ids.ts` | W3 audit source-of-truth | VERIFIED | KEY_DEFS_PRIMARY_IDS + KEY_DEFS_SHIFTED_IDS + KEY_DEFS_HANDLED_OUTSIDE_RESOLVE exported. |
| `hp41-gui/src/Display14Seg.tsx` | 14-segment SVG LCD component | VERIFIED | 12.8 KB; SEGMENT_PATHS length=14; SEGMENT_MAP 49 entries; DECIMAL_DOT_PATH overlay. |
| `hp41-gui/src/Display14Seg.test.tsx` | Vitest tests with W4/W5/W6 | VERIFIED | 22 tests; all green. |
| `hp41-gui/src/Keyboard.tsx` | KeyDef.keyCode hardcoded literals + USER-mode relabel | VERIFIED | 32 keyCode literals in main grid; `resolveUserLabel` at line 203-205 correctly uses key.keyCode. The component itself is correct; CR-01 in App.tsx is what disconnects it from the ASN flow. |
| `hp41-gui/src/HelpOverlay.tsx` | `?` overlay sourced from help_data.ts | EXISTS-with-defect | Component opens and renders 62 entries grouped by category; search filter works. CR-02 in App.tsx (not in HelpOverlay) leaks keystrokes to dispatch. |
| `hp41-gui/src/help_data.ts` | TypeScript port of hp41-cli/src/help_data.rs | VERIFIED | TS port via vite JSON import; 154 entries; helpEntries() + helpOverlayRows(). |
| `hp41-gui/vite.config.ts` | server.fs.allow widened to repo root (W8) | VERIFIED | `fs.allow` includes repo root for docs/hp41cv-functions.json import. |
| `hp41-gui/src/HelpOverlay.test.tsx` | 16 Vitest tests | VERIFIED | All green. |
| `hp41-gui/src/Keyboard.test.tsx` | 41 Vitest tests including W9 sentinel parity + XSS-safety | VERIFIED | All green. |

### Data-Flow Trace (Level 4)

Levels 1–3 (exists, substantive, wired) pass for every artifact. Level 4 surfaces the disconnected projections:

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| App.tsx displayText | calcState.display_str | CalcStateView via get_state | Yes | FLOWING |
| App.tsx displayText | calcState.display_override | Backend Phase 21 ROM ops AView/Prompt/View | Yes (backend sets) | DISCONNECTED — frontend never reads (CR-04) |
| App.tsx (no consumer) | calcState.event_buffer | Backend Phase 21 Beep/Tone push to event_buffer | Yes (backend drains) | HOLLOW_PROP — drained over IPC, ignored by React |
| Keyboard userKeymap | calcState.user_keymap | Backend assignments map | Yes (backend stores) | HOLLOW — wiring exists, but assignments are written at wrong keycode (CR-01) so userKeymap.find always misses |
| Keyboard keyCode | KeyDef literal | hp41-cli/src/keys.rs canonical mapping | Yes | FLOWING |

### Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| `handleClick` | MODAL_OPENERS table | intercept before invokeForKey | WIRED | Line 307 if-branch fires before busyRef set. |
| `handleModalKey` | `invokeForKey(parameterizedId)` | end-of-2-digit tuple decision | WIRED | Pitfall 3 ordering preserved; sto_ind_05 resolves to StoInd, not Sto. |
| `key_map::resolve_parameterized` | `Op::*Ind(NN)` variants | strip_prefix more-specific-first | WIRED | Confirmed via line 175 (sto_ind_) before line 444 (sto_) etc. |
| `types.rs::from_state` | event_buffer | drained in commands.rs and passed as event_lines | WIRED on backend | Backend drain confirmed. But frontend never consumes — see CR-04. |
| TS CalcStateView mirror | Rust CalcStateView | tsc --noEmit | WIRED | All 4 new fields present; tsc clean. |
| App.tsx assign_key click | makeKeyCodeMagic | key.row*10+(key.col+1) | BROKEN | CR-01: should be `key.keyCode` (canonical), not layout coord. |
| App.tsx handleKey | helpOpen short-circuit | only Escape and '?' gated | BROKEN | CR-02: missing `if (helpOpen) return;` before resolveKeyId path. |
| handleClick effectiveId | handleModalKey key alphabet | passes 'enter' to predicate expecting 'Enter' | BROKEN | CR-03: case mismatch; modal stays open. |
| Keyboard userKeymap.find | calcState.user_keymap entries | by code === keyCode | WIRED in isolation; BROKEN end-to-end | CR-01 stores at wrong code; lookup never matches. |
| App.tsx displayText | calcState.display_override | (not referenced) | DEAD | CR-04: derivation only checks pendingInput and display_str. |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Backend types compile | `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` | clean | PASS |
| Frontend TS compiles | `cd hp41-gui && npx tsc --noEmit` | clean | PASS |
| Vitest suite | `cd hp41-gui && npx vitest run` | 121/121 passed | PASS |
| Rust test suite | `cd hp41-gui/src-tauri && cargo test --no-fail-fast` | 61/61 passed | PASS |
| Budget test | `cargo test test_dispatch_op_payload_size` | 2 passed | PASS |
| SC-4 grep | `grep -rEn "fn op_(add\|sub\|...)" hp41-gui/src-tauri/src/` | 0 matches | PASS |
| `Op::Pi` resolver | `grep -n 'Op::Pi' hp41-gui/src-tauri/src/key_map.rs` | line 52 hit | PASS |
| sto_ind_ ordering | strip_prefix(sto_ind_) at line 175 before sto_ at 444 | confirmed | PASS |
| MODAL_OPENERS contains 13 D-26.5 prompt ids | grep | 13 entries (sto_prompt, rcl_prompt, fix_prompt, sci_prompt, eng_prompt, isg_prompt, sf_prompt, cf_prompt, fs_prompt, x_eq_y_prompt, x_le_y_prompt, x_gt_y_prompt, x_eq_0_prompt) | PASS |
| `direct` variant for 4 conditional-test prompts | grep `kind: 'direct'` | 4 entries (x_eq_y/x_le_y/x_gt_y/x_eq_0_prompt) | PASS |
| `single_digit` Tone/Catalog merge | grep `kind: 'single_digit'` | 2 entries (catalog max=3, tone max=9) | PASS-but-catalog-max-is-wrong-per-CR-05 |
| ASN keycode uses canonical | grep App.tsx:297 | `key.row * 10 + (key.col + 1)` | FAIL (CR-01) |
| Enter onscreen routing | grep App.tsx for `'enter' →` translation | none | FAIL (CR-03) |
| `helpOpen` short-circuit before dispatch | grep `if (helpOpen) return` in handleKey body | none | FAIL (CR-02) |
| `display_override` consumed in displayText | grep `display_override` in App.tsx (excluding interface declaration) | only interface field at line 38 | FAIL (CR-04) |
| `event_buffer` consumed by React | grep `event_buffer` in App.tsx (excluding interface declaration) | only interface field at line 39 | FAIL (CR-04) |
| Catalog max=3 vs op_catalog 1..=4 | App.tsx:156 vs hp41-core/src/ops/program.rs:295 | mismatch (0 invalid in core, 4 valid in core but unreachable in GUI) | FAIL (CR-05) |

### Probe Execution

Phase 26 is hp41-gui-only and has no conventional `scripts/*/tests/probe-*.sh` runners. Plan-declared verification is `just gui-ci` + Vitest. Both pass.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| FN-GUI-01 | 26-01 | All v2.2 Op variants resolve via key_map::resolve | PARTIAL | Bare ops + IND prefixes wired; Catalog 4 unreachable / Catalog 0 dispatched-then-rejected via frontend (CR-05). |
| FN-GUI-02 | 26-01 | KEY_DEFS carries correct three-label bindings for every keyboard-reachable function | SATISFIED | Phase 19 KEY_DEFS layout retained; W3 audit (key_defs_ids.ts + Rust mirror test) confirms no missing or unresolved ids. |
| FN-GUI-03 | 26-03 | Modal routing for previously-stubbed prompt IDs | PARTIAL | 13 D-26.5 prompts route to React modals; CR-03 breaks on-screen ENTER submit for clp/xeq_name/assign_label modals — they open but cannot be confirmed via click. |
| FN-GUI-04 | 26-03 | Toast pattern for v3.x-module ops only; no silent discards (D-07) | SATISFIED | Stub-error arm in key_map.rs only carries v3.x-module aliases + defense-in-depth modal-opener fallbacks; D-07 invariant intact. |
| FN-GUI-05 | 26-01 | CalcStateView extended with flags/display_override/event_buffer; JSON budget ≤500 bytes | PARTIAL | Projection added and 500-byte budget honored, but display_override and event_buffer are NEVER CONSUMED in App.tsx (CR-04). The letter of FN-GUI-05 ("CalcStateView extended ... if needed") is satisfied; the spirit (the fields are wired through and have user-observable effects) is broken. |
| FN-POLISH-01 | 26-02 | 14-segment SVG font replaces CSS-text display | SATISFIED | Display14Seg.tsx ships; 22 Vitest tests pass; wired into App.tsx. Visual sanity check deferred to human. |
| FN-POLISH-02 | 26-03 | Keyboard shortcut overlay accessible via `?` key | PARTIAL | Overlay opens and lists entries; CR-02 makes the search input unusable because background dispatch is not gated. |
| FN-POLISH-03 | 26-03 | Full keyboard assignment display in USER mode | FAILED | Keyboard.tsx side is correct; App.tsx CR-01 stores ASN at wrong keycode; end-to-end USER mode shows no relabel. |
| FN-POLISH-04 | 26-03 | `prgm_mode` binding for 'p' key (was prx) | SATISFIED | MAP swap confirmed; uppercase 'P' routes to prx; silence list narrowed. |

### Orphaned Requirements

None. All 9 phase requirement IDs are claimed by at least one plan's `requirements:` frontmatter and are covered above.

### Anti-Patterns Found (selected)

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| hp41-gui/src/App.tsx | 297 | Layout-coord keycode `key.row * 10 + (key.col + 1)` | BLOCKER | ASN writes at wrong code; CR-01 |
| hp41-gui/src/App.tsx | 346-410 | `handleKey` registered on window; no `if (helpOpen) return` gate | BLOCKER | Search input leaks keystrokes; CR-02 |
| hp41-gui/src/pending_input.ts | 244, 326, 372 | Case-sensitive `key === 'Enter'` | BLOCKER | On-screen ENTER click never matches; CR-03 |
| hp41-gui/src/App.tsx | 463-465 | `displayText` derivation ignores `display_override` | BLOCKER | AVIEW/PROMPT/VIEW have no visible effect; CR-04 |
| hp41-gui/src/App.tsx | (no consumer) | `event_buffer` never read by React | BLOCKER | BEEP/TONE silently dropped; CR-04 |
| hp41-gui/src/App.tsx | 156 | Catalog max=3 (should be 4); Catalog 0 accepted (should be 1..) | BLOCKER | Catalog 4 unreachable, Catalog 0 errors out; CR-05 |
| hp41-gui/src/App.tsx | 157 | `tone` MODAL_OPENERS entry with no KEY_DEFS source | WARNING | Dead code (WR-04) |
| hp41-gui/src/pending_input.ts | 491 | `isPrintableChar` regex excludes `<`, `>`, `?`, Unicode | WARNING | XEQ-by-Name for the 8 ROM conditional tests is undispatchable from XEQ modal (WR-02); CLI ↔ GUI parity gap |
| hp41-gui/src/pending_input.ts | 372 | assign_label Enter with empty acc dispatches `asn_NN_` | WARNING | Inconsistency vs clp/xeq_name (WR-03) |
| hp41-gui/src-tauri/src/types.rs | 170-209 | 500-byte ceiling may not cover worst-case display_override + many ASN | WARNING | Future CI false-positive (WR-06) |
| hp41-gui/src/pending_input.ts | 396-402 | `confirm_load`/`hex`/`print` arms are unreachable stubs | INFO | Dead variants (IN-03) |
| hp41-gui/src/Keyboard.tsx | 200-208 | Comment "6 chars" but slice(0,7) | INFO | Off-by-one in comment vs code (IN-01) |
| hp41-gui/src-tauri/src/key_map.rs | 362-364 | Catalog comment doesn't note frontend max divergence | INFO | Stale once CR-05 fixed (IN-02) |
| hp41-gui/src/key_defs_ids.ts | 38, 99 | `'e'` excluded from PRIMARY_IDS but is in KEY_DEFS | INFO | Naming clarity (IN-04) |

No debt markers (TBD/FIXME/XXX) found in Phase 26 modified files via grep.

### Human Verification Required

See `human_verification:` in frontmatter. Seven items — most exist to give the developer a concrete acceptance test after the BLOCKERs are fixed. Three (USER mode relabel, AVIEW visible, search input non-corrupting) WILL FAIL today as documented; they are still listed because re-verification after the fix will need to re-run them.

### Gaps Summary

Phase 26 ships substantial infrastructure (key_map resolver expansion, PendingInput discriminated union with 14 variants, Display14Seg SVG component, HelpOverlay, USER-mode relabel scaffold, `p`→prgm_mode remap, CalcStateView 4 new projections under the 500-byte budget) AND passes every declared automated gate (`just gui-ci`, Vitest 121/121, cargo test 61/61, tsc, clippy, SC-4 grep). Tests document the IPC contract correctly; the SUMMARY claims accurately describe what the test suite exercises.

However, the codebase contains **5 user-observable defects** uncovered by code review that the test suite does not catch — each represents a wiring break BETWEEN correctly-built layers:

1. **CR-01 (ASN keycode):** `App.tsx:297` uses layout coords, not canonical keyCode. Renders FN-POLISH-03 (USER mode relabel) dead-on-arrival because the assignments are written at codes no key advertises.
2. **CR-02 (HelpOverlay search):** `handleKey` doesn't gate on `helpOpen`. Renders FN-POLISH-02 search input unusable — every keystroke corrupts calculator state.
3. **CR-03 (on-screen ENTER):** Case mismatch between handleClick `'enter'` and pending_input `'Enter'`. Click-only ASN/CLP/XEQ/GTO/LBL flows cannot be confirmed.
4. **CR-04 (display_override + event_buffer):** Projections added correctly on the backend but never consumed by React. AView/Prompt/View Phase 21 ops have no visible effect; Beep/Tone are silently dropped. Half of FN-GUI-05 is hollow.
5. **CR-05 (Catalog max off-by-one):** `max: 3` rejects valid `4` and accepts invalid `0`. CATALOG 4 (XFNS) is unreachable from the GUI; CATALOG 0 errors out at backend.

**Root cause pattern:** all five defects are wiring/glue between layers that were each individually unit-tested. The unit tests confirm correctness in isolation but no integration test exercises the end-to-end click → modal → dispatch → render → user-visible-effect path. This is the same gap pattern that Phase 27 FN-QUAL-05 (Playwright E2E smoke test) is intended to close — but Phase 27 hasn't run yet.

**Group:** all 5 BLOCKERS are concentrated in `hp41-gui/src/App.tsx` and `hp41-gui/src/pending_input.ts`. A single focused gap-closure plan can address them all with shared test scaffolding (a Vitest @testing-library/react integration suite that exercises handleClick + handleKey + applyModalResult against a mocked Tauri invoke).

**Phase 26 status:** does not meet its goal. Re-plan with `/gsd-plan-phase --gaps` is required.

---

_Verified: 2026-05-15T08:30:00Z_
_Verifier: Claude (gsd-verifier)_
