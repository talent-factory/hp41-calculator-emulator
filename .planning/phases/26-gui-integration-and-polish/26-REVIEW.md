---
phase: 26-gui-integration-and-polish
reviewed: 2026-05-15T07:40:38Z
depth: standard
files_reviewed: 17
files_reviewed_list:
  - hp41-gui/package.json
  - hp41-gui/src-tauri/src/commands.rs
  - hp41-gui/src-tauri/src/key_map.rs
  - hp41-gui/src-tauri/src/types.rs
  - hp41-gui/src/App.css
  - hp41-gui/src/App.tsx
  - hp41-gui/src/Display14Seg.test.tsx
  - hp41-gui/src/Display14Seg.tsx
  - hp41-gui/src/HelpOverlay.test.tsx
  - hp41-gui/src/HelpOverlay.tsx
  - hp41-gui/src/Keyboard.test.tsx
  - hp41-gui/src/Keyboard.tsx
  - hp41-gui/src/help_data.ts
  - hp41-gui/src/key_defs_ids.ts
  - hp41-gui/src/pending_input.test.ts
  - hp41-gui/src/pending_input.ts
  - hp41-gui/vite.config.ts
findings:
  critical: 5
  warning: 7
  info: 4
  total: 16
status: issues_found
---

# Phase 26: Code Review Report

**Reviewed:** 2026-05-15T07:40:38Z
**Depth:** standard
**Files Reviewed:** 17
**Status:** issues_found

## Summary

Phase 26 wires Phase 20–24 ROM ops into the GUI's `key_map` resolver, adds a frontend-owned `PendingInput` modal state machine, an SVG 14-segment LCD component, a `?` help overlay, and a USER-mode ASN relabel layer. The SC-4 invariant holds (no `op_(add|sub|mul|div|...)` calculator helpers in `hp41-gui/src-tauri/src/`), and the v2.2 ROM-op resolver expansion is exhaustive and well-tested via two parallel parity audits (Rust-side `test_keyboard_skin_ids_resolve_or_are_modal_openers`, TS-side `key_defs_ids.ts`).

However, the integration of these layers has several correctness defects that break the user-observable contract:

1. **ASN keycode computation is wrong (CR-01)** — the on-screen `assign_key` flow computes a keycode from layout `row*10+(col+1)`, which does NOT match the canonical HP-41 hardware code stored in `KeyDef.keyCode`. ASN entries are stored at the wrong code and the USER-mode relabel never matches.
2. **Help overlay does not gate the global key listener (CR-02)** — typing in the search input fires calculator ops in the background (typing 'q' executes `Op::Sin`).
3. **Text-input modal ENTER from on-screen keyboard is dead (CR-03)** — `handleModalKey` expects `"Enter"` (capitalised, DOM-event spelling) but the click router passes the KeyDef id `"enter"` (lowercase). CLP / XEQ / GTO / LBL / ASN modals can only be confirmed via the physical keyboard.
4. **`display_override` projection is unused (CR-04)** — Phase 21 AVIEW / VIEW / PROMPT messages are projected over IPC but never rendered; the display continues to show stack X or entry_buf. Same defect for `event_buffer` (BEEP / TONE n lines are silently dropped).
5. **Catalog `max` off-by-one (CR-05)** — frontend allows `0..=3`, hp41-core accepts `1..=4`. `0` always errors; `4` is unreachable.

Additionally, several lower-severity defects exist: dead modal openers, inconsistent label length caps, race window in modal dispatch, and accept-empty-label divergences from sibling modals.

## Critical Issues

### CR-01: ASN keycode computed from layout coordinates, not canonical HP-41 keyCode

**File:** `hp41-gui/src/App.tsx:297`

**Issue:** The `assign_key` click handler computes the magic keycode as `key.row * 10 + (key.col + 1)`. This is layout-relative; it does NOT match the canonical HP-41 hardware code stored in `KeyDef.keyCode`. The `KEY_DEFS` doc comment (Keyboard.tsx line 36–52) explicitly warns: "HARDCODED LITERALS from the CLI canonical mapping, NOT computed `row * 10 + col`. … e.g. SIN at GUI (row 2, col 2) is HP-41 code 25 from CLI, but `2*10+2=22` is STO". The code in App.tsx commits exactly the W9 violation that the comment was authored to prevent.

Concrete consequences:
- Click SIN (row 2 col 2) as ASN target → keycode passed to magic = `2*10 + (2+1) = 23`. Canonical SIN code is 25. The ASN entry stored in `state.assignments` lives at key 23, not 25.
- USER-mode relabel resolver in `Keyboard.tsx::resolveUserLabel` searches `userKeymap` by `key.keyCode` (the canonical literal 25). 23 ≠ 25 → the ASN'd label is NEVER displayed on the key the user actually clicked.
- STO key (row 3 col 2) → `3*10+3 = 33`; canonical STO code is 22. Same mismatch.
- The bug is silent — the dispatch succeeds, the assignment is stored, no error surfaces, and the user concludes ASN is broken when in fact it stored the entry at a keycode no key in the skin will ever advertise.

**Fix:**
```tsx
const routedKey =
  pendingInput.kind === 'assign_key'
    ? makeKeyCodeMagic(key.keyCode ?? -1)   // use canonical KeyDef.keyCode
    : effectiveId;
```
Additionally, the click handler should reject keys whose `keyCode` is `undefined` (top-row, SHIFT, CHS, CL X/A, XGE Y — see Keyboard.tsx lines 80–96) because ASN cannot target a key with no hardware code. Either short-circuit with a toast ("This key cannot be assigned") or filter the click before opening the assign_label modal.

### CR-02: Help overlay search input does not stop physical-keyboard dispatch

**File:** `hp41-gui/src/App.tsx:346-410`

**Issue:** `handleKey` is registered on `window`. When `helpOpen=true` and the `<input autoFocus>` inside `HelpOverlay` has focus, key events still bubble to `window`. The handler gates only `Escape` (closes overlay) and `'?'` (no-op when already open) on `helpOpen`; every other key flows through to `resolveKeyId` → `dispatchKeyId`.

Concrete consequences:
- User opens help, types `sqrt` in the search box → keys `s`, `q`, `r`, `t` each dispatch ops in the background (`s` → `Op::Sqrt`, `q` → `Op::Sin`, `r` → `Op::Rdn`, `t` → `Op::Tan`). The calculator state mutates while the user is searching for documentation.
- Typing digits to filter — e.g. `7` to find "fix 7" — pushes `7` to `entry_buf` via the digit-entry branch in `handle_op_prepare`. The user has no way to know their search modified state; the overlay is opaque to the calculator behind it.
- `Backspace` in the search input is mapped to `clx` (App.tsx line 100) and dispatches a Clear-X. Editing the search term while a meaningful X value is on the stack destroys that value.
- `Tab` is `e.preventDefault()`'d and toggles `shiftActive` regardless of focus.

**Fix:** add `helpOpen` short-circuit immediately after the Tab branch:
```tsx
if (helpOpen) return;   // overlay owns input focus; calculator listener disabled
if (busyRef.current) return;
```
Place this BEFORE the modal-key branch (`if (pendingInput !== null) { ... }`) so a help overlay opened over an in-progress modal also gets focus precedence. Alternatively, register the listener on the calculator container element instead of `window` so the input's focus naturally captures keystrokes.

### CR-03: Text-input modal ENTER is dead via on-screen keyboard

**File:** `hp41-gui/src/pending_input.ts:244, 326, 372` (and `App.tsx:298`)

**Issue:** Modal cases `'clp'`, `'xeq_name'`, and `'assign_label'` check `if (key === 'Enter')` (capitalised — the DOM `KeyboardEvent.key` spelling). But when the user CLICKS the ENTER key on the on-screen keyboard, `handleClick` resolves `key.id = 'enter'` (lowercase, from `KEY_DEFS`) and passes that to `handleModalKey(effectiveId, ...)`. `'enter' !== 'Enter'` → no dispatch arm matches → falls into the "ignore unmapped keys" tail → modal stays open forever.

Same defect for `Backspace` vs `'clx_or_a'` (the click-time id of the back-arrow key — line 100 of Keyboard.tsx).

Concrete consequences:
- User opens XEQ-by-Name, clicks letters to type a label, clicks ENTER → nothing happens. Modal silently refuses to submit.
- Same for GTO, LBL, CLP, ASN-label.
- Physical-keyboard ENTER works (because `App.tsx::handleKey` line 393 translates `e.key === 'Enter'` to `modalKey = 'Enter'` correctly), so this defect is invisible during keyboard-only testing.

**Fix:** in `App.tsx::handleClick`, translate click-time ids before calling `handleModalKey`, OR widen the modal predicates to accept both spellings:
```tsx
// Option A (App.tsx, ~line 296):
let routedKey: string;
if (pendingInput.kind === 'assign_key') {
  routedKey = makeKeyCodeMagic(key.keyCode ?? -1);
} else if (effectiveId === 'enter') {
  routedKey = 'Enter';
} else if (effectiveId === 'clx_or_a') {
  routedKey = 'Backspace';
} else {
  routedKey = effectiveId;
}
```
Option B (pending_input.ts): change every `key === 'Enter'` to a helper `isEnter(key)` that accepts both. Option A is preferred because it keeps `handleModalKey`'s input alphabet consistent across input sources.

### CR-04: `display_override` and `event_buffer` projected but never rendered

**File:** `hp41-gui/src/App.tsx:23-40, 463-465`

**Issue:** Phase 26 D-26.11 added `display_override: Option<String>` and `event_buffer: Vec<String>` to `CalcStateView` (types.rs lines 47–57). The Rust side correctly drains `state.event_buffer` and projects `state.display_override`. The TypeScript `CalcStateView` interface lists both. But the React render NEVER reads them:

- `displayText` derivation at line 463–465 only checks `pendingInput` and `display_str`:
  ```tsx
  const displayText: string = pendingInput
    ? renderModalLcd(pendingInput)
    : calcState.display_str;
  ```
  `display_override` is ignored. Therefore `Op::AView`, `Op::Prompt`, `Op::View(n)` — Phase 21 ROM ops that the resolver now happily dispatches — have NO visible effect in the GUI. The user clicks AVIEW, the alpha string is supposed to render on the display, the backend correctly sets `display_override`, the projection serialises it, and React drops it.

- `event_buffer` is never read anywhere in `App.tsx`. `Op::Beep` and `Op::Tone(n)` push sound-event lines into `state.event_buffer`, `handle_op_finalize` drains them into `view.event_buffer`, and the frontend discards them. The "BEEP" sound or visual feedback the user expects on each Tone/Beep click is silently dropped.

Phase 26 ROM-op wiring (CR-04 sibling) clicks BEEP and dispatches `Op::Beep` successfully — but no `<audio>` plays, no toast shows "BEEP", no event_buffer rendering exists. The user perceives this as a no-op key.

**Fix:**
```tsx
// displayText priority: modal preview > display_override > display_str
const displayText: string = pendingInput
  ? renderModalLcd(pendingInput)
  : (calcState.display_override ?? calcState.display_str);
```
And for events:
```tsx
// Accumulate event_buffer into the toast or a dedicated event log.
useEffect(() => {
  if (calcState && calcState.event_buffer.length > 0) {
    for (const line of calcState.event_buffer) showToast(line);
  }
}, [calcState]);
```
The display_override behavior should match HP-41 semantics: once display_override is set, it persists across subsequent state polls until the next operation clears it.

### CR-05: Catalog `max` off-by-one in single_digit modal

**File:** `hp41-gui/src/App.tsx:156`

**Issue:** The Catalog modal is opened with `max: 3`:
```tsx
catalog: () => ({ kind: 'single_digit', op: 'Catalog', max: 3 }),
```
But `hp41-core::ops::program::op_catalog` (line 294) accepts `1..=4` (CATALOG 1=programs, 2=keys, 3=ALPHA programs, 4=XFNS). The modal's predicate `digit > max` rejects `4` (which IS valid) and accepts `0` (which is rejected by `op_catalog` with `HpError::InvalidOp`).

Concrete consequences:
- User can never reach CATALOG 4 from the GUI.
- User CAN dispatch `catalog_0` from the modal, which surfaces as a toast error from the backend — but the modal preview "CAT _" gave no warning that 0 was disallowed.

The CLI HP-41CV behavior is `1..=4`. The frontend should mirror that range exactly.

**Fix:**
```tsx
catalog: () => ({ kind: 'single_digit', op: 'Catalog', max: 4 }),
```
And in `pending_input.ts::single_digit` case, add a lower-bound check:
```tsx
case 'single_digit': {
  if (isDigit(key)) {
    const digit = Number(key);
    const minFor = (op: SingleDigitOp) => op === 'Catalog' ? 1 : 0;
    if (digit < minFor(pending.op) || digit > pending.max) {
      return { nextPending: pending, dispatchId: null, consumesShift: false };
    }
    // ...
  }
}
```
Update `pending_input.test.ts::"Catalog rejects digits > 3"` to test `> 4`, and add a `Catalog rejects 0` test.

## Warnings

### WR-01: Race window between handleClick busyRef check and applyModalResult busyRef set

**File:** `hp41-gui/src/App.tsx:264, 292-303, 236-246`

**Issue:** `handleClick` is `async`. It checks `if (busyRef.current) return;` at line 264 but does NOT set `busyRef.current = true` before the modal-key branch (lines 292–303). Inside that branch, `applyModalResult` is awaited; only if the result has a `dispatchId` does `applyModalResult` set `busyRef.current = true` (line 236).

For a non-dispatching modal-key result (most digit keystrokes inside a register/flag modal), busyRef never flips. Two near-simultaneous click events both pass the line-264 check, both await `handleModalKey` (synchronous), both set `setPendingInput` with the same accumulator-plus-one-digit state. React batches the setStates → second click's accumulator overwrites the first's. The user clicks `1` then `2` rapidly and ends up with acc=`'12'` OR acc=`'22'` depending on timing. Not catastrophic in practice (single-threaded JS), but the docstring claims a "two-layer busyRef guard against concurrent invoke()s" which is no longer literally true for the modal path.

**Fix:** Set `busyRef.current = true` immediately after the line-264 check (or before any async work) and clear it in a top-level `try { ... } finally { busyRef.current = false; }`. The current `applyModalResult` inner busyRef can stay as a defense-in-depth no-op.

### WR-02: `xeq_name` modal accepts only [A-Za-z0-9 +\-*/], blocking XEQ-by-Name dispatch of conditional tests

**File:** `hp41-gui/src/pending_input.ts:491`

**Issue:** `isPrintableChar` enforces `/^[A-Za-z0-9 +\-*/]$/`. Phase 25's `builtin_card_op` extension (D-25.8) accepts XEQ-by-Name labels including `X<>Y?`, `X<Y?`, `X≥Y?`, `X≠0?` — characters `<`, `>`, `?`, `≠`, `≤`, `≥` are not in the regex. The 8 ROM conditional tests that Phase 25 explicitly routed through XEQ-by-Name are therefore UNDISPATCHABLE from the GUI's XEQ modal.

This silently breaks the D-25.9 contract ("the other 8 conditionals route through XEQ-by-Name in Phase 25 Plan 03's `builtin_card_op` extension"). The Phase 26 wiring promised parity with CLI; this is a parity gap.

**Fix:** widen the regex to include `<`, `>`, `?`, and the Unicode ASCII spellings, OR accept any character that is not Enter / Escape / Backspace and cap at the existing length (current cap is 24, comfortable for any HP-41 label). For example:
```ts
function isPrintableChar(key: string): boolean {
  return key.length === 1 && key !== '\n' && key !== '\r';
}
```
The downstream resolver in Rust already validates labels at dispatch time.

### WR-03: `assign_label` Enter with empty acc dispatches `asn_NN_` (empty label)

**File:** `hp41-gui/src/pending_input.ts:372-377`

**Issue:** Unlike `'clp'` (line 245 — rejects empty acc) and `'xeq_name'` (line 327 — same), the `'assign_label'` Enter branch dispatches unconditionally:
```ts
if (key === 'Enter') {
  return {
    nextPending: null,
    dispatchId: `asn_${pending.keyCode}_${pending.acc}`,
    consumesShift: false,
  };
}
```
With `acc=''`, this produces `asn_22_` → backend `resolve_asn` splits at `'_'` getting `("22", "")`, returning `Op::Asn { name: "", key_code: 22 }`. Whether hp41-core accepts empty names should be checked, but the inconsistency with `clp` / `xeq_name` is itself a bug.

**Fix:**
```ts
if (key === 'Enter') {
  if (pending.acc.length === 0) {
    return { nextPending: pending, dispatchId: null, consumesShift: false };
  }
  return { /* existing dispatch */ };
}
```

### WR-04: `tone` is in `MODAL_OPENERS` but unreachable from KEY_DEFS

**File:** `hp41-gui/src/App.tsx:157`

**Issue:** `MODAL_OPENERS.tone` opens a single_digit modal with `op: 'Tone', max: 9`. But `tone` is not in `KEY_DEFS_PRIMARY_IDS` or `KEY_DEFS_SHIFTED_IDS` (key_defs_ids.ts) and is not present as a `key.id` or `key.shifted.id` anywhere in Keyboard.tsx. There is no click path that produces `effectiveId === 'tone'`. The MODAL_OPENERS entry is dead code.

The Rust stub-error arm (`key_map.rs:158`) also lists `tone`, which means a physical-keyboard or programmatic dispatch of bare `tone` reaches the backend and produces the "planned for a future phase" toast — but no UI affordance issues `tone`. The user can dispatch `tone_5` etc. via `xeq_name`, but the bare modal-opener is decorative.

Either wire `tone` into a keycap (a SHIFT-modifier slot under BEEP would be natural) or remove the dead entry from both MODAL_OPENERS and the Rust stub.

**Fix:** Remove `tone: () => ({ ... })` from MODAL_OPENERS and `| "tone"` from the stub arm in key_map.rs — OR wire `tone` to a key in KEY_DEFS.

### WR-05: `extractErrMessage` JSON.stringify can throw on circular reference even with try/catch

**File:** `hp41-gui/src/App.tsx:48-58`

**Issue:** The `try { return JSON.stringify(err); } catch { /* fallthrough */ }` is correct as far as it goes — circular references on Tauri framework rejections are extremely unlikely. However, the fallthrough `return String(err)` returns `"[object Object]"` for the very case the helper exists to prevent (the docstring at line 42-47 says exactly this). The path is unreachable today but if a future Tauri version emits cyclic error objects, the helper silently reverts to its degenerate output without any signal.

**Fix:** in the catch block, return a literal `"<unserialisable error>"` so the regression mode is at least visible to the user:
```ts
try { return JSON.stringify(err); }
catch { return '<unserialisable error>'; }
```

### WR-06: `display_override` is included in 500-byte CalcStateView budget assertion

**File:** `hp41-gui/src-tauri/src/types.rs:170-209`

**Issue:** `test_dispatch_op_payload_size` and `test_dispatch_op_payload_size_with_realistic_load` set an upper bound of 500 bytes. The latter inserts 5 ASN entries with short labels (3-5 chars each) and reports "measured load: 401 bytes". A realistic worst-case workload would include `display_override` set to a long AVIEW message (alpha_reg can hold up to 24 chars per HP-41 hardware) AND ~8-10 ASN entries (USER-mode users assign at least one program per mode key). With `display_override = Some("HELLO WORLD AGAIN ABCDE")` (24 bytes), 10 assignments (~10 * 18 = 180 bytes), and a few set flags, the budget is plausibly exceeded.

This is not a correctness bug today, but the 500-byte ceiling is sized for the v2.0 baseline + the v2.2 projections at "typical" load. If the user creates many ASN entries OR triggers a long AVIEW, the test would fail and the gate would block CI on legitimate growth.

**Fix:** Either raise the ceiling to 1024 bytes (the JSON budget is per-IPC-response, and the dispatch frequency is human-scale — even 1 KB at 10 Hz is 10 KB/s, well below any wire bandwidth concern), OR add a test case that explicitly constructs the worst-case load and documents the measured size.

### WR-07: `pending_input.ts::xeq_name` length cap is 24 — divergent from `clp`/`assign_label` caps of 7

**File:** `hp41-gui/src/pending_input.ts:343, 261, 386`

**Issue:** Three text-input modals have three different length caps:
- `clp`: 7 (line 261)
- `assign_label`: 7 (line 386)
- `xeq_name`: 24 (line 343)

HP-41 program label convention is "up to 6 chars" (Keyboard.tsx line 200 says "ASN labels are up to 6 chars per the ALPHA pack convention"). The CLP cap of 7 is one over that. The xeq_name cap of 24 matches alpha_reg width, which is appropriate for XEQ-by-Name (Phase 25 D-25.8 mnemonics can be up to 24 chars), but the inconsistency suggests cargo-culting rather than a deliberate design decision.

**Fix:** document the three caps with a one-line comment explaining the divergence, OR unify clp + assign_label at 6 (HP-41 hardware truth) and leave xeq_name at 24 (the long-mnemonic path).

## Info

### IN-01: Comment on line 261 (Keyboard.tsx slice cap) says "7 chars" but real limit is 6 per HP-41 ALPHA pack

**File:** `hp41-gui/src/Keyboard.tsx:200-208`

**Issue:** The defensive truncation in `resolveUserLabel` uses `slice(0, 7)`. The comment says "HP-41 ASN labels are up to 6 chars per the ALPHA pack convention; the slice caps the visual blast radius at 7". The 7th char position is meant for visual room, but the test `"long ASN labels are truncated at 7 chars (defensive)"` and the XSS test both pin the value at 7. The XSS test specifically asserts `'<scrip'` (6 chars) is present — actually that's 6 chars not 7. Let me re-check: `'<script>alert(1)</script>'.slice(0,7)` = `'<script'` (7 chars). OK, so the test does verify 7. Minor: the comment says "up to 6" but slice keeps 7. Pedantic — clarify whether 6 or 7 is the intended cap.

**Fix:** clarify the comment or align slice to 6 if hardware-faithful.

### IN-02: Catalog comment in `key_map.rs` is stale

**File:** `hp41-gui/src-tauri/src/key_map.rs:362-364`

**Issue:** Comment reads "Phase 22 catalog (1..=4 per HP-41CV; resolver accepts any u8, dispatch validates the range and returns InvalidOp on n==0 or n>=5)". Correct, but combined with CR-05 the frontend cap of `max: 3` makes the comment misleading — readers searching for the canonical Catalog range will land here and incorrectly conclude the frontend matches.

**Fix:** After CR-05 is fixed, add: "Frontend modal cap is `max: 4` (App.tsx::MODAL_OPENERS.catalog)."

### IN-03: `pending_input.ts::print` / `hex` / `confirm_load` modal arms are unreachable stubs

**File:** `hp41-gui/src/pending_input.ts:396-402`

**Issue:** The `case 'confirm_load' | 'hex' | 'print'` arm returns `{ nextPending: pending, dispatchId: null, consumesShift: false }` for any keystroke. The variants are declared in the union (lines 60-65) but no MODAL_OPENERS factory produces them, and no other code constructs them. Dead arms.

**Fix:** drop the three variants from `PendingInput` until v2.3 wires them, OR mark each with a `// TODO(v2.3):` comment indicating the planned phase. Leaving them silently no-op invites future authors to misread them as "supported but stubbed" rather than "not yet wired".

### IN-04: `KEY_DEFS_HANDLED_OUTSIDE_RESOLVE` includes `'e'` but `e` is NOT in KEY_DEFS_PRIMARY_IDS

**File:** `hp41-gui/src/key_defs_ids.ts:38, 99`

**Issue:** The PRIMARY_IDS comment at line 38 says "'e' is a digit-input id handled outside resolve" — but `'e'` is in `KEY_DEFS_HANDLED_OUTSIDE_RESOLVE` (line 99) AND `e` is a primary id on the EEX key in Keyboard.tsx (line 99 of Keyboard.tsx: `{ id: 'e', label: 'EEX', ..., keyCode: 83 }`). So `'e'` IS a primary id in KEY_DEFS but is intentionally excluded from KEY_DEFS_PRIMARY_IDS. The comment splits a hair that future readers will trip over.

**Fix:** rename `KEY_DEFS_PRIMARY_IDS` to `KEY_DEFS_PRIMARY_IDS_FOR_RESOLVER` or add a clarifying comment: "Excludes ids that are handled outside `key_map::resolve` (digit-input, special routes) — see KEY_DEFS_HANDLED_OUTSIDE_RESOLVE."

---

_Reviewed: 2026-05-15T07:40:38Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
