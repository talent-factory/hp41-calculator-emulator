# Milestones

## v1.0 — HP-41 Calculator Emulator CLI

**Status:** ✅ SHIPPED 2026-05-08
**Phases:** 8 (Phases 1–8)
**Plans:** 45 total, all complete
**Timeline:** 3 days (2026-05-06 → 2026-05-08)
**Source:** 13,399 lines Rust | 212 files | 68 feat+fix commits

### Delivered

A faithful Rust-based behavioral emulation of the HP-41C/CV/CX programmable RPN calculator — delivered as a keyboard-driven TUI CLI (`hp41-cli`) backed by a UI-agnostic library crate (`hp41-core`).

### Key Accomplishments

1. **4-level RPN stack with full HP-41 stack-lift semantics** — every one of ~130 operations correctly declares Enable/Disable/Neutral; `#![deny(clippy::unwrap_used)]` enforces zero panics at compile time
2. **Complete HP-41 math engine** — arithmetic, trig (DEG/RAD/GRAD), `FIX`/`SCI`/`ENG` formatting, R00–R99 registers, ALPHA mode; 10-digit `rust_decimal` accuracy with mantissa carry fix
3. **Full keystroke programming engine** — `LBL`/`GTO`/`XEQ`/`RTN`, all 12 conditional tests, `ISG`/`DSE` with CCCCC.FFFDD string-split counter (never float arithmetic)
4. **ratatui TUI** with persistent 4-level stack display, 12-char HP-41 alphanumeric display, annunciators, and complete physical keyboard mapping
5. **JSON persistence** — auto-save every 30s, exit save, `USER` mode with custom key assignments, 10 bundled sample programs
6. **Science & Engineering** — Σ+/−, MEAN, SDEV, L.R. (linear regression), HMS↔H conversions
7. **Hardened quality gates** — 2.2ms cold-start (228× under 500ms gate), 94.87% test coverage, 495/500 numerical accuracy (99%), CI green on Windows/macOS/Ubuntu
8. **Tech Debt Cleanup** — EEX scientific notation entry (`from_scientific` fallback), SIN on `'q'` key, CLREG on `'g'` key, `Delete` → AlphaClear in ALPHA mode, help overlay accuracy

### Quality at Ship

| Gate | Target | Achieved |
|------|--------|---------|
| Cold-start latency | ≤ 0.5 s | 2.2 ms |
| Key-press latency | ≤ 50 ms | ~65 ns/op |
| hp41-core coverage | ≥ 80% | 94.87% |
| Numerical accuracy | ≥ 98% (500 cases) | 99% (495/500) |
| Panics in hp41-core | 0 | 0 |
| CI platforms | Win/macOS/Ubuntu | ✅ all green |

### Archives

- [ROADMAP.md](v1.0-ROADMAP.md)
- [REQUIREMENTS.md](v1.0-REQUIREMENTS.md)
- [Milestone Audit](v1.0-MILESTONE-AUDIT.md)

### Known Deferred Items

- EEX trailing-e-without-exponent discards number silently (documented with test)
- STO arithmetic keyboard modals (`STO+/-/×/÷`) keyboard-accessible via programs; interactive modal deferred to v1.1
- Tauri v2 GUI (hp41-gui crate) — deferred to v2.0

---

## v1.1 — HP-41 Calculator Emulator CLI Feature Completeness

**Status:** ✅ SHIPPED 2026-05-09
**Phases:** 4 (Phases 9–12)
**Plans:** 14 total, all complete

### Delivered

- **Phase 9:** MSRV 1.85, rust_decimal 1.42, EEX trailing-e hardware-faithful fix, exponent placeholder in TUI
- **Phase 10:** STO arithmetic modals (S → op → register), stack register support (Y/Z/T/LASTX), Esc cancellation
- **Phase 11:** PRX/PRA/PRSTK print emulation via `print_buffer` on CalcState, `--print-log` file output
- **Phase 12:** GETKEY, NULL, hidden registers M/N/O, 2-digit HexModal (23-entry safe subset)
- **Bugfixes found in review:** Vec::insert panic after ISG/DSE skip-at-end (CR-01); F5 overwriting last_key_code before GETKEY (F5 → 0 in keycode_to_hp41_code)

### Quality at Ship

| Gate | v1.0 | v1.1 |
|------|------|------|
| hp41-cli tests | 86 | 99 |
| hp41-core tests | 150 | 150+ |
| Synthetic tests | — | 21 |
| All requirements | 15/15 complete | ✅ |

### Archives

- [ROADMAP.md](milestones/v1.1-ROADMAP.md)
- [REQUIREMENTS.md](milestones/v1.1-REQUIREMENTS.md)

### Known Deferred Items

- SYNT-05: Full FOCAL byte-code table (~200 codes)
- SYNT-06: GETKEY interrupt-style capture (requires event loop redesign)
- PRNT-05/06: Scrollable print history, ADV/PRREG/TRACE
- STOA-04: STO arithmetic via indirect addressing
- Tauri v2 GUI (hp41-gui crate) — shipped in v2.0

---

## v2.0 — HP-41 Calculator Emulator Tauri GUI

**Status:** ✅ SHIPPED 2026-05-10
**Phases:** 6 (Phases 13–18)
**Plans:** 19 total, all complete
**Timeline:** 2 days (2026-05-09 → 2026-05-10)
**Source:** 183 files changed | 30,358 insertions

### Delivered

A pixel-perfect HP-41C desktop application built with Tauri v2 + React + TypeScript, reusing `hp41-core` unchanged alongside the existing `hp41-cli`.

### Key Accomplishments

1. **Tauri v2 workspace skeleton** — `hp41-gui` nested standalone workspace isolated from CLI Cargo graph; `just gui-dev` launches HP-41 Calculator window; `just ci` stays green; bundle identifier `ch.talent-factory.hp41`
2. **IPC Layer** — `dispatch_op`/`get_state` Tauri v2 commands; `CalcStateView` (~170 bytes, ≤300 limit); `key_map::resolve()` for 50+ named ops + 7 prefix families; Tauri v2.11 permission TOML pattern; `print_buffer` drained on every command
3. **Display & Keyboard** — React `App.tsx` with 12-char HP-41 display, 5 annunciators, X/Y/Z/T/LASTX stack panel; `useCallback`+`useEffect` keyboard listener with `busyRef` debounce; `eex_chs` branch; all hp41-cli bindings covered
4. **SVG Skin** — Pixel-perfect HP-41C 44-key SVG layout (9+8+9+9+9 rows, ENTER double-width); authentic HP-41C color scheme; CSS `scale(0.92)` press animation with `transform-box: fill-box`; Tauri window 400×700
5. **Persistence & Print Output** — Shared `~/.hp41/autosave.json` auto-save thread (30s); v1.x CLI save files load without error; scrollable print panel with auto-show, history accumulation, auto-scroll
6. **Program Listing & CI/CD** — PRGM-mode program listing panel with SST/BST navigation, F7/F8 bindings, `activeStepRef` auto-scroll; cross-platform `ci-gui.yml` (3-OS matrix, path filter, `cargo test` before build, independent from `ci.yml`)

### Archives

- [ROADMAP.md](milestones/v2.0-ROADMAP.md)
- [REQUIREMENTS.md](milestones/v2.0-REQUIREMENTS.md)

### Known Deferred Items (v2.1)

- SKIN-04: 14-segment SVG font for authentic LCD rendering
- SKIN-05: Keyboard shortcut overlay (port `?` help panel from CLI)
- PROG-02: Full keyboard assignment display in USER mode
- `prgm_mode` binding for 'p' key (currently mapped to `prx`)

---

## v2.1 — Card Reader + Keyboard Authenticity

**Status:** ✅ SHIPPED 2026-05-13
**Recorded as:** two quick-task entries in STATE.md (no Phase 19 GSD directory; scope evolved out-of-band from the original "v2.1 Polish" plan)
**Commits:** 50 commits since `v2.0` tag (range `72530dc…ff56b97`)
**Pull requests:** #9 (Card Reader), #10 (Keyboard Authenticity)

### Delivered

Two coherent feature areas shipped under the v2.1 banner without the formal GSD discuss/plan/execute pipeline. Both areas landed via PRs against `develop` with code-review feedback rounds.

**1. Card Reader (PR #9)**
- New `Op` variants `Wdta`, `Rdta`, `Wprgm`, `Rdprgm` in `hp41-core/src/ops/mod.rs`; each stages a `CardOpRequest` for the frontend to drain
- `builtin_card_op()` XEQ-by-name resolver wired into `op_xeq`, `run_program`, and `run_loop`
- `cards` modules mirrored in `hp41-cli` and `hp41-gui/src-tauri`: directory resolution, name sanitization (dot-prefix rejection), SHA-256 round-trip integration tests
- `pending_card_op` drain wired into all dispatch sites (cli `app.rs`, gui `handle_op` + `handle_get_state`)
- Comfort shortcuts in CLI: `Ctrl+W` / `Ctrl+R` / `Ctrl+D` / `Ctrl+F` with sandboxed smoke tests
- User-facing manual verification procedure documented

**2. Keyboard Authenticity (PR #10)**
- 5-column × 8-row main grid + 4 top-row mode buttons (replacing the prior 8-col landscape layout); ENTER 2-wide; 39 key entries total
- Three-label `KeyDef` model: primary `id`/`label`, optional `shifted: { id, label }` (orange), optional `alphaChar` (blue)
- One-shot SHIFT prefix lives entirely frontend-side (`shiftActive: boolean` in `App.tsx`); never crosses IPC
- `run_stop` Tauri command (symmetric with `sst_step`/`bst_step`); reaches the R/S key for the first time
- Stub-error pattern (D-5): `pi`, `polar_to_rect`, `rect_to_polar`, `beep`, `asn`, `catalog`, `view`, `xeq_prompt`, `gto_prompt`, `lbl_prompt` return `GuiError { message: "'<id>' is planned for a future phase" }` — surfaced as 2 s toast overlay; never silently discarded
- `invokeForKey` + `extractErrMessage` helpers centralize Tauri command routing and error-message extraction
- New ops mapped (`sq`, `ypow`, `tenpow`, `xge_y`); ALPHA mode routes physical-keyboard letters correctly
- Toast overlay with `@keyframes toast-fade`; SHIFT armed-glow; annunciator colors

### Quality at Ship

- `hp41-core` coverage: 92.5 % lines / 89.9 % regions (down slightly from v1.0's 94.87 % high-water mark; new synthetic dispatch arms account for the slip)
- All CI gates green (`ci.yml` + `ci-gui.yml`, 3-OS matrix)
- Zero panics policy preserved; SC-4 invariant verified (no calculator logic in `hp41-gui`)

### Why Two Tasks, Not a Milestone

The work was scoped, planned and executed by Claude Code session-by-session against `develop` without the GSD discuss → plan → execute → verify cycle. The original "v2.1 Polish" milestone scope (14-segment LCD font, `?` shortcut overlay, USER mode keyboard display) was *not* delivered — those three items have been carried forward to v2.2 as a final GUI Polish phase per scope decision 2026-05-13.

### Known Deferred Items (→ v2.2)

- **130-function HP-41CV ROM built-in set** (the bulk of v2.2 scope):
  - Math/conversions: `PI`, `P→R`, `R→P`, `RND`, `FRC`, `MOD`, `ABS`, `FACT`, `SIGN`, stack `R↑`
  - 56 user flags + system flags: `SF`, `CF`, `FS?`, `FC?`, `FS?C`, `FC?C`
  - Display/prompt: `VIEW`, `AVIEW`, `PROMPT`, `AON`, `AOFF`, `CLD`
  - Program control: `STOP`, `PSE`, `CLP`, `DEL`, `INS`, `GTO IND`, `XEQ IND`, `BEEP`, `TONE n`
  - ALPHA ops: `ARCL`, `ASTO`, `ATOX`, `XTOA`, `AROT`, `POSA`
  - Indirect addressing for STO/RCL/ISG/DSE/SF/CF/FS?/FC?
  - Remaining conditional tests at the skin (only `X≥Y` keyboard-reachable today)
  - Modal routing for the prompt-IDs that currently surface as `unknown key` toast (`sto_prompt`, `rcl_prompt`, `fix_prompt`, `sci_prompt`, `eng_prompt`, `isg_prompt`, `sf_prompt`, `cf_prompt`, `fs_prompt`, `x_eq_y_prompt`, …)
  - Catalog: `CATALOG 1/2/3/4`, `ASN`, `CLA`, `CLST`, `SIZE`, `PACK`, `MEM LOST`
- **GUI Polish (carried over from original v2.1 scope):**
  - SKIN-04 14-segment SVG font for authentic LCD rendering
  - SKIN-05 `?` keyboard shortcut overlay (port from CLI `help_data.rs`)
  - PROG-02 Full keyboard assignment display in USER mode
  - `prgm_mode` binding for 'p' key (currently mapped to `prx`)

### Deferred Permanently to v3.x

- FR-21 Module emulation (Math 1 / Stat 1 / Time / Advantage Pacs) — separate milestone family; scope decision 2026-05-13

---

## v2.2 — HP-41CV Feature Completeness

**Status:** ✅ SHIPPED 2026-05-15
**Phases:** 8 (Phases 20–27)
**Plans:** 26 total, all complete
**Pull requests:** v2.2 milestone PR (8/8 phases merged into `develop`); tag `v2.2` on `main`

### Delivered

The HP-41CV ROM built-in function set (~130 named operations) completed end-to-end across `hp41-core`, `hp41-cli`, and `hp41-gui`, with a JSON-canonical documentation pipeline and a tightened quality gate.

- **Phase 20 — Core Math & Conversions:** ~25 new ops (PI, P↔R, RND, FRC, MOD, ABS, FACT, SIGN, R↑); polar conversions respect angle mode; numerical_accuracy suite extended.
- **Phase 21 — Flags, Display Control, Sound:** 56 user + system flags as `flags: u64` on `CalcState`; SF/CF/FS?/FC?/FS?C/FC?C; VIEW/AVIEW/PROMPT/AON/AOFF/CLD; BEEP/TONE (CLI-side silent stubs).
- **Phase 22 — Program Control & Memory Ops:** STOP, PSE, CLP, DEL nnn, INS, GTO/XEQ IND, CATALOG 1, ASN/CLA/CLST, SIZE, PACK, MEM LOST.
- **Phase 23 — ALPHA Operations:** ARCL, ASTO, ATOX, XTOA, AROT, POSA — full ALPHA-register manipulation.
- **Phase 24 — Indirect Addressing:** 11-variant `*Ind` family (STO/RCL/ISG/DSE/SF/CF/FS?/FC?/STO+/-/×/÷ IND).
- **Phase 25 — CLI Integration & JSON Pipeline:** f-prefix one-shot model on CLI (mirrors GUI `shiftActive`); hybrid `PendingInput` struct-variants (FlagPrompt / RegisterPrompt) collapsing 34 logical ops into 2 carriers; IND-toggle via shift-0 inside modals; `docs/hp41cv-functions.json` as single source of truth + scripts/docs-matrix code-generator (`just docs-matrix` / `just docs-matrix-check`).
- **Phase 26 — GUI Integration & Polish:** ~80 new key-map arms, 12-variant `PendingInput` TS union with modal LCD rendering; 14-seg SVG LCD font; `?` keyboard-shortcut overlay; USER-mode keyboard assignment display; `prgm_mode` rebound to `p`; stub-error arm shrunk to v3.x-only.
- **Phase 27 — Test Hardening:** coverage gate raised atomically from 80% → 95% (D-27.2); numerical_accuracy extended to 566 cases at 99.1% pass rate (v1.x 503-case floor preserved at ≥498); WebdriverIO + tauri-driver E2E smoke on Ubuntu; Vitest CI gating closed.

### Quality at Ship

| Gate | Target | Achieved |
|------|--------|---------|
| hp41-core line coverage | ≥ 95% | 95.25% (regions 93.75% / functions 97.68%) |
| Numerical accuracy | ≥ 98% (566 cases) | 99.1% (561/566); v1.x baseline floor 498/503 preserved |
| Panics in hp41-core | 0 | 0 (`#![deny(clippy::unwrap_used)]`) |
| CI | Win/macOS/Ubuntu | ✅ all green (`ci.yml` + `ci-gui.yml` + `e2e-linux`) |
| Workspace tests | green | 1202/1202 |
| Vitest | green | 142/142 |
| E2E smoke | green | 1/1 (Ubuntu) |

### Archives

- [REQUIREMENTS.md](v2.2-REQUIREMENTS.md)
- [ROADMAP.md](v2.2-ROADMAP.md)
- Phase plans: `milestones/v2.2-phases/` (20-27, 26 plan files)

### Known Deferred Items (→ v3.0+)

- FR-21 Module emulation — Math 1 Pac (v3.0), Stat 1 Pac (v3.1), Time + Advantage Pacs (v3.2+)
- Branch protection wiring for `e2e-linux` required-check (HUMAN-UAT item 2 — manual repo-setting follow-up)

---
*For current project status, see .planning/STATE.md*
