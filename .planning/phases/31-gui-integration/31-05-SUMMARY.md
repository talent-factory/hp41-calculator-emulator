---
phase: 31-gui-integration
plan: "05"
subsystem: hp41-gui/src-tauri + hp41-gui/src
tags: [lcd-alternation, modal-prompt, display14seg, pending-input, app-tsx, vitest, regression-test]
dependency_graph:
  requires: [31-03]
  provides: [LCD-alternation-routing, display14seg-continuation-marker, xeq-name-mode-discriminator, r-s-3-way-routing, esc-cascade, auto-open-collect-for-modal, gui07-stub-arm-regression]
  affects:
    - hp41-gui/src-tauri/src/types.rs
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs
    - hp41-gui/src-tauri/tests/key_map_stub_error_arms.rs
    - hp41-gui/src/Display14Seg.tsx
    - hp41-gui/src/Display14Seg.test.tsx
    - hp41-gui/src/pending_input.ts
    - hp41-gui/src/pending_input.test.ts
    - hp41-gui/src/App.tsx
    - hp41-gui/src/App.test.tsx
tech_stack:
  added: []
  patterns:
    - LCD-alternation branch at TOP of CalcStateView::from_state display_str priority chain
    - truncate_with_continuation helper with Unicode-correct char iteration
    - SEGMENT_MAP extension for U+2261 three-bar continuation marker
    - optional mode discriminator on PendingInput::XeqByName (Pitfall 13 backward-compat)
    - magic-prefix dispatch routing for submit_modal_with_label
    - include_str! file-text regression test (no API access needed)
key_files:
  created:
    - hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs
    - hp41-gui/src-tauri/tests/key_map_stub_error_arms.rs
  modified:
    - hp41-gui/src-tauri/src/types.rs
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src/Display14Seg.tsx
    - hp41-gui/src/Display14Seg.test.tsx
    - hp41-gui/src/pending_input.ts
    - hp41-gui/src/pending_input.test.ts
    - hp41-gui/src/App.tsx
    - hp41-gui/src/App.test.tsx
decisions:
  - "LCD-alternation routing lives in types.rs::CalcStateView::from_state (W2 fix — NOT handle_get_state, NOT display_override); display_override reserved for Phase 21 VIEW/AVIEW/PROMPT/CLD"
  - "types module made pub in lib.rs to allow integration test access to CalcStateView::from_state (key_map stays private per GUI-07 / Task 4)"
  - "BASELINE_N = 1 for key_map.rs stub-error template count (one format! template covering ~20 ids in one match arm); locked with assert_eq! not >="
  - "truncate_with_continuation: LCD_WIDTH=12, take(LCD_WIDTH-1)=11 chars + CONTINUATION char; 'FUNCTION NAME?' (14 chars) → 'FUNCTION NA≡' (11+1=12 chars)"
  - "PendingInput::XeqByName mode field is optional (mode?: 'normal' | 'collect-for-modal') for backward-compat; existing call sites default to 'normal'"
  - "Magic-prefix SUBMIT_MODAL_WITH_LABEL_PREFIX='__submit_modal_with_label__' routes CollectForModal Enter through invokeForKey to submit_modal_with_label Tauri thunk"
metrics:
  duration: "~90 minutes"
  completed: "2026-05-17"
  tasks_completed: 4
  tasks_total: 4
  files_modified: 10
  tests_added: 14
---

# Phase 31 Plan 05: LCD-alternation + R/S 3-way + Esc Cascade + GUI-07 Lock — Summary

**One-liner:** LCD modal-prompt alternation via a new 4th branch at the TOP of `CalcStateView::from_state` + `≡` glyph in Display14Seg + XeqByName `mode` discriminator + R/S 3-way (modal/cancel/run-stop) + Esc 4-way cascade + post-dispatch auto-open useEffect + GUI-07 file-text regression lock.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | LCD-alternation routing in types.rs + ≡ glyph + regression tests | db1764a | types.rs, lib.rs, Display14Seg.tsx, Display14Seg.test.tsx, lcd_alternation_modal_prompt.rs |
| 2 | PendingInput::XeqByName mode extension + magic-prefix routing | 68b9156 | pending_input.ts, pending_input.test.ts |
| 3 | R/S 3-way + Esc cascade + auto-open useEffect + App.test.tsx cases | 5a8a081 | App.tsx, App.test.tsx |
| 3a | Fix: remove unused 'container' TS6133 in H4 test | 32273a7 | App.test.tsx |
| 4 | GUI-07 stub-arm file-text count regression test | be2e824 | key_map_stub_error_arms.rs |

## What Was Built

### Task 1: LCD-alternation routing (db1764a)

**hp41-gui/src-tauri/src/types.rs:**
- Added `LCD_WIDTH: usize = 12` and `CONTINUATION: char = '\u{2261}'` module-level constants
- Added `fn truncate_with_continuation(s: &str) -> String` using `s.chars().collect::<Vec<char>>()` for Unicode-correct char iteration
- Inserted new 4th branch at TOP of `CalcStateView::from_state` display_str priority chain:
  ```
  modal_program.is_some() && entry_buf.is_empty() && modal_prompt.is_some()
    → truncate_with_continuation(modal_prompt)
  ```
  Placed BEFORE existing `if !state.entry_buf.is_empty()` branch

**Routing placement:** `types.rs::from_state` — W2 fix committed. `handle_get_state` NOT modified. `display_override` NOT used (reserved for Phase 21 VIEW/AVIEW/PROMPT/CLD per state.rs).

**hp41-gui/src-tauri/src/lib.rs:**
- Changed `mod types;` to `pub mod types;` so integration tests can access `CalcStateView::from_state` directly

**hp41-gui/src/Display14Seg.tsx:**
- Added `'\u{2261}': [0, 6, 7, 3]` to SEGMENT_MAP (top + middle-left + middle-right + bottom = three-bar shape)

**hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs:** 5 tests:
- `prompt_under_12_chars_no_truncation` — "ORDER=?" (7) renders verbatim
- `prompt_at_12_chars_no_truncation` — "DEGREE=?ABCD" (12) renders verbatim (boundary)
- `prompt_at_13_chars_truncated_with_marker` — "NO. SAMPLES=?" (13) → 11 chars + ≡
- `prompt_at_14_chars_truncated` — "FUNCTION NAME?" (14) → "FUNCTION NA≡" (11+1=12)
- `entry_buf_nonempty_overrides_prompt` — entry_buf wins over modal_prompt

**hp41-gui/src/Display14Seg.test.tsx:** 2 new tests:
- SEGMENT_MAP contains U+2261 entry assertion
- Renders ≡ as three-bar (segments 0,3,6,7 lit) in cell 11 of a 12-char string

### Task 2: PendingInput mode discriminator (68b9156)

**hp41-gui/src/pending_input.ts:**
- Extended `xeq_name` variant: `mode?: 'normal' | 'collect-for-modal'` (optional for backward-compat)
- In `handleModalKey` Enter branch: `mode === 'collect-for-modal'` → `__submit_modal_with_label__<acc>` magic-prefix dispatch; `mode` missing or `'normal'` → existing `<prefix>_<acc>` behavior
- Exported `SUBMIT_MODAL_WITH_LABEL_PREFIX = '__submit_modal_with_label__'`

**hp41-gui/src/pending_input.test.ts:** 6 new test cases covering normal/collect-for-modal Enter/Backspace/empty-acc behaviors

### Task 3: App.tsx R/S + Esc + auto-open (5a8a081)

**hp41-gui/src/App.tsx:**
- `invokeForKey(effectiveId, state)` extended with `state: CalcStateView | null` parameter
- Magic-prefix route: `effectiveId.startsWith(SUBMIT_MODAL_WITH_LABEL_PREFIX)` → `invoke('submit_modal_with_label', { label })`
- R/S 3-way (D-31.1): `state?.modal_program_active` → `submit_modal`; `state?.is_running` → `request_cancel + get_state`; else → `run_stop`
- Esc cascade (D-31.2): help → pendingInput-clear → `modal_program_active` → `cancel_modal`; `is_running` → `request_cancel + get_state`; `shiftActive` → clear; else → no-op
- Post-dispatch auto-open `useEffect([calcState, pendingInput])`: when `modal_program_active && modal_requires_alpha_label && pendingInput === null`, calls `setPendingInput({kind: 'xeq_name', mode: 'collect-for-modal', ...})`
- Updated all 3 `invokeForKey` call sites to pass `calcState`

**Pitfall 5 (R/S insertion site):** The R/S 3-way branch lives INSIDE `invokeForKey`, which is called AFTER the `pendingInput !== null` block in `handleClick` (line ~372). So pending_input routing always runs FIRST — D-07 never-discard invariant preserved.

**5-line context around R/S insertion in `invokeForKey`:**
```typescript
  if (effectiveId === 'r_s') {
    if (state?.modal_program_active) {
      return invoke<CalcStateView>('submit_modal');
    }
    if (state?.is_running) {
```

**hp41-gui/src/App.test.tsx:** 5 new test cases H1-H5 covering R/S 3-way, Esc cancel_modal, and auto-open CollectForModal

### Task 4: GUI-07 stub-arm regression lock (be2e824)

**hp41-gui/src-tauri/tests/key_map_stub_error_arms.rs:**
- Uses `include_str!("../src/key_map.rs")` — NO `hp41_gui_lib::key_map::*` import, NO `resolve()` call
- `BASELINE_N = 1` (one format! template occurrence) — locked per the file text as of v3.0
- `stub_error_message_count_locked_to_v21_baseline`: `assert_eq!` (NOT `>=`) on `KEY_MAP_SRC.matches("is planned for a future phase").count()`
- `key_map_file_contains_v21_baseline_ids`: asserts all 20 v2.1 id literals present (asn, catalog, view, 13 *_prompt, tone)
- `key_map_src_is_nonempty`: sanity check for include_str! resolution

## Verification Results

| Check | Result |
|-------|--------|
| `cargo test --test lcd_alternation_modal_prompt` | 5/5 pass |
| `cargo test --test key_map_stub_error_arms` | 3/3 pass |
| `cargo test --test sc4_invariant` | 1/1 pass (no SC-4 regression) |
| `cargo test --test d25_6_parity` | 4/4 pass (no D-25.6 regression) |
| `cd hp41-gui && npm test -- Display14Seg` | 24/24 pass |
| `cd hp41-gui && npm test -- pending_input` | 56/56 pass |
| `cd hp41-gui && npm test -- App.test` | 23/23 pass |
| `cd hp41-gui && npm test` (full suite) | 166/166 pass |
| `cd hp41-gui && npx tsc --noEmit` | 0 errors |
| `bash scripts/check-tauri-permissions.sh` | OK: all 9 commands |
| SC-4 grep | 0 matches (clean) |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Incorrect expected string in `prompt_at_14_chars_truncated` test**
- **Found during:** Task 1 test run
- **Issue:** Plan said `"FUNCTION NAME?"` → `"FUNCTION NAM≡"` but LCD_WIDTH=12, take(11) gives `"FUNCTION NA"` (11 chars), so the result is `"FUNCTION NA≡"` not `"FUNCTION NAM≡"`
- **Fix:** Corrected expected value in test to `"FUNCTION NA\u{2261}"` with comment explaining char count
- **Files modified:** hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs

**2. [Rule 2 - Invariant] types module made pub for integration test access**
- **Found during:** Task 1 (integration test compilation failure: `error[E0603]: module 'types' is private`)
- **Issue:** `types` was `mod types;` (private) but the plan required an integration test calling `CalcStateView::from_state`. Task 4 only prohibits `pub mod key_map` — `types` is not key_map.
- **Fix:** Changed `mod types;` to `pub mod types;` in lib.rs
- **Files modified:** hp41-gui/src-tauri/src/lib.rs

**3. [Rule 1 - Bug] Display14Seg test used 13-char string for a 12-cell assertion**
- **Found during:** Task 1 Vitest run
- **Issue:** Initial test used `"FUNCTION NAM\u{2261}"` (13 chars), but Display14Seg only renders 12 cells — the ≡ was cut off. The test asserted opacity on cell 11 but ≡ was at position 12 (beyond the display).
- **Fix:** Changed test text to `"FUNCTION NA\u{2261}"` (12 chars) — ≡ at cell 11
- **Files modified:** hp41-gui/src/Display14Seg.test.tsx

**4. [Rule 1 - Bug] Absolute path safety violation (worktree context)**
- **Found during:** Task 1 initial edits
- **Issue:** Write/Edit tool calls used paths under the main repo (`/Users/daniel/GitRepository/hp41-calculator-emulator/`) instead of the worktree (`/.../.claude/worktrees/agent-ada0f34e4b9eaf042/`). Changes landed in the main repo working tree (not committed).
- **Fix:** Manually copied changed files from main repo to worktree; reverted main repo changes; thereafter used relative paths from worktree CWD and worktree-absolute paths for all subsequent edits.
- **Note:** No test failures from this; the data was recovered. Future agents must use relative paths or worktree-derived absolute paths.

**5. [Rule 2 - Missing dependency] dispatchKeyId useCallback missing deps**
- **Found during:** Task 3 (TypeScript warnings, caught during review)
- **Issue:** `dispatchKeyId` callback captured `calcState` but its deps array was empty `[]`
- **Fix:** Added `[calcState, showToast]` deps; also updated `applyModalResult` deps from `[showToast]` to `[calcState, showToast]`
- **Files modified:** hp41-gui/src/App.tsx

## Key Architecture Notes

### LCD-alternation routing placement (W2 fix)
The routing lives in `types.rs::from_state`, NOT in `commands.rs::handle_get_state`. Reason: `display_override` (the alternative routing channel) is reserved for Phase 21 VIEW/AVIEW/PROMPT/CLD semantics and is cleared at the top of dispatch. Using `display_override` would cause collisions with Phase 21 behavior. The `from_state` branch isolates the change to one file and keeps `handle_get_state` unchanged.

### BASELINE_N = 1 explanation
The key_map.rs stub-error pattern uses ONE format! template:
```rust
"asn" | "catalog" | "view" | ... | "tone" => Err(GuiError {
    message: format!("'{key_id}' is planned for a future phase"),
})
```
This single template covers all ~20 ids. The count is 1, not 20.

### Pitfall 5 (R/S insertion-site) confirmation
The D-07 never-discard invariant is preserved because:
1. `handleClick` checks `pendingInput !== null` FIRST (line ~317) → routes through `handleModalKey`
2. Only if `pendingInput === null` does execution reach `invokeForKey`
3. The R/S 3-way branch lives INSIDE `invokeForKey` — so pending_input always wins

The 5-line context around the R/S branch in `invokeForKey`:
```typescript
async function invokeForKey(effectiveId: string, state: CalcStateView | null): Promise<CalcStateView> {
  if (effectiveId.startsWith(SUBMIT_MODAL_WITH_LABEL_PREFIX)) { ... }
  if (effectiveId === 'sst') return invoke<CalcStateView>('sst_step');
  if (effectiveId === 'bst') return invoke<CalcStateView>('bst_step');
  if (effectiveId === 'r_s') {
    if (state?.modal_program_active) { return invoke<CalcStateView>('submit_modal'); }
    if (state?.is_running) { await invoke<void>('request_cancel'); return invoke<CalcStateView>('get_state'); }
    return invoke<CalcStateView>('run_stop');
  }
```

## Manual Verification (VALIDATION.md "Manual-Only Verifications")

**Matrix-entry modal UX:** Could not perform live GUI verification in this worktree context (no Tauri runtime). The full E2E flow (`XEQ "MATRIX" Enter → ORDER=? → 3 ENTER → A1,1=? → ...`) requires `just gui-dev` on a machine with the Tauri binary. The behavior is verified by:
- Unit tests: `lcd_alternation_modal_prompt.rs` confirms display_str = "ORDER=?" after modal_prompt is set
- Integration flow: App.test.tsx H1 confirms R/S routes to submit_modal when modal is active
- App.test.tsx H5 confirms auto-open CollectForModal when modal_requires_alpha_label is set

## Wave 3 Ship Gates

| Gate | Status |
|------|--------|
| `cargo test --test lcd_alternation_modal_prompt` | 5/5 pass |
| `cargo test --test key_map_stub_error_arms` | 3/3 pass |
| `cargo test --test sc4_invariant` | 1/1 pass |
| `cargo test --test d25_6_parity` | 4/4 pass |
| `npm test` (166 Vitest tests) | 166/166 pass |
| `npx tsc --noEmit` | 0 errors |
| `bash scripts/check-tauri-permissions.sh` | 9/9 OK |
| WebdriverIO E2E smoke (Ubuntu) | requires CI — not run locally |

## Self-Check

### Created files exist:
- hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs: FOUND
- hp41-gui/src-tauri/tests/key_map_stub_error_arms.rs: FOUND

### Commits exist (git log):
- db1764a feat(31-05): LCD-alternation routing in types.rs + ≡ glyph in Display14Seg
- 68b9156 feat(31-05): PendingInput::XeqByName mode discriminator + magic-prefix routing
- 5a8a081 feat(31-05): R/S 3-way + Esc cascade + auto-open useEffect in App.tsx
- be2e824 test(31-05): GUI-07 stub-arm file-text regression test for key_map.rs
- 32273a7 fix(31-05): remove unused 'container' variable in H4 Esc test (TS6133)

## Self-Check: PASSED
