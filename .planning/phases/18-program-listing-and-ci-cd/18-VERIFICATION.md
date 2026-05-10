---
phase: 18-program-listing-and-ci-cd
verified: 2026-05-10T19:00:00Z
status: human_needed
score: 5/5 must-haves verified
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: 4/5
  gaps_closed:
    - "The cross-platform CI job runs cargo test before cargo build --release (test failures would be caught by CI)"
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "Verify PRGM mode program listing panel appears visually"
    expected: "Entering PRGM mode shows a dark panel below the keyboard with step numbers and mnemonic labels. The format matches hp41-cli (e.g., '000 END', '001 + ', '002 ENTER'). Panel header shows 'PRGM — N steps' step count."
    why_human: "Visual UI rendering cannot be verified programmatically without running the Tauri application"
  - test: "Verify SST advances the highlighted step (SC-2)"
    expected: "Pressing F7 or clicking the SST SVG key advances the program counter by 1 and re-highlights the new active step with green-on-dark-green background. Auto-scroll keeps the active step visible."
    why_human: "Runtime IPC call behavior and visual highlighting require a running Tauri session — already human-approved in 18-04-SUMMARY.md but cannot be re-verified programmatically"
  - test: "Verify BST moves the highlighted step backward and clamps at 000 (SC-3)"
    expected: "Pressing F8 or clicking the BST SVG key decrements pc, highlights the new step, and scrolls to keep it visible. Pressing BST at pc=0 leaves the listing unchanged."
    why_human: "Same as SC-2 — runtime behavior in a live Tauri window. Approved in 18-04-SUMMARY.md."
  - test: "Verify GUI CI independence from CLI CI (SC-5)"
    expected: "After pushing a change to hp41-gui/**, GitHub Actions shows two separate workflow runs: 'ci' and 'ci-gui'. A failure in ci-gui does not block the 'ci' run. The ci-gui run now executes cargo test before cargo build --release."
    why_human: "Requires an actual GitHub push and live Actions observation. Cannot be verified locally."
---

# Phase 18: Program Listing & CI/CD Verification Report

**Phase Goal:** Users in PRGM mode can view the complete program listing and step through it with SST and BST in the GUI; a cross-platform CI job (macOS, Windows, Ubuntu) builds and tests hp41-gui on every push to paths that affect the GUI or core.
**Verified:** 2026-05-10T19:00:00Z
**Status:** human_needed
**Re-verification:** Yes — after gap closure (gui-ci cargo test step added)

## Goal Achievement

### Observable Truths (from ROADMAP.md Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Entering PRGM mode in the GUI displays the program listing with step numbers and mnemonic labels matching the format shown in hp41-cli | ? UNCERTAIN | JSX conditional on `calcState.annunciators.prgm` present in App.tsx:163; `program_steps` populated by `format_all_steps()` in Rust; human-approved SC-1 documented in 18-04-SUMMARY.md |
| 2 | Pressing the SST key (via keyboard binding or SVG click) advances the program counter by one step and highlights the current step in the listing | ? UNCERTAIN | `handle_sst()` increments `calc.pc` with bounds check; `sst_step` Tauri command registered; F7/e.code routing in resolveKeyId; `activeStepRef` + `scrollIntoView` wired; human-approved SC-2 documented in 18-04-SUMMARY.md |
| 3 | Pressing the BST key steps backward one position in the program and the listing scrolls to keep the highlighted step visible | ? UNCERTAIN | `handle_bst()` uses `saturating_sub(1)`; `bst_step` Tauri command registered; same scroll mechanism as SC-2; human-approved SC-3 documented in 18-04-SUMMARY.md |
| 4 | The cross-platform CI job runs on Windows, macOS, and Ubuntu; the build completes without error on all three platforms and is triggered only on changes to `hp41-gui/**` or `hp41-core/**` | ✓ VERIFIED | `ci-gui.yml` has 3-OS matrix (`ubuntu-latest`, `macos-latest`, `windows-latest`), `fail-fast: false`, path filter for `hp41-gui/**` and `hp41-core/**` on both push and pull_request; `just gui-ci` used as CI entrypoint |
| 5 | `just ci` (the CLI pipeline) and the new GUI CI job are independent — a GUI build failure does not block CLI CI and vice versa | ✓ VERIFIED | `ci-gui.yml` is a separate file; `ci.yml` is byte-for-byte unchanged. The `gui-ci` Justfile recipe now runs `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` (line 77) before `cargo build --release` (line 78). All 27 tests pass. The CI job fulfils "builds AND tests hp41-gui." |

**Score:** 5/5 truths verified (SC-1, SC-2, SC-3 require human verification of runtime behavior; SC-4 and SC-5 pass automated checks including the gap closure confirmation)

---

### Gap Closure Verification

**Closed gap: gui-ci recipe missing cargo test**

The single blocker identified in the previous verification has been fixed. The `gui-ci` Justfile recipe (lines 74-78) now reads:

```
cd hp41-gui && npm install
cd hp41-gui && npx tsc --noEmit
cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml
cargo build --release --manifest-path hp41-gui/src-tauri/Cargo.toml
```

Behavioral spot-check confirms all 27 tests pass: `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` exits 0 (27 passed, 3 suites, 0.01s).

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-gui/src-tauri/src/prgm_display.rs` | `format_all_steps()` function | ✓ VERIFIED | `pub fn format_all_steps` confirmed present; no regression |
| `hp41-gui/src-tauri/src/types.rs` | `program_steps: Vec<String>` and `pc: usize` in CalcStateView | ✓ VERIFIED | Fields confirmed present; no regression |
| `hp41-gui/src-tauri/src/commands.rs` | `handle_sst`, `handle_bst`, `sst_step`, `bst_step` | ✓ VERIFIED | All four functions present; no regression |
| `hp41-gui/src-tauri/src/lib.rs` | `mod prgm_display` + sst_step/bst_step in invoke_handler | ✓ VERIFIED | Both confirmed present; no regression |
| `hp41-gui/src-tauri/permissions/sst-step.toml` | Tauri permission for sst_step | ✓ VERIFIED | `identifier = "allow-sst-step"` confirmed |
| `hp41-gui/src-tauri/permissions/bst-step.toml` | Tauri permission for bst_step | ✓ VERIFIED | `identifier = "allow-bst-step"` confirmed |
| `hp41-gui/src-tauri/capabilities/default.json` | `allow-sst-step` and `allow-bst-step` in permissions | ✓ VERIFIED | Both identifiers confirmed |
| `hp41-gui/src-tauri/tauri.conf.json` | `height: 900` | ✓ VERIFIED | `"height": 900` confirmed |
| `hp41-gui/src/App.tsx` | Program listing panel JSX, SST/BST routing, F7/F8 in resolveKeyId | ✓ VERIFIED | All wiring confirmed from initial verification; no regression |
| `hp41-gui/src/Keyboard.tsx` | `id: 'sst'` and `id: 'bst'` in KEY_DEFS | ✓ VERIFIED | Confirmed from initial verification |
| `hp41-gui/src/App.css` | Dark-themed prgm panel styles | ✓ VERIFIED | Confirmed from initial verification |
| `.github/workflows/ci-gui.yml` | Separate CI file from ci.yml | ✓ VERIFIED | File exists; triggers on correct path filters; calls `just gui-ci` |
| `Justfile` | `gui-ci` recipe with npm install + tsc + **cargo test** + build | ✓ VERIFIED | All four steps present at lines 75-78; gap is closed |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `types.rs from_state()` | `prgm_display::format_all_steps()` | function call | ✓ WIRED | Confirmed from initial verification; no regression |
| `lib.rs invoke_handler` | `commands::sst_step, commands::bst_step` | `tauri::generate_handler!` | ✓ WIRED | Lines 47-48 confirmed |
| `capabilities/default.json` | `sst-step.toml / bst-step.toml` | permission identifier strings | ✓ WIRED | Both identifiers confirmed |
| `App.tsx calcState.annunciators.prgm` | `.prgm-panel` JSX render | conditional JSX | ✓ WIRED | Confirmed from initial verification |
| `App.tsx handleClick` | `invoke('sst_step') / invoke('bst_step')` | if/else branches | ✓ WIRED | Confirmed from initial verification |
| `App.tsx activeStepRef` | step div at `calcState.pc` | `ref={calcState.pc === i ? activeStepRef : null}` | ✓ WIRED | Confirmed from initial verification |
| `ci-gui.yml on: push paths` | `hp41-gui/**, hp41-core/**` | GitHub Actions path filter | ✓ WIRED | Lines 7-8 and 12-13 confirmed |
| `Justfile gui-ci` | `cargo test` | Justfile step | ✓ WIRED | Line 77: `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` — gap closed |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `App.tsx prgm-panel` | `calcState.program_steps` | `format_all_steps(state)` in `types.rs::from_state()` → IPC → React state | Yes — iterates `state.program` Vec with enumerated indices | ✓ FLOWING |
| `App.tsx prgm-panel` | `calcState.pc` | `state.pc` in `types.rs::from_state()` → IPC → React state | Yes — direct field read from CalcState | ✓ FLOWING |
| `App.tsx step-active highlight` | `calcState.pc === i` | Rust `pc` field via IPC, compared to map index | Yes — real comparison, not hardcoded | ✓ FLOWING |
| `App.tsx scrollIntoView` | `activeStepRef.current` | `ref={calcState.pc === i ? activeStepRef : null}` | Yes — ref bound to DOM element at active step | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Rust test suite passes | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` | 27 passed, 0 failed, 3 suites, 0.01s | ✓ PASS |
| gui-ci recipe runs cargo test before build | `grep -n 'cargo test\|cargo build' Justfile` | cargo test at line 77, cargo build at line 78 | ✓ PASS |
| ci-gui.yml calls just gui-ci | `grep 'just gui-ci' .github/workflows/ci-gui.yml` | Returns 1 match at line 50 | ✓ PASS |
| Three-OS matrix in ci-gui.yml | `grep -c 'ubuntu-latest\|macos-latest\|windows-latest' ci-gui.yml` | Confirmed 3-OS matrix at line 26 | ✓ PASS |
| Path filter for hp41-gui/** and hp41-core/** | `grep -c 'hp41-gui/\*\*\|hp41-core/\*\*' ci-gui.yml` | 4 matches (push + pull_request, both paths) | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| PROG-01 | 18-01, 18-02, 18-03, 18-04 | User can view the current program listing and navigate steps with SST/BST in GUI PRGM mode | ✓ SATISFIED | Rust backend fully implemented and tested (27/27 tests pass); React frontend implemented and human-approved (SC-1, SC-2, SC-3 in 18-04-SUMMARY.md); CI gap CR-01 closed — cargo test now runs in gui-ci recipe. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `hp41-gui/src-tauri/src/prgm_display.rs` | 125 | `Op::Test(_) => "TEST"` wildcard — all 12 test conditions display identically | ⚠️ WARNING | Program listing cannot distinguish x=0? from x≠y? etc. Reduces fidelity for programs using conditional tests. Not a blocker for PROG-01 core requirement. Informational only. |

The previous blocker (missing `cargo test` in `gui-ci`) has been resolved and no longer appears in this table.

### Human Verification Required

#### 1. PRGM Mode Panel Visual Appearance (SC-1)

**Test:** Launch the Tauri GUI with `just gui-dev`. Press the PRGM key. Observe the area below the keyboard.
**Expected:** A dark panel appears showing "PRGM — 0 steps" in the header and "000 END" as the sole step row with green-on-dark-green highlight (#1e3a1e background, #c8e6c9 text). Window height is visibly taller (~900px) than before.
**Why human:** Visual UI rendering cannot be verified without running the application. Panel was human-approved in 18-04-SUMMARY.md (SC-1 approved), but re-verification is recommended before closing the phase.

#### 2. SST Advances Highlighted Step (SC-2)

**Test:** With PRGM mode active and a multi-step program loaded, press F7 (or click the SST key in the SVG skin).
**Expected:** The green highlight moves down by one step. Auto-scroll fires (`scrollIntoView smooth`) if the step was off-screen. If at the last instruction, SST advances to the "NNN END" row.
**Why human:** IPC call behavior and scroll animation require a live Tauri session. Approved in 18-04-SUMMARY.md.

#### 3. BST Moves Highlighted Step Backward (SC-3)

**Test:** With PRGM mode active and pc > 0, press F8 (or click the BST key). Press BST repeatedly until pc = 0.
**Expected:** Highlight moves up by one step each press. At pc=0, BST has no effect (clamped via saturating_sub). Scroll keeps the active step in view.
**Why human:** Same as SC-2. Approved in 18-04-SUMMARY.md.

#### 4. CI Independence in GitHub Actions (SC-5)

**Test:** Push a change to `hp41-gui/src/App.tsx`. Observe GitHub Actions.
**Expected:** Two separate workflow runs appear: "ci" and "ci-gui". The "ci" run is not triggered (no changes to CLI codebase). The "ci-gui" run executes on all three platforms and runs `cargo test` (27 tests) before `cargo build --release`.
**Why human:** Requires an actual GitHub push and live Actions observation. Cannot be simulated locally.

### Gaps Summary

No blocker gaps remain. The single gap from the initial verification — `gui-ci` Justfile recipe not running `cargo test` — has been resolved.

The `gui-ci` recipe now runs all four steps in order:
1. `cd hp41-gui && npm install`
2. `cd hp41-gui && npx tsc --noEmit`
3. `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` (27 tests pass)
4. `cargo build --release --manifest-path hp41-gui/src-tauri/Cargo.toml`

The phase is blocked only on human verification of the Tauri GUI runtime behavior (visual panel rendering, SST/BST highlight movement, scroll, and live GitHub Actions CI run). These were approved in 18-04-SUMMARY.md and are routine human sign-off items, not implementation gaps.

---

_Verified: 2026-05-10T19:00:00Z_
_Verifier: Claude (gsd-verifier)_
