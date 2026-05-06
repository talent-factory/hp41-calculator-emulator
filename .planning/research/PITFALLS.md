# Domain Pitfalls: HP-41 Calculator Emulator (Rust CLI)

**Domain:** RPN calculator emulator / keystroke programming engine / terminal TUI
**Researched:** 2026-05-06
**Applies to project:** hp41-calculator-emulator (behavioral emulation, not cycle-accurate)

---

## Critical Pitfalls

Mistakes that cause rewrites, user-visible fidelity failures, or hard-to-debug corruption.

---

### Pitfall C1: Binary f64 arithmetic breaks HP-41 decimal fidelity

**What goes wrong:**
Using Rust `f64` as the internal number representation causes well-known binary floating-point representation errors. HP-41 uses BCD (Binary-Coded Decimal) internally: a 10-digit decimal mantissa, 2-digit decimal exponent, and two sign nibbles. With `f64`, innocuous operations diverge from hardware: `0.1 + 0.2` does not equal `0.3`, repeated accumulation drifts, and round-trip display formatting produces wrong last digits.

**Why it happens:**
IEEE 754 double-precision stores numbers in base-2. Most decimal fractions (0.1, 0.3, etc.) are non-terminating in binary, so they are approximated. Free42 ships two builds explicitly because "numbers such as 0.1 cannot be represented exactly in binary, and this inexactness can cause some HP-42S programs to fail."

**Consequences:**
NFR-7 requires ≥98% numerical agreement with HP-41 reference across a 500-case test suite. `f64` alone will fail many of those cases at the last digit. Display formatting will show rounding artifacts (e.g., `3.9999999999` instead of `4.0000000000`).

**Prevention:**
Use a decimal arithmetic crate (`rust_decimal`, `bigdecimal`, or a custom BCD struct) for the register values. The internal representation should mirror the HP-41 hardware: 10-digit BCD mantissa with a signed 2-digit exponent. Maintain 13-digit extended precision internally (matching the hardware's dual-register math routines) and round to 10 digits only at display time. `f64` is acceptable only for intermediate calculations in transcendental functions where it is then rounded back to 10 decimal digits.

**Warning signs:**
- Any unit test comparing `sin(45° DEG)` to the reference value with naive `f64` equality
- Display formatter that converts via `f64::to_string()` rather than formatted BCD output
- Trig results differing in digit 9 or 10 from HP-41 reference tables

**Phase:** M1 (Core Arithmetic). Must be decided before any register code is written. Retrofitting is a full data model rewrite.

---

### Pitfall C2: Stack-lift semantics implemented incorrectly

**What goes wrong:**
Stack-lift (the "push flag") is the most commonly mis-implemented HP-41 feature in third-party emulators. The rule is subtle: most operations *enable* stack lift (the next number entry will push a new value onto the stack, lifting X→Y→Z→T and dropping T). A small set of operations *disable* it (ENTER, CLX, CHS during number entry, and a few others). A separate set is *neutral* (does not change the lift flag either way, e.g., STO, RCL).

**Specific invariants that are broken in naive implementations:**

1. `ENTER` disables stack lift. After pressing `3 ENTER ENTER`, the stack should be `3 / 3 / 3 / 3` not `3 / 3 / 0 / 0`.
2. `CLX` disables stack lift (it is a delete operation, not a result). After `CLX`, typing a digit should overwrite X, not lift it.
3. `RCL` enables stack lift. `3 RCL 00` lifts before placing register 00 into X.
4. Arithmetic results enable stack lift. After `3 + 4 =`, X is 7 and lift is enabled, so the next digit entry lifts.
5. Programs can inspect the lift flag (flag 22 = "numeric entry flag"). Misimplementation causes programs that test this flag to misbehave silently.

**Why it happens:**
Developers implement lift as "lift on every number entry" or "lift on ENTER only" — both are wrong. The real model is a per-operation flag that 30+ operations set/clear independently.

**Consequences:**
This is the single most common fidelity bug cited in HP emulator community discussions. Programs that use CLX as a programming idiom (frequent in HP-41 keystroke programs) silently compute wrong results.

**Prevention:**
Create an explicit `StackLiftState` enum (`{ Enabled, Disabled, Neutral }` — neutral operations leave it unchanged). Every operation in `hp41-core` must declare its stack-lift effect. Add a look-up table (or match arm) covering all ~130 operations. Write a dedicated integration test suite for stack-lift semantics with at minimum 20 scenarios covering the known edge cases.

**Warning signs:**
- `CLX` followed by digit entry pushes a new value instead of replacing X
- `ENTER ENTER` on a number does not produce a full stack of that number
- Any implementation where stack-lift is toggled by a boolean per keypress rather than per operation class

**Phase:** M1. Must be correct before any keystroke programming tests can be meaningful.

---

### Pitfall C3: ISG/DSE counter format misread causes silent loop errors

**What goes wrong:**
ISG (Increment and Skip if Greater) and DSE (Decrement and Skip if Equal or Less) use a packed decimal format stored in a register: `CCCCC.FFFDD` where `CCCCC` is the current counter (integer part), `FFF` is the final/test value (first 3 fractional digits), and `DD` is the step size (next 2 fractional digits). Example: `5.01002` means current=5, final=10, step=2.

There is an **asymmetry** between ISG and DSE: DSE was derived from DSZ (decrement and skip if zero) and skips when counter ≤ final. ISG skips when counter > final. The boundary conditions differ. Specifically for DSE with no fractional part, behavior degrades to DSZ-compatible (skip when counter = 0), which is different from how ISG behaves at its boundary.

**Consequences:**
Loop off-by-one errors. Programs that use ISG/DSE for iteration counts produce one too many or one too few iterations. This is silent — no error is thrown.

**Prevention:**
Implement ISG and DSE by parsing the register value string-as-decimal, splitting at the decimal point, extracting substrings for each field, then performing integer arithmetic on those fields. Never use floating-point arithmetic to extract the fields — `floor()` and `fmod()` of `f64` will miscompute the field boundaries for values near power-of-10 boundaries. Test with reference values from HP-41 documentation including the boundary case where current equals final.

**Warning signs:**
- Using `value.fract() * 100.0` to extract the `DD` step field
- A single test for ISG that doesn't cover the counter-equals-final case
- Missing test for DSE with an integer counter (zero fractional part)

**Phase:** M3 (Keystroke Programming Engine).

---

### Pitfall C4: GTO label lookup ignores HP-41 scan order, breaking programs

**What goes wrong:**
HP-41 label lookup is not a simple dictionary lookup. The rules are:

- **Numeric labels (00–99)** are local. GTO nn searches *forward* from the current PC to END, then wraps to the beginning of the current program only. It does NOT search other programs.
- **Alpha labels** are global. GTO "MYPROG" searches from the *beginning of program memory* forward through all programs.
- **A program that starts with a local (numeric) label and has no preceding global (alpha) LBL** becomes unreachable: GTO to it fails with NONEXISTENT, and it cannot even be deleted with CLP. Recovery requires CATALOG 1 to relocate it.

**Consequences:**
Programs that use numeric labels in ways that assume forward-only search will jump to the wrong label when another program with the same numeric label exists in memory. This is a hard-to-debug failure because the program runs but computes wrong results.

**Prevention:**
Implement two separate label resolution paths: local numeric (forward scan within current program, wrapping within program) and global alpha (full scan from start of memory). Validate this behavior against the HP-41 owner's manual. Reject or warn when a program sequence produces an unreachable local-label-only global program during load.

**Warning signs:**
- Label lookup implemented as a `HashMap<label, pc>` without considering scan order
- No distinction between local and global label types in the instruction representation
- GTO tests only cover the happy path (label found immediately after GTO)

**Phase:** M3 (Keystroke Programming Engine).

---

### Pitfall C5: Panic in TUI leaves terminal in raw mode (corrupted shell)

**What goes wrong:**
When a Rust panic occurs in a ratatui application using crossterm, the terminal is left in raw mode with the alternate screen active. The user's shell becomes unusable — no echo, no line buffering, stray escape sequences in prompt. This is the most common complaint in ratatui issue trackers. The older `Terminal::new()` API does not install a panic hook; only `ratatui::init()` (0.28+) does.

**Consequences:**
Any unhandled `unwrap()` or `expect()` in `hp41-cli` during development or in production leaves users with a broken terminal session. NFR-3 (crash-free ≥99.5%) means panics must be eliminated, but during development panics are frequent. A corrupted terminal requires the user to run `reset` or open a new terminal tab.

**Prevention:**
Always use `ratatui::init()` (not `Terminal::new()`) to get automatic panic hook installation. Additionally, wrap the TUI run function with a `std::panic::catch_unwind` and call `ratatui::restore()` before re-raising. Use `color_eyre` for ergonomic error reporting that also triggers terminal restoration. The `hp41-core` crate must be panic-free (NFR-3); all panics must be converted to `Result` before crossing the core/CLI boundary.

**Warning signs:**
- `Terminal::new(CrosstermBackend::new(stdout()))` used directly without a subsequent panic hook setup
- `unwrap()` calls on `io::Result` operations inside the draw loop
- No integration test that simulates a forced panic and checks terminal state

**Phase:** M2 (TUI Shell). First commit of TUI code must include panic hook setup.

---

### Pitfall C6: Windows generates duplicate key events; cross-platform input breaks

**What goes wrong:**
Crossterm on Windows generates two key events for every physical keypress: one for `KeyEventKind::Press` and one for `KeyEventKind::Release`. On Linux and macOS, only `Press` events are generated. Code that processes every key event without filtering by `kind` will execute every operation twice on Windows.

Additional crossterm Windows issues:
- SHIFT+TAB produces inconsistent `KeyCode` + `KeyModifiers` combinations across platforms.
- `Ctrl+C` on the Windows Console immediately terminates the process; it cannot be intercepted the same way as on Unix.
- WSL (Windows Subsystem for Linux) does not fire `crossterm::event::read` on keypress at all — raw mode in WSL outputs raw escape sequences instead.

**Consequences:**
Every calculator key press executes the operation twice on Windows. Number entry produces doubled digits. Stack operations fire twice. This makes the Windows build non-functional without a one-line fix.

**Prevention:**
In the event loop, immediately filter: `if key.kind != KeyEventKind::Press { continue; }`. Add this to the very first iteration of the TUI event handler. Create a CI matrix that runs tests on Windows (GitHub Actions `windows-latest`), macOS (`macos-latest`), and Ubuntu (`ubuntu-latest`) to catch platform differences early. Document that WSL is not a supported target (use native Windows or a Linux VM instead).

**Warning signs:**
- Event handler does not check `key.kind`
- No Windows CI job
- Manual test plan only covers macOS/Linux

**Phase:** M2 (TUI Shell). Must be in the first working event handler.

---

## Moderate Pitfalls

Mistakes that cause debugging time, user-visible bugs, or narrow correctness failures.

---

### Pitfall M1: Trig functions ignore active angle mode at function call site

**What goes wrong:**
The HP-41 has three angle modes: DEG, RAD, GRAD. All trig functions (SIN, COS, TAN, ASIN, ACOS, ATAN) must convert their input based on the *current* mode flag before calling the underlying math function. A naive implementation calls `f64::sin()` directly without conversion, producing wrong results in DEG mode (the most common user mode).

Grad mode is particularly tricky: 400 grad = 360 degrees = 2π radians. `ATAN` in grad mode must also convert the output. Forgetting output conversion on inverse trig in DEG or GRAD mode is a separate, independent bug from forgetting input conversion on forward trig.

**Prevention:**
Encapsulate all trig dispatch in a single module that takes `(value, AngleMode)` and applies the correct conversion. Cover DEG, RAD, and GRAD in the reference test suite (NFR-7). Include known HP-41 reference values: `sin(45 DEG) = 0.7071067812`, `sin(50 GRAD) = 0.7071067812`, `sin(π/4 RAD) = 0.7071067812`.

**Warning signs:**
- Trig functions that take only `f64` without an `AngleMode` parameter
- Test suite only covers RAD mode
- GRAD mode listed as "to be tested later"

**Phase:** M1 (Core Arithmetic).

---

### Pitfall M2: Display formatter does not reproduce HP-41 exact output

**What goes wrong:**
HP-41 has three display modes (FIX n, SCI n, ENG n) each with 0–9 decimal places. ENG mode is not the same as SCI mode — ENG forces the exponent to be a multiple of 3, shifting the mantissa accordingly. Number of displayed characters is capped at 12. Very large or very small numbers trigger automatic SCI display regardless of FIX mode (the HP-41 auto-switches at overflow or underflow).

**Specific traps:**
- `FIX 4` of `1234567.89` must display `1234567.8900` (10 digits total) not `1.2346E6`.
- `SCI 3` of `0.000123` must display `1.230E-4` not `1.23E-4` (note trailing zero in mantissa).
- ENG mode shifts mantissa to keep exponent as multiple of 3: `1230` in ENG 2 is `1.23E3`, not `1.230E3`.
- Negative numbers consume one display character for the minus sign, reducing available mantissa digits.

**Prevention:**
Implement display formatting against the HP-41 Owner's Manual Appendix B reference table. Maintain a set of golden reference outputs for 50+ format/value combinations, tested against the formatter. Do not rely on Rust's `format!("{:.4}", value)` — it does not implement ENG mode or the HP-41's exact rounding behavior.

**Phase:** M1 (Core Arithmetic / Display).

---

### Pitfall M3: Save/load JSON breaks across emulator versions

**What goes wrong:**
The calculator state (stack, registers, flags, program memory, USER mode assignments) saved to JSON in v0.1.0 becomes unreadable when new fields are added in v0.2.0 if the deserializer uses strict field matching. Serde's default `derive(Deserialize)` fails on unknown fields by default in some configurations, or silently omits new required fields, leaving them at `Default::default()` with no warning.

**Consequences:**
Users who save state across an emulator update lose their programs. This is a show-stopper for any user with long-running stored programs.

**Prevention:**
- Tag every save file with a `schema_version: u32` field, checked on load.
- Use `#[serde(default)]` on all new fields so old files deserialize without error.
- Use `#[serde(deny_unknown_fields)]` only in test/validation mode, not production load.
- Write a migration function for each schema version bump.
- Test: serialize with v_n, load with v_(n+1) code, verify state is correct.
- Consider `serde-evolve` or manual `match version {}` migration for non-trivial schema changes.

**Warning signs:**
- No `schema_version` field in the JSON root object
- No test that loads a v1 fixture file with the current code
- `#[derive(Deserialize)]` with no version gate and no `#[serde(default)]` on any field

**Phase:** M4 (Persistence). Must be designed before the first state file is written to disk.

---

### Pitfall M4: `hp41-core` acquires a TUI or I/O dependency

**What goes wrong:**
A developer adds a debug `println!` or imports `crossterm` into `hp41-core` for "convenience." The architectural invariant (zero UI dependencies in core) is silently violated. When v2.0 GUI reuses `hp41-core` via Tauri, the Tauri build fails or pulls in terminal dependencies.

**Prevention:**
Enforce in `hp41-core/Cargo.toml`: never add `crossterm`, `ratatui`, `tokio`, or any I/O crate as a dependency (even `dev-dependencies` should be scrutinized). Add a CI step that runs `cargo deny` or a custom script checking that `hp41-core`'s dependency tree does not include terminal or GUI crates. Use `tracing` (not `println!`) for diagnostics, and keep `tracing` as a feature-gated optional dependency.

**Warning signs:**
- `use std::io::stdout` anywhere in `hp41-core/src/`
- Any `crossterm` or `ratatui` import in core
- Debug output via `println!` that "will be removed later"

**Phase:** M1 onward. Enforce from day one; a CI check prevents regression.

---

### Pitfall M5: ALPHA register and mode state not fully decoupled from numeric state

**What goes wrong:**
The HP-41 has two entirely separate input modes: numeric and ALPHA. In ALPHA mode (flag 48 set), digit keys enter ASCII characters into the alpha register, not numeric digits into X. Number-entry functions (stack-lift, digit assembly) must be suppressed. Many emulators share a single "input buffer" and patch it for alpha mode rather than maintaining a clean state machine.

Specific traps:
- Pressing a digit in ALPHA mode must append the character encoding to the alpha register, not affect the stack.
- `ALPHA` key toggles mode; SHIFT+alpha keys access alternate character sets.
- The ALPHA register holds up to 24 characters — it is NOT the same as the X register.
- `CLA` clears the alpha register; `ARCL nn` appends register nn as a string to the alpha register — both must leave X unchanged.

**Prevention:**
Model input mode as an explicit enum (`InputMode { Numeric, Alpha, ProgramEntry }`). Each mode has its own key dispatch table. The alpha register is a dedicated `AlphaRegister(String)` field, entirely separate from the stack. Use a state machine pattern with explicit transitions, not conditionals scattered through the key handler.

**Phase:** M3 (ALPHA mode / FR-08). Design the input state machine before implementing any input handling.

---

### Pitfall M6: Blocking event read starves the auto-save timer and halts redraws

**What goes wrong:**
Crossterm's `event::read()` blocks indefinitely. If used naively in the main loop, the TUI never redraws (no animation, no auto-save trigger) until a key is pressed. The auto-save requirement (NFR-6: every 30 s) cannot be met with a blocking event loop.

**Why it happens:**
The simplest ratatui tutorial pattern uses `event::read()` in a loop. It works for toy apps with no background tasks. Adding a timer requires switching to `event::poll(Duration)` with a timeout, then handling the timeout case separately.

**Prevention:**
Use `crossterm::event::poll(Duration::from_millis(50))` with a timeout, then `event::read()` only if an event is available. In the timeout branch, check elapsed time for auto-save. Alternatively, use the `tokio` + `EventStream` async pattern, but only if there is genuine need for async concurrency (there is not for this project — synchronous polling is sufficient and simpler).

**Warning signs:**
- Main loop uses `event::read()` with no timeout
- Auto-save implemented with a `thread::sleep` in a background thread (risks data races on state)
- Any `tokio::main` annotation when all I/O is terminal-only

**Phase:** M2 (TUI Shell) and M4 (Auto-save, NFR-6).

---

### Pitfall M7: Ownership design forces `Arc<Mutex<>>` wrapping of calculator state

**What goes wrong:**
A common Rust TUI anti-pattern: the developer reaches for `Arc<Mutex<CalculatorState>>` to share state between the event handler and the render function, then fights the borrow checker, introduces lock contention, and makes the code harder to reason about. For a single-threaded TUI, this is unnecessary complexity.

**Why it happens:**
Developers come from multi-threaded backgrounds or follow async patterns that assume shared ownership. Ratatui's draw closure takes `&mut Frame`, which conflicts with simultaneously holding a `&mut App` unless the design is clean.

**Prevention:**
Keep `CalculatorState` as a single owned value in `main()`. Pass `&mut state` to the update function (event handler) and `&state` to the render function within the single-threaded event loop. The render closure and the update function never run concurrently — there is no need for `Arc` or `Mutex`. If background tasks are needed (e.g., auto-save), use `std::sync::mpsc` channels to send save requests to a dedicated thread, keeping the channel message type simple (e.g., a serialized snapshot).

**Warning signs:**
- `Arc<Mutex<App>>` in the struct definition
- `state.lock().unwrap()` inside the draw closure
- Multiple `&mut` borrows attempted in the same function scope

**Phase:** M2 (TUI Shell). Establish ownership model before writing the event loop.

---

## Minor Pitfalls

Mistakes that cause friction, confusion, or edge-case failures.

---

### Pitfall m1: Legal — accidental inclusion of HP ROM bytes or copyrighted text

**What goes wrong:**
Despite the HP-41C microcode being in the public domain in the US (confirmed: the HP-41C/CV/CX lack copyright notices, predating the 1984 requirement), HP still asserts trademark over calculator model names and may assert rights over specific program listings in later manuals. Copying sample programs verbatim from HP manuals, or including ROM dump bytes in test fixtures, creates legal ambiguity.

**Prevention:**
- Never include raw ROM bytes anywhere in the codebase (already stated in PROJECT.md).
- Write all sample programs independently; do not copy from HP manuals.
- Reference HP-41 behavior from community documentation (hpmuseum.org, hp41.org) rather than from HP's copyrighted PDF manuals.
- Before public release: run a license audit (`cargo deny check licenses`), verify no copyrighted HP text appears in source, README, or test fixtures.
- The behavioral emulation approach (not cycle-accurate Nut CPU simulation) avoids ROM dependency entirely.

**Warning signs:**
- Test fixture files containing hex dumps with "HP" in the comments
- Sample programs copied verbatim from HP-41 Owner's Handbook
- Any `rom.bin` or `rom.hex` file in the repository

**Phase:** Pre-release audit (before v1.0 tag). Flag should appear in M5 (Hardening) checklist.

---

### Pitfall m2: Trig edge cases at exact multiples of 90 degrees

**What goes wrong:**
`f64::sin(π)` does not return exactly `0.0` — it returns `1.2246467991473532e-16`. On the HP-41, `sin(180 DEG)` displays as `0.0000000000`. The transcendental function implementations in hardware use CORDIC algorithms that produce exact results at cardinal angles; naive host-math implementations do not.

**Prevention:**
After computing trig results, apply a final rounding step to 10 significant decimal digits (matching HP-41 display precision). Do not apply a hard-coded special case for each cardinal angle — proper rounding to 10 digits is sufficient to match HP-41 behavior for all cardinal angles.

**Phase:** M1 (Core Arithmetic). Part of the trig function implementation.

---

### Pitfall m3: `cargo test` passes but `hp41-core` unit tests do not run in isolation

**What goes wrong:**
When tests are run only with `cargo test` from the workspace root, feature flags from sibling crates (e.g., `hp41-cli`) can bleed into `hp41-core` compilation, masking dependency violations. A dependency that is only reachable through a feature flag appears absent in normal CI but present in production builds.

**Prevention:**
Run `cargo test -p hp41-core` explicitly in CI, in addition to `cargo test --workspace`. Add `cargo deny check` for license and dependency policy. Use `cargo check --no-default-features` on `hp41-core` to verify the core is self-contained.

**Phase:** M1. Set up in the initial CI configuration.

---

### Pitfall m4: Number entry state machine drops leading zeros and sign

**What goes wrong:**
The HP-41 number entry state machine handles: leading zeros (suppressed in display), decimal point (can be first character), sign toggle (CHS mid-entry changes the sign nibble without altering the mantissa), and exponent entry (EEX followed by digits builds the exponent). A naive implementation using string concatenation loses the sign state when CHS is pressed during mantissa entry, or corrupts the exponent when EEX is followed by CHS.

**Prevention:**
Model number entry as an explicit state machine: `{ Idle, MantissaPositive, MantissaNegative, ExponentEntry, AlphaEntry }`. Do not use a raw `String` buffer — use a structured type that tracks mantissa digits, decimal point position, exponent digits, and sign flags independently. Test: `5 EEX 3 CHS` should produce `5E-3 = 0.005`, not `−5E3`.

**Phase:** M1 (Core Arithmetic / Number Entry).

---

### Pitfall m5: Slow or incorrect rendering because `terminal.draw()` is called multiple times per frame

**What goes wrong:**
Calling `terminal.draw()` more than once per frame renders only the last call's content, because ratatui uses double-buffering: the previous frame is diffed against the new frame to produce minimal terminal writes. Calling draw twice per event loses the first draw's content.

**Prevention:**
Render all widgets in a single `terminal.draw(|frame| { ... })` closure. Never call `draw` in a loop without a corresponding event. The ratatui FAQ explicitly identifies this as a common mistake.

**Phase:** M2 (TUI Shell).

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| M1: BCD arithmetic foundation | C1 (f64 binary errors) | Choose decimal library before any register code |
| M1: Stack operations | C2 (stack-lift semantics) | Implement lift table for all operations; 20+ lift tests |
| M1: Trig functions | M1 (angle mode ignored) + m2 (cardinal angle rounding) | Angle-mode-aware dispatch; round to 10 digits |
| M1: Number entry | m4 (entry state machine) | Explicit state machine; test EEX + CHS sequences |
| M1: Display formatting | M2 (ENG mode, auto-SCI) | Golden reference table; do not rely on Rust format! |
| M2: TUI event loop | C5 (panic/raw mode) | Use ratatui::init(); color_eyre hooks |
| M2: Event handling | C6 (Windows key duplication) | Filter KeyEventKind::Press; CI on all 3 platforms |
| M2: Ownership | M7 (Arc<Mutex> overuse) | Single-owned state; mpsc for background tasks |
| M2: Rendering | m5 (multiple draw calls) | Single draw closure per frame |
| M2: Auto-save timer | M6 (blocking event::read) | Use poll() with timeout |
| M3: Keystroke programming | C3 (ISG/DSE format) | String-parse fields; boundary tests |
| M3: Label resolution | C4 (GTO scan order) | Separate local vs global lookup; scan-order tests |
| M3: ALPHA mode | M5 (alpha/numeric coupling) | Explicit InputMode enum; dedicated alpha register |
| M4: Persistence | M3 (JSON schema versioning) | schema_version field; migration functions; fixture tests |
| M4: Crate boundaries | M4 (core acquires TUI dep) | CI cargo deny check; no println! in core |
| M5: Pre-release | m1 (HP copyright) | License audit; no HP ROM bytes; no verbatim HP text |

---

## Sources

- HP-41 stack-lift behavior: [Automatic stack lift enable/disable — SwissMicros Forum](https://forum.swissmicros.com/viewtopic.php?t=2699); [HP-41 stack behavior — narkive](https://comp.sys.hp48.narkive.com/hJidHxqw/hp41-help-me-understand-the-stack-behaviour)
- ISG/DSE counter format: [ISG/DSE behavior — HPMuseum](https://www.hpmuseum.org/cgi-bin/archv017.cgi?read=118518); [HP-41 Commands PDF — Thimet](http://thimet.de/CalcCollection/Calculators/HP-41/HP-41-Commands.pdf)
- GTO label lookup: [XEQ and GTO indirect — hp41.org forum](https://forum.hp41.org/viewtopic.php?f=20&t=385); [HP-41 Programming — HPMuseum](https://www.hpmuseum.org/prog/hp41prog.htm)
- BCD number format: [13-digit OS routines — Fandom MCODE Wiki](https://mcode.fandom.com/wiki/13_digit_OS_routines); [HP CPU and Programming — HPMuseum](https://www.hpmuseum.org/techcpu.htm)
- Free42 decimal vs binary precision: [Free42 homepage — Thomas Okken](https://thomasokken.com/free42/)
- HP-41 microcode copyright: [HP Calculator Microcode Copyright Status — Nonpareil](https://nonpareil.brouhaha.com/microcode_copyright_status/)
- ratatui panic hook: [Setup Panic Hooks — Ratatui](https://ratatui.rs/recipes/apps/panic-hooks/); [ratatui FAQ](https://ratatui.rs/faq/); [GitHub issue #1005](https://github.com/ratatui/ratatui/issues/1005)
- crossterm Windows issues: [SHIFT+TAB inconsistency — crossterm #442](https://github.com/crossterm-rs/crossterm/issues/442); [Raw mode on Windows — crossterm #584](https://github.com/crossterm-rs/crossterm/issues/584); [WSL issue — crossterm #521](https://github.com/crossterm-rs/crossterm/issues/521)
- Async TUI pitfalls: [Tokio blocking in ratatui — ratatui forum](https://forum.ratatui.rs/t/understanding-tokio-spawn-and-tokio-spawn-blocking/74)
- Serde versioning: [Backward Compatible Serialization — Ivan Ermolaev/Medium](https://ivanbyte.medium.com/backward-compatible-data-de-serialization-with-serde-flow-in-rust-c87a2e8bc9ea); [serde_versioned — crates.io](https://crates.io/crates/serde_versioned_derive)
- HP-41 known bugs: [HP-41 Documentation, List of Bugs — hp41.org](https://forum.hp41.org/viewtopic.php?f=14&t=494)
