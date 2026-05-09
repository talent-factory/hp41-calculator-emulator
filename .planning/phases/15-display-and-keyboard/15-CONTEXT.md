# Phase 15: Display & Keyboard - Context

**Gathered:** 2026-05-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire the Phase 14 IPC layer into a working React UI. Deliver:
- **Display panel** — 12-char `display_str` rendered in monospace on a dark background, matching the HP-41 hardware output format
- **Annunciator row** — five indicators (USER, PRGM, ALPHA, RAD, GRAD) as text badges: bright when active, dim when inactive
- **Stack register panel** — shows X / Y / Z / T / LASTX values, each formatted via `format_hpnum()`, updating after every operation
- **Physical keyboard wiring** — atomic 1:1 key→op bindings from `hp41-cli/src/keys.rs::key_to_op()`, dispatched via `invoke("dispatch_op", ...)` on every keypress
- **EEX-CHS fix** — correctly dispatches `"eex_chs"` key ID (not `"chs"`) when user presses `'n'` while in exponent entry mode

This phase does NOT add the SVG calculator skin (Phase 16), persistence (Phase 17), program listing (Phase 18), or multi-step modal sequences (STO register entry, RCL, FIX/SCI/ENG digit count, ALPHA text, hex byte insertion, print modal). These are deferred.

</domain>

<decisions>
## Implementation Decisions

### CalcStateView Extension (Rust — types.rs)

- **D-01:** `CalcStateView` is extended to include the full stack: `y_str: String`, `z_str: String`, `t_str: String`, `lastx_str: String`. All four use `format_hpnum(&state.stack.{register}, &state.display_mode)` — same formatting as `x_str`. Phase 15 extends `CalcStateView::from_state()` to populate these fields.
- **D-02:** `CalcStateView` gains `in_eex_mode: bool`, derived from `state.entry_buf.contains('e')` in `from_state()`. React checks this flag to decide whether to send `"eex_chs"` or `"chs"` on the `'n'` keypress.
- **D-03:** No `entry_buf_raw` field. The boolean flag is sufficient and keeps the abstraction boundary clean.

### Keyboard Wiring (React)

- **D-04:** Phase 15 wires only **atomic 1:1 key→op bindings** — the set of keys that `key_to_op()` in `hp41-cli/src/keys.rs` maps to a non-None Op. This covers: Enter, Backspace/CLX, `+`, `-`, `*`, `/`, `n` (CHS), `r` (RDN), `x` (XY_SWAP), `l` (LASTX), `s` (SQRT), `p` (PRGM mode), `a` (ASIN), `c` (ACOS), `k` (ATAN), `C` (COS), `T` (TAN), `L` (LN), `G` (LOG), `E` (EXP), `H` (TENPOW), `I` (RECIP), `W` (SQ), `Y` (YPOW), `u` (USER mode), `z` (SIGMA+), `Z` (SIGMA-), `m` (MEAN), `D` (SDEV), `y` (YHAT), `b` (LR), `O` (CORR), `V` (CLSIGMASTAT), `h` (HMSTOH), `j` (HMSADD), `J` (HMSSUB), `q` (SIN — Phase 8 binding), `g` (CLREG — Phase 8 binding). Plus digit entry: `0`–`9`, `.`, `e` (EEX).
- **D-05:** Modal-triggering keys (`S` for STO, `R` for RCL, `f` for FIX mode, `F` for FmtDigits, `P` for Print, `X` for hex byte) are **silently ignored** in Phase 15. No toast, no error — they produce no IPC call and no state change. This matches `key_to_op()` returning `None` for these in the CLI.
- **D-06:** EEX-CHS fix: on `'n'` keypress, React checks `calcState.in_eex_mode`. If true, dispatches `"eex_chs"`; if false, dispatches `"chs"`. This closes the gap noted in STATE.md.
- **D-07:** Keyboard event listeners use `useCallback` + `useEffect` with cleanup return, following SC-4 of the Phase 15 success criteria. No duplicate IPC calls in React StrictMode.

### Layout & Visual Structure

- **D-08:** **Functional vertical stack layout:**
  1. Annunciator row — `USER PRGM ALPHA RAD GRAD` text badges in one horizontal row
  2. Display row — 12-char `display_str` in monospace, right-aligned, dark background
  3. Stack panel — five labeled rows: `X: ...`, `Y: ...`, `Z: ...`, `T: ...`, `L: ...`
  4. Placeholder area for Phase 16 SVG keyboard (empty `<div id="keyboard-area">`)
- **D-09:** Annunciators are **text badges**: the label is always visible, but dim (low opacity, gray) when inactive and bright (full opacity, white or yellow) when active. No color-coded pills, no icons — matches the HP-41 hardware dot-matrix annunciator aesthetic.
- **D-10:** **Vanilla CSS only** — no Tailwind, no CSS-in-JS, no new npm dependencies. Phase 15 adds styles to `src/index.css` or a new `src/App.css`. The Phase 13 scaffold already has `index.css` with a minimal reset.

### React State Architecture

- **D-11:** Component holds `CalcStateView` in `useState`. On mount, call `invoke("get_state")` to initialize. On each valid keypress, call `invoke("dispatch_op", { keyId })` and update state with the returned `CalcStateView`. No separate polling.
- **D-12:** The `useCallback` + `useEffect` pattern is mandatory for the keyboard listener per SC-4. The effect cleans up the event listener on unmount.

### Claude's Discretion

- Component breakdown (e.g., whether `Display`, `Annunciators`, `StackPanel` are separate components or one) is left to the implementer.
- Monospace font choice (system monospace or a specific font) is left to the implementer — the goal is a dark "calculator display" feel.
- Exact CSS values (font size, colors, spacing) are implementer's judgment; the dark-background calculator aesthetic is the guide.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap

- `.planning/ROADMAP.md` — Phase 15 goal, 5 success criteria (SC-1 through SC-5), dependency on Phase 14
- `.planning/REQUIREMENTS.md` — DISP-01, DISP-02, IPC-02 acceptance criteria

### Prior Phase Context

- `.planning/phases/14-ipc-layer/14-CONTEXT.md` — All Phase 14 decisions (CalcStateView shape, key ID convention, modal state ownership, error shape, capabilities)
- `.planning/phases/13-workspace-skeleton/13-CONTEXT.md` — Workspace decisions (bundle identifier, AppState type alias, just recipes, Tauri conf structure)

### Existing Source Files (must read before implementing)

- `hp41-gui/src-tauri/src/types.rs` — Current `CalcStateView`, `Annunciators`, `GuiError`; Phase 15 extends these
- `hp41-gui/src-tauri/src/commands.rs` — `dispatch_op`, `get_state` Tauri command implementations
- `hp41-gui/src-tauri/src/key_map.rs` — `resolve()` function mapping key ID strings to `Op` variants; source of truth for what key IDs are valid
- `hp41-cli/src/keys.rs` — `key_to_op()` — authoritative list of CLI key bindings Phase 15 must cover (IPC-02 / SC-5)
- `hp41-gui/src/App.tsx` — Current empty scaffold; Phase 15 populates this

### Architecture Decisions

- `hp41-gui/src-tauri/src/types.rs` D-01 through D-03 (Phase 14) — CalcStateView field decisions; Phase 15 extends, never replaces
- STATE.md decision: EEX-CHS gap — `"eex_chs"` key ID must be sent (not `"chs"`) when `in_eex_mode` is true

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `hp41-gui/src-tauri/src/types.rs::CalcStateView::from_state()` — Pure constructor; Phase 15 extends it by adding y_str, z_str, t_str, lastx_str, in_eex_mode fields
- `hp41-gui/src-tauri/src/commands.rs::dispatch_op` + `get_state` — Already registered Tauri commands; React calls these via `invoke()`
- `hp41-gui/src-tauri/src/key_map.rs::resolve()` — Already handles ~50 named ops + parameterized families; no changes needed for atomic key wiring
- `hp41-core::format_hpnum()` — Already used for x_str in CalcStateView; same function for y_str/z_str/t_str/lastx_str

### Established Patterns

- **Zero-panic policy:** `#![deny(clippy::unwrap_used)]` active in `hp41-gui/src-tauri/src/lib.rs` — all new Rust code uses `?`-propagation or `.unwrap_or_else(|e| e.into_inner())` for Mutex
- **No core duplication:** All calculator logic stays in `hp41-core`; `hp41-gui/src-tauri` is a thin adapter
- **Poisoned-lock recovery:** `.unwrap_or_else(|e| e.into_inner())` on Mutex lock calls
- **CalcStateView is the single source of truth** — React never computes or caches calculator values independently; it only renders what `CalcStateView` returns

### Integration Points

- `hp41-gui/src-tauri/src/types.rs` — Extend `CalcStateView` struct and `from_state()` (Rust side of Phase 15)
- `hp41-gui/src/App.tsx` — React entry point; add display, annunciator, stack components and keyboard listener here
- `hp41-gui/src/index.css` — Add calculator display styles (dark background, monospace font, annunciator dim/bright states)

</code_context>

<specifics>
## Specific Ideas

- Annunciator badge layout: `USER PRGM ALPHA RAD GRAD` in a single horizontal row, no separators. Active = full opacity white/yellow; inactive = ~30% opacity gray.
- Display panel shows `display_str` right-aligned in monospace on a dark (near-black) background — the "LCD screen" aesthetic.
- Stack panel rows: `X: {x_str}`, `Y: {y_str}`, `Z: {z_str}`, `T: {t_str}`, `L: {lastx_str}` — all right-aligned in the same monospace font.
- Keyboard area: empty `<div id="keyboard-area" />` placeholder so Phase 16 knows exactly where to inject the SVG skin.

</specifics>

<deferred>
## Deferred Ideas

- Multi-step modal sequences (STO register entry, RCL, FIX/SCI/ENG digit count, ALPHA text entry, hex byte insertion, print modal) — deferred to Phase 16 or quick tasks
- TypeScript type generation for `CalcStateView` (Tauri's `tauri-specta` or similar) — deferred; Phase 15 uses manual inline types or `any` with a TODO comment
- CSS styling framework (Tailwind, etc.) — deferred; Phase 15 uses vanilla CSS; revisit if Phase 16 needs it
- Keyboard shortcut overlay (port of `?` help panel from CLI) — v2.1 future requirement (SKIN-05)

</deferred>

---

*Phase: 15-Display & Keyboard*
*Context gathered: 2026-05-09*
