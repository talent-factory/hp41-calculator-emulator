# Phase 14: IPC Layer - Context

**Gathered:** 2026-05-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire `hp41-core` into the Tauri app via Rust commands. Deliver:
- `dispatch_op(key_id: &str)` — resolves key ID to Op, executes it, returns `CalcStateView`
- `get_state()` — returns current `CalcStateView` without executing any op
- `CalcStateView` — lean DTO (~200 bytes): `display_str`, `x_str`, `annunciators`, `print_lines`
- `key_map.rs` — `key_id: &str → Result<Op, GuiError>`; handles digit keys, named ops, compound parameterized keys
- `GuiError { message: String }` — serialized on unknown key ID or `HpError` from core
- `print_buffer` drain — every command clears `CalcState.print_buffer` and returns lines in `CalcStateView.print_lines`

This phase delivers purely the Rust-side IPC plumbing. It does NOT add React components (Phase 15), does NOT implement keyboard wiring (Phase 15), does NOT render the SVG skin (Phase 16), and does NOT add persistence (Phase 17).

</domain>

<decisions>
## Implementation Decisions

### CalcStateView Fields

- **D-01:** `CalcStateView` contains exactly these fields:
  - `display_str: String` — the 12-char HP-41 formatted display string (from `get_display_string()`)
  - `x_str: String` — X register value as a formatted string (same formatting as display_str but raw)
  - `annunciators: Annunciators` — struct with 5 boolean fields: `user`, `prgm`, `alpha`, `rad`, `grad`
  - `print_lines: Vec<String>` — drained from `CalcState.print_buffer` on every command
- **D-02:** `CalcStateView` does NOT include Y/Z/T/LASTX at Phase 14. Phase 15 will render the stack panel; it can call `get_state()` at that point or request a richer view. Keeping it lean now avoids over-engineering before Phase 15 validates the shape.
- **D-03:** `Annunciators` is a separate derived struct (not part of CalcState itself), populated by reading `CalcState` fields: `user_mode`, `prgm_mode`, `alpha_mode`, `angle_mode == Rad`, `angle_mode == Grad`.

### Key ID Convention

- **D-04:** **Digit keys use bare characters:** `"0"`–`"9"`, `"."`, `"e"`. key_map.rs maps these to `Op::PushNum` or the EEX-entry path (matching CLI digit handling). No prefix.
- **D-05:** **Named ops use snake_case:** `"enter"`, `"plus"`, `"minus"`, `"mul"`, `"div"`, `"chs"`, `"clx"`, `"rdn"`, `"xy_swap"`, `"lastx"`, `"sin"`, `"cos"`, `"tan"`, `"sqrt"`, `"enter"`, etc. Mirror the CLI `key_to_op()` mapping in `hp41-cli/src/keys.rs`.
- **D-06:** **Parameterized ops use compound key IDs:** e.g., `"sto_05"` (STO R05), `"rcl_12"` (RCL R12), `"fix_4"` (FIX 4), `"sci_2"` (SCI 2), `"eng_3"` (ENG 3), `"sto_arith_plus_05"` (STO+ R05), `"sto_arith_minus_y"` (STO− stack Y). key_map.rs parses these with prefix matching.
- **D-07:** **key_map.rs is exhaustive and returns `Result<Op, GuiError>`.** Unknown key IDs return `Err(GuiError { message: format!("unknown key: {key_id}") })`. No silent discard.

### Modal State Ownership

- **D-08:** **Phase 14 is stateless IPC only.** `key_map.rs` handles only atomic 1:1 (or compound-string) key→Op mappings. The frontend (Phase 15) owns multi-step modal sequencing: after the user presses S→+→05, the React layer constructs and sends `"sto_arith_plus_05"` as a single `dispatch_op` call.
- **D-09:** No `PendingModal` state is added to `AppState` in Phase 14. `AppState = Mutex<CalcState>` (already defined in `lib.rs`) remains the complete Rust-side state.

### Error Response Shape

- **D-10:** `dispatch_op` and `get_state` return `Result<CalcStateView, GuiError>`. Natural Tauri v2 pattern — the frontend's `invoke()` Promise rejects with the serialized `GuiError` on error.
- **D-11:** `GuiError` is minimal: `#[derive(Debug, Serialize)] pub struct GuiError { pub message: String }`. Covers both unknown key ID and `HpError` forwarding (via `impl From<HpError> for GuiError`).

### Tauri Capabilities

- **D-12:** `capabilities/default.json` is updated in Phase 14 to add IPC permissions for the two commands. Tauri v2 requires explicit `"hp41-gui:allow-dispatch-op"` and `"hp41-gui:allow-get-state"` entries (or equivalent plugin permission format).

### Claude's Discretion

- `hp41-core`'s `get_display_string()` already formats the 12-char display. Use it directly for `display_str`.
- `x_str` can use `state.stack.x.to_string()` or the same formatted path — Claude decides what's most useful for Phase 15 rendering.
- The `AppState` lock uses `.unwrap_or_else(|e| e.into_inner())` for poisoned-lock recovery (established in Phase 13 CONTEXT.md).
- `#![deny(clippy::unwrap_used)]` applies to all new Phase 14 code in `hp41-gui/src-tauri/src/`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap

- `.planning/ROADMAP.md` — Phase 14 goal, 5 success criteria (SC-1 through SC-5), dependency on Phase 13
- `.planning/REQUIREMENTS.md` — IPC-01 acceptance criteria

### Prior Phase Context

- `.planning/phases/13-workspace-skeleton/13-CONTEXT.md` — All Phase 13 decisions (workspace layout, bundle identifier, `AppState` type alias, capabilities pattern, just recipes, Tauri conf structure)

### Existing Source Files (must read before implementing)

- `hp41-gui/src-tauri/src/lib.rs` — `AppState = Mutex<CalcState>` already defined; `invoke_handler![]` placeholder where commands are registered
- `hp41-gui/src-tauri/capabilities/default.json` — current `core:default` only; must be extended with IPC permissions
- `hp41-gui/src-tauri/Cargo.toml` — nested workspace deps (`tauri 2.11`, `hp41-core` path dep)
- `hp41-core/src/ops/mod.rs` — full `Op` enum; `dispatch()` signature; `flush_entry_buf()` for digit entry
- `hp41-core/src/state.rs` — `CalcState` fields used to build `CalcStateView` and `Annunciators`
- `hp41-cli/src/keys.rs` — `key_to_op()` reference implementation; source of truth for named key ID → Op mapping

### Architecture Decisions

- `.planning/phases/13-workspace-skeleton/13-CONTEXT.md` D-04 through D-07 — workspace isolation guarantees that `cargo build --workspace` from root never touches `hp41-gui`

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `hp41-core::CalcState` — already `#[derive(Serialize, Deserialize, Clone)]`; `print_buffer: Vec<String>` field available to drain
- `hp41-core::dispatch()` — `pub fn dispatch(state: &mut CalcState, op: Op) -> Result<(), HpError>` in `hp41-core/src/ops/mod.rs:289`
- `hp41-core::get_display_string()` — formats the 12-char HP-41 display string; already used by `hp41-cli`
- `hp41-cli/src/keys.rs::key_to_op()` — complete key→Op mapping reference; key_map.rs is a string-keyed variant of this function
- `AppState = Mutex<CalcState>` — already defined in `hp41-gui/src-tauri/src/lib.rs`

### Established Patterns

- **Zero-panic policy:** `#![deny(clippy::unwrap_used)]` at `hp41-gui/src-tauri/src/lib.rs`. All new handlers use `?`-propagation or `.unwrap_or_else(|e| e.into_inner())` for Mutex.
- **Poisoned-lock recovery:** `.unwrap_or_else(|e| e.into_inner())` on `state.lock()` calls (from Phase 13 decision).
- **No core duplication:** all calculator logic stays in `hp41-core`; `hp41-gui/src-tauri` is a thin adapter.

### Integration Points

- `hp41-gui/src-tauri/src/lib.rs` — commands registered in `tauri::generate_handler![]`
- `hp41-core/src/state.rs` — `CalcState.print_buffer` drained after every `dispatch_op` call
- `hp41-core/src/ops/mod.rs` — digit-entry ops route through `flush_entry_buf()` as in the CLI

</code_context>

<specifics>
## Specific Ideas

- Key IDs mirror the CLI `key_to_op()` bindings: `"q"` → `Op::Sin` (Phase 8 assignment), `"g"` → `Op::Clreg`, `"enter"` → `Op::Enter`, etc.
- Compound key ID format for parameterized ops: `"sto_NN"`, `"rcl_NN"`, `"fix_N"`, `"sci_N"`, `"eng_N"`, `"sto_arith_<op>_<reg>"` where `<op>` is `plus`/`minus`/`mul`/`div` and `<reg>` is `NN` or `y`/`z`/`t`/`lastx`.

</specifics>

<deferred>
## Deferred Ideas

- Y/Z/T/LASTX in `CalcStateView` — deferred to Phase 15 when the stack panel is implemented; Phase 15 can request a richer view or call `get_state()` separately.
- TypeScript type generation for `CalcStateView` and `GuiError` — deferred to Phase 15 when the frontend first consumes these types.
- `get_state()` with no-op behavior — included in Phase 14 as a convenience for Phase 15 initial render; no state change, just returns current view.
- Stack panel rendering — Phase 15.
- Physical keyboard wiring — Phase 15.

</deferred>

---

*Phase: 14-IPC Layer*
*Context gathered: 2026-05-09*
