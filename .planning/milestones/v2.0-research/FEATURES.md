# Feature Landscape: HP-41 Calculator Emulator v2.0 GUI

**Domain:** Pixel-perfect retro scientific calculator desktop GUI (Tauri v2 + React + SVG)
**Researched:** 2026-05-09
**Scope:** v2.0 — GUI-specific features only; all hp41-core functionality is pre-existing
**Confidence:** HIGH for HP-41C hardware facts; MEDIUM for Tauri/SVG rendering patterns

---

## Summary

The v2.0 milestone adds `hp41-gui` as a Tauri v2 desktop binary to the existing Cargo
workspace. The entire calculator engine lives unchanged in `hp41-core`; the GUI is a thin
Tauri adapter exposing `hp41-core` ops through Tauri commands. The critical design question
is display fidelity: a pixel-perfect HP-41C replica means reproducing the exact physical
key layout (9 rows × 5 columns = 40 addressable keys), the 12-character 14-segment LCD,
the five display annunciators, and the two-layer shift system (gold shifted / ALPHA).

The industry standard for emulator GUI skins (established by Free42 and i41CX+) uses a
static skin bitmap (PNG or GIF) for the visual faceplate combined with a separate data
structure defining hit rectangles per key. For a new-build Tauri app, SVG is a better
choice than a raster bitmap: it scales perfectly to any window size or DPI, can be themed
with CSS, and allows React to treat individual key elements as first-class components with
click handlers and animation states.

---

## HP-41C Key Layout (Authoritative Reference)

The HP-41C has **9 rows of keys, 5 columns each**, giving 45 key positions total.
Five positions are occupied by the top-mode row (which has 4 keys, not 5, plus the ON key
at a different location), so the addressable programmable key count is **40** (keys 11–95
in row-column notation, where the tens digit is the row and the units digit is the column).

### Physical Row-by-Row Layout (top to bottom)

**Special / Mode Row (above the main grid)**
Keys: ON (power, not programmable), USER (toggle USER mode), PRGM (toggle PRGM mode),
ALPHA (toggle ALPHA keyboard)

**Row 1 — Scientific functions**
| Col 1 | Col 2 | Col 3 | Col 4 | Col 5 |
|-------|-------|-------|-------|-------|
| Sigma+ | 1/X | sqrt(X) | LOG | LN |
| *(gold: Sigma-)* | *(gold: Y^X)* | *(gold: X^2)* | *(gold: 10^X)* | *(gold: E^X)* |

**Row 2 — Stack and trig**
| Col 1 | Col 2 | Col 3 | Col 4 | Col 5 |
|-------|-------|-------|-------|-------|
| X<>Y | Rv | SIN | COS | TAN |
| *(gold: LAST X)* | *(gold: R^)* | *(gold: ASIN)* | *(gold: ACOS)* | *(gold: ATAN)* |

**Row 3 — Storage and XEQ**
| Col 1 | Col 2 | Col 3 | Col 4 | Col 5 |
|-------|-------|-------|-------|-------|
| (blank/user) | XEQ | STO | RCL | SST |
| | *(gold: GTO)* | *(gold: COMPLEX)* | *(gold: CATALOG)* | *(gold: BST)* |

**Row 4 — ENTER, CHS, EEX, backspace**
| Col 1 | Col 2 | Col 3 | Col 4 | Col 5 |
|-------|-------|-------|-------|-------|
| ENTER^ | CHS | EEX | <- (backspace) | *(no col 5 — ENTER is wide)* |
| *(gold: -)* | *(gold: PAUSE)* | *(gold: SHOW)* | *(gold: CLX)* | |

Note: ENTER is a double-width key spanning columns 1–2; the row effectively has 4 keys.

**Row 5 — Arithmetic operators, minus**
| Col 1 | Col 2 | Col 3 | Col 4 | Col 5 |
|-------|-------|-------|-------|-------|
| - (subtract) | 7 | 8 | 9 | / (divide) |
| *(gold: SET ALPHA)* | *(gold: FIX)* | *(gold: SCI)* | *(gold: ENG)* | *(gold: X!=Y?)* |

**Row 6 — Arithmetic operators, plus**
| Col 1 | Col 2 | Col 3 | Col 4 | Col 5 |
|-------|-------|-------|-------|-------|
| + (add) | 4 | 5 | 6 | * (multiply) |
| *(gold: ...)* | *(gold: VIEW)* | *(gold: DSE)* | *(gold: ISG)* | *(gold: X=Y?)* |

**Row 7**
| Col 1 | Col 2 | Col 3 | Col 4 | Col 5 |
|-------|-------|-------|-------|-------|
| (user) | 1 | 2 | 3 | (user) |

**Row 8**
| Col 1 | Col 2 | Col 3 | Col 4 | Col 5 |
|-------|-------|-------|-------|-------|
| (user) | 0 | . (decimal) | R/S | (user) |

Row 7 and Row 8 col 1 and col 5 are soft user-assignable keys labeled A/B/C/D/E (top row
of soft keys) and a/b/c/d/e (when accessed via shift). On the HP-41CX they also give access
to extra functions.

### Key Code Numbering

The HP-41 programs key presses as two-digit codes: tens digit = row (1–8), units digit =
column (1–5). GETKEY returns these codes; synthetic programming uses them directly.
- Key 11 = Row 1, Col 1 = Sigma+
- Key 15 = Row 1, Col 5 = LN
- Key 21 = Row 2, Col 1 = X<>Y
- Key 31 = Row 3, Col 1 (user/blank top-left)
- Key 51 = Row 5, Col 1 = Subtract
- etc.

### Display Region

Above the key rows, the display is a single-row LCD region spanning the full calculator
width. It shows:
- 12 character positions (14-segment alphanumeric, HP FOCAL character set)
- Annunciators: **BAT** (battery low), **USER** (user mode), **PRGM** (program mode),
  **ALPHA** (alpha mode), plus an **angle mode indicator** (DEG/RAD/GRAD)

The 14-segment display allows uppercase A-Z, digits 0-9, and limited punctuation.
Lowercase a-e are approximated (used for soft key labels). Some characters are ambiguous
(S/5 distinction is a recognized HP-41 quirk).

### Physical Appearance

- Case: Dark warm brown/chocolate body, beige/tan key legends
- Key bodies: Dark brown with gold-colored upper legends (shifted functions printed above
  each key in gold/yellow) and white lower legends (ALPHA characters on the slanted face)
- Display bezel: Charcoal/dark grey surround
- Dimensions: 14.6 × 7.3 × 3.3 cm (portrait orientation, taller than wide)
- Module ports: 4 ports on the top edge (HP-41CV/CX only; HP-41C has 0)

---

## Table Stakes

Features that users expect from a pixel-perfect HP-41C GUI. Missing any of these and the
product fails the "faithful replica" test.

| Feature | Why Expected | Complexity | hp41-core Dependency |
|---------|--------------|------------|---------------------|
| SVG key layout — all 9 rows x 5 cols, correct proportions | Without correct layout the product isn't a replica | High | None (pure SVG/CSS) |
| Clickable keys trigger correct Op dispatch | Core purpose of a GUI calculator | Medium | All Op variants via Tauri commands |
| 12-char 14-segment display renders current CalcState.display | Display is the primary output; must update after every op | Medium | `CalcState.display_string()` |
| Annunciators: BAT / USER / PRGM / ALPHA visible in display region | HP-41C users depend on mode indicators constantly | Low-Medium | `CalcState` flags |
| Gold shift key state highlights shifted labels on keys | SHIFT is a two-keystroke prefix — visual state is mandatory | Medium | `CalcState.shift_active` or equivalent |
| ALPHA mode indicator and key legend change | ALPHA completely changes key meanings; must be visible | Medium | `CalcState.alpha_mode` |
| Key press visual feedback (active/pressed state) | Without feedback the UI feels broken; users can't confirm input | Low-Medium | None (CSS :active or React state) |
| Static faceplate: correct HP-41C color scheme (dark brown, gold legends) | Aesthetic fidelity is the product promise | Medium | None (SVG fill/CSS) |
| Correct aspect ratio — calculator is taller than wide | Wrong proportions instantly betray a non-authentic replica | Low | None (SVG viewBox) |
| All hp41-core ops accessible via GUI keys (no missing ops) | Functional parity with CLI required | High | Full Op enum mapping |

### On Display Rendering

The 14-segment LCD character set should render using a custom SVG segment font or an
existing 14-segment SVG/CSS font library. A monospace system font is not acceptable for
table-stakes fidelity — the segment-based rendering is a recognizable visual signature.
A CSS-based 14-segment font (e.g., referencing the `ctrlcctrlv/lcd-font` or similar) or
hand-drawn SVG segment paths per character are both viable. The simpler approach is a
custom `<canvas>` or SVG per character cell with 14 individually toggled segments.

### On Tauri Command Architecture

Each key press in the React frontend must call a Tauri command such as:
```
invoke("dispatch_op", { op: "Sin" })  // or a serialized Op variant
```
The Tauri Rust side holds a `Mutex<CalcState>` managed state, calls
`hp41_core::dispatch()`, and returns the updated display string and annunciator flags.
The React component tree re-renders from that response. This is a one-call-per-keypress
round-trip; no streaming or event bus is needed for table-stakes.

---

## Differentiators

Features that set this GUI apart from generic emulators. Not expected, but valued by
serious HP-41 users.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Key press animation (scale-down + bounce) | Tactile satisfaction; distinguishes quality from cheap clone | Low | CSS transform: scale() on :active; 60 fps, no repaints |
| PRGM mode step display — show current program step in display area | HP-41C in PRGM mode shows step number + op name; GUI should replicate | Medium | `CalcState.program_counter`, `CalcState.program` |
| Stack register panel (X/Y/Z/T/LASTX) always visible alongside display | Power users want to see the full stack; CLI already does this | Medium | `CalcState` stack fields via periodic poll or per-op update |
| Print buffer / thermal printer panel | hp41-core already has `print_buffer`; surfacing it as a scrolling panel is high value | Medium | `CalcState.print_buffer` via `drain_print_buffer` Tauri command |
| SST/BST (single-step) navigation visible in PRGM mode | Stepping through programs is a core debugging workflow | Medium | `Op::SST`, `Op::BST` already in hp41-core |
| Keyboard shortcut overlay (same as CLI '?' help) | Preserves discoverability from CLI | Low | Port `HELP_DATA` from hp41-cli to a modal in React |
| Module port visual (4 ports visible on top edge for HP-41CX skin) | Completeness for CX variant; sets up future module emulation | Low | Visual only for v2.0 |
| Window resize / HiDPI support without blurring | SVG scales perfectly; critical for macOS Retina | Low | SVG viewBox + CSS width: 100% |
| Keyboard input still works in GUI (same keys as CLI) | Power users want to keep fingers on keyboard | Medium | Tauri window keyboard events → same key_to_op() mapping |

### On Stack Panel

The existing TUI panel (T/Z/Y/X/LASTX) is the model. In the GUI it should appear as a
secondary panel to the right of or below the calculator faceplate, always visible. It
requires a separate Tauri command `get_stack_state()` called after every op dispatch, or
the `dispatch_op` command can return a full state snapshot. Returning a snapshot is simpler
and more reliable.

### On Keyboard Input

`hp41-cli/src/keys.rs` contains `key_to_op()` and `KEY_REF_TABLE`. The same mapping logic
should be reproduced in the Tauri Rust layer (or shared as a method in hp41-core) so that
keyboard events reaching the Tauri window are dispatched identically to mouse clicks on
SVG keys. This is pure code reuse, not a new feature.

---

## Anti-Features

Features to explicitly NOT build in v2.0.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Cycle-accurate Nut CPU emulation | Enormous effort; zero perceptible user value for behavioral emulation | Keep behavioral dispatch model from hp41-core |
| Multiple theme / skin system in v2.0 | Scope creep; postponed in PROJECT.md | Ship one pixel-perfect HP-41C skin; theme support in v2.1+ |
| HP-IL peripheral emulation | Requires hardware simulation infrastructure not in hp41-core | Exclude permanently unless community demand |
| Cloud sync or telemetry | Privacy risk; no infrastructure; PROJECT.md explicit exclusion | Local JSON persistence (already shipping) |
| Animated display cursor blinking | Real HP-41 has no blinking cursor; adds authenticity but is a distraction and adds JS timer complexity | Static display update on keypress |
| Virtual keyboard / on-screen numeric keypad in browser | This is a desktop app, not web; keyboard input exists | Use physical keyboard or SVG click |
| Electron alternative | No reason to replace Tauri; Tauri already decided | Stay on Tauri v2 |
| Async dispatch or web worker for hp41-core | hp41-core is single-threaded and sub-microsecond per op; async adds complexity for zero benefit | Synchronous Tauri command invocation |
| Drag-to-rearrange key labels | Over-engineering UX; no precedent in HP emulators | Fixed layout |
| Touch / multi-touch gesture support | Desktop primary target; mobile is explicitly deferred | Defer to v3.0 if at all |

---

## Feature Dependencies

```
SVG key layout
  └─> Clickable keys (hit rects on SVG elements)
        └─> Tauri command: dispatch_op(op: String)
              └─> hp41-core: dispatch() on CalcState (EXISTING)
                    └─> Tauri command response: display_string + annunciators
                          └─> React: display re-render
                                └─> Optional: stack panel re-render

CalcState (EXISTING)
  └─> display_string (EXISTING)
  └─> shift_active state (EXISTING via PendingInput or shift flag)
  └─> alpha_mode (EXISTING)
  └─> print_buffer (EXISTING — Phase 11)
  └─> program_counter (EXISTING — Phase 3)

Key press animation
  └─> CSS only — no hp41-core dependency

Keyboard input in GUI
  └─> key_to_op() mapping (EXISTING in hp41-cli, needs porting or sharing)

Stack panel
  └─> dispatch_op returns CalcState snapshot, or separate get_stack() command
```

---

## MVP Recommendation

The minimal GUI that demonstrates HP-41 fidelity:

1. **SVG faceplate** with correct 9×5 key layout, HP-41C color scheme, key labels (primary + gold shift printed above)
2. **12-char display** rendering CalcState.display_string() after every op — a simple monospace font is acceptable for MVP if 14-segment font is not ready
3. **Annunciators** USER/PRGM/ALPHA as small indicator lights next to the display
4. **Clickable keys** — all 40 keys invoke dispatch_op Tauri command
5. **Keyboard input** — same physical keyboard bindings as CLI
6. **Key press CSS animation** — scale-down on mousedown, release on mouseup (2 lines of CSS)

Defer for v2.1:
- 14-segment font (replace monospace MVP display with authentic segment rendering)
- Stack panel (valuable but not blocking)
- Print buffer panel
- PRGM mode step display
- Help overlay / keyboard shortcut modal

The MVP above is buildable in a single phase. The deferred items form a natural second phase
("GUI polish") after the structural foundation is proven.

---

## Complexity Notes

| Feature | Estimate | Risk | Notes |
|---------|----------|------|-------|
| SVG faceplate construction | 3–5 days | Medium | Main effort is measuring and placing 40 key rects at accurate proportions; can be done by hand or extracted from a reference image |
| Tauri command layer | 1–2 days | Low | `invoke()` boilerplate; CalcState behind Mutex; standard Tauri pattern |
| Display rendering (monospace MVP) | 0.5 days | Low | Single `<pre>` or `<span>` updated from Tauri response |
| Display rendering (14-segment authentic) | 2–4 days | Medium | Either find an SVG 14-segment font library or implement segment paths; character set is FOCAL subset (36 chars) |
| Key click animation | 0.5 days | Low | Pure CSS, no state required |
| Annunciators | 0.5 days | Low | 4–5 boolean indicators from CalcState |
| Keyboard input routing | 1 day | Low-Medium | Port key_to_op() to Tauri; need to handle shift-state carefully |
| Stack panel | 1 day | Low | JSON response from Tauri already has all values |
| Print buffer panel | 1 day | Low | Drain command already exists in hp41-core |
| Shift state highlighting | 1 day | Medium | CSS class on SVG key elements; React state tracks shift |

**Highest risk item:** SVG key layout proportions. If the key positions are visually wrong,
the faceplate looks wrong regardless of functional correctness. Recommend using an official
HP-41C photo as an SVG underlay reference during design, then removing it from production.

**Second highest risk:** ENTER key is double-width (spans cols 1–2 in row 4). SVG hit
rectangle must be wider than a standard key. This is a layout edge case that needs explicit
handling in the SVG definition.

**Pre-existing in hp41-core that the GUI gets for free:**
- All arithmetic, trig, stat, programming ops — fully tested
- ALPHA mode state and buffer
- Shift/pending-input state machine
- Print buffer
- ISG/DSE counters
- Synthetic programming (GETKEY, NULL, M/N/O registers)
- JSON persistence (load/save CalcState)

---

## Sources

- HP-41C Wikipedia article: https://en.wikipedia.org/wiki/HP-41C (display specs, 14-segment, one shift key)
- HP-41C Owner's Handbook (archived): https://archived.hpcalc.org/greendyk/hp41c-manual/ (key layout, annunciators)
- finseth.com HP-41C data page: https://www.finseth.com/hpdata/hp41c.php (key grid structure, approx 5 cols x 8 rows)
- Free42 skin README: https://thomasokken.com/free42/skins/README.html (hit rects, display rect, skin bitmap pattern)
- i41CX+ site: https://alsoftiphone.com/i41CXplus/ (43 overlays, key glow, click sound, skin approach)
- Tauri v2 state management: https://v2.tauri.app/develop/state-management/ (Mutex<T> managed state pattern)
- Tauri v2 calling Rust from frontend: https://v2.tauri.app/develop/calling-rust/ (invoke() pattern)
- ctrlcctrlv/lcd-font (14-segment font): https://github.com/ctrlcctrlv/lcd-font
- HP-41C physical dimensions: National Air and Space Museum collection record, 14.6 × 7.3 × 3.3 cm
