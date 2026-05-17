# Phase 31: GUI Integration — Context

**Gathered:** 2026-05-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 31 mirrors the Phase 29 CLI Math Pac I surface into `hp41-gui` so Math Pac I reaches end-users through the GUI. Every behavioral path Phase 31 ships routes through Phase 28/29's shared `hp41-core` surface — `xrom_resolve`, `submit_modal`, `cancel_modal`, `requires_alpha_label`, and the new `request_cancel` trigger for the already-plumbed `cancel_requested: Arc<AtomicBool>`. D-25.6 (CLI ↔ GUI parity) is the contract: NO duplicate resolvers in `key_map.rs`, NO parallel Math Pac I math in `hp41-gui/src-tauri/`, NO frontend-only routing paths.

Five concrete consequences:

1. **GUI parity with Phase 29 CLI behavior** — R/S in `App.tsx` becomes 3-way state-routed: `modal_program.is_some()` → `submit_modal`; `is_running` → new `request_cancel`; else → existing `run_stop`. Esc becomes universal cancel: `modal_program.is_some()` → `cancel_modal`; `is_running` → `request_cancel`; else → clear `shiftActive` (existing v2.1 behavior). `PendingInput::XeqByName { acc, mode }` ports verbatim into `hp41-gui/src/pending_input.ts`; the post-dispatch auto-open mechanic (Phase 29 D-29.9) mirrors verbatim.
2. **Cancellation channel (Pitfall 11 mitigation)** — new `request_cancel` Tauri command + `permissions/request-cancel.toml`; the command flips `state.cancel_requested.store(true, Ordering::Relaxed)` and is idempotent / no-op when `is_running == false`. The per-64-samples polling and lock-release pattern in `op_integ`/`op_solve`/`op_difeq` already ships from Phase 28 (D-28.7); Phase 31 only wires the frontend trigger.
3. **Modal-prompt rendering on the LCD itself (hardware-faithful)** — `Display14Seg` shows `state.modal_prompt` (truncated to 12 chars with HP-41 `≡` continuation marker) when `entry_buf.is_empty() && modal_program.is_some()`; switches to `entry_buf` live when the user types; switches to the next prompt after R/S advances. This is exactly the real HP-41CV LCD-alternation behavior — the single 12-char LCD is the entire user interface; the CLI's Phase 29 D-29.3 status-bar is a TUI compromise the GUI does not need. Routing computed in `commands.rs::handle_get_state` (likely; Claude's discretion); CLI's status-bar continues working in parallel for now.
4. **`?`-overlay parallel-load + two-section layout** — `HelpOverlay.tsx` loads `docs/hp41-math1-functions.json` via Vite JSON-import alongside the existing `hp41cv-functions.json`; entries render in two top-level collapsible sections — "HP-41CV (built-in)" and "Math 1 Pac (XROM 7)" — both expanded by default. The JSON's per-program categories ("Math1 Hyperbolics", "Math1 Complex", "Math1 Matrix", etc.) preserve as 2nd-level headers within the Math 1 Pac section. Future v3.1+ pacs add new top-level sections without touching the existing two.
5. **CATALOG 2 hardware-faithful PSE scroll** — `Op::Catalog(2)` mirrors the v2.2 Phase 22 CAT 1 pattern: pushes a module header line ("XROM Math 1") followed by every function name into `state.print_buffer` one at a time with ~500ms PSE delay; R/S pauses/resumes; other keys cancel.

**In scope:**
- R/S 3-way state routing in `hp41-gui/src/App.tsx` (handleClick + handleKeyDown + invokeForKey)
- Esc universal cancel in `hp41-gui/src/App.tsx` (handleKeyDown)
- New `request_cancel` Tauri command in `hp41-gui/src-tauri/src/commands.rs`
- New `hp41-gui/src-tauri/permissions/request-cancel.toml`
- `request_cancel` registration in `generate_handler!` in `hp41-gui/src-tauri/src/lib.rs`
- `PendingInput::XeqByName { acc, mode: XeqByNameMode }` in `hp41-gui/src/pending_input.ts` (port from Phase 29 CLI)
- Post-dispatch auto-open for `FUNCTION NAME?` collection in `App.tsx` (mirror D-29.9 verbatim)
- LCD modal-prompt rendering: `commands.rs::handle_get_state` (or equivalent location, Claude's discretion) routes `modal_prompt` → `state.display` when `entry_buf.is_empty() && modal_program.is_some()`; HP-41 `≡` truncation
- `HelpOverlay.tsx` extended with two-section wrapper grouping
- Vite path alias / config in `hp41-gui/vite.config.ts` for `../../docs/*.json` JSON imports (or symlink — planner picks)
- `Op::Catalog(2)` body verification: research-prep MUST determine whether Phase 28 shipped full body or stub. If stub, Phase 31 takes a surgical `hp41-core` exception to wire CAT 2's per-XROM-module enumeration (similar to v2.2 Phase 25-03's `builtin_card_op` 4→12 extension; visibility stays `pub(super) fn`)
- `prgm_display.rs` verification + SC-4 grep test (Plan 31-01 shrunk per Phase 29 verification: arms already shipped in Phase 28 plans 28-02..28-10)
- `permissions/request-cancel.toml` covered by the v2.2 `scripts/check-tauri-permissions.sh` CI gate (Pitfall 21)

**Out of scope (explicit):**
- Any `hp41-core/src/ops/math1/` algorithm changes — the family is FROZEN; Phase 31's only acceptable core exception is the surgical CAT 2 body if Phase 28 shipped a stub
- CLI Esc/Ctrl-C → request_cancel backport — deferred to a quick-task follow-up after Phase 31 ships the shared `request_cancel` surface
- Backport of LCD-alternation routing into `hp41-core` `display()` to deprecate CLI's Phase 29 D-29.3 status-bar — deferred (CLI keeps status-bar; GUI gets hardware-faithful LCD; they coexist)
- AVIEW-style scrolling for prompts >12 chars on the GUI LCD — v3.1 polish (Phase 31 ships truncation only)
- Filter input on `?`-overlay — deferred unless v3.1+ pacs push entry count past discoverability threshold
- Tab UI for `?`-overlay — deferred until v3.1+ tab-overflow becomes a real problem
- Modal-prompt rendering in any location OTHER than the LCD (no banner above LCD, no print-panel header, no tooltip, no dedicated status row) — pure hardware fidelity per the user's explicit "as close as possible to real HP-41CV"
- Cancel button overlay — keyboard-first interaction (R/S + Esc) is the canonical UX
- WebdriverIO E2E smoke extension with Math Pac I workflow — Phase 32 / QUAL-03
- Free42 GPL-contamination guard CI script — Phase 32 / QUAL-05
- `op_display_name` arms in `hp41-gui/src-tauri/src/prgm_display.rs` — already shipped in Phase 28; Plan 31-01 reduces to verification

**Mandated by ROADMAP cross-cutting constraints (lines 35–45 of `.planning/ROADMAP.md`):**
- **SC-4 invariant**: stricter grep `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` must return nothing. Phase 31's `commands.rs` additions (`request_cancel`, optionally CAT 2 surgical body if Phase 28 stub) MUST NOT contain calculator-math functions. The existing `op_display_name` exception (display formatter only) continues to apply.
- **CLI ↔ GUI parity (D-25.6)**: every Phase 31 behavior routes through shared `hp41-core` functions — `xrom_resolve`, `submit_modal`, `cancel_modal`, `requires_alpha_label`, the new `request_cancel`. No GUI-only Math Pac I logic. The R/S 3-way state machine on the frontend is a routing decision over the same shared core entry points; D-25.6 contract preserved.
- **`pending_input` routing above modal interceptors** — D-07 (no silent discards) preserved on the GUI side: `pending_input.ts` routing block stays ABOVE modal-opening interceptors in `handleClick`/`handleKeyDown`.
- **MSRV 1.88 unchanged.** Zero new runtime dependencies in `hp41-core` or `hp41-gui/src-tauri/`. No new npm dependencies in `hp41-gui/` (Vite is already wired; JSON-import is a built-in feature).
- **4-exhaustive-match invariant**: `XeqByNameMode` enum on the GUI side (and any Phase 31 hp41-core enum surgical extension) carries compile-time exhaustive match per FN-CLI-04 / FN-GUI-04 invariant — no `_ =>` catch-all in any handler.

</domain>

<decisions>
## Implementation Decisions

### Already locked in PROJECT.md / STATE.md / 28-CONTEXT.md / 29-CONTEXT.md / 30-CONTEXT.md (carried forward — NOT re-decided here)

- **C-28.1 / ADR-001:** Op-strategy A — one `Op` variant per Math Pac I function (~40 variants already shipped in Phase 28). Phase 31 adds zero new `Op::*` variants (the only surgical possibility is the `Op::Catalog(2)` body if Phase 28 left a stub).
- **C-28.3 / ADR-005:** Separate `docs/hp41-math1-functions.json` sibling file (~55 entries) authored in Phase 29 D-29.1. Phase 31 consumes it read-only via Vite JSON-import.
- **C-28.4:** `xrom_resolve` fires LAST in the resolver chain. The GUI's XEQ-by-name modal resolves through the SAME `xrom_resolve` core function (D-25.6 trivially satisfied — no parallel resolver in `key_map.rs`).
- **D-25.6 CLI ↔ GUI parity:** GUI calls shared `submit_modal` / `cancel_modal` / `requires_alpha_label` / `xrom_resolve` / `request_cancel` core functions. NO parallel implementations.
- **D-28.4:** `modal_prompt: Option<String>` is the dedicated prompt channel; `print_buffer` carries PRX/PRA/PRSTK output ONLY. Phase 31's LCD-alternation routing reads `modal_prompt`; print_buffer remains untouched (D-31.7 below).
- **D-28.6:** XEQ-by-name only — Math Pac I has no dedicated key bindings. `KEY_DEFS` in `Keyboard.tsx` gains no new entries; the existing stub-error arm policy (GUI-07) is preserved.
- **D-28.7:** `cancel_requested: Arc<AtomicBool>` field on `CalcState` (`#[serde(skip)]`) + per-64-samples checks in `op_integ`/`op_solve`/`op_difeq` already ship from Phase 28. Phase 31 only wires the frontend trigger (`request_cancel` Tauri command).
- **D-29.5 (Phase 29 contract):** CLI R/S during `modal_program.is_some()` → `submit_modal`. GUI mirrors verbatim; routes via `invokeForKey('r_s')` with state-aware dispatch.
- **D-29.6 (Phase 29 contract):** CLI Esc during `modal_program.is_some()` → `cancel_modal`. GUI mirrors verbatim.
- **D-29.8 (Phase 29 contract):** `PendingInput::XeqByName { acc, mode: XeqByNameMode }` with `Normal` | `CollectForModal` variants. GUI `pending_input.ts` mirrors verbatim; exhaustive-match invariant preserved.
- **D-29.9 (Phase 29 contract):** Post-dispatch auto-open of `XeqByName { mode: CollectForModal }` when `modal_program.is_some() && modal_program.requires_alpha_label() && pending_input.is_none()`. GUI mirrors verbatim in `App.tsx`'s `useEffect` after every `get_state` call.
- **ROADMAP Phase 31 SC-3:** `?`-overlay parallel-loads `docs/hp41-math1-functions.json` via Vite JSON-import. LOCKED — Phase 31 does NOT relitigate (Tauri-command alternative and async-fetch alternative ruled out by ROADMAP wording).
- **Plan 31-01 already partially shipped:** `op_display_name` arms in `hp41-gui/src-tauri/src/prgm_display.rs` covered every Phase-28 `Op` variant during Phase 28 plans 28-02..28-10 (verified in 29-CONTEXT.md). Phase 31 plan 01 reduces to a SC-4 grep verification test and `Op::Catalog(2)` body check.
- **GUI-07 stub-arm policy preserved:** Math Pac I uses XEQ-by-name only — `key_map::resolve` stub-error arm for `pi`/`polar_to_rect`/`rect_to_polar`/`beep`/`asn`/`catalog`/`view`/`xeq_prompt`/`gto_prompt`/`lbl_prompt` (v2.1 D-5) is unchanged.

### Discussed and decided in this session (D-31.1 — D-31.14)

#### Cancellation UX

- **D-31.1: R/S 3-way state-routed.** `App.tsx` `handleClick` and `handleKeyDown` for the R/S key (id `r_s`) inspect `CalcStateView` state in this priority order:
  1. `modal_program.is_some()` → `invoke('submit_modal')` (calls shared `hp41-core::ops::math1::submit_modal`)
  2. `is_running` → `invoke('request_cancel')` (flips `cancel_requested` AtomicBool)
  3. otherwise → existing v2.1 `invoke('run_stop')`
  Rejected the dedicated-cancel-button option because keyboard-first interaction matches the rest of the GUI; rejected the Esc-only option because R/S is hardware-faithful (real HP-41 R/S did exactly this 3-way thing). The routing decision lives entirely on the frontend; core's `Op::Stop` semantics are untouched. `pendingActionRef`/`busyRef` debounce pattern from v2.1 applies unchanged.
  - **Why:** preserves hardware fidelity AND modern GUI ergonomics; D-25.6 parity preserved (CLI's Phase 29 D-29.5 only handled the modal branch — the GUI fully exercises the same physical key that the real HP-41 used for all three semantics, while routing each to a different shared core function).

- **D-31.2: Esc universal cancel — parallel to R/S.** `App.tsx` `handleKeyDown` for Esc inspects state in this priority order:
  1. `modal_program.is_some()` → `invoke('cancel_modal')` (calls shared `hp41-core::ops::math1::cancel_modal`)
  2. `is_running` → `invoke('request_cancel')` (same Tauri command as R/S branch 2)
  3. `shiftActive` → `setShiftActive(false)` (existing v2.1 behavior)
  4. otherwise → no-op
  Rejected R/S-only because modern GUI users expect Esc to "back out" of anything; rejected Esc-only because R/S preserves hardware fidelity for keyboard users mirroring CLI behavior. Both keys route to the same `request_cancel` Tauri command for branch 2 — no duplicate IPC surface.
  - **Why:** modern GUI norm (Esc = cancel/back) PLUS hardware fidelity (R/S = the HP-41 cancel key) layered in parallel. Users who learn one mental model don't need to learn the other.

- **D-31.3: Visual feedback — existing toast only.** `HpError::Canceled` returned by `op_integ`/`op_solve`/`op_difeq` flows through the existing `types.rs::From<HpError> for GuiError` conversion → existing 2s toast via `App.tsx`'s `toastMsg` state. Rejected an LCD "CANCELED" message (would add a new Display14Seg render path coupled to error states); rejected silent return (user can't tell if cancel actually worked); rejected inline status banner (couples cancellation feedback to whatever-banner-modal-prompt-uses — Phase 31 doesn't have one). The `HpError::Canceled` `Display` impl ("CANCELED") is the toast text.
  - **Why:** zero new render code; matches v2.1 stub-error toast pattern users already know; minimum implementation risk.

- **D-31.4: Discovery — `is_running` annunciator only.** The existing v2.1 `is_running` annunciator (lit when `CalcStateView.is_running == true`) is the sole hint that R/S/Esc will cancel. No inline hint text, no R/S key tooltip. Rejected "press Esc to cancel" inline hint because it diverges from real HP-41 hardware (which had only the BUSY annunciator); rejected R/S tooltip because it doesn't help keyboard users.
  - **Why:** hardware-faithful (real HP-41 had only the BUSY annunciator as the busy/cancel-affordance signal); minimum new surface; users learn R/S/Esc behavior from `?`-overlay's `INTG` entry description and README.

#### Modal-prompt placement

- **D-31.5: LCD alternation — prompt ↔ entry_buf on Display14Seg.** `state.display` (the 12-char string `Display14Seg` renders) is computed in `commands.rs::handle_get_state` (Claude's discretion — could also live elsewhere) per this priority:
  1. `modal_program.is_some() && entry_buf.is_empty()` → `state.display = truncate_with_continuation(state.modal_prompt.as_deref().unwrap_or(""))`
  2. otherwise → existing v2.2 logic (entry_buf live, or formatted X register)
  This produces real-HP-41CV LCD-alternation behavior: prompt shows when waiting for input; entry buffer shows when user types; next prompt shows after R/S advances modal. Rejected dedicated-banner-above-LCD because it requires a new DOM row and diverges from real HP-41CV (which has only the LCD); rejected print-panel option because it conflicts with D-28.4 spirit (`print_buffer` reserved for PRX/PRA/PRSTK); rejected inline-strip-above-print-panel for the same reason.
  - **Why:** maximum HP-41CV fidelity. The real device has a single 12-char LCD that alternates between prompt and typed value — this is exactly what real-hardware users would expect. D-29.4 ("LCD shows entry_buf live during entry") is preserved — that decision speaks to during entry; before entry begins the LCD is free to show the prompt.

- **D-31.6: Truncate prompts >12 chars with HP-41 `≡` continuation marker.** Long prompts in Math Pac I (`FUNCTION NAME?` = 14 chars, `NO. SAMPLES=?` = 13 chars, `1ST COEFF=?` = 11 chars [fits], `STEP SIZE=?` = 11 chars [fits], `DEGREE=?` = 8 chars [fits]) render as `FUNCTION NAM≡` and `NO. SAMPLES≡` respectively. The `≡` character (U+2261) is the HP-41 standard continuation marker. Rejected AVIEW-style scrolling because it adds a scrolling state machine + setInterval ticker + adds significant Phase 31 surface (deferred to v3.1 polish); rejected widening the LCD to 16 chars because it deviates from real-HP-41 hardware (real LCD is 12 chars, no exceptions).
  - **Why:** hardware-faithful (`≡` is the documented HP-41 truncation marker); minimum implementation risk; AVIEW scrolling can land as v3.1 polish if user feedback demands it.

- **D-31.7: No accessibility fallback for truncated prompts.** No print-panel header showing the full prompt; no LCD tooltip on hover; no separate status row. The truncated LCD is the entire user-facing surface for prompt text. Rejected the pinned-header-line-above-PRX option (recommended originally) because it diverges from real HP-41CV (which has no print-panel context for prompts); rejected tooltip (doesn't help non-mouse users); the user explicitly chose maximum hardware fidelity ("as close as possible to real HP-41CV").
  - **Why:** purest HP-41CV fidelity. D-28.4 (`print_buffer` reserved for PRX/PRA/PRSTK output ONLY) trivially preserved. Users learn the few long Math Pac I prompts from the `?`-overlay's full-text descriptions.

#### `?`-overlay sectioning

- **D-31.8: Two top-level collapsible sections — "HP-41CV (built-in)" + "Math 1 Pac (XROM 7)".** `HelpOverlay.tsx` wraps the existing per-category rendering inside a two-section parent grouping. Both sections expanded by default (open the overlay → see everything). The section header line carries the XROM-module identity ("XROM 7") for the Math 1 Pac entry. Rejected single-combined-list because it loses module identity (and CAT 2 vs `?`-overlay would diverge in shape); rejected tabs because they require an extra user action to discover Math Pac I existence; rejected the with-filter-input variant because the user picked the cleaner version without filter.
  - **Why:** clear module identity in the overlay; matches the JSON-pipeline separation locked in C-28.3/ADR-005; anticipates v3.1+ pacs (Stat 1, Time, Advantage) adding new top-level sections without touching the existing two. Future v3.1+ pacs add their own collapsible top-level section sibling.

- **D-31.9: JSON's per-program categories preserved as 2nd-level headers.** Within the "Math 1 Pac" section, the JSON's "Math1 Hyperbolics" / "Math1 Complex" / "Math1 Matrix" / etc. categories render as 2nd-level headers (or the equivalent of the existing v2.2 category-grouping rendering applied to the new entries). Math Pac I JSON entries authored in Phase 29 D-29.2 used these per-program categories per the Claude's-discretion recommendation.
  - **Why:** finer-grained groupings help users find functions by topic; preserves the JSON structure as the single source of truth without overlay-layer category-flattening logic.

- **D-31.10: Vite JSON-import for parallel-load (ROADMAP SC-3 locked).** `HelpOverlay.tsx` (or a sibling `help_data.ts`) statically imports both JSON files at build time:
  ```typescript
  import hp41cvJson from '../../docs/hp41cv-functions.json';
  import math1Json from '../../docs/hp41-math1-functions.json';
  ```
  Vite path resolution needs verification — `docs/` is outside `hp41-gui/src/`. Planner either adds a Vite alias / `resolve.alias` entry in `vite.config.ts` (cleanest) or symlinks the JSON files into `hp41-gui/src/` (uglier but simpler). Both file paths must be reachable at build time without a network fetch. Tauri command alternative (`get_help_data`) was rejected by ROADMAP wording.
  - **Why:** static import = build-time validation (malformed JSON fails the build, parallel to the Phase 29 D-25.16 hard-build-blocker pattern on the Rust side); zero IPC roundtrip; matches CLI's `include_str!` philosophy on the GUI side.

- **D-31.11: No filter input on the overlay.** Users navigate by section + category headers; 185 entries (130 v2.2 + 55 Math Pac I) is within scannable range with two top-level sections. Rejected adding a text-input filter because the user explicitly chose the cleaner option without filter. Deferred for v3.1+ when entry count may grow further (Pitfall 13 mitigation if it becomes a real problem).
  - **Why:** minimum new UI surface; matches the user's "clean overview" preference; future filter addition is non-breaking.

#### CATALOG 2 rendering

- **D-31.12: Hardware-faithful PSE scroll via `state.print_buffer`.** `Op::Catalog(2)` follows the v2.2 Phase 22 CAT 1 pattern: pushes one line per (module, function) entry into `state.print_buffer` with a ~500ms PSE delay between lines; R/S pauses/resumes; any other key cancels. The user sees lines appear in the existing scrollable print panel below the LCD. Rejected the scrollable-modal overlay because it diverges from real HP-41 CAT 2 (which was a print-scroll UX); rejected piping into the `?`-overlay because it conflates two UX channels.
  - **Why:** maximum HP-41 hardware fidelity; reuses v2.2 Phase 22 CAT 1 infrastructure (print_buffer + PSE-step pattern) — zero new render path.

- **D-31.13: First line is module header `XROM Math 1`; function names follow.** CAT 2 starts with a per-loaded-module header line ("XROM Math 1" for v3.0 — only Math Pac I is loaded). Each subsequent line is one function name from `MATH_1.ops` in registration order (matches `xrom_resolve` traversal order). Future v3.1+ pacs add their module-header + function-name block after Math 1's block. Rejected the verbose summary header ("XROM 7 MATH 1 (55 fns)") in favor of the simpler form matching the v2.2 CAT 1 visual style.
  - **Why:** consistency with v2.2 CAT 1 line shape; per-module header makes the listing self-describing when multiple pacs are loaded in v3.1+.

- **D-31.14: R/S pauses/resumes; other keys cancel — matches v2.2 CAT 1.** During PSE scroll, R/S behavior follows the v2.2 CAT 1 pause/resume convention; any other key cancels the scroll mid-listing. This is the standard HP-41 CAT-N behavior. No new key-handling code is needed — the v2.2 PSE-step infrastructure already handles this.
  - **Why:** users who know v2.2 CAT 1 immediately know CAT 2; no new mental model.

### Claude's Discretion

- **Implementation site for the LCD-alternation routing (D-31.5):** three plausible locations:
  1. `commands.rs::handle_get_state` — computes `state.display` per-call; GUI-only change; CLI unaffected
  2. `hp41-core/src/state.rs::display()` (or equivalent) — both CLI and GUI inherit; would make CLI's Phase 29 D-29.3 status-bar redundant (a follow-up CLI refactor)
  3. `hp41-gui/src/Display14Seg.tsx` — frontend reads `state.modal_prompt` + `state.entry_buf` + `state.modal_program` directly; least DRY
  Recommendation: option 1 — surgical, GUI-local, preserves the CLI Phase 29 status-bar behavior unchanged. Option 2 is the better long-term path (single source of truth) and can land as a quick-task backport after Phase 31 ships.
- **`Op::Catalog(2)` body in Phase 28 — stub vs full:** ROADMAP Phase 31 SC-3 references "(new `Op::Catalog(2)` arm in Phase 28)" but doesn't specify whether Phase 28 shipped the full body or just the variant declaration with a `todo!()` body. Planner MUST verify in research-prep. If Phase 28 shipped a stub, Phase 31 takes a surgical `hp41-core` exception (the only acceptable exception to the "Phase 31 is GUI-only" build stage), analogous to v2.2 Phase 25-03's `builtin_card_op` 4→12 extension. Visibility stays `pub(super) fn`; no API widening.
- **Vite path-alias vs symlink for cross-directory JSON imports:** `vite.config.ts` `resolve.alias` entry like `{ '@docs': path.resolve(__dirname, '../docs') }` is cleaner than a symlink and survives clean checkouts. Planner picks.
- **`request_cancel` Tauri command signature:** likely `fn request_cancel(state: State<'_, AppState>) -> Result<(), GuiError>` — sets `state.cancel_requested.store(true, Ordering::Relaxed)`; idempotent; no-op when `is_running == false` (silent — matches Esc-with-nothing-to-cancel philosophy). Permission file at `permissions/request-cancel.toml` per v2.0 Tauri v2.11 inline-command pattern; reviewed by `scripts/check-tauri-permissions.sh` CI gate (Pitfall 21).
- **`PendingInput::XeqByName { acc, mode }` port to TypeScript:** the `XeqByNameMode` enum is currently a Rust-side enum (Phase 29 D-29.8). In `hp41-gui/src/pending_input.ts`, it's a TypeScript string-literal union `'normal' | 'collect-for-modal'`. Planner picks the exact serialization shape that matches the IPC-serialized CalcStateView field.
- **HP-41 `≡` continuation marker character source:** `U+2261` (Unicode "IDENTICAL TO") is the HP-41 standard. Display14Seg's SVG font already includes it via Phase 26's 14-segment SVG font addition (SKIN-04). Planner verifies the glyph renders correctly in the truncated position; if not, fall back to ASCII `>` or `...` with a doc note.
- **PSE delay timing for CAT 2:** match v2.2 CAT 1 exactly (~500ms, verify with planner research-prep against `op_catalog` in `hp41-core/src/ops/program.rs`). Tunable in future polish if user feedback says it's too slow / too fast.
- **`?`-overlay default expand/collapse state persistence:** both sections expanded on first open per D-31.8. Whether to persist the user's collapse/expand choice across overlay open/close cycles or always reset to "both expanded" — Claude's discretion. Recommendation: reset on each open (simpler; no localStorage state).
- **Sort order within categories:** alphabetical vs OM-manual order. The v2.2 overlay uses first-appearance JSON order. Phase 29 likely authored the JSON in some order; if it's not alphabetical, planner can either re-sort in the JSON (Phase 31 surgical edit) or sort in the overlay renderer at display time. Recommendation: alphabetical within each category for consistency.
- **ALPHA annunciator behavior during a modal that collects an alpha-label:** when the post-dispatch auto-open fires for `FUNCTION NAME?` collection, the GUI enters the `XeqByName { mode: CollectForModal }` state — alpha-input mode. Whether to light the ALPHA annunciator matches the existing v2.2 XEQ-by-name modal behavior. Planner verifies the v2.2 path lights the ALPHA annunciator during normal `XEQ "<name>"` collection; if so, `CollectForModal` mode does the same; if not, neither does.
- **Precedence when `pending_input.is_some()` AND `modal_program.is_some()` simultaneously:** mirrors Phase 29's CLI discretion bullet — modal_program likely wins; tests assert both states are well-defined. Planner adds a regression test.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-level (always-on)

- `.planning/PROJECT.md` — v3.0 milestone scope; v3.0 additions block (Phase 28/29/30 sub-sections populated, Phase 31 stub `(in progress)` to be filled by Phase 31 ship)
- `.planning/REQUIREMENTS.md` — Phase 31 maps to GUI-01..07 (7 requirements)
- `.planning/ROADMAP.md` — Phase 31 section lines 154–185 (5 success criteria, 5 plans, notable risks/decisions); cross-cutting constraints lines 35–45
- `.planning/STATE.md` — accumulated context (Phase 30 complete, Phase 31 ready to plan, 2026-05-17)
- `CLAUDE.md` (repo root) — `### v2.2 additions` block + `### v3.0 additions (Math Pac I Emulation, Phases 28–30 — 31–32 IN PROGRESS)` block; v3.0 Phase 31 sub-section to be populated by this phase's ship

### Phase 28 / 29 / 30 (the contract Phase 31 builds on)

- `.planning/phases/28-xrom-framework-math-pac-i-core-ops/28-CONTEXT.md` — Phase 28 decisions; D-28.4 / D-28.5 / D-28.6 / D-28.7 carry forward
- `.planning/phases/28-…/28-RESEARCH.md` — Math Pac I behavioral inventory; cancellation/Pitfall-11 mitigation design
- `.planning/phases/29-cli-integration/29-CONTEXT.md` — Phase 29 decisions; D-29.5 / D-29.6 / D-29.8 / D-29.9 are the contract Phase 31 mirrors verbatim
- `.planning/phases/30-documentation-adrs/30-CONTEXT.md` — ADR template + divergence catalog conventions; Phase 31 may add D-31-NN entries if new divergences surface

### hp41-core public surface (the Math Pac I framework Phase 31 consumes through `hp41-gui`)

- `hp41-core/src/ops/math1/xrom.rs` — `xrom_resolve(name, modules) -> Option<Op>`, `MATH_1: XromModule` constant; Phase 31's `App.tsx` XEQ-by-name calls into this through `commands.rs` (NO duplicate resolver in `key_map.rs`)
- `hp41-core/src/ops/math1/modal.rs` — `ModalProgram` enum + per-program step enums + `requires_alpha_label()` method (Phase 29 addition); Phase 31's `App.tsx` post-dispatch auto-open reads this via CalcStateView
- `hp41-core/src/ops/math1/mod.rs` — `submit_modal(state: &mut CalcState) -> HpResult<()>` + `cancel_modal(state: &mut CalcState)` (Phase 29 additions); Phase 31's `App.tsx` invokes both through new Tauri command thunks
- `hp41-core/src/state.rs` — `CalcState` fields: `cancel_requested: Arc<AtomicBool>` (`#[serde(skip)]`), `modal_program`, `modal_prompt`, etc.; Phase 31's `request_cancel` Tauri command flips the AtomicBool
- `hp41-core/src/ops/program.rs::op_catalog` (or equivalent) — v2.2 Phase 22 CAT 1 pattern source for CAT 2's PSE-step behavior; Phase 31's CAT 2 mirrors verbatim. Visibility of any Phase 28 stub or full body MUST be verified by planner research-prep

### hp41-cli (Phase 29 reference implementation Phase 31 mirrors)

- `hp41-cli/src/keys.rs::xeq_by_name_local_resolve` — Phase 29 D-29.1 final-fallback into `xrom_resolve` is the reference path; GUI's `commands.rs` uses the same shared core function (different call site, identical resolver chain order)
- `hp41-cli/src/app.rs::handle_key` — Phase 29 D-29.5 / D-29.6 / D-29.9 modal interception + R/S submit + Esc cancel + post-dispatch auto-open patterns; Phase 31's `App.tsx` `handleClick` / `handleKeyDown` mirror verbatim
- `hp41-cli/src/app.rs::PendingInput::XeqByName { acc, mode: XeqByNameMode }` — Phase 29 D-29.8 enum extension; Phase 31's `hp41-gui/src/pending_input.ts` ports verbatim
- `hp41-cli/src/help_data.rs` — Phase 29 D-29.2 second `OnceLock` + merged accessor pattern; conceptual analogue for Phase 31's Vite parallel-load (different mechanism, same goal — both JSON files loaded at build/compile time, both gate against malformed JSON)
- `hp41-cli/src/ui.rs::pending_prompt` — Phase 29 D-29.3 status-bar rendering for `modal_prompt`; Phase 31 chooses LCD alternation instead (D-31.5), the CLI status-bar continues working in parallel

### hp41-gui (Phase 31 surface)

- `hp41-gui/src/App.tsx` — React root; Phase 31 extends `handleClick` + `handleKeyDown` + `invokeForKey` with R/S 3-way state routing + Esc universal cancel + post-dispatch auto-open hook + toast lifecycle (HpError::Canceled support already inherited)
- `hp41-gui/src/Keyboard.tsx` — `KEY_DEFS` UNCHANGED in Phase 31 (D-28.6 — Math Pac I uses XEQ-by-name only); SHIFT key handling unchanged
- `hp41-gui/src/Display14Seg.tsx` — renders `state.display` (12-char string); Phase 31 routes `modal_prompt` into `state.display` upstream (D-31.5); Display14Seg itself doesn't change behavior beyond what it does in v2.2 + `data-testid="lcd-display"` from Phase 27
- `hp41-gui/src/HelpOverlay.tsx` — Phase 31 wraps existing category-grouped rendering inside two top-level collapsible sections (D-31.8)
- `hp41-gui/src/pending_input.ts` — Phase 31 adds `XeqByNameMode` discriminator (D-29.8 port from CLI)
- `hp41-gui/src/help_data.ts` — Phase 31 adds Vite JSON-import for `docs/hp41-math1-functions.json`; merged accessor pattern parallel to CLI Phase 29 D-29.2
- `hp41-gui/vite.config.ts` — Phase 31 adds path alias / `resolve.alias` for cross-directory JSON imports
- `hp41-gui/src-tauri/src/commands.rs` — Phase 31 adds `request_cancel` thunk + extends `handle_get_state` with LCD-alternation routing (D-31.5)
- `hp41-gui/src-tauri/src/lib.rs` — Phase 31 registers `request_cancel` in `generate_handler!`
- `hp41-gui/src-tauri/src/key_map.rs` — Phase 31 makes NO new arms (D-28.6 + GUI-07); SC-4 invariant trivially preserved
- `hp41-gui/src-tauri/src/types.rs` — Phase 31 verifies `From<HpError> for GuiError` already converts `HpError::Canceled` (Phase 28 D-28.9 — confirm); if not, add the arm
- `hp41-gui/src-tauri/src/prgm_display.rs` — Phase 31 verifies all Phase-28 `Op` variants have `op_display_name` arms (already shipped per Phase 29 CONTEXT verification); no edits expected, only test
- `hp41-gui/src-tauri/permissions/request-cancel.toml` — NEW file per Tauri v2.11 inline-command permission pattern (v2.0 Plan 14 conventions)
- `hp41-gui/src-tauri/Cargo.toml` — NO new dependencies expected

### v2.2 baseline (pattern reservoir)

- `hp41-cli/src/app.rs::PendingInput::XeqByName` — v2.2 alpha-collection state machine
- `hp41-gui/src/App.tsx::invokeForKey` + `extractErrMessage` (v2.1) — single source of truth for resolving effective id to Tauri command; Phase 31 extends with `request_cancel` branch
- `hp41-gui/src/App.tsx::toast` machinery (v2.1) — 2s auto-dismiss; HpError::Canceled inherits unchanged
- `hp41-core/src/ops/program.rs::op_catalog` (v2.2 Phase 22 / FN-MEM-01) — CAT 1 implementation; CAT 2 mirrors verbatim
- `hp41-gui/src-tauri/permissions/run-stop.toml` (v2.1) — template for `request-cancel.toml` shape
- `scripts/check-tauri-permissions.sh` (v2.0 / Pitfall 21) — CI gate covering `generate_handler!` ↔ `permissions/*.toml` parity

### JSON pipeline (canonical pattern source)

- `docs/hp41cv-functions.json` — v2.2 baseline JSON (130 entries); Phase 31 Vite-imports unchanged
- `docs/hp41-math1-functions.json` — Phase 29 D-29.1 authored (~55 entries); Phase 31 Vite-imports read-only
- `docs/hp41-math1-divergences.md` — Phase 28 + Phase 30 catalog of D-30-NN entries; Phase 31 may add D-31-NN entries only if new divergences surface during implementation
- `docs/adr/v3.0-001-op-strategy.md` (Phase 30) — Op-strategy A locked
- `docs/adr/v3.0-002-user-callback-policy.md` (Phase 30) — user-callback strict-reject policy
- `docs/adr/v3.0-005-json-pipeline.md` (Phase 30) — separate JSON file shape

### HP Math Pac I primary source (HP-copyrighted — DO NOT redistribute)

- HP-41C/CV Math Pac Owner's Manual (HP 00041-90034, 1979) — relevant pages for Phase 31:
  - p.13: "Press R/S to continue" — D-28.5 hardware ground truth; D-29.5 (CLI) + D-31.1 (GUI R/S submit branch) inherit
  - Prompt mnemonic full text (`FUNCTION NAME?`, `NO. SAMPLES=?`, etc.) — for Phase 31's `≡` truncation verification + planner research-prep on any prompt > 12 chars

### Documentation context (Phase 30 inherited)

- `docs/hp41-math1-function-matrix.md` — Phase 30 generated from `hp41-math1-functions.json`; reference for the `?`-overlay's per-program category structure
- README.md v3.0 soft-claim (Phase 30 D-30.9) — sets expectation that Math Pac I is reachable in CLI + GUI; Phase 31 makes the GUI side true

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`hp41-gui/src/App.tsx::invokeForKey` + `extractErrMessage` (v2.1)** — Phase 31 extends `invokeForKey` with a `request_cancel` branch keyed on `(id === 'r_s' || event.key === 'Escape') && state.is_running`. `extractErrMessage` already handles object-shaped errors and continues working for `HpError::Canceled`.
- **`hp41-gui/src/App.tsx::toast` machinery (v2.1)** — `toastMsg` state + 2s `setTimeout` + CSS `.toast` + `@keyframes toast-fade`. Phase 31 needs zero changes — `HpError::Canceled` flows through the same path.
- **`hp41-gui/src/App.tsx::busyRef`/`pendingActionRef` debounce (v2.0)** — two-layer guard against concurrent `invoke()` calls; Phase 31's R/S 3-way + Esc handlers use the same pattern unchanged.
- **`hp41-gui/src/Display14Seg.tsx` SVG 14-segment font (Phase 26 / SKIN-04)** — already includes the full 12-char alphanumeric set; planner verifies the `≡` continuation marker (U+2261) renders. If the glyph is absent, fallback to ASCII per Claude's discretion.
- **`hp41-gui/src/HelpOverlay.tsx` category-grouped rendering (Phase 26 / SKIN-05)** — existing two-level (category → entry) structure; Phase 31 wraps in a third-level (section → category → entry) grouping for Math 1 Pac.
- **`hp41-gui/src-tauri/src/commands.rs::handle_get_state` (v2.0)** — already computes `CalcStateView` from `CalcState`; Phase 31 extends with the LCD-alternation routing (D-31.5). The `print_buffer` drain pattern at the end is unchanged.
- **`hp41-gui/src-tauri/src/types.rs::From<HpError> for GuiError` (v2.0)** — Phase 28 added `HpError::Canceled` to the enum; planner verifies the `From` impl handles it (likely already does via a catch-all or per-variant arm with the existing `Display` impl).
- **`hp41-gui/src-tauri/permissions/run-stop.toml` (v2.1)** — shape template for `request-cancel.toml`: `[[permission]] identifier = "allow-request-cancel"` + `commands.allow = ["request_cancel"]`.
- **`hp41-core/src/ops/program.rs::op_catalog` (v2.2 Phase 22 / FN-MEM-01)** — CAT 1 implementation: pushes lines into `state.print_buffer` with PSE-step delay between lines. Phase 31 CAT 2 mirrors the exact pattern (only the per-line content changes — XROM modules + functions instead of user programs).
- **Phase 29 CLI mirrors (28-CONTEXT.md / 29-CONTEXT.md)** — every Phase 31 GUI behavior has a CLI reference implementation Phase 29 already shipped. Cross-reference for parity verification.

### Established Patterns

- **D-25.6 CLI ↔ GUI parity:** every behavior added in Phase 31 routes through shared `hp41-core` functions — `xrom_resolve`, `submit_modal`, `cancel_modal`, `requires_alpha_label`, `request_cancel`. No GUI-only Math Pac I logic; `key_map::resolve` stub-error policy preserved (GUI-07).
- **v2.1 functional setState for state machines:** R/S 3-way + Esc cancel + shiftActive interactions use `setState(prev => ...)` form to avoid stale closures (v2.0 Pitfall 4).
- **v2.0 busyRef + pressedKey debounce:** Phase 31 extends both refs to cover the new `request_cancel` path; `pendingActionRef.current = 'r_s_cancel'` (or similar) prevents double-cancel.
- **v2.2 Tauri command + `permissions/<kebab>.toml` pattern:** Phase 31 follows for `request_cancel`; `scripts/check-tauri-permissions.sh` CI gate (Pitfall 21) covers the new command.
- **Phase 28 `cancel_requested: Arc<AtomicBool>` clone-and-poll pattern:** core ops clone the Arc at op entry, release the Mutex during computation, poll every 64 samples; Phase 31 only triggers the AtomicBool flip from the frontend.
- **v2.2 D-25.16 hard-build-blocker on malformed JSON:** Vite JSON-import gives the GUI side the same compile-time guarantee — malformed `hp41-math1-functions.json` fails the build per the Phase 29 D-29.2 pattern on the Rust side.
- **v2.2 `PendingInput` hybrid struct-variants (D-25.11):** the `XeqByName { acc, mode }` extension Phase 29 D-29.8 added ports verbatim to TypeScript as a discriminated union; FN-GUI-04 exhaustive match preserved (no `default:` catch-all in pending_input.ts switch statements).
- **Op variants land before consumers:** Phase 31 adds ZERO new `Op::*` variants — the surface is one new Tauri command (`request_cancel`) + frontend state-machine extensions + (possibly) one surgical `Op::Catalog(2)` body fill if Phase 28 stubbed it.

### Integration Points

- **`App.tsx::handleClick` (R/S key):** Phase 31 inserts a state-aware branch at the top: if `id === 'r_s'`, inspect `state.modal_program.is_some()` / `state.is_running` and dispatch to `submit_modal` / `request_cancel` / existing `run_stop` accordingly. `pending_input` routing block must REMAIN ABOVE this (D-07 never-discard invariant).
- **`App.tsx::handleKeyDown` (Esc key):** Phase 31 inserts a priority chain: `modal_program.is_some()` → `cancel_modal`; `is_running` → `request_cancel`; `shiftActive` → clear (existing v2.1); else no-op. SAME order applies as for R/S in branches 1+2.
- **`App.tsx` post-dispatch effect (D-29.9 mirror):** `useEffect` that runs after every `get_state` call: if `state.modal_program.is_some() && requires_alpha_label(state.modal_program) && pending_input === null`, set `pending_input = { type: 'xeq_by_name', acc: '', mode: 'collect-for-modal' }`. The `requires_alpha_label` helper either reads a CalcStateView-exposed boolean or hard-codes the variant check (mirror CLI shape).
- **`commands.rs::handle_get_state` LCD routing:** computes `state.display` per D-31.5 priority — `modal_prompt` truncated with `≡` when `entry_buf.is_empty() && modal_program.is_some()`; else existing v2.2 logic.
- **`commands.rs::request_cancel` Tauri command:** new function; acquires `AppState` Mutex briefly; flips `state.cancel_requested.store(true, Ordering::Relaxed)`; returns Ok(()). Idempotent. No `print_buffer` drain (cancellation is silent until the long-running op returns `HpError::Canceled`).
- **`lib.rs::generate_handler!`:** Phase 31 adds `request_cancel` to the macro invocation. CI gate `check-tauri-permissions.sh` verifies the matching `permissions/request-cancel.toml`.
- **`HelpOverlay.tsx`:** Phase 31 wraps the existing per-category rendering inside two-section grouping; categories whose entries have `xrom.module === "Math 1"` go in the Math 1 section, others in the HP-41CV section. (Per-entry `xrom` field from C-28.3 JSON schema is the discriminator.)
- **`pending_input.ts`:** Phase 31 adds the `mode` discriminator to `XeqByName`; `keyDown` handler at the Enter-commit step branches on mode — `'normal'` → resolve via XEQ chain; `'collect-for-modal'` → invoke `submit_modal_with_label` Tauri command (or equivalent — the label-passing path Phase 29 D-29.7 designed).
- **`Vite.config.ts`:** Phase 31 adds `resolve.alias` entry for `@docs` → `path.resolve(__dirname, '../docs')` (or equivalent). Cleanly enables `import math1Json from '@docs/hp41-math1-functions.json'`.
- **`Display14Seg.tsx`:** Phase 31 makes NO source changes — it already renders `state.display`. Phase 27's `data-testid="lcd-display"` continues to work; the v3.0 Phase 32 / FN-QUAL-03 E2E test extension can assert that after `XEQ "MATRIX" Enter`, `[data-testid="lcd-display"]` reads "ORDER=?".

</code_context>

<specifics>
## Specific Ideas

- **HP-41 `≡` continuation marker:** U+2261 (IDENTICAL TO), the documented HP-41 ALPHA-truncation marker. Used in `Display14Seg` when `modal_prompt.len() > 12`; rendered as the 12th character with the first 11 characters of the prompt preceding it. Example: `FUNCTION NAME?` (14 chars) → `FUNCTION NAM≡` (12 chars).
- **R/S 3-way priority order:** `modal_program.is_some()` > `is_running` > `run_stop` (default). Inspected in `App.tsx::invokeForKey('r_s')` via `CalcStateView`. Mirror CLI's Phase 29 D-29.5 (modal branch only) + add the new `is_running` branch.
- **Esc universal cancel priority order:** `modal_program.is_some()` > `is_running` > `shiftActive` > no-op. Inspected in `App.tsx::handleKeyDown(event.key === 'Escape')`. Mirror CLI's Phase 29 D-29.6 (modal branch only) + add the new `is_running` branch + preserve v2.1 shiftActive branch.
- **PSE delay for CAT 2:** ~500ms per line (verify against v2.2 Phase 22 CAT 1 timing in `op_catalog`). HP-41 hardware speed was actually slower (~1s per line); 500ms is a reasonable emulator compromise.
- **CAT 2 module header line format:** `XROM Math 1` (followed by per-function lines like `SINH`, `COSH`, etc.). Matches v2.2 CAT 1 line shape; minimum new visual style.
- **Vite path alias:** likely `@docs` → `../docs` in `vite.config.ts`'s `resolve.alias`. Enables `import math1Json from '@docs/hp41-math1-functions.json'` syntactic sugar. Alternative: relative `../../docs/...` import (less clean).
- **`?`-overlay default section state:** both sections expanded on first open per D-31.8. Per-session collapse/expand state — Claude's discretion (recommendation: reset on each overlay open).
- **`request_cancel` no-op when `is_running == false`:** silent return Ok(()) — matches Esc-with-nothing-to-cancel philosophy. Not an error. CI gate `check-tauri-permissions.sh` confirms permission file matches command registration.

</specifics>

<deferred>
## Deferred Ideas

- **CLI Esc/Ctrl-C → `request_cancel` backport** — Phase 29 CLI shipped before Phase 31 added the `request_cancel` Tauri command and the shared frontend trigger pattern; CLI currently has no long-compute cancel path. Once Phase 31 ships `request_cancel`, a quick-task can add Esc/Ctrl-C → `request_cancel` to `hp41-cli/src/app.rs::handle_key` for D-25.6 parity. Not blocking Phase 31.
- **Backport LCD-alternation routing into `hp41-core::state::display()`** — Phase 31's GUI gets hardware-faithful LCD alternation (D-31.5) via `handle_get_state` (Claude's discretion location); CLI's Phase 29 D-29.3 status-bar continues working. A quick-task can move the routing into `hp41-core` so both CLI and GUI share the same display computation, deprecating the CLI status-bar in favor of direct LCD prompt rendering. Future polish.
- **AVIEW-style scrolling for prompts > 12 chars** — Phase 31 ships truncation with `≡` marker (D-31.6). v3.1 polish can replace truncation with scrolling: the `Display14Seg` scrolls the prompt right-to-left every ~500ms while `entry_buf.is_empty() && modal_program.is_some()`. Requires a scrolling state machine + setInterval ticker; deferred until user feedback demands it.
- **Filter input on `?`-overlay** — Phase 31 ships two-section grouping without filter (D-31.11). If v3.1+ pacs push entry count past discoverability threshold (Pitfall 13), add a text-input filter to the overlay. Future v3.x polish.
- **Tab UI for `?`-overlay when v3.1+ pacs land** — currently two top-level sections handle Math 1 Pac + HP-41CV. When v3.1+ adds Stat 1 / Time / Advantage, four+ top-level sections may push the overlay past comfortable scroll length. Tab UI is the structural fix; Phase 31 doesn't need it yet.
- **CAT 2 module-header verbosity** — Phase 31 ships simple "XROM Math 1" headers (D-31.13). A future polish could expand to "XROM 7 MATH 1 (55 fns)" if user research shows it helps. Cosmetic — defer.
- **`Op::Catalog(2)` body — verify Phase 28 ship state** — research-prep MUST determine whether Phase 28 shipped full body or stub. If stub, Phase 31 takes a surgical hp41-core exception. If full body, Phase 31 is purely GUI-side wiring. Either path is acceptable; the determination shapes Plan 31-04 (or 31-05) scope.
- **ALPHA annunciator behavior during modal alpha-label collection** — Phase 31 mirrors whatever the v2.2 XEQ-by-name modal does today. If v2.2 doesn't light the ALPHA annunciator during name entry, Phase 31 doesn't either. Consistency over net-new.
- **Tauri command `submit_modal_with_label`** — Phase 29 D-29.7 designed an `advance_with_label(label: &str)` path for the post-dispatch auto-open in `CollectForModal` mode. Phase 31's GUI needs a Tauri command thunk for this; whether it lives as a new `submit_modal_with_label` command or as an extension of existing `submit_modal` taking an optional label parameter — planner picks (recommendation: separate command for clarity, follows the v2.1 single-purpose-Tauri-command pattern).

</deferred>

---

*Phase: 31-gui-integration*
*Context gathered: 2026-05-17*
