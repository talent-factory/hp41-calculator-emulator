# Phase 18: Program Listing & CI/CD - Context

**Gathered:** 2026-05-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver PRGM mode program listing in the GUI: when the user enters PRGM mode, a panel appears below the keyboard showing the complete program listing with step numbers and mnemonic labels (matching `hp41-cli` format). SST and BST navigation step through the program and highlight the current step. Also add a cross-platform CI job (`.github/workflows/ci-gui.yml`) that builds `hp41-gui/src-tauri` on macOS, Windows, and Ubuntu, triggered only on changes to `hp41-gui/**` or `hp41-core/**`, independent from the existing CLI CI.

This phase delivers PROG-01 and completes the v2.0 milestone.

</domain>

<decisions>
## Implementation Decisions

### Program Data in CalcStateView

- **D-01:** Add `program_steps: Vec<String>` and `pc: usize` to `CalcStateView` unconditionally — always populated from `CalcState.program` and `CalcState.pc`. The 300-byte size assertion in `types.rs` tests only on `CalcState::new()` (empty program); update the test comment to document this scope rather than adding a total-size assertion that breaks with real programs.
- **D-02:** `program_steps` is a `Vec<String>` where each entry is the formatted step string (`"{pc:03} {op_name}"` format from `prgm_display.rs`). Index 0 = step 000. The Rust backend formats all steps before sending; React only renders strings.
- **D-03:** Copy `hp41-cli/src/prgm_display.rs` into `hp41-gui/src-tauri/src/prgm_display.rs` and use it to format `program_steps` in `CalcStateView::from_state()`. Matches the Phase 17 persistence.rs copy pattern. The GUI copy may evolve independently.

### SST/BST Dispatch

- **D-04:** SST and BST are implemented as **two new Tauri commands**: `sst_step` and `bst_step`. Each locks `AppState`, directly manipulates `CalcState.pc` (increment by 1 capped at `program.len()`, or `saturating_sub(1)` for BST), drains `print_buffer`, and returns `CalcStateView`. Mirrors the CLI's F7/F8 special-case approach — not routed through `Op` dispatch.
- **D-05:** Register `sst_step` and `bst_step` in `lib.rs`'s `invoke_handler`. Add pure-Rust helpers `handle_sst()` and `handle_bst()` in `commands.rs` (same testability pattern as `handle_op` / `handle_get_state`).
- **D-06:** SST/BST SVG key IDs: set `id: 'sst'` and `id: 'bst'` in `Keyboard.tsx KEY_DEFS`. Click handler calls `invoke<CalcStateView>('sst_step')` / `invoke<CalcStateView>('bst_step')` via `App.tsx`'s `handleClick`.
- **D-07:** Physical keyboard bindings: add `F7 → sst_step` and `F8 → bst_step` to `resolveKeyId()` in `App.tsx`. Matches CLI behavior (D-15 from hp41-cli).

### Program Listing Panel UI

- **D-08:** The program listing panel appears **below the `<Keyboard />` component** inside `.calculator`, auto-shown when `calcState.annunciators.prgm === true`, hidden when `prgm_mode` is false. No manual toggle needed — follows PRGM mode toggle exactly.
- **D-09:** The panel is a scrollable `<div>` (e.g., `max-height: 160px`, `overflow-y: auto`) containing one `<div>` per step. The step at index `calcState.pc` receives a highlight class (e.g., `className="step-row step-active"`). Auto-scroll to the active step whenever `pc` changes (via `useEffect` + `scrollIntoView`).
- **D-10:** Window height grows from **700 to 900** in `tauri.conf.json` (`windows[0].height: 900`). The listing panel needs vertical space below the keyboard; extending the window is cleaner than overlapping or scrolling the calculator container.
- **D-11:** The panel header shows `"PRGM"` and the step count (`"PRGM — 12 steps"`). No close button — the panel disappears automatically when the user exits PRGM mode (by pressing the PRGM key again).

### CI Job for Tauri GUI

- **D-12:** New file **`.github/workflows/ci-gui.yml`** — separate from the existing `ci.yml` (which covers CLI and hp41-core). Ensures SC-5: a GUI build failure never blocks CLI CI.
- **D-13:** Path filter:
  ```yaml
  on:
    push:
      paths: ['hp41-gui/**', 'hp41-core/**']
    pull_request:
      paths: ['hp41-gui/**', 'hp41-core/**']
  ```
- **D-14:** CI job builds `cargo build --release` of `hp41-gui/src-tauri` on all three platforms (macOS, Windows, Ubuntu). Also runs `npm install && npx tsc --noEmit` for the TypeScript frontend check. No full `tauri build` bundle — `cargo build --release` satisfies SC-4 ("build completes without error").
- **D-15:** Linux CI step installs WebKit system dependencies before Cargo steps:
  ```yaml
  - name: Install Linux system deps
    if: matrix.os == 'ubuntu-latest'
    run: sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
  ```
- **D-16:** Use `Swatinem/rust-cache@v2` (same as existing CI) and `actions/setup-node@v4` for Node/npm. Branch filter: `main` and `develop` (matching existing CI).

### Claude's Discretion

- Exact CSS for the program listing panel (font size, step row height, highlight color) — should feel consistent with the existing dark calculator aesthetic. Monospace font for step labels.
- Whether step rows alternate background shading for readability.
- Exact wording of the panel header (`"PRGM"` vs `"PRGM MODE"` vs step count format).
- Whether `D-02` pads step numbers as `000`, `001`, etc. or omits leading zeros — follow `format_step()` behavior from `prgm_display.rs` (which uses `{:03}` zero-padding).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope
- `.planning/ROADMAP.md` §Phase 18 — Goal, 5 success criteria (SC-1 through SC-5), Requirement PROG-01
- `.planning/REQUIREMENTS.md` §PROG — PROG-01 definition

### Existing Program Display Logic (CLI)
- `hp41-cli/src/prgm_display.rs` — `format_step()`, `op_display_name()` — **copy this module** into `hp41-gui/src-tauri/src/prgm_display.rs` as-is; adapt imports if needed

### Existing GUI Backend
- `hp41-gui/src-tauri/src/types.rs` — `CalcStateView` struct and `from_state()` — Phase 18 adds `program_steps: Vec<String>` and `pc: usize` fields here
- `hp41-gui/src-tauri/src/commands.rs` — `handle_op`, `handle_get_state` pattern — Phase 18 adds `handle_sst()` and `handle_bst()` helpers with the same structure
- `hp41-gui/src-tauri/src/key_map.rs` — key ID registry; SST/BST use new Tauri commands, NOT this resolver
- `hp41-gui/src-tauri/src/lib.rs` — `invoke_handler` registration; Phase 18 adds `sst_step`, `bst_step`

### Existing Frontend
- `hp41-gui/src/App.tsx` — `calcState`, `busyRef`, `handleKey`, `handleClick`, `resolveKeyId()`; Phase 18 adds F7/F8 to `resolveKeyId()`, new program listing panel render, and `handleSst`/`handleBst` invoke calls
- `hp41-gui/src/Keyboard.tsx` — `KEY_DEFS` array; Phase 18 sets `id: 'sst'` at row 3 col 2 and `id: 'bst'` at row 4 col 8
- `hp41-gui/src/App.css` — add program listing panel styles

### Tauri Configuration
- `hp41-gui/src-tauri/tauri.conf.json` — `windows[0].height` to update from 700 → 900

### Existing CI
- `.github/workflows/ci.yml` — CLI CI; Phase 18 must NOT modify this file; new GUI CI is `.github/workflows/ci-gui.yml`

### Prior Phase Decisions
- `.planning/phases/17-persistence-and-print-output/17-CONTEXT.md` — D-05: copy-to-GUI pattern; D-07/D-08/D-09: collapsible print panel pattern (program listing panel follows similar auto-show/hide pattern)
- `.planning/phases/15-display-and-keyboard/15-CONTEXT.md` — D-10: Vanilla CSS; D-11/D-12: React state and useCallback/useEffect patterns

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `hp41-cli/src/prgm_display.rs` — `op_display_name()` covers all Op variants exhaustively; copy verbatim to GUI
- `CalcState.pc: usize` + `CalcState.program: Vec<Op>` — already in state; format all steps in `from_state()` by iterating `program.iter().map(op_display_name)`
- `busyRef: React.MutableRefObject<boolean>` — SST/BST clicks must check `if (busyRef.current) return;` before invoking
- Print panel CSS patterns (`print-panel`, `print-panel-content`, `overflow-y: auto`) in `App.css` — program listing panel reuses same structure

### Established Patterns
- **Vanilla CSS only** (Phase 15 D-10) — listing panel uses className strings + App.css rules
- **CalcStateView DTO enrichment** — add fields to `CalcStateView` rather than new IPC commands for display data
- **Pure-Rust helper + thin Tauri thunk** — `handle_sst()` / `handle_bst()` follow the same pattern as `handle_op()` / `handle_get_state()` for testability
- **`#![deny(clippy::unwrap_used)]`** — new Rust code uses `.expect("reason")` or `?`-propagation; poisoned-lock recovery via `.unwrap_or_else(|e| e.into_inner())`
- **`useCallback` + `useEffect` with cleanup** — keyboard listener pattern; SST/BST F7/F8 handled inside existing `handleKey` callback
- **`#[serde(default)]`** — not needed here (new CalcStateView fields have defaults)

### Integration Points
- `CalcStateView::from_state()` in `types.rs` — add `program_steps` population loop and `pc` field
- `lib.rs invoke_handler![]` macro — add `sst_step`, `bst_step` to the handler list
- `App.tsx resolveKeyId()` — add `'F7' → 'sst'` and `'F8' → 'bst'` branches; React handles dispatch to `sst_step`/`bst_step` invoke calls
- `App.tsx` render — add program listing panel after `<Keyboard />`, conditional on `calcState.annunciators.prgm`
- `.github/workflows/ci-gui.yml` — new file, no changes to existing `ci.yml`

</code_context>

<specifics>
## Specific Ideas

- The listing panel header `"PRGM — N steps"` should show the step count so the user can see program length at a glance.
- Auto-scroll: when `calcState.pc` changes (e.g., after SST/BST), `useEffect` scrolls the highlighted step into view via `scrollIntoView({ behavior: 'smooth', block: 'nearest' })`.
- `program_steps` index 0 is step `000 END` when the program is empty (from `format_step()` when `pc >= program.len()`). The listing should always show at least one row.
- F7/F8 keyboard bindings only make sense when `annunciators.prgm` is true; however, `resolveKeyId()` can dispatch them unconditionally — the backend `handle_sst()` / `handle_bst()` are safe to call at any time (pc stays at 0 if program is empty).
- The CI frontend check should be `npx tsc --noEmit` run from `hp41-gui/` directory — this catches TypeScript errors without a full Vite build.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 18-Program Listing & CI/CD*
*Context gathered: 2026-05-10*
