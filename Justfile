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

# Check coverage gate — ≥80% line coverage on hp41-core (clean first to avoid stale profraw data)
coverage:
	cargo llvm-cov clean --workspace
	cargo llvm-cov --fail-under-lines 80 -p hp41-core

# Full CI gate: lint → test → coverage
ci: lint test coverage

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

# GUI: production bundle (native app)
gui-build:
	cd hp41-gui && npm run tauri build

# GUI: Rust type-check (fast CI check without launching dev server)
gui-check:
	cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml

# gui-ci: CI gate — TypeScript type-check, Rust tests, and release build (called from ci-gui.yml)
gui-ci:
	cd hp41-gui && npm install
	cd hp41-gui && npx tsc --noEmit
	cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml
	cargo build --release --manifest-path hp41-gui/src-tauri/Cargo.toml

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
