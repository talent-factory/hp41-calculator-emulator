# Phase 31: GUI Integration — Research

**Researched:** 2026-05-17
**Domain:** Tauri v2 + React 19 + TypeScript GUI wiring of a v3.0 Math Pac I core surface (already shipped in Phase 28/29/30) into `hp41-gui`
**Confidence:** HIGH for all integration points (every dependency was inspected in-tree)

---

## Summary

Phase 31 is the FIFTH phase of v3.0 and the only one that touches `hp41-gui`. Math Pac I math (~40 `Op` variants, `xrom_resolve`, `submit_modal`, `cancel_modal`, `submit_modal_with_label`, `requires_alpha_label`, `ModalProgram`, `modal_prompt`, `cancel_requested: Arc<AtomicBool>`, per-64-samples cancellation polling in `op_integ` / `op_solve` / `op_difeq`, `HpError::Canceled`) has ALREADY landed in Phase 28; the CLI mirror has ALREADY landed in Phase 29; the docs have ALREADY landed in Phase 30. Phase 31 is exclusively GUI-side wiring: one new Tauri command (`request_cancel`) + two more for parity (`submit_modal`, `cancel_modal`) + an LCD-alternation routing in `handle_get_state` + a TypeScript discriminated-union extension to `PendingInput` + a two-section `?`-overlay + a surgical 1-arm extension to `Op::Catalog(2)` in `hp41-core` (because Phase 28 left it as a "NOT AVAILABLE" stub — verified line 335-339 of `hp41-core/src/ops/program.rs`).

Three structural risks dominate the planning surface:

1. **Cancellation channel correctness** — `cancel_requested` is already an `Arc<AtomicBool>` with the per-64-samples polling wired in `op_integ` / `op_solve` / `op_difeq` (verified, lines 326/384/305 respectively). The remaining work is the Tauri command thunk that flips it from the GUI thread. Critical sub-question: the AppState Mutex is held for the entire duration of `dispatch(state, Op::Integ)` — so `op_integ` itself does NOT release the lock mid-call. Phase 31 must either (a) verify that the polling-while-locked design is sufficient (it is — the cancel thunk can flip the AtomicBool without acquiring the Mutex), or (b) introduce a separate `Arc<AtomicBool>` cloned into the setup hook and exposed via `tauri::State` independently of the Mutex. Recommendation: option (a) — clone the Arc out at command entry and store on a separate `tauri::State`. (See §"AppState Mutex + AtomicBool interleaving" below.)
2. **`Op::Catalog(2)` is a stub.** Verified: lines 335-339 of `hp41-core/src/ops/program.rs` emit a single "NOT AVAILABLE" line for `n == 2`. Phase 31 takes a surgical `hp41-core` exception (analogous to v2.2 Phase 25-03's `builtin_card_op` 4→12 extension) to enumerate `MATH_1.ops` into `print_buffer`. SC-4 invariant trivially preserved (no `op_*` in `hp41-gui/src-tauri/`).
3. **Vite cross-directory JSON-import already works.** Verified: `hp41-gui/vite.config.ts` line 18-19 explicitly allows `path.resolve(__dirname, '..')` as `server.fs.allow`, AND `hp41-gui/src/help_data.ts` line 13 already imports `../../docs/hp41cv-functions.json`. Adding a sibling import for `hp41-math1-functions.json` requires zero config changes — it lives in the same `docs/` directory.

**Primary recommendation:** Land Plan 31-02 (cancellation channel) FIRST as a vertical slice — it's the only plan with cross-cutting changes (hp41-core stub exception in Plan 31-04 is non-blocking) and the highest concurrency/correctness risk. Then Plan 31-03 (XEQ-by-name through `xrom_resolve` — pure CLI port + 2 Tauri thunks), Plan 31-05 (LCD-alternation routing + post-dispatch auto-open + Esc/R-S 3-way) which depends on 31-03's Tauri thunks, Plan 31-04 (`?`-overlay + `Op::Catalog(2)` surgical extension), Plan 31-01 (SC-4 grep verification of pre-shipped `prgm_display.rs` arms — should be a one-task verify-only plan per CONTEXT.md line 30).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|--------------|-----------------|-----------|
| Math Pac I math + `xrom_resolve` + modal state machine | `hp41-core` (FROZEN since Phase 28) | — | D-25.6 CLI ↔ GUI parity invariant. All math + state changes happen here. |
| Cancellation poll loop | `hp41-core` (FROZEN since Phase 28) | — | Per-64-samples checks already wired in `op_integ` / `op_solve` / `op_difeq` via `state.cancel_requested.load(Ordering::Relaxed)` |
| Cancellation trigger (flip AtomicBool) | `hp41-gui/src-tauri` (Tauri command) | — | The only writer of `cancel_requested.store(true)`. New `request_cancel` thunk. |
| R/S 3-way + Esc universal-cancel routing | `hp41-gui/src` (React `App.tsx`) | — | State machine over `CalcStateView.is_running` + `CalcStateView.modal_program` + frontend-owned `shiftActive`. Pure frontend logic per D-31.1 / D-31.2. |
| XEQ-by-name modal (Math Pac I name → `Op`) | `hp41-core` (`xrom_resolve`) called via | `hp41-gui/src-tauri/src/commands.rs::dispatch_op` (existing `xeq_<NAME>` resolver path in `key_map.rs::resolve_parameterized`) | The existing v2.1 `xeq_<NAME>` key_id flows through `key_map.rs` → `Op::Xeq("<NAME>")` → `run_program::execute_op` → `xeq_by_name_local_resolve` → `xrom_resolve`. NO new resolver path needed; Phase 29 already wired `xrom_resolve` as the fallthrough in CLI. The GUI's existing `dispatch_op("xeq_SINH")` will route correctly through hp41-core. (See §"XEQ-by-name modal".) |
| `?`-overlay rendering of Math Pac I entries | `hp41-gui/src` (Vite static JSON-import; React render) | `docs/hp41-math1-functions.json` (existing, 45 entries) | Vite static-import pattern from `help_data.ts` line 13 already works for cross-directory imports. |
| LCD modal-prompt alternation | `hp41-gui/src-tauri/src/commands.rs::handle_get_state` | `hp41-gui/src/Display14Seg.tsx` (zero source change — it renders whatever `display_str` arrives) | Phase 31 routes `modal_prompt` → `state.display` upstream per D-31.5. Display14Seg's SEGMENT_MAP needs ≡ (U+2261) added or a documented ASCII fallback (`>` recommended — see §"Display14Seg glyph"). |
| CAT 2 PSE scroll | `hp41-core` (surgical 1-arm exception to `op_catalog`) | `hp41-gui/src` (existing print panel; no source change) | v2.2 Phase 22 CAT 1 pattern; pushes lines into `print_buffer`. Currently a stub. |
| Permission file lifecycle | `hp41-gui/src-tauri/permissions/*.toml` + `lib.rs::generate_handler!` | `scripts/check-tauri-permissions.sh` (DOES NOT EXIST — CONTEXT.md line 32 incorrectly references it; see Open Question Q1) | Tauri v2.11 requires per-command TOML. 3 new commands → 3 new TOMLs. |

---

## User Constraints (from CONTEXT.md)

### Locked Decisions (D-31.1 — D-31.14, carried-forward decisions C-28.1..C-28.4, D-25.6, D-28.4, D-28.6, D-28.7, D-29.5..D-29.9)

**Cancellation UX**
- **D-31.1:** R/S 3-way state-routed in `App.tsx`: priority `modal_program.is_some()` → `submit_modal`; `is_running` → `request_cancel`; else → existing `run_stop`. `pending_input` routing block stays ABOVE this (D-07 never-discard invariant).
- **D-31.2:** Esc universal cancel — parallel to R/S: priority `modal_program.is_some()` → `cancel_modal`; `is_running` → `request_cancel`; `shiftActive` → clear (existing v2.1 behavior); else no-op.
- **D-31.3:** Visual feedback = existing toast only. `HpError::Canceled` (already exists in `hp41-core/src/error.rs` line 37; `Display` impl returns `"canceled"`) flows through `From<HpError> for GuiError` → 2s toast. UI-SPEC overrides the literal toast string to `"CANCELED"` (uppercase); planner verifies that the `Display` impl returns "canceled" (lowercase) and either (a) UI-SPEC accepts "canceled" with a doc note, or (b) types.rs maps `HpError::Canceled` to "CANCELED" verbatim. **Planner action: pick option (b)** — preserves the v2.1 stub-error UPPER convention.
- **D-31.4:** Discovery via `is_running` annunciator only. No tooltip, no inline hint.

**Modal-prompt placement**
- **D-31.5:** LCD alternation — `state.display` priority computed in `commands.rs::handle_get_state`: `modal_program.is_some() && entry_buf.is_empty()` → `truncate_with_continuation(modal_prompt)`; else existing v2.2 logic.
- **D-31.6:** Truncate >12 chars with HP-41 `≡` (U+2261) continuation marker. Long prompts: `FUNCTION NAM≡` (14→12), `NO. SAMPLES≡` (13→12). Others fit verbatim.
- **D-31.7:** No accessibility fallback for truncated prompts. Pure hardware fidelity.

**`?`-overlay sectioning**
- **D-31.8:** Two top-level collapsible sections — "HP-41CV (built-in)" + "Math 1 Pac (XROM 7)". Both expanded by default.
- **D-31.9:** JSON's per-program categories preserved as 2nd-level headers ("Math1 Hyperbolics", "Math1 Complex Arithmetic", etc. — 11 distinct categories verified by inspecting `docs/hp41-math1-functions.json` — see §"JSON entries").
- **D-31.10:** Vite JSON-import at build time. NOT a Tauri command, NOT async fetch.
- **D-31.11:** No filter input (clean overview preferred).

**CATALOG 2 rendering**
- **D-31.12:** Hardware-faithful PSE scroll via `state.print_buffer` (mirror v2.2 CAT 1).
- **D-31.13:** First line `XROM Math 1`; function names follow.
- **D-31.14:** R/S pauses/resumes; other keys cancel.

**Inherited from CONTEXT carried-forward decisions:**
- **C-28.1 / ADR-001:** Op-strategy A — zero new `Op` variants in Phase 31 (`Op::Catalog` already exists).
- **C-28.3 / ADR-005:** Separate `docs/hp41-math1-functions.json` (already authored in Phase 29 D-29.1).
- **C-28.4:** `xrom_resolve` fires LAST in resolver chain.
- **D-25.6 CLI ↔ GUI parity:** GUI calls shared `hp41-core` functions — no parallel impls.
- **D-28.4:** `modal_prompt: Option<String>` is the prompt channel; `print_buffer` is for PRX/PRA/PRSTK only.
- **D-28.6:** Math Pac I = XEQ-by-name only. `KEY_DEFS` in `Keyboard.tsx` unchanged.
- **D-28.7:** `cancel_requested: Arc<AtomicBool>` + per-64-samples checks already wired in Phase 28.

### Claude's Discretion

- **LCD-alternation routing implementation site** (CONTEXT line 147-151): Option 1 (`commands.rs::handle_get_state`) recommended — GUI-only, CLI unaffected.
- **`Op::Catalog(2)` body verification** (CONTEXT line 152): VERIFIED — Phase 28 shipped a stub ("NOT AVAILABLE" line 338 of `hp41-core/src/ops/program.rs`). Phase 31 takes the surgical exception.
- **Vite path-alias vs symlink** (CONTEXT line 153): NEITHER needed — `vite.config.ts` line 18-19 already allows `..` (set up for Phase 26 D-25.16); the relative-path import `../../docs/hp41-math1-functions.json` works as-is.
- **`request_cancel` signature** (CONTEXT line 154-155): `fn request_cancel(state: State<'_, AppState>) -> Result<(), GuiError>`; idempotent.
- **`PendingInput::XeqByName { mode }` TypeScript port** (CONTEXT line 155): TS literal-union `'normal' | 'collect-for-modal'`; planner picks JSON-shape.
- **`≡` continuation marker rendering** (CONTEXT line 156): VERIFIED MISSING in Display14Seg's SEGMENT_MAP (lines 131-184). Planner picks: (a) add `\u{2261}` to SEGMENT_MAP as `[6, 7, 3]` (same as `=`), or (b) ASCII fallback `>` (also missing — but trivially mapped to `[0, 11]` upper-bar + SE-diagonal), or (c) `...` (three periods — periods fold into preceding cells so this doesn't work). **Recommendation: option (a) — `≡` shares the segment pattern with `=` plus an additional middle stripe; map to `[6, 7, 3, 0]` (top + middle + bottom) for a tri-bar look.**
- **PSE delay for CAT 2** (CONTEXT line 157): match v2.2 CAT 1 — but v2.2 CAT 1 emits ALL lines synchronously into `print_buffer` (verified — `op_catalog` is a single-pass loop). **There is NO PSE-step in v2.2 CAT 1.** This is a research finding that contradicts the CONTEXT line 157 assumption. See Open Question Q2.
- **Sort order within `?`-overlay categories** (CONTEXT line 159): alphabetical recommended.
- **ALPHA annunciator during modal alpha-label collection** (CONTEXT line 160): mirror v2.2.
- **Precedence when both `pending_input` and `modal_program` are active** (CONTEXT line 161): modal_program wins (per D-07 — never discard active modal).

### Deferred Ideas (OUT OF SCOPE)

- CLI Esc/Ctrl-C → `request_cancel` backport (D-25.6 parity quick-task after Phase 31 ships).
- Backport LCD-alternation routing into `hp41-core::state::display()`.
- AVIEW-style scrolling for prompts > 12 chars.
- `?`-overlay filter input.
- Tab UI for `?`-overlay (v3.1+).
- CAT 2 module-header verbosity ("XROM 7 MATH 1 (55 fns)").
- ALPHA annunciator new logic.
- Tauri `submit_modal_with_label` command (Phase 31's planner picks shape — see Plan 31-05 design).

---

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-------------------|
| GUI-01 | `hp41-gui/src-tauri/src/prgm_display.rs` ~40 new `op_display_name` arms | **ALREADY SHIPPED in Phase 28 plans 28-02..28-10** per CONTEXT line 31 + line 73; Plan 31-01 reduces to a one-task SC-4 grep verification. See §"Plan 31-01 scope reduction". |
| GUI-02 | XEQ modal resolves Math Pac I via shared `xrom_resolve`; D-25.6 parity | Phase 29 already wired `xeq_by_name_local_resolve` to call `xrom_resolve` (verified `hp41-cli/src/keys.rs` line 1499-1505). The GUI's existing `dispatch_op("xeq_<NAME>")` path flows through `key_map.rs::resolve` → `Op::Xeq("<NAME>")` → `run_program` → `xrom_resolve` automatically. **The XEQ-by-name modal in `App.tsx` already exists** (line 152 — `xeq_prompt: () => ({ kind: 'xeq_name', acc: '', dispatchPrefix: 'xeq' })`). Phase 31 task: verify that `xeq_SINH` resolves correctly end-to-end (this should already work — write a regression test). |
| GUI-03 | `?`-overlay loads Math Pac I JSON in parallel; categorized "Math 1 Pac" section | Plan 31-04. Vite import is trivial (cross-directory works since Phase 26). HelpOverlay needs two-section wrapper (D-31.8). |
| GUI-04 | `CATALOG 2` lists loaded XROM modules with function counts | Plan 31-04. Surgical `op_catalog` exception (Phase 31's only `hp41-core` change). |
| GUI-05 | Cancellation channel: `request_cancel` Tauri command + permission TOML | Plan 31-02. `cancel_requested: Arc<AtomicBool>` field + per-64-samples polling already wired in Phase 28. Phase 31 wires the frontend trigger only. |
| GUI-06 | Modal-prompt rendering — `ORDER=?`, `A1,1=?`, etc. on LCD; user input via Number-Entry pipeline; ENTER confirms; ESC cancels | Plan 31-05. **CONTEXT.md D-31.5 OVERRIDES REQUIREMENTS.md GUI-06** — the prompt renders on the LCD (alternation), NOT in the print panel. Number-Entry pipeline (existing `entry_buf` digit path) is unchanged. R/S submits (D-29.5), Esc cancels (D-29.6). |
| GUI-07 | Stub-error arm in `key_map::resolve` does NOT shrink in v3.0 | Plan 31-05 (verification only). Math Pac I has no dedicated keys — `key_map.rs` arms are preserved. |

---

## Standard Stack

### Already-installed dependencies (zero new deps in Phase 31)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tauri | 2.11 | Rust desktop runtime | Locked since Phase 13 (D-02) |
| React | 19.x | Frontend rendering | Verified via `hp41-gui/package.json` |
| Vite | 5.x | Bundler with static JSON-import support | `vite.config.ts` already configured for cross-directory JSON via `fs.allow` (Phase 26 W8) |
| `tauri::State<'_, AppState>` | (Tauri-managed) | Shared state extractor | Existing pattern in `commands.rs` (5 thunks) |
| `std::sync::Arc<AtomicBool>` | std-lib | Cross-thread cancel flag | Already field on `CalcState` (verified `state.rs` line 222) |
| TypeScript discriminated unions | TS 5.x | Modal state union (`PendingInput`) | Existing 14-variant union in `pending_input.ts` |

### NO new runtime dependencies in `hp41-gui/src-tauri/Cargo.toml`

Verified via CONTEXT.md cross-cutting line 51: "No new npm dependencies in `hp41-gui/` (Vite is already wired; JSON-import is a built-in feature)."

### Alternatives Considered

| Instead of | Could Use | Why Rejected |
|-----------|-----------|---------------|
| Vite static JSON-import | Tauri `get_help_data` command at runtime | ROADMAP SC-3 line 167 locked Vite import; matches CLI's `include_str!` philosophy; gives compile-time JSON-validation (D-25.16 hard-build-blocker pattern) |
| Vite path alias `@docs` | Relative `../../docs/...` | Relative import already works (verified `help_data.ts` line 13); alias would be cleaner but is a non-functional cosmetic |
| Symlink `docs/` into `hp41-gui/src/` | Vite relative import | Symlinks break on Windows checkouts; relative import is cross-platform |
| Separate `cancel_flag: Arc<AtomicBool>` in `tauri::State` (independent of AppState Mutex) | Clone the Arc out of AppState in the cancel thunk | Splitting the canonical owner introduces a second source of truth — `cancel_requested` field already lives on CalcState (verified). Cloning the Arc at thunk entry is one extra line and preserves single-source-of-truth. |

**Installation:**

```bash
# No new packages — Phase 31 is purely wiring.
```

---

## Package Legitimacy Audit

> **Not applicable.** Phase 31 installs ZERO external packages. CONTEXT.md cross-cutting line 51 forbids new runtime deps in `hp41-core` and `hp41-gui/src-tauri/`, AND forbids new npm deps in `hp41-gui/`. All work is wiring of already-installed packages (Tauri, React, Vite) and already-shipped Rust functions (`xrom_resolve`, `submit_modal`, `cancel_modal`, `submit_modal_with_label`).

| Package | Registry | Disposition |
|---------|----------|-------------|
| (none) | — | — |

slopcheck audit: trivially passes (empty package set).

---

## Architecture Patterns

### System Architecture Diagram

```
                              ┌──────────────────────────────────────┐
                              │  hp41-gui/src/ (React 19 + TS)        │
                              │                                       │
                              │  ┌────────────────────────────────┐   │
   user types XEQ "SINH" ───▶│  │ App.tsx                         │   │
   1.5 ENTER on keyboard     │  │  - handleClick (3-way R/S)      │   │
   or clicks on-screen R/S   │  │  - handleKeyDown (Esc cascade)  │   │
                              │  │  - MODAL_OPENERS table          │   │
                              │  │  - applyModalResult             │   │
                              │  │  - useEffect: post-dispatch     │   │
                              │  │    auto-open CollectForModal    │   │
                              │  └─────┬──────────────────────────┘   │
                              │        │                                │
                              │        ▼ invokeForKey(effectiveId)     │
                              │  ┌────────────────────────────────┐   │
                              │  │ pending_input.ts                │   │
                              │  │  - handleModalKey               │   │
                              │  │  - renderModalLcd               │   │
                              │  │  - PendingInput discriminated  │   │
                              │  │    union (extended +1 variant)  │   │
                              │  │    XeqByName.mode: 'normal' |   │   │
                              │  │      'collect-for-modal'        │   │
                              │  └─────┬──────────────────────────┘   │
                              │        │                                │
                              │        ▼ @tauri-apps/api/core::invoke  │
                              └────────┼────────────────────────────────┘
                                       │ IPC (Tauri command boundary)
                                       │
                              ┌────────▼────────────────────────────────┐
                              │  hp41-gui/src-tauri/src/ (Rust)         │
                              │                                          │
                              │  ┌────────────────────────────────┐    │
                              │  │ lib.rs::generate_handler!      │    │
                              │  │  - dispatch_op (existing)      │    │
                              │  │  - get_state (existing)        │    │
                              │  │  - sst_step / bst_step         │    │
                              │  │  - run_stop                    │    │
                              │  │  - submit_modal      [NEW]     │    │
                              │  │  - cancel_modal      [NEW]     │    │
                              │  │  - request_cancel    [NEW]     │    │
                              │  └─────┬──────────────────────────┘    │
                              │        │                                  │
                              │        ▼ AppState = Mutex<CalcState>     │
                              │  ┌────────────────────────────────┐    │
                              │  │ commands.rs                     │    │
                              │  │  - handle_op_prepare/finalize  │    │
                              │  │  - handle_get_state             │    │
                              │  │    + LCD-alternation routing   │    │
                              │  │      (D-31.5) [NEW]            │    │
                              │  │  - handle_request_cancel [NEW] │    │
                              │  │    state.cancel_requested      │    │
                              │  │      .store(true, Relaxed)     │    │
                              │  └─────┬──────────────────────────┘    │
                              │        │                                  │
                              │  ┌─────▼──────────────────────────┐    │
                              │  │ key_map.rs::resolve            │    │
                              │  │  (UNCHANGED — D-28.6 / GUI-07) │    │
                              │  │  Math Pac I via xeq_<NAME>     │    │
                              │  │  prefix routes to Op::Xeq(name)│    │
                              │  └─────┬──────────────────────────┘    │
                              └────────┼────────────────────────────────┘
                                       │
                              ┌────────▼────────────────────────────────┐
                              │  hp41-core (FROZEN since Phase 28)       │
                              │                                          │
                              │  dispatch(state, Op::Xeq("SINH"))        │
                              │   → run_program                          │
                              │   → execute_op                           │
                              │   → xeq_by_name_local_resolve  (NEW v2.2)│
                              │   → builtin_card_op                      │
                              │   → xrom_resolve (LAST per C-28.4)       │
                              │   → math1::math1_resolve                 │
                              │   → Op::Sinh                             │
                              │   → op_sinh(state)  → state.stack.x =     │
                              │     sinh(1.5) = 2.1293                    │
                              │                                          │
                              │  long-running ops:                       │
                              │   op_integ / op_solve / op_difeq         │
                              │    every 64 samples:                     │
                              │      if state.cancel_requested            │
                              │           .load(Relaxed)                 │
                              │         → clear *_state                  │
                              │         → Err(HpError::Canceled)         │
                              └──────────────────────────────────────────┘
```

### Recommended Project Structure

```
hp41-gui/
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs              # +3 entries in generate_handler!
│   │   ├── commands.rs         # +3 thunks (submit_modal, cancel_modal, request_cancel) + handle_get_state LCD-alternation
│   │   ├── key_map.rs          # UNCHANGED (D-28.6 / GUI-07)
│   │   ├── prgm_display.rs     # UNCHANGED — Plan 31-01 = grep verification only
│   │   ├── types.rs            # +1 field on CalcStateView: modal_program_active: bool (NEW — required for frontend dispatch decisions per D-31.1/D-31.2)
│   │   │                       # +1 field: modal_requires_alpha_label: bool (NEW — required for post-dispatch auto-open per D-29.9 mirror)
│   │   └── persistence.rs      # UNCHANGED
│   └── permissions/
│       ├── request-cancel.toml # NEW
│       ├── submit-modal.toml   # NEW
│       └── cancel-modal.toml   # NEW
├── src/
│   ├── App.tsx                 # extend handleClick + handleKeyDown + invokeForKey
│   ├── Display14Seg.tsx        # +1 SEGMENT_MAP entry: '\u{2261}' → [0, 6, 7, 3]
│   ├── HelpOverlay.tsx         # two-section wrapper around existing rendering
│   ├── help_data.ts            # +1 Vite JSON-import + merged accessor
│   ├── pending_input.ts        # XeqByName variant extended with mode field
│   └── App.css                 # +1 .help-overlay-section-heading selector
└── vite.config.ts              # UNCHANGED (../docs already allowed per Phase 26 W8)
```

### Pattern 1: Tauri v2.11 inline-command thunk + permission TOML

**What:** Every new Tauri command in `generate_handler!` requires a matching `permissions/<kebab>.toml` file with `identifier = "allow-<kebab>"` and `commands.allow = ["<snake_case_fn_name>"]`. Without the TOML, the frontend gets a runtime permission-denied error.

**When to use:** All 3 new commands in Phase 31 (`request_cancel`, `submit_modal`, `cancel_modal`).

**Example:**

```rust
// Source: hp41-gui/src-tauri/src/commands.rs lines 256-260 (existing run_stop pattern)
#[tauri::command]
pub fn request_cancel(state: State<'_, AppState>) -> Result<(), GuiError> {
    let calc = state.lock().unwrap_or_else(|e| e.into_inner());
    // Brief lock — just flip the AtomicBool. The long-running op (op_integ /
    // op_solve / op_difeq) holds the Mutex for its entire duration; this
    // command only acquires the Mutex to read the Arc and immediately drop
    // the guard. The Arc::clone gives us a writable handle that does NOT
    // require the Mutex.
    let cancel_flag = std::sync::Arc::clone(&calc.cancel_requested);
    drop(calc); // release Mutex before .store() — defense-in-depth
    cancel_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}
```

**CRITICAL CORRECTION:** The above signature is the IDEAL implementation but has a structural problem — `state.lock()` will BLOCK if `op_integ` holds the AppState Mutex for the duration of an active integration. See §"AppState Mutex + AtomicBool interleaving" for the correct pattern (clone the Arc out at app setup time and store on a separate `tauri::State`).

```toml
# Source: hp41-gui/src-tauri/permissions/run-stop.toml (verbatim template)
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-request-cancel"
description = "Allows the request_cancel command."
commands.allow = ["request_cancel"]
```

### Pattern 2: AppState Mutex Lock-Release During I/O (commit ff39017)

**What:** `lib.rs::run()` line 51 shows the auto-save thread releases the Mutex BEFORE disk I/O: clone state under lock, drop guard, then write. The same pattern applies to the cancellation channel — `request_cancel` flips the AtomicBool WITHOUT holding the Mutex for long.

**When to use:** Any Tauri thunk that must interleave with a long-running `dispatch()` call.

**Example:**

```rust
// Source: hp41-gui/src-tauri/src/lib.rs lines 49-55 (auto-save pattern)
let state = handle.state::<AppState>();
let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();  // guard dropped here
if let Err(e) = persistence::save_state(&thread_save_path, &snapshot) {  // no lock held
    eprintln!("auto-save failed: {e}");
}
```

### Pattern 3: Vite static JSON-import (Phase 26 W8 / D-25.16)

**What:** `import functions from '../../docs/hp41cv-functions.json'` resolves at BUILD TIME, validates schema, bakes content into bundle. Malformed JSON fails the build — hard-build-blocker, mirror of CLI's `include_str!` + `serde_json::from_str` panic.

**When to use:** Plan 31-04 for `hp41-math1-functions.json` parallel-load.

**Example:**

```typescript
// Source: hp41-gui/src/help_data.ts line 13 (existing pattern)
import hp41cvFunctions from '../../docs/hp41cv-functions.json';
import math1Functions from '../../docs/hp41-math1-functions.json'; // NEW
```

### Pattern 4: Discriminated-union extension with exhaustive switch

**What:** TypeScript discriminated union with `kind` field. Switch statements over all variants — no `default:` catch-all (FN-GUI-04 invariant). Extending the union requires updating every consumer's switch.

**When to use:** Plan 31-05 for `XeqByName { mode }` extension.

**Example:**

```typescript
// Source: hp41-gui/src/pending_input.ts line 58 (existing variant)
| { kind: 'xeq_name'; acc: string; dispatchPrefix: 'xeq' | 'gto' | 'lbl' }
// EXTENDED in Phase 31:
| { kind: 'xeq_name'; acc: string; dispatchPrefix: 'xeq' | 'gto' | 'lbl'; mode?: 'normal' | 'collect-for-modal' }
// or rename to xeq_by_name + add the mode discriminator as required field.
// Planner picks: extend existing variant (less churn) vs new variant
// 'xeq_by_name' (cleaner separation). Recommendation: extend with optional
// `mode` (default 'normal') so existing call sites continue working.
```

### Pattern 5: Post-dispatch auto-open hook (D-29.9 mirror)

**What:** After every `get_state` IPC response, check if `modal_program.is_some() && modal_program.requires_alpha_label() && pending_input === null`. If true, open `XeqByName { mode: 'collect-for-modal' }`.

**When to use:** Plan 31-05 — implements GUI side of D-29.9.

**Example:**

```typescript
// Mirror of hp41-cli/src/app.rs::maybe_auto_open_collect_for_modal (line 1782)
useEffect(() => {
  if (!calcState) return;
  if (pendingInput !== null) return;
  if (!calcState.modal_program_active) return;     // NEW field on CalcStateView
  if (!calcState.modal_requires_alpha_label) return; // NEW field on CalcStateView
  setPendingInput({
    kind: 'xeq_name',
    dispatchPrefix: 'xeq',
    acc: '',
    mode: 'collect-for-modal',
  });
}, [calcState, pendingInput]);
```

### Anti-Patterns to Avoid

- **Parallel resolver in `key_map.rs`** (forbidden by D-25.6 + GUI-02 + C-28.4): NEVER add `match name { "SINH" => Op::Sinh, ... }` to `key_map.rs`. The path is `dispatch_op("xeq_SINH")` → `key_map::resolve("xeq_SINH")` → existing `resolve_parameterized` xeq-prefix path → `Op::Xeq("SINH")` → core's `xrom_resolve`. ANY shortcut in `key_map.rs` breaks D-25.6 parity AND duplicates the resolver chain.
- **Holding the AppState Mutex while waiting for `request_cancel`** (Pitfall 11 amplifier): if `op_integ` re-acquired the Mutex on every sample, the cancel thunk would deadlock waiting for the dispatch lock. The actual design holds the Mutex for the entire `dispatch()` call but the polling reads the AtomicBool which is `Arc`-shared and lock-free. `request_cancel` reaches the AtomicBool via the Arc — see §"AppState Mutex + AtomicBool interleaving".
- **Routing `r_s` through `dispatch_op` when modal is active** (D-31.1 R/S branch 1): `submit_modal` is the correct Tauri command — calling `dispatch_op("r_s")` would invoke `run_stop` semantics which is wrong for modal-submit.
- **Catching `Op::Canceled` in `From<HpError> for GuiError`** with literal "canceled" string — UI-SPEC mandates uppercase "CANCELED". Either (a) override the literal at the From impl, or (b) modify `HpError::Canceled::Display` to return "CANCELED". Planner picks. **Recommendation: (a) — surgical change in `types.rs` only, no core touch.**
- **Pushing CAT 2 lines via PSE-step** (CONTEXT D-31.12, D-31.14 mistakenly references "PSE-step" infrastructure that does NOT exist in v2.2 CAT 1 — see Open Q2): v2.2 CAT 1 emits all lines into `print_buffer` in a single synchronous `op_catalog` call. There is no per-line PSE delay in core; the GUI's print panel renders all lines on the next `get_state` response. Planner picks: (a) accept the "instant scroll" (matches v2.2 CAT 1 behavior), (b) introduce PSE-step infrastructure (new abstraction — out of Phase 31 scope), or (c) frontend setInterval ticker to reveal lines progressively (frontend-only — no core touch). **Recommendation: (a) — match v2.2 CAT 1; the D-31.12/D-31.14 PSE references in CONTEXT.md are documentation drift that should be deferred to a v3.1 polish task.**

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cancellation signaling between threads | Custom condvar / channel | `Arc<AtomicBool>` with `Ordering::Relaxed` | Already shipped in Phase 28; lock-free; signaled via Tauri thunk |
| HP-41 prompt strings | Author them in GUI | `ModalProgram::current_prompt()` from `hp41-core/src/ops/math1/modal.rs` line 52 | Already returns OM-cited Option<String>; single source of truth |
| Math Pac I name → Op mapping | Author the map in `key_map.rs` | `xrom_resolve` already in resolver chain | D-25.6 parity invariant |
| Modal alpha-label collection | New text-input modal type | Existing `PendingInput::XeqByName` extended with `mode` discriminator | TS port of Phase 29 D-29.8 |
| Continuation marker truncation | Substring loop in TypeScript | `truncate_with_continuation()` in `commands.rs` (Rust) | Centralized — frontend just renders 12 chars; Unicode-correct via `s.chars().take(11)` |
| Permission-coverage CI check | Custom grep script | **No existing script — Phase 31 must AUTHOR it** per CONTEXT line 32 (Pitfall 21). See Plan 31-02 design. | One-time investment; protects every future Tauri command |
| Vite cross-directory JSON-import config | Manual `resolve.alias` + `fs.allow` | Already configured in `vite.config.ts` (Phase 26 W8 — line 18-20) | Zero new config |

**Key insight:** Phase 31 is wiring, not authoring. Every behavior already has a `hp41-core` or `hp41-cli` reference implementation; the GUI's job is to expose the IPC and route the keystrokes.

---

## Runtime State Inventory

> N/A — Phase 31 is a feature-addition phase, not a rename/refactor/migration phase. No string renames, no stored-data migration. Save-file backward compat is trivially preserved (zero new `CalcState` fields).

---

## Common Pitfalls

### Pitfall 1: Holding AppState Mutex during long-compute deadlocks cancellation

**What goes wrong:** A naive `request_cancel` thunk that does `state.lock()` to access `state.cancel_requested` will BLOCK until `op_integ`/`op_solve`/`op_difeq` releases the Mutex — which is exactly when the long compute finishes (i.e., never during the compute).

**Why it happens:** `dispatch(state, Op::Integ)` holds the AppState MutexGuard for the full duration of `op_integ`. The cancellation poll inside `op_integ` reads `state.cancel_requested.load(Relaxed)` — this is a field on the locked `CalcState`. But `Arc<AtomicBool>` is the WHOLE POINT: the AtomicBool is reachable WITHOUT going through the Mutex.

**How to avoid:** At app-setup time (`lib.rs::run()`), `Arc::clone` the `cancel_requested` field off the initial `CalcState` and store it as a SEPARATE `tauri::State` (a second managed-state slot). The `request_cancel` thunk takes `State<'_, CancelFlag>` (not `State<'_, AppState>`) and does NOT lock the AppState Mutex.

**Implementation sketch:**

```rust
// hp41-gui/src-tauri/src/lib.rs
use std::sync::{Arc, atomic::AtomicBool};
pub type CancelFlag = Arc<AtomicBool>;

// In setup():
let app_state = Mutex::new(initial_state);
let cancel_flag = Arc::clone(&app_state.lock().unwrap().cancel_requested);
app.manage(app_state);
app.manage(cancel_flag);  // second managed state
```

```rust
// hp41-gui/src-tauri/src/commands.rs
#[tauri::command]
pub fn request_cancel(cancel: State<'_, CancelFlag>) -> Result<(), GuiError> {
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}
```

**ALTERNATIVE that ALSO works:** if Tauri allows multiple `State` extractors in one signature (it does), the cancel thunk can take BOTH `AppState` and `CancelFlag` with the AppState lock acquired LAST and only briefly. But the separate-state pattern above is cleaner — no Mutex involvement at all.

**Warning signs:** the cancel thunk hangs in CI under WebdriverIO. The Pitfall is invisible in unit tests because they never hold the AppState Mutex for the duration of a real INTG run.

### Pitfall 2: `Op::Catalog(2)` is a stub and emits "NOT AVAILABLE"

**What goes wrong:** Phase 31 plan that assumes `Op::Catalog(2)` already enumerates XROM modules will produce a one-liner "NOT AVAILABLE" output in CAT 2, failing GUI-04 acceptance.

**Why it happens:** Verified `hp41-core/src/ops/program.rs` line 335-339: the `2..=4` arm pushes a single payload line. Phase 28 left this as a stub because XROM enumeration logic depends on Phase 31's `xrom_modules` field semantics.

**How to avoid:** Phase 31 Plan 31-04 includes a surgical `op_catalog` extension. Visibility stays `pub fn` (already public — line 300 of `program.rs`). The extension:

```rust
// hp41-core/src/ops/program.rs op_catalog n == 2 arm (surgical replacement)
2 => {
    // Enumerate loaded XROM modules. For v3.0, only Math 1 is loaded (bit 0).
    if state.xrom_modules & 0b0000_0001 != 0 {
        use crate::ops::math1::xrom::MATH_1;
        state.print_buffer.push(format!("{:<24}", format!("XROM {} {}", MATH_1.id, MATH_1.name)));
        for (name, _op) in MATH_1.ops {
            state.print_buffer.push(format!("{:<24}", name));
        }
    } else {
        state.print_buffer.push(format!("{:<24}", "NO XROM"));
    }
}
3..=4 => {
    // CAT 3 (HP-IL) + CAT 4 (peripherals) — still NOT AVAILABLE.
    state.print_buffer.push(format!("{:<24}", "NOT AVAILABLE"));
}
```

**Warning signs:** GUI test `XEQ "CATALOG" 2 Enter` reads "NOT AVAILABLE" instead of the function list.

### Pitfall 3: SC-4 grep regression after `op_catalog` surgical extension

**What goes wrong:** Phase 31's `op_catalog` extension lives in `hp41-core`, NOT in `hp41-gui/src-tauri`, so SC-4 grep is trivially preserved. But CI gates also run `tests/xrom_shadowing.rs` which iterates `MATH_1.ops` — adding a new XROM module name (which Phase 31 does NOT) would trip that gate. Phase 31's extension only READS `MATH_1.ops`, so it's fine.

**Why it happens:** The CI grep `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` is run on `hp41-gui/src-tauri/` only. `op_catalog` already lives in `hp41-core/src/ops/program.rs` (line 300) — not affected by SC-4.

**How to avoid:** Plan 31-04 documents the surgical exception in a SUMMARY.md trail (same as v2.2 Phase 25-03's `builtin_card_op` 4→12 extension). No new SC-4 invariants change.

**Warning signs:** none — this is a non-pitfall flag included for planner due-diligence.

### Pitfall 4: HpError::Canceled `Display` returns "canceled" lowercase but UI-SPEC mandates "CANCELED"

**What goes wrong:** Toast renders "canceled" — visual inconsistency with v2.1 stub-error UPPER convention.

**Why it happens:** `hp41-core/src/error.rs` line 48 asserts `HpError::Canceled.to_string() == "canceled"`. The `Display` impl returns lowercase.

**How to avoid:** In `hp41-gui/src-tauri/src/types.rs::From<HpError> for GuiError`, special-case `HpError::Canceled`:

```rust
impl From<HpError> for GuiError {
    fn from(e: HpError) -> Self {
        let message = match e {
            HpError::Canceled => "CANCELED".to_string(), // uppercase per UI-SPEC
            other => other.to_string(),
        };
        GuiError { message }
    }
}
```

**Warning signs:** WebdriverIO smoke `toHaveText("CANCELED")` fails after Plan 31-02.

### Pitfall 5: Frontend `pending_input` block routes BEFORE the modal-program submit check

**What goes wrong:** When `modal_program.is_some()` AND `pending_input !== null` simultaneously (e.g., CollectForModal is open), an R/S keystroke needs to route through the pending-input modal-key path FIRST (because that's what's "in focus"). Skipping the pending-input branch would silently discard the modal — D-07 invariant violation.

**Why it happens:** D-31.1 R/S routing has 3 branches priority `modal_program > is_running > else`. But the pending_input routing block in `handleClick` (currently lines 311-371 of App.tsx) runs ABOVE the modal-program check. This is correct — pending_input is the "innermost" modal layer.

**How to avoid:** The R/S 3-way state-routed branch in Plan 31-05 must be inserted BELOW the pending_input block (existing line 372) and ABOVE the MODAL_OPENERS block (line 374) in handleClick. **Critical: do not add a new branch that intercepts r_s BEFORE pending_input.** The exact insertion site is after line 372 ("// Rule 5: modal-opener intercept (D-26.5)") — extend the rule chain with a new "Rule 4.5: r_s 3-way modal/cancel/run-stop routing".

**Warning signs:** CollectForModal-active + R/S press dispatches `submit_modal` directly without giving the user a chance to type the label name first.

### Pitfall 6: `≡` (U+2261) not in Display14Seg SEGMENT_MAP

**What goes wrong:** Long modal prompts render as `FUNCTION NAM ` (the 12th char being a SPACE because SEGMENT_MAP lookup misses) instead of `FUNCTION NAM≡`.

**Why it happens:** Verified — `hp41-gui/src/Display14Seg.tsx` SEGMENT_MAP lines 131-184 has no entry for `\u{2261}`. The lookup `SEGMENT_MAP[cell.char.toUpperCase()] ?? []` returns empty array → renders as blank segments.

**How to avoid:** Plan 31-05 adds `'\u{2261}': [0, 6, 7, 3]` (top + middle + bottom — three-bar shape) to SEGMENT_MAP. Vitest snapshot of Display14Seg with text `"FUNCTION NAM\u{2261}"` confirms the glyph renders.

**Warning signs:** Vitest render snapshot shows trailing blank cell instead of segment pattern.

### Pitfall 7: Vite path-allow does NOT include the new JSON file

**What goes wrong:** Build fails with `EACCES` or `restricted to the project root` because `fs.allow` is set per-Vite-version and may not handle the new file path.

**Why it happens:** Vite 5's `server.fs.allow` is a serve-only setting; production builds use `assetsInclude` or implicit JSON-import support. The existing `hp41cv-functions.json` works because it's IMPORTED via `import functions from '../../docs/hp41cv-functions.json'` at module top-level — Vite's static analyzer follows the import and bakes the JSON in.

**How to avoid:** Verified the existing pattern works for ANY JSON file under `../../docs/` because `vite.config.ts` line 18-19 sets `fs.allow: [path.resolve(__dirname, '..')]` which is the entire `hp41-calculator-emulator/` repo root. Adding `import math1 from '../../docs/hp41-math1-functions.json'` is a one-line change.

**Warning signs:** `npm test` build error referencing the new JSON path.

### Pitfall 8: CAT 2 in the GUI fires AFTER `request_cancel` was sent

**What goes wrong:** The `cancel_requested` flag persists across operations because it's reset only at the START of `op_integ`/`op_solve`/`op_difeq` (verified: `integ.rs` line 223 comment — "cancel_requested is NOT reset here — the workflow opener resets it"). If the user (a) starts INTG, (b) presses R/S to cancel, (c) then presses CATALOG 2 — the CAT 2 op should NOT see a sticky cancel.

**Why it happens:** `op_catalog` does NOT poll `cancel_requested`. It's a synchronous push-to-buffer operation. The sticky flag doesn't affect CAT 2 directly. But the NEXT INTG might fire and immediately return Canceled because the flag is still set.

**How to avoid:** `op_integ` / `op_solve` / `op_difeq` should reset `cancel_requested.store(false, Relaxed)` at OP ENTRY (per the doc-comment at `integ.rs` line 223 which says "the workflow opener resets it" but doesn't say WHERE — verify Plan 29 / CLI-07 actually wired this reset). **Planner action: read `submit_step` for each program in Phase 28 and confirm the reset happens at modal-open time, not at op_integ_run_loop entry.** If not wired, Plan 31-02 adds the reset.

**Warning signs:** integration test `start_intg_cancel_then_start_again` fails on the second start with Canceled.

### Pitfall 9: Tauri command `submit_modal` is needed but missing from existing Tauri surface

**What goes wrong:** D-31.1 R/S branch 1 dispatches `submit_modal`. There is no such command — Phase 31 must add it. Same for `cancel_modal` (D-31.2 Esc branch 1).

**Why it happens:** Phase 29 added `submit_modal` and `cancel_modal` as pub fns in `hp41-core::ops::math1` but did not author Tauri thunks (CLI doesn't need them — it calls them directly).

**How to avoid:** Plan 31-05 (or 31-03 — see Plan structure recommendation) adds three thunks:

```rust
#[tauri::command]
pub fn submit_modal(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    hp41_core::ops::math1::submit_modal(&mut calc).map_err(GuiError::from)?;
    handle_get_state_inner(&mut calc)
}

#[tauri::command]
pub fn cancel_modal(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    hp41_core::ops::math1::cancel_modal(&mut calc); // no Result
    handle_get_state_inner(&mut calc)
}

#[tauri::command]
pub fn submit_modal_with_label(label: &str, state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    hp41_core::ops::math1::submit_modal_with_label(&mut calc, label).map_err(GuiError::from)?;
    handle_get_state_inner(&mut calc)
}
```

(All three return a `CalcStateView` so the frontend gets the updated state including new `modal_prompt` after submit advances the step.)

**Warning signs:** Frontend `invoke('submit_modal')` rejects with "command not found".

### Pitfall 10: CalcStateView missing the fields needed for frontend dispatch decisions

**What goes wrong:** D-31.1 R/S routing inspects `state.modal_program.is_some()` AND D-29.9 post-dispatch auto-open inspects `modal_program.requires_alpha_label()`. Neither is currently exposed on CalcStateView (verified — lines 26-57 of `types.rs` — no `modal_program` field).

**Why it happens:** CalcStateView was designed in Phase 14 (v2.0) before Math Pac I existed; Phase 28 added the field on CalcState but did NOT extend the projection.

**How to avoid:** Plan 31-03 (or 31-05 — wherever the frontend dispatch flow lands) adds two new projected fields:

```rust
// hp41-gui/src-tauri/src/types.rs CalcStateView (additions)
pub modal_program_active: bool,         // = state.modal_program.is_some()
pub modal_requires_alpha_label: bool,   // = state.modal_program.as_ref().map(|m| m.requires_alpha_label()).unwrap_or(false)
pub modal_prompt: Option<String>,       // = state.modal_prompt.clone() — exposed to LCD only via display_str routing per D-31.5, BUT useful for debugging + accessibility
pub is_running: bool,                   // = state.is_running — required by R/S 3-way branch 2 (D-31.1)
```

**Critical:** `is_running` is currently NOT on CalcStateView (verified — line 25-57 of types.rs lists 14 fields, none of which is is_running). The CLI's `state.is_running` is core-side; the GUI's R/S 3-way routing NEEDS this as a projection.

**Payload size check:** D-26.11 budget is 500 bytes. Adding `is_running: bool` (~10 bytes), `modal_program_active: bool` (~25 bytes), `modal_requires_alpha_label: bool` (~33 bytes), `modal_prompt: Option<String>` (~30 bytes for null, ~50 bytes for typical prompt) totals ~100 bytes additive. Current baseline 337 bytes → 437 bytes < 500 budget. **Headroom: 63 bytes.** Plan 31-03 verifies via the existing `test_dispatch_op_payload_size` test.

**Warning signs:** TypeScript type error in `App.tsx` referencing `calcState.modal_program_active` or `calcState.is_running`.

### Pitfall 11: GUI re-renders during CAT 2 print scroll discard intermediate `print_lines`

**What goes wrong:** v2.2 `op_catalog` pushes ALL lines into `print_buffer` synchronously (verified: `program.rs` line 300-345 — single-pass loop). The next `get_state` response carries all 45+1+1 = 47 lines in `print_lines`. React's `printLog` state appends them all at once. No PSE delay, no progressive reveal.

**Why it happens:** Phase 28 designed CAT 2 like v2.2 CAT 1 — single-shot enumeration. The CONTEXT.md D-31.12/D-31.14 references to "PSE-step infrastructure" describe a behavior that v2.2 CAT 1 does NOT have.

**How to avoid:** Two paths:
- (a) Accept the instant-scroll (matches v2.2 CAT 1) and update CONTEXT.md's D-31.12/D-31.14 wording.
- (b) Frontend setInterval that animates the print panel scroll position over ~22s (50 lines × 500ms) without changing core. This is purely cosmetic and adds frontend timer state — out of v3.0 scope.

**Recommendation:** option (a). Update planner to note CONTEXT D-31.12/D-31.14 is descriptive of the intended UX but the underlying mechanism is "synchronous push to print_buffer, instant render". Defer PSE-step infrastructure to v3.1 polish.

**Warning signs:** CONTEXT.md states CAT 2 is "scroll" but actual behavior is instant — visual feedback mismatch unless deferred.

---

## Code Examples

### Verified patterns from in-tree sources

#### Example 1: Tauri command thunk + AppState mutex (existing v2.1 run_stop)

```rust
// Source: hp41-gui/src-tauri/src/commands.rs:256-260
#[tauri::command]
pub fn run_stop(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_run_stop(&mut calc)
}
```

#### Example 2: Tauri permission TOML (existing v2.1 run-stop.toml)

```toml
# Source: hp41-gui/src-tauri/permissions/run-stop.toml (verbatim)
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-run-stop"
description = "Allows the run_stop command."
commands.allow = ["run_stop"]
```

#### Example 3: Vite static JSON-import (existing v2.2 help_data.ts)

```typescript
// Source: hp41-gui/src/help_data.ts:13
import functions from '../../docs/hp41cv-functions.json';

export function helpEntries(): readonly HelpEntry[] {
    return functions as readonly HelpEntry[];
}
```

#### Example 4: Per-64-samples cancellation check (existing Phase 28 op_integ)

```rust
// Source: hp41-core/src/ops/math1/integ.rs:325-329
for k in 0..=n_even {
    // ── Per-64-samples cancellation check (D-28.7 / D-28.8) ───────
    if k & 0x3F == 0 && state.cancel_requested.load(Ordering::Relaxed) {
        state.integ_state = None;
        return Err(HpError::Canceled);
    }
    // ... sample computation
}
```

#### Example 5: CLI R/S → submit_modal interception (existing Phase 29)

```rust
// Source: hp41-cli/src/app.rs:664-683
if key.code == KeyCode::F(5)
    && self.state.modal_program.is_some()
    && self.pending_input.is_none()
{
    match hp41_core::ops::math1::submit_modal(&mut self.state) {
        Ok(()) => {
            self.message = None;
            self.drain_and_show_print_output(None);
        }
        Err(e) => self.message = Some(format!("{e}")),
    }
    self.state.last_key_code = 0;
    return;
}
```

#### Example 6: CLI post-dispatch auto-open (existing Phase 29 D-29.9 — mirror target for GUI)

```rust
// Source: hp41-cli/src/app.rs:1782-1795
fn maybe_auto_open_collect_for_modal(&mut self) {
    if self.pending_input.is_some() {
        return;
    }
    let Some(ref mp) = self.state.modal_program else {
        return;
    };
    if mp.requires_alpha_label() {
        self.pending_input = Some(PendingInput::XeqByName {
            acc: String::new(),
            mode: XeqByNameMode::CollectForModal,
        });
    }
}
```

GUI mirror:

```typescript
// Plan 31-05 target (App.tsx new useEffect)
useEffect(() => {
  if (!calcState) return;
  if (pendingInput !== null) return;
  if (!calcState.modal_program_active) return;
  if (!calcState.modal_requires_alpha_label) return;
  setPendingInput({
    kind: 'xeq_name',
    dispatchPrefix: 'xeq',
    acc: '',
    mode: 'collect-for-modal',
  });
}, [calcState, pendingInput]);
```

#### Example 7: Auto-save thread lock-release pattern (existing v2.0 lib.rs commit ff39017)

```rust
// Source: hp41-gui/src-tauri/src/lib.rs:49-55
let state = handle.state::<AppState>();
let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
if let Err(e) = persistence::save_state(&thread_save_path, &snapshot) {
    eprintln!("auto-save failed: {e}");
}
```

#### Example 8: Truncate with continuation marker (Plan 31-05 target — Rust)

```rust
// Plan 31-05 target — hp41-gui/src-tauri/src/commands.rs (or types.rs helper)
const LCD_WIDTH: usize = 12;
const CONTINUATION: char = '\u{2261}'; // HP-41 ≡ truncation marker

fn truncate_with_continuation(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= LCD_WIDTH {
        return s.to_string();
    }
    let mut result: String = chars.iter().take(LCD_WIDTH - 1).collect();
    result.push(CONTINUATION);
    result
}

// LCD-alternation routing in handle_get_state:
if state.modal_program.is_some() && state.entry_buf.is_empty() {
    if let Some(ref prompt) = state.modal_prompt {
        display_str = truncate_with_continuation(prompt);
    }
}
```

#### Example 9: D-25.6 parity test pattern (Plan 31-03 target)

```rust
// hp41-gui/src-tauri/tests/d25_6_parity.rs (new test file)
//
// Drive identical input through xrom_resolve from both CLI's resolver and
// GUI's dispatch_op path. Both must produce identical X-register output.
#[test]
fn parity_sinh_1_5() {
    use hp41_core::{CalcState, HpNum, ops::dispatch, ops::Op};
    use hp41_core::ops::math1::xrom::xrom_resolve;

    // CLI path: xeq_by_name_local_resolve("SINH", state.xrom_modules) → Op::Sinh
    let resolved_cli = xrom_resolve("SINH", 0b0000_0001);
    assert_eq!(resolved_cli, Some(Op::Sinh));

    // GUI path: dispatch_op("xeq_SINH") → key_map → Op::Xeq("SINH") → run_program → xrom_resolve → Op::Sinh
    let mut state_gui = CalcState::new();
    state_gui.stack.x = HpNum::from_f64(1.5).unwrap();
    state_gui.program.push(Op::Lbl("MAIN".into()));
    state_gui.program.push(Op::Xeq("SINH".into()));
    state_gui.program.push(Op::Rtn);
    hp41_core::run_program(&mut state_gui, "MAIN").unwrap();

    // Direct dispatch baseline
    let mut state_direct = CalcState::new();
    state_direct.stack.x = HpNum::from_f64(1.5).unwrap();
    dispatch(&mut state_direct, Op::Sinh).unwrap();

    assert_eq!(state_gui.stack.x, state_direct.stack.x, "GUI path must match direct dispatch");
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|---|---|---|---|
| Tauri v1 `tauri.conf.json` allowlist | Tauri v2.11 per-command TOML permissions | Phase 13 (v2.0) | Each new command needs `permissions/<kebab>.toml` |
| Synchronous IPC commands holding state for long | Lock-release-during-IO pattern (commit ff39017) | Phase 16 / commit ff39017 | Long ops (auto-save, future Math Pac I cancellation) interleave correctly |
| Polling for cancellation via Tauri command | `Arc<AtomicBool>` + lock-free polling | Phase 28 | Mutex-free cancellation; sub-microsecond overhead per sample |
| `key_map.rs` containing function mnemonics | `xeq_<NAME>` key prefix → `Op::Xeq(name)` → core resolver chain | Phase 25 (v2.2) | `key_map.rs` stays small; new XROM modules add zero entries |
| Single-file JSON for help overlay | Multi-file parallel-load via Vite imports | Phase 26 W8 / D-25.16 | Math Pac I JSON loads at build time; malformed file fails build |
| `print_buffer` for both prompts and PRX output | `modal_prompt` for prompts; `print_buffer` for PRX only | Phase 28 D-28.4 | Clean lifecycle: modal_prompt cleared on submit; print_buffer accumulates |

**Deprecated/outdated:**

- **Hand-curated KEY_REF_TABLE** (replaced Phase 25 D-25.18): the right-panel listing derives from `help_data::help_entries()` filtered by non-null `key_path`. Math Pac I entries with `key_path == null` (Phase 29 D-29.1 authoring) won't appear in the v2.2 right panel — they'll appear in the `?`-overlay's Math 1 Pac section only.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The `scripts/check-tauri-permissions.sh` CI gate does NOT exist (CONTEXT line 32 references it as if it does) | §Open Questions Q1 | If it DOES exist, Plan 31-02's "author this script" task is redundant. Mitigation: Bash audit shows zero matches under `scripts/` — verified at research time |
| A2 | v2.2 CAT 1 has NO PSE-step delay; it's a synchronous push-to-buffer | §Pitfall 11 + Open Q2 | If a PSE infrastructure exists that I missed, Phase 31 inherits it; if not (as I observed), CONTEXT D-31.12/D-31.14 needs documentation drift fix |
| A3 | The R/S 3-way priority `modal_program > is_running > run_stop` is RIGHT — but `is_running` for active INTG/SOLVE/DIFEQ is only set during the actual long compute, not during modal-input phase. When `modal_program.is_some()` is true AND the user is at a numeric prompt, R/S submits the modal (branch 1). When `modal_program.is_some()` is FALSE (modal completed) AND `is_running` is true (long compute in flight), R/S cancels (branch 2). Mutex contention means is_running is observable ONLY between dispatch boundaries — the frontend's last `get_state` poll may show stale state during INTG. | §"Cancellation channel design" | If `is_running` is NOT set during INTG (verify by tracing op_integ_run_loop), the R/S branch-2 cancel never fires. Pitfall 1 may apply |
| A4 | The U+2261 `≡` glyph is best rendered as SEGMENT_MAP entry `[0, 6, 7, 3]` (top + middle + bottom — three-bar stripes). Other plausible patterns: `[6, 7, 3]` (same as `=`), `[6, 7]` (middle only, like `-`). | §Pitfall 6 + Claude's Discretion | Visual quality only — no functional impact |
| A5 | The cancel reset happens at modal-open time in Phase 28's `Op::Integ`/`Op::Solve`/`Op::Difeq` workflow openers, NOT at `op_integ_run_loop` entry. I have NOT verified this directly. | §Pitfall 8 | If not wired, sticky cancel breaks the "start INTG → cancel → start again" use case. Plan 31-02 audit task: grep for `cancel_requested.store(false` in `hp41-core/src/ops/math1/` |
| A6 | The Vite `vite.config.ts` `fs.allow` setting (line 18-19) applies in production builds as well as dev server. | §Pitfall 7 | If only dev, the production bundle may fail. I believe this is fine because static `import` statements are resolved by Rollup at build time and don't go through `fs.allow` — but verify in CI. |
| A7 | `extractErrMessage` in App.tsx line 48 handles `GuiError { message: "CANCELED" }` correctly. | §Pitfall 4 | If it doesn't, the toast renders "[object Object]" — fix in `types.rs::From<HpError>` |
| A8 | The Tauri command `submit_modal_with_label` accepts a `&str` parameter via `#[tauri::command]` thunk. Tauri v2 supports owned `String` and `&str` parameters via serde deserialization. | §Pitfall 9 | If `&str` is rejected by Tauri v2's command macro, switch to `String`. Trivial fix. |
| A9 | The 500-byte CalcStateView payload budget has 63 bytes of headroom after Phase 31 additions. | §Pitfall 10 | If `modal_prompt: Option<String>` carries a long prompt (>120 chars), budget may break. All known Math Pac I prompts are ≤14 chars — well within budget |

**Critical assumptions for planner verification:** A3, A5 — both are about cancellation correctness. Plan 31-02 must verify both via integration tests.

---

## Open Questions

1. **Does `scripts/check-tauri-permissions.sh` exist?**
   - What we know: CONTEXT.md line 32 + Code Insights line 225 reference it as a Pitfall 21 CI gate; CLAUDE.md mentions it; the v2.0 ROADMAP mentions it. Bash audit (`find scripts -name "*.sh"`) returns ZERO results. The `scripts/` directory contains only `docs-matrix/` (a Rust binary subdirectory).
   - What's unclear: whether the script was planned but never authored (research-time gap), or authored under a different name, or living in `.github/workflows/`.
   - Recommendation: Plan 31-02 includes a Wave 0 task to AUTHOR the script. Shape:
     ```bash
     #!/usr/bin/env bash
     # scripts/check-tauri-permissions.sh — verify every generate_handler! command has a TOML
     set -euo pipefail
     HANDLER_FILE="hp41-gui/src-tauri/src/lib.rs"
     PERMS_DIR="hp41-gui/src-tauri/permissions"
     commands=$(grep -oP 'commands::\K\w+' "$HANDLER_FILE")
     missing=0
     for cmd in $commands; do
         kebab=$(echo "$cmd" | sed 's/_/-/g')
         if [[ ! -f "$PERMS_DIR/$kebab.toml" ]]; then
             echo "MISSING: $PERMS_DIR/$kebab.toml (for command $cmd)"
             missing=1
         fi
     done
     exit $missing
     ```
     Plan 31-02 calls it from `justfile` `gui-ci` recipe.

2. **Should CAT 2 use PSE-step delay or instant-scroll?**
   - What we know: v2.2 `op_catalog` is a synchronous push-to-buffer (verified `program.rs` line 300-345). CONTEXT D-31.12/D-31.14 describe "PSE scroll" with "~500ms PSE delay between lines" and "R/S pauses/resumes" — but the underlying core mechanism for that doesn't exist in v2.2.
   - What's unclear: whether the CONTEXT.md drafter assumed v2.2 CAT 1 had PSE-step (it doesn't) or intended Phase 31 to BUILD it.
   - Recommendation: Defer PSE-step infrastructure to v3.1 polish. Plan 31-04 ships instant-scroll (matches v2.2 CAT 1). Update CONTEXT.md / 31-VERIFICATION.md note: D-31.12/D-31.14 effective UX is "lines appear in print panel synchronously".

3. **Is `state.is_running` exposed on CalcStateView already, or does Phase 31 add it?**
   - What we know: types.rs lines 25-57 enumerate 14 fields — none is `is_running`. The Annunciators struct (lines 16-23) doesn't include is_running either.
   - What's unclear: whether `is_running` is bound to one of the existing annunciators (e.g., `prgm` or a new one) or actually missing.
   - Recommendation: Phase 31 Plan 31-03 ADDS `is_running: bool` to CalcStateView. Verify via the existing `test_dispatch_op_payload_size` test that the new field fits in the 500-byte budget.

4. **For the matrix-prompt loop (`A1,1=?` → `A1,2=?` → ...), how is loop state held across resume-from-save-file?**
   - What we know: `matrix_dim`, `matrix_active_reg`, `modal_program: Option<ModalProgram>` are on CalcState with `#[serde(default, skip)]` (transient — never persisted).
   - What's unclear: if the user saves mid-MATRIX-entry, all loop state is lost. This is intentional per the `#[serde(skip)]` design but it's worth confirming.
   - Recommendation: Phase 31 deferred — Phase 32 / Pitfall 12 ships the round-trip test that confirms transient fields reset on load. No Phase 31 work.

5. **How is "long op is running" detected on the frontend for the R/S 3-way branch?**
   - What we know: D-31.1 R/S branch 2 fires when `state.is_running`. After Plan 31-03 adds the field projection, the frontend reads `calcState.is_running`.
   - What's unclear: between dispatch start and the first `get_state` poll, the frontend doesn't know `is_running` has flipped. Tauri's command queue is single-channel, so concurrent `get_state` calls during INTG would queue behind the `dispatch_op(Op::Integ)` and only resolve AFTER INTG completes. The user's R/S during INTG hits the frontend's stale state.
   - Recommendation: This is the structural reason cancellation must use a SEPARATE managed state (not AppState) — `request_cancel` doesn't go through the same command queue. The frontend optimistically routes R/S → `request_cancel` whenever `is_running` was true AT THE LAST get_state response — which is sufficient because the AtomicBool flip is idempotent and harmless if INTG isn't actually running. (If INTG completed between the last poll and the R/S press, the cancel flag is set but never read; the next INTG resets it.)

6. **Does `op_integ` etc. reset `cancel_requested` to false at workflow-opener time?**
   - What we know: comment at `integ.rs` line 223 says "cancel_requested is NOT reset here — the workflow opener resets it". Pitfall 8 assumes this is correct.
   - What's unclear: WHICH function in `hp41-core/src/ops/math1/integ.rs` is the "workflow opener" and whether it actually resets the flag.
   - Recommendation: Plan 31-02 audit task — grep `cancel_requested.store(false` in `hp41-core/src/ops/math1/`. If not found, ADD the reset at the workflow-opener entry (one-line fix).

7. **`submit_modal_with_label` vs extending `submit_modal` with optional param?**
   - What we know: CONTEXT line 316 says "Phase 31's GUI needs a Tauri command thunk for this; whether it lives as a new `submit_modal_with_label` command or as an extension of existing `submit_modal` taking an optional label parameter — planner picks". Phase 29 already wired `submit_modal_with_label` as a separate `hp41-core` fn (line 109 of mod.rs).
   - What's unclear: whether to expose two thunks or one.
   - Recommendation: Two thunks. Matches Phase 29's `hp41-core` shape; clearer semantics; each thunk has its own permission TOML.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | `hp41-core` + `hp41-gui/src-tauri` builds | ✓ | 1.88 MSRV (Cargo.toml verified) | — |
| Cargo | Standard | ✓ | bundled with rustc | — |
| Node.js | Vite + npm | ✓ | LTS (CI uses `lts/*` per `.github/workflows/ci-gui.yml`) | — |
| Tauri 2.11 | Existing dep | ✓ | locked in `hp41-gui/src-tauri/Cargo.toml` | — |
| React 19 + TypeScript | Existing dep | ✓ | package.json | — |
| `tauri-driver` 2.0.6 | WebdriverIO E2E (Phase 32) | ✓ | CI cache key from Phase 27 | — |
| `webkit2gtk-driver` + `xvfb` | Linux E2E | ✓ | apt installed in CI | — |
| `just` | Task runner | ✓ | CI installs via taiki-e/install-action | — |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** None.

**Skip condition: NOT met** — Phase 31 has external dependencies on Tauri, React, Vite, Node.js.

---

## AppState Mutex + AtomicBool interleaving (deep-dive)

This is the most technically delicate part of Plan 31-02. Worth a dedicated section.

### The threading model

- **Tauri command thread (per-IPC)**: each invocation of `dispatch_op`, `request_cancel`, etc. runs on a Tauri-managed thread. The runtime ensures commands are queued (no concurrent `dispatch_op` invocations — Tauri's single-channel command queue).
- **Auto-save thread**: spawned in `lib.rs::run()` setup. Sleeps 30s, locks AppState briefly to clone, drops lock, writes to disk.
- **AppState Mutex**: protects `CalcState`. The `dispatch(state, op)` call holds the MutexGuard for the entire duration of the op — including `op_integ` which can take minutes.

### Why `cancel_requested: Arc<AtomicBool>` works

The Mutex protects the `CalcState` struct AS A WHOLE — including the `cancel_requested` field. But `cancel_requested` is an `Arc<AtomicBool>`, which means:

- The Arc points to the same `AtomicBool` instance regardless of who holds the Mutex.
- Atomic operations (`.load(Relaxed)`, `.store(true, Relaxed)`) are lock-free.
- If we have an `Arc::clone` of `cancel_requested`, we can call `.store(true)` without going through the Mutex.

### The dispatch pattern (Plan 31-02 implementation)

```rust
// hp41-gui/src-tauri/src/lib.rs
use std::sync::{Arc, atomic::AtomicBool};

// New type alias for the second managed state
pub type CancelFlag = Arc<AtomicBool>;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let initial_state = /* load or fresh */;
            // Clone the Arc BEFORE wrapping in Mutex — both managed states see
            // the same underlying AtomicBool.
            let cancel_flag: CancelFlag = Arc::clone(&initial_state.cancel_requested);
            let app_state = Mutex::new(initial_state);
            app.manage(app_state);
            app.manage(cancel_flag);  // ★ second managed state — NO Mutex

            // ... existing auto-save thread spawn ...
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::dispatch_op,
            commands::get_state,
            commands::sst_step,
            commands::bst_step,
            commands::run_stop,
            commands::submit_modal,      // NEW
            commands::cancel_modal,      // NEW
            commands::submit_modal_with_label, // NEW
            commands::request_cancel,    // NEW
        ])
        .run(/* ... */)
        .expect("error while running tauri application")
}
```

```rust
// hp41-gui/src-tauri/src/commands.rs (request_cancel — the critical thunk)
use crate::CancelFlag;

#[tauri::command]
pub fn request_cancel(cancel: State<'_, CancelFlag>) -> Result<(), GuiError> {
    // No AppState lock! This thunk completes in microseconds even if
    // dispatch_op is holding the AppState Mutex for a 30-second INTG.
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}
```

### The user flow

1. User clicks XEQ "INTG" → frontend dispatches `dispatch_op("xeq_INTG")`.
2. Tauri thread T1 acquires AppState Mutex, calls `dispatch(state, Op::Integ)`.
3. `op_integ_run_loop` runs — holds the Mutex for the entire computation.
4. Every 64 samples, `op_integ` reads `state.cancel_requested.load(Relaxed)` — lock-free, fast.
5. User presses R/S. Frontend reads stale `calcState.is_running = true` (from last poll). Routes through `invokeForKey('r_s')` → branch 2 → `invoke('request_cancel')`.
6. Tauri thread T2 picks up `request_cancel`. Tries `State<'_, CancelFlag>` — succeeds INSTANTLY because CancelFlag is a separate managed state, not behind the AppState Mutex.
7. T2 calls `cancel.store(true, Relaxed)`. Returns Ok.
8. T1's next per-64-samples check sees `true`. Sets `integ_state = None`, returns `Err(HpError::Canceled)`.
9. `dispatch_op` returns `Err(GuiError { message: "CANCELED" })` to frontend.
10. Frontend's promise rejects → toast renders.

### Why this is safe

- **No deadlock**: T2 never tries to acquire the AppState Mutex.
- **No race**: AtomicBool with `Ordering::Relaxed` is sufficient because there's no other state that depends on it. The flag is "eventually consistent" — T1 may read it as `false` for one more poll cycle after T2's store, but the next cycle picks it up. 64 samples = ~64µs typically; user-perceived cancel latency = <1ms.
- **No starvation**: T1's MutexGuard is never released by `op_integ` — but that's fine because T2 doesn't need it.
- **Cleanup**: `op_integ` clears `integ_state = None` before returning Canceled — guards the next INTG against a stuck mid-iteration state.

### What about `submit_modal` and `cancel_modal`?

These thunks DO take `State<'_, AppState>` because they mutate CalcState fields beyond just the AtomicBool. They COULD theoretically be invoked while a long INTG is in progress — but per the user flow above, the user can't START a modal while INTG is running (the keyboard listener is disabled by `busyRef`, and dispatch_op is queued behind the active INTG). So in practice these thunks only fire when no long op is running. Plan 31-02 doesn't need special handling here.

### Reset of `cancel_requested` (Open Q6)

After a successful cancel, `cancel_requested == true`. The next `op_integ`/`op_solve`/`op_difeq` must reset to `false` AT WORKFLOW-OPENER ENTRY (not at `run_loop` entry — per the `integ.rs` line 223 comment). Plan 31-02 audit:

```bash
grep -rn "cancel_requested.store(false" hp41-core/src/ops/math1/
```

If this returns zero, Plan 31-02 ADDS the reset at the start of each `submit_step` (or equivalent workflow-opener) for Integ, Solve, Difeq.

---

## XEQ-by-name modal (deep-dive)

CONTEXT.md decision is that GUI-02 routes Math Pac I through the SAME `xrom_resolve` core function. Research finding: **this already works end-to-end with zero new code beyond Plan 31-03's parity test.**

### The current resolver chain in `dispatch_op`

```
dispatch_op("xeq_SINH", state)
  → key_map::resolve("xeq_SINH")
      → resolve_parameterized: prefix "xeq_" → Op::Xeq("SINH")    [v2.0 path]
  → dispatch(state, Op::Xeq("SINH"))
  → execute_op handles Op::Xeq
      → run_program(state, "SINH")
          → linear scan state.program for Op::Lbl("SINH") — miss
          → fallback: xeq_by_name_local_resolve("SINH", state.xrom_modules)  [v2.2 path]
              → builtin_card_op("SINH") — miss
              → xrom_resolve("SINH", state.xrom_modules) — HIT! → Op::Sinh   [Phase 29 wiring]
          → dispatch(state, Op::Sinh)
              → op_sinh(state)
              → state.stack.x = sinh(state.stack.x)
```

### Why this is automatic

Phase 29's CLI integration (Plan 29-01) added `xrom_resolve` as the FINAL fallback in `xeq_by_name_local_resolve` AND in `run_program`'s execute_op for Op::Xeq. Both code paths are in `hp41-core`, which the GUI shares. The GUI's existing `dispatch_op("xeq_SINH")` automatically benefits without any GUI-side change.

### What Plan 31-03 must actually do

Plan 31-03 reduces to:

1. **Add `submit_modal`, `cancel_modal`, `submit_modal_with_label` Tauri thunks** (for D-31.1 / D-31.2 / D-29.9 routing — these are NOT for XEQ-by-name itself but for the modal workflow that XEQ-by-name TRIGGERS for INTG/SOLVE/DIFEQ).
2. **Add the 3 permission TOMLs.**
3. **Add a parity test** (see Example 9) that drives `XEQ "SINH" 1.5 Enter` through both CLI and GUI paths and asserts identical X output.
4. **Add CalcStateView projection fields**: `modal_program_active`, `modal_requires_alpha_label`, `modal_prompt`, `is_running` (per Pitfall 10). Plan 31-03 owns this because the modal commands return updated CalcStateView and the frontend's auto-open hook needs these fields.

GUI-02 acceptance: `XEQ "SINH" 1.5 ENTER` → LCD reads `2.1293` (sinh(1.5) ≈ 2.12928...). The existing XEQ-by-name modal in App.tsx (line 152) already opens for the `xeq_prompt` key id. Typing `S I N H Enter` already dispatches `xeq_SINH`. End-to-end works after Phase 29's hp41-core wiring already shipped.

---

## Common Pitfalls (continued)

### Pitfall 12: SC-4 grep contradicts itself for `op_display_name`

**What goes wrong:** The stricter SC-4 grep `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` correctly excludes `op_display_name` (which is the documented exception). But Plan 31-01's "verify ~40 new arms" task is a no-op if Phase 28 plans 28-02..28-10 already shipped them.

**Why it happens:** CONTEXT line 73 + line 31 + line 45 explicitly state the arms ALREADY shipped in Phase 28 plans 28-02..28-10. Per Phase 29 / D-29.X plans verification.

**How to avoid:** Plan 31-01 is **a one-task verification plan**: read `hp41-gui/src-tauri/src/prgm_display.rs::op_display_name` and assert every `Op::*` variant from Phase 28's ~40 new variants has a matching arm. Compile-time exhaustive match ALREADY guarantees this (no `_ =>` catch-all). Plan 31-01 task: confirm no `_ =>` exists and that the file builds cleanly.

**Warning signs:** Plan 31-01 task description grows beyond 30 lines of verification — strong signal something has drifted from the "already shipped" state.

### Pitfall 13: Backward-compat `XeqByName.mode` field is optional but always read

**What goes wrong:** Existing `XeqByName` PendingInput entries in `pending_input.test.ts` and `App.tsx::MODAL_OPENERS` line 152-154 use the variant without a `mode` field. After Plan 31-05 extends the variant, TypeScript may fail to compile.

**Why it happens:** Discriminated-union extension with required field would break ALL existing entries.

**How to avoid:** Make `mode` optional with default `'normal'`:

```typescript
| { kind: 'xeq_name'; acc: string; dispatchPrefix: 'xeq' | 'gto' | 'lbl'; mode?: 'normal' | 'collect-for-modal' }
```

Existing entries continue to work (default mode = 'normal'). New `CollectForModal` entries explicitly set `mode: 'collect-for-modal'`. Switch statements consuming the variant treat undefined as 'normal' (one extra `?? 'normal'` per consumer).

**Warning signs:** TypeScript build error in App.tsx or pending_input.test.ts after the extension.

### Pitfall 14: D-25.6 parity test fails on x86 vs ARM due to f64 drift in `sinh`

**What goes wrong:** The parity test (Example 9) asserts `assert_eq!(state_gui.stack.x, state_direct.stack.x)`. But both paths use the SAME `op_sinh` function with the SAME inputs — there's no x86/ARM divergence between them. The test is bit-equal-checked because both paths feed identical inputs to identical code.

**Why it happens (or doesn't):** Parity tests inside the same process always produce identical f64 results regardless of architecture — there's no test failure here.

**How to avoid:** Use `assert_eq!` (strict) for the parity test, NOT `approx::assert_relative_eq!`. Reserve relative-tolerance assertions for cross-architecture comparisons (Phase 32 QUAL-06).

**Warning signs:** none expected — this is a clarifying note for the planner.

### Pitfall 15: Two parallel modal-dispatch IPC commands (submit_modal vs submit_modal_with_label) confuse the frontend state machine

**What goes wrong:** The frontend's XEQ-by-name `mode: 'collect-for-modal'` branch dispatches `submit_modal_with_label`. The frontend's R/S branch 1 dispatches `submit_modal`. The two are mutually exclusive but the frontend must pick the right one based on `modal_requires_alpha_label`.

**Why it happens:** Pattern is unclean — same-purpose action (advance the modal) split across two commands by label-vs-no-label.

**How to avoid:** Both thunks exist (Phase 29 already wired them in core). Frontend decision:

```typescript
// In the CollectForModal Enter branch (pending_input.ts handle_xeq_name)
if (pending.mode === 'collect-for-modal' && key === 'Enter') {
  // Caller dispatches submit_modal_with_label(pending.acc)
  return { ..., dispatchId: `__special__submit_modal_with_label:${pending.acc}` };
}

// In App.tsx invokeForKey, recognize the special prefix and route:
async function invokeForKey(effectiveId: string): Promise<CalcStateView> {
  if (effectiveId.startsWith('__special__submit_modal_with_label:')) {
    const label = effectiveId.slice('__special__submit_modal_with_label:'.length);
    return invoke<CalcStateView>('submit_modal_with_label', { label });
  }
  // ... existing routes ...
}
```

Alternative: use a separate frontend code-path (not invokeForKey) for label-bearing submit. Plan 31-05 picks.

**Warning signs:** `submit_modal` thunk is invoked with an empty alpha_reg, leaves the modal in FunctionNamePrompt state, frontend's auto-open re-fires, infinite loop. The `__special__` prefix prevents this.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Frontend Vitest framework | Vitest 1.x via `vite.config.ts` `test:` block |
| Backend Rust test framework | `cargo test` (standard) |
| Integration tests (Tauri) | `hp41-gui/src-tauri/tests/` (new) |
| E2E tests (WebdriverIO) | `hp41-gui/e2e/smoke.spec.ts` (existing — Phase 32 extends with Math Pac I workflow) |
| Quick run command | `just gui-ci` (TypeScript check + Vitest); `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` (Rust) |
| Full suite command | `just gui-ci && just test` (cargo workspace) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|--------------------|--------------|
| GUI-01 | `prgm_display.rs` has Op-variant arms for every Math Pac I Op | Compile-time (already shipped) + SC-4 grep | `grep -rn "fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)" hp41-gui/src-tauri/src/` returns nothing | ✅ existing |
| GUI-02 | XEQ "SINH" 1.5 → 2.1293 | Integration (Rust) | `cargo test --test d25_6_parity -- parity_sinh_1_5` | ❌ Wave 0 (Plan 31-03) |
| GUI-02 | `xeq_<NAME>` keys route through `xrom_resolve` correctly | Integration (Rust) | `cargo test --test d25_6_parity` | ❌ Wave 0 (Plan 31-03) |
| GUI-03 | Math Pac I JSON loads at build time | Build-time (Vite) | `cd hp41-gui && npm run build` (malformed JSON → build error) | ✅ existing pattern |
| GUI-03 | `?`-overlay renders two-section layout | Unit (Vitest) | `cd hp41-gui && npm test -- HelpOverlay` | ❌ Wave 0 (Plan 31-04) |
| GUI-04 | CAT 2 prints "XROM Math 1" + 45 function names | Integration (Rust) | `cargo test --test catalog_2_xrom_enumeration` | ❌ Wave 0 (Plan 31-04) |
| GUI-05 | `request_cancel` flips `cancel_requested` without locking AppState | Integration (Rust) | `cargo test --test request_cancel_no_deadlock` | ❌ Wave 0 (Plan 31-02) |
| GUI-05 | `op_integ` cancels mid-flight within 64 samples | Integration (Rust) | `cargo test math1::integ::cancel_during_run` (existing — line 870ish per integ.rs) | ✅ existing |
| GUI-05 | New permission TOMLs exist and match generate_handler | CI gate | `bash scripts/check-tauri-permissions.sh` | ❌ Wave 0 (Plan 31-02 authors the script) |
| GUI-06 | LCD shows `ORDER=?` after `XEQ "MATRIX" Enter` | Integration (Rust) | `cargo test --test lcd_alternation_modal_prompt` | ❌ Wave 0 (Plan 31-05) |
| GUI-06 | LCD truncates `FUNCTION NAME?` → `FUNCTION NAM≡` | Unit (Rust truncation helper) | `cargo test truncate_with_continuation_14_chars` | ❌ Wave 0 (Plan 31-05) |
| GUI-06 | R/S during modal calls submit_modal | Vitest (frontend) | `cd hp41-gui && npm test -- App` (new test case) | ❌ Wave 0 (Plan 31-05) |
| GUI-06 | Esc during modal calls cancel_modal | Vitest (frontend) | `cd hp41-gui && npm test -- App` (new test case) | ❌ Wave 0 (Plan 31-05) |
| GUI-07 | `key_map::resolve` stub-error arm unchanged | Integration (Rust) | `cargo test --test key_map_stub_error_arms` (existing — verify count = same) | ✅ existing |

### Sampling Rate

- **Per task commit:** `cd hp41-gui && npm test` + `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` (≤ 30s total for the GUI test surface)
- **Per wave merge:** `just gui-ci` (TypeScript check + Vitest + npm audit)
- **Phase gate:** Full suite: `just test` (cargo workspace) + `just gui-ci` + WebdriverIO E2E smoke (`xvfb-run just gui-e2e` on Linux) green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `hp41-gui/src-tauri/tests/d25_6_parity.rs` — D-25.6 CLI ↔ GUI parity (3+ Math Pac I functions: SINH, MATRIX/DET, INTG smoke)
- [ ] `hp41-gui/src-tauri/tests/request_cancel_no_deadlock.rs` — cancellation channel race-freeness (concurrent dispatch_op + request_cancel)
- [ ] `hp41-gui/src-tauri/tests/catalog_2_xrom_enumeration.rs` — CAT 2 prints expected lines
- [ ] `hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs` — handle_get_state routes modal_prompt → display_str
- [ ] `hp41-gui/src/HelpOverlay.test.tsx` — extend existing test with two-section assertion (existing file at `hp41-gui/src/HelpOverlay.test.tsx`)
- [ ] `hp41-gui/src/App.test.tsx` — extend existing test with R/S 3-way + Esc cascade cases
- [ ] `scripts/check-tauri-permissions.sh` — NEW CI gate script (see Open Q1)
- [ ] `hp41-core/src/ops/math1/` audit grep for `cancel_requested.store(false`. If zero matches, add the reset to each `submit_step` workflow-opener.

**Coverage expectations:** hold `hp41-core` line ≥ 95.25% / region ≥ 93.75% (no atomic raise this milestone per Phase 32 charter). New `op_catalog` arm gains 3-4 test cases (CAT 2 with module loaded / module not loaded / iteration order). `hp41-gui/src-tauri` coverage measured but NOT gated (Phase 27 D-27.4 — measure-only).

---

## Security Domain

Required per project security_enforcement default (absent = enabled). Phase 31 attack surface is narrow but worth enumerating.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-------------------|
| V2 Authentication | no | Desktop app, no auth |
| V3 Session Management | no | No sessions |
| V4 Access Control | no | Single-user local app |
| V5 Input Validation | yes | XEQ-by-name modal accepts arbitrary alpha labels; `key_map::resolve` returns Err(GuiError) for unknown ids (existing) |
| V6 Cryptography | no | No crypto in Phase 31 |
| V7 Error Handling | yes | `HpError::Canceled` → `GuiError { message: "CANCELED" }` via existing pattern; no stack trace leakage |
| V8 Data Protection | partial | `~/.hp41/autosave.json` saves modal_program transient fields are `#[serde(skip)]` — no user-input retention |
| V13 API and Web Service | yes (Tauri) | Per-command TOML permission allowlist; new `request-cancel.toml` etc. |

### Known Threat Patterns for Tauri Desktop App

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Tauri command without permission allowlist entry | Elevation of Privilege | `permissions/<kebab>.toml` per command. CI gate `scripts/check-tauri-permissions.sh` (NEW in Phase 31). |
| Frontend code constructs arbitrary Op enum payload | Tampering | All ops go through `key_map::resolve` → `Op` enum. Frontend cannot construct Op variants directly. Existing constraint. |
| Long-running INTG holds main thread, GUI freezes (DoS-like UX) | Denial of Service (self) | Cancellation channel (this Phase). |
| Math Pac I XEQ name shadows a built-in mnemonic | Tampering (privilege escalation between mnemonic intent and resolved Op) | `tests/xrom_shadowing.rs` (existing Phase 28 gate). Phase 31 reads MATH_1.ops in `op_catalog`; doesn't add new mnemonics. |
| Stale `cancel_requested` flag triggers spurious cancel on next op | Logic flaw | Workflow-opener reset of the flag (Open Q6 — verify Plan 31-02 audit). |
| Frontend `extractErrMessage` leaks JSON-serialized object containing sensitive fields | Information Disclosure | `GuiError` has only `message: String` — no leakage. Existing pattern. |
| Vite static JSON import path traversal | Tampering | Import paths are static literals — no user input. `vite.config.ts fs.allow` is repo-relative, can't escape. |

---

## Sources

### Primary (HIGH confidence)

- `.planning/phases/31-gui-integration/31-CONTEXT.md` — Phase 31 decision record (D-31.1..D-31.14 + carried decisions)
- `.planning/phases/31-gui-integration/31-UI-SPEC.md` — visual / interaction contract (Plan-checker reference)
- `.planning/REQUIREMENTS.md` — GUI-01..07 + Traceability table
- `.planning/ROADMAP.md` — Phase 31 section lines 154-185
- `CLAUDE.md` — v3.0 additions block (Phase 28/29/30) + SC-4 invariant + frozen invariants
- `hp41-core/src/state.rs` lines 200-235 — Phase 28 CalcState additions (cancel_requested, modal_program, etc.)
- `hp41-core/src/ops/math1/mod.rs` lines 54-135 — submit_modal, cancel_modal, submit_modal_with_label public API
- `hp41-core/src/ops/math1/xrom.rs` — xrom_resolve + MATH_1 registry
- `hp41-core/src/ops/math1/modal.rs` lines 24-83 — ModalProgram enum + requires_alpha_label() accessor
- `hp41-core/src/ops/math1/integ.rs` lines 200-330 — op_integ_run_loop with per-64-samples cancellation
- `hp41-core/src/ops/math1/solve.rs` line 384 — per-64-iterations cancellation check
- `hp41-core/src/ops/math1/difeq.rs` line 305 — per-64-steps cancellation check
- `hp41-core/src/ops/program.rs` lines 300-345 — op_catalog stub (CAT 2 = "NOT AVAILABLE")
- `hp41-core/src/error.rs` lines 37, 48 — HpError::Canceled variant + Display impl returns "canceled" (lowercase)
- `hp41-cli/src/app.rs` lines 664-697 — CLI R/S + Esc modal interception (Phase 29 D-29.5/D-29.6)
- `hp41-cli/src/app.rs` lines 1471-1527 — CLI XeqByName handler (Phase 29 D-29.8)
- `hp41-cli/src/app.rs` lines 1782-1795 — CLI maybe_auto_open_collect_for_modal (Phase 29 D-29.9)
- `hp41-gui/src-tauri/src/lib.rs` (verbatim 1-70) — generate_handler! macro + setup + auto-save thread pattern
- `hp41-gui/src-tauri/src/commands.rs` lines 49-289 — Tauri command thunk patterns; handle_op_prepare/finalize three-phase split
- `hp41-gui/src-tauri/src/types.rs` lines 16-146 — CalcStateView fields + From<HpError> impl
- `hp41-gui/src-tauri/permissions/run-stop.toml` (verbatim) — permission TOML template
- `hp41-gui/src/App.tsx` lines 60-410 — handleClick, handleKey, invokeForKey, MODAL_OPENERS, useEffects
- `hp41-gui/src/pending_input.ts` lines 52-69 — PendingInput discriminated union (14 variants)
- `hp41-gui/src/HelpOverlay.tsx` — existing two-level grouping pattern
- `hp41-gui/src/help_data.ts` line 13 — existing Vite JSON-import pattern
- `hp41-gui/src/Display14Seg.tsx` lines 131-184 — SEGMENT_MAP (≡ U+2261 absent)
- `hp41-gui/vite.config.ts` lines 18-19 — `fs.allow` already covers `..`
- `hp41-gui/src-tauri/Cargo.toml` — no new deps needed
- `docs/hp41-math1-functions.json` — 45 entries across 11 categories (verified via Python `json.load`)
- `.github/workflows/ci-gui.yml` — CI pipeline (build job + e2e-linux job)

### Secondary (MEDIUM confidence — cross-referenced but not direct primary)

- `.planning/phases/28-xrom-framework-math-pac-i-core-ops/28-CONTEXT.md` — Phase 28 decisions D-28.4..D-28.9
- `.planning/phases/29-cli-integration/29-CONTEXT.md` — Phase 29 decisions D-29.5..D-29.9 (CLI reference impl)
- `.planning/phases/30-documentation-adrs/30-CONTEXT.md` — ADR templates
- `docs/adr/v3.0-001-op-strategy.md` — Op-strategy A locked rationale
- `docs/adr/v3.0-002-user-callback-policy.md` — strict-reject nested INTG/SOLVE
- `docs/adr/v3.0-005-json-pipeline.md` — separate JSON file rationale

### Tertiary (LOW confidence — assumption-based)

- Open Q1: `scripts/check-tauri-permissions.sh` existence — bash audit `find scripts -name "*.sh"` returned zero. Documented as ASSUMPTION A1.
- Open Q2: CAT 2 PSE-step infrastructure — does NOT exist in v2.2 CAT 1 (verified). Documented as ASSUMPTION A2.
- Open Q3 / Pitfall 10: `is_running` not on CalcStateView (verified by reading types.rs). Documented as ASSUMPTION (verified at primary level).

---

## Metadata

**Confidence breakdown:**

- Standard stack (no new deps): HIGH — verified Cargo.toml + package.json, every "use" already exists in v2.2
- Architecture (cancellation channel design): HIGH — re-verified pattern against `lib.rs::setup` auto-save thread + integ.rs poll loop
- XEQ-by-name auto-routing: HIGH — verified via Phase 29's existing `xeq_by_name_local_resolve` + `xrom_resolve` chain
- Pitfalls: HIGH for Pitfalls 1-7, MEDIUM for Pitfalls 8-15 (some involve cross-thread races that need integration-test confirmation)
- Open Questions: 7 total — all flagged inline with recommended resolutions
- Validation Architecture: HIGH — every test scaffold has an existing pattern in v2.0+

**Research date:** 2026-05-17
**Valid until:** 2026-06-17 (30 days — Tauri v2.11 and React 19 are mature; risk of API drift in 30 days is LOW)

---

*Phase: 31-gui-integration*
*Research completed: 2026-05-17*
*Next: gsd-planner consumes this RESEARCH.md to produce 31-01-PLAN.md..31-05-PLAN.md*
