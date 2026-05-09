# Requirements: HP-41 Calculator Emulator v2.0

**Defined:** 2026-05-09
**Core Value:** Faithful HP-41 RPN fidelity — the four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to the original hardware; everything else is secondary.

## v2.0 Requirements

### WORKSPACE — Workspace & Infrastructure

- [ ] **WSPC-01**: User can build and launch `hp41-gui` from the Cargo workspace via `just gui-dev`; `just ci` (CLI pipeline) still passes without modification
- [ ] **WSPC-02**: User can build both `hp41-cli` and `hp41-gui` in the same workspace without either binary's CI regressing

### SKIN — HP-41C Visual Skin

- [ ] **SKIN-01**: User sees a pixel-perfect SVG HP-41C key layout (9×5 grid, ENTER double-width, correct key labels and legends, HP-41C proportions and color scheme — dark brown body, gold shift labels)
- [ ] **SKIN-02**: User can click any key in the SVG skin and the corresponding HP-41 operation executes in `hp41-core` (same result as pressing the equivalent CLI key binding)
- [ ] **SKIN-03**: User sees visual press feedback (CSS scale-down animation) on every key click

### DISP — Display & Readouts

- [ ] **DISP-01**: User sees the 12-char HP-41 display string and all five annunciators (USER, PRGM, ALPHA, RAD, GRAD) update in the GUI after every operation
- [ ] **DISP-02**: User sees a stack register panel showing X/Y/Z/T/LASTX values alongside the calculator skin, updating after every operation

### IPC — IPC & Core Integration

- [ ] **IPC-01**: All user operations reach `hp41-core` via Tauri Rust commands (`dispatch_op`, `get_state`); the response is a `CalcStateView` (~200 bytes); `print_buffer` is drained on every command; no `hp41-core` logic is duplicated in the GUI crate
- [ ] **IPC-02**: User can operate the GUI entirely from the physical keyboard using the same key bindings as `hp41-cli` (e.g. `0–9`, `+`, `-`, `*`, `/`, `ENTER`, `q`→SIN, etc.)

### PERS — Persistence

- [ ] **PERS-01**: User's calculator state persists across GUI restarts via `~/.hp41/autosave.json` (shared with `hp41-cli`); state auto-saves every 30 seconds; save files created by v1.x `hp41-cli` load without error in `hp41-gui`
- [ ] **PERS-02**: User sees PRX/PRA/PRSTK print output in a scrollable panel in the GUI (output from the `print_buffer` is surfaced, not silently discarded)

### PROG — Program Display

- [ ] **PROG-01**: User can view the current program listing and navigate steps with SST/BST in GUI PRGM mode

## Future Requirements

### v2.1 Polish

- **SKIN-04**: 14-segment SVG font for the HP-41 display (authentic LCD rendering) — medium complexity; deferred until core GUI is stable
- **SKIN-05**: Keyboard shortcut overlay (port `?` help panel from CLI) — low complexity
- **PROG-02**: Full keyboard assignment display in USER mode — medium complexity

### v2.x / v3.0

- **PRNT-03**: `ADV`, `PRREG`, FLAG 26 / TRACE mode printer peripheral ops
- **WSPC-03**: STO arithmetic via indirect addressing (`STO+ IND NN`) — v1.2+ in CLI, then GUI
- **MOD-01**: HP-41 module emulation (Math/Stat/Time/Advantage) — v1.2+ or v3.0

## Out of Scope

| Feature | Reason |
|---------|--------|
| Cycle-accurate Nut CPU simulation | Architecture decision: behavioral emulation; high effort, low user value |
| HP-copyrighted ROM image redistribution | Legal risk; excluded permanently |
| HP-IL / peripheral bus emulation | Niche, high complexity |
| Cloud sync or telemetry | Privacy; local-only data storage |
| Mobile (iOS/Android) | Deferred until desktop GUI is stable |
| Multiple skin themes | v2.x after core GUI is solid |
| `.raw` HP-41 program file import/export | v1.2+ CLI feature before GUI integration |
| `println!` / direct I/O in `hp41-core` | Enforced invariant: zero UI dependencies in core library |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| WSPC-01     | Phase 13 | Pending |
| WSPC-02     | Phase 13 | Pending |
| SKIN-01     | Phase 16 | Pending |
| SKIN-02     | Phase 16 | Pending |
| SKIN-03     | Phase 16 | Pending |
| DISP-01     | Phase 15 | Pending |
| DISP-02     | Phase 15 | Pending |
| IPC-01      | Phase 14 | Pending |
| IPC-02      | Phase 15 | Pending |
| PERS-01     | Phase 17 | Pending |
| PERS-02     | Phase 17 | Pending |
| PROG-01     | Phase 18 | Pending |

**Coverage:**
- v2.0 requirements: 12 total
- Mapped to phases: 12
- Unmapped: 0

---
*Requirements defined: 2026-05-09*
*Last updated: 2026-05-09 — traceability assigned after v2.0 roadmap creation*
