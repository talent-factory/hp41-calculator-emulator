---
gsd_state_version: 1.0
milestone: v2.2
milestone_name: — HP-41CV Feature Completeness
status: executing
last_updated: "2026-05-14T14:25:00Z"
last_activity: 2026-05-14 -- Phase 23 ALPHA Operations shipped (6/6 must-haves verified)
progress:
  total_phases: 8
  completed_phases: 4
  total_plans: 15
  completed_plans: 11
  percent: 50
---

# Project State: HP-41 Calculator Emulator

---

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-08)

**Core value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.
**Shipped:** v1.0 CLI (2026-05-08)
**Shipped:** v1.1 CLI Feature Completeness (2026-05-09) — Phases 9–12 complete
**Shipped:** v2.0 Tauri GUI (2026-05-10) — Phases 13–18 complete
**Shipped:** v2.1 Card Reader + Keyboard Authenticity (2026-05-13) — recorded as quick tasks, no Phase 19 GSD directory
**Current focus:** v2.2 HP-41CV Feature Completeness — roadmap complete (Phases 20–27); awaiting first phase planning
**Repo:** hp41-calculator-emulator
**Architecture:** Cargo workspace — `hp41-core` (library) + `hp41-cli` (binary) + `hp41-gui` (nested standalone Tauri workspace); `hp41-core` has zero UI/CLI dependencies enforced at compile time.

---

## Current Position

Phase: 24: Indirect Addressing (Cross-Cutting) — NOT STARTED
Plan: —
Status: Ready to discuss/plan
Resume file: —
Last activity: 2026-05-14 -- Phase 23 ALPHA Operations shipped: 6/6 must-haves verified, 2 plans, 50 new tests (28 inline + 22 integration), WR-01 (op_size text_regs pruning) and WR-02 (op_atox stack-helper routing) fixed in-cycle from code review

Progress: 4 / 8 phases (Phase 23 complete → Phase 24 next)

---

## Performance Metrics (v1.0 Shipped Values)

| Metric | Target | Achieved |
|--------|--------|---------|
| Cold-start latency | ≤ 0.5 s | 2.2 ms (M1) — 228× under gate |
| Key-press latency (median) | ≤ 50 ms | ~65 ns/op |
| `hp41-core` test coverage | ≥ 80% (v2.2 raises to ≥ 95%) | 92.68% lines (Phase 21 — non-regression vs. Phase 20 baseline 92.65%; v2.2 target ≥ 95% at Phase 27) |
| Numerical accuracy (500-case) | ≥ 98% | 500/500 (Phase 20 confirmed; up from 495/500 v2.1 baseline) |
| Panics in `hp41-core` | 0 | 0 — enforced by `#![deny(clippy::unwrap_used)]` |
| CI platforms | Win/macOS/Ubuntu | All green (`ci.yml` + `ci-gui.yml`) |

---

## v2.2 Phase Plan (Phases 20–27)

| Phase | Name | Requirements | Build Stage |
|-------|------|--------------|-------------|
| 20 | Core Math & Conversions | FN-MATH-01..09, FN-STACK-01 (10) | core |
| 21 | Flags, Display Control & Sound | FN-FLAG-01..02, FN-DISP-01..05, FN-SOUND-01..02 (9) | core |
| 22 | Program Control & Memory Ops | FN-PROG-01..07, FN-MEM-01..05, FN-KEY-01 (13) | core |
| 23 | ALPHA Operations | FN-ALPHA-01..06 (6) | core |
| 24 | Indirect Addressing (Cross-Cutting) | FN-IND-01..02 (2) | core |
| 25 | CLI Integration & Documentation | FN-TEST-01, FN-CLI-01..04, FN-DOC-01..04 (9) | cli + docs |
| 26 | GUI Integration & Polish | FN-GUI-01..05, FN-POLISH-01..04 (9) | gui |
| 27 | Test Hardening | FN-QUAL-01..05 (5) | tests |

**Total: 63 requirements across 8 phases — 100% coverage.**

---

## Accumulated Context

### Key Decisions

| Decision | Rationale | Phase |
|----------|-----------|-------|
| BCD vs f64 | `rust_decimal` with 10-digit rounding; avoid custom BCD struct | Phase 1 |
| Stack-lift as `lift_enabled: bool` | Every op declares Enable/Disable/Neutral | Phase 1 |
| `CalcState` as single source of truth | One `&mut CalcState` through all ops; no global state | Phase 1 |
| ISG/DSE string-split counter fields | Never `floor()`/`fmod()` on f64 | Phase 3 |
| `ratatui::init()` not `Terminal::new()` | Installs panic hook for terminal restore | Phase 4 |
| Digit entry via `entry_buf` | Auto-flushed on next non-digit | Phase 4 |
| `serde_json` for persistence | Human-readable, diff-able, versioned JSON | Phase 5 |
| No async in `hp41-core` | Single-threaded event loop | All |
| `#![deny(clippy::unwrap_used)]` | Compile-time zero-panic guarantee | Phase 7 |
| `print_buffer: Vec<String>` on CalcState | Keeps hp41-core I/O-free; hp41-cli drains buffer | Phase 11 |
| EEX trailing-e → append "00" | Hardware fidelity; `flush_entry_buf` normalizes before parse chain | Phase 9 |
| Empty-buffer EEX inserts "1e" | HP-41 hardware behavior; implicit mantissa | Phase 9 |
| `format_entry_buf_display` in ui.rs | TUI exponent placeholder rendering separated from `get_display_string` | Phase 9 |
| `pending_input` routing before modal interceptors | Prevents active dialogs being silently discarded by S/R/Ctrl+A | Phase 9 |
| `entry_buf` preserved on parse failure | Silent data loss fix; clear only on successful parse | Phase 9 |
| MSRV 1.85 with workspace inheritance | `rust-version.workspace = true` in member crates; CI job with llvm-tools | Phase 9 |
| CHS during EEX entry toggles exponent sign | 'n' in EEX mode mutates entry_buf in-place (no flush); "e-" normalized to "e-00" in flush_entry_buf | Quick 260508-y30 |
| Bundle identifier `ch.talent-factory.hp41` (D-02) | Overrides scaffold default `com.tauri.dev`; avoids macOS sandbox/keychain issues | Phase 13 |
| capabilities/default.json core:default only | Minimum Tauri v2 capability; hp41-specific IPC permissions added in Phase 14 when commands are registered | Phase 13 |
| Mutex lock: `.unwrap_or_else(\|e\| e.into_inner())` | Poisoned-lock recovery required by zero-panic policy; applies to all Phase 14+ command handlers | Phase 13 |
| Tauri v2.11 app-command permissions: TOML files required | For inline app commands (not plugins), Tauri v2.11 does NOT auto-generate allow-<cmd> permissions. Create TOML in src-tauri/permissions/<cmd-kebab>.toml with `[[permission]] identifier + commands.allow = ["fn_name"]` | Phase 14 |
| CalcStateView display_str priority: entry_buf → format_alpha(alpha_mode) → format_hpnum(stack.x) | Matches hp41-cli get_display_string logic; x_str always uses format_hpnum for Phase 15 stack panel | Phase 14 |
| EEX-CHS gap in handle_op | In-buffer exponent sign toggle (Op::Chs during EEX entry) is missing from commands.rs handle_op; deferred to Phase 15 keyboard wiring. Frontend must send "eex_chs" key ID | Phase 14 |
| KEY_DEFS has 44 entries, not 40 | HP-41C has 44 key positions (9+8+9+9+9 across 5 rows); ENTER is one entry with colSpan:2. Plan text said "40" in error; implementation follows the actual key list. | Phase 16 |
| SVG shadow: manual rect over filter | Shadow implemented as 1px-offset black rect (45% opacity) rather than SVG feDropShadow filter — simpler, no GPU compositing overhead, no per-element filter allocation | Phase 16 |
| transform-box: fill-box required for SVG animation | Without this CSS property, scale() transforms from SVG canvas origin rather than each key's own center — keys would translate instead of shrink in place | Phase 16 |

### Critical Implementation Traps (v2.2 — adapted for new milestone)

- **Every new Op variant must be added to 4 places (not 2!):** `dispatch()` in `ops/mod.rs` + `execute_op()` in `ops/program.rs` + `hp41-cli/src/prgm_display.rs` + `hp41-gui/src-tauri/src/prgm_display.rs`. The exhaustive matches will fail to compile if any of these is missed.
- **New CalcState fields need `#[serde(default)]`** for backward compatibility with v1.0/v1.1/v2.0/v2.1 save files. Critical for `flags`, `display_override`, `event_buffer`.
- **SC-4 invariant (no core duplication in hp41-gui):** Use stricter grep `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` — `op_display_name` is the only intentional exception.
- **No `println!`/`eprintln!` in hp41-core:** `BEEP`/`TONE` must route through a buffer (existing `print_buffer` or new `event_buffer`).
- **`pending_input` routing block must remain ABOVE modal-opening interceptors** (S/R/Ctrl+A) to prevent active dialogs being silently discarded.
- **D-07 (no silent discards) preserved in GUI:** Phase 26 modal routing replaces toast for HP-41CV builtins, but unhandled IDs still produce `GuiError` toast — never silent.

### Blockers

None.

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260508-y30 | CHS during EEX entry: toggle minus sign in exponent | 2026-05-08 | aa0904b | [260508-y30-eex-chs-exponent-sign-toggle](./quick/260508-y30-eex-chs-exponent-sign-toggle/) |
| 260508-06h | FIX/SCI/ENG digit-count modal via F key (0–9) | 2026-05-08 | 7ff792c | [260508-06h-fix-sci-eng-digit-input](./quick/260508-06h-fix-sci-eng-digit-input/) |
| 260513-v21a | v2.1 Card Reader: WDTA/RDTA/WPRGM/RDPRGM + XEQ-by-name + cards module + PR #9 fixes | 2026-05-13 | 72530dc…f4b3f8b | — (no GSD dir; see MILESTONES.md v2.1) |
| 260513-v21b | v2.1 Keyboard Authenticity: 5-col grid, three-label keys, one-shot SHIFT, run_stop Tauri cmd, stub-error pattern, toast overlay, PR #10 fixes | 2026-05-13 | 8cd2de4…ff56b97 | — (no GSD dir; see MILESTONES.md v2.1) |

---

## Session Continuity

**Last active:** 2026-05-14
**Last action:** `/gsd-execute-phase 23` complete — both plans shipped sequentially via worktree-isolated executors (23-01 merged at `7bb0daf`, 23-02 merged at `29133da`). 6 new Op variants (Arcl/Asto/Atox/Xtoa/Arot/Posa) landed in all 4 places (D-23.12); `text_regs: BTreeMap<u8, String>` field added to CalcState with `#[serde(default)]`; D-23.4 sidecar-clearing audit applied to op_sto/op_sto_arith/op_clreg AND op_size (WR-01 post-review fix). Op::Atox now routes through `crate::stack::enter_number` (WR-02 fix). 50 new tests (28 inline + 22 integration) — all green. `just test` + `just lint` clean. Verifier scored 6/6 must-haves. REVIEW.md → status `fixed`. VERIFICATION.md → status `passed`. Commits 57a75e1, 915e22c, 4b108b1, d12347a, 3349ec1, 737683a, 7284c88, f109740, 990d1bb, 4ba85c4, 51c9718.
**Next action:** `/gsd-discuss-phase 24` or `/gsd-plan-phase 24` — Phase 24 (Indirect Addressing) wires `_IND` variants on all addressable ops. Fresh `/clear` recommended.

---
*State initialized: 2026-05-06*
*Last updated: 2026-05-14 — Phase 23 ALPHA Operations shipped; ready to start Phase 24*
