# Card Reader Manual Verification — Design

**Date:** 2026-05-13
**Phase:** 19 (Wiring & Verification)
**Status:** Approved (brainstorming)
**Author:** Daniel Senften

## Context

Phase 19 landed the Card Reader codecs and four `Op` variants
(`Wdta`, `Rdta`, `Wprgm`, `Rdprgm`) into `hp41-core`. The core engine
is complete and unit-tested: ops stage a `CardOpRequest` on
`state.pending_card_op`, codecs round-trip `.raw` and `.card.json`
files, and `cardreader_tests.rs` covers the codec contract.

**The four ops are not yet reachable from either UI.** Concrete gaps
established by code inspection on 2026-05-13:

- `hp41-cli/src/keys.rs::key_to_op()` — no key binding produces
  `Op::Wdta` / `Op::Rdta` / `Op::Wprgm` / `Op::Rdprgm`.
- `hp41-cli/src/app.rs` — no code path drains `pending_card_op`; even
  if the op were dispatched, the staged request would never reach disk.
- `hp41-gui/src-tauri/src/key_map.rs::resolve()` — no string ID maps
  to these ops.
- `hp41-gui/src-tauri/src/commands.rs` — no drain logic.
- `hp41-core/src/ops/program.rs::op_xeq()` resolves user `LBL`s only;
  there is no fallback to built-in op names, so `XEQ "WPRGM"` does
  not work either.

This spec closes those gaps and adds a published step-by-step manual
verification procedure so an end user (and any future regression
audit) can prove the round-trip works.

## Goal

Make the Card Reader feature reachable, drainable, and provably
correct from **both** UIs, in a single bundled phase. Deliver a
user-facing verification document that walks an operator through:

1. Enter a known program on the calculator.
2. Save it to a named card (`WPRGM`).
3. Clear the program.
4. Restore it from the card (`RDPRGM`).
5. Run it again and verify identical results.
6. Repeat with a data card (`WDTA` / `RDTA`).
7. Verify byte-stable round-trip via SHA-256.

## Scope Decisions (from brainstorming)

| Decision | Choice |
|----------|--------|
| UIs | Both `hp41-cli` and `hp41-gui`, CLI first, GUI right after |
| Trigger mechanism | Hardware-faithful `XEQ "name"` fallback for the four card ops + optional CLI comfort shortcuts |
| XEQ-fallback scope | Focused: 4-entry table for the card ops only — **not** a generic built-in dispatcher |
| Card storage | `~/.hp41/cards/<name>.{raw,card.json}` — shared by CLI and GUI |
| Demo program | Quadratic formula for `x² − 5x + 6 = 0` (roots 3 and 2) |
| Card scope | Both program (`WPRGM`/`RDPRGM`) **and** data (`WDTA`/`RDTA`) round-trips |
| Verification rigor | Behavioural identity + visual listing match + SHA-256 byte round-trip |

## Out of Scope

- A generic XEQ-by-name dispatcher for other ROM/built-in functions
  (`SIZE nn`, `FIX nn`, Time-Module ops). Deferred to its own phase.
- Hardware-faithful magnetic-card *visual* in the GUI.
- USER-mode `ASIGN` of a card-reader function to a key.
- Multi-slot / multi-card management. One card == one file, full stop.

## Architecture

### End-to-End Op Flow (CLI)

```
Operator:  ALPHA "QUAD" ALPHA  XEQ  W P R G M  ENTER
              │
              ▼
   key_to_op() / pending_input  →  Op::Xeq("WPRGM")
              │
              ▼
   dispatch(Op::Xeq(name))  →  op_xeq(state, name)
              │  1. find_label(state.program, name)  →  None
              │  2. builtin_card_op(name)            →  Some(Op::Wprgm)
              │  3. re-dispatch on Op::Wprgm
              ▼
   op_wprgm(state)
              │  - state.alpha empty?         → Err(HpError::AlphaData)
              │  - pending_card_op.is_some()? → Err(HpError::CardData("pending"))
              │  - state.pending_card_op = Some(CardOpRequest::WriteProgram{name})
              ▼
   call_dispatch_and_drain(app)
              │  - dispatch result reaches the display
              │  - drain_pending_card_op(state, cards_dir)            ◄── NEW
              │  - drain_and_show_print_output(...)                   (existing)
              ▼
   drain_pending_card_op
              - take(state.pending_card_op)
              - WriteProgram{name}:  encode_program → fs::write(cards_dir/name.raw)
              - WriteData   {name}:  capture_data_card → serde_json → fs::write
              - ReadProgram {name}:  fs::read → decode_program → insert_program_ops
              - ReadData    {name}:  fs::read → decode_data    → load_data_card
              - any fs error / codec error → surface as HpError::CardData(msg)
                so the display shows "CARD DATA"
```

The GUI path is structurally identical: `commands.rs::handle_op`
calls `dispatch` and then `drain_pending_card_op` on the same shared
helper before serialising `CalcStateView` back to the frontend.

### XEQ-by-name Fallback

A four-entry helper in `hp41-core/src/ops/program.rs`:

```rust
fn builtin_card_op(name: &str) -> Option<Op> {
    match name {
        "WPRGM"  => Some(Op::Wprgm),
        "RDPRGM" => Some(Op::Rdprgm),
        "WDTA"   => Some(Op::Wdta),
        "RDTA"   => Some(Op::Rdta),
        _ => None,
    }
}
```

Wired into both XEQ pathways:

- `op_xeq(state, name)` — interactive dispatch (operator typed
  `XEQ "WPRGM"` outside a running program).
- The `Op::Xeq(label) => …` arm of `run_loop()` — a running program
  that contains `XEQ "WPRGM"` as a step.

**User-label precedence.** The fallback fires **only on label miss**.
A user `LBL "WPRGM"` in the operator's program always wins, matching
the real HP-41's `XEQ "name"` resolution order.

### Path Resolution

Centralised in a new `cards.rs` module per frontend:

```rust
fn cards_dir() -> PathBuf {
    dirs::home_dir().expect("home dir").join(".hp41").join("cards")
}

fn sanitize_name(name: &str) -> Result<&str, HpError> {
    if name.is_empty() {
        return Err(HpError::AlphaData);
    }
    if name.contains(['/', '\\', '\0']) || name == "." || name == ".." {
        return Err(HpError::CardData(format!("invalid card name: {name:?}")));
    }
    Ok(name)
}
```

`drain_pending_card_op` takes `cards_dir: &Path` as a parameter — no
global singleton. Production callers pass `cards_dir()`; tests pass
`tempfile::tempdir().path()`.

### SC-4 Invariant (no core duplication)

Both `hp41-cli/src/cards.rs` and `hp41-gui/src-tauri/src/cards.rs`
call **only** the public `cardreader::*` helpers from `hp41-core`
(`encode_program`, `decode_program`, `encode_data`, `decode_data`,
`capture_data_card`, `load_data_card`, `insert_program_ops`). No
codec logic in the UI layers.

### Code Touchpoints

| File | Change |
|------|--------|
| `hp41-core/src/ops/program.rs::op_xeq` | Label-miss → `builtin_card_op` fallback |
| `hp41-core/src/ops/program.rs::run_loop` | Same fallback in the `Op::Xeq(label)` arm |
| `hp41-cli/src/cards.rs` *(new)* | `cards_dir()`, `sanitize_name()`, `drain_pending_card_op()` |
| `hp41-cli/src/app.rs::call_dispatch_and_drain` | Call drain before `drain_and_show_print_output` |
| `hp41-cli/src/app.rs::run_program*` sites | Same drain after `run_program()` returns |
| `hp41-cli/src/keys.rs` | Four comfort shortcuts (see Risk 6.3 for binding choice) |
| `hp41-cli/src/help_data.rs::HELP_DATA` | Entries for the four shortcuts |
| `hp41-cli/Cargo.toml` | Add `dirs` to `[dependencies]`, `sha2` to `[dev-dependencies]` |
| `hp41-gui/src-tauri/src/cards.rs` *(new)* | Mirror drain helper for GUI |
| `hp41-gui/src-tauri/src/commands.rs::handle_op` | Drain right after `dispatch` |
| `hp41-gui/src-tauri/src/commands.rs::handle_get_state` | Drain (idempotent — almost always a no-op here) |

## Demo Program

**`LBL "QUAD"` — quadratic-formula solver hardcoded for `x² − 5x + 6 = 0`.**

```
01 LBL "QUAD"          ; alpha-label path  →  raw codec: F4 51 55 41 44
02   5         ENTER
03   ENTER
04   *                 ; X = 25
05   4         ENTER
06   1         *
07   6         *
08   −                 ; discriminant = 1
09   SQRT
10   STO 01            ; two-digit register prefix  →  E1 01
11   5         ENTER
12   RCL 01            ; E0 01
13   +
14   2         /       ; x₁ = 3
15   STO 02
16   5         ENTER
17   RCL 01
18   −
19   2         /       ; x₂ = 2
20   STO 03
21   RCL 02            ; X = x₁ for display
22 END
```

**Expected end-state after `XEQ "QUAD" + ENTER`:**

| Slot | Value |
|------|-------|
| X (display) | `3.` |
| R01 | `1.` |
| R02 | `3.` |
| R03 | `2.` |

**Why this program.** It triggers every non-trivial codec path: alpha
labels (`F<len>` prefix), two-digit `STO nn` / `RCL nn` (`0xE1` /
`0xE0` prefix collisions tested), constant entry via `flush_entry_buf`,
a representative spread of single-byte FOCAL ops, and the appended
`END` marker (`C0 00 0D`).

**Data setup for the WDTA/RDTA half** (run after the program completes):

```
π    STO 50      ; R50 := 3.141592653...
1 CHS STO 99     ; R99 := -1   (boundary: highest valid register index)
```

R00–R03 already carry the values from the program run. Combined, the
data card exercises: small positive integer, small negative integer,
irrational floating-point (mantissa test), and full `0..=99` coverage.

## Verification Procedure (`docs/verifying-card-reader.md`)

Six sections; each step lists the keypress and the expected display
or filesystem state so that drift surfaces on the next line, not three
steps later.

### 1. Preparation

```
$ rm -f ~/.hp41/autosave.json
$ rm -rf ~/.hp41/cards/
$ hp41             # or: just gui-dev
[Operator: Ctrl+G (CLREG) — fresh state; program memory is empty
 by default after the autosave reset above]
```

### 2. Enter and Verify the Program

22 numbered keypresses (the program in the previous section). After
the program is entered, the operator runs a **reference run**:

```
XEQ "QUAD" + ENTER
Expected:
  X:   3.
  R01: 1.
  R02: 3.
  R03: 2.
```

This is the reference run against which the post-restore run is
compared.

### 3. Program Card: WPRGM → Clear → RDPRGM

```
1.  ALPHA   Q U A D   ALPHA            ; ALPHA = "QUAD"
2.  XEQ "WPRGM" + ENTER                ; → ~/.hp41/cards/QUAD.raw exists (~30–40 B)
3.  $ sha256sum ~/.hp41/cards/QUAD.raw → hash A
4.  PRGM mode → CLP → confirm          ; listing shows only "00 END."
5.  ALPHA   Q U A D   ALPHA
6.  XEQ "RDPRGM" + ENTER               ; listing identical to original (22 lines)
7.  XEQ "QUAD" + ENTER                 ; X=3., R01=1., R02=3., R03=2.  ← behavioural identity
8.  XEQ "WPRGM" + ENTER                ; QUAD.raw overwritten
9.  $ sha256sum ~/.hp41/cards/QUAD.raw → hash B
10. ASSERT hash A == hash B            ; byte-stable round-trip
```

### 4. Data Card: WDTA → Clear → RDTA

```
1.  [data setup above:  π STO 50  /  1 CHS STO 99]
2.  ALPHA   B A C K U P   ALPHA
3.  XEQ "WDTA" + ENTER                 ; ~/.hp41/cards/BACKUP.card.json exists
                                       ; format = "hp41-data-v1", registers.len() >= 100
4.  $ sha256sum BACKUP.card.json       → hash C
5.  Ctrl+G (CLREG)                     ; R00 = R50 = R99 = 0
6.  ALPHA   B A C K U P   ALPHA
7.  XEQ "RDTA" + ENTER                 ; R00..R03 restored, R50 = π, R99 = -1
8.  XEQ "WDTA" + ENTER                 ; BACKUP.card.json overwritten
9.  $ sha256sum BACKUP.card.json       → hash D
10. ASSERT hash C == hash D
```

### 5. Error Paths

```
F1.  ALPHA empty + XEQ "WPRGM"          → "ALPHA DATA"
F2.  ALPHA "NOPE" + XEQ "RDPRGM"        → "CARD DATA"   (file missing)
F3.  $ echo 'kaputt' > ~/.hp41/cards/BAD.card.json
     ALPHA "BAD" + XEQ "RDTA"           → "CARD DATA"   (bad JSON / wrong tag)
```

### 6. Same Procedure in the GUI

Mirror of sections 3 and 4 but the ALPHA entry is done via the GUI's
SVG keyboard (or its physical-keyboard pass-through, see Risk 6.2),
and the `sha256sum` steps remain terminal commands.

**Cross-UI guarantee:** hashes A and C must be **identical** between
CLI and GUI runs. This is the empirical SC-4 proof.

## Automated Tests

### Core: `hp41-core/tests/cardreader_xeq_tests.rs` (new)

- `xeq_wprgm_resolves_to_wprgm_when_no_label_matches` — `Op::Xeq("WPRGM")`
  with no matching `LBL` stages a `WriteProgram` request.
- `xeq_wprgm_prefers_user_label_over_builtin` — a user `LBL "WPRGM"`
  shadows the built-in fallback. Guards hardware-compat.
- `xeq_unknown_name_yields_existing_error` — `Op::Xeq("XYZ")` continues
  to surface today's "label not found" error; the fallback escalates
  nothing new.
- Parametrised across `WPRGM` / `RDPRGM` / `WDTA` / `RDTA`.

### CLI: `hp41-cli/tests/card_io_tests.rs` (new)

- `roundtrip_program_via_tempdir` — write program with `tempdir` as
  `cards_dir`, clear, read back, compare program listing **and**
  SHA-256 of the `.raw` file across two consecutive writes.
- `roundtrip_data_via_tempdir` — analogous for data cards; verifies
  R00, R50, R99 and hash stability.
- `empty_alpha_yields_alpha_data` — path F1.
- `missing_file_yields_card_data` — path F2.
- `corrupt_data_json_yields_card_data` — path F3.
- `sanitize_rejects_path_separators` — `ALPHA "../etc/passwd"` →
  `HpError::CardData(...)`, no FS access outside `cards_dir`.

### GUI: `hp41-gui/src-tauri/tests/card_io_tests.rs` (new)

- One CLI test mirrored against the GUI drain helper, proving
  byte-identical `.raw` output between UIs. Catches SC-4 drift in CI
  (`ci-gui.yml`).

### Local verification

```bash
just test       # core + CLI, includes the new card-IO tests
just gui-ci    # GUI tests
sha256sum ~/.hp41/cards/*.raw ~/.hp41/cards/*.card.json   # manual audit hook
```

No new `just` recipe — the card-IO tests run inside the existing
pipelines.

## Risks, Caveats & Assumptions

### R1 — Multiple card ops in one program run

`pending_card_op` is drained between operator key-presses (CLI) or
Tauri calls (GUI), **not** inside `run_loop()`. A program that
executes `WPRGM` followed by another card op without returning to
the operator will fail the second op with `CARD DATA ("pending")`.
This matches the documented behaviour in `architecture.md` and is
acceptable for v1. Verification doc carries a note.

### R2 — GUI ALPHA entry of card names

The HP-41C's ALPHA-mode letter input on the SVG keyboard is sparse.
Today the GUI accepts ALPHA characters from the **physical keyboard**
via `resolveKeyId()` in `App.tsx`. Assumption: this is enough to type
`"QUAD"` and `"BACKUP"`. **Implementation step:** verify physical
ALPHA typing works in the GUI before opening the PR. If it doesn't,
the GUI half of the verification procedure is postponed until ALPHA
SVG buttons exist.

### R3 — Comfort-shortcut collision risk

Proposed CLI bindings: `Ctrl+W` (WPRGM), `Ctrl+R` (RDPRGM),
`Ctrl+D` (WDTA), `Ctrl+F` (RDTA). Known conflicts to avoid:

- `Ctrl+C` — Quit (CLAUDE.md, Phase 8).
- `Ctrl+A` — ALPHA mode.

At implementation time, grep for `KeyCode::Char.*CONTROL` /
`modifiers.*CONTROL` in `keys.rs` and confirm the proposed set is
free. If any clash, fall back to `Alt+W/R/D/F`.

### R4 — `~/.hp41/cards/` on CI runners

Integration tests must **never** write into the operator's
`~/.hp41/cards/`. `drain_pending_card_op(state, cards_dir: &Path)`
takes the directory as a parameter; tests inject
`tempfile::tempdir()`. CLI/GUI compute the default once at startup.
No global singleton.

### R5 — Built-in XEQ fallback in both dispatch paths

The fallback must hang in **both** `op_xeq` (interactive) and the
`Op::Xeq(label)` arm of `run_loop()` (programmatic). Missing either
breaks the user-facing flow. Test 5.1 covers both.

### R6 — JSON pretty-print hash stability

`serde_json::to_string_pretty` is deterministic **only** while the
`DataCard` struct's field ordering is stable. A future refactor that
reorders fields changes the hash without changing semantics. The
verification doc notes: hash comparison from section 4 is valid only
across code versions where `DataCard` hasn't been altered. Codec
version bumps require updating this spec.

### R7 — Dependency budget

- `hp41-cli/Cargo.toml`: add `dirs` to `[dependencies]`, `sha2` to
  `[dev-dependencies]`. **Not** `sha2` in `[dependencies]` — it isn't
  needed at runtime.
- `hp41-gui/src-tauri/Cargo.toml`: `dirs` is already present (per
  CLAUDE.md). No new dep.
- Existing `cardreader_tests.rs` in `hp41-core` is **untouched** —
  the new tests in this phase complement it, never replace.

## Deliverables Checklist

- [ ] `hp41-core/src/ops/program.rs::op_xeq` — built-in fallback.
- [ ] `hp41-core/src/ops/program.rs::run_loop` — built-in fallback in
      the `Op::Xeq(label)` arm.
- [ ] `hp41-core/tests/cardreader_xeq_tests.rs` — XEQ fallback tests.
- [ ] `hp41-cli/src/cards.rs` — new module: `cards_dir`,
      `sanitize_name`, `drain_pending_card_op`.
- [ ] `hp41-cli/src/app.rs` — drain wired into `call_dispatch_and_drain`
      and every `run_program*` call site.
- [ ] `hp41-cli/src/keys.rs` — four comfort shortcuts (conflict-checked).
- [ ] `hp41-cli/src/help_data.rs` — `HELP_DATA` entries.
- [ ] `hp41-cli/Cargo.toml` — `dirs` + `sha2` (dev).
- [ ] `hp41-cli/tests/card_io_tests.rs` — CLI round-trip + error-path tests.
- [ ] `hp41-gui/src-tauri/src/cards.rs` — mirror drain module.
- [ ] `hp41-gui/src-tauri/src/commands.rs` — drain in `handle_op` and
      `handle_get_state`.
- [ ] `hp41-gui/src-tauri/tests/card_io_tests.rs` — GUI mirror test.
- [ ] `docs/verifying-card-reader.md` — end-user verification procedure.
- [ ] `docs/operations-reference.md` — pointer to the new verification doc.
- [ ] `docs/programming-guide.md` — pointer to the new verification doc.

## Approval

Brainstormed and approved on 2026-05-13. The remaining steps are
spec self-review, user review of this file, and creation of the
implementation plan via `superpowers:writing-plans`.
