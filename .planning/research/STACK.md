# Technology Stack

**Project:** HP-41 Calculator Emulator — v1.1 Stack Delta
**Researched:** 2026-05-08
**Scope:** NEW dependencies or version changes needed for v1.1 features only. The existing v1.0 stack is validated and unchanged.

---

## Verdict: No New Runtime Dependencies Required

All four v1.1 features can be built on the existing stack. The analysis below explains why for each feature, then documents the one version bump worth applying.

---

## Feature-by-Feature Analysis

### Feature 1: STO Arithmetic Keyboard Modals

**Question:** Does the multi-step modal flow (STO+ → register number prompt → dispatch) need a new crate?

**No.** The infrastructure is already in place:

- `op_sto_arith()` in `hp41-core/src/ops/registers.rs` is fully implemented and tested.
- `PendingInput::StoAdd/StoSub/StoMul/StoDiv(String)` variants are already declared in `hp41-cli/src/app.rs` (lines 29-34), gated with `#[allow(dead_code)]`. They were stubbed in v1.0 Phase 8 with an explicit comment: "Keyboard binding deferred to v1.1."
- The existing `PendingInput::StoRegister` flow (two-digit accumulation → dispatch) is the exact same pattern. The STO arithmetic variants extend it with one extra step: select the operation (triggered by `+/-/×/÷` after pressing `STO`), then accumulate the register number.
- The modal UI is built from ratatui 0.30's `Clear` + `Block` + `Paragraph` widgets — all present in the current dependency. `Clear` exists precisely for this use case: render it before the popup to erase the underlying display region.

**Confidence: HIGH** — verified in codebase and ratatui 0.30 docs (docs.rs/ratatui/0.30.0).

---

### Feature 2: EEX Trailing-e-Without-Exponent Hardware Lock

**Question:** Does locking input when `entry_buf` ends with `'e'` and no exponent has been typed require a new crate?

**No.** This is pure state machine logic in `app.rs::handle_key()`.

Current behavior: when `'e'` is in `entry_buf` and a non-digit key arrives, `dispatch()` calls `flush_entry_buf()`, which tries `Decimal::from_str("1.5e")` then `Decimal::from_scientific("1.5e")`, both fail, and `HpError::InvalidOp` is returned — the number is discarded. The HP-41 hardware instead locks: the display shows "1.5  00" (mantissa + blank exponent field) and only accepts digits (for the exponent), `CHS` (to negate the exponent), or `EEX` again (to cancel the EEX entry). No non-EEX op fires until the exponent is complete or EEX is cancelled.

**Implementation:** Add an `eex_locked: bool` field to `CalcState` (or detect the condition inline from `entry_buf.contains('e') && last char is 'e'`), and in `handle_key()` before the normal dispatch path, check that condition and restrict which key codes are accepted. No new crates. The `entry_buf` `String` type already stores the state.

**Confidence: HIGH** — analysis based on direct reading of `ops/mod.rs:flush_entry_buf()` and `app.rs` handle_key() digit-entry block.

---

### Feature 3: Print Emulation (PRX / PRA / PRSTK)

**Question:** Do print-to-console and print-to-file require new I/O crates?

**No.** Rust's `std::io` and `std::fs` cover everything needed.

The HP-41 printer (HP 82143A) outputs ASCII text — no binary protocol, no graphics. The three operations:

- `PRX` — print X register formatted as the current display string
- `PRA` — print the ALPHA register contents
- `PRSTK` — print all four stack registers (T, Z, Y, X) as formatted strings

Print targets:
1. Console (stdout): `std::io::stdout()` with `writeln!()` — zero dependencies.
2. File: `std::fs::OpenOptions::new().append(true).open(path)` + `std::io::BufWriter` — zero dependencies. Path can be set via a `--print-log` clap flag.

**Design recommendation:** Add a `PrintSink` enum to `hp41-cli` (not `hp41-core`):

```rust
pub enum PrintSink {
    Stdout,
    File(std::io::BufWriter<std::fs::File>),
}
impl std::io::Write for PrintSink { ... }
```

The print operations themselves live in `hp41-core` as `Op::PrintX`, `Op::PrintAlpha`, `Op::PrintStack` — they format the string and return it as `Ok(String)` (or via a trait object). The `hp41-cli` layer writes it to the sink. This preserves the `hp41-core` zero-UI-dependency invariant.

**No new crates.** `std::io::Write` + `std::io::BufWriter` + `std::fs::File` + existing `clap` for the `--print-log` flag.

**Confidence: HIGH** — stdlib-only solution, no ecosystem dependency needed.

---

### Feature 4: Synthetic Programming (Byte-Code Injection, FOCAL Internals)

**Question:** Does a FOCAL byte-code lookup table require a new crate (e.g., `phf` for perfect hash maps)?

**No.** A `match` on `u8` in Rust compiles to a jump table — identical performance to a perfect hash map for a 256-entry lookup. `phf` adds a proc-macro build dependency for zero runtime benefit here.

The synthetic programming feature maps FOCAL byte values (`0x00`–`0xFF`) to `Op` variants (or to raw byte sequences for byte-code injection). This is a 256-entry static mapping:

```rust
pub fn focal_byte_to_op(byte: u8) -> Option<Op> {
    match byte {
        0x40 => Some(Op::Add),
        0x41 => Some(Op::Sub),
        // ...
        _ => None, // unimplemented / reserved
    }
}
```

The FOCAL byte-code table itself is research/domain data (HP-41 FOCAL internals documentation), not a Rust crate. The byte-code injection mechanism extends `CalcState::program: Vec<Op>` — the existing data structure. If raw byte sequences need to be stored (for roundtrip fidelity of synthetic instructions that have no `Op` equivalent), a new `Op::RawByte(u8)` variant handles it.

**No new crates.** The `Op` enum extension + `match`-based lookup is purely in `hp41-core`.

**Confidence: HIGH** — verified by reading the existing `Op` enum and `dispatch()` structure.

---

## Version Bump: rust_decimal 1.41 → 1.42

| Crate | Current | Latest | MSRV | Bump? | Rationale |
|-------|---------|--------|------|-------|-----------|
| `rust_decimal` | 1.41 | 1.42.0 | 1.67.1 | YES | Minor version; released 2026-05-06. Patch bug fixes. No breaking changes in semver minor. Low-risk. |
| `ratatui` | 0.30 | 0.30.0 | — | No | Already at latest stable. |
| `crossterm` | 0.29 | 0.29.0 | — | No | Already at latest stable. |
| `clap` | `"4"` | 4.6.1 | 1.85 | No (see note) | 4.6.1 requires Rust 1.85; project MSRV is 1.78+. Pinning `"4"` already resolves to 4.6.1 on new installs — this is a latent v1.0 discrepancy, not introduced by v1.1. Address separately by either bumping MSRV to 1.85 or pinning `"4.5"`. |
| `proptest` | 1.11 | 1.11.0 | — | No | Already at latest stable. |
| `insta` | 1.47 | 1.47.2 | — | No | Already at latest patch. |
| `criterion` | 0.5 | 0.8.2 | — | No (advisory) | Major version jump; not a v1.1 blocker. Benchmarks are advisory-only (not CI-gated). Keep at 0.5 for now. |
| `dirs` | 6 | 6.0.0 | — | No | Already at latest. |

**Confidence for version data: HIGH** — verified against crates.io API for all crates on 2026-05-08.

---

## MSRV Note: clap 4.6.1 vs Rust 1.78+

`clap = "4"` in the workspace resolves to 4.6.1 (released 2026-04-15), which declares `rust_version = "1.85"`. The project's stated MSRV in `CLAUDE.md` and `PROJECT.md` is "Rust stable 1.78+". This is a latent inconsistency introduced before v1.1 — clap 4.5.x had MSRV 1.74. v1.1 does not introduce any new clap usage (the existing `--state-file` flag and `--help`/`--version` are unchanged). The options are:

1. Bump project MSRV to 1.85 (recommended — Rust 1.85 is the 2024 edition release, CI should already be tracking stable).
2. Pin `clap = "~4.5"` to stay on 1.74-compatible releases.

This decision belongs to the requirements phase, not stack research.

---

## What NOT to Add

| Crate | Reason to Exclude |
|-------|-------------------|
| `phf` | Compile-time perfect hash maps — unnecessary for a 256-entry `u8` lookup; `match` compiles identically. |
| `tokio` | No async operations in v1.1. Print-to-file is synchronous. |
| `crossterm::execute!` / `style` extensions | Print emulation outputs to a file/stdout, not the terminal widget area. No new terminal control needed. |
| `tui-popup` / any ratatui widget crate | The `Clear` + `Block` + `Paragraph` modal pattern is sufficient for the STO arithmetic prompt; no third-party widget library needed. |
| `tracing` / `log` | Print output is user-visible data (simulated HP-41 printer), not a logging concern. `std::io::Write` is the right abstraction. |
| `unicode-width` / `textwrap` | Printer output is 24-column ASCII (HP-41 printer hardware spec). No Unicode layout needed. |

---

## Final Dependency Delta for v1.1

```toml
# Cargo.toml (workspace) — CHANGE
[workspace.dependencies]
rust_decimal = "1.42"   # was 1.41; minor bump, released 2026-05-06

# All other workspace dependencies: UNCHANGED
# All crate-level dependencies: UNCHANGED
# No new [dependencies] entries in any crate
```

That is the complete change. One version number.

---

## Sources

- rust_decimal crates.io: https://crates.io/crates/rust_decimal (v1.42.0, released 2026-05-06)
- ratatui crates.io: https://crates.io/crates/ratatui (v0.30.0, released 2025-12-26)
- crossterm crates.io: https://crates.io/crates/crossterm (v0.29.0, released 2025-04-05)
- clap crates.io: https://crates.io/crates/clap (v4.6.1, MSRV 1.85, released 2026-04-15)
- clap 4.5.37 MSRV: https://crates.io/crates/clap/4.5.37 (rust_version: 1.74)
- criterion crates.io: https://crates.io/crates/criterion (v0.8.2, released 2026-02-04)
- insta crates.io: https://crates.io/crates/insta (v1.47.2, released 2026-03-30)
- proptest crates.io: https://crates.io/crates/proptest (v1.11.0, released 2026-03-24)
- dirs crates.io: https://crates.io/crates/dirs (v6.0.0)
- ratatui Clear widget: https://docs.rs/ratatui/0.30.0/ratatui/widgets/struct.Clear.html
- ratatui popup recipe: https://ratatui.rs/recipes/render/overwrite-regions/
- hp41-cli/src/app.rs: PendingInput enum (lines 22-38) — STO arithmetic variants already stubbed
- hp41-core/src/ops/registers.rs: op_sto_arith() fully implemented
- hp41-core/src/ops/mod.rs: flush_entry_buf() — EEX parse failure path identified
