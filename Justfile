# On Windows, force just to use Git Bash. Cygwin's /usr/bin/sh (if present on PATH)
# breaks rustup's cargo-proxy argv[0] detection, causing `cargo <subcmd>` to fall
# through to `rustup` itself. Linux/macOS keep the default sh.
set windows-shell := ["C:/Program Files/Git/bin/bash.exe", "-cu"]

# Default — show available recipes
default:
	@just --list

# Build all workspace crates
build:
	cargo build --workspace

# Build release binary (required before bench-startup)
build-release:
	cargo build --release

# Run all tests
test:
	cargo test --workspace

# Run hp41-core tests with optional filter args (e.g. `just test-core --test phase21_flags`)
test-core *args:
	cargo test -p hp41-core {{args}}

# Lint with clippy (warnings treated as errors)
lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run the CLI (placeholder until Phase 4)
run:
	cargo run -p hp41-cli

# Check coverage gate — ≥95% line coverage on hp41-core (raised from 80 in Phase 27 / FN-QUAL-01, atomic per D-27.2).
# The matching CI job in ci.yml is named "Coverage (>=95%)" — keep in sync if the threshold ever changes.
coverage:
	cargo llvm-cov clean --workspace
	cargo llvm-cov --fail-under-lines 95 -p hp41-core

# Full CI gate: lint → test → coverage
ci: lint test coverage

# CI gate for MSRV jobs: lint → test (NO coverage).
# Coverage is rustc-version-dependent — different rustc versions instrument llvm-cov
# differently and produce slightly different line counts. Phase 27's atomic 80→95 gate
# raise (D-27.2) was calibrated on stable (≈ 95.25 % lines); MSRV 1.88 produces ≈
# 94.42 % on the same source. Gating MSRV on coverage would force an artificial
# test-padding arms race tied to the lowest measurement-tool baseline. The dedicated
# `Coverage (>=80%)` job in `ci.yml` runs on stable and enforces the 95 % gate; the
# MSRV job verifies code still builds and tests still pass on the declared minimum rustc.
ci-msrv: lint test

# Check formatting without modifying files (mirrors CI)
fmt-check:
	cargo fmt --all -- --check

# Auto-format all Rust sources
fmt:
	cargo fmt --all

# Run criterion benchmarks for hp41-core dispatch latency (advisory — does not gate CI)
bench:
	cargo bench -p hp41-core

# Install the pre-push git hook (run once after cloning)
install-hooks:
	@printf '#!/usr/bin/env bash\nset -euo pipefail\necho "🔍 pre-push: cargo fmt --check ..."\ncargo fmt --all -- --check || { echo ""; echo "❌ Run: cargo fmt --all"; exit 1; }\necho "🔍 pre-push: just lint ..."\njust lint || { echo ""; echo "❌ Run: just lint"; exit 1; }\necho "✅ pre-push checks passed"\n' > .git/hooks/pre-push
	@chmod +x .git/hooks/pre-push
	@echo "✅ pre-push hook installed"

# Measure cold-start latency with hyperfine (manual pre-release step — not a CI gate)
# Usage: just bench-startup
# Prerequisite: just build-release (or cargo build --release) must be run first
bench-startup:
	hyperfine --warmup 3 --runs 10 './target/release/hp41 --bench-startup'

# GUI: install npm dependencies (run once after cloning or after package.json changes)
gui-install:
	cd hp41-gui && npm install

# GUI: launch development window (Rust hot-reload + Vite HMR)
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
gui-build:
	cd hp41-gui && npm ci
	cd hp41-gui && npm run tauri build

# GUI: Rust type-check (fast CI check without launching dev server)
gui-check:
	cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml

# `npm ci` (lockfile-strict) catches drift between package.json and the lockfile.
# `npm audit --omit=dev --audit-level=high || true` is a non-blocking warning
# surface for new high-severity advisories in production deps; CI does NOT fail
# on this (advisory drift would block merges on infra-side events outside our
# control). For developer follow-up, run `cd hp41-gui && npm audit fix` manually.
#
# gui-ci: CI gate — TS type-check, Rust tests, release build, Vitest (D-27.14)
gui-ci:
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
gui-e2e:
	test -x hp41-gui/src-tauri/target/release/hp41-gui \
	  || (echo "ERROR: production binary missing. Run 'just gui-build' first." >&2 && exit 1)
	cd hp41-gui && npm ci
	cd hp41-gui && npx wdio run wdio.conf.cjs

# Regenerate the HP-41CV function matrix from canonical JSON (developer-side).
# Reads docs/hp41cv-functions.json and writes docs/hp41cv-function-matrix.md.
docs-matrix:
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41cv-functions.json docs/hp41cv-function-matrix.md

# CI-friendly drift catch (Pitfall 8): regenerate to a temp file and diff
# against the committed copy. Exits non-zero on mismatch so CI fails fast.
docs-matrix-check:
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41cv-functions.json /tmp/hp41cv-function-matrix-check.md
	diff -u docs/hp41cv-function-matrix.md /tmp/hp41cv-function-matrix-check.md
