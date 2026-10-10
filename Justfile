# On Windows, force just to use Git Bash. Cygwin's /usr/bin/sh (if present on PATH)
# breaks rustup's cargo-proxy argv[0] detection, causing `cargo <subcmd>` to fall
# through to `rustup` itself. Linux/macOS keep the default sh.
set windows-shell := ["C:/Program Files/Git/bin/bash.exe", "-cu"]

# Default — show available recipes
default:
	@just --list

# ─── Build & Run ────────────────────────────────────────────────────────────

# Build all workspace crates
[group('build')]
build:
	cargo build --workspace

# Build release binary (required before bench-startup)
[group('build')]
build-release:
	cargo build --release

# Run the CLI (placeholder until Phase 4)
[group('build')]
run:
	cargo run -p hp41-cli

# hp41-gui is a standalone nested workspace with its OWN target/, so a single
# `cargo clean` misses hp41-gui/src-tauri/target/ — including the bundled .app.
# A stale GUI bundle there shares the `ch.talent-factory.hp41` bundle id and can
# shadow an installed release in LaunchServices (showing the old version).
# Remove Rust build artifacts from BOTH workspaces (root + nested GUI)
[group('build')]
clean:
	cargo clean
	cargo clean --manifest-path hp41-gui/src-tauri/Cargo.toml

# ─── Test ───────────────────────────────────────────────────────────────────

# Run all tests
[group('test')]
test:
	cargo test --workspace

# Run hp41-core tests with optional filter args (e.g. `just test-core --test phase21_flags`)
[group('test')]
test-core *args:
	cargo test -p hp41-core {{args}}

# ─── Quality (lint & format) ────────────────────────────────────────────────

# Lint with clippy (warnings treated as errors)
[group('quality')]
lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Check formatting without modifying files (mirrors CI)
[group('quality')]
fmt-check:
	cargo fmt --all -- --check

# Auto-format all Rust sources
[group('quality')]
fmt:
	cargo fmt --all

# ─── CI & Coverage ──────────────────────────────────────────────────────────

# Check coverage gate — ≥95% line coverage on hp41-core (raised from 80 in Phase 27 / FN-QUAL-01, atomic per D-27.2).
# The matching CI job in ci.yml is named "Coverage (>=95%)" — keep in sync if the threshold ever changes.
[group('ci')]
coverage:
	cargo llvm-cov clean --workspace
	cargo llvm-cov --fail-under-lines 95 -p hp41-core

# Phase 32 Plan 32-03 (D-32.7 + D-32.8 + Pitfall 19). Greps hp41-core/src/ops/math1/
# for distinctive Free42 identifiers (Intel BID, decNumber, Free42 internals + GPL/AGPL
# copyright markers) with allowlist for the legitimate per-file disclaim header.
# The matching CI job in ci.yml is named "License audit (Free42 contamination)" — keep
# script invocation in sync if the path ever changes.
[group('ci')]
license-audit:
	bash scripts/check-free42-contamination.sh

# Phase 61 Plan 61-04 (HSQUAL-03). Validates the `search_aliases` field across
# all SIX help-data JSON pools: where present it must be an array of strings, and
# every status:"implemented" entry must carry >=1 alias. The matching CI job in
# ci.yml is named "Schema gate (search_aliases)" — keep script invocation in sync
# if the path ever changes. Lives in ci.yml (NOT ci-gui.yml) because ci-gui.yml's
# paths: filter excludes docs/*.json, so an alias-schema gate there would never fire.
[group('ci')]
schema-aliases-check:
	bash scripts/check-aliases-schema.sh

# Full CI gate: lint → test → coverage → license-audit (Phase 32 D-32.8 belt+suspenders)
[group('ci')]
ci: lint test coverage license-audit

# CI gate for MSRV jobs: lint → test (NO coverage).
# Coverage is rustc-version-dependent — different rustc versions instrument llvm-cov
# differently and produce slightly different line counts. Phase 27's atomic 80→95 gate
# raise (D-27.2) was calibrated on stable (≈ 95.25 % lines); MSRV 1.88 produces ≈
# 94.42 % on the same source. Gating MSRV on coverage would force an artificial
# test-padding arms race tied to the lowest measurement-tool baseline. The dedicated
# `Coverage (>=80%)` job in `ci.yml` runs on stable and enforces the 95 % gate; the
# MSRV job verifies code still builds and tests still pass on the declared minimum rustc.
[group('ci')]
ci-msrv: lint test

# ─── Benchmarks ─────────────────────────────────────────────────────────────

# Run criterion benchmarks for hp41-core dispatch latency (advisory — does not gate CI)
[group('bench')]
bench:
	cargo bench -p hp41-core

# Measure cold-start latency with hyperfine (manual pre-release step — not a CI gate)
# Usage: just bench-startup
# Prerequisite: just build-release (or cargo build --release) must be run first
[group('bench')]
bench-startup:
	hyperfine --warmup 3 --runs 10 './target/release/hp41 --bench-startup'

# ─── Setup ──────────────────────────────────────────────────────────────────

# Install the pre-push git hook (run once after cloning)
[group('setup')]
install-hooks:
	@printf '#!/usr/bin/env bash\nset -euo pipefail\necho "🔍 pre-push: cargo fmt --check ..."\ncargo fmt --all -- --check || { echo ""; echo "❌ Run: cargo fmt --all"; exit 1; }\necho "🔍 pre-push: just lint ..."\njust lint || { echo ""; echo "❌ Run: just lint"; exit 1; }\necho "✅ pre-push checks passed"\n' > .git/hooks/pre-push
	@chmod +x .git/hooks/pre-push
	@echo "✅ pre-push hook installed"

# ─── GUI (Tauri v2) ─────────────────────────────────────────────────────────

# GUI: install npm dependencies (run once after cloning or after package.json changes)
[group('gui')]
gui-install:
	cd hp41-gui && npm install

# GUI: launch development window (Rust hot-reload + Vite HMR)
[group('gui')]
gui-dev:
	cd hp41-gui && npm run tauri dev

# Self-sufficient: installs npm deps first so the Tauri CLI from `@tauri-apps/cli`
# is on PATH (required by `npm run tauri build`). CI e2e-linux job runs on a fresh
# runner with no prior `npm install`; making this recipe self-installing matches
# the gui-ci / gui-e2e pattern. Uses `npm ci` (lockfile-strict) instead of
# `npm install` so a stale `package.json` doesn't quietly upgrade transitive deps
# in CI — the 9000-line `package-lock.json` is the authoritative dep set.
#
# GUI: production bundle (native app) — installs npm deps then builds via Tauri CLI.
[group('gui')]
gui-build:
	cd hp41-gui && npm ci
	cd hp41-gui && npm run tauri build

# GUI: Rust type-check (fast CI check without launching dev server)
[group('gui')]
gui-check:
	cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml

# `npm ci` (lockfile-strict) catches drift between package.json and the lockfile.
# `npm audit --omit=dev --audit-level=high || true` is a non-blocking warning
# surface for new high-severity advisories in production deps; CI does NOT fail
# on this (advisory drift would block merges on infra-side events outside our
# control). For developer follow-up, run `cd hp41-gui && npm audit fix` manually.
#
# gui-ci: CI gate — TS type-check, Rust tests, release build, Vitest (D-27.14)
# Phase 31 Plan 31-02: permission-coverage gate fires FIRST so a missing TOML
# fails fast before any expensive build step (T-31-W1-permission-coverage).
[group('gui')]
gui-ci:
	bash scripts/check-tauri-permissions.sh
	cd hp41-gui && npm ci
	cd hp41-gui && npm audit --omit=dev --audit-level=high || true
	cd hp41-gui && npx tsc --noEmit
	cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml
	cargo build --release --manifest-path hp41-gui/src-tauri/Cargo.toml
	cd hp41-gui && npm test

# Phase 27 Plan 27-04, FN-QUAL-05, D-27.15 AMENDED 2026-05-15.
# Preconditions:
#   1. `cargo install tauri-driver --locked --version 2.0.6` is on PATH
#      (typically ~/.cargo/bin/tauri-driver)
#   2. `webkit2gtk-driver` apt package is installed (Pitfall 6)
#   3. When running on a headless Ubuntu runner, wrap with `xvfb-run -a` (A5)
#
# Hard precondition check: the production binary must exist before launch.
# Without this guard a developer running `just gui-e2e` locally without first
# running `just gui-build` sees a confusing "Failed to execute child process"
# from tauri-driver. The check surfaces the missing step at recipe entry.
#
# gui-e2e: WebdriverIO + tauri-driver E2E smoke (Linux only — from ci-gui.yml)
[group('gui')]
gui-e2e:
	test -x hp41-gui/src-tauri/target/release/hp41-gui \
	  || (echo "ERROR: production binary missing. Run 'just gui-build' first." >&2 && exit 1)
	cd hp41-gui && npm ci
	cd hp41-gui && npx wdio run wdio.conf.cjs

# ─── Docs ───────────────────────────────────────────────────────────────────

# Regenerate all six function matrices from their canonical JSON sources (developer-side).
# Reads docs/hp41cv-functions.json -> docs/hp41cv-function-matrix.md (unchanged, D-30.2).
# Reads docs/hp41-math1-functions.json -> docs/hp41-math1-function-matrix.md (unchanged, D-30.1).
# Reads docs/hp41-stat1-functions.json -> docs/hp41-stat1-function-matrix.md (D-35.1).
# Reads docs/hp41-time-functions.json -> docs/hp41-time-function-matrix.md (D-40.1).
# Reads docs/hp41-advantage-functions.json -> docs/hp41-advantage-function-matrix.md (D-44.1).
# Reads docs/hp41-xmem-functions.json -> docs/hp41-xmem-function-matrix.md (D-52.2).
[group('docs')]
docs-matrix:
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41cv-functions.json docs/hp41cv-function-matrix.md
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41-math1-functions.json docs/hp41-math1-function-matrix.md
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41-stat1-functions.json docs/hp41-stat1-function-matrix.md
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41-time-functions.json docs/hp41-time-function-matrix.md
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41-advantage-functions.json docs/hp41-advantage-function-matrix.md
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41-xmem-functions.json docs/hp41-xmem-function-matrix.md

# CI-friendly drift catch (Pitfall 8): regenerate to temp files and diff all
# six against their committed copies. Exits non-zero on mismatch so CI fails fast.
[group('docs')]
docs-matrix-check:
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41cv-functions.json /tmp/hp41cv-function-matrix-check.md
	diff -u docs/hp41cv-function-matrix.md /tmp/hp41cv-function-matrix-check.md
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41-math1-functions.json /tmp/hp41-math1-function-matrix-check.md
	diff -u docs/hp41-math1-function-matrix.md /tmp/hp41-math1-function-matrix-check.md
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41-stat1-functions.json /tmp/hp41-stat1-function-matrix-check.md
	diff -u docs/hp41-stat1-function-matrix.md /tmp/hp41-stat1-function-matrix-check.md
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41-time-functions.json /tmp/hp41-time-function-matrix-check.md
	diff -u docs/hp41-time-function-matrix.md /tmp/hp41-time-function-matrix-check.md
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41-advantage-functions.json /tmp/hp41-advantage-function-matrix-check.md
	diff -u docs/hp41-advantage-function-matrix.md /tmp/hp41-advantage-function-matrix-check.md
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41-xmem-functions.json /tmp/hp41-xmem-function-matrix-check.md
	diff -u docs/hp41-xmem-function-matrix.md /tmp/hp41-xmem-function-matrix-check.md

# Generate DE+EN search aliases for all six help JSON pools.
# Re-run only when functions are added or their semantics change (not on every build).
# LLM (claude) runs on the developer's machine only — NOT in CI and NOT in shipped binaries.
# Fill-only: entries that already have search_aliases are left byte-for-byte untouched (D-60.2).
# In-place: reads and writes the same pool file (single path argument per pool).
[group('docs')]
help-aliases:
	cargo run --quiet --manifest-path scripts/help-aliases/Cargo.toml -- \
		docs/hp41cv-functions.json
	cargo run --quiet --manifest-path scripts/help-aliases/Cargo.toml -- \
		docs/hp41-math1-functions.json
	cargo run --quiet --manifest-path scripts/help-aliases/Cargo.toml -- \
		docs/hp41-stat1-functions.json
	cargo run --quiet --manifest-path scripts/help-aliases/Cargo.toml -- \
		docs/hp41-time-functions.json
	cargo run --quiet --manifest-path scripts/help-aliases/Cargo.toml -- \
		docs/hp41-advantage-functions.json
	cargo run --quiet --manifest-path scripts/help-aliases/Cargo.toml -- \
		docs/hp41-xmem-functions.json

# ─── iOS (Tauri v2 Mobile) ──────────────────────────────────────────────────
#
# iOS target triples (P-iOS-07 — keep device and simulator distinct; using the
# device triple for the simulator yields the silent "building for iOS Simulator,
# but linking in object file built for iOS" linker error):
#   aarch64-apple-ios       physical iPhone (Tauri CLI short name: aarch64, the
#                           `tauri ios build` default — baked in via that default)
#   aarch64-apple-ios-sim   Apple-Silicon Simulator (Tauri short name: aarch64-sim)
#
# Every iOS recipe `cd hp41-gui` first (P-iOS-03 — NEVER invoke the Tauri CLI from
# the repo root; the nested-standalone-workspace bundler #5865 breaks otherwise)
# and drives the CLI through the @tauri-apps/cli devDependency via `npm run tauri`
# (so the pinned CLI from package-lock.json is on PATH — same idiom as gui-dev).
#
# P-iOS-09: the generated Xcode build phase shells out to `cargo`, but Xcode does
# NOT inherit the interactive shell PATH. If a build phase reports
# "cargo: command not found", add `~/.cargo/bin` to PATH via
# hp41-gui/src-tauri/gen/apple/.xcode.env.local.

# iOS: generate the Xcode project at src-tauri/gen/apple/ (idempotent; cargo-mobile2)
[group('ios')]
ios-init:
	cd hp41-gui && npm run tauri ios init

# iOS: release build + IPA for the physical-device triple aarch64-apple-ios
# (the `tauri ios build` default target `aarch64` — D-53.9/P-iOS-07)
[group('ios')]
ios-build:
	cd hp41-gui && npm run tauri ios build

# iOS: signed App Store IPA via MANUAL signing.
# Xcode 26's -allowProvisioningUpdates (Tauri's built-in export path) 401s against ASC, so we let
# Tauri ARCHIVE the app and then export the archive ourselves with an explicit cert + profile
# (.github/ios-export-options.plist). See the reference_ios_signing_testflight memory.
# Prereqs (CI installs them; see ci-ios.yml): the "Apple Distribution: Talent Factory AG" cert in the
# keychain and the "HP-41 App Store (ci-ios)" profile installed. Env IOS_BUILD_NUMBER -> CFBundleVersion.
[group('ios')]
ios-build-release:
	cd hp41-gui && npm ci
	# Remove any stale archive so a failed build can never be exported by mistake.
	rm -rf "hp41-gui/src-tauri/gen/apple/build/hp41-gui_iOS.xcarchive"
	# Tauri archives; its own export step fails under manual signing — ignore its exit code, the archive check below is authoritative.
	cd hp41-gui && npm run tauri ios build -- --export-method app-store-connect || true
	test -d "hp41-gui/src-tauri/gen/apple/build/hp41-gui_iOS.xcarchive" || { echo "ERROR: archive missing — the iOS build failed (see log above)"; exit 1; }
	# Tauri overwrites the bundle version from the crate version; stamp a unique, increasing build number.
	/usr/libexec/PlistBuddy -c "Set :CFBundleVersion {{ env_var_or_default('IOS_BUILD_NUMBER', '1') }}" "hp41-gui/src-tauri/gen/apple/build/hp41-gui_iOS.xcarchive/Products/Applications/HP-41 Calculator.app/Info.plist"
	rm -rf "hp41-gui/src-tauri/gen/apple/build/export"
	xcodebuild -exportArchive -archivePath "hp41-gui/src-tauri/gen/apple/build/hp41-gui_iOS.xcarchive" -exportOptionsPlist ".github/ios-export-options.plist" -exportPath "hp41-gui/src-tauri/gen/apple/build/export"
	# Tauri rewrites the tracked Info.plist (CFBundleVersion) on every build; restore it so the tree stays clean.
	git checkout -- hp41-gui/src-tauri/gen/apple/hp41-gui_iOS/Info.plist

# iOS: boot in the Simulator on the simulator triple aarch64-apple-ios-sim.
# P-iOS-01: list the CURRENT available devices instead of hardcoding a name (a
# hardcoded "iPhone 13" breaks after an Xcode upgrade). Pass an explicit device
# with `just ios-sim device="iPhone 17 Pro"`, or leave it empty to let Tauri pick.
[group('ios')]
ios-sim device="":
	xcrun simctl list devices available | grep -iE 'iPhone|iPad'
	cd hp41-gui && npm run tauri ios dev {{device}}

# iOS: hot-reload dev loop on a connected physical iPhone. P-iOS-02 — connect +
# trust the device in Xcode → Devices and Simulators (network) FIRST; add
# `-- --force-ip-prompt` if the LAN handshake fails. Usage:
# `just ios-dev device="Daniel's iPhone"`.
[group('ios')]
ios-dev device="":
	cd hp41-gui && npm run tauri ios dev {{device}}
