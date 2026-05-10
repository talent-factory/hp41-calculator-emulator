---
gsd_state_version: 1.0
milestone: v2.0
milestone_name: Tauri GUI
current_phase: 16
current_plan: "16-00"
status: phase_complete
last_updated: "2026-05-10T00:00:00.000Z"
progress:
  total_phases: 6
  completed_phases: 3
  total_plans: 10
  completed_plans: 7
  percent: 50
---

# Project State: HP-41 Calculator Emulator

---

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-08)

**Core value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.
**Shipped:** v1.0 CLI (2026-05-08)
**Shipped:** v1.1 CLI Feature Completeness (2026-05-09) — Phases 9–12 complete
**Current focus:** v2.0 Tauri GUI — planning phase
**Repo:** hp41-calculator-emulator
**Architecture:** Cargo workspace — `hp41-core` (library) + `hp41-cli` (binary); `hp41-core` has zero UI/CLI dependencies enforced at compile time.

---

## Current Position

Phase: 16 — SVG Skin
Plan: 16-00 READY TO PLAN — Phase 15 complete
Status: Phase 15 complete (verified 2026-05-10, 5/5 SC); Phase 16 ready to discuss/plan
Last activity: 2026-05-10 — Phase 15 Display & Keyboard verified 5/5 SC; advancing to Phase 16

Progress: [████████████████] 50% (3/6 phases complete — Phase 15 done, Phase 16 next)

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

---

## Session Continuity

**Last active:** 2026-05-10
**Last action:** Phase 16 context gathered — 4 areas discussed, 13 decisions locked (SVG authoring, key labels, component integration, click handler pattern)
**Next action:** `/gsd-plan-phase 16` — plan Phase 16 SVG Skin

---
*State initialized: 2026-05-06*
*Last updated: 2026-05-10 — Phase 16 context gathered*
