# Card Reader Manual Verification — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the four Card Reader ops (`WPRGM`/`RDPRGM`/`WDTA`/`RDTA`) reachable, drainable, and provably correct from both `hp41-cli` and `hp41-gui`, then publish a user-facing manual verification procedure that walks an operator through enter → save → clear → load → run with behavioural + byte-stable SHA-256 round-trip checks.

**Architecture:** A focused 4-entry XEQ-by-name fallback in `hp41-core/src/ops/program.rs` plumbs `Op::Xeq("WPRGM")` / etc. into the existing op handlers. Each frontend gets a new `cards.rs` module that owns path resolution and the `pending_card_op` drain helper; codec calls go through the public `hp41-core::cardreader` API only (SC-4 invariant). The drain runs at the same sites that already call `drain_and_show_print_output()` in the CLI and right after `dispatch` in the GUI command handlers.

**Tech Stack:** Rust 1.88 (workspace MSRV), `dirs` crate for `~/.hp41/cards/` resolution, `sha2` (dev-only) for round-trip hash tests, `tempfile` (already a workspace dev-dep) for sandboxed FS tests, Tauri v2.11 for GUI commands. No new runtime deps in `hp41-core` or `hp41-gui`.

**Spec:** `docs/superpowers/specs/2026-05-13-card-reader-manual-verification-design.md`

---

## File Map

**New files:**

- `hp41-core/tests/cardreader_xeq_tests.rs` — XEQ-by-name resolution tests.
- `hp41-cli/src/cards.rs` — `cards_dir()`, `sanitize_name()`, `drain_pending_card_op()` for the CLI.
- `hp41-cli/tests/card_io_tests.rs` — CLI round-trip + error-path integration tests.
- `hp41-gui/src-tauri/src/cards.rs` — mirror drain helper for the GUI.
- `hp41-gui/src-tauri/tests/card_io_tests.rs` — GUI round-trip integration test, asserts byte-identity with CLI.
- `docs/verifying-card-reader.md` — user-facing manual verification procedure.

**Modified files:**

- `hp41-core/src/ops/program.rs` — add `builtin_card_op()` helper, wire into `run_program`, `run_loop::Op::Xeq` arm, and `op_xeq`.
- `hp41-cli/Cargo.toml` — add `dirs` to `[dependencies]`, `sha2` to `[dev-dependencies]`.
- `hp41-cli/src/lib.rs` (or `main.rs` — discover at first edit) — `mod cards;` declaration.
- `hp41-cli/src/app.rs` — call `cards::drain_pending_card_op(...)` in `call_dispatch_and_drain` and after every `hp41_core::run_program(...)` call site (currently lines ~280, ~447, ~944).
- `hp41-cli/src/keys.rs` — four comfort-shortcut arms in `key_to_op()` plus four `KEY_REF_TABLE` entries.
- `hp41-cli/src/help_data.rs` — four `HELP_DATA` entries.
- `hp41-gui/src-tauri/src/lib.rs` — `mod cards;` declaration.
- `hp41-gui/src-tauri/src/commands.rs` — call drain in `handle_op` and (defensively) `handle_get_state`.
- `docs/operations-reference.md` — link to `docs/verifying-card-reader.md` in See Also.
- `docs/programming-guide.md` — link to `docs/verifying-card-reader.md` in See Also.

---

## Task 1: Core — `builtin_card_op` helper

**Files:**
- Modify: `hp41-core/src/ops/program.rs` (add helper near the existing `find_in_program` / `find_label_in_state` helpers, ~line 435)

- [ ] **Step 1: Write the failing test**

Add to the existing `program_tests` module at the bottom of `hp41-core/src/ops/program.rs`:

```rust
#[test]
fn builtin_card_op_resolves_four_names() {
    use crate::ops::program::builtin_card_op;
    use crate::ops::Op;
    assert_eq!(builtin_card_op("WPRGM"), Some(Op::Wprgm));
    assert_eq!(builtin_card_op("RDPRGM"), Some(Op::Rdprgm));
    assert_eq!(builtin_card_op("WDTA"), Some(Op::Wdta));
    assert_eq!(builtin_card_op("RDTA"), Some(Op::Rdta));
    assert_eq!(builtin_card_op("wprgm"), None, "case-sensitive — HP-41 names are uppercase");
    assert_eq!(builtin_card_op("UNKNOWN"), None);
    assert_eq!(builtin_card_op(""), None);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `just test -p hp41-core program_tests::builtin_card_op_resolves_four_names`
Expected: FAIL — "cannot find function `builtin_card_op` in module `crate::ops::program`".

- [ ] **Step 3: Implement the helper**

Add just above `mod program_tests` in `hp41-core/src/ops/program.rs`:

```rust
/// XEQ-by-name fallback: resolves the four Card Reader op names to their
/// `Op` variants. Returns `None` for anything else — including unknown
/// names, lowercase variants, and any built-in not in the Card Reader set.
///
/// Used only as a label-miss fallback in `run_program`, `run_loop` (the
/// `Op::Xeq` arm), and `op_xeq`. User `LBL "name"` matches take precedence,
/// matching real HP-41 `XEQ "name"` resolution order.
///
/// Deliberately *not* a general built-in dispatcher — Spec §"Out of Scope".
pub(super) fn builtin_card_op(name: &str) -> Option<Op> {
    match name {
        "WPRGM" => Some(Op::Wprgm),
        "RDPRGM" => Some(Op::Rdprgm),
        "WDTA" => Some(Op::Wdta),
        "RDTA" => Some(Op::Rdta),
        _ => None,
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `just test -p hp41-core program_tests::builtin_card_op_resolves_four_names`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
/git-workflow:commit --with-skills
```

Suggested message:

```
✨ feat(core): add builtin_card_op XEQ-by-name resolver

Focused 4-entry helper that maps "WPRGM"/"RDPRGM"/"WDTA"/"RDTA"
to their Op variants. Will be wired into run_program, run_loop,
and op_xeq as a label-miss fallback in subsequent commits.
```

---

## Task 2: Core — Wire fallback into `run_program`

**Files:**
- Modify: `hp41-core/src/ops/program.rs:127-145` (the `run_program` function)
- Test: `hp41-core/tests/cardreader_xeq_tests.rs` (new)

The CLI calls `hp41_core::run_program(state, &label)` directly for interactive XEQ. The fallback fires when the label scan returns no match: dispatch the built-in op once, return Ok(()).

- [ ] **Step 1: Create the new test file with a failing test**

Create `hp41-core/tests/cardreader_xeq_tests.rs`:

```rust
//! XEQ-by-name fallback tests for the four Card Reader ops.
//! Spec: docs/superpowers/specs/2026-05-13-card-reader-manual-verification-design.md

#![allow(clippy::unwrap_used)]

use hp41_core::cardreader::CardOpRequest;
use hp41_core::ops::Op;
use hp41_core::run_program;
use hp41_core::state::CalcState;

fn state_with_alpha(name: &str) -> CalcState {
    let mut state = CalcState::new();
    state.alpha = name.to_string();
    state
}

#[test]
fn run_program_xeq_wprgm_stages_write_program_request() {
    let mut state = state_with_alpha("QUAD");
    // program is empty — no LBL "WPRGM" anywhere
    run_program(&mut state, "WPRGM").expect("XEQ WPRGM via run_program must succeed");
    assert_eq!(
        state.pending_card_op,
        Some(CardOpRequest::WriteProgram { name: "QUAD".to_string() }),
    );
}

#[test]
fn run_program_xeq_rdprgm_stages_read_program_request() {
    let mut state = state_with_alpha("QUAD");
    run_program(&mut state, "RDPRGM").expect("XEQ RDPRGM via run_program must succeed");
    assert_eq!(
        state.pending_card_op,
        Some(CardOpRequest::ReadProgram { name: "QUAD".to_string() }),
    );
}

#[test]
fn run_program_xeq_wdta_stages_write_data_request() {
    let mut state = state_with_alpha("BACKUP");
    run_program(&mut state, "WDTA").unwrap();
    assert_eq!(
        state.pending_card_op,
        Some(CardOpRequest::WriteData { name: "BACKUP".to_string() }),
    );
}

#[test]
fn run_program_xeq_rdta_stages_read_data_request() {
    let mut state = state_with_alpha("BACKUP");
    run_program(&mut state, "RDTA").unwrap();
    assert_eq!(
        state.pending_card_op,
        Some(CardOpRequest::ReadData { name: "BACKUP".to_string() }),
    );
}

#[test]
fn run_program_unknown_label_still_errors() {
    let mut state = state_with_alpha("X");
    let err = run_program(&mut state, "TOTALLY_UNKNOWN").unwrap_err();
    // Existing behavior is HpError::InvalidOp on label miss; the fallback
    // must not change that for non-card names.
    assert!(
        matches!(err, hp41_core::error::HpError::InvalidOp),
        "unknown label must still surface InvalidOp, got {err:?}",
    );
}

#[test]
fn user_label_takes_precedence_over_builtin() {
    // If the operator's program has LBL "WPRGM", that label must win.
    // Guards against accidental shadowing of legitimate user code.
    let mut state = state_with_alpha("QUAD");
    state.program = vec![
        Op::Lbl("WPRGM".to_string()),
        Op::Rtn,
    ];
    run_program(&mut state, "WPRGM").expect("user LBL WPRGM must run, not stage a card op");
    assert!(
        state.pending_card_op.is_none(),
        "user LBL must take precedence over builtin fallback — no card op should be staged",
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `just test -p hp41-core --test cardreader_xeq_tests`
Expected: The four `_stages_*_request` tests FAIL with `HpError::InvalidOp` (label not found). `unknown_label_still_errors` PASSES already. `user_label_takes_precedence_over_builtin` PASSES already (the LBL exists, so no miss occurs).

- [ ] **Step 3: Wire the fallback into `run_program`**

In `hp41-core/src/ops/program.rs`, replace the label-scan block in `run_program` (around lines 131-135):

```rust
    // Linear scan for entry label (D-02)
    let start = program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if l == entry_label))
        .ok_or(HpError::InvalidOp)?;
```

With:

```rust
    // Linear scan for entry label (D-02). On miss, try the XEQ-by-name
    // fallback for the four Card Reader ops (Phase 19 spec). User labels
    // always take precedence — fallback only fires on a true miss.
    let start = match program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if l == entry_label))
    {
        Some(idx) => idx,
        None => {
            if let Some(op) = builtin_card_op(entry_label) {
                // Dispatch the built-in once and return — no program to run.
                // is_running stays false; we never enter run_loop.
                return crate::ops::dispatch(state, op);
            }
            return Err(HpError::InvalidOp);
        }
    };
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `just test -p hp41-core --test cardreader_xeq_tests`
Expected: All six tests PASS.

- [ ] **Step 5: Run full core suite**

Run: `just test -p hp41-core`
Expected: All tests PASS (no regression in existing program_tests, lift_tests, etc.).

- [ ] **Step 6: Commit**

```bash
/git-workflow:commit --with-skills
```

Suggested message:

```
✨ feat(core): wire XEQ-by-name fallback into run_program

run_program(state, "WPRGM") now stages a WriteProgram request
when no user LBL "WPRGM" exists, enabling the CLI's interactive
XEQ flow to reach the Card Reader ops. User labels still win.
```

---

## Task 3: Core — Wire fallback into `run_loop` XEQ arm

**Files:**
- Modify: `hp41-core/src/ops/program.rs:181-189` (the `Op::Xeq(label)` arm of `run_loop`)
- Test: `hp41-core/tests/cardreader_xeq_tests.rs` (extend)

A program step `XEQ "WPRGM"` inside a running program must also stage a request. Today the arm calls `find_in_program(program, &label)?` which errors on miss.

- [ ] **Step 1: Add failing test**

Append to `hp41-core/tests/cardreader_xeq_tests.rs`:

```rust
#[test]
fn run_loop_xeq_wprgm_inside_program_stages_request() {
    // Program: LBL "MAIN" / XEQ "WPRGM" / RTN
    // Running MAIN should execute the XEQ step, which stages a WriteProgram
    // request and then returns to MAIN's RTN (top-level → terminate).
    let mut state = state_with_alpha("CARD1");
    state.program = vec![
        Op::Lbl("MAIN".to_string()),
        Op::Xeq("WPRGM".to_string()),
        Op::Rtn,
    ];
    run_program(&mut state, "MAIN").expect("MAIN must run cleanly");
    assert_eq!(
        state.pending_card_op,
        Some(CardOpRequest::WriteProgram { name: "CARD1".to_string() }),
    );
}
```

- [ ] **Step 2: Run test, verify it fails**

Run: `just test -p hp41-core --test cardreader_xeq_tests run_loop_xeq_wprgm_inside_program_stages_request`
Expected: FAIL — `find_in_program` returns `InvalidOp` because no `LBL "WPRGM"` exists.

- [ ] **Step 3: Wire fallback into the `run_loop` XEQ arm**

In `hp41-core/src/ops/program.rs`, replace the `Op::Xeq(label) =>` arm (lines 181-189):

```rust
            Op::Xeq(label) => {
                if state.call_stack.len() >= 4 {
                    return Err(HpError::CallDepth); // D-13/D-14: error before mutation
                }
                // find target before pushing to call_stack (error-before-mutation)
                let target = find_in_program(program, &label)?;
                state.call_stack.push(state.pc);
                state.pc = target + 1;
            }
```

With:

```rust
            Op::Xeq(label) => {
                if state.call_stack.len() >= 4 {
                    return Err(HpError::CallDepth); // D-13/D-14: error before mutation
                }
                // User-label lookup first; on miss fall back to the four
                // Card Reader built-ins (Phase 19 spec). Built-in dispatch
                // does NOT push the call stack — it's a single op, not a
                // subroutine call, so pc just advances.
                match find_in_program(program, &label) {
                    Ok(target) => {
                        state.call_stack.push(state.pc);
                        state.pc = target + 1;
                    }
                    Err(_) => {
                        if let Some(op) = builtin_card_op(&label) {
                            crate::ops::dispatch(state, op)?;
                            state.pc += 1;
                        } else {
                            return Err(HpError::InvalidOp);
                        }
                    }
                }
            }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `just test -p hp41-core --test cardreader_xeq_tests`
Expected: All seven tests PASS.

- [ ] **Step 5: Run full core suite**

Run: `just test -p hp41-core`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
/git-workflow:commit --with-skills
```

Suggested message:

```
✨ feat(core): wire XEQ-by-name fallback into run_loop

XEQ "WPRGM" / "RDPRGM" / "WDTA" / "RDTA" steps inside a running
program now stage the corresponding card request instead of
erroring out with InvalidOp.
```

---

## Task 4: Core — Wire fallback into `op_xeq` (interactive dispatch)

**Files:**
- Modify: `hp41-core/src/ops/program.rs:61-69` (the `op_xeq` function)
- Test: `hp41-core/tests/cardreader_xeq_tests.rs` (extend)

The GUI dispatches `Op::Xeq("WPRGM")` through `dispatch()`, which lands in `op_xeq` — and today returns `InvalidOp` interactively. Wire the fallback here so the GUI flow works.

- [ ] **Step 1: Add failing test**

Append to `hp41-core/tests/cardreader_xeq_tests.rs`:

```rust
#[test]
fn op_xeq_interactive_dispatch_stages_card_request() {
    // Mirrors the GUI path: dispatch(Op::Xeq("WPRGM")) with is_running=false.
    use hp41_core::ops::dispatch;
    let mut state = state_with_alpha("QUAD");
    assert!(!state.is_running);
    dispatch(&mut state, Op::Xeq("WPRGM".to_string())).expect("interactive XEQ WPRGM must succeed");
    assert_eq!(
        state.pending_card_op,
        Some(CardOpRequest::WriteProgram { name: "QUAD".to_string() }),
    );
}

#[test]
fn op_xeq_interactive_unknown_name_still_errors() {
    use hp41_core::ops::dispatch;
    let mut state = state_with_alpha("X");
    let err = dispatch(&mut state, Op::Xeq("UNKNOWN_XYZ".to_string())).unwrap_err();
    assert!(
        matches!(err, hp41_core::error::HpError::InvalidOp),
        "interactive XEQ with unknown name must keep returning InvalidOp, got {err:?}",
    );
}
```

- [ ] **Step 2: Run tests, verify they fail**

Run: `just test -p hp41-core --test cardreader_xeq_tests op_xeq_interactive`
Expected: `op_xeq_interactive_dispatch_stages_card_request` FAILS with `InvalidOp`. The unknown-name test PASSES already.

- [ ] **Step 3: Rewrite `op_xeq`**

Replace `op_xeq` in `hp41-core/src/ops/program.rs` (lines 56-69):

```rust
/// XEQ: subroutine call. Enforces 4-level call stack limit (D-14).
/// Interactive XEQ (not running) → InvalidOp; XEQ inside a running program is
/// handled by run_loop directly (not this function). Phase 4 TUI can add
/// interactive subroutine-run support via run_program().
/// LiftEffect: Neutral.
pub fn op_xeq(state: &mut CalcState, _label: &str) -> Result<(), HpError> {
    if !state.is_running {
        return Err(HpError::InvalidOp);
    }
    // run_loop handles Op::Xeq directly (with call-depth check and label search).
    // This arm is only reached if someone calls op_xeq() outside run_loop,
    // which should not happen — return InvalidOp as a safe guard.
    Err(HpError::InvalidOp)
}
```

With:

```rust
/// XEQ: subroutine call. Enforces 4-level call stack limit (D-14).
///
/// Interactive XEQ (not running): tries the four-entry Card Reader
/// XEQ-by-name fallback (Phase 19 spec) before erroring. This is the
/// path the GUI uses — `dispatch(Op::Xeq("WPRGM"))` with `is_running=false`.
///
/// Programmatic XEQ inside `run_loop` is handled there directly (with
/// call-depth check + user-label scan + same builtin fallback) — this
/// function is never reached during program execution.
///
/// LiftEffect: Neutral.
pub fn op_xeq(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    if !state.is_running {
        // Built-in XEQ-by-name fallback for the four Card Reader ops.
        // No user-label scan here: user-program XEQ goes through run_loop,
        // not op_xeq. If a user wants to call their own LBL interactively
        // they use run_program(state, label) directly, not dispatch.
        if let Some(op) = builtin_card_op(label) {
            return crate::ops::dispatch(state, op);
        }
        return Err(HpError::InvalidOp);
    }
    // run_loop handles Op::Xeq directly. Reaching here while running is a
    // logic bug elsewhere — return InvalidOp as a safe guard.
    Err(HpError::InvalidOp)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `just test -p hp41-core --test cardreader_xeq_tests`
Expected: All nine tests PASS.

- [ ] **Step 5: Run full workspace tests**

Run: `just test`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
/git-workflow:commit --with-skills
```

Suggested message:

```
✨ feat(core): wire XEQ-by-name fallback into op_xeq

dispatch(Op::Xeq("WPRGM")) with is_running=false now stages a
WriteProgram request instead of returning InvalidOp. This is
the GUI's interactive XEQ path.
```

---

## Task 5: CLI — Add `dirs` dependency and `cards.rs` skeleton

**Files:**
- Modify: `hp41-cli/Cargo.toml`
- Create: `hp41-cli/src/cards.rs`
- Modify: `hp41-cli/src/lib.rs` or `main.rs` (whichever currently has the `mod` declarations)

- [ ] **Step 1: Add `dirs` to `[dependencies]` in `hp41-cli/Cargo.toml`**

Locate the `[dependencies]` block and add (alphabetical placement preferred):

```toml
dirs = "5"
```

- [ ] **Step 2: Verify the build picks up the new dep**

Run: `just check -p hp41-cli` (or `cargo check -p hp41-cli`)
Expected: `Compiling dirs vX.Y.Z`, `Finished` — no errors.

- [ ] **Step 3: Create the `cards.rs` module with a failing unit test**

First locate the module list. Run: `grep -n '^mod ' hp41-cli/src/main.rs hp41-cli/src/lib.rs 2>/dev/null` — add `mod cards;` near the existing `mod` declarations in whichever file declares them.

Then create `hp41-cli/src/cards.rs`:

```rust
//! Card Reader frontend integration for hp41-cli.
//!
//! Owns:
//! - `cards_dir()` — `~/.hp41/cards/` resolution.
//! - `sanitize_name()` — rejects path separators and dot-paths.
//! - `drain_pending_card_op()` — performs the staged disk I/O after dispatch.
//!
//! SC-4 invariant: this module calls only the public `hp41_core::cardreader::*`
//! API for encoding/decoding. No codec logic lives here.

use std::fs;
use std::path::{Path, PathBuf};

use hp41_core::cardreader::{
    self, capture_data_card, decode_data, decode_program, encode_data, encode_program,
    insert_program_ops, load_data_card, CardOpRequest,
};
use hp41_core::error::HpError;
use hp41_core::state::CalcState;

/// Default cards directory: `~/.hp41/cards/`. Shared with hp41-gui.
///
/// Returns `None` if `dirs::home_dir()` is unavailable (rare; CI / containers
/// with no $HOME). Callers should treat that as a fatal startup error since
/// any card op would fail.
pub fn cards_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".hp41").join("cards"))
}

/// Reject card names that would escape `cards_dir` or otherwise be unsafe.
///
/// Sanitisation, not normalisation: we surface `HpError::CardData` rather
/// than silently mangling the user's input.
pub fn sanitize_name(name: &str) -> Result<&str, HpError> {
    if name.is_empty() {
        // Defensive: the op handlers should have already returned AlphaData,
        // but guard anyway so this helper is safe to call standalone.
        return Err(HpError::AlphaData);
    }
    if name.contains('/') || name.contains('\\') || name.contains('\0') {
        return Err(HpError::CardData(format!("invalid card name: {name:?}")));
    }
    if name == "." || name == ".." {
        return Err(HpError::CardData(format!("invalid card name: {name:?}")));
    }
    Ok(name)
}

/// Drain `state.pending_card_op` and perform the staged disk I/O against `cards_dir`.
///
/// No-op if no request is pending. Errors are surfaced as `HpError::CardData(msg)`
/// (already the contract for codec errors) so the CLI display shows "CARD DATA"
/// with a useful suffix.
///
/// `cards_dir` is a parameter rather than computed inside — keeps integration
/// tests sandboxable via `tempfile::tempdir()`.
pub fn drain_pending_card_op(state: &mut CalcState, cards_dir: &Path) -> Result<(), HpError> {
    let Some(req) = state.pending_card_op.take() else {
        return Ok(());
    };

    fs::create_dir_all(cards_dir)
        .map_err(|e| HpError::CardData(format!("io: cannot create {}: {e}", cards_dir.display())))?;

    match req {
        CardOpRequest::WriteProgram { name } => {
            let safe = sanitize_name(&name)?;
            let mut bytes = Vec::new();
            encode_program(&state.program, &mut bytes)
                .map_err(|e| HpError::CardData(format!("encode: {e}")))?;
            let path = cards_dir.join(safe).with_extension("raw");
            fs::write(&path, &bytes)
                .map_err(|e| HpError::CardData(format!("io: write {}: {e}", path.display())))?;
            Ok(())
        }
        CardOpRequest::WriteData { name } => {
            let safe = sanitize_name(&name)?;
            let card = capture_data_card(state);
            let json = serde_json::to_string_pretty(&card)
                .map_err(|e| HpError::CardData(format!("encode-json: {e}")))?;
            let path = cards_dir.join(safe).with_extension("card.json");
            fs::write(&path, json.as_bytes())
                .map_err(|e| HpError::CardData(format!("io: write {}: {e}", path.display())))?;
            Ok(())
        }
        CardOpRequest::ReadProgram { name } => {
            let safe = sanitize_name(&name)?;
            let path = cards_dir.join(safe).with_extension("raw");
            let bytes = fs::read(&path)
                .map_err(|e| HpError::CardData(format!("io: read {}: {e}", path.display())))?;
            let ops = decode_program(&bytes)
                .map_err(|e| HpError::CardData(format!("decode: {e}")))?;
            insert_program_ops(state, ops);
            Ok(())
        }
        CardOpRequest::ReadData { name } => {
            let safe = sanitize_name(&name)?;
            let path = cards_dir.join(safe).with_extension("card.json");
            let json = fs::read_to_string(&path)
                .map_err(|e| HpError::CardData(format!("io: read {}: {e}", path.display())))?;
            let card = decode_data(&json)
                .map_err(|e| HpError::CardData(format!("decode-json: {e}")))?;
            load_data_card(state, card);
            Ok(())
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_rejects_path_separators() {
        assert!(matches!(sanitize_name("../etc"), Err(HpError::CardData(_))));
        assert!(matches!(sanitize_name("a/b"), Err(HpError::CardData(_))));
        assert!(matches!(sanitize_name("a\\b"), Err(HpError::CardData(_))));
        assert!(matches!(sanitize_name("a\0b"), Err(HpError::CardData(_))));
        assert!(matches!(sanitize_name("."), Err(HpError::CardData(_))));
        assert!(matches!(sanitize_name(".."), Err(HpError::CardData(_))));
        assert_eq!(sanitize_name("QUAD"), Ok("QUAD"));
        assert_eq!(sanitize_name("BACKUP-1"), Ok("BACKUP-1"));
    }

    #[test]
    fn sanitize_empty_yields_alpha_data() {
        assert!(matches!(sanitize_name(""), Err(HpError::AlphaData)));
    }

    #[test]
    fn drain_with_no_request_is_noop() {
        let mut state = CalcState::new();
        let tmp = tempfile::tempdir().unwrap();
        assert!(state.pending_card_op.is_none());
        drain_pending_card_op(&mut state, tmp.path()).unwrap();
        assert!(state.pending_card_op.is_none());
    }
}

// Suppress unused-import warning for `cardreader` re-exports above —
// they're used by integration tests in tests/card_io_tests.rs but not
// by the unit tests in this file.
#[allow(dead_code)]
fn _unused_imports_marker() {
    let _: fn(&[u8]) -> _ = decode_program;
    let _: fn(&str) -> _ = decode_data;
}
```

Note: `encode_program` and `encode_data` already exist per the spec's `cardreader` module description. Verify their exact signatures by checking `hp41-core/src/cardreader/raw.rs::encode_program` (should be `fn encode_program(ops: &[Op], out: &mut Vec<u8>) -> Result<(), HpError>`) and `hp41-core/src/cardreader/data.rs::encode_data` before pasting — adjust call sites if signatures differ.

- [ ] **Step 4: Verify the module compiles and the unit tests pass**

Run: `just test -p hp41-cli cards::tests`
Expected: Three tests PASS (`sanitize_rejects_path_separators`, `sanitize_empty_yields_alpha_data`, `drain_with_no_request_is_noop`).

If the compile fails with "no matching function for call to `encode_program`" etc., adjust the imports/calls to match the actual public API in `hp41-core/src/cardreader/{raw,data,mod}.rs` — the spec wording (e.g. `cardreader::encode_data`) may differ slightly from the implementation. The unit tests do not call those functions, so the basic skeleton can compile even before the encode/decode wiring is exactly right.

- [ ] **Step 5: Commit**

```bash
/git-workflow:commit --with-skills
```

Suggested message:

```
✨ feat(cli): add cards module — dir resolution, sanitize, drain

drain_pending_card_op performs the staged WPRGM/RDPRGM/WDTA/RDTA
disk I/O against a parameterised cards_dir. SC-4: codec calls go
through hp41-core::cardreader only.
```

---

## Task 6: CLI — Wire `drain_pending_card_op` into `app.rs`

**Files:**
- Modify: `hp41-cli/src/app.rs` (multiple sites)

The CLI calls `drain_and_show_print_output()` at three sites after `run_program` succeeds (~lines 283, 447, 947) and inside `call_dispatch_and_drain`. Each of those becomes drain-card-op then drain-print.

- [ ] **Step 1: Survey actual call sites**

Run: `grep -n 'drain_and_show_print_output\|call_dispatch_and_drain' hp41-cli/src/app.rs`

Note every line number. There should be one definition and three call sites in the function bodies. Confirm against the spec's expectation — line numbers may have shifted since the spec was written.

- [ ] **Step 2: Add an `app.cards_dir` field (or compute lazily)**

Pick **one** of these two patterns; the constructor-field pattern is simpler:

In `App::new()` (or wherever `App` is constructed), add a field `cards_dir: PathBuf`:

```rust
pub struct App {
    // ... existing fields ...
    cards_dir: std::path::PathBuf,
}

impl App {
    pub fn new(/* existing args */) -> Self {
        let cards_dir = crate::cards::cards_dir()
            .unwrap_or_else(|| std::path::PathBuf::from(".hp41/cards"));
        Self {
            // ... existing init ...
            cards_dir,
        }
    }
}
```

Confirm the actual `App::new` signature by reading the surrounding lines first — the exact placement depends on the existing constructor shape.

- [ ] **Step 3: Add a `drain_pending_card_op` helper on `App`**

Add a new method on `App` right above `drain_and_show_print_output` (~line 968):

```rust
/// Drain the staged Card Reader request (if any), performing the disk I/O.
///
/// Mirrors `drain_and_show_print_output` — same call sites, surfaces
/// `HpError::CardData(msg)` into `self.message` so the CLI display shows
/// "CARD DATA" with a diagnostic suffix.
fn drain_pending_card_op(&mut self) {
    if let Err(e) = crate::cards::drain_pending_card_op(&mut self.state, &self.cards_dir) {
        self.message = Some(format!("{e}"));
    }
}
```

- [ ] **Step 4: Call the new helper at every dispatch/run_program site**

For each site that today reads:

```rust
match hp41_core::run_program(&mut self.state, &label) {
    Ok(()) => {
        self.message = None;
        self.drain_and_show_print_output();
    }
    Err(e) => self.message = Some(format!("{e}")),
}
```

Change to (drain card *before* print, so a CARD DATA error doesn't get clobbered by a print-summary):

```rust
match hp41_core::run_program(&mut self.state, &label) {
    Ok(()) => {
        self.message = None;
        self.drain_pending_card_op();
        self.drain_and_show_print_output();
    }
    Err(e) => self.message = Some(format!("{e}")),
}
```

Apply at every `run_program` site identified in Step 1.

Also patch `call_dispatch_and_drain` (its definition will be near `drain_and_show_print_output`): add a `self.drain_pending_card_op();` call right before its existing `drain_and_show_print_output();` call.

- [ ] **Step 5: Compile and run the CLI tests**

Run: `just test -p hp41-cli`
Expected: All existing tests PASS. The wiring change is additive (drain card before drain print) and should not affect existing assertions.

- [ ] **Step 6: Commit**

```bash
/git-workflow:commit --with-skills
```

Suggested message:

```
✨ feat(cli): wire pending_card_op drain into app dispatch sites

drain_pending_card_op runs before drain_and_show_print_output at
every dispatch + run_program site so card I/O happens between user
key presses, as documented in architecture.md's staging-drain
contract.
```

---

## Task 7: CLI — Integration test for round-trip + error paths

**Files:**
- Create: `hp41-cli/tests/card_io_tests.rs`
- Modify: `hp41-cli/Cargo.toml` (add `sha2` to `[dev-dependencies]`)

- [ ] **Step 1: Add `sha2` as a dev-dep**

In `hp41-cli/Cargo.toml`, add under `[dev-dependencies]`:

```toml
sha2 = "0.10"
```

Tempfile should already be available (workspace dev-dep) — verify by `grep tempfile Cargo.toml hp41-cli/Cargo.toml`. If not present in either, add `tempfile = "3"` to `hp41-cli/Cargo.toml`'s `[dev-dependencies]`.

- [ ] **Step 2: Create the integration test file**

Create `hp41-cli/tests/card_io_tests.rs`:

```rust
//! End-to-end Card Reader integration tests for hp41-cli.
//! Spec: docs/superpowers/specs/2026-05-13-card-reader-manual-verification-design.md

#![allow(clippy::unwrap_used)]

use std::fs;

use hp41_cli::cards::drain_pending_card_op;
use hp41_core::error::HpError;
use hp41_core::num::HpNum;
use hp41_core::ops::Op;
use hp41_core::state::CalcState;
use hp41_core::{dispatch, run_program};
use sha2::{Digest, Sha256};

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn make_state_with_program() -> CalcState {
    let mut state = CalcState::new();
    state.program = vec![
        Op::Lbl("QUAD".to_string()),
        Op::Const(HpNum::from(5i32)),
        Op::Enter,
        Op::Mul, // simplified: 5 * 5 = 25 — full quadratic program lives in the verification doc
        Op::Rtn,
    ];
    state
}

#[test]
fn roundtrip_program_via_tempdir() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state = make_state_with_program();
    state.alpha = "QUAD".to_string();
    let original_program = state.program.clone();

    // 1. Save: run XEQ "WPRGM" via run_program, then drain.
    run_program(&mut state, "WPRGM").unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    let raw_path = tmp.path().join("QUAD.raw");
    assert!(raw_path.exists(), "QUAD.raw must exist after WPRGM");
    let hash_a = sha256_hex(&fs::read(&raw_path).unwrap());

    // 2. Clear the program.
    state.program.clear();
    state.pc = 0;

    // 3. Load: XEQ "RDPRGM" + drain.
    state.alpha = "QUAD".to_string();
    run_program(&mut state, "RDPRGM").unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    assert_eq!(
        state.program, original_program,
        "program after RDPRGM must equal the original",
    );

    // 4. Re-save and compare hashes — byte-stable round-trip.
    state.alpha = "QUAD".to_string();
    run_program(&mut state, "WPRGM").unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    let hash_b = sha256_hex(&fs::read(&raw_path).unwrap());
    assert_eq!(hash_a, hash_b, "SHA-256 of QUAD.raw must be byte-stable across save→load→save");
}

#[test]
fn roundtrip_data_via_tempdir() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state = CalcState::new();
    state.regs[0] = HpNum::from(42i32);
    state.regs[50] = HpNum::from(314i32); // stand-in for π for hash stability
    state.regs[99] = HpNum::from(-1i32);

    // Save.
    state.alpha = "BACKUP".to_string();
    run_program(&mut state, "WDTA").unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    let path = tmp.path().join("BACKUP.card.json");
    assert!(path.exists());
    let hash_c = sha256_hex(&fs::read(&path).unwrap());

    // Clear.
    for r in &mut state.regs {
        *r = HpNum::zero();
    }
    assert_eq!(state.regs[0], HpNum::zero());

    // Load.
    state.alpha = "BACKUP".to_string();
    run_program(&mut state, "RDTA").unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    assert_eq!(state.regs[0], HpNum::from(42i32));
    assert_eq!(state.regs[50], HpNum::from(314i32));
    assert_eq!(state.regs[99], HpNum::from(-1i32));
    assert!(state.regs.len() >= 100, "load_data_card must keep len ≥ 100");

    // Re-save → hash stability.
    state.alpha = "BACKUP".to_string();
    run_program(&mut state, "WDTA").unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    let hash_d = sha256_hex(&fs::read(&path).unwrap());
    assert_eq!(hash_c, hash_d);
}

#[test]
fn empty_alpha_yields_alpha_data() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state = CalcState::new();
    // state.alpha defaults to "" — empty
    let err = run_program(&mut state, "WPRGM").unwrap_err();
    assert!(
        matches!(err, HpError::AlphaData),
        "empty ALPHA + WPRGM must surface AlphaData, got {err:?}",
    );
    // Nothing staged → drain is a no-op.
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    assert!(!tmp.path().join(".raw").exists());
}

#[test]
fn missing_file_yields_card_data() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state = CalcState::new();
    state.alpha = "NOPE".to_string();
    run_program(&mut state, "RDPRGM").unwrap();
    let err = drain_pending_card_op(&mut state, tmp.path()).unwrap_err();
    assert!(
        matches!(err, HpError::CardData(_)),
        "missing file must yield CardData, got {err:?}",
    );
}

#[test]
fn corrupt_data_json_yields_card_data() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path()).unwrap();
    fs::write(tmp.path().join("BAD.card.json"), b"this is not json").unwrap();
    let mut state = CalcState::new();
    state.alpha = "BAD".to_string();
    run_program(&mut state, "RDTA").unwrap();
    let err = drain_pending_card_op(&mut state, tmp.path()).unwrap_err();
    assert!(matches!(err, HpError::CardData(_)));
}

#[test]
fn dispatch_op_xeq_then_drain_works_for_gui_path() {
    // Mirrors the GUI path: dispatch(Op::Xeq("WPRGM")) instead of run_program.
    let tmp = tempfile::tempdir().unwrap();
    let mut state = make_state_with_program();
    state.alpha = "FROMGUI".to_string();
    dispatch(&mut state, Op::Xeq("WPRGM".to_string())).unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    assert!(tmp.path().join("FROMGUI.raw").exists());
}
```

Note on `Op::Const(HpNum::from(5i32))`: if `Op::Const` isn't the actual variant name for numeric constants, replace with whatever the program-recording code uses for "push a literal" (search: `grep -n 'Const\|PushNum\|Lit' hp41-core/src/ops/mod.rs`). The point is to record a small valid program — the exact ops don't matter as long as encode/decode round-trips them.

- [ ] **Step 3: Run integration tests**

Run: `just test -p hp41-cli --test card_io_tests`
Expected: All six tests PASS.

- [ ] **Step 4: Run full CLI suite (regression check)**

Run: `just test -p hp41-cli`
Expected: All existing tests still PASS.

- [ ] **Step 5: Commit**

```bash
/git-workflow:commit --with-skills
```

Suggested message:

```
🧪 test(cli): card I/O integration tests with SHA-256 round-trip

Six tests in hp41-cli/tests/card_io_tests.rs cover program +
data round-trips against a tempdir, the AlphaData/CardData
error paths, and the GUI-style dispatch(Op::Xeq) entry point.
```

---

## Task 8: CLI — Comfort shortcuts (`Ctrl+W/R/D/F`)

**Files:**
- Modify: `hp41-cli/src/keys.rs`
- Modify: `hp41-cli/src/help_data.rs`

- [ ] **Step 1: Confirm shortcut keys are free**

Run: `grep -n 'KeyCode::Char.*CONTROL\|modifiers.*CONTROL\|KeyModifiers::CONTROL' hp41-cli/src/keys.rs hp41-cli/src/app.rs`

Verify that `Ctrl+W`, `Ctrl+R`, `Ctrl+D`, `Ctrl+F` are unbound. (Confirmed unbound at spec time, but always re-check.) If any clash, fall back to `Alt+W/R/D/F`.

- [ ] **Step 2: Add four `key_to_op` arms in `keys.rs`**

Locate `fn key_to_op(key: KeyEvent, ...) -> Option<Op>`. Find the `Ctrl+...` section (or add a new one). Add:

```rust
// Card Reader comfort shortcuts (Phase 19). Hardware-faithful path is
// ALPHA "name" + XEQ "WPRGM" — these are convenience bindings for
// maintainer testing.
(KeyCode::Char('w'), m) if m.contains(KeyModifiers::CONTROL) => Some(Op::Wprgm),
(KeyCode::Char('r'), m) if m.contains(KeyModifiers::CONTROL) => Some(Op::Rdprgm),
(KeyCode::Char('d'), m) if m.contains(KeyModifiers::CONTROL) => Some(Op::Wdta),
(KeyCode::Char('f'), m) if m.contains(KeyModifiers::CONTROL) => Some(Op::Rdta),
```

The exact match shape will depend on how `key_to_op` currently destructures `KeyEvent` — read the surrounding code and follow the existing pattern.

- [ ] **Step 3: Add four `KEY_REF_TABLE` entries**

Add entries for the four shortcuts in the `KEY_REF_TABLE` array (kebab-case binding string + label). Follow the existing pattern of nearby entries (e.g. `("Ctrl+A", "ALPHA mode")`).

- [ ] **Step 4: Add four `HELP_DATA` entries**

In `hp41-cli/src/help_data.rs`, locate `HELP_DATA` and add four rows:

```rust
HelpEntry::new("Ctrl+W", "WPRGM",  "Write current program to card (uses ALPHA name)"),
HelpEntry::new("Ctrl+R", "RDPRGM", "Read program from card (uses ALPHA name)"),
HelpEntry::new("Ctrl+D", "WDTA",   "Write data registers to card (uses ALPHA name)"),
HelpEntry::new("Ctrl+F", "RDTA",   "Read data registers from card (uses ALPHA name)"),
```

Adjust the constructor / struct literal shape to match the existing `HelpEntry` definition.

- [ ] **Step 5: Smoke-test by adding a key-handling test**

If `hp41-cli/src/keys.rs` already has a `#[cfg(test)]` module, append:

```rust
#[test]
fn ctrl_w_maps_to_wprgm() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let ev = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);
    // Construct an App for context, or pass whatever key_to_op currently needs.
    // If key_to_op takes an &App parameter, build a default App.
    // ... shape matches the existing test conventions in this file ...
    assert_eq!(key_to_op(ev, &dummy_app()), Some(Op::Wprgm));
}
```

Then a parallel one for each of `Ctrl+R/D/F`. If the existing test conventions don't make this easy, skip the smoke test — the integration test in Task 7 already covers the underlying op behavior, and visual verification (Task 11) proves the keys land.

- [ ] **Step 6: Run tests**

Run: `just test -p hp41-cli`
Expected: All tests PASS.

- [ ] **Step 7: Commit**

```bash
/git-workflow:commit --with-skills
```

Suggested message:

```
✨ feat(cli): comfort shortcuts for WPRGM/RDPRGM/WDTA/RDTA

Ctrl+W/R/D/F directly dispatch the four Card Reader ops, with
KEY_REF_TABLE and HELP_DATA entries. Hardware-faithful path
(ALPHA + XEQ) keeps working in parallel.
```

---

## Task 9: GUI — `cards.rs` mirror module

**Files:**
- Create: `hp41-gui/src-tauri/src/cards.rs`
- Modify: `hp41-gui/src-tauri/src/lib.rs` (add `mod cards;`)
- (`dirs` is already a `hp41-gui/src-tauri/Cargo.toml` dependency per CLAUDE.md — verify but no edit needed)

- [ ] **Step 1: Verify `dirs` is present**

Run: `grep -n 'dirs' hp41-gui/src-tauri/Cargo.toml`
Expected: A `dirs = "..."` line under `[dependencies]`. If missing (CLAUDE.md may be stale), add `dirs = "5"`.

- [ ] **Step 2: Create the GUI mirror module**

Create `hp41-gui/src-tauri/src/cards.rs` with the same content as `hp41-cli/src/cards.rs` from Task 5, modulo:
- Module-doc comment refers to "hp41-gui" instead of "hp41-cli".
- The unit-test module is identical.

The implementation is identical because both frontends share the same `~/.hp41/cards/` location and call the same `hp41-core::cardreader` helpers — that's the whole point of the SC-4 invariant. **Do not** factor into a shared crate at this stage; the workspace already has a clean two-crate boundary and a third "frontend-common" crate would be premature.

- [ ] **Step 3: Add `mod cards;` to `hp41-gui/src-tauri/src/lib.rs`**

Locate the existing `mod` declarations (`mod commands;`, `mod key_map;`, etc.) and add `mod cards;` alongside them. If the module needs to be public to be reachable from `commands.rs`, use `pub(crate) mod cards;` — match the pattern of neighbouring modules.

- [ ] **Step 4: Run GUI tests**

Run: `just gui-ci` (or `cd hp41-gui/src-tauri && cargo test`)
Expected: GUI build + tests PASS. New `cards::tests` module runs (three tests, identical to Task 5's).

- [ ] **Step 5: Commit**

```bash
/git-workflow:commit --with-skills
```

Suggested message:

```
✨ feat(gui): add cards module — mirror of hp41-cli helper

Same path resolution, same drain logic, same SC-4 invariant
(codec calls go through hp41-core::cardreader only).
```

---

## Task 10: GUI — Wire drain into Tauri command handlers

**Files:**
- Modify: `hp41-gui/src-tauri/src/commands.rs:36` (`handle_op`) and ~`45` (`handle_get_state`)

- [ ] **Step 1: Locate the handlers**

Read `hp41-gui/src-tauri/src/commands.rs:1-130` to see the exact shape of `handle_op` and `handle_get_state`. The drain has to land **after** `dispatch` and **before** `CalcStateView` is built (so `print_buffer` is drained as-is, but the card request has already been performed).

- [ ] **Step 2: Add a GUI test that fails**

In `hp41-gui/src-tauri/tests/card_io_tests.rs` (create new file):

```rust
//! GUI card-IO integration test — mirrors the CLI round-trip and proves
//! byte-identical .raw output (SC-4 cross-UI guarantee).

#![allow(clippy::unwrap_used)]

use std::fs;

use hp41_core::error::HpError;
use hp41_core::ops::Op;
use hp41_core::state::CalcState;
use hp41_core::{dispatch, run_program};
use hp41_gui_lib::cards::drain_pending_card_op;

#[test]
fn gui_drain_writes_raw_file() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state = CalcState::new();
    state.program = vec![Op::Lbl("X".to_string()), Op::Rtn];
    state.alpha = "TESTGUI".to_string();
    dispatch(&mut state, Op::Xeq("WPRGM".to_string())).unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    assert!(tmp.path().join("TESTGUI.raw").exists());
}
```

The crate name `hp41_gui_lib` is a guess — match whatever `[package].name` in `hp41-gui/src-tauri/Cargo.toml` actually says (it's likely `hp41-gui` or similar; the Rust import name is the dash-to-underscore version).

- [ ] **Step 3: Run test, verify it compiles and either passes (drain isn't wired so the file appears via the test's own drain call — it should PASS already) or fails**

Run: `just gui-ci` or `cd hp41-gui/src-tauri && cargo test --test card_io_tests`
Expected: PASS — the test calls `drain_pending_card_op` directly, which works as soon as Task 9 is in. This test is the **lower bound**: it proves the helper works.

- [ ] **Step 4: Add a second test that requires the wiring in `handle_op`**

This test goes through the public `handle_op` path so it fails until Step 5's wiring is in:

```rust
#[test]
fn handle_op_drains_card_request() {
    // Use the public Tauri command helpers exposed for testing.
    // (handle_op, handle_get_state are in hp41-gui-lib::commands per
    //  commands.rs's module doc.)
    use hp41_gui_lib::commands::handle_op;

    // Override the cards_dir via env-var if the GUI supports it; otherwise
    // this test runs against ~/.hp41/cards/ — which is a test-pollution
    // hazard. If no env override exists, prefer to test the drain helper
    // directly (above) and rely on visual verification (Task 11) to
    // assert the wiring. In that case, delete THIS test and document why.
    //
    // If you go ahead: build a known program, call handle_op("xeq_WPRGM"),
    // then assert the file appeared under cards_dir().

    // (Implementation deferred until env-override is in place — see
    //  Risk R4 in the spec.)
}
```

Decision point: if there's no env-override mechanism in the GUI today, **delete this test** and rely on (a) the helper test above and (b) the manual verification in Task 11. Don't write a test that pollutes the developer's real `~/.hp41/cards/`.

- [ ] **Step 5: Wire `drain_pending_card_op` into `handle_op`**

In `hp41-gui/src-tauri/src/commands.rs`, locate `handle_op`. After the `dispatch` call but before any `CalcStateView` construction, add:

```rust
// Drain any staged Card Reader request before serialising state back to
// the frontend. Surfaces fs/codec errors via the standard GuiError path
// (same conversion as HpError already uses).
if let Some(dir) = crate::cards::cards_dir() {
    if let Err(e) = crate::cards::drain_pending_card_op(calc, &dir) {
        return Err(e.into());
    }
}
```

If `cards_dir()` returns `None` we silently skip — same fail-soft as the CLI default — but in practice this never fires on macOS/Linux/Windows where `dirs::home_dir()` is reliable.

Add the same block to `handle_get_state` (defensive — should never have a pending request, but the cost is one `Option::is_none` check).

- [ ] **Step 6: Run GUI tests**

Run: `just gui-ci`
Expected: All tests PASS.

- [ ] **Step 7: Commit**

```bash
/git-workflow:commit --with-skills
```

Suggested message:

```
✨ feat(gui): drain pending_card_op in handle_op / handle_get_state

WPRGM/RDPRGM/WDTA/RDTA dispatched via Op::Xeq now perform their
disk I/O before the next CalcStateView reaches the frontend.
```

---

## Task 11: User-facing verification document

**Files:**
- Create: `docs/verifying-card-reader.md`
- Modify: `docs/operations-reference.md` (add See Also pointer)
- Modify: `docs/programming-guide.md` (add See Also pointer)

- [ ] **Step 1: Write `docs/verifying-card-reader.md`**

Create the file with the structure mandated by Spec §4. The skeleton:

```markdown
# Verifying the Card Reader

This procedure walks an operator through a complete Card Reader
round-trip on both `hp41-cli` and `hp41-gui` to confirm the feature
behaves identically across UIs and that card files are byte-stable
across save → clear → load → save cycles.

It exercises both program cards (`WPRGM`/`RDPRGM`) and data cards
(`WDTA`/`RDTA`), all three known error paths, and the SHA-256
round-trip invariant.

## TL;DR

| Step | CLI keys | GUI clicks | Expected |
|------|---------|-----------|----------|
| Save program | `ALPHA QUAD ALPHA XEQ WPRGM ENTER` | identical | `~/.hp41/cards/QUAD.raw` exists, ≈30–40 B |
| Clear program | `PRGM CLP` | identical | listing shows `00 END.` only |
| Load program | `ALPHA QUAD ALPHA XEQ RDPRGM ENTER` | identical | 22 lines identical to original |
| Run program | `XEQ QUAD ENTER` | identical | `X = 3.`, `R02 = 3.`, `R03 = 2.` |
| Round-trip hash | `sha256sum QUAD.raw` (terminal) | (terminal) | hash stable across re-saves |

## 1. Preparation

[per Spec §4.1 — reset autosave + cards dir, launch CLI/GUI]

## 2. Enter and verify the program

[full 22-line keypress table for the quadratic-formula program, with
expected display state after each STO/RCL, ending with the reference
run `XEQ "QUAD" + ENTER` and expected X / R01 / R02 / R03]

## 3. Program card: WPRGM → Clear → RDPRGM

[10 numbered steps per Spec §4.3]

## 4. Data card: WDTA → Clear → RDTA

[10 numbered steps per Spec §4.4]

## 5. Error paths

[three F-tests per Spec §4.5]

## 6. Same procedure in the GUI

[mirror of §3 and §4 with ALPHA via physical-keyboard pass-through;
sha256sum steps stay in the terminal; the hashes from CLI and GUI
must match — that's the SC-4 cross-UI proof]

## Known limitations

- A program containing two card ops in sequence will fail the second
  with `CARD DATA ("pending")` — drains run between operator
  key-presses, not inside `run_loop`. Document this so it doesn't
  surprise users.
- SHA-256 stability requires the `DataCard` struct's field order to
  be unchanged across the two saves. A codec version bump invalidates
  the cached hash.

## See Also

- [Operations Reference — Card Reader (HP 82104A)](operations-reference.md#card-reader-hp-82104a)
- [Programming Guide — Saving and Loading via Card Reader](programming-guide.md#saving-and-loading-via-card-reader)
```

Flesh out the placeholder bracketed sections from Spec §4 — paste the exact keypresses and expected outputs there. No new content, just the spec moved into a published doc.

- [ ] **Step 2: Add pointers from existing docs**

In `docs/operations-reference.md`, find the `See Also` section (line ~324) and add as the **first** entry:

```markdown
- [Verifying the Card Reader](verifying-card-reader.md) — step-by-step manual round-trip procedure for CLI and GUI
```

Same in `docs/programming-guide.md`'s See Also section.

- [ ] **Step 3: Verify the doc renders**

Run: `grep -l 'verifying-card-reader' docs/*.md`
Expected: at least three files match (the new doc itself, operations-reference.md, programming-guide.md).

- [ ] **Step 4: Commit**

```bash
/git-workflow:commit --with-skills
```

Suggested message:

```
📚 docs: user-facing Card Reader manual verification procedure

docs/verifying-card-reader.md walks operators through the
quadratic-formula round-trip with expected state at every step,
covers both program and data cards, lists the three error paths,
and mirrors the procedure for the GUI. operations-reference.md
and programming-guide.md gain See Also pointers.
```

---

## Task 12: Smoke-test the full procedure manually

The first nine tasks are TDD-driven; this one is the deliverable that the spec was written for: an operator runs the published procedure end-to-end and confirms it works.

- [ ] **Step 1: Build both UIs cleanly**

Run: `just ci` and `just gui-ci`.
Expected: both PASS — clippy clean, all tests green, coverage gate intact on `hp41-core`.

- [ ] **Step 2: Reset state**

```bash
rm -f ~/.hp41/autosave.json
rm -rf ~/.hp41/cards/
```

- [ ] **Step 3: Run `docs/verifying-card-reader.md` Sections 1–5 in the CLI**

```bash
just run-cli   # or your usual CLI launch recipe
```

Follow each numbered step exactly. Record any deviation between expected and observed display state — those are bugs.

- [ ] **Step 4: Capture both hashes**

```bash
sha256sum ~/.hp41/cards/QUAD.raw ~/.hp41/cards/BACKUP.card.json
```

Note: hash values are environment-dependent (program contents, register values); the test is **stability across two saves**, not a fixed reference value.

- [ ] **Step 5: Run Section 6 in the GUI**

```bash
just gui-dev
```

Repeat the program and data steps. Recompute hashes:

```bash
sha256sum ~/.hp41/cards/QUAD.raw ~/.hp41/cards/BACKUP.card.json
```

The hashes must match the CLI's — same codec, same input, same bytes out.

- [ ] **Step 6: If everything matches, no commit needed**

This step is verification, not code. If you find a bug, file an issue or fix it in a follow-up commit referencing this plan.

---

## Self-Review Summary

**Spec coverage** — every checklist item in Spec §"Deliverables Checklist" is addressed:

| Spec deliverable | Plan task |
|------------------|-----------|
| `op_xeq` fallback | Task 4 |
| `run_loop` fallback | Task 3 |
| `run_program` fallback (added during planning, missing from spec) | Task 2 |
| `cardreader_xeq_tests.rs` | Tasks 2–4 (incremental) |
| `hp41-cli/src/cards.rs` | Task 5 |
| `app.rs` drain wiring | Task 6 |
| `keys.rs` comfort shortcuts | Task 8 |
| `help_data.rs` entries | Task 8 |
| `hp41-cli/Cargo.toml` deps | Tasks 5 + 7 |
| `hp41-cli/tests/card_io_tests.rs` | Task 7 |
| `hp41-gui/src-tauri/src/cards.rs` | Task 9 |
| `commands.rs` drain | Task 10 |
| `hp41-gui/src-tauri/tests/card_io_tests.rs` | Task 10 |
| `docs/verifying-card-reader.md` | Task 11 |
| `docs/operations-reference.md` pointer | Task 11 |
| `docs/programming-guide.md` pointer | Task 11 |

**Spec gap surfaced during planning:** The spec described two XEQ fallback sites (`op_xeq` and `run_loop`); code inspection showed the CLI calls `run_program(state, label)` directly for interactive XEQ, so a **third** site is needed. Task 2 covers it.

**Risks reflected from spec §6:**

- R1 (multiple card ops in one program run) — Task 11 documents in "Known limitations".
- R2 (GUI ALPHA entry) — Task 11 §6 assumes physical-keyboard ALPHA passthrough works; Task 12 §5 is where it gets empirically verified.
- R3 (shortcut conflicts) — Task 8 §1 explicit re-check.
- R4 (CI runner pollution) — Tasks 7 and 10 inject `tempdir()`; Task 10 §4 explicitly declines to write a test that touches `~/.hp41/cards/`.
- R5 (fallback in all dispatch paths) — Tasks 2–4 cover all three sites.
- R6 (JSON hash stability) — Task 11 "Known limitations" notes it.
- R7 (dep budget) — Tasks 5 (`dirs`) and 7 (`sha2` dev-only).
