---
phase: 13-workspace-skeleton
status: clean
files_reviewed: 14
findings:
  critical: 0
  warning: 0
  info: 2
  total: 2
---

# Code Review: Phase 13 — Workspace Skeleton

## Summary

All 14 mandatory invariants specified in the review charter pass. The Tauri v2 + React + TypeScript GUI scaffold is correctly isolated as a nested Cargo workspace: `hp41-gui/src-tauri/Cargo.toml` declares its own `[workspace]` with `resolver = "2"`, the root `Cargo.toml` members list remains `["hp41-core", "hp41-cli"]` unchanged, and `tauri`/`tauri-build` are absent from the root workspace dependencies. Every cross-file consistency constraint holds: the `devUrl` port (5173), vite `strictPort`, vite `outDir` (`dist`), and tauri `frontendDist` (`../dist`) are aligned; `ch.talent-factory.hp41` is the bundle identifier throughout; `#![deny(clippy::unwrap_used)]` is active in `lib.rs`; `React.StrictMode` wraps the app entry; and `core:default` is present in the Tauri v2 capability file. Two informational observations are noted — neither is a defect in the shipped code.

## Findings

### Info 1 — `tauri.conf.json` contains `"macOSPrivateApi": true` not present in the PATTERNS.md template

**File:** `hp41-gui/src-tauri/tauri.conf.json` — `app` block
**Confidence:** 30 (informational only — not a defect)

The planning document `13-PATTERNS.md` shows the `app` section containing only `windows: [...]`. The actual file adds `"macOSPrivateApi": true` in the `app` block. This addition is intentional and correct: it pairs with the `features = ["macos-private-api"]` feature flag in `hp41-gui/src-tauri/Cargo.toml`. The Tauri documentation requires both the Cargo feature and the `tauri.conf.json` flag to enable the macOS private API; having only one half would silently do nothing. The PATTERNS.md template was intentionally minimal and the implementation correctly completed the pairing. No action required.

### Info 2 — SUMMARY documentation drift: `AppState` described as placeholder but real type is already wired

**File:** `.planning/phases/13-workspace-skeleton/13-03-SUMMARY.md`
**Confidence:** 25 (documentation only — code is correct)

The Phase 14 handoff section of `13-03-SUMMARY.md` describes `AppState` as a placeholder. The actual `hp41-gui/src-tauri/src/lib.rs` contains `pub type AppState = Mutex<hp41_core::CalcState>;` — the full Mutex-wrapped implementation, not a placeholder. The Mutex registration via `app.manage(Mutex::new(hp41_core::CalcState::new()))` is already present in the `setup` closure. Phase 14 planners should read `lib.rs` directly. The Phase 14 constraint about using `.unwrap_or_else(|e| e.into_inner())` for mutex lock in command handlers remains valid and must be enforced. No code change required.

## Conclusion

Pass. The Phase 13 workspace skeleton meets all mandatory invariants. No critical or warning-level issues were identified. The two informational observations do not require code changes before Phase 14 begins.
