# Roadmap: HP-41 Calculator Emulator

**Project:** HP-41 Calculator Emulator
**Current milestone:** v2.2 HP-41CV Feature Completeness (planning)

---

## Milestones

- ✅ **v1.0 CLI** — Phases 1–8, shipped 2026-05-08 · [Archive](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 CLI Feature Completeness** — Phases 9–12, EEX fix, STO modals, print emulation, synthetic programming — SHIPPED 2026-05-09 · [Archive](milestones/v1.1-ROADMAP.md)
- ✅ **v2.0 Tauri GUI** — Phases 13–18, pixel-perfect HP-41C desktop app — SHIPPED 2026-05-10 · [Archive](milestones/v2.0-ROADMAP.md)
- ✅ **v2.1 Card Reader + Keyboard Authenticity** — recorded as two quick tasks (no Phase 19 GSD directory); SHIPPED 2026-05-13 · see MILESTONES.md
- 🟡 **v2.2 HP-41CV Feature Completeness** — Phases 20–27, ROM built-in 130-function set + docs + GUI integration + polish + test hardening (in planning)

---

## Phases

### v2.2 — HP-41CV Feature Completeness (Phases 20–27)

- [ ] **Phase 20: Core Math & Conversions** — Land the 10 missing ROM math/stack ops (`PI`, `P→R`, `R→P`, `RND`, `FRC`, `MOD`, `ABS`, `FACT`, `SIGN`, `R↑`) in hp41-core with full dispatch + execute_op + prgm_display coverage
- [ ] **Phase 21: Flags, Display Control & Sound** — Add 56-flag storage, `SF/CF/FS?/FC?/FS?C/FC?C`, `VIEW/AVIEW/PROMPT/AON/AOFF/CLD`, `BEEP/TONE` (event buffer pattern) — all in hp41-core
- [ ] **Phase 22: Program Control & Memory Ops** — Land `STOP/PSE/CLP/DEL/INS/GTO IND/XEQ IND` and `SIZE/CLA/CLST/PACK/CATALOG/ASN` in hp41-core (direct addressing for IND prep)
- [ ] **Phase 23: ALPHA Operations** — Land `ARCL/ASTO/ATOX/XTOA/AROT/POSA` direct-address forms in hp41-core
- [ ] **Phase 24: Indirect Addressing (Cross-Cutting)** — Wire `_IND` variants on all addressable ops (STO/RCL/ISG/DSE/SF/CF/FS?/FC?/FS?C/FC?C/STO+/-/×/÷/ARCL/ASTO/VIEW) — single shared resolver, rejects non-integer
- [ ] **Phase 25: CLI Integration & Documentation** — Wire every new Op into `keys.rs` + `KEY_REF_TABLE` + new `PendingInput` modals + exhaustive `pending_prompt()` + `help_data.rs`; ship HP-41CV ROM function matrix; sync PROJECT/CLAUDE/README
- [ ] **Phase 26: GUI Integration & Polish** — Register all new key IDs in `key_map.rs` + `KEY_DEFS`; route previously-stubbed prompts to real modals; 14-seg LCD font; `?`-overlay; USER keyboard display; `p`-key remap to `prgm_mode`
- [ ] **Phase 27: Test Hardening** — Restore `hp41-core` coverage ≥95%; extend numerical accuracy suite; flag-semantics proptest; indirect-addressing integration tests; Playwright GUI E2E smoke test

---

## Phase Details

### Phase 20: Core Math & Conversions
**Goal**: Users can execute the 10 missing HP-41CV ROM math/stack operations from the CLI and inside keystroke programs with hardware-faithful semantics — `PI` pushes the constant, `P→R`/`R→P` respect the current angle mode, `RND` rounds to the current display digit count, `FRC` returns sign-preserving fractional parts, `MOD` returns Y mod X with HP-41 sign semantics, `FACT` rejects X > 69 and non-integer inputs, `R↑` mirrors `Rdn`.
**Depends on**: v2.1 baseline (Card Reader, Keyboard Authenticity, all v1.0/v1.1/v2.0 work)
**Requirements**: FN-MATH-01, FN-MATH-02, FN-MATH-03, FN-MATH-04, FN-MATH-05, FN-MATH-06, FN-MATH-07, FN-MATH-08, FN-MATH-09, FN-STACK-01
**Success Criteria** (what must be TRUE):
  1. From the CLI, typing the keystroke for `PI` followed by ENTER pushes 3.141592654 (10-digit rounded HP-41 hardware value) onto X and lifts the stack
  2. In DEG mode, entering `3 ENTER 4 ENTER 5` and pressing `R→P` produces magnitude ≈ 5 in Y and angle ≈ 53.13° in X; in RAD mode the same inputs return the angle in radians
  3. `5.7 CHS RND` with FIX 1 active produces `-5.7`; with FIX 0 active produces `-6`; `FACT` with X=70 returns `HpError::OutOfRange` and is observable in the CLI display
  4. Starting from stack X=1 Y=2 Z=3 T=4, pressing `R↑` produces X=4 Y=1 Z=2 T=3 (mirror of `Rdn`)
  5. Every one of the 10 new `Op` variants appears in `dispatch()` (ops/mod.rs), `execute_op()` (ops/program.rs), and BOTH `prgm_display.rs` copies (hp41-cli + hp41-gui); compile-time exhaustive matches confirm coverage
**Plans**: 1 plan
  - [ ] 20-01-PLAN.md — Single plan (per D-21): RND helper extraction (Wave-0), 10 new Op variants + dispatch + execute_op + impls (Wave-1), prgm_display in both copies (Wave-1), integration tests (Wave-2)
**Cross-cutting constraints:**
  - `#![deny(clippy::unwrap_used)]` applies — all new code uses `?`-propagation or `.expect("reason")`
  - Each `Op` variant must land in 4 places: `dispatch()` + `execute_op()` + `hp41-cli/src/prgm_display.rs` + `hp41-gui/src-tauri/src/prgm_display.rs` (the built-in trap from CLAUDE.md)
  - SC-4 invariant preserved: no math/calculator logic in `hp41-gui/src-tauri/`
  - `LiftEffect` declared for each new op (PI=Enable, R↑=Neutral, others follow `Rdn`/`Sin` precedent)

### Phase 21: Flags, Display Control & Sound
**Goal**: `CalcState` carries 56 user flags + system flags as `flags: u64` (or equivalent bitfield with `#[serde(default)]`); users can `SF/CF/FS?/FC?/FS?C/FC?C` any flag from the keyboard and inside programs with conditional-skip behavior; `VIEW`/`AVIEW`/`PROMPT`/`AON`/`AOFF`/`CLD` modify the display layer without touching the stack; `BEEP` and `TONE n` emit events into an `event_buffer` (or extend `print_buffer`) so hp41-core stays I/O-free.
**Depends on**: Phase 20
**Requirements**: FN-FLAG-01, FN-FLAG-02, FN-DISP-01, FN-DISP-02, FN-DISP-03, FN-DISP-04, FN-DISP-05, FN-SOUND-01, FN-SOUND-02
**Success Criteria** (what must be TRUE):
  1. From the CLI, executing `SF 05` followed by `FS? 05` causes the next program step to execute (flag set → test passes); `CF 05` followed by `FS? 05` skips the next step
  2. `FS?C 10` on a set flag clears it as a side effect; `FS?C 10` on a clear flag leaves it clear — both observable via `FS? 10` after
  3. `VIEW 03` shows the contents of register R03 in HP-41 display format until the next key is pressed; stack remains unchanged; `CLD` clears the display without touching stack/ALPHA
  4. Executing `BEEP` or `TONE 5` from a program adds a structured event line into the print/event buffer (the exact channel is a settled decision recorded in CLAUDE.md); no `println!`/`eprintln!` appears in hp41-core
  5. A v1.x JSON save file (created before the `flags` field existed) loads in hp41-cli/hp41-gui without error — `#[serde(default)]` initializes flags to 0
**Plans**: TBD
**Cross-cutting constraints:**
  - `flags: u64` (or `[u8; N]`) field on `CalcState` carries `#[serde(default)]` — non-negotiable for save-file backward compat
  - `BEEP`/`TONE` MUST route through a buffer pattern (extends print_buffer OR a new `event_buffer: Vec<String>` with `#[serde(skip)]`) — NO direct I/O in hp41-core
  - Conditional flag tests (`FS?`/`FC?`/`FS?C`/`FC?C`) must apply the skip-next-step semantic inside `run_loop` exactly like the existing `X=Y` family
  - Display-control ops (`VIEW`/`AVIEW`/`PROMPT`/`CLD`) likely need a `display_override: Option<String>` on CalcState — also `#[serde(default)]`
  - All new `Op` variants land in 4 places (dispatch + execute_op + 2× prgm_display)
  - `#![deny(clippy::unwrap_used)]` enforced

### Phase 22: Program Control & Memory Ops
**Goal**: Users can pause/resume programs (`STOP`, `PSE`), edit programs in PRGM mode (`CLP`, `DEL`, `INS`), branch indirectly (`GTO IND`, `XEQ IND` — direct-form only here; IND-resolver lives in Phase 24), and manage memory (`SIZE`, `CLA`, `CLST`, `PACK`, `CATALOG 1..4`, `ASN`). All in hp41-core.
**Depends on**: Phase 21
**Requirements**: FN-PROG-01, FN-PROG-02, FN-PROG-03, FN-PROG-04, FN-PROG-05, FN-PROG-06, FN-PROG-07, FN-MEM-01, FN-MEM-02, FN-MEM-03, FN-MEM-04, FN-MEM-05, FN-KEY-01
**Success Criteria** (what must be TRUE):
  1. A program containing `STOP` halts execution at that step; pressing R/S in the CLI resumes from the next step
  2. A program containing `PSE` briefly displays the current X then continues; `CLP "MYPRG"` removes every step from `LBL MYPRG` to the next `END`/`.END.`
  3. `DEL 005` from PRGM mode removes 5 steps starting at the current PC; `INS` adds one blank step at PC; `PACK` returns success (no-op in our flat-Vec model but the `Op` variant exists and dispatches cleanly)
  4. `CATALOG 2` (programs) emits a structured listing into `print_buffer`; `CATALOG 1` (registers) lists R00–R99 with values; `CLST` zeroes X/Y/Z/T while LASTX is preserved; `CLA` clears ALPHA
  5. `ASN "SIN" 11` records a key assignment that survives a JSON save/load round-trip (existing `assignments` map extended with the new `Op::Asn` variant)
**Plans**: TBD
**Cross-cutting constraints:**
  - `STOP` interacts with `run_loop` — needs a `paused: bool` state transition; `PSE` requires a pause-then-resume marker the CLI/GUI can observe
  - `CLP`/`DEL`/`INS` operate on the existing `Vec<ProgramStep>` — must keep PC consistent (existing CR-01-style fixes apply)
  - `GTO IND nn` / `XEQ IND nn` use the same indirect-resolver groundwork that Phase 24 generalizes; Phase 22 lands the direct PROG-side calls, Phase 24 implements the shared `resolve_indirect()` helper
  - `CATALOG` output goes into `print_buffer` (existing pattern); no direct I/O
  - `Op::Asn` integrates with the existing `assignments: HashMap<u8, Op>` field; `#[serde(default)]` already present
  - All new `Op` variants land in 4 places; `#![deny(clippy::unwrap_used)]` enforced

### Phase 23: ALPHA Operations
**Goal**: Users can manipulate the ALPHA register beyond v1.0's append/clear primitives — `ARCL nn` appends a register's formatted value; `ASTO nn` packs the first 6 ALPHA chars into a register; `ATOX`/`XTOA` interconvert the first ALPHA char with its ASCII code in X; `AROT n` rotates ALPHA (negative N rotates right); `POSA` returns the substring position. Direct-address forms only; IND variants come in Phase 24.
**Depends on**: Phase 22
**Requirements**: FN-ALPHA-01, FN-ALPHA-02, FN-ALPHA-03, FN-ALPHA-04, FN-ALPHA-05, FN-ALPHA-06
**Success Criteria** (what must be TRUE):
  1. With ALPHA="HELLO" and R05 containing 3.14 (FIX 2 active), `ARCL 05` produces ALPHA="HELLO3.14"; switching to SCI 3 then `ARCL 05` again appends in SCI format
  2. With ALPHA="GOODBYE", `ASTO 12` packs "GOODBY" (first 6 chars) into R12 as packed text; `RCL 12` and `ARCL 12` reproduce "GOODBY" in ALPHA
  3. With ALPHA="A" (capital A, ASCII 65), `ATOX` puts 65 in X; with X=66 and ALPHA="", `XTOA` makes ALPHA="B"
  4. With ALPHA="HELLO", `AROT 2` produces ALPHA="LLOHE"; `AROT -1` (i.e. `AROT` with X=-1) produces ALPHA="OHELL"
  5. With ALPHA="THE QUICK BROWN FOX" and X holding "QUICK" (or however POSA encodes the search arg), `POSA` returns 4 in X; for a missing substring returns -1
**Plans**: TBD
**Cross-cutting constraints:**
  - ALPHA register packing (`ASTO`) uses HP-41 6-char ASCII pack — document the exact encoding in CLAUDE.md so future ops match
  - `ARCL` formatting respects the current display mode (FIX/SCI/ENG) — re-uses `format_hpnum()` from `hp41-core/src/format.rs`
  - `AROT` must accept N from the X register per HP-41 hardware behavior (not as immediate operand) — note in plan
  - All new `Op` variants land in 4 places
  - `#![deny(clippy::unwrap_used)]` enforced — particular care with byte-slicing the ALPHA register; use `chars()` not byte indices

### Phase 24: Indirect Addressing (Cross-Cutting)
**Goal**: A single `resolve_indirect(state, reg) -> Result<u8, HpError>` helper in `hp41-core` lets every addressable op (`STO`, `RCL`, `ISG`, `DSE`, `SF`, `CF`, `FS?`, `FC?`, `FS?C`, `FC?C`, `STO+/-/×/÷`, `ARCL`, `ASTO`, `VIEW`) accept an `_IND` form that reads the register-N integer part as the effective address. Non-integer register contents return `HpError::InvalidOp`.
**Depends on**: Phase 23 (all direct-address variants must exist first — IND is layered on top)
**Requirements**: FN-IND-01, FN-IND-02
**Success Criteria** (what must be TRUE):
  1. With R05 containing 12 and R12 containing 99, `RCL IND 05` puts 99 in X; `STO IND 05` followed by `RCL 12` confirms the indirect store wrote to R12
  2. With R05 containing 03 and flag 03 currently clear, `SF IND 05` sets flag 03; `FS? IND 05` then succeeds (no skip)
  3. With R07 containing 12.345 (non-integer pointer), `RCL IND 07` returns `HpError::InvalidOp` in the CLI display — never panics, never silently rounds
  4. With R10 containing 25 and ALPHA empty, `ARCL IND 10` appends the formatted contents of R25 to ALPHA; `ASTO IND 10` packs the first 6 ALPHA chars into R25
  5. `GTO IND 05` (with R05=42) jumps to LBL 42 or step 42 per HP-41 semantics; `XEQ IND 05` similarly invokes the subroutine — verified by program execution test in `hp41-core/tests/`
**Plans**: TBD
**Cross-cutting constraints:**
  - `resolve_indirect()` is the ONE place that converts register-N to a u8 address — no duplication across ops
  - All IND variants are NEW `Op` enum variants (e.g. `StoInd(u8)`, `RclInd(u8)`, `SfInd(u8)`, …) — they MUST land in dispatch + execute_op + both prgm_display copies
  - Non-integer rejection uses `HpError::InvalidOp` (not a new error type) — keeps the error surface stable
  - This is the LARGEST single-phase Op variant count in v2.2 (≈15 new IND variants); plan for a Wave-0 test scaffold then a single Wave-1 implementation plan
  - `#![deny(clippy::unwrap_used)]` enforced — IND resolution path is heavily tested via Phase 27 proptest

### Phase 25: CLI Integration & Documentation
**Goal**: Every new `Op` from Phases 20–24 is reachable from the hp41-cli keyboard with explicit `KEY_REF_TABLE` entries; new `PendingInput` modal variants (`SfPrompt`, `CfPrompt`, `FsPrompt`, `FcPrompt`, `ViewPrompt`, `TonePrompt`, `DelPrompt`, `ClpLabelPrompt`, IND variants) are exhaustively handled by `pending_prompt()`; all 12 conditional tests are keyboard-reachable; `help_data.rs::HELP_DATA` is the up-to-date single source of truth; and the v2.2 documentation deliverables ship together — HP-41CV function matrix (≥130 entries with status column), CLAUDE.md settled-architecture additions (flag storage, indirect resolution, sound buffer), README "feature-complete HP-41CV" claim with cross-link.
**Depends on**: Phase 24 (all `Op` variants must exist before keyboard wiring can compile)
**Requirements**: FN-TEST-01, FN-CLI-01, FN-CLI-02, FN-CLI-03, FN-CLI-04, FN-DOC-01, FN-DOC-02, FN-DOC-03, FN-DOC-04
**Success Criteria** (what must be TRUE):
  1. Every `Op` variant added in Phases 20–24 has a matching entry in `key_to_op()` and `KEY_REF_TABLE` in `hp41-cli/src/keys.rs`; pressing the documented key in the CLI dispatches the correct op
  2. Pressing the `?` help key in the CLI lists every new v2.2 op grouped under a recognizable category (math, flags, display, program control, ALPHA, IND) — `help_data.rs` is updated
  3. All 12 conditional tests (`X=Y`, `X≠Y`, `X<Y`, `X>Y`, `X≤Y`, `X≥Y`, `X=0`, `X≠0`, `X<0`, `X>0`, `X≤0`, `X≥0`) are reachable from the CLI keyboard — verified by typing each one and observing the dispatch result
  4. `docs/hp41cv-function-matrix.md` exists and lists ≥130 HP-41CV ROM ops with an implementation-status column (`✓ v2.x` / `⏳ v3.x module` / `— N/A`); CLAUDE.md "Settled Architecture Decisions" section gains a "v2.2 additions" block; README.md links to the function matrix
  5. `pending_prompt()` in `hp41-cli/src/ui.rs` handles every new `PendingInput` variant without `unreachable!()` or `_ =>` catch-all — verified by exhaustive match compile-check
**Plans**: TBD
**Cross-cutting constraints:**
  - This phase ONLY touches `hp41-cli` and `docs/` and project-root `*.md` — no `hp41-core` changes (all core Ops landed in Phases 20–24)
  - Documentation runs synchronously with CLI integration per PROJECT.md "Build sequence: core → cli → docs → gui → tests" — the two are bundled here so the function matrix has authoritative coverage data
  - Function matrix entry format: one row per HP-41CV ROM op with columns `Op | Category | Status | Phase | Notes`
  - `help_data.rs` remains the SINGLE SOURCE OF TRUTH for key descriptions — `?` overlay reads from it; no hardcoded help strings elsewhere

### Phase 26: GUI Integration & Polish
**Goal**: Every new v2.2 key ID resolves via `hp41-gui/src-tauri/src/key_map.rs::resolve` (both bare and parameterized prefixes); `KEY_DEFS` in `Keyboard.tsx` carries correct three-label (primary/shifted/alphaChar) bindings for every new HP-41C keyboard-reachable function; previously-stubbed prompt IDs (`sto_prompt`, `rcl_prompt`, `fix_prompt`, `sci_prompt`, `eng_prompt`, `isg_prompt`, `sf_prompt`, `cf_prompt`, `fs_prompt`, `x_eq_y_prompt`, `x_le_y_prompt`, `x_gt_y_prompt`, `x_eq_0_prompt`) route to real modal flows in the React frontend — no more `unknown key` toasts for HP-41CV built-ins; the stub-error arm shrinks to v3.x module ops only. GUI Polish ships in the same phase: 14-seg SVG LCD font replaces the CSS-text display; `?` keyboard shortcut overlay ports from `help_data.rs`; USER mode shows current key assignments overlaid on the skin; the `p` key remaps from `prx` to `prgm_mode` (resolves the v2.0 deferred conflict).
**Depends on**: Phase 25 (CLI must be fully wired with all `Op` variants before GUI key_map can reference them; help_data.rs is the source for the `?`-overlay port)
**Requirements**: FN-GUI-01, FN-GUI-02, FN-GUI-03, FN-GUI-04, FN-GUI-05, FN-POLISH-01, FN-POLISH-02, FN-POLISH-03, FN-POLISH-04
**Success Criteria** (what must be TRUE):
  1. Every new v2.2 op key ID (e.g. `pi`, `p_to_r`, `r_to_p`, `rnd`, `frc`, `mod_op`, `abs`, `fact`, `sign`, `r_up`, `sf`, `cf`, `fs_q`, `view`, `aview`, `prompt`, `beep`, `tone`, `arcl`, `asto`, `atox`, `xtoa`, `arot`, `posa`, …) resolves via `key_map::resolve` and clicking the corresponding `KEY_DEFS` entry in the GUI dispatches the correct op — verified by `test_all_keyboard_skin_ids_are_valid` (Phase 16 pattern, extended)
  2. Clicking the SHIFT-modified key for previously-stubbed prompt IDs opens an in-app modal (frontend React component, not a toast) — e.g. clicking `STO` (with SHIFT off) opens a register-picker modal; clicking the `f`-prefixed `SF` key opens a flag-picker (0–55); the modal dispatches the final parameterized op via `dispatch_op`
  3. The HP-41 12-char display in the GUI renders via a 14-segment SVG font (one `<g>` per character with 14 line/polygon segments) — visually distinguishable from the previous CSS-text version; matches HP-41C hardware look
  4. Pressing `?` in the GUI opens a keyboard shortcut overlay populated from a TypeScript port of `help_data.rs`; toggling USER mode (existing `Op::User` already wired) overlays current `Op::Asn` mappings on the SVG skin keys
  5. Pressing the `p` key now opens PRGM mode (not PRX); the SC-4 invariant grep (`fn op_(add|sub|...|format_hpnum)`) returns nothing in `hp41-gui/src-tauri/src/` — no calculator/math logic duplicated; `CalcStateView` size stays ≤500 bytes (relaxed from ≤300 to accommodate `flags` field — recorded in CLAUDE.md)
**Plans**: TBD
**Cross-cutting constraints:**
  - **SC-4 invariant non-negotiable**: NEVER add `op_*` / `flush_entry_*` / `format_hpnum` to `hp41-gui/src-tauri/` — `op_display_name` in `prgm_display.rs` remains the ONLY display-formatter exception
  - Stub-error arm shrinks to v3.x-only — every HP-41CV ROM op resolves successfully; only module-Pac functions (Math 1 / Stat 1 / Time / Advantage) remain as stubs
  - D-07 (no silent discards) preserved — unhandled IDs still produce `GuiError` toast, never silent
  - Modal frontend components are React-only (TypeScript); they call `dispatch_op` with the resolved parameterized ID (e.g. `sto_05`, `sf_12`)
  - `CalcStateView` may gain `flags`, `display_override`, `event_buffer` fields — JSON budget relaxed to ≤500 bytes (FN-GUI-05); document the new size envelope in CLAUDE.md
  - 14-seg font: a single TypeScript SVG component with 14 segments per glyph; new `Display14Seg.tsx` replaces the `.display-text` CSS span; HP-41C character set (A–Z, 0–9, period, comma, minus, special chars)
  - `?` overlay ports `help_data.rs` to TypeScript — extract a JSON-shape data file in Phase 25 that both Rust and TypeScript can read, OR maintain a TypeScript mirror with a doctest enforcing parity
  - `p` key remap: existing `KEY_DEFS` entry for `p` changes from `prx` to `prgm_mode`; `prx` migrates to a different key (e.g. shifted variant)
**UI hint**: yes

### Phase 27: Test Hardening
**Goal**: `hp41-core` line coverage returns to ≥95% (recovering from the 92.5% slip recorded in v1.1/v2.1); the 500-case numerical accuracy suite is extended with cases for every new math/conversion op (PI, P→R, R→P, RND, FRC, MOD, FACT) and maintains the ≥98% pass rate gate; proptest covers flag set/clear/test invariants across all 56 user flags; integration tests verify indirect addressing resolution and non-integer rejection on every `_IND` op; a Playwright E2E smoke test in `ci-gui.yml` boots the Tauri app, clicks a representative subset of keys, and asserts display state.
**Depends on**: Phase 26 (all functionality must be in place before final coverage push and E2E test)
**Requirements**: FN-QUAL-01, FN-QUAL-02, FN-QUAL-03, FN-QUAL-04, FN-QUAL-05
**Success Criteria** (what must be TRUE):
  1. `just coverage` reports `hp41-core` line coverage ≥ 95.0% — the gate config in `justfile` is updated to enforce this threshold (raised from 80%)
  2. `hp41-core/tests/numerical_accuracy.rs` reports ≥ 490 / 500 cases passing (≥98%) with the v2.2 case extensions for PI, P→R, R→P, RND, FRC, MOD, FACT added
  3. A proptest module (e.g. `hp41-core/tests/flag_properties.rs`) asserts: for all u8 n in 0..56, `SF(n); FS?(n) == true`; `CF(n); FC?(n) == true`; `SF(n); FS?C(n); FC?(n) == true` — runs in CI as part of `just test`
  4. Integration tests in `hp41-core/tests/indirect_addressing.rs` verify every `_IND` op (STO/RCL/ISG/DSE/SF/CF/FS?/FC?/FS?C/FC?C/STO+/-/×/÷/ARCL/ASTO/VIEW) — happy path + non-integer rejection
  5. A Playwright spec runs as a new job in `.github/workflows/ci-gui.yml` on Linux, boots `just gui-dev` (or production build), clicks `2 ENTER 3 +`, and asserts the display reads `5.0000` (or current display-mode equivalent) — green on the Ubuntu runner
**Plans**: TBD
**Cross-cutting constraints:**
  - Coverage gate raise (80% → 95%) is a `justfile`/`just coverage` recipe change — must be committed atomically with the test additions or CI will fail
  - Proptest cases should NOT exceed 256 iterations per case to keep CI runtime reasonable; flag invariants are fast — 1024 iterations is fine for those
  - Playwright runs ONLY on Ubuntu in `ci-gui.yml` (macOS/Windows runners are slow and Playwright headless is best supported on Linux); document this scope in CLAUDE.md
  - No new `Op` variants in this phase — purely test and gate work
  - `#![deny(clippy::unwrap_used)]` continues to apply; test modules carry `#[allow(clippy::unwrap_used)]` at the test mod level

---

## Progress Table

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Foundation | v1.0 | 4/4 | Complete | 2026-05-06 |
| 2. Core Math | v1.0 | 7/7 | Complete | 2026-05-07 |
| 3. Programming Engine | v1.0 | 6/6 | Complete | 2026-05-07 |
| 4. TUI & Input | v1.0 | 5/5 | Complete | 2026-05-07 |
| 5. Persistence & UX | v1.0 | 11/11 | Complete | 2026-05-07 |
| 6. Science & Engineering | v1.0 | 3/3 | Complete | 2026-05-07 |
| 7. Hardening | v1.0 | 6/6 | Complete | 2026-05-07 |
| 8. Tech Debt Cleanup | v1.0 | 3/3 | Complete | 2026-05-08 |
| 9. Infrastructure & EEX Fix | v1.1 | 3/3 | Complete | 2026-05-08 |
| 10. STO Arithmetic Modals | v1.1 | 3/3 | Complete | 2026-05-08 |
| 11. Print Emulation | v1.1 | 4/4 | Complete | 2026-05-08 |
| 12. Synthetic Programming | v1.1 | 3/3 | Complete | 2026-05-09 |
| 13. Workspace Skeleton | v2.0 | 3/3 | Complete | 2026-05-09 |
| 14. IPC Layer | v2.0 | 4/4 | Complete | 2026-05-09 |
| 15. Display & Keyboard | v2.0 | 3/3 | Complete | 2026-05-10 |
| 16. SVG Skin | v2.0 | 2/2 | Complete | 2026-05-10 |
| 17. Persistence & Print Output | v2.0 | 3/3 | Complete | 2026-05-10 |
| 18. Program Listing & CI/CD | v2.0 | 4/4 | Complete | 2026-05-10 |
| 19. Card Reader + Keyboard Authenticity | v2.1 | quick tasks | Complete | 2026-05-13 |
| 20. Core Math & Conversions | v2.2 | 0/1 | Planned    |  |
| 21. Flags, Display Control & Sound | v2.2 | 0/TBD | Not started | — |
| 22. Program Control & Memory Ops | v2.2 | 0/TBD | Not started | — |
| 23. ALPHA Operations | v2.2 | 0/TBD | Not started | — |
| 24. Indirect Addressing | v2.2 | 0/TBD | Not started | — |
| 25. CLI Integration & Documentation | v2.2 | 0/TBD | Not started | — |
| 26. GUI Integration & Polish | v2.2 | 0/TBD | Not started | — |
| 27. Test Hardening | v2.2 | 0/TBD | Not started | — |
