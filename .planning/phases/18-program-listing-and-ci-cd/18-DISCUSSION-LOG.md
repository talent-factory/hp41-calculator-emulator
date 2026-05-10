# Phase 18: Program Listing & CI/CD - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-10
**Phase:** 18-program-listing-and-ci-cd
**Areas discussed:** Program data in CalcStateView, SST/BST dispatch mechanism, Program listing panel UI, CI scope for Tauri

---

## Program Data in CalcStateView

### Q1: How to transport program listing data to React?

| Option | Description | Selected |
|--------|-------------|----------|
| Add conditional fields, relax the size gate | Add program_steps + pc unconditionally, update 300-byte test scope | ✓ |
| Separate Tauri command get_program | React calls get_program() on prgm_mode change | |
| Conditional payload — only when prgm_mode is true | Empty Vec/0 when not in PRGM mode | |

**User's choice:** Add fields unconditionally, relax size gate  
**Notes:** Follows established CalcStateView DTO enrichment pattern from Phases 14–17. The 300-byte size test was scoped to empty-program state anyway.

### Q2: Where does op formatting logic live?

| Option | Description | Selected |
|--------|-------------|----------|
| Copy prgm_display.rs to hp41-gui | Same copy-to-GUI pattern as persistence.rs (Phase 17) | ✓ |
| Move op_display_name() to hp41-core | Expose as public function; CLI and GUI share it | |

**User's choice:** Copy prgm_display.rs to hp41-gui  
**Notes:** Avoids adding public surface to hp41-core. Consistent with Phase 17 D-05.

---

## SST/BST Dispatch Mechanism

### Q1: How should SST/BST work in the GUI?

| Option | Description | Selected |
|--------|-------------|----------|
| New Tauri commands sst_step / bst_step | Mirrors CLI F7/F8 special-case, not routed through Op dispatch | ✓ |
| Special-cased inside handle_op in commands.rs | Match 'sst'/'bst' before Op resolver; fewer command registrations | |

**User's choice:** New Tauri commands sst_step / bst_step  
**Notes:** Clean separation; testable via pure-Rust helpers following existing pattern.

### Q2: Physical keyboard bindings for SST/BST?

| Option | Description | Selected |
|--------|-------------|----------|
| SVG click only — no keyboard shortcut | Simpler App.tsx; faithful to HP-41 (no F-keys on real hardware) | |
| Wire F7/F8 as keyboard shortcuts too | Matches CLI D-15 behavior; power-user friendly | ✓ |

**User's choice:** Wire F7/F8 as keyboard shortcuts  
**Notes:** Consistent with hp41-cli pattern where F7=SST, F8=BST.

---

## Program Listing Panel UI

### Q1: Where does the listing appear?

| Option | Description | Selected |
|--------|-------------|----------|
| Panel below keyboard, auto-show on prgm mode | Mirrors print panel from Phase 17 | ✓ |
| Replace stack panel content in PRGM mode | Compact, no window size change needed | |
| Full-window overlay modal | Max screen real estate but disrupts aesthetic | |

**User's choice:** Panel below keyboard  
**Notes:** Consistent with print panel established in Phase 17.

### Q2: Window height handling?

| Option | Description | Selected |
|--------|-------------|----------|
| Grow window height to 900 | Clean layout; everything visible at once in PRGM mode | ✓ |
| Keep 700px, panel overlaps keyboard | No config change but obscures keyboard | |
| Keep 700px, calculator container scrolls | Preserves window size but awkward UX | |

**User's choice:** Grow to 400×900  
**Notes:** tauri.conf.json windows[0].height updated from 700 → 900.

---

## CI Scope for Tauri

### Q1: What does the Tauri CI job build?

| Option | Description | Selected |
|--------|-------------|----------|
| cargo build --release of hp41-gui/src-tauri + tsc --noEmit | ~5 min per platform; satisfies SC-4 | ✓ |
| Full tauri build on all 3 platforms | ~15 min each; produces artifacts; overkill for CI | |
| cargo check only | ~1 min but doesn't catch linker errors | |

**User's choice:** cargo build --release + tsc --noEmit  
**Notes:** Satisfies "build completes without error" without the overhead of full Tauri bundling.

### Q2: Separate workflow file or extend existing ci.yml?

| Option | Description | Selected |
|--------|-------------|----------|
| New file .github/workflows/ci-gui.yml | Clean isolation; SC-5 (GUI failure ≠ CLI CI block) | ✓ |
| Add gui job to existing ci.yml | GitHub path filters work at workflow level; harder to isolate | |

**User's choice:** New file ci-gui.yml  
**Notes:** SC-5 requires independence between CLI CI and GUI CI.

---

## Claude's Discretion

- Exact CSS for program listing panel (font, row height, highlight color, shading)
- Step row alternating background
- Panel header wording (`"PRGM"` vs `"PRGM MODE"` vs `"PRGM — N steps"`)
- Step number zero-padding format — follow `format_step()` behavior (`{:03}`)

## Deferred Ideas

None — discussion stayed within phase scope.
