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

# Lint with clippy (warnings treated as errors)
lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run the CLI (placeholder until Phase 4)
run:
	cargo run -p hp41-cli

# Check coverage gate — ≥80% line coverage on hp41-core
coverage:
	cargo llvm-cov --fail-under-lines 80 -p hp41-core

# Full CI gate: lint → test → coverage
ci: lint test coverage

# Check formatting without modifying files (mirrors CI)
fmt-check:
	cargo fmt --all -- --check

# Auto-format all Rust sources
fmt:
	cargo fmt --all

# Run criterion benchmarks for hp41-core (advisory — does not gate CI)
bench:
	cargo bench -p hp41-core

# Install the pre-push git hook (run once after cloning)
install-hooks:
	@printf '#!/usr/bin/env bash\nset -euo pipefail\necho "🔍 pre-push: cargo fmt --check ..."\ncargo fmt --all -- --check || { echo ""; echo "❌ Run: cargo fmt --all"; exit 1; }\necho "🔍 pre-push: just lint ..."\njust lint || { echo ""; echo "❌ Run: just lint"; exit 1; }\necho "✅ pre-push checks passed"\n' > .git/hooks/pre-push
	@chmod +x .git/hooks/pre-push
	@echo "✅ pre-push hook installed"

# Run criterion benchmarks for hp41-core dispatch latency (advisory — does not gate CI)
bench:
	cargo bench -p hp41-core

# Measure cold-start latency with hyperfine (manual pre-release step — not a CI gate)
# Usage: just bench-startup
# Prerequisite: just build-release (or cargo build --release) must be run first
bench-startup:
	hyperfine --runs 10 ./target/release/hp41
