---
phase: 09-infrastructure-and-eex-fix
reviewed: 2026-05-08T00:00:00Z
depth: standard
files_reviewed: 5
files_reviewed_list:
  - Cargo.toml
  - .github/workflows/ci.yml
  - hp41-core/src/ops/mod.rs
  - hp41-cli/src/app.rs
  - hp41-cli/src/ui.rs
findings:
  critical: 2
  warning: 2
  info: 1
  total: 5
status: issues_found
---

# Phase 9: Code Review Report

**Reviewed:** 2026-05-08
**Depth:** standard
**Files Reviewed:** 5
**Status:** issues_found

## Summary

Phase 9 delivers two independent streams: Stream A (MSRV enforcement — `rust-version = "1.85"`, `rust_decimal` bump to 1.42, MSRV CI job) and Stream B (EEX entry behavioral fix — trailing-e commit, empty-buffer EEX implicit mantissa, TUI exponent placeholder display). The EEX core logic and TUI display helpers are correctly implemented and well-tested. Two blockers were found — one in the CI infrastructure (MSRV job guaranteed to fail on every run) and one pre-existing behavioral defect in `app.rs` (modal interruption) that Phase 9 left unaddressed and undocumented. Two warnings round out the review.

---

## Critical Issues

### CR-01: MSRV CI job will always fail — missing `cargo-llvm-cov` and `clippy` component

**File:** `.github/workflows/ci.yml:63-73`

**Issue:** The MSRV job runs `just ci`, which resolves to the `ci: lint test coverage` justfile recipe. The `lint` recipe calls `cargo clippy` and the `coverage` recipe calls `cargo llvm-cov`. However, the MSRV job step only installs `just`:

```yaml
- uses: taiki-e/install-action@v2
  with:
    tool: just
- run: just ci
```

Two dependencies are missing:
1. `cargo-llvm-cov` binary is not installed (`tool: just,cargo-llvm-cov` is required).
2. The `llvm-tools-preview` Rust component is not declared (required by `cargo-llvm-cov`).
3. The `clippy` component is not declared for the 1.85 toolchain (the `dtolnay/rust-toolchain@1.85` action without `components: clippy` installs a minimal toolchain that may not include clippy).

Compare the working `coverage` job (lines 48-61) which correctly declares both `components: llvm-tools-preview` and `tool: just,cargo-llvm-cov`.

**Fix:**
```yaml
  msrv:
    name: MSRV (Rust 1.85)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.85
        with:
          components: clippy,llvm-tools-preview
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with:
          tool: just,cargo-llvm-cov
      - run: just ci
```

---

### CR-02: Modal interruption — 'S', 'R', and Ctrl+A silently discard any active `pending_input` modal

**File:** `hp41-cli/src/app.rs:159-178`

**Issue:** The guards that open STO, RCL, and key-assignment modals execute *before* the routing check `if self.pending_input.is_some()`. When any other modal is currently active (e.g., `ConfirmLoad`, `AssignLabel`, or even `StoRegister` itself), pressing 'S', 'R', or Ctrl+A silently discards the active modal state and replaces it with a new one. There is no Esc or confirmation — the user's prior confirmation prompt disappears without feedback.

```rust
// Lines 159-178: 'S'/'R'/Ctrl+A checks come BEFORE the pending_input routing guard
if key.code == KeyCode::Char('S') && !key.modifiers.contains(KeyModifiers::CONTROL) {
    self.pending_input = Some(PendingInput::StoRegister(String::new()));  // stomps ConfirmLoad
    ...
    return;
}
// ... same for 'R' and Ctrl+A ...
if self.pending_input.is_some() {   // too late — already overwritten
    self.handle_pending_input(key);
    return;
}
```

A concrete bad scenario: user sees "Load 'Fibonacci'? Current program will be lost. [Y/n]" and accidentally taps 'S' — the load confirmation is silently discarded, a STO modal opens, and the user never learns the load was cancelled.

**Fix:** Move the 'S', 'R', and Ctrl+A interceptors to *after* the `pending_input` routing check, so an active modal takes priority:

```rust
// Route to pending_input handler if modal is active — FIRST
if self.pending_input.is_some() {
    self.handle_pending_input(key);
    return;
}

// Only open new modals when no modal is currently active
if key.code == KeyCode::Char('S') && !key.modifiers.contains(KeyModifiers::CONTROL) {
    self.pending_input = Some(PendingInput::StoRegister(String::new()));
    self.message = None;
    return;
}
// ... R and Ctrl+A follow the same pattern ...
```

---

## Warnings

### WR-01: Member crates do not inherit `rust-version` from workspace — MSRV enforcement is incomplete

**File:** `Cargo.toml:6`, `hp41-core/Cargo.toml`, `hp41-cli/Cargo.toml`

**Issue:** The workspace root declares `rust-version = "1.85"` under `[workspace.package]`, but neither `hp41-core` nor `hp41-cli` opt in to this field. In Cargo's workspace inheritance model, `workspace.package` fields are opt-in: member crates must explicitly declare `rust-version.workspace = true` in their `[package]` section to inherit the workspace value. Without this, `cargo metadata` reports `rust_version: None` for both member crates, meaning `cargo check --manifest-path hp41-core/Cargo.toml` ignores the 1.85 minimum entirely.

Verified via:
```
cargo metadata --no-deps | jq '.packages[] | {name, rust_version}'
# hp41-core: null
# hp41-cli: null
```

**Fix:** Add `rust-version.workspace = true` to both member crate `[package]` sections:

```toml
# hp41-core/Cargo.toml
[package]
name = "hp41-core"
version = "0.1.0"
edition = "2021"
rust-version.workspace = true
```

```toml
# hp41-cli/Cargo.toml
[package]
name = "hp41-cli"
version = "0.1.0"
edition = "2021"
rust-version.workspace = true
```

---

### WR-02: `entry_buf` is cleared before parse — silent data loss on malformed input

**File:** `hp41-core/src/ops/mod.rs:211-222`

**Issue:** `flush_entry_buf()` clears `state.entry_buf` on line 212, unconditionally, before the parse attempt on lines 220-222. If parsing fails and `HpError::InvalidOp` is returned, the user's in-progress entry string is already gone. The caller (`dispatch()`) propagates the error up to `call_dispatch()` in `app.rs`, which sets `self.message = Some(format!("{e}"))` — but the display now shows the raw X register value rather than the partial number that was being typed.

This is a pre-existing pattern (present before Phase 9), but Phase 9's new trailing-'e' normalization touches this function and is the right moment to address it. The comment on line 205 calls this path "defensive" and "unreachable in practice" — but that is a design assumption, not a guarantee, and it means the bug silently manifests when the assumption breaks.

**Fix:** Preserve the buffer on parse failure so the user can see what went wrong:

```rust
pub fn flush_entry_buf(state: &mut CalcState) -> Result<(), HpError> {
    if state.entry_buf.is_empty() {
        return Ok(());
    }
    let mut s = state.entry_buf.clone();
    if s.ends_with('e') || s.ends_with('E') {
        s.push_str("00");
    }
    let d = Decimal::from_str(&s)
        .or_else(|_| Decimal::from_scientific(&s))
        .map_err(|_| HpError::InvalidOp)?;  // return Err WITHOUT clearing entry_buf
    state.entry_buf.clear();  // only clear after successful parse
    let n = HpNum::rounded(d);
    // ... rest unchanged
```

---

## Info

### IN-01: `format_entry_buf_display` handles `'+'` exponent sign that can never appear in `entry_buf`

**File:** `hp41-cli/src/ui.rs:171-177`

**Issue:** The display formatting function `format_entry_buf_display` includes a branch to strip and preserve a leading `'+'` from the exponent part:

```rust
} else if let Some(rest) = after_e.strip_prefix('+') {
    ("+", rest)
}
```

No code path in `handle_key()` ever appends `'+'` to `entry_buf`. Only digits and `'e'` (and potentially `'-'` if CHS during exponent entry were implemented, which it is not) are ever pushed. The `'+'` branch is dead code. It does no harm and the function is pure/isolated, but it is misleading: it implies `entry_buf` can contain `'+'`, which it cannot, and it makes the format contract harder to reason about.

**Fix:** Remove the `'+'` branch and add a comment explaining why sign characters are not expected:

```rust
// Only '-' (negative exponent) is a possible sign; '+' is never appended to entry_buf.
let (sign, digits) = if let Some(rest) = after_e.strip_prefix('-') {
    ("-", rest)
} else {
    ("", after_e)
};
```

---

_Reviewed: 2026-05-08_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
