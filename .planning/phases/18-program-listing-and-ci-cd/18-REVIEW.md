---
phase: 18-program-listing-and-ci-cd
reviewed: 2026-05-10T12:00:00Z
depth: standard
files_reviewed: 13
files_reviewed_list:
  - .github/workflows/ci-gui.yml
  - hp41-gui/src-tauri/capabilities/default.json
  - hp41-gui/src-tauri/permissions/bst-step.toml
  - hp41-gui/src-tauri/permissions/sst-step.toml
  - hp41-gui/src-tauri/src/commands.rs
  - hp41-gui/src-tauri/src/lib.rs
  - hp41-gui/src-tauri/src/prgm_display.rs
  - hp41-gui/src-tauri/src/types.rs
  - hp41-gui/src-tauri/tauri.conf.json
  - hp41-gui/src/App.css
  - hp41-gui/src/App.tsx
  - hp41-gui/src/Keyboard.tsx
  - Justfile
findings:
  critical: 1
  warning: 4
  info: 4
  total: 9
status: issues_found
---

# Phase 18: Code Review Report

**Reviewed:** 2026-05-10T12:00:00Z
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Phase 18 adds the program listing panel (SST/BST navigation, `format_all_steps`, React `prgm-panel`) and the `ci-gui.yml` workflow. The IPC layer additions (`sst_step`, `bst_step`) are correctly wired and the Rust helpers are unit-tested. The frontend routing of `sst`/`bst` key IDs through `handleClick` instead of `dispatch_op` is sound. The main structural defect is that the CI workflow never runs the Tauri unit tests, meaning the test suite added across commands.rs, types.rs, and prgm_display.rs is completely invisible to CI. A secondary security concern exists around `macOSPrivateApi: true` with no documented justification. Several informational items cover dead code, a misleading comment, and a stale capability description.

## Critical Issues

### CR-01: Tauri unit tests never executed in CI

**File:** `.github/workflows/ci-gui.yml:50` / `Justfile:74-77`
**Issue:** The `gui-ci` recipe (and therefore the CI job) runs `npm install`, `tsc --noEmit`, and `cargo build --release`. It does NOT run `cargo test`. Every unit test in `commands.rs` (7 tests covering SC-2, SC-3, SST/BST clamping, eex_chs), `types.rs` (6 tests covering payload size, annunciators, GuiError, Phase 18 fields), and `prgm_display.rs` (2 tests) is silently skipped on every CI run. A regression in any of those code paths would not be caught by the pipeline.

**Fix:** Add a test step to `gui-ci` in the Justfile and add the corresponding step in the workflow:

```just
# gui-ci: CI gate — TypeScript type-check and Rust release build + tests
gui-ci:
    cd hp41-gui && npm install
    cd hp41-gui && npx tsc --noEmit
    cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml
    cargo build --release --manifest-path hp41-gui/src-tauri/Cargo.toml
```

The CI workflow already calls `just gui-ci`, so no change to `.github/workflows/ci-gui.yml` is needed beyond the Justfile fix.

## Warnings

### WR-01: `Op::Test(_)` display name discards test condition — all 12 variants render as "TEST"

**File:** `hp41-gui/src-tauri/src/prgm_display.rs:125`
**Issue:** The wildcard `Op::Test(_) => "TEST".to_string()` silently drops the `TestKind` inner value. In a program listing, every conditional test — whether `x=0`, `x≠y`, `x>0`, etc. — displays identically as `"TEST"`. A user inspecting their program listing cannot distinguish which test is at each step. The HP-41 hardware displays the specific condition (e.g., `X=0?`).

**Fix:** Match on `TestKind` variants explicitly:
```rust
Op::Test(kind) => {
    let cond = match kind {
        TestKind::XEqZero  => "X=0?",
        TestKind::XNeZero  => "X\u{2260}0?",
        TestKind::XLtZero  => "X<0?",
        TestKind::XGtZero  => "X>0?",
        TestKind::XLeZero  => "X\u{2264}0?",
        TestKind::XGeZero  => "X\u{2265}0?",
        TestKind::XEqY     => "X=Y?",
        TestKind::XNeY     => "X\u{2260}Y?",
        TestKind::XLtY     => "X<Y?",
        TestKind::XGtY     => "X>Y?",
        TestKind::XLeY     => "X\u{2264}Y?",
        TestKind::XGeY     => "X\u{2265}Y?",
    };
    cond.to_string()
}
```

### WR-02: Misleading comment — `drop(state)` drops the Arc, not the MutexGuard

**File:** `hp41-gui/src-tauri/src/lib.rs:33-35`
**Issue:** The inline comment reads `"Clone state under lock, then drop guard before disk I/O (CR-01)"`. The `MutexGuard` is a temporary that is already dropped at the semicolon ending line 34. `drop(state)` on line 35 drops the `tauri::State<'_, AppState>` Arc-backed reference — it has no effect on the mutex lock. The comment incorrectly attributes the lock-release semantics to the `drop(state)` call, which could mislead a maintainer into thinking the guard is still held until that point.

**Fix:** Remove the redundant `drop(state)` and correct the comment:
```rust
// Lock, clone, and immediately release the guard (guard is a temporary —
// dropped at the semicolon). Disk I/O happens outside the lock.
let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
if let Err(e) = persistence::save_state(&thread_save_path, &snapshot) {
    eprintln!("auto-save failed: {e}");
}
```

### WR-03: `macOSPrivateApi: true` — private API use with no documented justification

**File:** `hp41-gui/src-tauri/tauri.conf.json:11`
**Issue:** `macOSPrivateApi: true` enables Tauri's use of private macOS APIs (primarily transparent/vibrancy window effects). Enabling private APIs in a production app risks App Store rejection and introduces API-stability risk across macOS versions. Nothing in the current CSS or UI code appears to require transparency or vibrancy effects — the calculator uses solid dark backgrounds. There is no comment or design decision doc explaining why this flag is set.

**Fix:** If private APIs are not needed, set to `false`. If they are required for a specific documented reason, add a comment in `tauri.conf.json`:
```json
"macOSPrivateApi": false
```
Or, if intentional:
```json
"macOSPrivateApi": true  // Required for: <specific reason>
```

### WR-04: npm packages not cached in CI — reinstalled on every run

**File:** `.github/workflows/ci-gui.yml:37-39`
**Issue:** `actions/setup-node@v4` supports a `cache: 'npm'` parameter that caches the npm package registry. Without it, `npm install` fetches and unpacks all packages from the network on every CI run. Since `gui-ci` already runs `npm install` as its first step, this means every CI invocation re-downloads the full npm dependency tree.

**Fix:**
```yaml
- uses: actions/setup-node@v4
  with:
    node-version: 'lts/*'
    cache: 'npm'
    cache-dependency-path: hp41-gui/package-lock.json
```

## Info

### IN-01: `format_step()` is dead code — allowed but never called

**File:** `hp41-gui/src-tauri/src/prgm_display.rs:16-25`
**Issue:** `format_step()` is suppressed with `#[allow(dead_code)]` and is never called anywhere in the codebase. The companion `format_all_steps()` explicitly documents "do NOT use format_step() in a loop" and implements its own iteration. The function exists as dead weight; suppressing the compiler warning instead of removing the function hides a real signal.

**Fix:** Remove `format_step()` entirely if it has no planned future use. If it is retained for potential future use, document why.

### IN-02: Capability description is stale — still references "Phase 14"

**File:** `hp41-gui/src-tauri/capabilities/default.json:3`
**Issue:** The `description` field reads `"Default capability for hp41-gui — core + Phase 14 IPC commands"`. Phase 18 added `allow-sst-step` and `allow-bst-step` to this capability. The description no longer accurately reflects the full set of registered permissions.

**Fix:**
```json
"description": "Default capability for hp41-gui — core IPC commands (dispatch_op, get_state, sst_step, bst_step)"
```

### IN-03: `test_dispatch_op_payload_size` asserts a limit that is only valid for the empty-program case

**File:** `hp41-gui/src-tauri/src/types.rs:121-131`
**Issue:** The test asserts `json.len() <= 400` for an empty `CalcState`, but its own comment says `"Real programs grow beyond this limit"`. Since `program_steps` is an unbounded `Vec<String>` serialized inline in every IPC response, a 100-step program easily exceeds 400 bytes. The test passes trivially for the empty case and provides zero coverage of the actual production payload shape. It will never fail regardless of how large the struct grows.

**Fix:** Either remove the byte-budget assertion (it cannot be meaningfully enforced without capping `program_steps`), or replace it with a structural assertion that verifies field presence and correct types, which is what the other tests already do:
```rust
// Just verify structural correctness — payload size is unbounded by design
let view = CalcStateView::from_state(&state, vec![]);
assert_eq!(view.program_steps, vec!["000 END"]);
assert_eq!(view.pc, 0);
```

### IN-04: `console.error` calls in App.tsx are production error handling

**File:** `hp41-gui/src/App.tsx:71,87`
**Issue:** IPC errors (from `get_state` and `dispatch_op`/`sst_step`/`bst_step`) are silently swallowed with `console.error(...)`. The UI provides no visual feedback when an IPC call fails. From the user's perspective, the calculator becomes unresponsive with no indication of why. This is acceptable for MVP, but the `console.error` calls are the only error surface.

**Fix (MVP-appropriate):** At minimum, set an error state that renders in the display:
```tsx
const [errorMsg, setErrorMsg] = useState<string | null>(null);
// In .catch: setErrorMsg('IPC error — restart required');
// In render: show errorMsg in the display div when set
```

---

_Reviewed: 2026-05-10T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
