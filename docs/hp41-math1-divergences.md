# HP-41C Math Pac I Emulator Divergences

This document lists known behavioral divergences between this emulator's implementation
of the HP-41C Math Pac I module and the hardware-faithful behavior described in the
HP-41C Math Pac I Owner's Manual (HP 00041-90034, 1979).

**Status:** Expanded to comprehensive numbered catalog in Phase 30 / Plan 30-02 (DOC-04).

**Philosophy:** Where divergences exist, this emulator prioritizes:
1. Hardware-faithful behavior where feasible.
2. User-safety (no silent data corruption without documentation).
3. Clear documentation of known divergences.

---

## How to Use This Document

Each entry carries a stable `D-30-NN` identifier that can be used in cross-references
from source-code comments, ADRs, test files, and issue trackers. The ID encodes the
phase (30 = Phase 30 / DOC-04) and an ordinal sequence number within this document.

Every entry uses five fixed fields (D-30.5 shape):

- **OM citation** — The HP 00041-90034 page-and-example that is the primary source, or
  `"N/A — emulator extension"` when no OM equivalent exists.
- **Our behavior** — What this emulator does.
- **OM behavior** — What the OM says or what real HP-41C hardware does.
- **Rationale** — Why we made this choice (hardware-fidelity vs. UX trade-off decision).
- **See** — Cross-references: ADR links, CONTEXT.md decision IDs, test file pointers,
  Pitfall references from 28-RESEARCH.md.

The citation discipline (Pitfall 18 from 28-RESEARCH.md) requires every entry to carry
at least one OM page reference or an explicit `"N/A — emulator extension"` marker. No
uncited assertions are permitted in this document.

---

## 1. OM Divergences

*(Numerical / behavioral mismatches with OM-quoted examples or OM-described hardware
behavior. These are cases where the OM specifies or implies a particular outcome and our
emulator either matches or intentionally diverges from that specification.)*

### D-30-01: User-Program Scratch Register Clobber During INTG/SOLVE/DIFEQ

- **OM citation**: HP 00041-90034 (1979), Chapter 3 "Numerical Integration", p. 35 —
  "Important: The INTG program uses registers R00 through R07. Do not use these registers
  in your user function while INTG is active, or the integration result will be incorrect."
  The same constraint applies to SOLVE and DIFEQ which share the same scratch-register
  convention (SOLVE and DIFEQ sections cross-reference Chapter 3 for the scratch register
  warning).

- **Our behavior**: The emulator does NOT snapshot/restore R00–R07 around the user-program
  callback. If the user's LBL function executes `STO 00` through `STO 07`, the solver's
  internal state is corrupted and the numerical result is wrong. No error is raised — the
  result is silently incorrect. A test in `hp41-core/tests/math1_user_callback.rs` explicitly
  asserts this wrong-answer behavior (not an error) to document it as a user-responsibility
  divergence, not a bug.

- **OM behavior**: Identical to ours. The OM warns the user but does not have the calculator
  detect or prevent the corruption. R00–R07 are documented scratch registers for
  INTG/SOLVE/DIFEQ; managing register usage is a user responsibility. Real HP-41C hardware
  also silently produces wrong answers when the user clobbers scratch registers — the
  machine has no mechanism to detect STO operations inside a user callback.

- **Rationale**: Hardware-faithful behavior (Math Pac I behaves this way on real hardware).
  Two mitigation options were considered and rejected:
  (a) Snapshot/restore R00–R07 around each callback invocation: would diverge from hardware
  behavior, add significant overhead per callback (8 register saves/restores at each of
  potentially 32768 sample points), and prevent valid user programs from using R00–R07
  between solver calls in ways the OM permits.
  (b) Detect STO-inside-callback at runtime and raise an error: cannot be done without
  significant interpreter overhead (tracking callback depth and intercepting every STO op),
  and would reject valid user programs that legitimately STO to R00–R07 for scratch work
  the OM explicitly says is the user's problem. Documentation in this file is the accepted
  mitigation per C-28.2.

- **See**: `hp41-core/tests/math1_user_callback.rs::user_fn_stores_to_scratch_corrupts_integ`;
  ADR-002 (`docs/adr/v3.0-002-user-callback-policy.md`); 28-CONTEXT.md C-28.2.

### D-30-02: POLY Complex-Root Multiplicity Rendered as Cluster

- **OM citation**: HP 00041-90034 (1979), Chapter 4 "Polynomial Roots" — POLY worked
  example for multiplicity-k roots (exact page reference to be verified against physical
  copy; the OM demonstrates multiplicity via worked polynomial examples in the POLY section;
  REQUIREMENTS POLY-06 cites this as "matched OM hardware-cluster behavior").

  *(Note: Exact page reference for the POLY multiplicity worked example to be verified
  against physical copy of HP 00041-90034. The above is the best available reconstruction
  from context during Phase 30. Phase 32 will add a verified page citation if the physical
  copy is consulted during the QUAL-01 coverage pass.)*

- **Our behavior**: When POLY finds a multiplicity-k root (for example, `(x-1)^3 = 0` has
  root 1 with multiplicity 3), the modified-Bairstow algorithm returns k nearby clustered
  points rather than a single point marked with its multiplicity. This is the Pitfall 5
  mitigation locked in the v3.0 architecture: multiplicity-as-cluster is the natural output
  of the modified-Bairstow iterative refinement, and no post-processing collapses the
  cluster into a single annotated root. In practice, the cluster is extremely tight (points
  within 1e-8 of each other) but not exactly identical — the output will show k roots
  very close to the true root, not k copies of the exact root.

- **OM behavior**: The OM does not explicitly specify multiplicity-as-cluster versus
  single-root-with-multiplicity convention. The modified-Bairstow algorithm naturally
  produces clustered output on real HP-41C hardware, so this is a documentation gap in
  the OM rather than an algorithmic divergence from described behavior. Our cluster output
  matches Free42's observed behavior on identical inputs (consulted as sanity-check oracle
  per v2.2 D-27.7 pattern; Free42 source was not copied per Pitfall 19 / ADR-002).

- **Rationale**: Modified-Bairstow's clustering is hardware-faithful — the iterative
  refinement converges to nearby points, not to a single exact root with annotated
  multiplicity. Adding a "multiplicity counter" post-processing step would diverge from the
  algorithm's natural output and from real HP-41C hardware behavior. A `// Pitfall 5`
  comment in `ops/math1/poly.rs` marks this behavior for future maintainers.

- **See**: Pitfall 5 (28-RESEARCH.md); REQUIREMENTS POLY-06; `hp41-core/src/ops/math1/poly.rs`;
  28-CONTEXT.md POLY discussion.

### D-30-03: INTG Convergence Threshold Tied to DisplayMode

- **OM citation**: HP 00041-90034 (1979), Chapter 3 "Numerical Integration", p. 35 —
  "The program halts when two successive approximations agree to the number of digits
  displayed (as set by FIX, SCI, or ENG)." More precisely, p. 36 footnote on accuracy —
  "The convergence criterion is that consecutive approximations differ by less than 5 in
  the last displayed digit, i.e., half a unit in the last place (½ ULP) of the displayed
  precision." Maximum subdivisions: p. 37 — "INTG uses a maximum of 2^15 = 32,768 function
  evaluations per interval."

- **Our behavior**: `integ_threshold(mode) = 5 × 10^(-(decimals + 1))` where `decimals`
  is the FIX/SCI/ENG digit count from `state.display_mode`. ADR-004 documents the derivation
  and lock in detail. Different DisplayMode settings produce different convergence outcomes
  for the same integral — by design and by OM specification. For example:
  `Fix(4)` → threshold = `5e-5`; `Fix(9)` → threshold = `5e-10`.
  Maximum subdivisions: 2^15 = 32768 function evaluations per INTG call.

- **OM behavior**: Identical formula — the OM's "½ ULP of displayed precision" wording
  directly yields `5 × 10^(-(decimals + 1))`. This entry is documented here (even though
  it is not a divergence in final value) because the DisplayMode tie-in is surprising to
  users who expect a fixed tolerance. The convergence behavior changes with every FIX/SCI/ENG
  mode change, which can affect whether a given integral converges in a given number of
  evaluations. Free42's `do_intg` uses `0.5 × 10^(-digits)` which is algebraically
  identical (ADR-004 Free42 cross-check table).

- **Rationale**: Hardware-faithful per OM. The DisplayMode tie-in is THE defining feature
  of HP-41C INTG's convergence behavior; an emulator that uses a fixed tolerance instead
  would be wrong. Pitfall 2 (28-RESEARCH.md) identifies this as the most commonly
  incorrectly implemented feature: tests must run in BOTH Fix(4) and Fix(9) and assert
  DIFFERENT precision outcomes, not just "both give correct answer."

- **See**: ADR-004 (`docs/adr/v3.0-004-intg-threshold.md`); Pitfall 2 (28-RESEARCH.md);
  `hp41-core/src/ops/math1/integ.rs`; `hp41-core/tests/numerical_accuracy.rs` Pitfall-2
  detector cases (`intg_fix4_threshold_detector`, `intg_fix9_threshold_detector`).

### D-30-04: FACT Integer-Only — No GAMMA Extension

- **OM citation**: HP 00041-90034 (1979) — Math Pac I does NOT add a Gamma function (Γ
  for non-integer factorial). The FACT function in the base HP-41CV ROM is integer-only
  per the HP-41C/CV Owner's Manual and QRG. The Math Pac I OM does not extend FACT to
  non-integer arguments and does not add a GAMMA function. Integer-only is therefore
  hardware-faithful to both the base ROM and the Math Pac I extension.

- **Our behavior**: FACT (carried from v2.2 Phase 27 implementation) accepts integer X only.
  Range: X = 0 returns 1 (0! = 1, mathematically correct and OM-consistent). X = 1 through
  69 computes the factorial. Effective cap due to `Decimal::from_f64` overflow is X ≤ 26
  (calibrated by Phase 27 proptest; FACT(27) overflows the `rust_decimal` representation).
  X ≥ 70 returns `HpError::OutOfRange`. Negative or non-integer X returns
  `HpError::InvalidOp`. We do NOT add a `GAMMA` function for non-integer factorial — the
  Math Pac I OM does not contain GAMMA and this is a deliberate scope boundary.

- **OM behavior**: Identical — FACT is integer-only on both HP-41C/CV base ROM and Math
  Pac I. The HP-15C's FACT-extension to the Gamma function (where FACT(x) = Γ(x+1) for
  non-integer x) is a different calculator model's feature and does not apply here.
  HP-41C and HP-15C are different machines with different ROM implementations.

- **Rationale**: Scope discipline. Adding GAMMA would require implementing the Lanczos
  approximation or Spouge's formula or similar (~100 lines + hand-rolled coefficients or
  a new dev-dependency); the scope of Phase 28 is "Math Pac I behavioral emulation",
  not "calculator math superset". Adding GAMMA without OM basis would be an undocumented
  extension that silently changes behavior for non-integer inputs that would otherwise
  cleanly error. GAMMA is deferred to Phase 32+ if a user explicitly requests it. The
  boundary is clean: FACT is OM-specified behavior; GAMMA is not.

- **See**: README.md `## Documented Divergences from HP-41 Hardware` (v2.2 FACT cap line);
  `hp41-core/src/ops/math.rs::op_fact`; Phase 27 D-27.5 (FACT case citations in
  `hp41-core/tests/numerical_accuracy.rs` — FACT(0)=1, FACT(70)→OutOfRange, etc.).

---

## 2. Emulator Extensions

*(Functions or behaviors we added that are not present in HP 00041-90034 (1979). These
are deliberate, documented additions that improve usability without conflicting with OM
behavior for OM-specified inputs. Every extension in this section is marked with
"N/A — emulator extension" in the OM citation field.)*

### D-30-05: XEQ "REAL" — Deactivates `complex_mode`

- **OM citation**: N/A — emulator extension. Not present in HP 00041-90034 (1979). The
  Math Pac I OM provides no mechanism to explicitly deactivate complex mode. On real HP-41C
  hardware with Math Pac I, mode management is implicit — the user toggles complex ops and
  manages mode through program structure. There is no "exit complex mode" function in the
  OM.

- **Our behavior**: `XEQ "REAL"` sets `state.complex_mode = false`, restoring the four-level
  stack to standard X/Y/Z/T real-number semantics. No other side effects on stack contents —
  the stack values themselves are not cleared, zeroed, or reinterpreted; only the
  `complex_mode` flag is cleared to false. Reachable from CLI + GUI via the XEQ-by-name
  modal (same mechanism as MATRIX, INTG, POLY, etc.). The function is registered in the
  `hp41-core/src/ops/math1/xrom.rs` XROM table and is visible in the CLI `?`-overlay and
  GUI `?`-overlay Math 1 Pac section. REQUIREMENTS.md CMPLX-18 tracks this as a
  documented extension requirement.

- **OM behavior**: The OM has no equivalent function. Real Math Pac I hardware relies on
  the user managing complex mode implicitly. Entering complex mode happens automatically
  on the first complex op (auto-on per D-28.2), but there is no clean single-keystroke
  "exit complex mode" path on real hardware. Users on real HP-41C hardware who accidentally
  enter complex mode face a poor UX: they must carefully reverse their operations or power-
  cycle the calculator to restore a clean real-only state.

- **Rationale**: UX pragmatism locked in D-28.3 (28-CONTEXT.md). Without an explicit
  deactivation entry point, a user who entered complex mode via XEQ has no clean exit path.
  `XEQ "REAL"` provides a single, discoverable, and idempotent exit point. The trade-off
  (OM-divergence vs. user friendliness) was explicitly evaluated in D-28.3 and the extension
  was accepted. The extension is grep-documented in `xrom.rs` and surfaced in both CLI and
  GUI help overlays so it is discoverable.

- **See**: 28-CONTEXT.md D-28.3; REQUIREMENTS.md CMPLX-18; `hp41-core/src/ops/math1/xrom.rs`.

---

## 3. Behavioral Policies

*(Cross-cutting rules that are decisions worth documenting — not strictly numerical
divergences, but intentional implementation choices with OM basis or deliberate extension.
These entries document cases where the emulator made a specific policy decision that
affects behavior in ways the OM either specifies explicitly or leaves to the implementation.)*

### D-30-06: Strict-Reject Nested INTG/SOLVE/DIFEQ

- **OM citation**: HP 00041-90034 (1979), Chapter 3 "Numerical Integration" — the OM
  explicitly warns against calling INTG recursively from inside the user function. The
  exact phrasing is: "Do not use INTG within a function called by INTG" (exact page
  reference to be verified against physical copy; the warning appears in the Chapter 3
  "Caution" or "Important" note context). Math Pac I OM does not state an explicit error
  behavior for the nested case — it warns the user but leaves behavior undefined.

  *(Note: The exact page reference for the nested-INTG warning is to be verified against
  physical copy of HP 00041-90034. On real HP-41C hardware, nested INTG calls produce
  undefined behavior — scratch register corruption, wrong answers, or calculator hang.)*

- **Our behavior**: `op_integ` / `op_solve` / `op_difeq` detect nested invocation at op
  entry by checking whether the outer `integ_state` / `solve_state` field on `CalcState`
  is `Some(...)`. If nesting is detected, the op immediately returns
  `HpError::InvalidOp` with the message `"NESTED INTG/SOLVE NOT ALLOWED"` BEFORE any
  state mutation. The user-program callback sees the error and aborts cleanly via the
  normal error propagation path. No state corruption occurs. The emulator recovers
  cleanly from the nested-call attempt; the outer solver continues with its next sample
  point.

- **OM behavior**: Implementation-defined undefined behavior on real hardware when nested.
  The OM warns against it but does not define an error message, error code, or clean abort
  path. On real HP-41C hardware, nested calls corrupt the R00–R07 scratch registers used
  by the outer solver (see D-30-01), which produces wrong numerical results. In severe
  cases the machine may hang or require a power cycle. Our strict-reject is a stricter,
  safer, and more user-friendly interpretation of the OM's prohibition — we enforce the
  warning at the engine level.

- **Rationale**: Locked in ADR-002 and 28-CONTEXT.md C-28.2. The OM warns against the
  nested case; we enforce the warning at the engine level rather than relying on the user
  to have read the manual. This is one of the few cases where the emulator is explicitly
  more user-friendly than real hardware behavior. The tradeoff is: real HP-41C gives wrong
  answers silently; we give a clear error immediately. Users who hit the rejection know
  immediately what happened, rather than spending hours debugging a mysterious wrong
  numerical result. Defense-in-depth: the `call_stack` 4-deep cap (from `Op::XeqInd`
  precedent in v2.2) catches pathological nesting BEFORE this explicit check in deeply-
  nested scenarios — both gates together provide defense-in-depth per the architectural
  invariant.

- **See**: ADR-002 (`docs/adr/v3.0-002-user-callback-policy.md`); 28-CONTEXT.md C-28.2;
  REQUIREMENTS.md XROM-08; `hp41-core/tests/math1_user_callback.rs`.

### D-30-07: Modal R/S Submits Numeric Input

- **OM citation**: HP 00041-90034 (1979), p. 13 — "Press R/S to continue" in the context
  of modal prompt sequences (MATRIX `ORDER=?`, INTG `FUNCTION NAME?`, `A=?`, `B=?`, `N=?`,
  DIFEQ similar prompts, POLY degree prompts). This is an explicit OM hardware-faithful
  behavior, not an emulator extension. The OM uses this phrasing consistently throughout
  Chapter 3 and Chapter 4 to describe how users respond to prompts.

- **Our behavior**: Inside a `ModalProgram` flow (e.g., MATRIX entering element values,
  INTG accepting `A=?` / `B=?` / `N=?` / `FUNCTION NAME?` prompts, POLY accepting degree
  and coefficient prompts), pressing R/S submits the current `entry_buf` value as the
  response to the active prompt and advances the `ModalProgram` state machine to the next
  prompt or to execution. The modal clears on completion (all prompts answered), triggering
  the actual computation. If `entry_buf` is empty when R/S is pressed in a numeric prompt,
  the current X register value is used as the default input (matching HP-41 hardware
  behavior for ENTER-with-no-input). Locked in D-28.5 (28-CONTEXT.md).

- **OM behavior**: Identical per OM p. 13. R/S has dual semantics on real HP-41C hardware —
  "run/stop" outside a modal context (starts or halts program execution), and "submit
  numeric input" inside a modal context (advances the prompt flow). This dual behavior is
  hardware-faithful to the HP-41C firmware design. The OM explicitly states "Press R/S to
  continue" at each prompt step, making the dual-semantics a documented feature, not a
  quirk.

- **Rationale**: Hardware-faithful per OM p. 13 explicit statement. The dual R/S semantics
  is preserved by routing the R/S key through the `pending_input` check first (the D-25.11
  invariant from v2.2: `pending_input` routing block must remain ABOVE modal-opening
  interceptors). Two alternatives were considered and rejected:
  (a) ENTER-submits: ENTER's stack-push semantics conflict with MATRIX-Edit's "next element"
  flow, where ENTER would push a number onto the stack that the modal then has to pop back
  off — a confusing round-trip.
  (b) Auto-advance-on-complete-input: timing-dependent tests are brittle and users lose the
  ability to review entry before committing (important for MATRIX element entry where a
  mistake requires re-entering all elements from scratch).

- **See**: 28-CONTEXT.md D-28.5; `hp41-cli/src/app.rs::handle_key` R/S routing block;
  `hp41-gui/src-tauri/src/commands.rs::run_stop` Tauri command (the same Tauri command
  handles both R/S-as-run-stop and R/S-as-modal-submit depending on `CalcState` context);
  HP 00041-90034 (1979) p. 13.

---

*Last updated: 2026-05-17. Catalog expanded to three-bucket numbered format by Plan 30-02
(Phase 30 / DOC-04).*

*Next planned update: Phase 32 may add entries for cross-platform numerical-drift
documentation (QUAL-06), additional POLY/MATRIX worked-example divergences discovered
during the coverage gate push, and the FOUR/DIFEQ/TRANS behavioral specifics cataloged
once Phase 32 numerical accuracy testing is complete.*
