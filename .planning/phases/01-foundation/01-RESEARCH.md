# Phase 1: Foundation - Research

**Researched:** 2026-05-06
**Domain:** Rust Cargo workspace setup, HP-41 RPN stack model, BCD vs f64 numeric representation, stack-lift semantics, error types, `just` task runner
**Confidence:** HIGH (core stack/Rust topics), MEDIUM (stack-lift classification completeness)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **BCD vs f64:** Evaluate `rust_decimal` vs custom BCD struct; commit decision to `state.rs` as ADR comment
- **Stack-lift flag:** `lift_enabled: bool` in `Stack`; every operation declares Enable/Disable/Neutral effect
- **CalcState:** Single `&mut CalcState` passed through all operations; no global mutable state
- **Error type:** `Result<T, HpError>` — no panics in hp41-core
- **Workspace structure:** `hp41-core/` (library, zero UI deps) + `hp41-cli/` (binary, thin adapter)

### Claude's Discretion
All implementation choices are at Claude's discretion — pure infrastructure phase.

### Deferred Ideas (OUT OF SCOPE)
None — infrastructure phase.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CORE-01 | User has a 4-level RPN stack (X/Y/Z/T) and LASTX register that behaves identically to HP-41 hardware | Stack struct design, `binary_result()` pattern from Free42, LASTX capture rules |
| CORE-02 | All ~130 operations implement correct stack-lift semantics (Enable / Disable / Neutral) per HP-41 specification | Stack-lift classification table, `lift_enabled: bool` flag semantics |
</phase_requirements>

---

## Summary

Phase 1 builds the foundation that all subsequent phases depend on. The two most consequential decisions are the numeric representation (BCD vs f64) and the data model for `CalcState`. Getting these wrong means a full rewrite before Phase 2 ships.

**Numeric representation verdict:** Use `rust_decimal` 1.41 with a thin `HpNum` newtype wrapper that enforces 10-significant-digit rounding after every operation. The HP-41 hardware stores numbers as 10-digit BCD mantissa + 2-digit exponent (56-bit register), but behavioral emulation does not require cycle-accurate BCD storage. `rust_decimal` is decimal-native (no binary float rounding error), supports up to 28 significant digits (excess is rounded away via `round_sf(10)`), and is actively maintained. A custom BCD struct adds ~500 lines of arithmetic code and identical rounding behavior for the emulation use case. The ISG/DSE counter format (`CCCCC.FFFDD`) must be handled by string-splitting at the decimal point regardless of which representation is chosen — this is explicitly noted in CLAUDE.md.

**Stack-lift model verdict:** The `lift_enabled: bool` flag approach (already decided) matches the Free42 reference implementation (`flags.f.stack_lift_disable`). Every operation must declare one of three stack-lift effects: Enable (most operations — set `lift_enabled = true`), Disable (ENTER, CLX — set `lift_enabled = false`), or Neutral (display/mode ops — leave `lift_enabled` unchanged). Number entry checks `lift_enabled` to decide whether to push or overwrite X.

**Primary recommendation:** `HpNum = newtype over rust_decimal::Decimal` with `round_sf(10)` post-operation enforcement; `Stack { x, y, z, t, lastx, lift_enabled: bool }` as the sole numeric state carrier; `CalcState { stack: Stack, .. }` as the root; `HpError` via `thiserror 2.0`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| 4-level RPN stack state | `hp41-core` lib | — | Zero-UI requirement; CalcState is pure Rust struct |
| Numeric representation (HpNum) | `hp41-core` lib | — | All arithmetic belongs in core, not CLI adapter |
| Stack-lift flag management | `hp41-core` lib | — | Flag is part of CalcState; must be consistent across all ops |
| LASTX register update | `hp41-core` lib | — | Hardware behavior: capture X before consuming it in binary ops |
| Error propagation | `hp41-core` lib | `hp41-cli` (display) | Core returns `Result<T, HpError>`; CLI formats for display |
| Build / test / lint targets | Justfile (workspace root) | — | Single Justfile at workspace root; no per-crate Justfiles needed for Phase 1 |
| Coverage reporting | Justfile + cargo-llvm-cov | — | CI recipe calls `cargo llvm-cov --fail-under-lines 80` |

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rust_decimal | 1.41.0 | Decimal arithmetic (HpNum backing) | 28-digit decimal-native, no binary float error, `round_sf()` built-in |
| thiserror | 2.0.18 | `#[derive(Error)]` for HpError | Zero boilerplate for typed errors; no panic |
| just | 1.49.0 | Task runner (installed) | Sole task runner per CLAUDE.md; already installed on this machine |

[VERIFIED: cargo search rust_decimal, cargo search thiserror, `just --version` 1.49.0]

### Supporting (test / CI only)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| proptest | 1.11.0 | Property-based tests | Stack invariant verification across random op sequences |
| insta | 1.47.2 | Snapshot tests | Lock in exact stack state after named operation sequences |
| cargo-llvm-cov | 0.8.5 | Coverage gate (≥80%) | `just ci` → `cargo llvm-cov --fail-under-lines 80 -p hp41-core` |

[VERIFIED: cargo search proptest / insta / cargo-llvm-cov]

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| rust_decimal | Custom BCD struct | Custom BCD is ~500 LOC of nibble arithmetic with identical user-visible behavior; choose rust_decimal unless Phase 6 reveals precision gap vs hardware |
| rust_decimal | f64 + manual rounding | f64 has binary rounding artifacts (0.1 + 0.2 ≠ 0.3); ISG/DSE string-splitting is still required; not recommended |
| thiserror | anyhow | anyhow is for applications; thiserror gives typed errors in libraries — required here |
| proptest | quickcheck | proptest has richer Strategy API and better shrinking; preferred for stateful sequence testing |

**Installation:**
```bash
# Workspace dependencies (add to root Cargo.toml [workspace.dependencies])
# rust_decimal = "1.41"
# thiserror = "2.0"

# Dev/test dependencies (hp41-core/Cargo.toml [dev-dependencies])
# proptest = "1.11"
# insta = { version = "1.47", features = ["yaml"] }

# Install cargo-llvm-cov (one-time, not a Cargo dep)
cargo install cargo-llvm-cov --locked
rustup component add llvm-tools-preview
```

**Version verification:** [VERIFIED: `cargo search` on 2026-05-06 — rust_decimal 1.41.0, thiserror 2.0.18, proptest 1.11.0, insta 1.47.2, cargo-llvm-cov 0.8.5]

---

## Architecture Patterns

### System Architecture Diagram

```
User keystroke / test
        │
        ▼
 ┌─────────────┐
 │  hp41-cli   │  (Phase 4+) thin adapter, key→Op mapping
 └──────┬──────┘
        │ &mut CalcState + Op enum
        ▼
 ┌───────────────────────────────────────────────┐
 │                 hp41-core                     │
 │                                               │
 │  op::dispatch(state, op) → Result<(), HpError>│
 │        │                                      │
 │        ├─ stack::lift_or_overwrite(state, val)│
 │        │       checks state.stack.lift_enabled│
 │        │                                      │
 │        ├─ binary_result(state, result)        │
 │        │       state.stack.lastx ← old X      │
 │        │       state.stack.x ← result         │
 │        │       rotates y←z, z←t, t unchanged  │
 │        │       sets lift_enabled = true        │
 │        │                                      │
 │        └─ lift_effect of op sets lift_enabled │
 │                                               │
 │  CalcState { stack: Stack, .. }               │
 │  Stack { x, y, z, t, lastx, lift_enabled }    │
 └───────────────────────────────────────────────┘
```

### Recommended Project Structure
```
hp41-calculator-emulator/          (workspace root)
├── Cargo.toml                     (workspace manifest, shared deps)
├── Cargo.lock
├── Justfile                       (all build/test/lint/run/ci recipes)
├── hp41-core/
│   ├── Cargo.toml                 (lib crate, NO ui/cli deps)
│   └── src/
│       ├── lib.rs                 (public API re-exports)
│       ├── state.rs               (CalcState, Stack, HpNum — ADR comment here)
│       ├── error.rs               (HpError enum with thiserror)
│       ├── num.rs                 (HpNum newtype over Decimal, round_sf enforcement)
│       ├── stack.rs               (push, lift_or_overwrite, binary_result, pop)
│       └── ops/
│           ├── mod.rs             (Op enum + dispatch fn)
│           ├── arithmetic.rs      (add, sub, mul, div)
│           └── stack_ops.rs       (enter, clx, chs, rdn, xy_swap, lastx)
└── hp41-cli/
    ├── Cargo.toml                 (bin crate, depends on hp41-core)
    └── src/
        └── main.rs                (placeholder — thin adapter for later phases)
```

### Pattern 1: CalcState as Single Source of Truth
**What:** One `CalcState` struct passed by `&mut` to all operations. No global mutable state.
**When to use:** Every operation function signature.
**Example:**
```rust
// Source: CONTEXT.md decision + Free42 core_globals.h pattern
pub struct CalcState {
    pub stack: Stack,
    // future: regs: [HpNum; 100], alpha: String, flags: CalcFlags, ...
}

pub struct Stack {
    pub x: HpNum,
    pub y: HpNum,
    pub z: HpNum,
    pub t: HpNum,
    pub lastx: HpNum,
    pub lift_enabled: bool,
}

pub fn op_add(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.x.checked_add(state.stack.y)?;
    binary_result(state, result);    // sets lift_enabled = true
    Ok(())
}
```

### Pattern 2: Stack-Lift Tri-State via Enum + Per-Op Declaration
**What:** Every operation is tagged with its stack-lift effect at the callsite. Three effects: Enable, Disable, Neutral.
**When to use:** Every operation implementation.
**Example:**
```rust
// Source: Free42 common/core_main.cc + core_commands1.cc patterns
pub enum LiftEffect { Enable, Disable, Neutral }

/// Called by ALL operations that produce a result (arithmetic, math fns, RCL).
/// Saves X to LASTX, rotates stack down, places result in X, enables lift.
pub fn binary_result(state: &mut CalcState, result: HpNum) {
    state.stack.lastx = state.stack.x.clone();
    state.stack.x = result;
    state.stack.y = state.stack.z.clone();
    state.stack.z = state.stack.t.clone();
    // T is duplicated (HP-41 hardware behavior: T stays)
    state.stack.lift_enabled = true;
}

/// Called when user enters a digit sequence / numeric literal.
pub fn enter_number(state: &mut CalcState, value: HpNum) {
    if state.stack.lift_enabled {
        // push: T←Z, Z←Y, Y←X, X←value
        state.stack.t = state.stack.z.clone();
        state.stack.z = state.stack.y.clone();
        state.stack.y = state.stack.x.clone();
    }
    // if not enabled: X is simply overwritten
    state.stack.x = value;
    // number entry itself does NOT change lift_enabled
}
```

### Pattern 3: HpNum Newtype with 10-Digit Enforcement
**What:** Wrap `rust_decimal::Decimal` so every arithmetic result is rounded to 10 significant decimal digits immediately.
**When to use:** All numeric results leaving `hp41-core`.
**Example:**
```rust
// Source: [ASSUMED] — pattern based on rust_decimal 1.41 API (round_sf, RoundingStrategy)
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps; // if needed for future trig

#[derive(Clone, Debug, PartialEq)]
pub struct HpNum(Decimal);

impl HpNum {
    /// Enforce HP-41 10-significant-digit precision after any calculation.
    pub fn rounded(d: Decimal) -> Self {
        HpNum(d.round_sf(10).unwrap_or(d))
    }

    pub fn checked_add(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        self.0.checked_add(rhs.0)
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }
}
```

### Pattern 4: HpError with thiserror
**What:** Typed error enum for all hp41-core failure modes. No `unwrap()`, no `panic!`.
**When to use:** All fallible operations.
**Example:**
```rust
// Source: thiserror 2.0 docs.rs documentation
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum HpError {
    #[error("overflow")]
    Overflow,
    #[error("divide by zero")]
    DivideByZero,
    #[error("invalid operation")]
    InvalidOp,
    #[error("domain error")]
    Domain,
}
```

### Pattern 5: Justfile for Rust Workspace
**What:** All developer commands defined as `just` recipes. `just ci` is the single gate.
**When to use:** Root Justfile at workspace root.
**Example:**
```justfile
# Default — show available recipes
default:
    @just --list

# Build all workspace crates
build:
    cargo build --workspace

# Run all tests
test:
    cargo test --workspace

# Lint with clippy (warnings-as-errors)
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run the CLI (placeholder until Phase 4)
run:
    cargo run -p hp41-cli

# Check coverage gate (≥80% line coverage on hp41-core)
coverage:
    cargo llvm-cov --fail-under-lines 80 -p hp41-core

# Full CI gate: lint → test → coverage
ci: lint test coverage
```

[VERIFIED: just 1.49.0 syntax via just.systems/man/en and WebSearch; recipe dependency syntax confirmed]

### Anti-Patterns to Avoid
- **`unwrap()` / `panic!` in hp41-core:** Zero-panic invariant; all fallible ops return `Result<_, HpError>`. Use `?` for propagation.
- **Global mutable state (lazy_static / OnceLock with Mutex):** Everything through `&mut CalcState`; makes testing trivial and threading safe.
- **`f64` for HP-41 arithmetic:** Binary representation causes 0.1+0.2 ≠ 0.3; ISG/DSE field extraction via `floor()`/`fmod()` is explicitly forbidden (STATE.md).
- **Rolling your own BCD arithmetic:** ~500 extra LOC for byte-for-byte identical user-visible behavior; `rust_decimal` covers the behavioral requirement.
- **Per-crate Justfiles:** Use one root Justfile; `cargo -p <crate>` targets from a single file are sufficient for Phase 1.
- **Mutable references in test helpers:** Tests should construct fresh `CalcState` for each assertion; shared mutable state between test cases causes ordering bugs.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Decimal arithmetic without binary rounding error | Custom BCD nibble arithmetic | `rust_decimal` | 28-digit decimal-native; correct base-10 rounding; `round_sf(10)` built-in; handles special cases (overflow, NaN policy) |
| Typed error boilerplate | Manual `impl Display + Error` | `thiserror` derive macro | Eliminates 40+ lines of boilerplate per error type; `#[from]` auto-generates conversions |
| Coverage measurement | Manual `grcov` shell scripts | `cargo-llvm-cov` | Single command; `--fail-under-lines` flag for CI gate; HTML reports for local review |

**Key insight:** The HP-41 emulation correctness requirement (≥98% numerical agreement, QUAL-06) is behavioral, not structural. You need decimal-correct rounding, not BCD nibble storage.

---

## Runtime State Inventory

This is a greenfield phase — no existing runtime state. Section omitted per instructions (not a rename/refactor/migration phase).

---

## Common Pitfalls

### Pitfall 1: Stack-Lift Neutral Operations Mis-Classified as Enable
**What goes wrong:** PRGM mode toggle, VIEW, SST, BST, ALPHA mode entry all set `lift_enabled = true` accidentally, causing a spurious stack push when the user next types a number.
**Why it happens:** Default implementation sets `lift_enabled = true` as a catch-all; neutral ops are easy to forget.
**How to avoid:** Every operation function must explicitly declare its `LiftEffect`. Start with an exhaustive match or a dispatch table keyed by `LiftEffect`. Neutral ops call `apply_lift_effect(state, LiftEffect::Neutral)` which is a no-op on the flag.
**Warning signs:** A test where `VIEW 42` followed by digit entry produces a lifted stack.

### Pitfall 2: LASTX Captured at Wrong Time
**What goes wrong:** LASTX captures the result of the operation rather than the X value before the operation consumed it.
**Why it happens:** Setting `state.stack.lastx = result` instead of `state.stack.lastx = state.stack.x` (old value).
**How to avoid:** `binary_result()` must save `state.stack.x` into `lastx` **before** overwriting X. See Free42's `binary_result()` pattern above.
**Warning signs:** `1 ENTER 2 + LASTX` returns 3 instead of 2.

### Pitfall 3: ISG/DSE Counter Field Extraction via Float Arithmetic
**What goes wrong:** Using `floor(counter)` to extract integer part and `fmod(counter, 1.0) * 1000` to extract `.fff` causes rounding artifacts that shift the dd (step) field.
**Why it happens:** Floating-point representation of e.g. `1.01001` is not exactly representable.
**How to avoid:** Format `counter` as a decimal string, split at `.`, parse substrings for `iiiii`, `fff`, `dd` fields. The CONTEXT.md and CLAUDE.md both flag this explicitly.
**Warning signs:** ISG counter with dd=01 increments by 0 or 2.

### Pitfall 4: ENTER Semantics on Second ENTER Press
**What goes wrong:** Pressing ENTER twice should produce X duplicated in Y (T dropped), not three copies.
**Why it happens:** First ENTER lifts stack + duplicates X + disables lift. Second ENTER with lift disabled should lift again (ENTER always lifts) — ENTER's lift behavior is unconditional push regardless of `lift_enabled`.
**How to avoid:** ENTER implementation always pushes the stack (`T←Z, Z←Y, Y←X`) and then disables lift. It does NOT check `lift_enabled` before pushing.
**Warning signs:** `3 ENTER ENTER` shows 3 in X but Y is garbage instead of 3.

### Pitfall 5: T Register Not Duplicating on Stack Push
**What goes wrong:** After a stack lift, T register is empty/zero instead of holding the old T value.
**Why it happens:** Implementing lift as a 4-element rotate discards T.
**How to avoid:** HP-41 hardware duplicates T on lift: `T←T` (T stays), `Z←T`, `Y←Z_old`, `X←Y_old`. T is only destroyed when it is popped off (arithmetic consumes Y, so Z→Y, T→Z).
**Warning signs:** Stack test pushing 4 values then doing addition shows 0 in T instead of first-pushed value.

### Pitfall 6: rust_decimal Default Rounding (Bankers Rounding) Instead of HP-41 Rounding
**What goes wrong:** `rust_decimal` defaults to `MidpointNearestEven` (Bankers Rounding). HP-41 hardware uses "round half away from zero" (MidpointAwayFromZero) for display rounding.
**Why it happens:** rust_decimal's default rounding strategy is financial/statistical, not calculator-style.
**How to avoid:** Always use `round_sf_with_strategy(10, RoundingStrategy::MidpointAwayFromZero)` in `HpNum::rounded()`.
**Warning signs:** 2.5 rounds to 2 instead of 3 in test output.

### Pitfall 7: cargo-llvm-cov Not Installed in CI
**What goes wrong:** `just ci` fails on a fresh checkout because `cargo-llvm-cov` is not in PATH.
**Why it happens:** It is not a `[dev-dependencies]` entry — it must be installed separately.
**How to avoid:** Wave 0 task installs it: `cargo install cargo-llvm-cov --locked && rustup component add llvm-tools-preview`. Document in README. Add a Justfile recipe `install-tools` that runs both install commands.
**Warning signs:** `just ci` exits with "error: no such subcommand: `llvm-cov`".

---

## Code Examples

### Workspace Cargo.toml
```toml
# Source: doc.rust-lang.org/book/ch14-03-cargo-workspaces.html [CITED]
[workspace]
resolver = "2"
members = ["hp41-core", "hp41-cli"]

[workspace.dependencies]
rust_decimal = "1.41"
thiserror = "2.0"
```

### hp41-core/Cargo.toml
```toml
[package]
name = "hp41-core"
version = "0.1.0"
edition = "2021"

[dependencies]
rust_decimal = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
proptest = "1.11"
insta = { version = "1.47", features = ["yaml"] }
```

### hp41-cli/Cargo.toml
```toml
[package]
name = "hp41-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "hp41"
path = "src/main.rs"

[dependencies]
hp41-core = { path = "../hp41-core" }
```

### Proptest Stack Sequence Pattern
```rust
// Source: rtpg.co/2024/02/02/property-testing-with-imperative-rust/ [CITED]
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum StackOp { Push(f64), Enter, Add, Clx }

proptest! {
    #[test]
    fn stack_never_panics(ops in proptest::collection::vec(stack_op_strategy(), 0..20)) {
        let mut state = CalcState::new();
        for op in ops {
            let _ = apply_op(&mut state, op); // ignore errors, must not panic
        }
        // stack always has defined values
        prop_assert!(state.stack.x.is_finite());
    }
}
```

### Insta Snapshot for Named Sequences
```rust
// Source: docs.rs/insta/latest/insta/ [CITED]
#[test]
fn enter_then_add() {
    let mut state = CalcState::new();
    push_number(&mut state, HpNum::from(3));
    op_enter(&mut state).unwrap();
    push_number(&mut state, HpNum::from(4));
    op_add(&mut state).unwrap();
    insta::assert_debug_snapshot!(state.stack);
    // First run creates snapshot; subsequent runs verify it
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Cargo `resolver = "1"` | `resolver = "2"` (or `"3"` on Rust 2024) | Rust 1.51 / 2024 edition | Feature unification is more predictable |
| `failure` crate for errors | `thiserror` 2.0 | ~2020 (thiserror); 2.0 released 2024 | `failure` is unmaintained; thiserror 2.0 has improved `#[from]` behavior |
| `grcov` for coverage | `cargo-llvm-cov` | ~2022 | Simpler workflow; native `--fail-under-*` flags; official taiki-e tooling |
| Manual error `Display` impls | `#[derive(Error)]` from thiserror | ~2019 | Eliminates boilerplate |

**Deprecated/outdated:**
- `failure` crate: unmaintained, replaced by `thiserror` + `anyhow`
- Cargo `resolver = "1"`: use `"2"` minimum; `"3"` if targeting Rust 1.84+ (2024 edition)
- `grcov`: functional but more complex; `cargo-llvm-cov` is the current standard

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `rust_decimal::Decimal::round_sf_with_strategy(10, MidpointAwayFromZero)` matches HP-41 display rounding exactly across all 500 test cases | Standard Stack, Pattern 3 | Phase 7 (QUAL-06) could fail; would require tuning rounding strategy or switching to custom BCD |
| A2 | HP-41 T register is duplicated (not dropped) on every stack lift | Pattern 2 / Pitfall 5 | Stack tests pass but programs that depend on T holding old value fail |
| A3 | The LiftEffect for operations not yet implemented (SIGMA+, MEAN, trig) follows the same three-category rule as HP-11C/HP-35s documentation | Stack-lift classification | Incorrect lift after stats/trig ops; discoverable in Phase 2 test suite |
| A4 | Justfile recipe dependency syntax `ci: lint test coverage` (space-separated) is valid in just 1.49.0 | Pattern 5 | `just ci` fails to run prerequisite recipes; fixable quickly |
| A5 | `resolver = "2"` is the correct workspace resolver for Rust 1.89 stable (not `"3"`) | Code Examples | Build warns about resolver version; harmless but noisy |

---

## Open Questions

1. **Rust 2021 vs 2024 edition for hp41-core**
   - What we know: Rust 1.89 stable supports edition 2024; `resolver = "3"` is used with 2024 edition
   - What's unclear: Whether CLAUDE.md's "MSRV 1.78+" constraint means edition 2021 is preferred for broader compatibility
   - Recommendation: Use `edition = "2021"` and `resolver = "2"` (stable, widely supported); upgrade to 2024 when MSRV is updated

2. **rust_decimal `maths` feature: needed in Phase 1?**
   - What we know: `rust_decimal`'s `maths` feature enables `ln()`, `exp()`, `pow()` — needed for Phase 2 (trig, logs)
   - What's unclear: Whether enabling `maths` in Phase 1 is premature or acceptable
   - Recommendation: Add `rust_decimal = { version = "1.41", features = [] }` in Phase 1 with no maths feature; Phase 2 adds `features = ["maths"]`

3. **HpNum handling of HP-41 special display values (OVERFLOW, UNDERFLOW)**
   - What we know: HP-41 displays "9.999999999E99" on overflow and "0" on underflow
   - What's unclear: Whether `HpError::Overflow` should return the overflow display value or just an error
   - Recommendation: Return `Err(HpError::Overflow)` in Phase 1; Phase 4 (display) maps errors to display strings

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustc / cargo | All Rust compilation | ✓ | 1.89.0 stable | — |
| just | Justfile task runner | ✓ | 1.49.0 | — |
| cargo-llvm-cov | `just ci` coverage gate | ✗ | — | Wave 0 must install: `cargo install cargo-llvm-cov --locked` |
| llvm-tools-preview | cargo-llvm-cov dependency | ✗ | — | Wave 0: `rustup component add llvm-tools-preview` |

**Missing dependencies with no fallback:**
- `cargo-llvm-cov` + `llvm-tools-preview` — required by `just ci` coverage recipe; must be installed before the CI recipe will pass

**Missing dependencies with fallback:**
- None beyond the above

[VERIFIED: `rustc --version`, `cargo --version`, `just --version`, `cargo llvm-cov --version` — all run 2026-05-06]

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + proptest 1.11.0 + insta 1.47.2 |
| Config file | none — standard `cargo test` discovery |
| Quick run command | `cargo test -p hp41-core` |
| Full suite command | `cargo llvm-cov --fail-under-lines 80 -p hp41-core` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CORE-01 | 4-level stack push/pop, LASTX capture on binary ops | unit | `cargo test -p hp41-core stack` | ❌ Wave 0 |
| CORE-01 | ENTER duplicates X, disables lift | unit | `cargo test -p hp41-core enter` | ❌ Wave 0 |
| CORE-01 | CLX zeros X, disables lift | unit | `cargo test -p hp41-core clx` | ❌ Wave 0 |
| CORE-01 | CHS negates X, neutral lift | unit | `cargo test -p hp41-core chs` | ❌ Wave 0 |
| CORE-01 | RCL copies register to X, enables lift | unit | `cargo test -p hp41-core rcl` | ❌ Wave 0 |
| CORE-02 | All ~130 ops declare lift effect; none are undeclared | unit | `cargo test -p hp41-core lift_effects` | ❌ Wave 0 |
| CORE-02 | Stack-lift invariant holds for random op sequences | proptest | `cargo test -p hp41-core proptest` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p hp41-core`
- **Per wave merge:** `cargo llvm-cov --fail-under-lines 80 -p hp41-core`
- **Phase gate:** Full suite green + `cargo check -p hp41-core` with zero UI deps before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `hp41-core/src/lib.rs` — public API skeleton
- [ ] `hp41-core/src/state.rs` — CalcState, Stack, HpNum definitions
- [ ] `hp41-core/src/error.rs` — HpError enum
- [ ] `hp41-core/tests/stack_tests.rs` — CORE-01 unit tests
- [ ] `hp41-core/tests/lift_tests.rs` — CORE-02 lift-effect tests
- [ ] `hp41-core/tests/proptest_stack.rs` — CORE-02 property tests
- [ ] `Justfile` — workspace root with all 5 required recipes
- [ ] cargo-llvm-cov installation: `cargo install cargo-llvm-cov --locked && rustup component add llvm-tools-preview`

---

## Security Domain

`security_enforcement` is not set in `.planning/config.json` (treated as enabled). However, this phase implements a pure local math library with no I/O, no user input parsing from untrusted sources, no network calls, and no file operations. The applicable ASVS scope is minimal.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | partial | `HpError::Domain` for invalid math inputs (sqrt of negative, log(0)); no untrusted string parsing in Phase 1 |
| V6 Cryptography | no | — |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Integer overflow / unwrap panic in arithmetic | Tampering / DoS | `checked_add`, `checked_mul` on `rust_decimal`; return `Err(HpError::Overflow)` |
| Stack index out of bounds (future array-based stack) | Tampering | Use fixed-size struct (not Vec); no indexing required |

---

## Sources

### Primary (HIGH confidence)
- `cargo search rust_decimal` / `cargo search thiserror` — verified crate versions on 2026-05-06
- `rustc --version` — confirmed Rust 1.89.0 stable installed
- `just --version` — confirmed just 1.49.0 installed
- [github.com/SammysHP/free42-linux-archive common/core_main.cc](https://github.com/SammysHP/free42-linux-archive) — Free42 stack lift implementation (`mode_disable_stack_lift`, `flags.f.stack_lift_disable`)
- [docs.rs/rust_decimal/latest/rust_decimal/struct.Decimal.html](https://docs.rs/rust_decimal/latest/rust_decimal/struct.Decimal.html) — precision (28 digits), `round_sf`, RoundingStrategy
- [docs.rs/thiserror/latest/thiserror](https://docs.rs/thiserror/latest/thiserror/) — `#[derive(Error)]` syntax
- [doc.rust-lang.org/book/ch14-03-cargo-workspaces.html](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) — workspace Cargo.toml structure

### Secondary (MEDIUM confidence)
- [HP-41C Quick Reference Guide PDF](https://literature.hpcalc.org/community/hp41c-qrg-en.pdf) — full function index (pages 6-9), stack diagram (page 4), ISG/DSE format (page 13), flags (page 11-12)
- [manualsdir.com HP-35s page 305](https://www.manualsdir.com/manuals/89925/hp-35s-scientific-calculator.html?page=305) — disabling/neutral operations (HP-35s shares stack-lift conventions with HP-41)
- [rtpg.co property testing stateful Rust](https://rtpg.co/2024/02/02/property-testing-with-imperative-rust/) — proptest operation-enum pattern for stateful testing
- [github.com/taiki-e/cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) — `--fail-under-lines` flag, install command

### Tertiary (LOW confidence)
- WebSearch results on stack-lift neutral operation classification — corroborate Enable/Disable/Neutral tri-state but full HP-41-specific list is not publicly available as machine-readable text; must be reconstructed from HP-41CV Owner's Handbook PDF section ~page 254 [ASSUMED]

---

## Metadata

**Confidence breakdown:**
- Standard stack (crates, versions): HIGH — verified via `cargo search` on 2026-05-06
- Workspace structure: HIGH — verified against official Rust book
- Stack-lift semantics (tri-state model): HIGH — confirmed via Free42 source + HP-35s manual + multiple forum sources
- Stack-lift per-operation classification (neutral list): MEDIUM — HP-41CV manual section exists but full text not extracted; pattern confirmed from HP-35s manual and Free42 source
- Justfile recipe syntax: MEDIUM — confirmed via WebSearch + just.systems docs (specific recipe dep syntax not extracted from manual page)
- rust_decimal rounding strategy for HP-41 match: LOW (A1) — needs Phase 7 validation against 500-case suite

**Research date:** 2026-05-06
**Valid until:** 2026-08-06 (stable libraries; just and rust_decimal have long release cycles)
