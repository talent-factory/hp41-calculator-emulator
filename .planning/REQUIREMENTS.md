# Milestone v2.2 Requirements — HP-41CV Feature Completeness

**Milestone goal:** Liefere den vollständigen HP-41CV ROM-Built-in-Funktionsumfang (≈ 130 named ops), integriere ihn in CLI + GUI, vervollständige die Dokumentation und sichere die Test-Coverage zurück auf v1.0-Niveau.

**Scope boundary (locked 2026-05-13):** Strict ROM built-ins only. Module-Pacs (Math 1 / Stat 1 / Time / Advantage) sind explizit Scope von v3.x — gehören NICHT in v2.2-Anforderungen.

**Build sequence:** core → cli → docs → gui → tests. Jede neue `Op`-Variante muss in `hp41-core` (Enum + dispatch + execute_op + `prgm_display.rs`) existieren, bevor sie in `hp41-cli` und `hp41-gui` (key_map + KEY_DEFS) gewired wird.

---

## v2.2 Requirements

### Core Math & Conversions (FN-MATH)

- [ ] **FN-MATH-01**: User can push the constant π (`PI`) onto the stack via keyboard and program; lift_enabled = Enable
- [ ] **FN-MATH-02**: User can convert polar coordinates to rectangular (`P→R`): Y = magnitude, X = angle (in current angle_mode) → Y = X-coord, X = Y-coord
- [ ] **FN-MATH-03**: User can convert rectangular coordinates to polar (`R→P`): Y = X-coord, X = Y-coord → Y = magnitude, X = angle (in current angle_mode)
- [ ] **FN-MATH-04**: User can round X (`RND`) to the precision specified by the current display mode (FIX/SCI/ENG digit count)
- [ ] **FN-MATH-05**: User can extract the fractional part (`FRC`) — complement of `INT`; FRC(-3.7) = -0.7
- [ ] **FN-MATH-06**: User can compute Y mod X (`MOD`) with HP-41 sign semantics: result = `Y - X * trunc(Y/X)` where `trunc` is truncate-toward-zero (matches HP-41C Owner's Manual + Free42 source). Result follows sign of Y (e.g., `7 MOD -3 = 1`; `-7 MOD 3 = -1`). Domain error on X = 0.
- [ ] **FN-MATH-07**: User can compute absolute value (`ABS`) of X
- [ ] **FN-MATH-08**: User can compute factorial (`FACT`) of integer 0–69; HpError::OutOfRange for X > 69 or non-integer
- [ ] **FN-MATH-09**: User can compute sign function (`SIGN`): -1 / 0 / +1 for negative / zero / positive X; SIGN("alpha") = 0 per HP-41 semantics
- [ ] **FN-STACK-01**: User can roll the stack up (`R↑`) — symmetric mirror of `Rdn`; T→Z→Y→X→T

### Flags & Display Control (FN-FLAG, FN-DISP)

- [ ] **FN-FLAG-01**: `CalcState` exposes 56 user flags (00–55) plus HP-41 system flags via `flags: u64` (or equivalent), `#[serde(default)]` for v1.x save compat
- [ ] **FN-FLAG-02**: User can set a flag (`SF n`), clear (`CF n`), test (`FS? n` / `FC? n`), and test-then-clear (`FS?C n` / `FC?C n`) for n=0..55; tests skip next program step on false
- [ ] **FN-DISP-01**: User can display register N (`VIEW nn`) until next keypress; preserves stack
- [ ] **FN-DISP-02**: User can display ALPHA register (`AVIEW`) until next keypress
- [ ] **FN-DISP-03**: User can prompt with ALPHA (`PROMPT`) — display ALPHA and pause running program until R/S
- [ ] **FN-DISP-04**: User can enable/disable ALPHA auto-display (`AON` / `AOFF`)
- [ ] **FN-DISP-05**: User can clear the display (`CLD`) without modifying stack or ALPHA

### Program Control & Sound (FN-PROG, FN-SOUND)

- [ ] **FN-PROG-01**: User can pause a running program (`STOP` — R/S inside program); execution resumes on next R/S press
- [ ] **FN-PROG-02**: User can insert a brief pause (`PSE`) — approx. 1 s display refresh, then program continues
- [ ] **FN-PROG-03**: User can clear a program by global label (`CLP label`) — removes from LBL to next END/.END.
- [ ] **FN-PROG-04**: User can delete N program steps from current PC (`DEL nnn`)
- [ ] **FN-PROG-05**: User can insert a blank step at current PC (`INS`)
- [ ] **FN-PROG-06**: User can branch indirectly via register (`GTO IND nn`) — branch target is register-N's contents
- [ ] **FN-PROG-07**: User can call subroutine indirectly via register (`XEQ IND nn`)
- [ ] **FN-SOUND-01**: User can emit the default beep (`BEEP`) — represented in `print_buffer` or new `event_buffer` (NO direct I/O in hp41-core, per existing invariant)
- [ ] **FN-SOUND-02**: User can emit a numbered tone (`TONE n`, n=0..9) — same buffer-based event channel

### ALPHA Operations & Indirect Addressing (FN-ALPHA, FN-IND)

- [ ] **FN-ALPHA-01**: User can append a register's value to ALPHA (`ARCL nn`) using current display format
- [ ] **FN-ALPHA-02**: User can store the first 6 ALPHA chars into a register as packed text (`ASTO nn`)
- [ ] **FN-ALPHA-03**: User can convert first ALPHA char to ASCII code in X (`ATOX`); X register holds character code
- [ ] **FN-ALPHA-04**: User can append ASCII code in X as ALPHA char (`XTOA`)
- [ ] **FN-ALPHA-05**: User can rotate ALPHA register by N chars (`AROT n`); negative N rotates right
- [ ] **FN-ALPHA-06**: User can search ALPHA for substring (`POSA`) — returns position in X, -1 if not found
- [ ] **FN-IND-01**: All addressable ops (`STO`, `RCL`, `ISG`, `DSE`, `SF`, `CF`, `FS?`, `FC?`, `FS?C`, `FC?C`, `STO+/-/×/÷`, `ARCL`, `ASTO`, `VIEW`) support indirect addressing (IND-Varianten) — register-N's integer part is the effective register/flag number
- [ ] **FN-IND-02**: Indirect resolution rejects non-integer register contents with `HpError::InvalidOp` (HP-41 hardware behavior)

### Memory & Catalog (FN-MEM)

- [ ] **FN-MEM-01**: User can query free memory (`SIZE nnn` sets register count; `MEM LOST` clears all on cold start)
- [ ] **FN-MEM-02**: User can clear ALPHA (`CLA`) — alias for existing `AlphaClear` with explicit op variant
- [ ] **FN-MEM-03**: User can clear the stack (`CLST`) — X=Y=Z=T=0, LASTX preserved
- [ ] **FN-MEM-04**: User can pack memory (`PACK`) — no-op in our flat-Vec program model, but op exists for compatibility
- [ ] **FN-MEM-05**: User can run catalog (`CATALOG n`, n=1..4) — list programs/registers/etc.; `print_buffer` output

### Key Assignment & Conditional Tests (FN-KEY, FN-TEST)

- [ ] **FN-KEY-01**: User can assign a function to a key (`ASN "name" key_code`) — already documented; needs `Op::Asn` variant and keyboard modal
- [ ] **FN-TEST-01**: All 12 conditional tests are keyboard-reachable at the GUI skin (today only `X≥Y` is): `X=Y`, `X≠Y`, `X<Y`, `X>Y`, `X≤Y`, `X≥Y`, `X=0`, `X≠0`, `X<0`, `X>0`, `X≤0`, `X≥0`

### CLI Integration (FN-CLI)

- [ ] **FN-CLI-01**: All new Op variants from FN-MATH/FLAG/DISP/PROG/SOUND/ALPHA/IND/MEM/KEY/TEST are wired in `hp41-cli/src/keys.rs::key_to_op` with explicit `KEY_REF_TABLE` entries
- [ ] **FN-CLI-02**: New `PendingInput` modal variants for `SF/CF/FS?/FC?`-Prompt, `VIEW`-Prompt, `TONE`-Prompt, `DEL`-Prompt, `CLP`-Label-Prompt, indirect variants
- [ ] **FN-CLI-03**: `help_data.rs::HELP_DATA` updated to include every new key binding with description
- [ ] **FN-CLI-04**: `hp41-cli/src/ui.rs::pending_prompt()` exhaustive match handles all new modal variants without `unreachable!()`

### Documentation (FN-DOC)

- [ ] **FN-DOC-01**: `docs/hp41cv-function-matrix.md` lists all ≈ 130 HP-41CV ROM ops with implementation status (✓ implemented in v2.x / ⏳ deferred to v3.x module / — N/A)
- [ ] **FN-DOC-02**: `CLAUDE.md` updated with v2.2 settled architecture decisions (flag storage, indirect resolution, sound buffer pattern)
- [ ] **FN-DOC-03**: `README.md` updated to reflect "feature-complete HP-41CV" claim with link to function matrix
- [ ] **FN-DOC-04**: `hp41-core` rustdoc cross-references stay accurate (linked from function matrix where helpful)

### GUI Integration (FN-GUI)

- [ ] **FN-GUI-01**: All new Op variants resolve via `hp41-gui/src-tauri/src/key_map.rs::resolve` for both bare ids and parameterized prefixes; the stub-error arm shrinks to only v3.x-module ops
- [ ] **FN-GUI-02**: `KEY_DEFS` in `hp41-gui/src/Keyboard.tsx` carries correct three-label bindings (primary / shifted / alphaChar) for every new function reachable from the original HP-41C keyboard
- [ ] **FN-GUI-03**: Modal-routing wired for the previously-stubbed prompt IDs (`sto_prompt`, `rcl_prompt`, `fix_prompt`, `sci_prompt`, `eng_prompt`, `isg_prompt`, `sf_prompt`, `cf_prompt`, `fs_prompt`, `x_eq_y_prompt`, …) — no `unknown key` toasts for any HP-41CV built-in
- [ ] **FN-GUI-04**: Toast pattern preserved for actual v3.x-module ops only; no silent discards (D-07 invariant)
- [ ] **FN-GUI-05**: `CalcStateView` extended with `flags`, `display_override`, `event_buffer` fields if needed; JSON budget stays ≤ 500 bytes (relaxed from ≤300 to accommodate flags)

### GUI Polish (FN-POLISH — carried over from original v2.1 scope)

- [ ] **FN-POLISH-01** (SKIN-04): 14-segment SVG font for authentic HP-41 LCD rendering; replaces current CSS-text display
- [ ] **FN-POLISH-02** (SKIN-05): Keyboard shortcut overlay (port `?` help panel from CLI `help_data.rs`) accessible via `?` key
- [ ] **FN-POLISH-03** (PROG-02): Full keyboard assignment display in USER mode — overlay current `Op::Asn` mappings onto the skin
- [ ] **FN-POLISH-04** (PROG-03): `prgm_mode` binding for 'p' key (currently mapped to `prx`); resolve the v2.0 deferred shortcut conflict

### Test Hardening (FN-QUAL)

- [x] **FN-QUAL-01**: `hp41-core` line coverage ≥ 95 % (recover from current 92.5 %); enforced by `just coverage` gate ✅ 95.25 % lines / 93.75 % regions measured 2026-05-15 (Phase 27-01)
- [x] **FN-QUAL-02**: Numerical accuracy suite (`hp41-core/tests/numerical_accuracy.rs`) extended with cases for every new math/conversion op (PI, P→R, R→P, RND, FRC, MOD, FACT); ≥ 98 % pass rate maintained ✅ 561/566 = 99.1 % combined; 27 Free42/manual citations (Phase 27-01 hand + 27-02 shape)
- [x] **FN-QUAL-03**: Flag-semantics property tests (proptest) — set/clear/test invariants across all 56 user flags ✅ 14 properties × 1024 cases in `proptest_flags.rs`; IND-resolved equivalence + conditional-skip sentinel covered (Phase 27-02)
- [x] **FN-QUAL-04**: Indirect-addressing integration tests — every IND op resolves correctly and rejects non-integer with `HpError::InvalidOp` ✅ 42 tests in `indirect_addressing.rs` covering all 17 _IND ops (happy + non-integer reject); skip-semantic ops drive through `run_program` (Phase 27-03)
- [x] **FN-QUAL-05**: GUI E2E smoke test via WebdriverIO + tauri-driver (D-27.15 AMENDED — replaces Playwright) in `ci-gui.yml` — boots the app, clicks `2 ENTER 3 +`, asserts the LCD reads `5.0000` ✅ `e2e-linux` job green on Ubuntu CI 2026-05-15 (Phase 27-04)

---

## Future Requirements (deferred)

None deferred for v2.x. Module-Pacs explicitly out of scope (→ v3.x).

---

## Out of Scope (v2.x — permanent exclusions until v3.x)

- **Math 1 Pac** (matrix ops, complex numbers, polynomial solver, integration) — v3.x
- **Stat 1 Pac** (extended statistics beyond Σ-registers) — v3.x
- **Time Pac** (date arithmetic, alarms, stopwatch) — v3.x
- **Advantage Pac** (financial functions, advanced regression) — v3.x
- **HP-IL peripheral emulation** (loop interface, mass storage, plotter) — permanent
- **Wand / barcode reader** — permanent
- **Cycle-accurate Nut CPU simulation** — permanent (behavioral emulation only)
- **HP-copyrighted ROM byte redistribution** — permanent (legal)
- **Cloud sync / network features** — permanent (privacy-first design)

---

## Traceability (filled by roadmap)

| REQ-ID | Phase | Plan |
|--------|-------|------|
| FN-MATH-01 | Phase 20 — Core Math & Conversions | 20-01-PLAN.md |
| FN-MATH-02 | Phase 20 — Core Math & Conversions | 20-01-PLAN.md |
| FN-MATH-03 | Phase 20 — Core Math & Conversions | 20-01-PLAN.md |
| FN-MATH-04 | Phase 20 — Core Math & Conversions | 20-01-PLAN.md |
| FN-MATH-05 | Phase 20 — Core Math & Conversions | 20-01-PLAN.md |
| FN-MATH-06 | Phase 20 — Core Math & Conversions | 20-01-PLAN.md |
| FN-MATH-07 | Phase 20 — Core Math & Conversions | 20-01-PLAN.md |
| FN-MATH-08 | Phase 20 — Core Math & Conversions | 20-01-PLAN.md |
| FN-MATH-09 | Phase 20 — Core Math & Conversions | 20-01-PLAN.md |
| FN-STACK-01 | Phase 20 — Core Math & Conversions | 20-01-PLAN.md |
| FN-FLAG-01 | Phase 21 — Flags, Display Control & Sound | TBD |
| FN-FLAG-02 | Phase 21 — Flags, Display Control & Sound | TBD |
| FN-DISP-01 | Phase 21 — Flags, Display Control & Sound | TBD |
| FN-DISP-02 | Phase 21 — Flags, Display Control & Sound | TBD |
| FN-DISP-03 | Phase 21 — Flags, Display Control & Sound | TBD |
| FN-DISP-04 | Phase 21 — Flags, Display Control & Sound | TBD |
| FN-DISP-05 | Phase 21 — Flags, Display Control & Sound | TBD |
| FN-SOUND-01 | Phase 21 — Flags, Display Control & Sound | TBD |
| FN-SOUND-02 | Phase 21 — Flags, Display Control & Sound | TBD |
| FN-PROG-01 | Phase 22 — Program Control & Memory Ops | TBD |
| FN-PROG-02 | Phase 22 — Program Control & Memory Ops | TBD |
| FN-PROG-03 | Phase 22 — Program Control & Memory Ops | TBD |
| FN-PROG-04 | Phase 22 — Program Control & Memory Ops | TBD |
| FN-PROG-05 | Phase 22 — Program Control & Memory Ops | TBD |
| FN-PROG-06 | Phase 22 — Program Control & Memory Ops | TBD |
| FN-PROG-07 | Phase 22 — Program Control & Memory Ops | TBD |
| FN-MEM-01 | Phase 22 — Program Control & Memory Ops | TBD |
| FN-MEM-02 | Phase 22 — Program Control & Memory Ops | TBD |
| FN-MEM-03 | Phase 22 — Program Control & Memory Ops | TBD |
| FN-MEM-04 | Phase 22 — Program Control & Memory Ops | TBD |
| FN-MEM-05 | Phase 22 — Program Control & Memory Ops | TBD |
| FN-KEY-01 | Phase 22 — Program Control & Memory Ops | TBD |
| FN-ALPHA-01 | Phase 23 — ALPHA Operations | TBD |
| FN-ALPHA-02 | Phase 23 — ALPHA Operations | TBD |
| FN-ALPHA-03 | Phase 23 — ALPHA Operations | TBD |
| FN-ALPHA-04 | Phase 23 — ALPHA Operations | TBD |
| FN-ALPHA-05 | Phase 23 — ALPHA Operations | TBD |
| FN-ALPHA-06 | Phase 23 — ALPHA Operations | TBD |
| FN-IND-01 | Phase 24 — Indirect Addressing (Cross-Cutting) | TBD |
| FN-IND-02 | Phase 24 — Indirect Addressing (Cross-Cutting) | TBD |
| FN-TEST-01 | Phase 25 — CLI Integration & Documentation | TBD |
| FN-CLI-01 | Phase 25 — CLI Integration & Documentation | TBD |
| FN-CLI-02 | Phase 25 — CLI Integration & Documentation | TBD |
| FN-CLI-03 | Phase 25 — CLI Integration & Documentation | TBD |
| FN-CLI-04 | Phase 25 — CLI Integration & Documentation | TBD |
| FN-DOC-01 | Phase 25 — CLI Integration & Documentation | TBD |
| FN-DOC-02 | Phase 25 — CLI Integration & Documentation | TBD |
| FN-DOC-03 | Phase 25 — CLI Integration & Documentation | TBD |
| FN-DOC-04 | Phase 25 — CLI Integration & Documentation | TBD |
| FN-GUI-01 | Phase 26 — GUI Integration & Polish | 26-01-modal-architecture-and-key-wiring-PLAN.md |
| FN-GUI-02 | Phase 26 — GUI Integration & Polish | 26-01-modal-architecture-and-key-wiring-PLAN.md |
| FN-GUI-03 | Phase 26 — GUI Integration & Polish | 26-03-polish-bundle-PLAN.md |
| FN-GUI-04 | Phase 26 — GUI Integration & Polish | 26-03-polish-bundle-PLAN.md |
| FN-GUI-05 | Phase 26 — GUI Integration & Polish | 26-01-modal-architecture-and-key-wiring-PLAN.md |
| FN-POLISH-01 | Phase 26 — GUI Integration & Polish | 26-02-14-seg-lcd-PLAN.md |
| FN-POLISH-02 | Phase 26 — GUI Integration & Polish | 26-03-polish-bundle-PLAN.md |
| FN-POLISH-03 | Phase 26 — GUI Integration & Polish | 26-03-polish-bundle-PLAN.md |
| FN-POLISH-04 | Phase 26 — GUI Integration & Polish | 26-03-polish-bundle-PLAN.md |
| FN-QUAL-01 | Phase 27-01 — Coverage push | ✅ 2026-05-15 (95.25 % lines / 93.75 % regions; gate 80→95 ratchet) |
| FN-QUAL-02 | Phase 27-01 (hand) + 27-02 (shape) | ✅ 2026-05-15 (561/566 = 99.1 %; 27 Free42 citations) |
| FN-QUAL-03 | Phase 27-02 — Proptest flags | ✅ 2026-05-15 (14 props × 1024 cases) |
| FN-QUAL-04 | Phase 27-03 — IND integration | ✅ 2026-05-15 (42 tests, all 17 _IND ops) |
| FN-QUAL-05 | Phase 27-04 — WebdriverIO E2E | ✅ 2026-05-15 (e2e-linux green on Ubuntu CI; D-27.15 AMENDED) |

**Coverage:** 63 / 63 requirements mapped ✓
