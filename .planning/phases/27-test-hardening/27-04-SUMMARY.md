---
phase: 27-test-hardening
plan: 04
subsystem: gui-e2e + ci
tags:
  - e2e
  - webdriverio
  - tauri-driver
  - vitest-ci-gating
  - ubuntu-only
  - fn-qual-05
dependency_graph:
  requires:
    - hp41-gui/src/Keyboard.tsx (data-key-id attributes — Phase 26 Plan 04 Task 3)
    - hp41-gui/src-tauri/Cargo.toml ([[bin]] name = "hp41-gui")
    - hp41-gui/src-tauri/tauri.conf.json (productName)
    - justfile (existing gui-ci recipe; gui-build recipe)
    - .github/workflows/ci-gui.yml (existing build job)
  provides:
    - hp41-gui/wdio.conf.js (WebdriverIO config; mochaOpts.retries=1)
    - hp41-gui/e2e/smoke.spec.ts (FN-QUAL-05 literal ROADMAP smoke)
    - hp41-gui/src/Display14Seg.tsx (data-testid + data-text on outer SVG)
    - justfile gui-e2e recipe (new)
    - justfile gui-ci recipe (+ npm test, D-27.14)
    - .github/workflows/ci-gui.yml e2e-linux job (Ubuntu only)
    - CLAUDE.md v2.2 additions (Phase 27 E2E block + Key Files rows)
  affects:
    - hp41-gui/package.json devDependencies (+5 WDIO packages, ^9.19)
    - hp41-gui/package-lock.json (resolved 9.27.1; +453 packages)
    - hp41-gui/vite.config.ts (test.exclude adds e2e/**)
tech-stack:
  added:
    - webdriverio ^9.19 (resolved 9.27.1)
    - "@wdio/cli ^9.19 (resolved 9.27.1)"
    - "@wdio/local-runner ^9.19"
    - "@wdio/mocha-framework ^9.19"
    - "@wdio/spec-reporter ^9.19"
  removed_or_not_added:
    - "@playwright/test (per D-27.15 AMENDED 2026-05-15)"
    - "@vitest/coverage-v8 (per D-27.4 measure-only no-devDep clause)"
  patterns:
    - WebdriverIO + tauri-driver E2E via WebDriver classic protocol
    - Mocha framework with bdd UI + retries: 1
    - Cargo bin cache for tauri-driver 2.0.6 (Pitfall 5)
    - xvfb-run wrapper on headless Ubuntu runner (Assumption A5)
    - data-testid + data-text dual locator pattern for SVG-rendered LCD
key-files:
  created:
    - hp41-gui/wdio.conf.js
    - hp41-gui/e2e/smoke.spec.ts
    - .planning/phases/27-test-hardening/27-04-SUMMARY.md
  modified:
    - hp41-gui/src/Display14Seg.tsx
    - hp41-gui/package.json
    - hp41-gui/package-lock.json
    - hp41-gui/vite.config.ts
    - justfile
    - .github/workflows/ci-gui.yml
    - CLAUDE.md
decisions:
  - D-27.13 honored — single literal ROADMAP smoke (`2 ENTER 3 + → 5.0000`); no broader flows
  - D-27.14 honored — Vitest gated via `cd hp41-gui && npm test` appended to gui-ci
  - D-27.15 AMENDED honored — WebdriverIO + tauri-driver, NOT Playwright (audit trail in CLAUDE.md + commit messages)
  - D-27.16 honored — `mochaOpts.retries: 1` in wdio.conf.js (NOT playwright.config.ts)
  - D-27.4 honored — GUI coverage measured one-shot (77.92 % lines on hp41-gui/src-tauri/); NO CI gate; NO devDep added
  - SC-4 invariant preserved — no `hp41-gui/src-tauri/` source changes
  - MSRV 1.88 unchanged — `tauri-driver` 2.0.6 MSRV 1.77 is compatible
metrics:
  duration_min: 8
  completed: 2026-05-15
---

# Phase 27 Plan 04: E2E (WebdriverIO + tauri-driver) + Vitest CI gating Summary

**One-liner:** WebdriverIO + `tauri-driver` E2E smoke gates `2 ENTER 3 + → 5.0000` against the production Tauri release binary on Ubuntu CI; existing 5 Vitest files (142 tests) gate on every CI push.

**Plan:** 27-04
**Phase:** 27 (Test Hardening)
**Wave:** 2
**Requirement satisfied:** FN-QUAL-05 (and the D-27.14 CI hygiene hole)

---

## Goal

Land the FN-QUAL-05 ROADMAP success criterion `2 ENTER 3 + → 5.0000` as a real CI gate that drives the production Tauri release binary end-to-end via WebDriver classic, AND close the D-27.14 quiet hole where the 5 existing Vitest files passed locally but weren't run on CI.

## What shipped

### Six atomic commits (Task → commit)

| Task | Commit  | Subject                                                       | Files modified                                                              |
| ---- | ------- | ------------------------------------------------------------- | --------------------------------------------------------------------------- |
| 1    | 6bfa1c3 | feat(27-04): add data-testid/data-text on Display14Seg LCD SVG | hp41-gui/src/Display14Seg.tsx                                               |
| 2    | 6d10a59 | test(27-04): add WebdriverIO + tauri-driver E2E smoke (FN-QUAL-05) | hp41-gui/package.json, hp41-gui/package-lock.json, hp41-gui/vite.config.ts, hp41-gui/wdio.conf.js (NEW), hp41-gui/e2e/smoke.spec.ts (NEW) |
| 3    | d6429b0 | chore(27-04): gui-e2e recipe + Vitest gating in gui-ci         | Justfile                                                                    |
| 4    | a92568d | ci(27-04): add e2e-linux job for WebdriverIO + tauri-driver smoke | .github/workflows/ci-gui.yml                                                |
| 5    | (no commit — measurement only per D-27.4) | GUI coverage one-shot measurement              | (no files)                                                                  |
| 6    | bbe8b7e | docs(27-04): CLAUDE.md v2.2 additions for WebdriverIO E2E + Vitest gating | CLAUDE.md                                                                   |

### File inventory

| File                                  | Status  | Purpose                                                                                                                                  |
| ------------------------------------- | ------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `hp41-gui/wdio.conf.js`               | NEW (64 lines) | WebdriverIO config: framework=mocha, mochaOpts.retries=1 (D-27.16), tauri:options.application='../src-tauri/target/release/hp41-gui', beforeSession spawns tauri-driver, afterSession kills it |
| `hp41-gui/e2e/smoke.spec.ts`          | NEW (59 lines) | Literal ROADMAP smoke (D-27.13): clicks `[data-key-id="2"|"enter"|"3"|"plus"]` and asserts `[data-testid="lcd-display"]` data-text='5.0000' (falls back to getText if data-text absent) |
| `hp41-gui/src/Display14Seg.tsx`       | EDIT (+7 lines) | Adds `data-testid="lcd-display"` + `data-text={text}` on outer `<svg>`. Allowed under SC-4 (hp41-gui/src/ is outside the SC-4 boundary). |
| `hp41-gui/package.json`               | EDIT    | +5 WDIO devDeps at ^9.19 (resolved to 9.27.1 in lockfile): `@wdio/cli`, `@wdio/local-runner`, `@wdio/mocha-framework`, `@wdio/spec-reporter`, `webdriverio`. NO `@playwright/test`. |
| `hp41-gui/package-lock.json`          | EDIT    | +453 packages resolved from the WDIO dep tree. Lockfile committed for CI reproducibility. |
| `hp41-gui/vite.config.ts`             | EDIT (+5 lines) | `test.exclude: ['**/node_modules/**', '**/dist/**', 'e2e/**']` — keeps Vitest from picking up the WebdriverIO spec. |
| `Justfile`                            | EDIT (+14 lines) | `gui-ci:` appends `cd hp41-gui && npm test` (D-27.14); NEW `gui-e2e:` recipe with documented preconditions. |
| `.github/workflows/ci-gui.yml`        | EDIT (+51 lines) | Appends `webkit2gtk-driver xvfb` to existing matrix-build apt step; NEW `e2e-linux` job (Ubuntu only, `needs: build`, cargo-bin cache for tauri-driver 2.0.6, apt deps incl. webkit2gtk-driver + xvfb, `cargo install tauri-driver --locked --version 2.0.6`, `just gui-build`, `xvfb-run -a just gui-e2e`). |
| `CLAUDE.md`                           | EDIT (+7 lines) | Appends 4 bullets to existing `### v2.2 additions (Test Hardening, Phase 27)` block; adds wdio.conf.js + e2e/smoke.spec.ts rows to the GUI Key Files table. |

### Selector inventory in smoke.spec.ts

- `[data-key-id="2"]` — Keyboard.tsx line 285/303 (Phase 26 Plan 04 Task 3)
- `[data-key-id="enter"]` — Keyboard.tsx KEY_DEFS line 97
- `[data-key-id="3"]` — Keyboard.tsx KEY_DEFS line 120
- `[data-key-id="plus"]` — Keyboard.tsx KEY_DEFS line 111
- `[data-testid="lcd-display"]` — Display14Seg.tsx outer `<svg>` (added in Task 1)

### Justfile diff

**gui-ci** appended one line:
```make
cd hp41-gui && npm test
```

**gui-e2e** new recipe:
```make
gui-e2e:
	cd hp41-gui && npm install
	cd hp41-gui && npx wdio run wdio.conf.js
```

### ci-gui.yml diff

- Matrix build job: existing apt line gained `webkit2gtk-driver xvfb` (kept on the same single line for minimal diff).
- New `e2e-linux` job: 11 steps — checkout → Rust toolchain → Swatinem cache → cargo-bin cache for tauri-driver → setup-node → install just → apt deps → install tauri-driver → `just gui-build` → `xvfb-run -a just gui-e2e`.

## GUI Coverage Snapshot (D-27.4 — measure only, NOT a gate)

Per D-27.4 these numbers are a one-shot advisory snapshot for v3.x roadmap reference. NO CI gate added. NO devDep added.

### Vitest line coverage on `hp41-gui/src/`

**Skipped per D-27.4 no-devDep clause.** Vitest 4 surfaces `--coverage` but the v8 provider is NOT bundled — running `npx vitest run --coverage` reports `Cannot find dependency '@vitest/coverage-v8'`. D-27.4 explicitly forbids adding `@vitest/coverage-v8` to `hp41-gui/package.json`, so this measurement is deferred. The 5 existing Vitest files cover the 5 source files they're named after (App.tsx, Display14Seg.tsx, HelpOverlay.tsx, Keyboard.tsx, pending_input.ts) plus dependencies they import; a v3.x snapshot can quantify exact %.

### cargo llvm-cov on `hp41-gui/src-tauri/`

Measured 2026-05-15 with `cargo llvm-cov --manifest-path hp41-gui/src-tauri/Cargo.toml --summary-only` (preceded by `cargo llvm-cov clean --workspace`):

| File             | Lines covered  | Lines % | Regions covered | Regions % | Functions covered | Functions % |
| ---------------- | -------------- | ------- | --------------- | --------- | ----------------- | ----------- |
| `cards.rs`       | 129/170        | 75.88   | 219/313         | 69.97     | 14/26             | 53.85       |
| `commands.rs`    | 219/282        | 77.66   | 442/556         | 79.50     | 22/34             | 64.71       |
| `key_map.rs`     | 548/615        | 89.11   | 983/1086        | 90.52     | 20/23             | 86.96       |
| `lib.rs`         | 0/31           |  0.00   | 0/56            |  0.00     | 0/5               |  0.00       |
| `main.rs`        | 0/3            |  0.00   | 0/3             |  0.00     | 0/1               |  0.00       |
| `persistence.rs` | 98/107         | 91.59   | 213/224         | 95.09     | 12/14             | 85.71       |
| `prgm_display.rs`| 114/252        | 45.24   | 211/463         | 45.57     | 8/10              | 80.00       |
| `types.rs`       | 145/148        | 97.97   | 275/281         | 97.86     | 15/15             | 100.00      |
| **TOTAL**        | **1253/1608**  | **77.92** | **2343/2982** | **78.57** | **91/128**       | **71.09**   |

Observations:
- `main.rs` and `lib.rs` at 0 % are Tauri boilerplate — `setup()`, `generate_handler!` registration, auto-save thread spawning. These run only when the binary boots, which `cargo test` doesn't do. The E2E job exercises them at runtime in CI on every push (orthogonal coverage signal).
- `key_map.rs` at 89.11 % lines reflects the 270+ key-id resolution branches; the 11 % uncovered are likely the stub-error branches (`pi`, `polar_to_rect`, `beep`, etc.) plus parameterized-prompt fallthroughs.
- `prgm_display.rs` at 45.24 % is the lowest non-boilerplate file — many `Op` variants don't have dedicated formatter coverage. Candidate for v3.x test addition if the function matrix grows.
- `types.rs` at 97.97 % validates the CalcStateView serialization contract is well-covered.

No CI gate added. No devDep added. v3.x candidate: add a gate at the achieved level after a soak period.

## Verification

### Local verifications run during this plan

- `cd hp41-gui && npx tsc --noEmit` — clean
- `cd hp41-gui && npm test` — 5/5 files, 142/142 tests pass
- `cd hp41-gui && grep -c "@wdio/" package.json` — 4 (the four @wdio packages)
- `cd hp41-gui && grep -c "webdriverio" package.json` — 1
- `cd hp41-gui && grep -c "@playwright/test" package.json` — 0
- `grep -c 'data-testid="lcd-display"' hp41-gui/src/Display14Seg.tsx` — 1
- `grep -q "retries: 1" hp41-gui/wdio.conf.js` — exit 0
- `test -f hp41-gui/wdio.conf.js && test -f hp41-gui/e2e/smoke.spec.ts` — exit 0
- `just --list | grep -E "gui-ci|gui-e2e"` — both recipes shown
- `grep -A 6 "^gui-ci:" Justfile | grep -c "npm test"` — 1
- `grep -c "^gui-e2e:" Justfile` — 1
- `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci-gui.yml'))"` — clean
- `grep -c "e2e-linux:" .github/workflows/ci-gui.yml` — 1
- `grep -c "needs: build" .github/workflows/ci-gui.yml` — 1
- `grep -c "webkit2gtk-driver" .github/workflows/ci-gui.yml` — 3 (matrix build + e2e-linux apt + e2e-linux step name comment)
- `grep -c "tauri-driver" .github/workflows/ci-gui.yml` — 7 (cache key + install step + comments)
- `grep -c "xvfb-run" .github/workflows/ci-gui.yml` — 1
- `grep -c "just gui-e2e" .github/workflows/ci-gui.yml` — 1
- `grep -c "Test Hardening, Phase 27" CLAUDE.md` — 1
- `grep -c "WebdriverIO" CLAUDE.md` — 4
- `grep -c "D-27.15 AMENDED" CLAUDE.md` — 1
- `grep -c "wdio.conf.js" CLAUDE.md` — 1
- `grep -rn "fn op_\|fn flush_entry\|fn format_hpnum" hp41-gui/src-tauri/src/` — only matches the existing `op_display_name` formatter (SC-4 invariant preserved)
- `cargo llvm-cov --manifest-path hp41-gui/src-tauri/Cargo.toml --summary-only` — 77.92 % lines (snapshot recorded)

### CI-side verification (after push)

The `e2e-linux` job runs on the GitHub Ubuntu-latest runner. Expected sequence:
1. `actions/checkout@v4` checks out the merge commit.
2. `dtolnay/rust-toolchain@stable` installs the stable Rust toolchain.
3. `Swatinem/rust-cache@v2` restores the hp41-gui/src-tauri target cache.
4. `actions/cache@v4` restores `~/.cargo/bin` keyed on `cargo-bin-Linux-tauri-driver-2.0.6` (cold first run; cached subsequent runs).
5. `actions/setup-node@v4` provisions Node LTS.
6. `taiki-e/install-action@v2` installs the `just` binary.
7. `apt-get install` adds `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf webkit2gtk-driver xvfb`.
8. `cargo install tauri-driver --locked --version 2.0.6` (cache miss only on first run; subsequent runs are no-ops).
9. `just gui-build` builds the release Tauri binary at `hp41-gui/src-tauri/target/release/hp41-gui`.
10. `xvfb-run -a just gui-e2e` runs the WebdriverIO spec under a virtual display.

Expected runtime budget per RESEARCH Assumption A6: **~3–5 min cached / 6–8 min cold.**

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Vitest picked up `e2e/smoke.spec.ts`**

- **Found during:** Task 2 verification — `npm test` reported "Test Files 1 failed | 5 passed (6)" with `e2e/smoke.spec.ts (0 test)` listed as failing.
- **Root cause:** Vitest's default test discovery glob includes `**/*.spec.ts` from the project root, which matched the new WebdriverIO spec. The WDIO spec uses Mocha runtime globals (`describe`, `it`, `$`) that Vitest doesn't provide.
- **Fix:** Added `exclude: ['**/node_modules/**', '**/dist/**', 'e2e/**']` to `test:` in `hp41-gui/vite.config.ts`. This preserves the default Vitest excludes (node_modules, dist) and adds the new `e2e/**` glob so the WebdriverIO spec is not discovered by Vitest.
- **Files modified:** `hp41-gui/vite.config.ts`
- **Commit:** included in 6d10a59 (Task 2)
- **Why this is Rule 3:** The blocking issue was directly caused by my Task 2 work (creating the new `.spec.ts` file in a discoverable location). The fix is minimal (one config line) and preserves Vitest's existing behavior on all other paths.

**2. [Rule 3 - Blocking] Case-sensitive filename `Justfile` vs `justfile`**

- **Found during:** Task 3 attempt to commit `justfile` — git refused to commit `justfile` because the tracked path is `Justfile` (capital J). The plan referred to the file as `justfile` throughout.
- **Root cause:** macOS case-insensitive filesystem masks the git-tracked capitalization (`Justfile`). The Read tool worked with `justfile` lowercase because the FS resolved it case-insensitively, but git operations need the exact tracked path.
- **Fix:** Used `git add Justfile` (capital J) for the commit; my Read/Edit operations had already succeeded with the lowercase variant because the FS resolved them.
- **Files modified:** None (just a git-add invocation correction).
- **Why this is Rule 3:** A pure tooling-mechanic blocker. No semantic change to the file.

No other deviations. The plan's Branch B path (CLAUDE.md `### v2.2 additions (Test Hardening, Phase 27)` block already exists from Plan 27-01) applied as documented — I appended bullets to the existing block rather than creating a new one.

## Manual follow-ups (post-merge)

1. **Configure branch protection for `develop` and `main`** to mark the `GUI E2E (Ubuntu only — WebdriverIO + tauri-driver) / e2e-linux` job as a required-status-check. GitHub's YAML alone cannot enforce required-for-merge; this is a repo-settings configuration via the web UI or GraphQL API.
2. **First CI run will exercise the cargo-bin cache cold path** for `tauri-driver`. Expect ~6–8 min runtime; subsequent runs cache to ~3–5 min.
3. **If the apt package `webkit2gtk-driver` is not available on the GitHub Ubuntu 24.04 runner** (some distros ship the driver bundled with `libwebkit2gtk-4.1-dev`), check `apt list --installed | grep -i webkit` and adjust the apt-install step accordingly. Per RESEARCH Failure Modes, `webkit2gtk-driver` is the Debian-standard name and should be available; if not, the e2e-linux step will fail with `WebKitWebDriver not found` and the iteration is on the YAML only (no spec or Display14Seg changes needed).

## D-27.15 AMENDED Audit Trail

The decision history is preserved in three places to make the Playwright → WebdriverIO substitution auditable:

1. **CLAUDE.md line 106** — explicit `D-27.15 AMENDED 2026-05-15` note explaining the protocol mismatch.
2. **27-04-PLAN.md must_haves and out-of-scope** — explicit `NOT Playwright` callouts.
3. **Commit 6d10a59 message body** — explicit `NO @playwright/test (D-27.15 AMENDED supersedes original Playwright wording — tauri-driver 2.0.6 speaks WebDriver classic which Playwright does NOT)`.
4. **27-CONTEXT.md lines 106–109** — the original D-27.15 entry with the AMENDED suffix and the rejected-alternatives rationale.

## Threat Flags

No new threat surface beyond what was documented in PLAN.md `<threat_model>`. The `data-testid` attribute is a React-auto-escaped DOM attribute (no XSS). The `tauri-driver` server is localhost-only on `127.0.0.1:4444`. `cargo install --locked --version 2.0.6` uses Cargo.lock checksum validation.

## Self-Check: PASSED

- `hp41-gui/wdio.conf.js` — FOUND (64 lines)
- `hp41-gui/e2e/smoke.spec.ts` — FOUND (59 lines)
- `hp41-gui/src/Display14Seg.tsx` — `data-testid="lcd-display"` present (line 219)
- `hp41-gui/package.json` — 5 WDIO devDeps present; `@playwright/test` correctly absent
- `Justfile` `gui-ci:` ends with `cd hp41-gui && npm test` — verified
- `Justfile` `gui-e2e:` recipe — verified
- `.github/workflows/ci-gui.yml` `e2e-linux` job — verified (YAML lint clean, both jobs present)
- `CLAUDE.md` `### v2.2 additions (Test Hardening, Phase 27)` block — coherent single instance with E2E + Vitest + data-testid + GUI coverage bullets
- Commit hashes in git log: 6bfa1c3, 6d10a59, d6429b0, a92568d, bbe8b7e — all FOUND
- SC-4 invariant preserved — only the pre-existing `op_display_name` matches the SC-4 grep
- Vitest still passes: 5/5 files, 142/142 tests
- TypeScript still clean: `npx tsc --noEmit` exits 0
