---
phase: 27-test-hardening
plan: 04
type: execute
wave: 2
depends_on: []
files_modified:
  - hp41-gui/wdio.conf.js
  - hp41-gui/e2e/smoke.spec.ts
  - hp41-gui/src/Display14Seg.tsx
  - hp41-gui/package.json
  - justfile
  - .github/workflows/ci-gui.yml
  - CLAUDE.md
autonomous: true
requirements:
  - FN-QUAL-05
user_setup: []
tags:
  - e2e
  - webdriverio
  - tauri-driver
  - vitest-ci-gating
  - ubuntu-only

must_haves:
  truths:
    - "WebdriverIO + `tauri-driver` E2E smoke (NOT Playwright per D-27.15 AMENDED 2026-05-15): boots the production Tauri binary, clicks `2 ENTER 3 +` on the SVG keyboard, asserts `[data-testid=\"lcd-display\"]` reads `5.0000` (or the current FIX(4) display-mode equivalent)."
    - "E2E job runs ONLY on Ubuntu in `.github/workflows/ci-gui.yml` (ROADMAP cross-cutting constraint, D-27.15). macOS/Windows matrix jobs UNCHANGED. The job is REQUIRED for merge with 1 retry via `mochaOpts.retries: 1` (D-27.16)."
    - "Devdeps added: `webdriverio` + `@wdio/cli` + `@wdio/local-runner` + `@wdio/mocha-framework` + `@wdio/spec-reporter` (^9.19) in `hp41-gui/package.json` (D-27.15 AMENDED). Playwright (`@playwright/test`) is NOT added — the original D-27.15 wording is superseded by the amendment."
    - "`hp41-gui/wdio.conf.js` exists pointing at `tauri-driver` on `127.0.0.1:4444`, with `mochaOpts.retries: 1` (D-27.16), `framework: 'mocha'`, and `tauri:options.application: '../src-tauri/target/release/hp41-gui'` (or the actual binary name verified during read_first)."
    - "`hp41-gui/e2e/smoke.spec.ts` (or `.js`) exists with a single Mocha `describe`/`it` smoke that clicks `[data-key-id=\"2\"]` → `[data-key-id=\"enter\"]` → `[data-key-id=\"3\"]` → `[data-key-id=\"plus\"]` and asserts the LCD reads `5.0000`."
    - "`hp41-gui/src/Display14Seg.tsx` carries a `data-testid=\"lcd-display\"` attribute on its outermost rendered element (RESEARCH Pitfall 10, D-27 Claude's Discretion — one-line edit; allowed under SC-4 because it lives in `hp41-gui/src/` not `hp41-gui/src-tauri/`)."
    - "`justfile` `gui-ci:` recipe appends `cd hp41-gui && npm test` so the existing 5 Vitest files (`App.test.tsx`, `Display14Seg.test.tsx`, `HelpOverlay.test.tsx`, `Keyboard.test.tsx`, `pending_input.test.ts`) gate on every CI run (D-27.14)."
    - "`justfile` gains a new `gui-e2e:` recipe for local Linux runs (developer-side; not strictly required by CI but matches the docs-matrix / docs-matrix-check pattern from Plan 25-04)."
    - "`.github/workflows/ci-gui.yml` gains a separate `e2e-linux` job (`runs-on: ubuntu-latest`, depends on `build`) that: (a) `apt-get install` adds `webkit2gtk-driver` + `xvfb` to the existing line (Pitfall 6); (b) caches `~/.cargo/bin/tauri-driver` (Pitfall 5) via `actions/cache@v4` keyed on tauri-driver version 2.0.6; (c) installs `tauri-driver` via `cargo install tauri-driver --locked --version 2.0.6`; (d) runs `xvfb-run -a just gui-e2e` (Assumption A5)."
    - "GUI coverage (Vitest + cargo llvm-cov on hp41-gui/src-tauri) is measured ONE-SHOT during this plan's execution and recorded in `27-04-SUMMARY.md` per D-27.4. NO CI gate is added. NO coverage provider devDep is added to `hp41-gui/package.json`."
    - "CLAUDE.md gains a Phase 27 settled-architecture block recording: WebdriverIO + tauri-driver (Ubuntu only) per D-27.15 AMENDED, Vitest CI gating per D-27.14, one-line `data-testid` on Display14Seg.tsx, and the GUI coverage measurement snapshot."
    - "SC-4 invariant preserved: NO source changes to `hp41-gui/src-tauri/`. The Display14Seg.tsx edit is in `hp41-gui/src/` which is OUTSIDE the SC-4 boundary (RESEARCH Assumption A3 verified against CONTEXT.md text)."
    - "MSRV 1.88 unchanged. `tauri-driver` 2.0.6 MSRV is 1.77 (compatible). WebdriverIO 9.x is a Node tool, not a Rust dependency."
  artifacts:
    - path: "hp41-gui/wdio.conf.js"
      provides: "NEW — WebdriverIO config; framework=mocha; mochaOpts.retries=1; tauri:options.application points at the release binary; onPrepare builds; beforeSession spawns tauri-driver"
      contains: "tauri-driver"
      contains_2: "mochaOpts"
      contains_3: "retries: 1"
    - path: "hp41-gui/e2e/smoke.spec.ts"
      provides: "NEW — single Mocha smoke test: 2 ENTER 3 + → 5.0000"
      contains: "data-key-id=\"2\""
      contains_2: "data-testid=\"lcd-display\""
      contains_3: "5.0000"
    - path: "hp41-gui/src/Display14Seg.tsx"
      provides: "EDITED — adds `data-testid=\"lcd-display\"` to outermost rendered element (one-line change per RESEARCH Pitfall 10)"
      contains: "data-testid=\"lcd-display\""
    - path: "hp41-gui/package.json"
      provides: "EDITED — adds webdriverio + @wdio/cli + @wdio/local-runner + @wdio/mocha-framework + @wdio/spec-reporter (^9.19) to devDependencies"
      contains: "webdriverio"
      contains_2: "@wdio/cli"
    - path: "justfile"
      provides: "EDITED — gui-ci: appends `cd hp41-gui && npm test` (D-27.14); new gui-e2e: recipe for Linux developer-side E2E runs"
      contains: "gui-e2e"
      contains_2: "npm test"
    - path: ".github/workflows/ci-gui.yml"
      provides: "EDITED — new e2e-linux job; webkit2gtk-driver + xvfb apt; tauri-driver cargo-install with cache; xvfb-run wrapper; required for merge with 1 retry"
      contains: "e2e-linux"
      contains_2: "webkit2gtk-driver"
      contains_3: "tauri-driver"
    - path: "CLAUDE.md"
      provides: "EDITED — Phase 27 v2.2 additions block notes WebdriverIO Ubuntu-only scope; Vitest CI gating; data-testid Display14Seg edit; GUI coverage snapshot"
      contains: "WebdriverIO"
      contains_2: "data-testid"
  key_links:
    - from: "hp41-gui/e2e/smoke.spec.ts"
      to: "hp41-gui/src/Keyboard.tsx (data-key-id attributes verified at lines 285, 303) + hp41-gui/src/Display14Seg.tsx (new data-testid)"
      via: "WebdriverIO $('[data-key-id=...]') selectors + $('[data-testid=...]') for the assertion"
      pattern: "data-key-id|data-testid"
    - from: "hp41-gui/wdio.conf.js"
      to: "tauri-driver process (~/.cargo/bin/tauri-driver) + production binary (hp41-gui/src-tauri/target/release/hp41-gui)"
      via: "beforeSession spawn; afterSession kill; capabilities.tauri:options.application"
      pattern: "tauri-driver|target/release"
    - from: ".github/workflows/ci-gui.yml::e2e-linux"
      to: "justfile gui-e2e"
      via: "xvfb-run -a just gui-e2e"
      pattern: "just gui-e2e|xvfb-run"
    - from: "justfile gui-ci"
      to: "5 existing Vitest files (App.test.tsx, Display14Seg.test.tsx, HelpOverlay.test.tsx, Keyboard.test.tsx, pending_input.test.ts)"
      via: "appended `cd hp41-gui && npm test` (mapped to `vitest run` per package.json)"
      pattern: "npm test"
---

# Plan 27-04: E2E (WebdriverIO + tauri-driver) + Vitest CI gating

**Goal:** Close FN-QUAL-05 with a WebdriverIO + `tauri-driver` E2E smoke that boots the production Tauri binary and verifies `2 ENTER 3 + → 5.0000` end-to-end on the Ubuntu CI runner. Concurrently close the quietly-surfaced D-27.14 hole by adding `npm test` to `gui-ci` so the existing 5 Vitest files gate on every CI push.

**Requirement IDs:** FN-QUAL-05 (and D-27.14 CI hygiene)
**Touches:** `hp41-gui/` (1 new config, 1 new spec, 1 edited component, 1 edited package.json), `justfile`, `.github/workflows/ci-gui.yml`, `CLAUDE.md`
**Plan depends on:** none (independent file surface). Recommended execution order — LAST in the wave, so all prior coverage / proptest / IND additions are stable before the new CI job activates.

<objective>
Land the FN-QUAL-05 Playwright/E2E job per the AMENDED D-27.15 (WebdriverIO + tauri-driver — NOT Playwright; the original wording is superseded by the 2026-05-15 amendment after research established the WebDriver-classic protocol mismatch with Playwright). The job boots the real Tauri release binary, drives it via `tauri-driver` over WebDriver classic on `127.0.0.1:4444`, clicks `2 ENTER 3 +` on the rendered SVG keyboard, and asserts the LCD reads `5.0000`. Ubuntu-only per ROADMAP cross-cutting constraint. Required for merge with 1 retry per D-27.16.

Concurrently close the D-27.14 quiet hole: the existing 5 Vitest files (`App.test.tsx`, `Display14Seg.test.tsx`, `HelpOverlay.test.tsx`, `Keyboard.test.tsx`, `pending_input.test.ts`) pass locally but are NOT run on CI today. One-line edit to `justfile gui-ci` adds `npm test` so they gate every PR.

GUI coverage (D-27.4) is measured one-shot during this plan's execution and recorded in `27-04-SUMMARY.md` — no CI gate added, no devDep added, advisory snapshot only.

Purpose: FN-QUAL-05 is the canary that proves the Tauri ↔ React ↔ `hp41-core` dispatch chain works end-to-end after every change. Per D-27.13, the smoke is intentionally minimal (literal ROADMAP scope) — not a comprehensive E2E suite; broader flows (modal interactions, persistence roundtrip) are explicitly deferred. The Vitest CI gating is a one-line fix that closes a sunk-cost regression hole at zero new test cost.

Output: WebdriverIO config + smoke spec + Display14Seg one-line edit + package.json devDep additions + justfile recipe edits + ci-gui.yml new e2e-linux job + CLAUDE.md settled-architecture note + 27-04-SUMMARY.md GUI coverage measurement.

Out of scope (explicit):
- Playwright (`@playwright/test`) — D-27.15 AMENDED 2026-05-15 supersedes the original wording. The mismatch between Playwright (CDP/native) and `tauri-driver` (WebDriver classic) is the source of the amendment per RESEARCH §Open Question 1.
- Broader E2E coverage (modal flows, autosave persistence roundtrip) — deferred to v2.3+ per D-27.13.
- `tauri-plugin-playwright` (single-author crate, contradicts the "no `hp41-gui/src-tauri/` changes" invariant) — explicitly rejected per RESEARCH §Standard Stack (Option C).
- GUI coverage CI gate — D-27.4 explicit "measure only, no gate".
- macOS / Windows E2E runners — ROADMAP cross-cutting constraint (Ubuntu only).
- Coverage push, accuracy extension → Plan 27-01
- Proptest suites → Plan 27-02
- IND integration suite → Plan 27-03
- Any `hp41-core/src/` change (frozen)
- Any `hp41-gui/src-tauri/` change (SC-4)
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/REQUIREMENTS.md
@CLAUDE.md

# Phase 27 inputs
@.planning/phases/27-test-hardening/27-CONTEXT.md
@.planning/phases/27-test-hardening/27-RESEARCH.md

# Justfile + CI workflow (targets of the edits)
@justfile
@.github/workflows/ci-gui.yml

# Frontend (the spec selectors must exist; Display14Seg edit lands here)
@hp41-gui/src/Keyboard.tsx
@hp41-gui/src/Display14Seg.tsx
@hp41-gui/src/App.tsx
@hp41-gui/package.json
@hp41-gui/vite.config.ts

# Existing Vitest files (D-27.14 verifies they still pass once CI-gated)
@hp41-gui/src/App.test.tsx
@hp41-gui/src/Display14Seg.test.tsx
@hp41-gui/src/HelpOverlay.test.tsx
@hp41-gui/src/Keyboard.test.tsx
@hp41-gui/src/pending_input.test.ts

# Backend reference (READ ONLY — no edits; Tauri binary name verification)
@hp41-gui/src-tauri/Cargo.toml
@hp41-gui/src-tauri/tauri.conf.json

<interfaces>
<!-- Key contracts the executor needs. Extracted from the codebase so no
     scavenger hunt is required. -->

# WebdriverIO config (RESEARCH Pattern 5 — verbatim with Ubuntu adaptations):
#   // hp41-gui/wdio.conf.js
#   const os = require('os');
#   const path = require('path');
#   const { spawn, spawnSync } = require('child_process');
#   let tauriDriver;
#   exports.config = {
#     specs: ['./e2e/**/*.spec.ts'],   // or '.js' — verify based on TS support
#     maxInstances: 1,
#     capabilities: [{
#       maxInstances: 1,
#       'tauri:options': {
#         application: '../src-tauri/target/release/hp41-gui',   // verify exact binary name
#       },
#     }],
#     reporters: ['spec'],
#     framework: 'mocha',
#     mochaOpts: { ui: 'bdd', timeout: 60000, retries: 1 },   // D-27.16
#     onPrepare: () => spawnSync('cargo', ['build', '--release', '--manifest-path', '../src-tauri/Cargo.toml']),
#     beforeSession: () => {
#       tauriDriver = spawn(
#         path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
#         [],
#         { stdio: [null, process.stdout, process.stderr] }
#       );
#     },
#     afterSession: () => tauriDriver.kill(),
#   };
#
# IMPORTANT: TypeScript support for wdio configs requires extra setup (ts-node
# preload or a JS wrapper). Recommended: write the config in plain JS
# (wdio.conf.js) and the spec in TS (smoke.spec.ts) with WDIO's built-in TS
# loader. If the executor finds wdio TS spec support requires additional
# devdeps (e.g. @wdio/tsconfig + ts-node), document the additions in the
# SUMMARY but keep the wdio.conf in .js to minimize devDep churn.
# Acceptable alternative: write the spec as .js too if TS preload adds
# complexity — the spec is 15 lines and doesn't benefit much from types.

# Smoke spec (RESEARCH Pattern 5 + Pitfall 9 + Pitfall 10):
#   // hp41-gui/e2e/smoke.spec.ts
#   describe('HP-41 GUI smoke', () => {
#     it('2 ENTER 3 + displays 5.0000', async () => {
#       await $('[data-key-id="2"]').click();
#       await $('[data-key-id="enter"]').click();
#       await $('[data-key-id="3"]').click();
#       await $('[data-key-id="plus"]').click();
#       const display = await $('[data-testid="lcd-display"]');
#       await expect(display).toHaveText('5.0000');
#     });
#   });

# Display14Seg.tsx one-line edit (RESEARCH Pitfall 10):
#   The component renders an SVG-wrapped container at line ~209. The
#   outermost element should carry data-testid="lcd-display". Verify which
#   element makes sense — typically the wrapping <div> or <svg>. If the
#   component's textContent comes from the SEGMENT_MAP path data, the test
#   may need to assert against a different attribute (e.g. data-text) — read
#   the existing Display14Seg.test.tsx to see how the value is exposed in
#   the DOM. If the rendered glyph paths don't expose readable text content
#   in the DOM, ALSO add a `data-text={text}` attribute on the same element
#   so the assertion can read `attr('data-text')` instead of toHaveText().

# Keyboard.tsx — data-key-id confirmed at lines 285 and 303 (RESEARCH §Sources).
# IDs in use (Pitfall 9): '2', 'enter', '3', 'plus' — ALL kebab-case strings,
# NOT glyph characters. The smoke spec selectors match these literals.

# justfile edits (RESEARCH Example 5 + Example 6):
#   coverage:                                       # UNCHANGED in this plan
#     cargo llvm-cov clean --workspace             # (Plan 27-01 owns the 80→95 raise)
#     cargo llvm-cov --fail-under-lines 95 -p hp41-core
#
#   gui-ci:
#     cd hp41-gui && npm install
#     cd hp41-gui && npx tsc --noEmit
#     cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml
#     cargo build --release --manifest-path hp41-gui/src-tauri/Cargo.toml
#     cd hp41-gui && npm test            # NEW (D-27.14)
#
#   # NEW: WebdriverIO + tauri-driver smoke (Linux only — invoked from ci-gui.yml)
#   gui-e2e:
#     cd hp41-gui && npm install
#     cd hp41-gui && npx wdio run wdio.conf.js

# ci-gui.yml e2e-linux job (RESEARCH Example 6 — adapted for D-27.15 AMENDED):
#   e2e-linux:
#     name: GUI E2E (Ubuntu only)
#     runs-on: ubuntu-latest
#     needs: build   # only runs if the matrix build succeeds (any platform)
#     steps:
#       - uses: actions/checkout@v4
#       - uses: dtolnay/rust-toolchain@stable
#       - uses: Swatinem/rust-cache@v2
#         with:
#           workspaces: hp41-gui/src-tauri -> hp41-gui/src-tauri/target
#       - name: Cache cargo bin (tauri-driver)
#         uses: actions/cache@v4
#         with:
#           path: ~/.cargo/bin
#           key: cargo-bin-${{ runner.os }}-tauri-driver-2.0.6
#       - uses: actions/setup-node@v4
#         with: { node-version: 'lts/*' }
#       - uses: taiki-e/install-action@v2
#         with: { tool: just }
#       - name: Install Linux system deps
#         run: |
#           sudo apt-get update
#           sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev \
#             librsvg2-dev patchelf webkit2gtk-driver xvfb
#       - name: Install tauri-driver
#         run: cargo install tauri-driver --locked --version 2.0.6
#       - name: Build release binary
#         run: just gui-build
#       - name: Run E2E smoke under Xvfb
#         run: xvfb-run -a just gui-e2e
#         env:
#           CI: true

# CLAUDE.md update — append to the v2.2 additions section OR create a new
# "### v2.2 additions (Phase 27 Test Hardening)" block. Coordinate with
# Plan 27-01 Task 4 — if Plan 27-01 already added this block, this plan
# APPENDS bullets to the same block. Otherwise this plan CREATES the block
# and Plan 27-01 appends. Either way, the final CLAUDE.md should have a
# single coherent Phase 27 settled-architecture block.

# 5 existing Vitest files (D-27.14 — they pass locally per CONTEXT.md line 103,
# verified passing in Phase 26 ship; this plan just CI-gates them):
#   hp41-gui/src/App.test.tsx              (Phase 26 gap-closure 14.7K, 13 tests)
#   hp41-gui/src/Display14Seg.test.tsx     (Phase 26)
#   hp41-gui/src/HelpOverlay.test.tsx      (Phase 26)
#   hp41-gui/src/Keyboard.test.tsx         (Phase 26)
#   hp41-gui/src/pending_input.test.ts     (Phase 26)
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add `data-testid` on Display14Seg.tsx + verify keyboard selectors</name>

  <files>hp41-gui/src/Display14Seg.tsx</files>

  <read_first>
    - hp41-gui/src/Display14Seg.tsx (the entire file — 209+ lines per the grep above; the outermost element to receive `data-testid` is at line ~209 inside the `Display14Seg` function component's return JSX)
    - hp41-gui/src/Display14Seg.test.tsx (verify how the existing Vitest tests query the component — `getByText` vs `container.querySelector` vs `data-testid`; the new attribute must not break existing tests)
    - hp41-gui/src/Keyboard.tsx lines 280–310 (confirm `data-key-id` attribute is present on the click targets — RESEARCH cites lines 285, 303)
    - hp41-gui/src/App.tsx (verify that App.tsx renders `<Display14Seg text={displayText} />` and that `displayText` derives from `calcState.display_str` so the E2E spec sees `"5.0000"` after a successful `2 ENTER 3 + → 5`)
    - .planning/phases/27-test-hardening/27-RESEARCH.md §Pitfall 10
  </read_first>

  <action>
    1. Open `hp41-gui/src/Display14Seg.tsx`. Locate the outermost JSX element returned by the `Display14Seg` function component (line ~209 per the grep `<svg`; actual root may be a `<div>` wrapping the `<svg>`, or the `<svg>` itself — confirm via Read).

    2. Add the attribute `data-testid="lcd-display"` to that element. ALSO add a `data-text={text}` attribute on the SAME element (where `text` is the prop being rendered). The dual attribute serves two purposes: (a) `data-testid` is the WebdriverIO selector hook per Pitfall 10; (b) `data-text` makes the rendered value queryable as a DOM attribute even when the LCD's textContent is unreadable (because Display14Seg renders SVG segment paths, not a plain text node — the WebdriverIO `toHaveText('5.0000')` assertion may fail if textContent is empty; `toHaveAttribute('data-text', '5.0000')` is the fallback).

    3. Example diff shape (verify the exact line and existing JSX during Read):
       ```diff
        export default function Display14Seg({ text }: Display14SegProps) {
          // ... existing logic ...
          return (
       -    <div className="display" >
       +    <div className="display" data-testid="lcd-display" data-text={text}>
              <svg ...>
                ...
              </svg>
            </div>
          );
        }
       ```
       The exact JSX shape (whether `<div>` wraps `<svg>`, or `<svg>` is the root) determines which element receives the attributes. If the root is `<svg>` directly, attach to it.

    4. Verify the existing `Display14Seg.test.tsx` Vitest suite still passes:
       - `cd hp41-gui && npm test -- Display14Seg.test.tsx`
       - The new attributes are passive — existing tests using `container.querySelector('.display')` or `getByRole('img')` are unaffected.

    5. Run the full Vitest suite as a regression check: `cd hp41-gui && npm test`. All 5 files should still pass (the 121-test baseline per CONTEXT.md line 103 / `26-04-PLAN.md` line 300).

    Self-check after Task 1:
    - `grep -c 'data-testid="lcd-display"' hp41-gui/src/Display14Seg.tsx` returns 1.
    - `grep -c 'data-text=' hp41-gui/src/Display14Seg.tsx` returns ≥ 1.
    - `cd hp41-gui && npx tsc --noEmit` clean.
    - `cd hp41-gui && npm test` passes (no regressions).
    - `grep -n "fn op_\|fn flush_entry\|fn format_hpnum" hp41-gui/src-tauri/src/` shows only the existing `op_display_name` formatter — SC-4 invariant preserved.
  </action>

  <verify>
    <automated>cd hp41-gui && npx tsc --noEmit && npm test 2>&1 | tail -10</automated>
  </verify>

  <done>
    `Display14Seg.tsx` has `data-testid="lcd-display"` + `data-text={text}` on its outermost rendered element; existing Vitest suite still passes; TypeScript clean; SC-4 invariant preserved (no `hp41-gui/src-tauri/` changes).
  </done>
</task>

<task type="auto">
  <name>Task 2: Install WebdriverIO devdeps + create wdio.conf.js + smoke.spec.ts</name>

  <files>hp41-gui/package.json, hp41-gui/wdio.conf.js, hp41-gui/e2e/smoke.spec.ts</files>

  <read_first>
    - hp41-gui/package.json (verify current devDependencies; the WDIO packages must NOT conflict with existing vitest/jsdom)
    - .planning/phases/27-test-hardening/27-RESEARCH.md §Pattern 5 (the full WebdriverIO config template — copy verbatim with the Ubuntu adaptations)
    - .planning/phases/27-test-hardening/27-RESEARCH.md §Pitfall 9 (selector IDs verified)
    - .planning/phases/27-test-hardening/27-RESEARCH.md §Pitfall 10 (data-testid hook just landed in Task 1)
    - .planning/phases/27-test-hardening/27-CONTEXT.md D-27.15 AMENDED + D-27.16
    - hp41-gui/src-tauri/Cargo.toml (verify the binary product name — likely `hp41-gui` or `hp41-gui-bin`; the wdio.conf.js `tauri:options.application` path must match the actual release binary at `hp41-gui/src-tauri/target/release/<binary-name>`)
    - hp41-gui/src-tauri/tauri.conf.json (verify the `productName` / `mainBinaryName` if Tauri 2.11 uses one of those keys to derive the binary name)
  </read_first>

  <action>
    **2.1 — Edit `hp41-gui/package.json` devDependencies:**

    Add the following entries to `devDependencies` (in alphabetical order — npm sorts these on install):
    ```
    "@wdio/cli": "^9.19",
    "@wdio/local-runner": "^9.19",
    "@wdio/mocha-framework": "^9.19",
    "@wdio/spec-reporter": "^9.19",
    "webdriverio": "^9.19"
    ```
    Do NOT add Playwright (`@playwright/test`) — D-27.15 AMENDED replaces Playwright with WebdriverIO.

    Run `cd hp41-gui && npm install` to install. The lockfile (`package-lock.json` if present, or yarn.lock) will update — commit the lockfile change.

    **2.2 — Create `hp41-gui/wdio.conf.js`:**

    Use RESEARCH Pattern 5 verbatim, adapted for D-27.16 (1 retry) and the verified Tauri binary path:

    ```javascript
    // WebdriverIO + tauri-driver E2E config (D-27.15 AMENDED).
    // Drives the production Tauri binary on Ubuntu (WebKitGTK).
    // FN-QUAL-05 + Plan 27-04.

    const os = require('os');
    const path = require('path');
    const { spawn, spawnSync } = require('child_process');

    let tauriDriver;

    exports.config = {
      specs: ['./e2e/**/*.spec.ts'],
      maxInstances: 1,
      capabilities: [{
        maxInstances: 1,
        'tauri:options': {
          application: '../src-tauri/target/release/<BINARY-NAME>',  // resolved during read_first
        },
      }],
      reporters: ['spec'],
      framework: 'mocha',
      mochaOpts: {
        ui: 'bdd',
        timeout: 60000,
        retries: 1,   // D-27.16
      },

      // Spawn tauri-driver before each session; kill after.
      beforeSession: () => {
        tauriDriver = spawn(
          path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
          [],
          { stdio: [null, process.stdout, process.stderr] }
        );
      },
      afterSession: () => {
        if (tauriDriver) tauriDriver.kill();
      },
    };
    ```

    Replace `<BINARY-NAME>` with the actual binary name verified during read_first (e.g. `hp41-gui` if `[[bin]] name = "hp41-gui"` in Cargo.toml, or whatever `productName` is in `tauri.conf.json` if Tauri uses that key).

    NOTE: the original RESEARCH Pattern 5 had an `onPrepare` step that builds the release binary. We REMOVE that from `wdio.conf.js` because Task 4's CI workflow runs `just gui-build` before `just gui-e2e` — the CI step builds explicitly, and `wdio.conf.js` should not re-build (redundant + slow). For local developer runs (Task 3's `gui-e2e:` recipe), the developer is expected to have run `just gui-build` first (this is documented as a precondition in the recipe comment).

    Use TypeScript in the spec file (.ts), JS in the config (.js). WDIO 9 supports `.ts` spec files via its built-in TS loader when `tsconfig.json` is present and `@types/mocha` types are available. If TS-spec support requires additional devdeps (e.g. `@wdio/tsconfig` or `ts-node`), document the additions in the SUMMARY and add them to `package.json` — OR fall back to a `.js` spec and remove `.ts` from the `specs` glob. The 15-line smoke does not strongly benefit from types; .js is acceptable.

    **2.3 — Create `hp41-gui/e2e/smoke.spec.ts`** (use `.ts` if WDIO TS loader works out-of-the-box; fall back to `.js` if not — both versions follow the same structure):

    ```typescript
    // FN-QUAL-05 smoke: boots the Tauri release binary, drives via tauri-driver,
    // asserts the literal ROADMAP success criterion `2 ENTER 3 + → 5.0000`.
    // Selectors: data-key-id on Keyboard.tsx (verified at lines 285, 303);
    //            data-testid on Display14Seg.tsx (added in Plan 27-04 Task 1).

    describe('HP-41 GUI smoke', () => {
      it('2 ENTER 3 + displays 5.0000', async () => {
        await $('[data-key-id="2"]').click();
        await $('[data-key-id="enter"]').click();
        await $('[data-key-id="3"]').click();
        await $('[data-key-id="plus"]').click();

        const display = await $('[data-testid="lcd-display"]');

        // Display14Seg renders SVG segment paths, not plain text. Prefer
        // data-text attribute (added alongside data-testid in Task 1) — if
        // the rendered DOM exposes text content directly, toHaveText also
        // works. Choose the assertion based on what's actually in the DOM
        // after Task 1; both are acceptable.
        const text = await display.getAttribute('data-text');
        if (text !== null) {
          if (text !== '5.0000') {
            throw new Error(`expected lcd-display data-text='5.0000', got '${text}'`);
          }
        } else {
          await expect(display).toHaveText('5.0000');
        }
      });
    });
    ```

    Verify the assertion shape against the actual rendered DOM (run the test once locally if Linux is available; CI will catch any divergence on the first push). The fallback to `toHaveText` is defensive.

    **2.4 — Verify the directory exists:**

    `mkdir -p hp41-gui/e2e` if it doesn't (the directory should be tracked once smoke.spec.ts exists — git auto-tracks the file).

    Self-check after Task 2:
    - `cd hp41-gui && cat package.json | grep -c '@wdio/' ` returns ≥ 4 (the four @wdio packages).
    - `cd hp41-gui && cat package.json | grep -c 'webdriverio'` returns ≥ 1.
    - `test -f hp41-gui/wdio.conf.js` exits 0.
    - `test -f hp41-gui/e2e/smoke.spec.ts || test -f hp41-gui/e2e/smoke.spec.js` exits 0 (allow .ts or .js depending on WDIO TS support).
    - `grep -c "mochaOpts" hp41-gui/wdio.conf.js` returns ≥ 1.
    - `grep -c "retries: 1" hp41-gui/wdio.conf.js` returns 1 (D-27.16).
    - `grep -c "data-key-id=\"2\"\|data-key-id=\"enter\"\|data-key-id=\"3\"\|data-key-id=\"plus\"" hp41-gui/e2e/smoke.spec.*` returns 4.
    - `cd hp41-gui && npx tsc --noEmit` still clean (no TS errors introduced by the spec — TS spec compiles independently of the config).
    - `cd hp41-gui && npm test` still passes (Vitest only — WDIO doesn't run under `npm test`).

    **DO NOT** attempt to run `npx wdio run wdio.conf.js` locally if not on Linux with WebKitGTK + tauri-driver installed. The CI workflow in Task 4 is where the spec actually executes; local execution is a developer-side option.
  </action>

  <verify>
    <automated>cd hp41-gui && npx tsc --noEmit && grep -q "webdriverio" package.json && grep -q "retries: 1" wdio.conf.js && (test -f e2e/smoke.spec.ts || test -f e2e/smoke.spec.js) && echo "OK"</automated>
  </verify>

  <done>
    `package.json` includes the 5 WDIO devdeps at ^9.19; `wdio.conf.js` exists pointing at the verified Tauri release binary path with `mochaOpts.retries: 1`; `e2e/smoke.spec.ts` (or .js) exists with the 4 click selectors and the `data-testid`/`data-text` assertion; TypeScript still clean; Vitest still passes.
  </done>
</task>

<task type="auto">
  <name>Task 3: Justfile edits — gui-ci appends `npm test` (D-27.14); new gui-e2e recipe</name>

  <files>justfile</files>

  <read_first>
    - justfile (the current `gui-ci:` recipe at lines 82–87 and the docs-matrix recipes at lines 89–100 — match the style)
    - .planning/phases/27-test-hardening/27-CONTEXT.md D-27.14
    - .planning/phases/27-test-hardening/27-RESEARCH.md §Example 5 (the justfile delta — note that Example 5 already has the coverage 80→95 raise, but Plan 27-01 owns that edit; this plan's edits are only to gui-ci: and the new gui-e2e:)
  </read_first>

  <action>
    Edit `justfile`. Two changes:

    **3.1 — Append `npm test` to `gui-ci:` (D-27.14):**

    Current lines 82–87:
    ```
    # gui-ci: CI gate — TypeScript type-check, Rust tests, and release build (called from ci-gui.yml)
    gui-ci:
    	cd hp41-gui && npm install
    	cd hp41-gui && npx tsc --noEmit
    	cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml
    	cargo build --release --manifest-path hp41-gui/src-tauri/Cargo.toml
    ```

    Append one line after `cargo build --release ...`:
    ```
    	cd hp41-gui && npm test
    ```

    Update the doc comment (line 82) to reflect the new gate:
    ```
    # gui-ci: CI gate — TypeScript type-check, Rust tests, release build, Vitest (D-27.14)
    ```

    **3.2 — Add new `gui-e2e:` recipe** (after `gui-ci:`, before `docs-matrix:`):

    ```
    # gui-e2e: WebdriverIO + tauri-driver smoke (Linux only — invoked from ci-gui.yml).
    # Precondition: `just gui-build` has produced hp41-gui/src-tauri/target/release/<binary>.
    # Precondition: tauri-driver is installed (`cargo install tauri-driver --locked --version 2.0.6`)
    #               and webkit2gtk-driver is on PATH (apt-get install webkit2gtk-driver).
    # Precondition: when on Ubuntu with no display, wrap with xvfb-run -a.
    gui-e2e:
    	cd hp41-gui && npm install
    	cd hp41-gui && npx wdio run wdio.conf.js
    ```

    Indentation: TAB characters per Just convention (verify existing recipes use tabs, not spaces).

    Self-check after Task 3:
    - `grep -A 6 "^gui-ci:" justfile | grep -c "npm test"` returns 1.
    - `grep -c "^gui-e2e:" justfile` returns 1.
    - `just --list 2>&1 | grep -E "gui-ci|gui-e2e"` shows both recipes.
    - DO NOT run `just gui-ci` or `just gui-e2e` locally if the platform isn't Ubuntu with the required setup; the CI workflow in Task 4 is the verification surface.
  </action>

  <verify>
    <automated>just --list 2>&1 | grep -E "gui-ci|gui-e2e" | wc -l</automated>
  </verify>

  <done>
    `justfile gui-ci:` recipe appends `npm test` (D-27.14); new `gui-e2e:` recipe exists with documented preconditions; `just --list` shows both recipes.
  </done>
</task>

<task type="auto">
  <name>Task 4: ci-gui.yml — new e2e-linux job (Ubuntu only, required, 1 retry) per D-27.15 AMENDED + D-27.16</name>

  <files>.github/workflows/ci-gui.yml</files>

  <read_first>
    - .github/workflows/ci-gui.yml (the current 51-line file with the single `build` job, matrix on ubuntu/macos/windows)
    - .planning/phases/27-test-hardening/27-RESEARCH.md §Example 6 (the full ci-gui.yml e2e-linux job template)
    - .planning/phases/27-test-hardening/27-RESEARCH.md §Pitfalls 5, 6, 7 (cargo bin cache, webkit2gtk-driver apt, atomicity — note Pitfall 7 is about coverage gate, not E2E)
    - .planning/phases/27-test-hardening/27-CONTEXT.md D-27.15 AMENDED + D-27.16
  </read_first>

  <action>
    Edit `.github/workflows/ci-gui.yml`. Two changes:

    **4.1 — Add `webkit2gtk-driver` + `xvfb` to the existing apt-get line (Pitfall 6):**

    Current line 47:
    ```
          - name: Install Linux system deps
            if: matrix.os == 'ubuntu-latest'
            run: sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
    ```

    Change to (add `webkit2gtk-driver xvfb` to the end of the apt-get install line):
    ```
          - name: Install Linux system deps
            if: matrix.os == 'ubuntu-latest'
            run: sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf webkit2gtk-driver xvfb
    ```

    NOTE: this changes the Linux matrix-build job's apt step too, so `webkit2gtk-driver` + `xvfb` are available on the existing build runner. The new `e2e-linux` job repeats the apt install to be self-contained (matrix and e2e are different jobs; CI mounts a fresh runner per job).

    **4.2 — Add new `e2e-linux` job after the existing `build:` job:**

    Append this job to the `jobs:` section:

    ```yaml
      e2e-linux:
        name: GUI E2E (Ubuntu only — WebdriverIO + tauri-driver)
        runs-on: ubuntu-latest
        needs: build   # only run after the matrix build is green (D-27.15 AMENDED)
        steps:
          - uses: actions/checkout@v4

          - uses: dtolnay/rust-toolchain@stable

          - uses: Swatinem/rust-cache@v2
            with:
              workspaces: hp41-gui/src-tauri -> hp41-gui/src-tauri/target

          - name: Cache cargo bin (tauri-driver — Pitfall 5)
            uses: actions/cache@v4
            with:
              path: ~/.cargo/bin
              key: cargo-bin-${{ runner.os }}-tauri-driver-2.0.6

          - uses: actions/setup-node@v4
            with:
              node-version: 'lts/*'

          - uses: taiki-e/install-action@v2
            with:
              tool: just

          - name: Install Linux system deps (Pitfall 6 — webkit2gtk-driver)
            run: |
              sudo apt-get update
              sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev \
                librsvg2-dev patchelf webkit2gtk-driver xvfb

          - name: Install tauri-driver
            run: cargo install tauri-driver --locked --version 2.0.6

          - name: Build release binary
            run: just gui-build

          - name: Run E2E smoke under Xvfb (Assumption A5)
            run: xvfb-run -a just gui-e2e
            env:
              CI: true
    ```

    Style notes:
    - Indentation: 2-space YAML; align with the existing `build:` job's structure.
    - `needs: build` ensures e2e-linux only runs after the matrix build (all 3 platforms) is green.
    - `cache` action keyed on `tauri-driver-2.0.6` per Pitfall 5 — bumping the version pin invalidates the cache automatically.
    - `xvfb-run -a` per RESEARCH Assumption A5 — defensive; if the GitHub Ubuntu runner has a display already, the wrapper is harmless.

    **4.3 — Verify the job is REQUIRED for merge:**

    GitHub branch-protection rules are configured in the repo settings (web UI / GraphQL API), not in the YAML. The plan's job-naming requirement is: when configuring branch protection for `develop` or `main`, the user adds `GUI E2E (Ubuntu only — WebdriverIO + tauri-driver) / e2e-linux` to the required-status-checks list. Document this expectation in `27-04-SUMMARY.md` as a manual post-merge follow-up; the YAML alone cannot enforce the required-for-merge constraint.

    Self-check after Task 4:
    - `grep -c "^  e2e-linux:" .github/workflows/ci-gui.yml` returns 1.
    - `grep -c "needs: build" .github/workflows/ci-gui.yml` returns 1 (in the e2e-linux job).
    - `grep -c "webkit2gtk-driver" .github/workflows/ci-gui.yml` returns ≥ 2 (matrix build job + e2e-linux job).
    - `grep -c "tauri-driver" .github/workflows/ci-gui.yml` returns ≥ 2 (cache key + cargo install).
    - `grep -c "xvfb-run" .github/workflows/ci-gui.yml` returns 1.
    - `grep -c "just gui-e2e" .github/workflows/ci-gui.yml` returns 1.
    - YAML lint passes: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci-gui.yml'))"` exits 0 (or use `actionlint` if installed). DO NOT push and trigger a real CI run until ALL Tasks 1–5 land in the same commit batch; the e2e-linux job will FAIL on its first run if `wdio.conf.js` or `smoke.spec.ts` is missing.

    **CRITICAL ATOMICITY:** the workflow edit, the wdio.conf.js, the smoke.spec.ts, the Display14Seg.tsx edit, and the package.json deps must ALL be present in the same git push (one or multiple commits within the same PR is fine; an isolated push that adds the e2e-linux job without the WDIO setup will hard-fail CI). If the executor commits per-task, the LAST commit of this plan (Task 6 CLAUDE.md update) is the push trigger; pushing earlier risks a red CI signal.
  </action>

  <verify>
    <automated>python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci-gui.yml'))" && grep -c "e2e-linux:" .github/workflows/ci-gui.yml</automated>
  </verify>

  <done>
    `ci-gui.yml` has a new `e2e-linux` job (Ubuntu only, depends on build, with webkit2gtk-driver + xvfb apt, tauri-driver cargo install + cache, xvfb-run wrapper); YAML lint clean; the e2e-linux job is the manual required-for-merge target documented in the SUMMARY.
  </done>
</task>

<task type="auto">
  <name>Task 5: GUI coverage one-shot measurement (D-27.4, measure-only)</name>

  <files>(no files modified — measurement only; results recorded in 27-04-SUMMARY.md after the plan ships)</files>

  <read_first>
    - .planning/phases/27-test-hardening/27-CONTEXT.md D-27.4 (measure-only, no CI gate, no devDep)
    - hp41-gui/vite.config.ts (Vitest config — see if `coverage` is already configured; if not, the one-shot run uses `--coverage` flag directly)
    - hp41-gui/src-tauri/Cargo.toml (the manifest path for `cargo llvm-cov` on the Rust side)
  </read_first>

  <action>
    Per D-27.4 this is a one-shot measurement during plan execution — no CI gate, no permanent devDep, no justfile recipe.

    **5.1 — Measure Vitest coverage on `hp41-gui/src/`:**

    ```
    cd hp41-gui && npx vitest run --coverage --reporter=verbose 2>&1 | tail -30
    ```

    Vitest 4 supports `--coverage` via v8 (the default provider on Vitest 4+). If the command fails with "provider not installed" or similar, the v8 provider may need an explicit `@vitest/coverage-v8` devDep. Per D-27.4 the planner does NOT add a devDep — instead, record the absence in the SUMMARY as "GUI Vitest coverage NOT measured this phase; D-27.4 measure-only path required `@vitest/coverage-v8` which would violate the no-devDep constraint. Deferred to v3.x."

    If Vitest 4 already has v8 bundled (verify during execution), record the output:
    - Total line coverage % across `hp41-gui/src/`
    - Per-file breakdown for the 5 source files with Vitest test partners (App.tsx, Display14Seg.tsx, HelpOverlay.tsx, Keyboard.tsx, pending_input.ts)

    **5.2 — Measure cargo llvm-cov on `hp41-gui/src-tauri/`:**

    ```
    cargo llvm-cov clean --workspace
    cargo llvm-cov --manifest-path hp41-gui/src-tauri/Cargo.toml --text 2>&1 | tail -30
    ```

    Record:
    - Total line coverage % on `hp41-gui/src-tauri/src/`
    - Function coverage %
    - Region coverage %

    **5.3 — Record the numbers in `27-04-SUMMARY.md`** (written after all tasks complete):

    Add a `## GUI Coverage Snapshot (D-27.4 — measure only, NOT a gate)` section with the measured numbers. Explicitly note: "Per D-27.4, GUI coverage is OUT of scope for Phase 27 CI gating. These numbers are advisory for v3.x roadmap reference. No CI gate added. No devDep added (Vitest's bundled v8 provider used if available; otherwise measurement skipped per D-27.4 no-devDep clause)."

    Self-check after Task 5:
    - Vitest coverage measured (or explicit "skipped per D-27.4 no-devDep" note recorded).
    - cargo llvm-cov on hp41-gui/src-tauri measured.
    - No new entries in `hp41-gui/package.json devDependencies`.
    - No new `justfile` recipe for GUI coverage.
    - No new gate in `ci-gui.yml`.
    - Numbers ready for the SUMMARY in Task 6.
  </action>

  <verify>
    <automated>cd hp41-gui && grep -c "@vitest/coverage" package.json || echo "0 — coverage devDep correctly absent per D-27.4"</automated>
  </verify>

  <done>
    Vitest and cargo llvm-cov coverage numbers measured for `hp41-gui/`; recorded in the planner's scratch buffer for inclusion in `27-04-SUMMARY.md`; NO devDep added; NO CI gate added; D-27.4 measure-only constraint honored.
  </done>
</task>

<task type="auto">
  <name>Task 6: CLAUDE.md update — Phase 27 settled-architecture block (WebdriverIO, Vitest gating, data-testid, GUI coverage snapshot)</name>

  <files>CLAUDE.md</files>

  <read_first>
    - CLAUDE.md "## Settled Architecture Decisions" section (locate the v2.2 additions block — Phase 21–25 sub-block; the Phase 27 block follows immediately after)
    - CLAUDE.md "## Quality Gates" table (verify whether Plan 27-01 already updated the coverage row; if so, this plan only adds the WebdriverIO / Vitest gating note)
    - .planning/phases/27-test-hardening/27-CONTEXT.md D-27.14, D-27.15 AMENDED, D-27.16
    - Your scratch buffer with the GUI coverage snapshot numbers from Task 5
  </read_first>

  <action>
    Edit `CLAUDE.md`. The Phase 27 settled-architecture block coordinates between Plan 27-01 (which adds the coverage / accuracy bullets) and this plan (which adds the E2E / Vitest gating / data-testid / GUI coverage bullets).

    **Branch A — Plan 27-01 has not yet shipped its CLAUDE.md edit:**

    Add a new sub-block `### v2.2 additions (Test Hardening, Phase 27)` after the existing `### v2.2 additions (HP-41CV Feature Completeness, Phases 20–25)` block. Content:

    ```
    ### v2.2 additions (Test Hardening, Phase 27)

    - **E2E smoke via WebdriverIO + tauri-driver** (D-27.15 AMENDED 2026-05-15, FN-QUAL-05): the original D-27.15 named Playwright, but `tauri-driver` 2.0.6 speaks WebDriver classic which Playwright does NOT (CDP/native only). The spirit of D-27.15 (production binary + real IPC + Ubuntu only) is preserved via WebdriverIO 9.x — the Tauri v2 official E2E client. `hp41-gui/wdio.conf.js` spawns `tauri-driver` on `127.0.0.1:4444` with `mochaOpts.retries: 1` (D-27.16). `hp41-gui/e2e/smoke.spec.ts` clicks `2 ENTER 3 +` and asserts `[data-testid="lcd-display"]` reads `5.0000`. Runs ONLY on Ubuntu in `ci-gui.yml::e2e-linux` (ROADMAP cross-cutting constraint); macOS/Windows matrix jobs unchanged.
    - **Vitest CI gating** (D-27.14): the existing 5 Vitest files (`App.test.tsx`, `Display14Seg.test.tsx`, `HelpOverlay.test.tsx`, `Keyboard.test.tsx`, `pending_input.test.ts`) now gate on every CI push via `just gui-ci` appended `cd hp41-gui && npm test`. They pass locally since Phase 26 ship; the CI gate closes a quiet hole.
    - **`data-testid="lcd-display"` on `Display14Seg.tsx`** (RESEARCH Pitfall 10): one-line edit allowed under SC-4 because `hp41-gui/src/` is OUTSIDE the SC-4 boundary (which constrains `hp41-gui/src-tauri/` only). The dual `data-text={text}` attribute is the fallback WebdriverIO assertion path since the LCD renders SVG segment paths (no plain text content).
    - **GUI coverage measured one-shot** (D-27.4): Vitest line coverage on `hp41-gui/src/` = NN.N %; cargo llvm-cov on `hp41-gui/src-tauri/` = NN.N %. Measure-only snapshot for v3.x reference; NO CI gate, NO devDep added.
    - **Apt deps added to Ubuntu runner**: `webkit2gtk-driver` (Pitfall 6) + `xvfb` (Assumption A5) appended to the existing libwebkit2gtk-4.1-dev install line in `ci-gui.yml`. Cargo bin cache (Pitfall 5) for `tauri-driver` 2.0.6 lives at `~/.cargo/bin`.
    - **Frozen invariants preserved**: NO `hp41-core/src/` changes; NO `hp41-gui/src-tauri/` changes (SC-4); MSRV 1.88 unchanged.
    ```

    Replace `NN.N %` placeholders with the actual numbers from Task 5.

    **Branch B — Plan 27-01 already added its block:**

    Plan 27-01 already created `### v2.2 additions (Test Hardening, Phase 27)` with coverage + accuracy bullets. This plan APPENDS the additional bullets (E2E, Vitest, data-testid, GUI coverage, apt deps) to the SAME block. The "Frozen invariants preserved" bullet is collapsed if both plans had it; one canonical instance.

    The end-state CLAUDE.md should have a single coherent `### v2.2 additions (Test Hardening, Phase 27)` block containing:
    1. Coverage gate raise (Plan 27-01 Task 4)
    2. Numerical accuracy extension (Plan 27-01)
    3. Flag-semantics proptest (Plan 27-02 — optional addition, not strictly required in CLAUDE.md)
    4. IND integration suite (Plan 27-03 — optional addition)
    5. E2E smoke via WebdriverIO + tauri-driver (this plan)
    6. Vitest CI gating (this plan)
    7. `data-testid` on Display14Seg.tsx (this plan)
    8. GUI coverage measure-only snapshot (this plan)
    9. Apt deps added to Ubuntu runner (this plan)
    10. Frozen invariants preserved (one canonical line)

    Plans 27-02 and 27-03 are TEST-ONLY additions that don't change settled architecture — they may not need their own CLAUDE.md bullets. Use planner discretion: if the proptest paradigm or the IND test layout would benefit future-readers as architectural facts, include them; otherwise the test-file paths in the "Key Files" table at the bottom of CLAUDE.md are sufficient.

    **Add WebdriverIO config to the "Key Files" table** in CLAUDE.md (the third-to-bottom section listing key files per crate). Append a row under GUI:
    ```
    | `hp41-gui/wdio.conf.js` | NEW (Phase 27) — WebdriverIO + tauri-driver smoke config; mochaOpts.retries=1 (D-27.16); Ubuntu-only |
    | `hp41-gui/e2e/smoke.spec.ts` | NEW (Phase 27) — FN-QUAL-05 literal ROADMAP smoke: 2 ENTER 3 + → 5.0000 |
    ```

    Self-check after Task 6:
    - `grep -c "Test Hardening, Phase 27" CLAUDE.md` returns 1 (a single coherent block).
    - `grep -c "WebdriverIO" CLAUDE.md` returns ≥ 1 (the E2E note).
    - `grep -c "data-testid.*lcd-display" CLAUDE.md` returns ≥ 1.
    - `grep -c "D-27.15 AMENDED" CLAUDE.md` returns ≥ 1 (the amendment audit trail).
    - `grep -c "wdio.conf.js\|e2e/smoke.spec" CLAUDE.md` returns ≥ 1 (Key Files table).
    - `grep -c "frozen\|FROZEN\|SC-4" CLAUDE.md` shows the existing invariant notes are intact.

    **Final commit (FROM ALL Tasks 1–6 in this plan):** the commit message MUST reference D-27.15 AMENDED to record the Playwright → WebdriverIO substitution in the git history. Example: `test(27-04): land WebdriverIO + tauri-driver E2E smoke + Vitest CI gating (D-27.14, D-27.15 AMENDED, D-27.16, FN-QUAL-05)`.
  </action>

  <verify>
    <automated>grep -c "Test Hardening, Phase 27" CLAUDE.md && grep -c "WebdriverIO\|D-27.15 AMENDED" CLAUDE.md</automated>
  </verify>

  <done>
    `CLAUDE.md` has a coherent Phase 27 settled-architecture block recording WebdriverIO Ubuntu-only scope, Vitest CI gating, data-testid edit, GUI coverage snapshot, apt deps; Key Files table updated with wdio.conf.js + e2e/smoke.spec.*; commit message references D-27.15 AMENDED.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Tauri IPC (`dispatch_op` invoke) | The E2E spec exercises this real path through the production binary (NOT a Vite dev server with mocked `invoke`). Real IPC = real attack surface; the smoke is a one-shot canary, not a fuzzing tool. |
| `tauri-driver` WebDriver server | Listens on `127.0.0.1:4444` during E2E job. Localhost-only; not exposed to the public internet. Apt-installed `webkit2gtk-driver` is the bridge. |
| `cargo install tauri-driver --locked` | Cargo's checksum validation via `Cargo.lock` blocks tampered downloads (RESEARCH §Known Threat Patterns). |
| Display14Seg.tsx `data-testid` attribute | New DOM attribute on the LCD container; React auto-escapes attribute values. No XSS surface (test-only hook, not user-controlled). |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-27-04-01 | Tampering | `tauri-driver` cargo install in CI | mitigate | `--locked` flag enforces Cargo.lock checksums; `--version 2.0.6` pin prevents semver-rollover surprises. |
| T-27-04-02 | Denial of Service | E2E job runtime in CI | mitigate | RESEARCH Assumption A6: literal smoke completes in ~3–5 min cached / 6–8 min cold. `tauri-driver` cargo binary cache (Pitfall 5) keeps cold-install rare. Single retry (D-27.16) tolerates infra hiccups without masking real regressions. |
| T-27-04-03 | Information Disclosure | E2E test logs | accept | The smoke captures only DOM attribute values (5.0000) and click events. No secrets in test scope. WebdriverIO's spec reporter is stdout-only. |
| T-27-04-04 | Tampering | Branch protection (required-for-merge enforcement) | accept | The YAML alone cannot mark a job required-for-merge; that's a repo-settings configuration. The plan documents the manual follow-up in the SUMMARY. Until configured, the e2e-linux job is informational on PRs. |
</threat_model>

<verification>
## Phase-level checks (run after all Tasks 1–6 land)

- `cd hp41-gui && npm install && npx tsc --noEmit && npm test` exits 0 (all 5 Vitest files pass).
- `grep -c 'data-testid="lcd-display"' hp41-gui/src/Display14Seg.tsx` returns 1 (Pitfall 10 mitigation).
- `cd hp41-gui && cat package.json | python3 -c "import sys,json; d=json.load(sys.stdin); assert 'webdriverio' in d['devDependencies']; assert '@wdio/cli' in d['devDependencies']; print('OK')"` exits 0.
- `test -f hp41-gui/wdio.conf.js && grep -q "retries: 1" hp41-gui/wdio.conf.js` exits 0 (D-27.16).
- `test -f hp41-gui/e2e/smoke.spec.ts || test -f hp41-gui/e2e/smoke.spec.js` exits 0.
- `grep -c "e2e-linux:" .github/workflows/ci-gui.yml` returns 1.
- `grep -c "webkit2gtk-driver" .github/workflows/ci-gui.yml` returns ≥ 2 (matrix build + e2e-linux).
- `grep -c "xvfb-run -a just gui-e2e" .github/workflows/ci-gui.yml` returns 1.
- `grep -A 6 "^gui-ci:" justfile | grep -c "npm test"` returns 1 (D-27.14).
- `grep -c "^gui-e2e:" justfile` returns 1.
- YAML lint: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci-gui.yml'))"` exits 0.
- `grep -c "Test Hardening, Phase 27" CLAUDE.md` returns 1.
- `grep -c "WebdriverIO\|D-27.15 AMENDED" CLAUDE.md` returns ≥ 1.
- `grep -rn "fn op_\|fn flush_entry\|fn format_hpnum" hp41-gui/src-tauri/src/` shows ONLY the pre-existing `op_display_name` formatter — SC-4 invariant preserved (no new matches added by this plan).
- **CI verification:** after the commit batch pushes, the `ci-gui.yml::e2e-linux` job runs on the GitHub Ubuntu runner and reports green. If the first run fails for a tooling reason (apt package mismatch, tauri-driver install failure), iterate on the workflow only; do NOT modify the spec or the Display14Seg edit unless the failure is in the spec itself.

## Nyquist verification dimensions (record in plan SUMMARY)

- **Behavioral:** the smoke spec runs on the Ubuntu CI runner and asserts `5.0000` — the literal ROADMAP success criterion line 200. Captured automatically by the CI job; manual local verification optional for non-Linux developers.
- **Functional:** all 5 existing Vitest files pass under `just gui-ci` (D-27.14 hole closed); the smoke spec exercises the Tauri ↔ React ↔ hp41-core dispatch chain end-to-end.
- **Regression:** any future change that breaks the dispatch chain (wrong IPC key, missing dispatch_op handler, Display14Seg refactor without data-testid) fails the smoke job. The required-for-merge configuration (manual repo-settings follow-up) ensures regressions block merges.
</verification>

<success_criteria>
- [x] FN-QUAL-05 satisfied: WebdriverIO + tauri-driver smoke runs in ci-gui.yml::e2e-linux on Ubuntu and asserts `5.0000` after `2 ENTER 3 +`
- [x] D-27.14 satisfied: `just gui-ci` includes `npm test` so the existing 5 Vitest files gate on every CI push
- [x] D-27.15 AMENDED honored: WebdriverIO replaces Playwright; the amendment audit trail is in CLAUDE.md
- [x] D-27.16 satisfied: `mochaOpts.retries: 1` in wdio.conf.js
- [x] D-27.4 honored: GUI coverage measured one-shot; numbers in 27-04-SUMMARY.md; NO CI gate; NO devDep added
- [x] RESEARCH Pitfalls 5, 6, 9, 10 mitigated (cargo bin cache, webkit2gtk-driver apt, selector ID verification, data-testid added)
- [x] SC-4 invariant preserved: no `hp41-gui/src-tauri/` source changes
- [x] MSRV 1.88 unchanged
- [x] Ubuntu-only constraint preserved: macOS/Windows matrix jobs untouched
</success_criteria>

<output>
After completion, create `.planning/phases/27-test-hardening/27-04-SUMMARY.md` recording:
- WDIO devdeps installed + package-lock.json diff size
- wdio.conf.js full path + retries setting
- e2e/smoke.spec.ts (or .js) lines of code + selector inventory (4 click selectors + 1 assertion selector)
- Display14Seg.tsx diff (the one-line `data-testid` + `data-text` addition)
- justfile diff: `gui-ci` appended line + new `gui-e2e` recipe
- ci-gui.yml diff: new e2e-linux job + webkit2gtk-driver/xvfb additions to existing apt line
- GUI coverage measurement results per D-27.4:
  - Vitest line coverage on `hp41-gui/src/` = NN.N % (or "skipped per D-27.4 no-devDep clause")
  - cargo llvm-cov on `hp41-gui/src-tauri/` = NN.N %
- Manual follow-up: configure GitHub branch protection on `develop` and `main` to mark `GUI E2E (Ubuntu only — WebdriverIO + tauri-driver) / e2e-linux` as a required-status-check (the YAML alone cannot enforce this)
- Confirmation of the D-27.15 AMENDED audit trail in CLAUDE.md
- Final commit hash and CI run URL (after push)
</output>

<failure_modes>
## Failure Modes & Mitigations

- **Tauri binary name mismatch:** `wdio.conf.js`'s `tauri:options.application` path must match the actual binary at `hp41-gui/src-tauri/target/release/<binary-name>`. Verify against `Cargo.toml [[bin]] name = ...` OR `tauri.conf.json mainBinaryName` during read_first. If mismatch, the e2e-linux job hard-fails with "binary not found". Iterate on the YAML/config; do not change `hp41-gui/src-tauri/` source.
- **WebdriverIO TS spec loading:** WDIO 9 supports TS specs via built-in loader, but may require `@types/mocha` or `@wdio/tsconfig`. If the executor finds extra setup needed, fall back to a `.js` spec (15 lines, type-tolerant) and document the deferred TS-spec support in the SUMMARY.
- **`webkit2gtk-driver` apt name mismatch:** Ubuntu 24.04 may package this under a slightly different name (e.g. `libwebkit2gtk-4.1-dev` already bundles the driver as `WebKitWebDriver` on some distros). If `tauri-driver` exits with "WebKitWebDriver not found", check `apt list --installed | grep -i webkit` on the runner and add the correct package name; the RESEARCH-cited `webkit2gtk-driver` is the Debian-standard name.
- **`tauri-driver` cargo install fails on first CI run:** likely a transient crates.io issue; `--locked --version 2.0.6` should prevent semver surprises. If persistent, check `cargo install tauri-driver --list` for the latest available version and pin accordingly.
- **`xvfb-run -a` hangs:** rare; the GitHub Ubuntu runner typically has Xvfb pre-installed and `-a` auto-assigns a display number. If hangs occur, switch to explicit `Xvfb :99 & export DISPLAY=:99` per the RESEARCH §Open Questions.
- **The smoke spec passes locally but fails in CI:** likely a display-mode mismatch — the LCD may show `5.` or `5` in display modes other than FIX(4). Verify the default DisplayMode at app boot (CalcState::new() in state.rs) — if it's FIX(4) by default, the assertion `5.0000` is correct; otherwise widen the assertion to `assert text starts with "5"` or set FIX(4) explicitly via an additional click sequence at the start of the spec.
- **Required-for-merge configuration:** the YAML alone cannot mark a job required. The plan documents the follow-up in the SUMMARY; the user (or a maintainer) configures the branch protection rule manually via GitHub UI or the GraphQL API.
- **Vitest `--coverage` requires `@vitest/coverage-v8`:** if Vitest 4 doesn't bundle the provider, D-27.4 forbids adding it. Record the absence in the SUMMARY; cargo llvm-cov on `hp41-gui/src-tauri/` still happens (Rust path).
- **`npm test` in `gui-ci` triggers a vitest watch loop:** the script in package.json line 11 is `"test": "vitest run"` — `run` means single-shot, not watch. Verified during read_first.
- **The atomicity warning in Task 4 is critical:** push the entire plan's diff together (single PR is fine; pushing the e2e-linux job edit without the WDIO setup will hard-fail CI red).

## Out of Scope (explicit)
- Playwright (`@playwright/test`) — replaced by WebdriverIO per D-27.15 AMENDED 2026-05-15
- Broader E2E flows (modal interactions, autosave persistence roundtrip) — deferred to v2.3+ per D-27.13
- `tauri-plugin-playwright` (Option C) — rejected per RESEARCH §Standard Stack
- GUI coverage CI gate — D-27.4 explicit "measure only"
- macOS / Windows E2E runners — ROADMAP cross-cutting (Ubuntu only)
- Coverage push, accuracy extension → Plan 27-01
- Proptest suites → Plan 27-02
- IND integration suite → Plan 27-03
- `hp41-core/src/` edits (frozen)
- `hp41-gui/src-tauri/` edits (SC-4)
- Web Audio API for BEEP/TONE (v3.x territory)

## References
- 27-CONTEXT.md D-27.4 (GUI coverage measure-only), D-27.13 (literal ROADMAP smoke), D-27.14 (Vitest CI gating), D-27.15 AMENDED (WebdriverIO replaces Playwright — 2026-05-15), D-27.16 (1 retry)
- 27-RESEARCH.md §Pattern 5 (WebdriverIO config template), §Example 6 (ci-gui.yml e2e job), §Pitfalls 5/6/9/10
- 27-RESEARCH.md §Open Question 1 (the Playwright/tauri-driver protocol mismatch — closed by the 2026-05-15 amendment)
- 27-RESEARCH.md §Assumption A5 (xvfb defensive wrapping)
- 27-RESEARCH.md §Assumption A6 (E2E job runtime ~3–8 min)
- ROADMAP.md SC-5 line 200 (the literal smoke success criterion: `2 ENTER 3 + → 5.0000`)
- ROADMAP.md cross-cutting line 205 (Playwright runs ONLY on Ubuntu — superseded by D-27.15 AMENDED, but Ubuntu-only constraint unchanged)
- CLAUDE.md "v2.0 additions" + "v2.1 additions" sections (Tauri 2.11 patterns, data-key-id precedent)
- Tauri v2 official docs: https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/
</failure_modes>
