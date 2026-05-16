# Research Summary: HP-41 Calculator Emulator v2.0 Tauri GUI

**Synthesized:** 2026-05-09
**Sources:** STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md, PROJECT.md
**Milestone:** v2.0 — `hp41-gui` Tauri v2 + React + TypeScript desktop app

---

## Overview

The v2.0 milestone adds a pixel-perfect graphical desktop calculator to a well-established Rust workspace. The architecture is deliberately thin: `hp41-gui` is a Tauri v2 adapter crate that wraps `hp41-core` (unchanged) behind a small set of Tauri commands and renders an SVG HP-41C skin in a React 19 + TypeScript frontend. All calculator logic stays in `hp41-core`. The GUI never implements math.

`CalcState` already derives `Serialize + Deserialize + Clone`, the dispatch function is synchronous and sub-microsecond, and the command surface is just 5 commands. The main engineering challenges are frontend-side: constructing an accurate SVG key layout for 40+ keys, wiring keyboard events correctly with React lifecycle discipline, and keeping the IPC payload lean with a purpose-built `CalcStateView` struct.

The biggest risks are all avoidable at the scaffolding stage. Phase 13 (workspace skeleton) is a gate — get CI green with an empty Tauri window before writing any application code.

---

## Stack Additions

### Rust (`hp41-gui/src-tauri/Cargo.toml`)

| Name | Version | Purpose |
|------|---------|---------|
| `tauri` | `2.11` | Desktop shell, webview, IPC runtime |
| `tauri-build` | `2.6` | Build-time codegen — must match tauri minor version |
| `serde` | workspace | Serialize/Deserialize for command return types |
| `serde_json` | workspace | JSON serialization for IPC |

> **Critical:** `tauri` and `tauri-build` must NOT enter `[workspace.dependencies]` due to Tauri issue #6122. Declare both exclusively in `hp41-gui/Cargo.toml`.

### Frontend (`hp41-gui/package.json`)

| Name | Version | Purpose |
|------|---------|---------|
| `react` | `^19.2` | UI framework |
| `react-dom` | `^19.2` | DOM renderer |
| `@tauri-apps/api` | `^2.11` | `invoke()` / `listen()` IPC API |
| `@tauri-apps/cli` | `^2.11` | Tauri CLI — must match crate version |
| `vite` | `^8.0` | Build tool and dev server |
| `@vitejs/plugin-react` | `^6.0` | React HMR and JSX transform |
| `typescript` | `^6.0` | Compiler, strict mode |
| `@types/react` + `@types/react-dom` | `^19.2` | TypeScript types |
| `tailwindcss` | `^4.3` | CSS utility framework |
| `@tailwindcss/vite` | `^4.3` | Tailwind v4 Vite plugin |

**Excluded:** `zustand`/`redux` (state lives in Rust), `tauri-specta` (still RC), `shadcn/ui` (custom SVG widget), `next.js` (incompatible with Tauri webview).

---

## Key Architecture Decisions

| Decision | Rationale |
|----------|-----------|
| `std::sync::Mutex<CalcState>` in Tauri AppState | Dispatch is ~65 ns, synchronous, never crosses `.await` |
| Key-ID string API, not Op enum over IPC | Op has complex nested variants; stable string IDs insulate frontend from enum changes |
| `CalcStateView` return type (not full `CalcState`) | Trims IPC payload from ~6 KB to ~200 bytes by excluding `Vec<Op> program` and `call_stack` |
| Key-to-Op mapping in Rust (`key_map.rs`) | TypeScript must not reference Op variants, StoArithKind, or StackReg |
| Nested workspace: `hp41-gui/src-tauri` standalone | Keeps Tauri/wry/tao out of `cargo build --workspace`; CLI CI unaffected |
| Shared save path `~/.hp41/autosave.json` | CLI and GUI are the same calculator, two shells |
| `print_buffer` explicitly drained per command | `#[serde(skip)]` means it never appears in automatic JSON — must drain manually |
| No tauri-specta | RC dependency; 5 commands are hand-writable |

**Canonical data flow:**
```
SVG key click / keyboard event (TypeScript)
  → invoke("dispatch_op", { keyId: "sin" })
  → Tauri IPC (JSON over webview)
  → lock Mutex, resolve keyId→Op, hp41_core::dispatch(), drain print_buffer, build CalcStateView
  → ~200 byte CalcStateView JSON returned
  → React useState → re-render display + annunciators + print output
```

---

## Feature Table Stakes

| Feature | Requirement |
|---------|------------|
| `hp41-gui` builds and launches on macOS, Windows, Linux | GUI-01 |
| SVG HP-41C key layout — 9 rows × 5 cols, ENTER double-width | GUI-02 |
| Clickable keys invoke correct Op dispatch | GUI-03 |
| 12-char display + annunciators render after every op | GUI-04 |
| `hp41-core` integrated via Tauri commands, no logic duplication | GUI-05 |
| `hp41-cli` fully functional and unmodified | GUI-06 |
| Key press visual feedback (CSS `:active`) | Usability gate |
| HP-41C color scheme (dark brown body, gold shift legends) | Aesthetic fidelity |
| Keyboard input (same bindings as CLI) | Power user baseline |
| `last_key_code` written before every dispatch | Correctness for GETKEY / synthetic programming |

---

## Feature Differentiators

| Feature | Complexity | Best Phase |
|---------|------------|-----------|
| Key press CSS animation (scale-down + bounce) | Low | Phase 16 (SVG skin) |
| Stack register panel (X/Y/Z/T/LASTX visible) | Medium | Phase 14 (commands) |
| Print buffer / thermal printer panel | Medium | Phase 17 (persistence) |
| PRGM mode step display | Medium | Phase 18 |
| 14-segment SVG font for display | Medium | Defer to v2.1 |
| Keyboard shortcut overlay | Low | Defer to v2.1 |

---

## Critical Pitfalls (Top 5)

| # | Pitfall | Phase | One-Line Prevention |
|---|---------|-------|---------------------|
| 1 | `tauri-build` version drifts from `tauri` crate | Phase 13 | Declare both exclusively in `hp41-gui/Cargo.toml`; upgrade together |
| 2 | Adding `hp41-gui` breaks `hp41-cli` CI | Phase 13 | Nested/standalone workspace; run `just ci` immediately after adding |
| 3 | Wrong `State<'_, CalcState>` (missing Mutex wrapper) = silent runtime panic | Phase 14 | `type AppState = Mutex<CalcState>;` alias makes it a compile error |
| 4 | `print_buffer` silently dropped (`#[serde(skip)]`) | Phase 14 | Drain `state.print_buffer.drain(..)` explicitly in every command handler |
| 5 | Duplicate React keyboard listeners = multiple IPC calls per keypress | Phase 15 | `useCallback` + always return cleanup in `useEffect`; enable React StrictMode |

Also critical: return `CalcStateView` (~200 bytes) never full `CalcState` (~6 KB); `GuiError` must be `Serialize` (use `thiserror`); Linux CI needs `libwebkit2gtk-4.1-dev` (Tauri v2, not v1's `4.0`); `last_key_code` must be set before `dispatch()`.

---

## Build Order

| Phase | Name | Delivers | Key Pitfalls |
|-------|------|----------|-------------|
| **13** | Workspace Skeleton + Tauri Shell | Empty Tauri window; `just gui-dev` works; `just ci` still green | P1, P2 |
| **14** | CalcState Commands + IPC Layer | `dispatch_op`, `get_state`, `GuiError`, `key_map.rs`, `CalcStateView` | P3, P4, P8, P10, P17 |
| **15** | Basic Display + Keyboard Wiring | React `Display` + annunciators; keyboard events with cleanup | P5, P11 |
| **16** | SVG Keyboard Skin | All 40+ keys; HP-41C color scheme; click handlers; CSS animation | P12 |
| **17** | Persistence + Print Output | `load/save_state_cmd`; 30s auto-save; `PrintOutput` panel | P9 |
| **18** | Program Listing + CI/CD | `get_program_listing`; PRGM mode UI; cross-platform CI | P13–15 |

> Phase numbering continues from v1.1 (last phase: 12). v2.0 starts at Phase 13.

---

## Open Questions

| Question | Decision Needed Before | Recommendation |
|----------|----------------------|----------------|
| Nested vs. root workspace for `hp41-gui`? | Phase 13 | Nested — keeps CLI CI unaffected |
| Extract `key_to_op()` from `hp41-cli` to `hp41-core`? | Phase 14 | Yes — prevents CLI/GUI key-mapping drift |
| 14-segment font: custom SVG, library, or defer? | Phase 16 | Defer to v2.1; monospace acceptable for v2.0 |
| Fixed 400×700 window or resizable? | Phase 13 (`tauri.conf.json`) | Fixed; SVG `viewBox` handles Retina scaling |
| GUI CI: same matrix as CLI or separate job? | Phase 18 | Separate job triggered on `hp41-gui/**` or `hp41-core/**` changes |
| Auto-save: background thread or Tauri timer? | Phase 17 | Background thread via `AppHandle::clone()` in `setup()` |
