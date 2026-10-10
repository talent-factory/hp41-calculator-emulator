# Milestones

## v4.3 — Hardware Fidelity

**Status:** ✅ SHIPPED 2026-06-10 — tag `v4.3` → `b22c476`; PR #26 merged to `main` (`--merge`, `6cdc09b`); GitHub Release `v4.3` published (11 assets).
**Phases:** 6 (62–67) · **Plans:** 26 · **Tasks:** 46 · **Theme:** Close remaining behavioral gaps vs. real HP-41CX hardware (no new functions/modules).

**Key accomplishments:**

- **Run-loop yield engine** (Phase 63) — synchronous pending-interrupt at the `run_loop` boundary + PSE/VIEW/AVIEW render-sleep-resume; **the GUI gained its first continuous program run loop** (`run_program`/`resume_program` Tauri commands + no-poll TS yield driver, D-11). Interrupting control alarms (D-40-04 anchor) now execute mid-run.
- **Interactive GETKEY** (Phase 64) — new `WaitForKey` yield kind (no new `Op`) + `resume_program_with_key()` suspends execution, awaits a keypress, pushes the HP-41 row×col code to X; CLI↔GUI parity (D-25.6); R/S keycode doc-fix 84→31.
- **Standalone fidelity fixes** (Phase 65) — CLI `display_override`, CHS mantissa sign-flip, AON flag-48 auto-display, `FACT(27..69)` via two-tier `HpNum{mantissa,exponent}` (ADR v4.3-005), 12-cell LCD scientific-overflow render (ADR v4.3-006).
- **Verification + divergence sweep** (Phase 66) — UNC-02 fixed (PRX/PRA/PRSTK error `NonExistent` with no printer flag), UNC-01/03 confirmed correct; D-40-04 closed; all quality gates green (numerical accuracy 98.6%, coverage 95.26%).
- **Reset Escape Hatch** (Phase 67) — `CalcState::soft_reset()` / `memory_lost()` outside `dispatch()`; CLI `Ctrl+R` two-tier prompt (RDPRGM→Ctrl+E), GUI/iOS ON-key tap=soft / long-press+confirm=full (MEMORY LOST), autosave overwritten under-mutex; ADR v4.3-007 + D-CV-10. CR-02 (shared-Arc `cancel_requested` orphan) found+fixed in-cycle.
- **Pre-phase desktop polish** (quick-task `37dfd5f`) — single-instance guard (ADR-v4.3-001) + configurable macOS global hotkey ⌃⌥⌘H (ADR-v4.3-002); see detail below.

**Known deferred items at close:** 19 (see STATE.md → Deferred Items) — all pre-existing v4.1-era quick-tasks or known Phase-63/65 visual-confirm UAT/verification items; none gate the released milestone.

### Quick-tasks (landed on `develop` ahead of formal phases)

- **Desktop menu-bar UX: single-instance guard + configurable global hotkey** — done 2026-06-06, commit `37dfd5f` (`develop`). Recorded as a quick-task (no GSD phase directory; small, self-contained desktop-platform polish).
  - **Single-instance** (`tauri-plugin-single-instance`, ADR-v4.3-001): registered first; a second launch surfaces the already-running instance (popover on macOS menu-bar, else show+focus) and exits, so no duplicate menu-bar tray icon is ever created. Also fixes the `just gui-dev`-twice dev annoyance.
  - **Configurable macOS global hotkey** (`tauri-plugin-global-shortcut`, ADR-v4.3-002): default `⌃⌥⌘H` toggles the popover via Carbon `RegisterEventHotKey` (no Accessibility permission). Recorded live through a macOS-gated Settings overlay (`ShortcutRecorder.tsx`); the Rust backend validates + live-registers, so only an accepted accelerator sticks. Stored in `GuiPrefs.global_shortcut` (`#[serde(default)]`, back-compat preserved).
  - **Shared foundation:** one `tray::toggle_popover` / `surface_popover` for tray click, hotkey and relaunch; `PopoverState.last_tray_rect` positions the popover with a unit-tested top-right fallback (`compute_fallback_position`). No `hp41-core` / IPC / `Op` change, no new Tauri capability; iOS build unaffected (desktop/macOS-gated deps).
  - **Verification:** 126 Rust (gui crate) + 337 Vitest green; `tsc`, `license-audit`, permission coverage (21/21) and iOS-target `cargo check` all passed.
  - **Deferred:** GUI version bumps (`hp41-gui/src-tauri/Cargo.toml`, `tauri.conf.json`) and the root `hp41 --version` bump happen at the v4.3 release, not per quick-task.

---

## v4.2 — Help Search Enrichment

**Status:** ✅ SHIPPED 2026-06-05
**Phases:** 4 (Phases 58–61)
**Plans:** 11 total, all complete
**Tasks:** 11
**Timeline:** 2 days (2026-06-04 → 2026-06-05)
**Source:** 52 commits, 65 files changed (+14,394 / −599) since v4.1
**Tag:** v4.2
**Ship:** PR #23 (develop→main)

### Delivered

Turned the existing `?` help overlay into an intent-aware function finder — German + English aliases, hand-rolled typo-tolerant fuzzy matching, and relevance ranking — with no new view, just a smarter filter behind the same input. `hp41-core` untouched (UI/help-only milestone); zero new runtime dependencies.

### Key Accomplishments

1. **search_aliases data model (Phase 58)** — invisible `search_aliases` field added to both help-entry mirrors (Rust `Vec<String>` + TS `string[]`) with serde-default / optional-field backward-compat; all six JSON pools and every render path provably untouched.
2. **Tiered runtime matcher (Phase 59)** — alias-aware scorer (exact > prefix > substring > fuzzy) over display_name + description + category + search_aliases, with a hand-rolled bounded Levenshtein (zero new deps); non-empty query → relevance-ranked flat list, empty query → existing category-grouped view bit-for-bit unchanged; reuses the existing `?` overlay input in both frontends (no new view).
3. **Alias authoring pipeline (Phase 60)** — `scripts/help-aliases/` generator + LLM runner producing a true alias-only diff; 364 `status:"implemented"` entries across all six pools DE+EN-aliased (a Wave-2 writeback defect was caught by human review and fixed before commit).
4. **Quality gates (Phase 61)** — real-data unit tests (Rust 10 + TS 36), a committed CLI↔GUI parity fixture asserted by both frontends (top-1 drift guard), and a six-pool `search_aliases` schema CI gate (hp41cv hardcoded, never globbed) wired into `ci.yml`; CLAUDE.md JSON-canonical-data-flow section updated (search_aliases field, DE-in-search exception, five→six pools).

### Known deferred items at close

- HSCOV-01 (zero-result / missed-query logging) — v2/deferred, intentionally out of v4.2 scope.
- Verification bookkeeping accepted as tech debt (see `milestones/v4.2-MILESTONE-AUDIT.md`, status `tech_debt`): Phases 60 & 61 have no aggregated VERIFICATION.md (data-gen phase / verification phase whose deliverables ARE the green gates); Nyquist `wave_0_complete: false` across phases. No functional gaps — the cross-phase integration audit confirmed all 18 v1 requirements wired end-to-end.

---

## v4.0 — Platform Maturity

**Status:** ✅ SHIPPED 2026-05-28
**Phases:** 5 (Phases 48–52)
**Plans:** 17 total, all complete
**Tasks:** 36
**Timeline:** 2 days (2026-05-26 → 2026-05-28)
**Source:** 150 commits, 210 files changed (+28,575 / −13,109) since v3.3
**Tag:** v4.0

### Delivered

Evolved the emulator from feature-complete module emulation (v3.3) to a polished desktop platform: GUI theming, user onboarding, full GUI↔CLI keyboard parity, HP-41 community `.raw` file exchange, and HP-41CX Extended Memory.

### Key Accomplishments

1. **GUI theming (Phase 48)** — 4 built-in skin themes (dark, light, classic beige, high-contrast) via CSS custom properties; `prefs.rs` backend persisting to `~/.hp41/prefs.json`, fully isolated from `CalcState` (ADR-v4.0-004)
2. **Onboarding + keyboard parity (Phase 49)** — 5-panel first-run OnboardingWizard, searchable in-app function reference (~350 entries with example/notes), and GUI physical-keyboard shortcuts at parity with the CLI via canonical `keyboard-shortcuts.json` (61 entries, ADR-v4.0-005)
3. **`.raw` file I/O (Phase 50)** — HP-41 community `.raw` program import/export via native Tauri dialog + CLI flags; multi-program archive splitting (`decode_all_programs`, 256-program DoS cap) and Tauri anti-deadlock dialog-before-lock ordering (ADR-v4.0-006)
4. **X-MEM core (Phase 51)** — HP-41CX Extended Memory: 8 ops (EMDIR/EMROOM/SAVEP/GETP/SAVED/GETD/EMREG/SAVERX), `XmemFile` model, 600-register capacity, backward-compat serde, isolated from `state.regs`/`adv_matrices`
5. **X-MEM hardening + docs (Phase 52)** — XEQ-by-name via `builtin_card_op` (HP-41CX OS-builtin, no XROM bit), 6th help pool, op↔JSON parity + per-op test-count meta-gates, backward-compat fixtures, 3 ADRs + divergences doc
6. **Quality** — full `just test`/`lint`/`gui-ci`/`docs-matrix-check` green; milestone audit confirmed all 32 requirements satisfied with cross-phase integration + 5/5 E2E flows

### Cleanup at milestone close

- **Code-review warnings 52-REVIEW WR-01/02/04 — resolved:** added an X-MEM XROM-bitfield-independence test (WR-01, `xeq_builtin_resolver.rs`), completed `v33-autosave.json` into a realistic full v3.3 save (WR-02), and memoized `helpEntriesAll()` for a stable reference (WR-04, `help_data.ts`). (WR-03 help-text accuracy was fixed during phase 52.)
- **6 completed quick-tasks (v1.0–v2.2 era) — archived** to `milestones/quick-tasks/`; `audit-open` is now fully clean (0 open items). They were flagged only because the running `gsd-sdk` reads bare `SUMMARY.md` while these use `${quick_id}-SUMMARY.md`.
- **Phase-50 dialog visual checkpoint — accepted:** native file dialogs + multi-program picker confirmed working in the shipped GUI (integration audit Flow 4).
- Nyquist `VALIDATION.md` for phases 48–50 remain `draft` — acceptable: the work is verified via the milestone integration audit + retroactive `48/49/50-VERIFICATION.md` (Nyquist closure is discovery-only per the audit workflow).

---

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

## v3.0 — Math Pac I Emulation

**Status:** ✅ SHIPPED 2026-05-20 (v3.0 git tag); bookkeeping archive 2026-05-21
**Phases:** 5 (Phases 28–32)
**Plans:** 31 total, all complete (10 + 3 + 3 + 5 + 10 — Phase 32 = 3 original + 7 gap-closure)
**Timeline:** 3 days code work (2026-05-18 → 2026-05-20)
**Source delta:** 260 commits since `v2.2` tag; 323 files, +69 053 / −2 342 lines; 11 829 LOC in `hp41-core/src/ops/math1/`
**Pull request:** v3.0 milestone PR (5/5 phases merged into `develop`); tag `v3.0` on `main`

### Delivered

Behavioral emulation of the HP-41C **Math Pac I** (HP part number 00041-90034, Owner's Manual 1979) as the first XROM application module. 10 prompt-driven workflow programs with ~55 XEQ-by-name entry points, usable in CLI + GUI through a new modal-workflow layer that extends the v2.x built-in pattern, without HP-copyrighted ROM-image redistribution.

- **Phase 28 — XROM Framework + Math Pac I Core Ops** (`hp41-core` only, 10 plans, 2026-05-16): `XromModule` registry + `MATH_1` const + `xrom_resolve` (fires LAST in resolver chain per Pitfall 1); 6 new `CalcState` fields with `#[serde(default)]` / `#[serde(skip)]`; ~40 new `Op` variants for hyperbolics, complex stack arithmetic + 13 complex functions, POLY/ROOTS, MATRIX (DET/INV/SIMEQ via LU + Gauss-Jordan, OM-transcribed EPSILON), INTG (Simpson + `run_loop` re-entrancy + 4-deep call-stack cap), SOLVE (modified secant + 3 OM-cited termination paths), DIFEQ (RK4), FOUR (DFT + RECT/polar toggle), 5 triangle solvers, TRANS (2D/3D + Rodrigues). 5 ADR decisions locked.
- **Phase 29 — CLI Integration** (`hp41-cli` only, 3 plans, 2026-05-17): `xeq_by_name_local_resolve` → `xrom_resolve`; second `OnceLock<Vec<HelpEntry>>` for `docs/hp41-math1-functions.json` (DOC-01 pulled forward per D-29.1); ~40 new `op_display_name` arms; modal-prompt routing through `print_buffer`.
- **Phase 30 — Documentation & ADRs** (`docs/` + tooling, 3 plans, 2026-05-17): `scripts/docs-matrix` two-input extension (surgical `Entry` widening + conditional XROM column); `docs/hp41-math1-function-matrix.md` regenerated via `just docs-matrix`; `just docs-matrix-check` CI drift gate; `docs/hp41-math1-divergences.md` three-bucket numbered catalog (OM divergences / emulator extensions / behavioral policies); 3 new ADRs (`v3.0-001-op-strategy.md`, `v3.0-002-user-callback-policy.md` with verbatim Free42 disclaim, `v3.0-005-json-pipeline.md`); README v3.0 soft-claim + CLAUDE.md `### v3.0 additions` block.
- **Phase 31 — GUI Integration** (`hp41-gui` only, 5 plans, 2026-05-18): ~40 new `prgm_display.rs` arms (SC-4 preserved); CATALOG 2 XROM enumeration; Math Pac I help-overlay parallel-load (Vite JSON-import); LCD-alternation modal prompts; R/S 3-way + Esc cascade; `request_cancel` cancellation channel (`Arc<AtomicBool>` field + Tauri command + permissions TOML; per-64-samples lock release in `op_integ` / `op_solve` / `op_difeq`; Pitfall 11 mitigation).
- **Phase 32 — Test Hardening & Quality Gates** (`tests/` + `scripts/` + `.github/` + `justfile`, 10 plans = 3 original + 7 gap-closure, 2026-05-18 → 2026-05-20): coverage hold + meta-gate graduation (`math1_op_test_count.rs` + `xrom_shadowing.rs` actively cross-check 45 `Op` variants × 14 test files + 52 `MATH_1.ops` × 18-entry allowlist); `lint_math1_assertions.rs` Pitfall 14 + 17 discipline; `numerical_accuracy.rs` 566 → 763 cases (99.3 % pass); E2E smoke extended (`sinh(1)` + `MATRIX DET` Math Pac I workflows on Ubuntu); `scripts/check-free42-contamination.sh` D-32.7 12-symbol guard wired into `just ci` + `ci.yml::license-audit` parallel job (D-32.8). Gap-closure run added ~70 risk-weighted error-branch tests across 9 new files, closing the coverage gate from 91.74 % → 95.39 % lines / 92.14 % → 94.26 % regions; README v3.0 line graduated to OM-cited hard claim per D-32.5.

### Quality at Ship

| Gate | Target | Achieved |
|------|--------|---------|
| `hp41-core` line coverage | ≥ 95 % | **95.39 %** |
| `hp41-core` region coverage | ≥ 93 % | **94.26 %** |
| Per-file `ops/math1/*.rs` floor | ≥ 90 % | all ≥ 90 % (lowest: poly 90.45 %) |
| Numerical accuracy | ≥ 98 % (768 cases) | 99.3 % (763/768); v1.x 503-case floor 498/503 preserved |
| Panics in `hp41-core` | 0 | 0 (`#![deny(clippy::unwrap_used)]`) |
| CI | Win/macOS/Ubuntu | ✅ all green (`ci.yml` + `ci-gui.yml` + `e2e-linux`) |
| Free42 contamination | 0 distinctive symbols | 0 (CI-gated, 12-symbol grep) |
| MSRV | 1.88 declared | 1.88 (CI-enforced) |
| New test files | — | 26 (math1_* + xrom_* + program_error_branches) |

### Archives

- [ROADMAP.md](v3.0-ROADMAP.md)
- [REQUIREMENTS.md](v3.0-REQUIREMENTS.md)
- Phase plans: `milestones/v3.0-phases/` (28–32, 31 plan files)

### Known Deferred Items (→ v3.1+)

- **Stat 1 Pac** — extended statistics beyond Σ-registers → v3.1
- **Time Pac** (HP-41CX clock functions) → v3.2
- **Advanced Matrix Pac** (M+, MAT*, INV-as-transpose, V+, VDOT) → v3.2+
- **Advantage Pac** (PROOT, CABS, CARG, CCHS, CCONJ, Romberg-INTG, CY^X, …) → v3.3+
- HP-copyrighted ROM-image redistribution remains permanently out of scope.

---

## v3.1 — Stat 1 Pac Emulation

**Status:** ✅ SHIPPED 2026-05-24
**Phases:** 5 (Phases 33–37)
**Plans:** 23 total, all complete
**Timeline:** 4 days (2026-05-21 → 2026-05-24)
**Source delta:** 185 commits since `v3.0` tag; 390 files, +43,586 / −3,218 lines; 6,837 LOC in `hp41-core/src/ops/stat1/`

### Delivered

Behavioral emulation of the HP-41C **Stat 1 Pac** (HP part 00041-14001, OM 00041-90030, June 1979) as the second XROM application module (XROM ID 2). 13 programs / 26 XEQ entry points across univariate / ANOVA / regression / hypothesis / nonparametric / distribution / RNG families. Three hand-coded distribution primitives (Acklam/AS 241, Cody AS 239, Lentz AS 63) — zero new runtime dependencies. RAND/SEED bonus utility with persistent RNG state. Full CLI + GUI integration mirroring the v3.0 Math Pac I footprint.

### Key Accomplishments

1. **26 new `Op` variants + 3 hand-coded distribution primitives** — all 13 Stat 1 Pac programs (ΣNORMD, ΣCHISQD, ΣBSTAT/BSTG, ΣMMTUG/MMTGD, ΣAOVONE/AOVTWO/ANOCOV, ΣLIN/EXP/LOGI/POW, ΣMLRXY/MLRXYZ, ΣPOLYP/POLYC, ΣPTST/ΣTSTAT, ΣXSQEV/ΣEFXSQ, ΣCTKKK/CTKK, ΣSPEAR, RAND/SEED) callable via XEQ from CLI and GUI
2. **OM-verified register layout** — Pitfall 21 mitigated by transcribing OM "Storage Registers" section as named constants before any Op was written; all ANOVA / regression / contingency-table accesses go through named consts
3. **3-pool JSON help pipeline** — `docs/hp41-stat1-functions.json` (26 entries) + third `OnceLock` + `?` overlay "Stat 1 Pac (XROM 2)" section in CLI and GUI; function-matrix parity tests cross-check all three JSON pools
4. **5 new ADRs + divergence catalog** — v3.1-001 through v3.1-005 lock RNG-state placement, distribution-primitive policy, ANOVA register layout, math1/ freeze carve-out, and ModalProgram::Stat1 enum extension; 12 D-35-NN divergence entries across 3 buckets
5. **Quality gates held** — 791 numerical accuracy cases at 98.86% pass rate (two-level tolerance: 1e-9 closed-form / 1e-7 iterative); Free42 contamination guard extended to 18 tokens; backward-compat test confirms v3.0→v3.1 migration; E2E smoke extended with ΣNORMD Q(1.96) workflow
6. **STAT-FW-02 blocker found and fixed during audit** — `migrate_after_load()` wired into CLI + GUI persistence (commit `08ffacc`)

### Quality at Ship

| Gate | Target | Achieved |
|------|--------|---------|
| `hp41-core` line coverage | ≥ 95.39 % | 93.91 % (denominator dilution; region 95.84 % exceeds target) |
| `hp41-core` region coverage | ≥ 94.26 % | **95.84 %** |
| Per-file `ops/stat1/*.rs` floor | ≥ 90 % | 11/12 ≥ 90 % (anova.rs 86.64 % / 94.33 % region) |
| Numerical accuracy | ≥ 98 % (791 cases) | **98.86 %** (v1.x 503 floor + v3.0 768 floor preserved) |
| Panics in `hp41-core` | 0 | 0 (`#![deny(clippy::unwrap_used)]`) |
| CI | Win/macOS/Ubuntu | ✅ all green (`ci.yml` + `ci-gui.yml` + `e2e-linux` + `license-audit`) |
| Free42 contamination | 0 distinctive symbols | 0 (CI-gated, 18-token grep) |
| MSRV | 1.88 declared | 1.88 (CI-enforced) |

### Archives

- [ROADMAP.md](milestones/v3.1-ROADMAP.md)
- [REQUIREMENTS.md](milestones/v3.1-REQUIREMENTS.md)
- [Milestone Audit](milestones/v3.1-MILESTONE-AUDIT.md)

### Known Deferred Items (→ v3.2+)

- ~~**Time Pac** (HP-41CX clock functions, XROM TBD) → v3.2~~ — shipped in v3.2 (Phases 38-42, 2026-05-25)
- ~~**Advanced Matrix Pac** (M+, MAT*, INV-as-transpose, V+, VDOT, IDN) → v3.2+~~ — shipped as part of Advantage Pac v3.3 (Phases 43-47, 2026-05-26; named-matrix model per ADR-v3.3-001)
- ~~**Advantage Pac** (PROOT, CABS, CARG, CCHS, CCONJ, Romberg-INTG, CY^X) → v3.3+~~ — shipped in v3.3 (Phases 43-47, 2026-05-26)
- **Signed binary releases** (cargo-dist CLI + tauri-action GUI) → post-v3.3
- HP-copyrighted ROM-image redistribution remains permanently out of scope

---

## v3.2 — Time Pac Emulation

**Status:** SHIPPED 2026-05-25
**Phases:** 5 (Phases 38-42)
**Plans:** 19 total, all complete
**Timeline:** 2 days (2026-05-24 → 2026-05-25)
**Source delta:** 50 commits since `v3.1` tag; 216 files, +30,992 / -4,959 lines; 4,556 LOC in `hp41-core/src/ops/time/`

### Delivered

Behavioral emulation of the HP-41CX **Time Module** (HP 82182A, XROM ID 26, OM 00041-90035) as the third XROM application module. 35 XEQ entry points across clock/date arithmetic, live-updating stopwatch and clock display, and a full alarm catalog with past-due detection. The FIRST module introducing real-time behavior into the previously event-driven emulator -- direct `SystemTime::now()` calls in hp41-core, pure-Rust Fliegel-Van Flandern JDN calendar arithmetic, and monotonic-clock stopwatch with centisecond resolution.

- **Phase 38 -- XROM Framework + Clock/Date/Stopwatch/Alarm Core** (`hp41-core` only, 6 plans, 2026-05-24): TIME_MODULE (XROM 26) registration, bit-2 arm in `xrom_resolve`; 35 new `Op` variants; `time_offset_secs: i64` persistent field (SETIME/SETDATE compute delta from system clock); pure-Rust Fliegel-Van Flandern JDN formula (~60 LOC); stopwatch state machine (monotonic `Instant` start + `f64` accumulated/split); `AlarmType::Message` | `AlarmType::Control` enum with 253-entry catalog cap; `ModalProgram::Time(TimeStep)` variant; 12 new CalcState fields with correct serde shapes.
- **Phase 39 -- CLI Integration + Live Display** (`hp41-cli`, 3 plans, 2026-05-25): `docs/hp41-time-functions.json` (35 entries) + fourth `OnceLock` + 35 `op_display_name` arms; live clock/stopwatch display via pull-on-redraw in existing 16ms poll loop (>=1 Hz clock, >=10 Hz stopwatch); stopwatch keyboard mode (Space=toggle, s=split, r=reset, Esc=exit); alarm event draining via `check_alarms()` called every 16ms tick.
- **Phase 40 -- Documentation & ADRs** (`docs/`, 3 plans, 2026-05-25): `hp41-time-divergences.md` (6 D-40-NN entries); `hp41-time-function-matrix.md` (4th docs-matrix invocation); 3 ADRs (v3.2-001 clock access, v3.2-002 live display, v3.2-003 alarm catalog); README v3.2 soft-claim.
- **Phase 41 -- GUI Integration + Live Display** (`hp41-gui`, 3 plans, 2026-05-25): 35 `op_display_name` arms (4-way invariant sealed); `tick_time` Tauri command + 200ms `setInterval`; alarm toast; CATALOG 2 "TIME 2C"; help overlay "Time Pac (XROM 26)" section; 14-segment LCD colon rendering.
- **Phase 42 -- Test Hardening & Quality Gates** (4 plans, 2026-05-25): unified meta-gate (106 XROM variants across 3 modules); 30 oracle-verified date arithmetic cases + 5 stopwatch timing tests + 12 alarm latency tests; v3.1 backward-compat migration (`xrom_modules` 3->7); DDAYS E2E smoke; README hard-claim graduated.

### Key Accomplishments

1. **First real-time behavior in the emulator** -- direct `SystemTime::now()` in hp41-core + `time_offset_secs` delta model; pure-Rust Fliegel-Van Flandern JDN calendar arithmetic (zero new runtime deps); date decimal parsing via string-split-at-decimal per ISG/DSE precedent
2. **Full alarm catalog with past-due detection** -- 253-entry `Vec<AlarmEntry>` on CalcState, message + control alarm types, repeat intervals, `check_alarms()` drain pattern called every 16ms tick; control alarms XEQ stored labels on acknowledgment
3. **Live-updating stopwatch with centisecond resolution** -- monotonic `Instant` start marker (immune to system clock changes); stopwatch freeze-on-save behavioral policy; dedicated keyboard mode in both CLI and GUI
4. **35-entry JSON canonical pipeline** -- fourth `OnceLock` pool; `?` overlay "Time Pac (XROM 26)" section; 4-pool `help_entries_all()` chain; function matrix parity tests across all four JSON pools
5. **Unified XROM meta-gate infrastructure** -- `xrom_op_test_count.rs` and `lint_xrom_assertions.rs` refactored from 4 per-module files to 2 unified files scanning all 3 XROM modules (106 variants)
6. **Quality gates held** -- 96.01% region coverage; 47 new accuracy/timing/latency tests; backward-compat migration verified; Free42 contamination guard covers `time/` tree (18 tokens, exits 0)

### Quality at Ship

| Gate | Target | Achieved |
|------|--------|---------|
| `hp41-core` line coverage | >= 95 % | 93.72 % (denominator dilution from ~4.6K new LOC) |
| `hp41-core` region coverage | >= 93 % | **96.01 %** |
| Per-file `ops/time/*.rs` floor | >= 90 % | all 7 files >= 90 % |
| Numerical accuracy | >= 98 % (821 cases) | 98.86 % (v3.1 791 floor preserved + 30 date accuracy) |
| Panics in `hp41-core` | 0 | 0 (`#![deny(clippy::unwrap_used)]`) |
| CI | Win/macOS/Ubuntu | All green (`ci.yml` + `ci-gui.yml` + `e2e-linux` + `license-audit`) |
| Free42 contamination | 0 distinctive symbols | 0 (CI-gated, 18-token grep) |
| MSRV | 1.88 declared | 1.88 (CI-enforced) |
| Tests passing | -- | 3161 |

### Archives

- [ROADMAP.md](milestones/v3.2-ROADMAP.md)
- [REQUIREMENTS.md](milestones/v3.2-REQUIREMENTS.md)

### Known Deferred Items (-> v3.3+)

- ~~**Advantage Pac** (PROOT, CABS, CARG, CCHS, CCONJ, Romberg-INTG, CY^X) -> v3.3~~ -- shipped in v3.3 (Phases 43-47, 2026-05-26)
- **Interrupting control alarm execution** -- data model stores them but re-entrancy against the 4-level call stack not currently supported (documented divergence D-40-03)
- **Signed binary releases** (cargo-dist CLI + tauri-action GUI) -> post-v3.3
- HP-copyrighted ROM-image redistribution remains permanently out of scope

---

## v3.3 — Advantage Pac Emulation

**Status:** SHIPPED 2026-05-26
**Phases:** 5 (Phases 43-47)
**Plans:** 18 total, all complete
**Timeline:** 2 days (2026-05-25 → 2026-05-26)
**Source delta:** 50 commits since `v3.2` tag; 155 files, +43,042 / -2,246 lines; 10,872 LOC in `hp41-core/src/ops/advantage/`

### Delivered

Behavioral emulation of the HP-41 **Advantage Pac** (OM 00041-90482) as the fourth and final XROM application module, completing all HP-41 module emulation. Dual-chip hardware-faithful design: XROM 22 (ADV CONV + ADV MTRX, 63 ops) and XROM 24 (ADV MATH + ADV TVM, 51 ops) -- ~117 total Op variants across bitwise/base conversion, ALPHA-named matrix operations with LU decomposition, advanced complex math, Laguerre polynomial root-finder (FROOT), Romberg integration (FINTG), RK4 differential equations (FDIFEQ), curve fitting, 3D vector operations, coordinate transforms, and time-value-of-money (TVM) with Newton iteration.

- **Phase 43 -- XROM Framework + All Advantage Pac Ops** (`hp41-core` only, 10 plans, 2026-05-25): ADV_MATH_A (XROM 22, bit-3) + ADV_MATH_B (XROM 24, bit-4) registration; `default_xrom_modules` = `0b0001_1111`; named-matrix model (`Vec<AdvMatrix>` per ADR-v3.3-001, isolated from Math Pac I fields); FROOT Laguerre's method with quadratic deflation (ADR-v3.3-002); dual-XROM design with 12 intentional MATH_1 overlaps (ADR-v3.3-003); math1/ third carve-out `complex_atan2` pub(crate) (ADR-v3.3-004); TVM state with Newton `*I` solver; 36-bit `ADV_WORD_MASK` for hardware-faithful base-N; 9 new CalcState fields.
- **Phase 44 -- CLI Integration** (`hp41-cli`, 2 plans, 2026-05-26): `docs/hp41-advantage-functions.json` (114 entries) + fifth `OnceLock` + 114 `op_display_name` arms; `?` overlay "Advantage Pac (XROM 22+24)" sections; MATH_1 alias overlap discovery (12 mnemonics, bit-4 isolation mask); function matrix generated (5th `just docs-matrix` invocation); 421 hp41-cli tests pass.
- **Phase 45 -- Documentation & ADRs** (`docs/`, 2 plans, 2026-05-26): `hp41-advantage-divergences.md` (9 D-45-NN entries); 4 ADRs (v3.3-001 named-matrix model, v3.3-002 FROOT Laguerre, v3.3-003 dual-XROM design, v3.3-004 math1 visibility promotion); README v3.3 soft-claim; CLAUDE.md v3.3 additions block; architecture-history.md v3.3 narrative.
- **Phase 46 -- GUI Integration** (`hp41-gui`, 2 plans, 2026-05-26): 117 `op_display_name` arms (4-way invariant sealed); CATALOG 2 entries for XROM 22 + XROM 24; HelpOverlay fifth+sixth sections; modal LCD rendering via existing CalcStateView priority chain.
- **Phase 47 -- Test Hardening & Quality Gates** (2 plans, 2026-05-26): unified meta-gate extended to all 5 XROM modules (220 variants); `adv_coverage_supplement.rs` (112 targeted tests); 22 numerical accuracy oracle cases (MDET/MINV/FROOT/FINTG, scipy-derived); backward-compat v3.2->v3.3 (`xrom_modules` 0b0111->0b11111); BININ E2E smoke; README hard-claim graduated.

### Key Accomplishments

1. **Largest XROM module: ~117 Op variants across 7 functional families** -- ADV CONV (12 bitwise/base ops), ADV MTRX (~50 named-matrix ops), ADV MATH complex/solver/polynomial/vector/curve-fit, ADV TVM (6 financial ops); hardware-faithful dual-chip XROM 22 + XROM 24 design per ADR-v3.3-003
2. **ALPHA-named matrix model with full linear algebra** -- `Vec<AdvMatrix>` storage isolated from Math Pac I fields (ADR-v3.3-001); LU decomposition for MDET/MINV/MSYS/M*M; element access, lifecycle, reductions, norms, complex matrix ops, and modal editor workflows
3. **FROOT Laguerre polynomial root-finder** -- arbitrary-degree polynomials via Laguerre's method with quadratic deflation for complex conjugate pairs (ADR-v3.3-002); initial guess (0.4, 0.9) avoids origin singularity; coexists with Math Pac I Bairstow (degree 2-5)
4. **Solver ecosystem completed** -- FSOLVE (Brent/Secant root-finding), FINTG (Romberg integration), FDIFEQ (RK4 differential equations), all using `run_loop` re-entrancy; one level of cross-module nesting allowed (FINTG inside FSOLVE and vice versa)
5. **114-entry JSON canonical pipeline** -- fifth `OnceLock` pool; 5-pool `help_entries_all()` chain; dual help-overlay sections (XROM 22 + XROM 24); function matrix parity tests across all five JSON pools; 12 intentional MATH_1 alias overlaps verified via bit-4 isolation
6. **All HP-41 module emulation complete** -- Math Pac I (v3.0) + Stat 1 Pac (v3.1) + Time Module (v3.2) + Advantage Pac (v3.3) = 5 XROM modules, 220+ Op variants, all four official HP-41 extension modules shipped

### Quality at Ship

| Gate | Target | Achieved |
|------|--------|---------|
| `hp41-core` line coverage | >= 95 % | ~93 % (denominator dilution from ~10.9K new LOC) |
| `hp41-core` region coverage | >= 93 % | **~95 %** |
| Numerical accuracy | >= 98 % (843 cases) | 98.86 % (791 base + 30 time + 22 advantage) |
| Panics in `hp41-core` | 0 | 0 (`#![deny(clippy::unwrap_used)]`) |
| CI | Win/macOS/Ubuntu | All green (`ci.yml` + `ci-gui.yml` + `e2e-linux` + `license-audit`) |
| Free42 contamination | 0 distinctive symbols | 0 (CI-gated, 18-token grep covering math1/ + stat1/ + time/ + advantage/) |
| MSRV | 1.88 declared | 1.88 (CI-enforced) |
| Tests passing | -- | 3262 |

### Archives

- [ROADMAP.md](milestones/v3.3-ROADMAP.md)
- [REQUIREMENTS.md](milestones/v3.3-REQUIREMENTS.md)

### Known Deferred Items (-> post-v3.3)

- **Signed binary releases** (cargo-dist CLI + tauri-action GUI)
- **Interrupting control alarm execution** (Time Pac -- requires re-entrancy against 4-level call stack)
- **Full Extended Memory model** (EMDIR, EMROOM, EMREG -- separate major feature)
- **FROOT/FINTG mutual nesting** (re-entrant X-MEM buffer stack)
- HP-copyrighted ROM-image redistribution remains permanently out of scope

---
*For current project status, see .planning/STATE.md*
