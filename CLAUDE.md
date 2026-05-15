# HP-41 Calculator Emulator — Project Guide

## What this is

A faithful Rust-based behavioral emulation of the HP-41C/CV/CX programmable RPN calculator.
- `hp41-core` — UI-agnostic library crate; zero CLI/UI dependencies enforced by Cargo workspace
- `hp41-cli` — TUI binary (ratatui 0.30 + crossterm 0.29)
- `hp41-gui` — Tauri v2 + React + TypeScript desktop app (nested standalone workspace)

**Core invariant:** `hp41-core` must never depend on `hp41-cli` or `hp41-gui`. Enforced at compile time. Root `Cargo.toml` members stay `["hp41-core", "hp41-cli"]`; `hp41-gui` is a nested standalone workspace.

**Status:**
- v1.0 CLI shipped 2026-05-08 — 8 phases, 45 plans
- v1.1 CLI Feature Completeness shipped 2026-05-09 — Phases 9–12, 13 plans (EEX-fix, STO arithmetic modals, print emulation, synthetic programming)
- v2.0 Tauri GUI shipped 2026-05-10 — Phases 13–18, 19 plans (pixel-perfect HP-41C desktop app)
- v2.1 Keyboard Authenticity (in progress) — Phase 19, 10 tasks (5-col layout, one-shot SHIFT, three-label keys, run_stop command, stub-error pattern)

## Git Workflow

**Commits:** Always use `/git-workflow:commit --with-skills` — never commit directly via `git commit`.

**Commit language: English only.** All commit messages (subject line and body) must be written in English, regardless of any global or plugin defaults that specify another language.

## GSD Workflow

Planning artifacts live in `.planning/`. v1.0 + v1.1 + v2.0 are shipped and archived under `.planning/milestones/`. Next milestone: v2.1 Polish.

```
/gsd-progress           — check current status
/gsd-new-milestone      — start v2.1 planning
```

**Phase history:**
- v1.0 (1–8): Foundation → Core Math → Programming Engine → TUI & Input → Persistence & UX → Science & Engineering → Hardening → Tech Debt Cleanup
- v1.1 (9–12): Infrastructure & EEX Fix → STO Arithmetic Modals → Print Emulation → Synthetic Programming
- v2.0 (13–18): Workspace Skeleton → IPC Layer → Display & Keyboard → SVG Skin → Persistence & Print Output → Program Listing & CI/CD
- v2.1 (19): Keyboard Authenticity

## Settled Architecture Decisions

These decisions are final — do not revisit without strong justification:

### Core engine (v1.0)

- **BCD/f64:** `rust_decimal` wrapping f64 with 10-significant-digit rounding. Custom BCD was evaluated and rejected. `HpNum` in `hp41-core/src/num.rs`.
- **Stack-lift:** `lift_enabled: bool` in `Stack`. Every one of ~130 operations declares `LiftEffect::Enable / Disable / Neutral` in `ops/`. The most commonly mis-implemented HP-41 feature — always check.
- **ISG/DSE counter:** Fields extracted by string-splitting at the decimal point — **never** `floor()`/`fmod()` on f64. See `ops/program.rs::parse_counter()`.
- **TUI:** Always use `ratatui::init()` (not `Terminal::new()`) to install the panic hook. Filter `KeyEventKind::Release` on Windows immediately or every op fires twice.
- **No async in core:** Event loop is `poll(timeout) → update → redraw`, single-threaded throughout. The hp41-gui spawns a separate auto-save thread but `hp41-core` itself stays single-threaded.
- **Zero panics in `hp41-core`:** `#![deny(clippy::unwrap_used)]` is active at the crate root (`hp41-core/src/lib.rs`). All production code must use `.expect("reason")` or proper `?`-propagation. Test modules carry `#[allow(clippy::unwrap_used)]`. Mutex locks in `hp41-gui` use `.unwrap_or_else(|e| e.into_inner())` for poisoned-lock recovery.
- **Key bindings (Phase 8):** `'q'` → `Op::Sin`, `'g'` → `Op::Clreg`, `Delete` in ALPHA mode → `Op::AlphaClear`. `'S'` opens STO register modal (handled before `key_to_op()`). Quit is `Ctrl+C` only.
- **Coverage gate:** `just coverage` runs `cargo llvm-cov clean --workspace` first to discard stale `.profraw` data from worktree runs before measuring.

### v1.1 additions

- **EEX trailing-e (hardware-faithful):** `flush_entry_buf()` appends `"00"` before the parse chain (`Decimal::from_str` → `Decimal::from_scientific`); empty-buffer EEX inserts implicit mantissa `"1"`; `format_entry_buf_display()` in `hp41-cli/src/ui.rs` renders the underscore placeholder cursor. **Never** discard a trailing-e number silently.
- **STO arithmetic modal:** 3-step keyboard flow `S → +/−/×/÷ → R00–R99 | Y/Z/T/L`. `StackReg` enum + `Op::StoArithStack` variant in `ops/mod.rs`; `op_sto_arith_stack()` in `registers.rs`. `pending_input` routing block must remain ABOVE modal-opening interceptors (`S`/`R`/`Ctrl+A`) so an active modal is not silently discarded.
- **Print emulation:** `print_buffer: Vec<String>` field on `CalcState` with `#[serde(skip)]` (transient, never persisted). `ops/print.rs::{PRX, PRA, PRSTK}` push lines into the buffer; `println!`/`eprintln!` are forbidden inside `hp41-core`. `hp41-cli` drains via `call_dispatch_and_drain()` (interactive) and `drain_and_show_print_output()` (programmatic `run_program` paths — wire ALL `run_program()` call sites or print output gets dropped).
- **Synthetic programming:** `last_key_code`, `reg_m`, `reg_n`, `reg_o` fields on `CalcState`, all with `#[serde(default)]` for backward-compat with v1.0 save files. `HexModal(String)` 2-digit accumulation modal; `synthetic_byte_to_op()` validates against the 23-entry safe subset **before** `state.program.insert()` (security invariant T-12-W2-02). `keycode_to_hp41_code()` in `hp41-cli/src/keys.rs` uses row×10+col encoding. F5 / R / S code paths reset `last_key_code` to 0 BEFORE GETKEY runs.
- **MSRV:** declared at `[workspace.package]` (`rust-version = "1.88"`); member crates inherit via `rust-version.workspace = true`. CI MSRV job runs in parallel — no `needs:`.

### v2.0 additions (Tauri GUI)

- **Nested workspace isolation:** Root `Cargo.toml` `members = ["hp41-core", "hp41-cli"]` — never add `hp41-gui` here. `tauri` / `tauri-build` must appear ONLY in `hp41-gui/src-tauri/Cargo.toml`, never in root `[workspace.dependencies]`.
- **Bundle identifier:** `ch.talent-factory.hp41` (overrides scaffold default `com.tauri.dev`; avoids macOS sandbox/keychain issues).
- **IPC contract:** `dispatch_op(key_id: &str)` and `get_state()` Tauri v2 commands; response is `CalcStateView` (~170 bytes, JSON budget ≤300 bytes). `key_map::resolve()` in `hp41-gui/src-tauri/src/key_map.rs` maps string IDs to `Op` variants — frontend never touches Rust enums. `print_buffer` is drained on every command response.
- **SC-4 invariant (no core duplication):** the spirit is "no calculator/math logic duplicated in hp41-gui". The literal grep `grep -rn "fn op_\|fn flush_entry\|fn format_hpnum" hp41-gui/src-tauri/src/` currently matches `fn op_display_name(...)` in `prgm_display.rs` — that function is a display formatter, not calculator logic, so the spirit is preserved. When checking SC-4 manually, use the stricter pattern `grep -rn "fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)" hp41-gui/src-tauri/src/` which excludes display helpers. Note: `op_display_name` is duplicated in both `hp41-cli/src/prgm_display.rs` and `hp41-gui/src-tauri/src/prgm_display.rs` — every new `Op` variant must be added in both copies.
- **Tauri v2.11 permissions:** For inline app commands (not plugins), Tauri does NOT auto-generate `allow-<cmd>` permissions. Create TOML in `hp41-gui/src-tauri/permissions/<cmd-kebab>.toml` with `[[permission]] identifier + commands.allow = ["fn_name"]`, then reference the kebab-case ID in `capabilities/default.json`. Run a `cargo check` first so the permission registry is generated.
- **SVG animation:** `transform-box: fill-box` + `transform-origin: center` on `.key` is REQUIRED for SVG `scale()` to animate from each key's own center; without it, keys translate from the canvas origin instead of shrinking in place.
- **busyRef debounce:** `useRef(false)` pattern in both `App.tsx` (handleClick) and `Keyboard.tsx` (handleKeyClick) — two-layer guard against concurrent `invoke()` calls. Always pair with `pressedKey` state machine using functional setState form to avoid stale closure (Pitfall 4).
- **Persistence sharing:** `hp41-gui` reads/writes the SAME `~/.hp41/autosave.json` file as `hp41-cli`. `serde(default)` on every `CalcState` field added since v1.0 keeps v1.x save files loadable. Auto-save thread releases the `AppState` Mutex BEFORE disk I/O (commit ff39017 fix).
- **`Op` variants land before TUI code:** Every new `Op` variant must appear in BOTH `dispatch()` in `ops/mod.rs` AND `execute_op()` in `ops/program.rs` AND the `prgm_display.rs` exhaustive match before any caller (`hp41-cli` or `hp41-gui`) can compile.

### v2.1 additions (Keyboard authenticity, Phase 19)

- **Authentic 5×8 layout**: `hp41-gui/src/Keyboard.tsx` renders 4 top-row mode buttons + a 5-column × 8-row main grid (ENTER 2-wide) with one orange SHIFT key — replaces the prior 8-col landscape layout with cosmetic `f`/`g` keys. Total key count: 4 top-row + 35 main grid = 39 entries.
- **Three-label `KeyDef`**: each key carries `id`/`label` (primary), optional `shifted: { id, label }` (orange, above), and `alphaChar` (blue, below). Old `fShiftLabel` field is gone. `KeyDef` is exported from `Keyboard.tsx`.
- **One-shot SHIFT is frontend-only**: `shiftActive: boolean` lives in `App.tsx`; never appears in `CalcState`, `CalcStateView`, or IPC. After a shifted op fires, `setShiftActive(false)` resets. `Tab` and clicking SHIFT toggle; `Esc` cancels. SHIFT joins the annunciator list as a frontend-derived value.
- **Click resolution priority**: if the clicked key is SHIFT (`variant: 'shift'`), toggle `shiftActive` and return without dispatching. Otherwise resolve the effective id by `ALPHA + alphaChar` → `shiftActive + shifted.id` (consumes the one-shot) → `primary.id`. ALPHA mode overrides SHIFT (known divergence from real HP-41 — v2.2 deferred).
- **Stub-error pattern** (D-5): `key_map::resolve` returns `Err(GuiError { message: "'<id>' is planned for a future phase" })` for `pi`, `polar_to_rect`, `rect_to_polar`, `beep`, `asn`, `catalog`, `view`. Also stubs `xeq_prompt`, `gto_prompt`, `lbl_prompt` explicitly — these would otherwise be silently swallowed by the label-bearing `xeq_`/`gto_`/`lbl_` prefixes in `resolve_parameterized`. Frontend surfaces a 2s toast overlay. NEVER silently discard — D-07 holds.
- **Modal-prompt ids** (`sto_prompt`, `rcl_prompt`, `fix_prompt`, `sci_prompt`, `eng_prompt`, `isg_prompt`, `sf_prompt`, `cf_prompt`, `fs_prompt`, `x_eq_y_prompt`, `x_le_y_prompt`, `x_gt_y_prompt`, `x_eq_0_prompt`) are KEY_DEFS-only frontend ids; they fall through to `resolve_parameterized` and fail (numeric-suffix parse rejects `"prompt"`), surfacing as `unknown key: <id>` toast. v2.2 will route these to actual modals.
- **`run_stop` Tauri command**: new dedicated command symmetric with `sst_step`/`bst_step`, toggles `CalcState.is_running`. R/S key is now click-reachable for the first time (was `id: ''` in v2.0). Permission file: `permissions/run-stop.toml`. Frontend special-routes id `r_s` via the `invokeForKey` helper, NOT through `dispatch_op`. v2.1 scope is flag-toggle only; actual stepping deferred to v2.2.
- **`invokeForKey` + `extractErrMessage` helpers** in `App.tsx`: single source of truth for resolving an effective id to a Tauri command (`sst_step`/`bst_step`/`run_stop`/`dispatch_op`) and for extracting the `GuiError.message` field from Tauri rejections (`String(err)` would produce `"[object Object]"` for object-shaped errors).
- **`SST`/`BST`/`CL X/A` special routes**: `App.tsx`'s click handler routes these ids to dedicated paths — `sst`/`bst` → `sst_step`/`bst_step` (via `invokeForKey`), `clx_or_a` → `clx` or `alpha_clear` depending on `annunciators.alpha`. Adding a new such key in the future requires updating either `invokeForKey` (single-id route) or the `clx_or_a` branch in `handleClick` (alpha-aware route).
- **Toast overlay**: `App.tsx` renders `<div className="toast" role="status">{toastMsg}</div>` when `toastMsg` is set, with a 2s auto-dismiss `useEffect`. CSS lives in `App.css` with `.toast` + `@keyframes toast-fade`. Single-toast policy (newest replaces older — no queue).
- **No core/CLI changes**: Phase 19 is hp41-gui only. SC-4 invariant preserved. Save-file backward compat unchanged.

### v2.2 additions (HP-41CV Feature Completeness, Phases 20–25)

- **Phase 20–24 ROM ops landed in `hp41-core`** — ~90 new `Op` variants spanning unary math + polar conversions + RND/FRC/MOD/PI (Phase 20), flags/display-control/sound (Phase 21), program-control + memory + catalog + ASN (Phase 22), ALPHA ops (Phase 23), and the 11-variant `*Ind` indirect family (Phase 24). `hp41-core` was FROZEN from Plan 25-01 onward; the surgical 4→12 `builtin_card_op` extension in Plan 25-03 is the documented exception.
- **f-prefix one-shot model on `hp41-cli`** (D-25.1 / D-25.4): `App.shift_armed: bool` mirrors hp41-gui v2.1's `shiftActive` bit-for-bit per the D-25.6 CLI ↔ GUI parity invariant. ALPHA overrides f-prefix (D-25.5); one-shot consumption always clears `shift_armed` regardless of whether the resolver matched (Pitfall 5). Status bar prepends `f→` when armed.
- **Four conditional tests on f-arith keys** (D-25.7, hardware ground truth from the user's physical HP-41CV): `f -`/`f +`/`f *`/`f /` dispatch `Op::Test(TestKind::{XEqY,XLeY,XGtY,XEqZero})`. These four are the ONLY conditional tests on the physical keyboard; the remaining 8 ROM conditionals (X≠Y, X<Y, X≥Y, X≠0, X<0, X>0, X≤0, X≥0) route through the XEQ-by-Name modal (D-25.8 / D-25.9).
- **Hybrid `PendingInput` struct-variants** (D-25.11): `FlagPrompt { kind: FlagPromptKind, ind: bool, acc: String }` and `RegisterPrompt { op: RegisterOpKind, ind: bool, acc: String }` collapse 34 logical operations (12 flag × {direct, IND} + 22 register × {direct, IND}) into 2 carrier variants. `FlagPromptKind` and `RegisterOpKind` wrap (not duplicate) `hp41_core::ops::FlagTestKind` / `StoArithKind` per D-25.13. `pending_prompt()` stays exhaustive over all 18 variants (no `_ =>`, no `unreachable!()`) — FN-CLI-04 compile-time guarantee.
- **IND-toggle via shift-0 inside an open modal** (D-25.12 / Pitfall 10): hardware-faithful per HP-41C/CV QRG p.14. The `App.shift_armed` bit is REUSED — pressing `f` arms shift inside a `FlagPrompt`/`RegisterPrompt`, then pressing `0` toggles the modal's `ind` field (not appended to the accumulator). End-of-2-digit dispatch is a single tuple-match decision point that picks `Op::*Ind(n)` vs `Op::*(n)` per D-25.12.
- **JSON-canonical data flow** (D-25.16): `docs/hp41cv-functions.json` is the single source of truth. `hp41-cli/src/help_data.rs` loads it via `include_str!` + `std::sync::OnceLock` (Phase 25-04 / Plan-04). Malformed JSON panics with `.expect("hp41cv-functions.json is malformed")` per D-25.17 (hard-build-blocker by design; Pitfall 7 mitigated by the smoke test). `scripts/docs-matrix/` (standalone non-workspace crate) regenerates `docs/hp41cv-function-matrix.md` via `just docs-matrix`; `just docs-matrix-check` is the CI drift-catch (Pitfall 8). Bidirectional Op-enum ↔ JSON parity is asserted by `hp41-cli/tests/function_matrix_parity.rs` (4 tests per Pitfall 6); key-coverage closure is `hp41-cli/tests/key_coverage.rs` per D-25.18.
- **`builtin_card_op` 4→12 extension** (D-25.8, Plan 25-03 surgical exception): the resolver in `hp41-core/src/ops/program.rs` grew from 4 v2.1 card-reader names to 4 + 8 conditional-test mnemonics (with ASCII + Unicode spellings). Visibility stays `pub(super) fn` per the W1 fix — no API widening. Mnemonic dispatch from both the keyboard (`xeq_by_name_local_resolve` fast-path) and programmatic `Op::Xeq("X<>Y?")` inside a saved program resolves identically (must_have truth #4 of Plan 03).
- **`KEY_REF_TABLE` is JSON-derived** (D-25.18): the right-panel discoverability listing in `hp41-cli/src/ui.rs::render_right_panel` reads from `help_data::help_entries()` filtered by non-null `key_path` — no parallel hand-curated table. Plan 25-04 deletes the legacy `pub const KEY_REF_TABLE` and replaces it with a derivation. Drift between bindings and discovery is caught by `key_coverage.rs` (every implemented JSON entry with `key_path != null` dispatches via `key_to_op` / `shifted_key_to_op` / modal-opener / `xeq_by_name_local_resolve` to a known `Op::` variant — no `InvalidOp`, no panics).
- **README soft-claim** (D-25.17): "feature-complete HP-41CV with documented divergences" + link to `docs/hp41cv-function-matrix.md`. Hard claim is deferred to Phase 27 conditional on coverage gate ≥95 % (FN-QUAL-01).
- **SC-4 invariant unchanged**: Phase 25 touches `hp41-cli`, `docs/`, `scripts/`, root `*.md`, and `justfile` only — NO `hp41-gui` changes (those land in Phase 26). The CLI ↔ GUI parity invariant D-25.6 is the contract Phase 26 must satisfy.
- **Save-file backward compat preserved**: NO new `CalcState` fields in Phase 25. v1.0–v2.1 save files continue to load without migration.

### v2.2 additions (Test Hardening, Phase 27)

- **Coverage gate raise (FN-QUAL-01):** `just coverage` enforces ≥ 95 % line coverage on `hp41-core` (raised atomically from 80 % per D-27.2 — gate-and-test atomicity invariant). The 5 new test files (`program_execution_coverage.rs`, `phase22_stats_size_shrink.rs`, `phase21_phase22_interactive_no_ops.rs`, `format_eng_edges.rs`, `numerical_accuracy.rs` v2.2 extension) close the gap with risk-weighted tests catching real bug classes per D-27.1, not coverage padding per D-27.3. Final achieved coverage: 95.25 % lines / 93.75 % regions (baseline pre-Task-1: 93.59 % / 91.21 % per 2026-05-15 RESEARCH measurement). Largest single uplift: `ops/stats.rs` 84.04 % → 100 % lines via Pitfall-5 SIZE-shrink sentinels.
- **Numerical accuracy ≥ 98 % gate extended (FN-QUAL-02):** the 503-case v1.x baseline grew to 566 cases covering PI / P→R / R→P / RND / FRC / MOD / FACT per D-27.5. Quirky cases (FACT(70) → OutOfRange, MOD(7,-3) = 1 sign-follows-Y, FACT(0) = 1) carry Free42 / Owner's Manual citations per D-27.7 (27 total citations). The v1.x baseline non-regression floor is asserted independently: `baseline_passes >= 498` per D-27.6 (5 pre-existing HP-41 hardware-rounding divergences acceptable per the historical failure budget). Combined pass rate: 99.1 % (561/566).
- **E2E smoke via WebdriverIO + tauri-driver (FN-QUAL-05, D-27.15 AMENDED 2026-05-15):** the original D-27.15 named Playwright, but `tauri-driver` 2.0.6 speaks WebDriver classic which Playwright does NOT (CDP/native only). The spirit of D-27.15 (production binary + real IPC + Ubuntu only) is preserved via WebdriverIO 9.x — the Tauri v2 official E2E client. `hp41-gui/wdio.conf.js` spawns `tauri-driver` on `127.0.0.1:4444` with `framework: 'mocha'` and `mochaOpts.retries: 1` (D-27.16). `hp41-gui/e2e/smoke.spec.ts` clicks `2 ENTER 3 +` via `[data-key-id]` selectors and asserts `[data-testid="lcd-display"]` reads `5.0000` (literal ROADMAP scope per D-27.13 — no broader flows). Runs ONLY on Ubuntu in `.github/workflows/ci-gui.yml::e2e-linux` (ROADMAP cross-cutting line 205); macOS/Windows matrix jobs UNCHANGED. Apt deps added to both build job and e2e-linux job: `webkit2gtk-driver` (Pitfall 6) + `xvfb` (Assumption A5). Cargo bin cache (Pitfall 5) keyed on `tauri-driver-2.0.6` keeps cold installs rare. Required-for-merge is a repo branch-protection setting — manual follow-up tracked in 27-04-SUMMARY.md.
- **Vitest CI gating (D-27.14):** the 5 existing Vitest files (`App.test.tsx`, `Display14Seg.test.tsx`, `HelpOverlay.test.tsx`, `Keyboard.test.tsx`, `pending_input.test.ts`, 142 tests total) now gate on every CI push via `just gui-ci` appended `cd hp41-gui && npm test`. They pass locally since Phase 26 ship; the CI gate closes a quiet hole. `hp41-gui/vite.config.ts` `test.exclude` adds `e2e/**` so the WebdriverIO spec is not picked up by Vitest.
- **`data-testid="lcd-display"` on `Display14Seg.tsx`** (RESEARCH Pitfall 10): one-line edit (plus a `data-text={text}` fallback attribute) on the outermost `<svg>`. Allowed under SC-4 because `hp41-gui/src/` is OUTSIDE the SC-4 boundary (which constrains `hp41-gui/src-tauri/` only). The dual `data-text` is the assertion path the smoke prefers because the 14-segment LCD renders SVG path fills — no plain text content for a `toHaveText` assertion to read.
- **GUI coverage measured one-shot (D-27.4, measure-only — NOT a gate):** `cargo llvm-cov --manifest-path hp41-gui/src-tauri/Cargo.toml` reports 77.92 % lines / 78.57 % regions / 71.09 % functions on `hp41-gui/src-tauri/src/` (8 source files; lowest covered: `main.rs` 0 % and `lib.rs` 0 % — both Tauri boilerplate; highest: `types.rs` 97.97 %). Vitest line coverage on `hp41-gui/src/` is intentionally NOT measured this phase — D-27.4's no-devDep clause forbids adding `@vitest/coverage-v8`. Advisory snapshot for v3.x reference only; no CI gate, no devDep added.
- **Frozen invariants preserved:** no `hp41-core/src/` source changes (frozen since Plan 25-01); no `hp41-gui/src-tauri/` source changes (SC-4 invariant); MSRV 1.88 unchanged (`tauri-driver` 2.0.6 MSRV 1.77 is compatible; WebdriverIO 9.x is a Node tool); `#![deny(clippy::unwrap_used)]` continues to apply (new test files carry `#![allow]` at file scope per the established Phase 1 onward pattern).
- **`// Catches: <bug class>` rationale (D-27.1):** every new test in Phase 27 plans 01–04 carries this doc comment naming the bug class it guards against. The `case!` macro invocations in `numerical_accuracy.rs` carry equivalent provenance via Free42 / OM citations. A grep audit on the four new test files yields ≥ 80 `// Catches:` comments total (program_execution_coverage 42, phase22_stats_size_shrink 14, phase21_phase22_interactive_no_ops 12, format_eng_edges 24).

## Tech Stack

**Core / CLI (v1.0 + v1.1):**
- Rust stable, MSRV `1.88` (declared in `[workspace.package]`)
- **`just`** — sole task runner; all build/test/lint/run/ci targets are `just` recipes. **Never call `cargo` directly in CI or docs.** GUI recipes: `just gui-dev` / `just gui-build` / `just gui-ci` / `just gui-check`.
- `rust_decimal` 1.42 (HpNum BCD-accurate arithmetic)
- ratatui 0.30 + crossterm 0.29 (TUI)
- serde + serde_json (state persistence, human-readable JSON)
- proptest (property tests for stack invariants)
- cargo-llvm-cov (coverage gate: ≥95% on `hp41-core` — Phase 27 / FN-QUAL-01, atomic raise from 80% per D-27.2)
- criterion (dispatch benchmarks — advisory, not CI-gated)
- clap 4.x (CLI argument parsing)

**GUI (v2.0):**
- Tauri v2.11 (Rust desktop runtime — nested standalone workspace in `hp41-gui/src-tauri/`)
- React 18 + TypeScript + Vite (frontend in `hp41-gui/src/`)
- `dirs` crate (resolves `~/.hp41/autosave.json` shared with hp41-cli)
- Two-layer CI: `ci.yml` (CLI, unchanged) + `ci-gui.yml` (3-OS matrix, path-filtered to `hp41-gui/**` and `hp41-core/**`, runs `cargo test` before `cargo build --release`)

## Quality Gates (maintained across v1.0 → v2.2)

| Gate | Target | v1.0 | v1.1 / v2.0 | v2.2 (Phase 27) |
|------|--------|------|-------------|------------------|
| Cold-start | ≤ 0.5 s | 2.2 ms (M1) | unchanged (CLI); GUI cold-start not gated | unchanged |
| Key latency | ≤ 50 ms median | ~65 ns/op | unchanged | unchanged |
| Numerical accuracy | ≥ 98% (combined ~570 cases) | 99% (495/500) | unchanged | 99.1% (561/566); v1.x 503-case baseline floor 498 preserved per D-27.6 |
| `hp41-core` coverage | ≥ 95% (raised from 80% per D-27.2) | 94.87% | 92.5% lines / 89.9% regions (slipped from v1.0 high-water mark — Phase 12 / ops/mod.rs synthetic arms) | 95.25% lines / 93.75% regions (FN-QUAL-01 closed, Phase 27 risk-weighted push) |
| Panics in `hp41-core` | 0 | 0 | 0 | 0 |
| CI | Win 10+, macOS 12+, Ubuntu 22.04+ | ✅ `ci.yml` | ✅ `ci.yml` + `ci-gui.yml` (independent) | unchanged |
| MSRV | declared | — | 1.88 (CI-enforced) | 1.88 |

## Key Files

**Core engine:**

| File | Purpose |
|------|---------|
| `hp41-core/src/ops/mod.rs` | `Op` enum, `dispatch()`, `flush_entry_buf()` — central integration hub |
| `hp41-core/src/state.rs` | `CalcState` — single source of truth (incl. `print_buffer`, `last_key_code`, `reg_m/n/o`) |
| `hp41-core/src/stack.rs` | `Stack`, `apply_lift_effect()` |
| `hp41-core/src/ops/program.rs` | `run_program()`, `run_loop()`, `parse_counter()`, `execute_op()` — ISG/DSE logic |
| `hp41-core/src/ops/print.rs` | `op_prx()`, `op_pra()`, `op_prstk()` — buffer-only, NO `println!` |
| `hp41-core/src/ops/registers.rs` | `op_sto_arith()`, `op_sto_arith_stack()`, M/N/O hidden-register ops |
| `hp41-core/src/ops/mod.rs::synthetic_byte_to_op` | 24-entry safe-subset validator for Phase 12 HexModal insertion |
| `hp41-core/src/format.rs` | `format_hpnum()`, `format_alpha()` — display formatting shared by core/cli/gui |
| `hp41-core/tests/numerical_accuracy.rs` | 500-case accuracy suite — must stay ≥ 490 passing |

**TUI (`hp41-cli`):**

| File | Purpose |
|------|---------|
| `hp41-cli/src/app.rs` | `App`, `handle_key()`, `handle_alpha_mode_key()`, `PendingInput`, event loop, `call_dispatch_and_drain()`, `drain_and_show_print_output()` |
| `hp41-cli/src/keys.rs` | `key_to_op()`, `KEY_REF_TABLE`, `keycode_to_hp41_code()` |
| `hp41-cli/src/ui.rs` | `format_entry_buf_display()` — EEX placeholder cursor; `pending_prompt()` exhaustive match |
| `hp41-cli/src/help_data.rs` | `HELP_DATA` — SINGLE SOURCE OF TRUTH for key descriptions in `?` overlay |
| `hp41-cli/src/persistence.rs` | `save_state()`, `load_state()` — JSON serde |

**GUI (`hp41-gui`):**

| File | Purpose |
|------|---------|
| `hp41-gui/src-tauri/src/lib.rs` | `setup()`, `AppState = Mutex<CalcState>`, 30s auto-save thread, `generate_handler!` registration |
| `hp41-gui/src-tauri/src/commands.rs` | `dispatch_op`, `get_state`, `sst_step`, `bst_step`, `run_stop` Tauri thunks + `handle_op`/`handle_get_state` helpers. `run_stop` toggles `CalcState.is_running`; symmetric with sst_step/bst_step; reaches R/S key (v2.1). |
| `hp41-gui/src-tauri/src/types.rs` | `CalcStateView`, `Annunciators`, `GuiError`, `From<HpError>` |
| `hp41-gui/src-tauri/src/key_map.rs` | `resolve()` — string ID → `Op`; SC-4 invariant (no `op_*`/`flush_*`/`format_hpnum` here) |
| `hp41-gui/src-tauri/src/persistence.rs` | Shared `~/.hp41/autosave.json` (same schema as hp41-cli) |
| `hp41-gui/src-tauri/src/prgm_display.rs` | `format_all_steps()` — always appends END so `pc == program.len()` highlights correctly |
| `hp41-gui/src-tauri/permissions/*.toml` | Tauri v2.11 inline-command permission registry |
| `hp41-gui/src/App.tsx` | React root: display, annunciators, stack panel, keyboard listener, `busyRef`, `resolveKeyId`, program panel, `shiftActive` state machine, `invokeForKey`/`extractErrMessage` helpers, toast overlay. |
| `hp41-gui/src/Keyboard.tsx` | Authentic 5×8 grid + top-row band; `KEY_DEFS` with three-label model (primary + shifted + alphaChar); SHIFT key variant; `keyPosition` helper. |
| `hp41-gui/src/App.css` | Layout, key animation, program panel styles; requires `transform-box: fill-box` on `.key` |
| `hp41-gui/wdio.conf.js` | NEW (Phase 27) — WebdriverIO + tauri-driver smoke config; `framework: 'mocha'`; `mochaOpts.retries: 1` (D-27.16); Ubuntu-only via `ci-gui.yml::e2e-linux`. |
| `hp41-gui/e2e/smoke.spec.ts` | NEW (Phase 27) — FN-QUAL-05 literal ROADMAP smoke: `2 ENTER 3 +` → `[data-testid="lcd-display"]` reads `5.0000`. Excluded from Vitest (`vite.config.ts test.exclude`). |
