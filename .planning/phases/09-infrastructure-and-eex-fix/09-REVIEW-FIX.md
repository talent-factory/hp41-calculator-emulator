---
phase: 09-infrastructure-and-eex-fix
fixed_at: 2026-05-08T00:00:00Z
review_path: .planning/phases/09-infrastructure-and-eex-fix/09-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 9: Code Review Fix Report

**Fixed at:** 2026-05-08
**Source review:** .planning/phases/09-infrastructure-and-eex-fix/09-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 5
- Fixed: 5
- Skipped: 0

## Fixed Issues

### CR-01: MSRV CI job will always fail — missing `cargo-llvm-cov` and `clippy` component

**Files modified:** `.github/workflows/ci.yml`
**Commit:** f160b52
**Applied fix:** Added `components: clippy,llvm-tools-preview` to the `dtolnay/rust-toolchain@1.85`
step and extended the `taiki-e/install-action` tool list from `just` to `just,cargo-llvm-cov`.
This mirrors the exact setup of the working `coverage` job (lines 48-61).

---

### CR-02: Modal interruption — 'S', 'R', and Ctrl+A silently discard any active `pending_input` modal

**Files modified:** `hp41-cli/src/app.rs`
**Commit:** e39ddb1
**Applied fix:** Moved the `if self.pending_input.is_some()` routing block to appear BEFORE
the modal-opening interceptors for 'S', 'R', and Ctrl+A in `handle_key()`. An active modal now
takes priority — pressing 'S' while a `ConfirmLoad` dialog is active forwards the key to
`handle_pending_input()` instead of silently replacing the modal. Updated the surrounding comment
to explain the ordering constraint.

---

### WR-01: Member crates do not inherit `rust-version` from workspace — MSRV enforcement is incomplete

**Files modified:** `hp41-core/Cargo.toml`, `hp41-cli/Cargo.toml`
**Commit:** 6b8d074
**Applied fix:** Added `rust-version.workspace = true` to the `[package]` section of both
`hp41-core/Cargo.toml` and `hp41-cli/Cargo.toml`. Verified via `cargo metadata --no-deps` that
both packages now report `rust_version: "1.85"` (previously `null`).

---

### WR-02: `entry_buf` is cleared before parse — silent data loss on malformed input

**Files modified:** `hp41-core/src/ops/mod.rs`, `hp41-core/tests/entry_buf_tests.rs`
**Commit:** a4bb6e1
**Applied fix:** Moved `state.entry_buf.clear()` from before the parse attempt to immediately
after the successful `Decimal::from_str`/`from_scientific` result. The clone of `entry_buf`
used for parsing is unchanged — on parse failure, `Err` is returned and the original
`entry_buf` is preserved. Also updated `test_entry_buf_invalid_content_returns_error` in
`hp41-core/tests/entry_buf_tests.rs` to assert the new correct behavior (entry_buf preserved
on error) rather than the old incorrect behavior (cleared on error). Full test suite: 461 passed.

---

### IN-01: `format_entry_buf_display` handles `'+'` exponent sign that can never appear in `entry_buf`

**Files modified:** `hp41-cli/src/ui.rs`
**Commit:** ea74cbf
**Applied fix:** Removed the `else if let Some(rest) = after_e.strip_prefix('+')` branch from
`format_entry_buf_display`. Replaced the comment on the preceding branch to document why `'+'`
is not expected: "Only '-' (negative exponent) is a possible sign; '+' is never appended to
entry_buf." The function logic and all existing unit tests are unaffected.

---

_Fixed: 2026-05-08_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
