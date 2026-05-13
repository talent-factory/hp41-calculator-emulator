---
gsd_state_version: 1.0
milestone: v2.2
milestone_name: HP-41CV Feature Completeness
current_phase: null
current_plan: null
status: awaiting-milestone-planning
last_updated: "2026-05-13T00:00:00.000Z"
progress:
  total_phases: 0
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
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
**Current focus:** v2.2 HP-41CV Feature Completeness — awaiting milestone planning (130-function ROM-built-in set; modules deferred to v3.x)
**Repo:** hp41-calculator-emulator
**Architecture:** Cargo workspace — `hp41-core` (library) + `hp41-cli` (binary) + `hp41-gui` (nested standalone Tauri workspace); `hp41-core` has zero UI/CLI dependencies enforced at compile time.

---

## Current Position

Phase: — (between milestones)
Plan: —
Status: v2.1 complete (2026-05-13); recorded as quick tasks (no Phase 19 directory). Awaiting `/gsd-new-milestone "v2.2 HP-41CV Feature Completeness"`.
Last activity: 2026-05-13 — STATE.md / MILESTONES.md / PROJECT.md reconciled with shipped v2.1 work (50 commits since v2.0 tag).

Progress: — (v2.2 not yet planned)

---

## Performance Metrics (v1.0 Shipped Values)

| Metric | Target | Achieved |
|--------|--------|---------|
| Cold-start latency | ≤ 0.5 s | 2.2 ms (M1) — 228× under gate |
| Key-press latency (median) | ≤ 50 ms | ~65 ns/op |
| `hp41-core` test coverage | ≥ 80% | 94.87% (Phase 9: 94.22%) |
| Numerical accuracy (500-case) | ≥ 98% | 99% (495/500) |
| Panics in `hp41-core` | 0 | 0 — enforced by `#![deny(clippy::unwrap_used)]` |
| CI platforms | Win/macOS/Ubuntu | All green |

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

### Critical Implementation Traps (v1.1)

- Every new Op variant must be added to BOTH `dispatch()` in `ops/mod.rs` AND `execute_op()` in `ops/program.rs`
- New CalcState fields need `#[serde(default)]` for backward compatibility with v1.0 save files
- STO arithmetic core (`op_sto_arith`) is already implemented in hp41-core — Phase 10 adds StackReg enum + Op::StoArithStack variant + op_sto_arith_stack() function (core) and TUI routing (cli)
- Phase 10 hp41-core changes: StackReg enum in ops/mod.rs, Op::StoArithStack variant, op_sto_arith_stack() in registers.rs, dispatch()/execute_op() arms
- `pending_input` routing block must remain ABOVE modal-opening interceptors (S/R/Ctrl+A) to prevent modal interruption

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

**Last active:** 2026-05-13
**Last action:** v2.1 (Card Reader + Keyboard Authenticity) reconciled into GSD state as two quick-task entries; no Phase 19 directory created. PROJECT.md + MILESTONES.md updated. Git tag `v2.1` not yet created.
**Next action:** `/gsd-new-milestone "v2.2 HP-41CV Feature Completeness"` — strict ROM built-in 130-function set; module emulation (Math/Stat/Time/Advantage Pacs) deferred to v3.x. Include the three v2.1 Polish items (14-seg font, ?-overlay, USER keyboard display) as a final GUI Polish phase of v2.2.

---
*State initialized: 2026-05-06*
*Last updated: 2026-05-13 — v2.1 reconciled (Card Reader + Keyboard Authenticity); awaiting v2.2 planning*
