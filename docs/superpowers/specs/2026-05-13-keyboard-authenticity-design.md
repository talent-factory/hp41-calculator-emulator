# Keyboard Authenticity Refactor — Design

**Date:** 2026-05-13
**Scope:** `hp41-gui` only (`hp41-core` and `hp41-cli` untouched)
**Target milestone:** v2.1 Polish
**Status:** Approved by user (brainstorming gate passed)

## Problem

The current `hp41-gui` keyboard is a functional but **non-authentic** rendering of the HP-41C:

- 8-column landscape grid (vs. the real HP-41C's 5-column portrait layout)
- Two shift keys `f` and `g` (HP-41 physically has only **one** orange SHIFT)
- The `f`/`g` keys have `id: ''` and are **not wired** to any backend logic
- `fShiftLabel` text above keys is **purely decorative** — ~14 already-implemented core ops (`Sq`, `YPow`, `TenPow`, `Exp`, `Asin`/`Acos`/`Atan`, `SigmaMinus`, `Mean`, `Sdev`, `Lr`, `Yhat`, `Corr`, `ClSigmaStat`, `HmsToH`, `HToHms`, `HmsAdd`, `HmsSub`, `Int`, `Rtn`) are **unreachable via click**
- ALPHA-mode characters (blue letters under each key on the real HP-41) are missing entirely
- Warm-brown body color diverges from the authentic black HP-41C chassis

Result: a calculator that works but doesn't *feel* like an HP-41 and silently hides a third of its capability behind dead shift labels.

## Goals

1. Pixel-faithful HP-41C keyboard layout (5×7 grid + 4 mode buttons in a separate top row).
2. **One** functional orange SHIFT key with HP-41-authentic one-shot prefix semantics.
3. Every key shows three labels: shift-function (orange, top), primary (white, center), ALPHA character (blue, bottom).
4. Every previously-unreachable but already-implemented core op becomes click-reachable.
5. Honest "not implemented" feedback for shift functions whose core op is planned for v2.2 (no silent discards — respects D-07).
6. Zero changes to `hp41-core`, `hp41-cli`, IPC schema, persistence, or the display/stack panels.

## Non-Goals

- Implementing missing core ops (`Pi`, `PolarToRect`, `RectToPolar`, `Beep`, `Asn`, `Catalog`, `View`, flag ops `SF`/`CF`/`FS?`). Those land in **v2.2** as a separate milestone.
- Changing the display section, annunciator content (other than adding a `SHIFT` indicator), or stack panel.
- Modifying `hp41-cli` keybindings.
- Touching auto-save, persistence, or the Tauri command surface beyond the additions required to map newly clickable ops.

## Architecture Decisions

### D-1: SHIFT is frontend-only state

`shiftActive: boolean` lives in `App.tsx`. It does **not** appear in `CalcState`, `CalcStateView`, or any Tauri command response. Rationale:

- SHIFT is a GUI prefix mechanic, not a calculator state. The Core engine receives the **resolved** op (e.g., `Op::Sq`) — it never knows a SHIFT was involved.
- Keeps the IPC JSON budget (≤300 bytes) intact.
- Keeps `CalcState` clean for save-file compatibility.
- The `hp41-cli` TUI doesn't have a SHIFT key concept — it directly binds shift functions to physical keys — so this asymmetry is correct.

### D-2: One-shot SHIFT prefix (not latched)

After SHIFT is active, the next non-SHIFT click consumes it and resets `shiftActive` to `false`. This matches HP-41C hardware. Toggling SHIFT a second time without consuming it also resets it. Pressing physical `Esc` also resets it (parity with TUI cancel semantics).

### D-3: Three-label key model

Each non-special key carries up to three labels in `KEY_DEFS`:

```ts
type KeyDef = {
  id: string;              // primary op key id (e.g., 'sin')
  label: string;           // primary visible label ('SIN')
  shifted?: {
    id: string;            // shifted op key id ('asin')
    label: string;         // shifted label ('SIN⁻¹')
  };
  alphaChar?: string;      // alpha-mode character ('H')
  row: number;
  col: number;
  colSpan?: number;
  variant?: 'top' | 'shift' | 'enter';  // styling hint
};
```

Special keys: SHIFT (`variant: 'shift'`, no labels, solid orange), top-row buttons (`variant: 'top'`, no shift/alpha labels but may carry an orange caption like `ALPHA` text styling).

### D-4: Click resolution priority

When a key is clicked, the routing in `App.tsx`'s click handler decides the effective op id with this priority:

1. **Key is SHIFT** → toggle `shiftActive`, do not dispatch.
2. **ALPHA-mode active AND key has `alphaChar`** → dispatch `alpha_<char>`. Shift is **ignored** in this iteration (known divergence from HP-41: real hardware permits some shifted ops in alpha mode; deferred as out-of-scope).
3. **`shiftActive` AND key has `shifted.id`** → dispatch `shifted.id`, then `setShiftActive(false)`.
4. **Else** → dispatch `id` (primary).

### D-5: Stub-error pattern for unimplemented shift functions

Shift labels for ops not yet in the core engine (`π`, `P→R`, `R→P`, `BEEP`, `ASN`, `CATALOG`, `VIEW`) are rendered visually for HP-41 authenticity, but their key ids resolve to an explicit `GuiError` in `key_map::resolve`:

```rust
"pi" | "polar_to_rect" | "rect_to_polar" | "beep" | "asn"
| "catalog" | "view" => Err(GuiError {
    message: format!("'{key_id}' is planned for a future phase"),
}),
```

The frontend surfaces this message as a transient ~2s toast in the display area. This honors D-07 (never silent discard) while letting the layout be visually complete.

### D-6: Flag ops (`SF`, `CF`, `FS?`) deferred entirely

These need a 2-digit modal (analogous to STO register entry) plus a `flags` field in `CalcState`. Scope-too-large for this refactor — labels appear, click returns the same stub error as D-5.

## Layout Specification

```
Top row (separate band, gap in center):
┌──────┬──────┐                          ┌──────┬──────┐
│  ON  │ USER │                          │ PRGM │ ALPHA│
└──────┴──────┘                          └──────┴──────┘

Main grid (5 columns × 8 rows; ENTER spans 2):
Row 1:  Σ+   1/x   √x    LOG   LN          ← math
Row 2:  x≥y  R↓    SIN   COS   TAN         ← trig + stack
Row 3: [SHIFT] XEQ  STO   RCL   SST         ← program
Row 4: [  ENTER↑  ]  CHS  EEX   ←           ← entry (4 keys, ENTER 2-wide)
Row 5:   −    7    8     9    ·             ← operator + digits
Row 6:   +    4    5     6    ·
Row 7:   ×    1    2     3    ·
Row 8:   ÷    0    .     R/S  ·
                  (col 4 of rows 5-8 is empty space)
```

**Per-key three-label content (selected examples):**

| Key | Primary | Shifted | Alpha |
|-----|---------|---------|-------|
| Σ+ | Σ+ | Σ− | A |
| 1/x | 1/x | yˣ | B |
| √x | √x | x² | C |
| LOG | LOG | 10ˣ | D |
| LN | LN | eˣ | E |
| x≥y | x≥y | CLΣ | F |
| R↓ | R↓ | % | G |
| SIN | SIN | SIN⁻¹ | H |
| COS | COS | COS⁻¹ | I |
| TAN | TAN | TAN⁻¹ | J |
| XEQ | XEQ | ASN ⚠️ stub | K |
| STO | STO | LBL | L |
| RCL | RCL | GTO | M |
| SST | SST | BST | — |
| ENTER | ENTER↑ | CATALOG ⚠️ stub | N |
| CHS | CHS | ISG | O |
| EEX | EEX | RTN | P |
| ← | ← | CL X/A | — |
| − | − | x=y? ⚠️ stub | Q |
| 7 | 7 | SF ⚠️ stub | R |
| 8 | 8 | CF ⚠️ stub | S |
| 9 | 9 | FS? ⚠️ stub | T |
| + | + | x≤y? ⚠️ stub | U |
| 4 | 4 | BEEP ⚠️ stub | V |
| 5 | 5 | P→R ⚠️ stub | W |
| 6 | 6 | R→P ⚠️ stub | X |
| × | × | x>y? ⚠️ stub | Y |
| 1 | 1 | FIX | Z |
| 2 | 2 | SCI | = |
| 3 | 3 | ENG | ? |
| ÷ | ÷ | x=0? ⚠️ stub | : |
| 0 | 0 | π ⚠️ stub | SPACE |
| . | . | LAST X | , |
| R/S | R/S | VIEW ⚠️ stub | — |

⚠️ stub = label shown, click returns "planned for a future phase" toast (v2.2 backlog).

Compare-test ops (`x=y?`, `x≤y?`, `x>y?`, `x=0?`) are marked as stubs even though their core ops (`XEqY`, `XLeY`, `XGtY`, `XEqZero`) exist — because in the HP-41 they take a label argument for the conditional branch target, which requires a label-entry modal. To avoid scope creep, we defer the modal to v2.2 and stub them. The bare comparison ops remain reachable programmatically.

`FIX`, `SCI`, `ENG` also take a digit argument — they already resolve as `fix_N`/`sci_N`/`eng_N` parameterized keys. For shift-click access from the keyboard, a 1-digit modal will appear (re-use the entry-buf pattern from STO modal — a separate small plan).

## Color & Style

| Element | Color | Notes |
|---------|-------|-------|
| Body background | `#0d0d0d` → `#000000` (vertical gradient) | Black, replaces brown `#5a3828` → `#1e100a` |
| Top-row & main keys | `#303030` → `#080808` (current `grad-dark`) | Unchanged — already authentic |
| SHIFT key (active) | `#f5a423` → `#c97d10` | Solid orange, distinct from any other key |
| SHIFT key (idle) | `#d68a1c` → `#a35d0a` | Same hue, slightly darker |
| ENTER key | current `grad-enter` (green) | Unchanged |
| Shift labels (above) | `#d68a1c` | Slightly warmer than current `#d4a800` |
| Alpha letters (below) | `#5b8fb9` | HP-41 powder blue |
| Primary labels | `#e8e8e8` | Slight warm-off-white, unchanged |
| SHIFT active glow | `box-shadow: 0 0 6px #f5a423` | CSS on `.key-shift-active` |

## Component Architecture

### `hp41-gui/src/Keyboard.tsx` (rewrite)

- **Props change:** `onKey: (keyId: string) => void` stays. Add `shiftActive: boolean` (read-only) and `alphaActive: boolean` (read-only, sourced from `calcState.annunciators.alpha`). Both drive visual state only.
- **`KEY_DEFS`** rewritten per the layout table above.
- **Layout math:** new `KEY_W`, `KEY_H`, column count = 5, `viewBox` height grows from 230 to ~440 (8 rows × ~50px + top row + padding).
- **Three-label rendering:** `<text>` elements for shift label (top), primary (center), alpha (bottom).
- `handleKeyClick` invokes the parent `onKey` with the **raw** primary id; resolution to shifted/alpha happens in `App.tsx` (single source of truth for mode state).
- Special-case rendering for SHIFT key (no labels, solid orange fill, `.key-shift-active` class when `shiftActive`).
- Special-case rendering for top-row keys (no alpha letter, may have orange caption above for `ON`/`USER`/`PRGM`/`ALPHA` — these have no shift function themselves).

### `hp41-gui/src/App.tsx` (additive)

- New state: `const [shiftActive, setShiftActive] = useState(false);`
- New state: `const [toastMsg, setToastMsg] = useState<string | null>(null);`
- **Click handler refactor:** `handleClick(rawKey: KeyDef)` (signature changes — receives the full def, not just id) implements D-4 routing.
- **Toast:** when `dispatch_op` returns `GuiError`, set `toastMsg` for 2s. Render as a small overlay near the display area.
- **ESC handler:** physical Escape key cancels `shiftActive` (added to existing keydown handler).
- **Annunciator list extended:** `['user', 'shift', 'prgm', 'alpha', 'rad', 'grad']` — `shift` is driven by frontend `shiftActive`, the others by `calcState.annunciators`.
- `resolveKeyId(e: KeyboardEvent, …)` for physical keyboard: physical `Tab` (or another unused key) toggles SHIFT — chosen to avoid conflict with existing bindings (decision flagged for explicit confirmation in plan phase).

### `hp41-gui/src/App.css` (additive)

- Body color update.
- New classes: `.key-shift-active`, `.alpha-label`, `.shift-label`, `.shift-key`, `.top-row-button`, `.toast`.
- Toast positioning, fade-out animation.

### `hp41-gui/src-tauri/src/key_map.rs` (additive)

- Add named-op resolvers (some already present): `sq`, `ypow`, `tenpow`, `exp`, `int`. (`asin`/`acos`/`atan`/`sigma_minus`/`mean`/`sdev`/`lr`/`yhat`/`corr`/`cl_sigma_stat`/`hms_*`/`rtn` already mapped — confirm with test.)
- Add stub-error branch (D-5).
- Add unit tests for all new ids and the stub-error path.
- **No** Tauri command additions; the existing `dispatch_op` flow handles the error return via the existing `GuiError → frontend` channel.

### What does **not** change

- `hp41-core/**` — zero changes.
- `hp41-cli/**` — zero changes.
- `hp41-gui/src-tauri/src/types.rs`, `persistence.rs`, `prgm_display.rs` — zero changes.
- `hp41-gui/src-tauri/src/commands.rs`, `lib.rs` — **one addition possible** (a `run_stop` command + `generate_handler!` registration) pending Edge Case 3 resolution. No other changes.
- `CalcStateView` schema and the ≤300-byte JSON budget — unchanged.
- Auto-save file format and v1.x backward compatibility — unchanged.
- SC-4 invariant (no calculator logic in `hp41-gui`) — preserved.

## Edge Cases & Known Divergences

1. **SHIFT + ALPHA combined:** Real HP-41 permits some shifted ops in alpha mode (e.g., SHIFT-ENTER → CATALOG). In v2.1 we **ignore** SHIFT when ALPHA is active. Documented in CHANGELOG.
2. **`SST`/`BST` in PRGM mode — special Tauri commands:** Both invoke the existing dedicated commands `sst_step` and `bst_step`, **not** `dispatch_op`. The `App.tsx` click handler must route ids `sst` and `bst` to those commands. `BST` is now reachable via SHIFT+SST (was previously a bottom-row standalone key — that key disappears in the new layout).
3. **`R/S` (Run/Stop) routing:** Currently has `id: ''` in `KEY_DEFS` and is **unreachable from the GUI today** (only TUI/programmatic). After this refactor it becomes a primary key on row 8. R/S is not a single `Op` variant — it toggles the `running` state of a program. **Implementation route is open**: either add a new `run_stop` Tauri command (preferred — symmetric with `sst_step`/`bst_step`) or add `Op::RunStop` to core. Decided in plan phase; spec flags this as the one open question.
4. **`CL X/A` (SHIFT+←):** Behaves as `Clx` in non-alpha mode, `AlphaClear` in alpha mode. **No new `key_map.rs` resolver** — the `App.tsx` click handler reads `calcState.annunciators.alpha` at click time and dispatches either `clx` or `alpha_clear` (both already in `resolve()`).
5. **Stub-error toast spam:** If user mashes a stub key, toasts queue — implement single-toast policy (newest replaces older, no queue).
6. **Physical-keyboard SHIFT modifier:** Decided in plan phase. Default proposal: `Tab` toggles SHIFT (rarely used, no current binding); alternative is leaving SHIFT click-only.
7. **Snapshot regression risk:** Existing screenshot baselines (if any) will diff. New baselines must be captured as part of P4.

## Delivery — 4 Plans inside one phase

Phase id (proposed): **Phase 19 — Keyboard Authenticity** (under v2.1 milestone).

| Plan | Scope | Atomic? | Est. |
|------|-------|---------|------|
| **P19-1** | New `KEY_DEFS` (3-label model), 5-column SVG layout, black body, top-row separation. Existing key ids preserved where possible. | ✓ commits cleanly | ½ day |
| **P19-2** | `shiftActive` state in `App.tsx`, click resolution per D-4, annunciator extension, CSS active state, ESC handler, physical-keyboard SHIFT toggle. | ✓ commits cleanly | ½ day |
| **P19-3** | `key_map.rs`: missing named-op resolvers + stub-error branch + tests. Toast UI in `App.tsx` for `GuiError`. | ✓ commits cleanly | ¼ day |
| **P19-4** | E2E click-through every key in all three modes (default / shift / alpha). Re-capture any UI screenshot fixtures. Update `CLAUDE.md` with new layout invariants. | ✓ commits cleanly | ¼ day |

Total: ~1.5 dev days.

### v2.2 backlog (out of scope, listed for tracking)

- `Op::Pi`, `Op::PolarToRect`, `Op::RectToPolar`, `Op::Beep`, `Op::Asn`, `Op::Catalog`, `Op::View` in `hp41-core`.
- Flag system: `flags: u64` in `CalcState`, `Op::Sf(u8)`, `Op::Cf(u8)`, `Op::Fs(u8)` with 2-digit modal.
- Compare-with-label modal for `x=y?`/`x≤y?`/`x>y?`/`x=0?` shift access.
- 1-digit modal for FIX/SCI/ENG shift access.
- SHIFT+ALPHA combined-mode handling.

## Testing Strategy

- **Unit (Rust):** `key_map.rs` tests cover every new resolver id and every stub-error id. Existing `test_all_keyboard_skin_ids_are_valid` test extended to include the new id set.
- **Component (TypeScript):** Snapshot test of `Keyboard.tsx` rendering in three modes (none/shift/alpha). Mock `onKey` and assert correct id is forwarded for each click priority case.
- **E2E (manual P19-4):** Click every key in every mode, verify display updates and toast appears for stubs. Compare against reference image checklist.
- **Regression:** Existing `cargo test --workspace` and `cargo test -p hp41-gui` (or equivalent) suites must remain green. `just gui-ci` must pass.

## Success Criteria

- Visual: side-by-side with reference image, layout, color, and label placement match within reasonable rendering tolerance.
- Functional: every previously-unreachable but core-implemented op is now clickable via SHIFT.
- Honest: every stub key returns a visible toast within 200ms; no silent discards.
- Clean: zero changes outside `hp41-gui/`, zero impact on CLI/core tests, IPC budget intact.
- Documented: `CLAUDE.md` v2.1 section updated with new layout invariants, SHIFT mechanic, stub-error pattern.
