# Default — show available recipes
default:
	@just --list

# Build all workspace crates
build:
	cargo build --workspace

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
