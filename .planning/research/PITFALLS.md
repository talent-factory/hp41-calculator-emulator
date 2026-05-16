# Pitfalls: HP-41 Math 1 Pac Emulation (v3.0)

**Milestone:** v3.0 Math 1 Pac Emulation (XROM module emulation + numerical methods)
**Researched:** 2026-05-16
**Scope:** Pitfalls that are SPECIFIC to adding XROM module emulation and the Math-1
numerical-methods feature surface (matrix, complex, polynomial root-finder, integrator,
solver, vector) on top of the shipped v2.2 calculator. Generic Rust / Tauri / serde /
ratatui / stack-lift pitfalls are documented elsewhere (v2.0 PITFALLS.md, CLAUDE.md
settled invariants) and explicitly NOT repeated here.

**Confidence (overall):** MEDIUM. The XROM-numbering and module-collision claims come
from public HP-41 community sources (Owner's Manual, Mike Sebastian's calculator pages,
Free42 source comments, MoHPC forum). The Math-1-specific numerical-method behaviour
(INTEG adaptive Romberg, SOLVE secant fallback, PROOT Laguerre) is documented in the
1981 HP-41 Math 1 Pac Owner's Manual which is the authoritative reference; community
errata may refine edge cases. Treat numerical-accuracy boundary cases as MEDIUM until
the 60-case Math-1 baseline (Phase 31 / FN-QUAL-02 extension) is built.

---

## Summary

Seven categories dominate the v3.0 risk surface. Six are NEW; the seventh (test
infrastructure) is an extension of v2.2's coverage gate to a much larger surface:

1. **XROM-namespace mistakes** — module-ID collisions between Math 1 and built-in ops
   (`Σ+`, `FACT`), `XEQ` "ALPHA" name resolution order, future-Stat-1 hooks that
   accidentally land in Math 1's slot.
2. **Numerical-method ground-truth mismatches** — `INTEG`'s adaptive scheme, `SOLVE`'s
   secant/bisection fallback, `PROOT`'s Laguerre convergence, branch cuts for `CARG`,
   matrix singularity policy, `FACT` extension. Wrong reference behaviour invalidates
   the whole milestone.
3. **User-callback re-entrancy** (INTEG / SOLVE / matrix conditional GTO) — the existing
   `is_running` guard was never designed for nested `run_program` invocations from a
   built-in op; register/flag/stack scratch corruption; recursion bombs.
4. **Long-running ops freeze the Tauri UI** — `dispatch_op` is sync; an INTEG on a
   slow user function can hold the `Mutex<CalcState>` for many seconds; no
   cancellation channel exists.
5. **Op-enum explosion vs `XromCall(u16)` dispatch** — adding ~40 variants pushes
   `Op` from ~150 to ~190; criterion benchmark may regress; `OpDisplay` exhaustive
   matches grow to four parallel arms (dispatch, execute_op, both prgm_display.rs).
6. **Save-file mid-INTEG / mid-SOLVE state** — a running solver's interim register
   set is not safe to round-trip through autosave; partial matrix-mode header bytes
   in `regs` look like ordinary numbers but represent module-state.
7. **Test infrastructure under numerical drift** — float-equality assertions fail
   non-deterministically across x86/ARM; ~40 new functions × N cases may temporarily
   drop the 95 % coverage gate mid-milestone; Math 1 has no canonical answer key.

Two cross-cutting pitfalls touch GUI keyboard reachability (Pitfall 21) and
documentation provenance (Pitfall 22) — both are MEDIUM-severity but easy to
prevent if addressed in Phase 28.

---

## Critical Pitfalls

These mistakes cause rewrites, silent wrong answers, or legal exposure.

### Pitfall 1: Function-Name Collision Between Math 1 and Built-In Ops

**What goes wrong:**
The HP-41 Math 1 Pac contains a `Σ+` function for matrix statistics that is NOT the
same operation as the built-in `Σ+` (Statistics accumulation into R11–R16). On real
hardware they are distinguished by XROM-number priority: built-ins win when both are
visible. A naive Rust implementation that adds `Op::MatSumPlus` and binds the string
`"Σ+"` for XEQ-by-name will route ALL `Σ+` calls (including user programs that pre-date
v3.0) to the wrong target.

Other collision candidates discovered in 5 minutes of the Owner's Manual:
- `MEAN`, `SDEV` (Math 1 matrix-statistics variants vs Σ-register built-ins)
- `MOD` (Math 1 may use it for matrix modulus / determinant sign — verify)
- `FACT` (Math 1 may extend factorial to gamma — see Pitfall 9)
- `ABS` (Math 1 has complex / vector ABS that must NOT shadow built-in)

**Warning sign:** A user program saved under v2.2 that calls `XEQ "Σ+"` produces a
different numeric result after upgrading to v3.0. Worst case: silent wrong answer in
a savings/financial calculation.

**Prevention:**
- `xeq_by_name_local_resolve` (the fast-path resolver) MUST look up built-in mnemonics
  BEFORE consulting the XROM table. Encode this as the first arm of the match.
- Add a regression suite that loads every v2.2 sample program and asserts identical
  numeric output before and after the v3.0 merge.
- For each Math-1 function that shadows a built-in, store the Math-1 variant under a
  disambiguated mnemonic in the JSON catalog (`MAT-Σ+`, `C-ABS`, `V-ABS`) and accept
  the unprefixed form ONLY when the matrix/complex/vector mode-flag is set.
- A frontend-only "module catalog" overlay should make the disambiguation visible — if
  the user types `XEQ "ABS"` in plain numeric mode, the built-in wins silently; the
  catalog explains why.

**Which phase:** Phase 28 (XROM framework — the resolver order is a framework decision,
not a per-op decision). Pitfall surfaces in Phase 29 (first Math-1 op that shadows a
built-in name).

**Detection:** New CI gate `tests/xrom_shadowing.rs` that enumerates every
`docs/hp41-math1-functions.json` entry and asserts: (a) the unprefixed mnemonic
resolves to the built-in if one exists, (b) the prefixed mnemonic resolves to the
Math-1 variant, (c) no Math-1 entry has the SAME `xrom: (mod, fn)` tuple as another
entry.

---

### Pitfall 2: `INTEG` Convergence Behaviour Differs from the Owner's Manual

**What goes wrong:**
The HP-41 Math 1 `INTEG` is not a simple Simpson's rule — it is an adaptive scheme
(community-reconstructed as a Romberg-with-acceleration variant) that doubles the
subdivision count until two successive estimates agree to the current display
precision (`FIX n` / `SCI n`). Two mistakes are equally bad:

1. **Too few subdivisions (premature convergence):** a smooth-but-narrow peak (e.g.
   `f(x) = exp(-100*(x-0.5)^2)` on `[0,1]`) returns wrong-by-50 % because the initial
   sampling missed the peak entirely. Real hardware also misses it — but real hardware
   reports the wrong answer with NO warning. The emulator must reproduce that
   behaviour (silent wrong answer) rather than emit a warning we invented.

2. **Too many subdivisions (slow but correct):** holding the calculator busy for 30+
   seconds with no progress signal locks the GUI (see Pitfall 11).

A subtler trap: the Owner's Manual ties the convergence threshold to the CURRENT
`DisplayMode` setting. A user in `FIX 9` will get a much more expensive INTEG than the
same INTEG in `FIX 2`. If the emulator hard-codes the threshold (e.g. always 1e-10),
the visible behaviour diverges from real hardware in a way that is not auditable.

**Warning sign:** `tests/numerical_accuracy.rs` adds an INTEG case for
`∫₀^π sin(x) dx` and the result is 2.0001 in `FIX 4` but 2.000000183 in `FIX 9`. If
the test passes in both modes with the SAME tolerance, the convergence-threshold logic
is wrong.

**Prevention:**
- Read the HP-41 Math 1 Owner's Manual (Section 5) and the HP Journal April 1979
  ("Personal Programmable Calculator Routines") for the canonical adaptive scheme.
  Free42 source (`integration.c`) is the cleanest reference re-implementation — but
  Free42 is GPL, so cite it for behaviour-verification only and re-derive from
  first principles for the implementation (Pitfall 22).
- Tie the convergence-threshold to `state.display_mode` explicitly. Encode the
  formula `threshold = 10^(-decimals - 1)` in a single helper in
  `hp41-core/src/ops/math1/integ.rs` and unit-test the threshold separately from the
  integration.
- Cap the maximum subdivision count at 2^15 = 32768 sample points. Real hardware caps
  somewhere around 2^12 (it runs out of time before memory); the emulator should cap
  for the same user-experience reason, not to save memory.
- Document divergences in `docs/hp41-math1-divergences.md` for any test case where the
  emulator's answer differs from the manual's quoted example by more than 1 ULP at
  10-digit precision.

**Which phase:** Phase 30 (numerical-method ops: INTEG, SOLVE, PROOT). The threshold
ground-truth research belongs in Phase 28 research-prep — do not start Phase 30 until
the Owner's Manual reference table is transcribed.

**Detection:** A new numerical-accuracy file `tests/math1_accuracy.rs` with ≥ 60 cases
covering the Manual's worked examples in Sections 4–7. Each case carries an Owner's
Manual citation (page + example #) in the test doc-comment, matching the v2.2 pattern
(`// Catches: <bug class>`, D-27.1).

---

### Pitfall 3: `SOLVE` on f(x) with No Real Root — Wrong Reference Behaviour

**What goes wrong:**
The HP-41 Math 1 `SOLVE` uses the secant method with bisection fallback when the
bracket sign changes. Three input regimes need different handling and the manual is
NOT unambiguous about all three:

1. **No real root** (e.g. `f(x) = x² + 1`): real hardware returns a local minimum
   of `|f(x)|` and sets flag 25 (error-ignore) — but only in `f(x)≠0` mode. In
   `f(x)=0` mode it returns `DATA ERROR`. The emulator must replicate THIS specific
   convention.
2. **Multiple roots** (e.g. `f(x) = sin(x)` with seed near multiple zeroes): hardware
   returns the root nearest the seed, but the convergence path depends on the
   ROUNDING of intermediate results — meaning the answer can flip between two roots
   based on `DisplayMode`. Reproduce this; do NOT "fix" it.
3. **Discontinuity at the root** (e.g. `f(x) = 1/(x-2)` with seed 1.9): hardware loops
   indefinitely until interrupted. Emulator must surface this as a step-limit error,
   NOT as silent infinite-loop.

**Warning sign:** `SOLVE` of `x² + 1` returns `0` (a local min of |x²+1|) and a unit
test that expected `DATA ERROR` fails. Or the opposite: it returns `DATA ERROR` when
the user's mental model said "give me the local minimum, that's useful."

**Prevention:**
- Encode the three regimes as named branches in `op_solve()`. Document each branch
  with a citation to the Math 1 Owner's Manual section.
- Hard-cap the solver at 100 iterations (real hardware caps around 64; we give a small
  buffer for the rounding-divergence between rust_decimal and HP BCD).
- Re-use `run_program`'s 1000-step `MAX_STEPS` ceiling for the user-callback budget.
  EACH iteration of SOLVE runs the user function once; the budget is solver-iterations
  × user-callback-steps. Document this product as the effective limit.
- Add a regression test for each of the three input regimes from the Owner's Manual.

**Which phase:** Phase 30 (numerical-method ops).

**Detection:** Three new accuracy cases — `solve_no_real_root`, `solve_multiple_roots_
seed_drift`, `solve_discontinuity_at_root` — each citing the Owner's Manual example.
Add a property test that asserts: if `f` is a polynomial of degree ≤ 2 with a known
real root, SOLVE finds it within 100 iterations.

---

### Pitfall 4: User-Callback Re-Entrancy Corrupts Solver State

**What goes wrong:**
INTEG and SOLVE take a user-defined function (a global LBL) as an argument. The
solver calls `run_program(state, label)` for each sample point. Inside that user
function the user CAN:

(a) Call `STO 00`–`STO 99` — but Math 1's INTEG uses some of these as scratch
    (community-documented: R03–R06, exact set varies by version). User STO clobbers
    the solver's bookkeeping → wrong answer with no warning.
(b) Call another `INTEG` / `SOLVE` (nested numerical method) → real hardware allows
    one level of nesting; further nesting overflows scratch registers and silently
    gives wrong answers. Emulator must enforce the same depth limit (see Pitfall 12).
(c) Set / clear flags 11–25 — Math 1 uses some of these to signal mid-integration
    error states. User flag changes corrupt the solver's "I am inside INTEG" tracking.
(d) Execute `STOP` / `GTO` out of the function — should abort the surrounding INTEG.
    Real hardware does abort. Naive emulator code returns the solver to the next
    sample point unchanged → infinite loop.
(e) Execute `BEEP`, `PSE`, `PROMPT` — must be allowed; the solver must pass through
    these to the frontend's event_buffer without corruption.

The existing v2.x re-entrancy guard (`is_running` flag in `run_program`) was designed
for ONE level of programmatic execution. It is set to `false` in the `Err` path
(Pitfall 2 in program.rs line 419 comment) and the cleanup is currently correct for
single-level execution. But Math 1's nested-INTEG case will trip it: outer INTEG sets
`is_running=true`, calls `run_program` for user fn, user fn calls `XEQ "INTEG"`,
which observes `is_running=true` and bails. Either we relax the guard (allowing
re-entry from within `is_running=true` contexts) or the nested-INTEG case will
DATA ERROR when the manual says it should work-once.

**Warning sign:** A test that integrates `f(x) = sin(x)` over `[0,π]` where `f` uses
`STO 03` for a local variable returns garbage (not 2.0). Or: a nested-INTEG returns
`InvalidOp` instead of running.

**Prevention:**
- Before Phase 30, audit the v2.2 `is_running` guard and decide between three policies:
  - **Strict (current):** nested run_program returns `InvalidOp`. Forces Math 1 INTEG
    to call user fn via a NEW non-guarded entry point. Pro: smallest surface change.
    Con: nested-INTEG impossible.
  - **Counter-based:** replace `is_running: bool` with `run_depth: u8`. Allow up to
    `MAX_RUN_DEPTH = 2` (one user-fn nesting). Pro: matches hardware. Con: cascading
    changes through every `if !state.is_running` check in `ops/`.
  - **Caller-supplied callback budget:** `op_integ` saves+restores its own scratch
    region (R03–R06 by Math-1 convention) AROUND each user-fn call. Pro: hardware-
    faithful, no `is_running` change. Con: 40 lines of save-restore boilerplate per
    numerical-method op.
- Pick policy in Phase 28 framework research, not Phase 30 implementation.
- Whichever policy: add an integration test where the user function deliberately
  STOs to the solver's scratch register, and assert the solver still returns the
  correct integral (because save-restore happens) OR a specific error code (because
  we picked the strict policy and documented it).

**Which phase:** Phase 28 (framework decision); pitfall first surfaces in Phase 30
implementation.

**Detection:** Five regression tests under `tests/math1_user_callback.rs`:
`user_fn_stores_to_scratch`, `user_fn_nested_integ`, `user_fn_modifies_flag`,
`user_fn_executes_stop`, `user_fn_executes_goto_out`. Each one's expected outcome is
documented from the Owner's Manual.

---

### Pitfall 5: `PROOT` Convergence Failure on Ill-Conditioned Polynomials

**What goes wrong:**
The HP-41 Math 1 `PROOT` finds all complex roots of a polynomial of degree ≤ 5 (Math 1
limit; some Advantage Pac versions go to 10). The published algorithm is Laguerre's
method with deflation. Two failure modes:

1. **Ill-conditioned polynomial** (e.g. `x⁵ - 5x⁴ + 10x³ - 10x² + 5x - 1` = `(x-1)⁵`
   has a 5-fold root at 1; even small rounding error during deflation produces a
   "cluster" of roots near 1 with imaginary parts ~10⁻³ — visibly wrong if naively
   displayed as 5 distinct complex numbers).
2. **Roots at very different magnitudes** (e.g. roots at 1 and 10⁹) → deflation
   accumulates catastrophic cancellation. Hardware reports `DATA ERROR`; a naive
   emulator returns garbage roots.

Math 1's specific convention for case 1 is: return the cluster (the user is expected
to recognize the clustering as multiplicity). Case 2: DATA ERROR.

**Warning sign:** `PROOT` of `(x-1)⁵` returns 5 roots with imag parts up to ±0.001.
Test author thinks that's a bug; it's actually correct per Math 1, but the test
asserts exactness and fails.

**Prevention:**
- Encode the Laguerre-with-deflation algorithm exactly as documented in the Math 1
  Owner's Manual (NOT a "better" modern alternative like Jenkins-Traub — the goal is
  behavioural emulation, not better answers).
- Document the multiplicity-as-cluster convention in `docs/hp41-math1-divergences.md`
  so test authors do not write exactness assertions.
- Set the deflation-failure trigger to match hardware: any root with |imag| > 10⁹
  during a real-polynomial PROOT is treated as "did not converge" → DATA ERROR.
- Hard-cap the per-root Laguerre iteration count at 50 (manual is silent; community
  decompiles say ~30).

**Which phase:** Phase 30 (numerical-method ops).

**Detection:** Owner's Manual Section 7 contains ≥ 5 worked PROOT examples. Each one
goes into `tests/math1_accuracy.rs` with relative-tolerance comparators (Pitfall 17).

---

### Pitfall 6: Complex `CARG` and `CDIV` Branch-Cut / Zero Conventions

**What goes wrong:**
Complex argument `CARG` (= `atan2(imag, real)`) must follow the principal-value
convention. The HP-41 Math 1 convention is the standard mathematical one:
`CARG ∈ (-π, π]`. The trap: many naive `atan2(0, 0)` implementations return 0; some
return NaN; on real HP-41 hardware it returns 0 in DEG/RAD modes AND `DATA ERROR`
is NOT raised. Test infrastructure built on Rust's `f64::atan2` is mostly correct,
but `rust_decimal` does not have `atan2` — a hand-rolled implementation MUST
explicitly handle the (0,0) case.

`CDIV` with the divisor 0+0i: hardware returns `DATA ERROR`. Naive code returns
`+inf+0i` or NaN.

`CDIV` with a near-zero divisor (e.g. 10⁻⁹⁹ + 0i): hardware overflows the result and
returns `DATA ERROR`. Emulator's HpNum max is around 10⁹⁹ (rust_decimal's 96-bit
range); pick whichever is closer to hardware.

**Warning sign:** `CARG(0, 0)` returns NaN and the display shows "NaN" — never happens
on real hardware.

**Prevention:**
- Implement `complex_atan2(im: HpNum, re: HpNum) -> HpNum` as a single explicit
  function in `hp41-core/src/ops/math1/complex.rs`. Handle the (0,0) → 0 case as the
  first arm; cite the Owner's Manual page-and-paragraph in the doc-comment.
- For `CDIV`, branch on `re == 0 && im == 0` BEFORE the actual division and return
  `HpError::DataError`. Mirror the v2.2 `1/0 → DATA ERROR` convention from
  `op_recip()`.
- Add edge-case tests: `CARG(0,0)`, `CARG(0, 1e-99)`, `CARG(-0.0, 1.0)`, `CDIV` by
  `0+0i`, `CDIV` by `1e-99 + 0i`.

**Which phase:** Phase 29 (complex-number ops are the first Math-1 ops to implement
after the framework).

**Detection:** New `tests/math1_complex_edge_cases.rs` file with ≥ 8 edge cases —
each citing Owner's Manual Section 6.

---

### Pitfall 7: Matrix `INV` Singularity Detection and DATA-ERROR Convention

**What goes wrong:**
HP-41 Math 1 `INV` (matrix inverse) uses LU decomposition with partial pivoting.
Singular matrices are detected by `|pivot| < EPSILON` for a hardware-specific EPSILON
(community-documented as `5e-10`, but the manual quotes `1e-9` for the `DET` zero
test — they are NOT the same threshold). Three failure modes:

1. **Truly singular** (e.g. `[[1,2],[2,4]]`): hardware reports `DATA ERROR`. Easy case.
2. **Near-singular** (e.g. `[[1,1],[1,1.0000000001]]`): hardware DOES invert,
   returning huge but finite numbers. Naive `det == 0` check incorrectly DATA-ERRORs.
3. **Floating-point underflow during back-substitution**: hardware returns DATA ERROR;
   naive implementation returns infinity or unexpectedly-shaped result.

The EPSILON value is HARDWARE-SPECIFIC. Picking the wrong value means our INV
disagrees with the manual on EVERY near-singular test case.

**Warning sign:** `INV` of `[[1,1],[1,1.000000001]]` returns `DATA ERROR` (because our
EPSILON is 1e-8 instead of the correct 5e-10) — but the manual's worked example
expects a finite inverse.

**Prevention:**
- Source the EPSILON value from the manual (or, failing that, from a hardware
  experiment). DO NOT pick a "reasonable" value out of intuition.
- Encode the threshold as a `pub const EPSILON: HpNum` in
  `hp41-core/src/ops/math1/matrix.rs` so it shows up in code review and in the
  divergence document.
- The 10⁻⁹⁹ overflow case during back-substitution: branch on
  `result.abs() > 1e99` after the back-substitution loop and emit DATA ERROR.

**Which phase:** Phase 29 (matrix ops).

**Detection:** Three accuracy cases — `inv_singular`, `inv_near_singular`,
`inv_back_sub_overflow`.

---

## Moderate Pitfalls

### Pitfall 8: XROM-Number Allocation Collides with Stat 1 Plans

**What goes wrong:**
Math 1 is module 02 in HP's catalogue. Stat 1 is module 03. Time is 04. Advantage is
06. The 6-bit module ID range is 0–63. v3.0 ships Math 1 (mod 02) only; v3.1 will
ship Stat 1 (mod 03). If we hard-code module 02 in v3.0 framework code (e.g. in an
enum `Module::Math1 = 2`), v3.1 either has to add `Module::Stat1 = 3` (fine) or — if
we accidentally use `u8` literals instead of the enum — has to find every `0x02 |`
in the codebase and decide if it's a Math 1 reference or unrelated.

Worse: a future user can XEQ "BY XROM" syntax `XEQ "02,15"` (module 2, function 15).
v3.0 must accept that syntax for Math 1. v3.1 adding module 3 must NOT change v3.0's
behaviour for module 2 calls.

**Warning sign:** v3.1 work breaks a v3.0 test that called `XEQ "02,15"`.

**Prevention:**
- Encode module IDs as constants in a single `hp41-core/src/ops/xrom/registry.rs`,
  not as `u8` literals scattered across call sites.
- The XROM-by-number syntax (`XEQ "02,15"`) must resolve through the same registry
  table as the by-name syntax. No parallel mapping.
- v3.0 leaves explicit empty rows in the registry for modules 03–10 with comments
  pointing to which version will populate them.

**Which phase:** Phase 28 (XROM framework).

**Detection:** Unit test that asserts module 02 has ≥ 40 entries (Math 1) and modules
03–10 have 0 entries. v3.1 will flip the assertion for module 03.

---

### Pitfall 9: `FACT` Extension Beyond v2.2's [0, 69] Range

**What goes wrong:**
v2.2's `FACT` is range-limited to `[0, 69]` (because `70! > 10⁹⁹` overflows `HpNum`).
HP-41 Math 1 may extend `FACT` via the gamma function — `FACT(2.5) = gamma(3.5)
≈ 3.323` — but ONLY in some Math 1 versions; others leave FACT integer-only and add
a separate `GAMMA` function. Picking the wrong convention means a user upgrading from
v2.2 sees a different FACT semantics.

Even if we choose "leave v2.2 FACT alone and add Math1 GAMMA", the namespace
collision (Pitfall 1) still applies — if any Math 1 ROM function happens to be named
`FACT` it must NOT shadow the built-in.

**Warning sign:** A v2.2 program that did `2.5 FACT → DATA ERROR` now returns
`3.323` after upgrade.

**Prevention:**
- Decide explicitly in Phase 28 research: extend FACT, or add a separate GAMMA?
- Document the decision in `docs/v3.0-decisions.md` with citation to the Owner's
  Manual function list.
- A regression test loads v2.2 sample programs and asserts identical output.

**Which phase:** Phase 28 (framework decision); pitfall surfaces Phase 29
implementation.

**Detection:** v2.2 numerical-accuracy suite re-runs unchanged. Any FACT case that
flips outcome is a red flag.

---

### Pitfall 10: Op-Enum Variant Explosion vs `XromCall(u16)` Dispatch

**What goes wrong:**
Math 1 has approximately 40 named functions. Adding 40 variants to `Op` pushes it
from ~150 to ~190. Stat 1 adds another ~30. Time adds ~25. Advantage adds another ~60.
By v3.3 we are at 300+ Op variants. Two consequences:

1. **`prgm_display.rs` exhaustive match** — currently exhaustive (no `_ =>` arm),
   per CLAUDE.md "Every new Op variant must be added in both copies". Going to 300
   arms in two files makes drift between hp41-cli and hp41-gui copies likely.
2. **Match-table codegen** — the Rust compiler synthesizes a jump table for large
   `match` over enums. 300+ arms across `dispatch()`, `execute_op()`, two
   `prgm_display.rs`, `synthetic_byte_to_op`, JSON serde — each match recompiles
   when ANY arm is added. Build-time grows quadratically.

The alternative `Op::XromCall(module: u8, function: u8)` keeps the enum small but
puts dispatch behind a runtime lookup hop on a hot path. v2.2 measured ~65 ns/op
dispatch (QUAL-02); a hashmap lookup adds ~30 ns. Net 95 ns is still well inside the
50 ms gate, but the criterion benchmark will visibly regress.

**Warning sign:** `cargo bench` shows a 50 % regression on the dispatch benchmark
after Phase 28. CI does not gate on bench, but the regression number gets quoted
in a PR review and someone reverts the framework.

**Prevention:**
- Decide enum-vs-XromCall in Phase 28 framework research with an explicit ADR
  (Architecture Decision Record).
- If enum: write a `match` code-generator in `scripts/op-codegen` that produces the
  four exhaustive arms from the JSON catalog, eliminating manual drift.
- If XromCall: cache the `(module, function) → fn pointer` lookup in a `OnceLock<
  Vec<fn(&mut CalcState, ...)>>` populated at startup. Lookup becomes an array index
  (~5 ns), not a hashmap probe.
- Set a criterion-gate floor: `dispatch_overhead < 200 ns/op` (current 65, headroom 3x).

**Which phase:** Phase 28 (framework — irrevocable decision).

**Detection:** Criterion benchmark `bench/dispatch_overhead.rs` with the 200 ns/op
floor. v2.2 baseline 65 ns is documented in the benchmark output.

---

### Pitfall 11: Long-Running INTEG / SOLVE Freezes the GUI

**What goes wrong:**
`dispatch_op` in `hp41-gui/src-tauri/src/commands.rs` is synchronous. It takes the
`Mutex<CalcState>` lock, calls `hp41_core::ops::dispatch(...)`, releases the lock,
returns. v2.2's longest dispatch is a `XEQ` of a user program with the 1000-step
`MAX_STEPS` ceiling — measured at ~5 ms worst case.

Math 1's INTEG of a slow user function can run for 10+ seconds:
- 2^12 = 4096 sample points at the convergence cap
- Each sample = one `run_program` invocation of the user function
- User function ~50 steps at ~1 ms each
- Total: 4096 × 50 ms = ~200 seconds worst case

During this time:
- The `Mutex<CalcState>` lock is held — the 30s auto-save thread deadlocks waiting
  for it (or worse, the auto-save grabs the lock between sample points and pauses
  INTEG).
- The frontend's `await invoke('dispatch_op', ...)` never returns — the UI is frozen,
  the user cannot press R/S to abort.
- The Tauri main thread is blocked on the lock — other Tauri commands queue.

`run_program`'s existing `MAX_STEPS = 1000` cap protects the CLI but does NOT scale
to numerical methods.

**Warning sign:** User clicks INTEG, GUI freezes for 30+ seconds, user force-quits.

**Prevention:**
- Build a cancellation channel: `CalcState.cancel_requested: Arc<AtomicBool>` (NEW
  field; `#[serde(default, skip)]`). `op_integ` / `op_solve` check it every N sample
  points and return `HpError::Interrupted` if set.
- Add a `request_cancel` Tauri command that flips the AtomicBool. R/S key on the
  frontend calls `request_cancel` (NOT `run_stop` — different semantic).
- Make `op_integ` release-and-reacquire the lock between sample-batches: every 64
  samples, drop the MutexGuard, sleep 0 ms (yields to scheduler), reacquire. This
  lets the auto-save thread and `request_cancel` interleave.
- Document that the auto-save thread MUST tolerate a held lock — extend the
  ff39017 fix (release Mutex BEFORE disk I/O) to also retry-with-backoff on a
  contended lock.
- A separate progress channel: `state.progress: Option<String>` (`serde skip`),
  written by INTEG every N samples. Frontend polls `get_state` to display it.

**Which phase:** Phase 32 (GUI integration of Math 1 ops). Pitfall surfaces immediately
when the first INTEG test runs in the GUI.

**Detection:** New E2E smoke (extension of v2.2 FN-QUAL-05): click INTEG of a slow
user function, click R/S after 1 second, assert the GUI returns to responsive within
2 seconds. Run on Ubuntu only (matching v2.2 e2e-linux job).

---

### Pitfall 12: Mid-Solver Save-File State Is Not Round-Trippable

**What goes wrong:**
INTEG and SOLVE store interim state in scratch registers (R03–R06 per community
documentation). The 30s auto-save thread can fire WHILE INTEG is running. If the
autosave round-trips and the user re-opens later, the registers look like ordinary
numbers — but they encode mid-integration state. The user has no way to know.

Worse: if INTEG itself adds new CalcState fields (e.g. `integ_state: Option<
IntegState>` for the convergence-tracking), and we forget `#[serde(default, skip)]`,
the save-file embeds the partial state and resuming continues a half-finished
integration — wrong answer.

**Warning sign:** User closes app mid-INTEG, reopens, sees R03–R06 set to weird
values but X register looks normal. Continues computing on top of dirty scratch.

**Prevention:**
- All Math-1 interim state fields (matrix-mode header, complex-mode flag, INTEG
  iteration buffer, SOLVE bracket) get `#[serde(default, skip)]` from the FIRST
  commit that introduces them.
- The 30s auto-save thread skips when `state.is_running == true` — already true in
  v2.2 (verify this). Extend the skip condition to also cover `state.cancel_requested`
  being set (mid-cancellation is not a stable save point).
- A startup-load check: if R03–R06 are non-zero AND no user-program has been loaded,
  display a `WARNING: scratch registers non-zero (from previous session?)` toast.
  Phase 32 GUI; Phase 31 CLI shows it in the status bar.

**Which phase:** Phase 29 (matrix interim state) onward; Phase 32 GUI hooks for the
warning toast.

**Detection:** A round-trip test that: starts INTEG, suspends after 10 samples,
serializes CalcState, deserializes into a fresh state, asserts that INTEG-related
fields are all default. Then runs INTEG fresh and asserts identical-to-direct result.

---

### Pitfall 13: GUI Keyboard Reachability — Math 1 Has No Dedicated Keys

**What goes wrong:**
Math 1 functions are XEQ-by-name only. v2.2's XEQ-by-name modal scaled to ~130
built-in mnemonics. Adding 40 Math-1 mnemonics tips the modal past the "easily
scrollable" threshold. Users complain.

Worse: Math 1's `MAT-Σ+` disambiguation prefix (Pitfall 1) means users have to know
to type the prefix — discovery problem. Real HP-41 hardware solves this via
`CATALOG 2` (lists ROM functions including modules). v2.2 has `CATALOG 1` (user
programs) but not `CATALOG 2`. Without `CATALOG 2`, Math 1 functions are invisible.

**Warning sign:** A user types `INV` in the XEQ-by-name modal expecting matrix
inverse, the modal shows zero matches because we registered it as `MAT-INV`. User
gives up.

**Prevention:**
- Add `Op::Catalog(u8)` with the `2` variant in Phase 28; implement Phase 31.
  CATALOG 2 lists all XROM functions grouped by module, paged through with R/S.
- The XEQ-by-name modal in Phase 31 (CLI) and Phase 32 (GUI) does prefix-tolerant
  matching: typing `INV` matches both `INV` (if a built-in by that name exists) AND
  `MAT-INV` (the Math 1 variant), with the built-in winning on the unprefixed input.
- A "Math 1 cheat-sheet" overlay (key binding: `Shift+M`) is a NICE-TO-HAVE.

**Which phase:** Phase 31 (CLI integration); pitfall first reported by Phase 32
human-UAT.

**Detection:** Manual UAT — task-driven test where the tester is asked "compute the
inverse of a 2x2 matrix" without being told the function name. If they cannot find
it within 60 seconds, Pitfall 13 is open.

---

### Pitfall 14: Cross-Platform Numerical Drift Breaks Determinism

**What goes wrong:**
Math 1's algorithms (Romberg integration, Laguerre root-finding) accumulate rounding
error across many iterations. Even with `rust_decimal` as the base type, some
operations route through f64 (e.g. `Decimal::powd` uses f64 internally) — and f64
results differ between x86 (Intel) and ARM (Apple Silicon, ARM64 Linux) at the last
bit. Within 1000 Romberg iterations, that last-bit difference compounds to ~10⁻⁸
visible divergence.

v2.2's 566-case accuracy suite tolerated this for built-ins (single-op tests, no
iteration). Math 1's iterative methods will FAIL on one platform and PASS on another.

**Warning sign:** macOS CI green, Ubuntu CI red on `integ_sin_0_to_pi` with a
`tolerance = 1e-10` assertion. Random which platform fails depending on iteration
count.

**Prevention:**
- Use **relative tolerance**, not absolute: `assert_relative_eq!(actual, expected,
  max_relative = 1e-7)` (approx crate or hand-rolled `HpNum::within_relative`).
- Document that Math 1 numerical-accuracy tolerance is 1e-7 (relative) — 6 of HP-41's
  10 digits guaranteed, last 4 platform-dependent.
- Avoid f64 wherever possible inside Math 1 ops; use `Decimal::powu` (integer power)
  in preference to `Decimal::powd`. Phase 28 audits the `rust_decimal::maths` API to
  list which functions go through f64.
- Pin the criterion benchmark to a single platform (ubuntu-latest); regression
  detection is per-platform.

**Which phase:** Phase 30 (numerical methods); pitfall surfaces first time tests
run in CI.

**Detection:** `tests/math1_accuracy.rs` uses a `RELATIVE_TOL: f64 = 1e-7` constant.
Three-OS CI catches platform drift on the first PR.

---

## Minor Pitfalls

### Pitfall 15: New `Op` Variants Drift Between Four Exhaustive Match Arms

**What goes wrong:**
CLAUDE.md documents: "Every new `Op` variant must appear in BOTH `dispatch()` in
`ops/mod.rs` AND `execute_op()` in `ops/program.rs` AND the `prgm_display.rs`
exhaustive match before any caller can compile." Math 1 adds ~40 variants. Each
must land in 4 places (the two prgm_display copies are mirror-pinned per CLAUDE.md).
Even with the compile-time enforcement, the four-way drift increases PR review load.

**Warning sign:** PR review comment "Did you update both `prgm_display.rs` copies?".

**Prevention:**
- Use the JSON-canonical pipeline (already established in v2.2 D-25.16): one JSON
  file `docs/hp41-math1-functions.json` is the source of truth; a code generator
  produces the four match arms.
- Until the code generator exists, a CI gate compares the variants of `Op` against
  the JSON catalog (mirror of `tests/function_matrix_parity.rs`).

**Which phase:** Phase 28 (framework — JSON catalog established before any
implementation).

**Detection:** `tests/math1_op_parity.rs` (new): every JSON entry has a matching
`Op` variant; every `Op::*` variant prefixed `Mat`, `C`, `V`, `P` has a JSON entry.

---

### Pitfall 16: Adding 40 Functions × N Tests Temporarily Drops Coverage

**What goes wrong:**
v2.2 raised the coverage gate atomically to 95 % (D-27.2). Math 1's ~40 new
functions will land Op-by-Op over Phase 29 / 30. If each new Op merges with 2 tests
but has 10+ branches (matrix dimension validation, complex-zero check, convergence
failure, etc.), the FIRST commit that lands a Math 1 op drops coverage below 95 %.
CI fails. Reviewer reverts. Progress stalls.

**Warning sign:** First Math-1 Op PR: CI red, coverage 94.6 %. Reviewer asks for
more tests; author adds them; PR grows to 800 LOC.

**Prevention:**
- Land Math 1 ops in TEST-FIRST commits: the test file appears in the same commit
  as the Op variant + implementation. This is the v2.2 phase-27 pattern (D-27.3 — no
  coverage padding, risk-weighted tests).
- Set per-Op test-count guidance: ≥ 5 tests per new Op (covers happy path, two error
  paths, two edge cases). Document in the v3.0 ROADMAP.
- Accept a temporary GATE LOWERING ONLY at a SINGLE pre-announced commit (e.g.
  "Phase 29 lands all matrix ops; coverage gate stays at 95 %"). NO casual lowering.

**Which phase:** Phase 29 onward (every Math-1 Op-adding phase).

**Detection:** `just coverage` per-PR.

---

### Pitfall 17: Float-Equality Assertions Fail Non-Deterministically

**What goes wrong:**
Existing v2.2 test pattern is `assert_eq!(actual, expected)` on `HpNum`. This works
for single-op tests because `rust_decimal` is exact. It does NOT work for matrix
operations where rounding accumulates. `matrix_inv_2x2` test that asserts
`actual_inv == known_inv` will fail at the last digit.

**Warning sign:** `assertion failed: left == right` showing identical-looking
decimals that differ at the 10th digit.

**Prevention:**
- Add `HpNum::within_relative(self, other: HpNum, tol: HpNum) -> bool` helper in
  `hp41-core/src/num.rs`. Document the tolerance levels (1e-9 for non-iterative,
  1e-7 for iterative).
- A macro `assert_hp_close!(actual, expected, tol)` for ergonomic test writing.
- Math 1 test files ONLY use the macro; never raw `assert_eq!` on iterated results.

**Which phase:** Phase 28 (helper added before Phase 29 tests are written).

**Detection:** Clippy lint OR a `tests/lint_math1_assertions.rs` that greps for
`assert_eq!.*Decimal\|assert_eq!.*HpNum` in `tests/math1_*.rs` and fails if found.

---

### Pitfall 18: Owner's Manual Provenance for Edge Cases

**What goes wrong:**
The 1981 HP-41 Math 1 Pac Owner's Manual is the authoritative spec for documented
behaviour. Some edge cases (e.g. SOLVE on `f(x) = 1/(x-2)`) are NOT in the manual —
the documented behaviour comes from community reverse-engineering of HP-41 hardware
on Mike Sebastian's calculator pages, the MoHPC forum, and Free42 source comments.
Citing these as "authoritative" requires care: they are MEDIUM-confidence sources,
not HIGH.

**Warning sign:** A PR adds a test case with no Owner's Manual citation. Reviewer
cannot confirm it's correct. Test merges as "best guess".

**Prevention:**
- Every Math-1 test docstring includes a citation: Owner's Manual page-and-example,
  Mike Sebastian URL, MoHPC thread, or Free42 source path. NO uncited assertions.
- Pull the Math 1 Owner's Manual scan from `hpmuseum.org` (public-domain redistribution
  permitted with attribution) and check it into `docs/references/` as PDF for offline
  reference.

**Which phase:** Phase 28 (research-prep reading list).

**Detection:** Manual PR review; a CI gate `tests/math1_citation_coverage.rs` that
greps for `// Manual:` or `// Source:` comments in every `math1_*.rs` test (at least
one citation per test function).

---

### Pitfall 19: Clean-Room Re-Derivation vs Free42 Source

**What goes wrong:**
Free42 (GPL) is the cleanest open-source HP-41 emulator and contains battle-tested
implementations of INTEG, SOLVE, PROOT. The temptation to "translate Free42's
integration.c to Rust" is strong. Doing so contaminates `hp41-core` with GPL code,
breaking the project's permissive-licensing position.

The project's stated policy (PROJECT.md L20): "HP-copyrighted ROM-Image-Redistribution
bleibt permanent ausgeschlossen — wir liefern BEHAVIORAL Emulation der dokumentierten
Funktionen (Owner's Manual als Verhaltens-Spec), nicht die ROM-Bytes." This protects
against HP claims but NOT against Free42 GPL contamination.

**Warning sign:** Code comment "ported from Free42 integration.c" in a Math 1 op
file. Or: an algorithm in `op_integ` is line-by-line identical to Free42's.

**Prevention:**
- Free42 source can be CONSULTED for behaviour verification ("does my output match
  Free42's on this input?") but never COPIED.
- Each numerical-method op file carries a header comment:
  `// Implementation re-derived from <Owner's Manual citation>. NOT copied from Free42.`
- A `licensing.md` audit before the v3.0 PR merge: every Math-1 op file lists its
  source (Owner's Manual section / community page) and explicitly disclaims Free42.

**Which phase:** Phase 28 (policy); Phase 30 (highest contamination risk —
numerical methods).

**Detection:** Pre-merge legal review; `scripts/check-no-free42-strings.sh` greps
for distinctive Free42 variable names (`free42`, `integ_state` if it appears in
Free42, etc.) in `hp41-core/src/ops/math1/`.

---

### Pitfall 20: Module-Slot Semantics — Always-Loaded vs Slot-Empty

**What goes wrong:**
Real HP-41 hardware has 4 ROM slots. Math 1 is a physical chip plugged into a slot.
If the slot is empty, the function calls return `NONEXISTENT` error. v3.0's
statically-linked Math 1 is ALWAYS loaded → no NONEXISTENT path.

A user reading old HP-41 documentation will expect `XEQ "INV"` to return NONEXISTENT
when Math 1 is "removed" — but it always works in our emulator. Educational mismatch.

**Warning sign:** A user logs an issue "INV worked even though I disabled Math 1 in
settings, this isn't authentic." (Hypothetical, but a real category of user-feedback
trap.)

**Prevention:**
- v3.0 PHILOSOPHY: Math 1 is always loaded. Document this in `docs/v3.0-decisions.md`
  with the rationale (avoiding settings-driven feature gates that complicate testing).
- v3.x roadmap (post-v3.0): MAYBE add a "Modules: Math1 / Stat1 / ..." settings panel
  that simulates slot-empty by routing the Math 1 mnemonics to NONEXISTENT. NOT v3.0.
- If a user enables "authentic slot mode" in a future release, the emulator returns
  `NONEXISTENT` from the XROM table lookup BEFORE the actual function runs.

**Which phase:** Phase 28 (policy decision).

**Detection:** N/A — policy decision; no CI gate.

---

### Pitfall 21: New Tauri Permissions File for Each New Math-1 Command

**What goes wrong:**
v2.2 documented (CLAUDE.md): "Tauri v2.11 inline-command permissions need TOML in
`hp41-gui/src-tauri/permissions/<cmd-kebab>.toml`." Math 1 may need NEW commands
beyond `dispatch_op` — e.g. a `request_cancel` command (Pitfall 11) or a
`get_progress` command. Each needs a permissions TOML AND a `cargo check` to
regenerate the registry.

Forgetting a permission file means `invoke('request_cancel')` returns
`"Command not allowed"` at runtime, not a compile error.

**Warning sign:** Phase 32 GUI testing: R/S key during INTEG does nothing; DevTools
console: `"Command 'request_cancel' not allowed by capabilities"`.

**Prevention:**
- Pre-create permissions/ TOML files for every planned Math 1 command in Phase 28
  framework PR — even if the commands aren't implemented yet (empty TOML with TODO).
- A CI gate (`scripts/check-tauri-permissions.sh`) compares `generate_handler![...]`
  members against `permissions/*.toml` filenames.

**Which phase:** Phase 32 (GUI integration).

**Detection:** New script `scripts/check-tauri-permissions.sh` run in `just gui-ci`.

---

### Pitfall 22: Phase Numbering / `.planning/` Drift Across a Long Milestone

**What goes wrong:**
v3.0 will have ~5 phases (28–32). Each has its own subdirectory in
`.planning/milestones/v3.0-research/` and per-phase notes. If phase numbering drifts
(e.g. someone renames Phase 30 to Phase 30A and Phase 30B during execution), the
`PHASE_HISTORY.md` references break. v2.2 had this risk; the team handled it but it
cost time.

**Warning sign:** GSD `/gsd-progress` shows "phase 30" but `.planning/milestones/`
has subdirs `30A`, `30B`, `30C`.

**Prevention:**
- Phase 28 ROADMAP locks the phase numbers. Any in-flight split goes into a SUB-plan
  (e.g. `plan-30-04` instead of `phase-30A`).
- The phase boundary is a release boundary. Splitting Phase 30 into 30A/30B means
  shipping 30A first — usually the wrong call for a numerical-method phase.

**Which phase:** Phase 28 (process discipline).

**Detection:** N/A — process discipline.

---

## Phase-Specific Warning Matrix

| Phase | Topic | Highest-Risk Pitfall | Mitigation |
|-------|-------|----------------------|------------|
| 28: XROM framework | Resolver order | Built-in vs Math-1 name shadow (P1) | First-match-wins on built-in mnemonics |
| 28: XROM framework | Numbering | Hard-coded module ID `0x02` (P8) | Constants in `xrom/registry.rs` |
| 28: XROM framework | Dispatch | Enum bloat vs `XromCall(u16)` (P10) | ADR in Phase 28 research; codegen for matches |
| 28: XROM framework | Re-entrancy policy | `is_running` flag's nested-call behaviour (P4) | Pick strict / counter / save-restore in research |
| 28: XROM framework | Free42 contamination | Clean-room re-derivation (P19) | Header comment + audit script |
| 29: Matrix / Complex | Singularity | `INV` near-singular EPSILON (P7) | Source from Owner's Manual; named const |
| 29: Matrix / Complex | Branch cuts | `CARG(0,0)`, `CDIV` by zero (P6) | Explicit (0,0) and zero-divisor arms |
| 29: Matrix / Complex | FACT extension | v2.2 [0,69] vs Math1 gamma (P9) | Phase 28 decision; v2.2 regression test |
| 30: Numerical methods | INTEG threshold | Display-mode-tied convergence (P2) | Helper `integ_threshold(display_mode)` |
| 30: Numerical methods | SOLVE convention | No-root vs DATA-ERROR (P3) | Three named branches, manual-cited |
| 30: Numerical methods | PROOT clusters | Multiplicity-as-cluster convention (P5) | Document in divergence file |
| 30: Numerical methods | User callback | Solver state / nested INTEG (P4) | Save-restore scratch per Phase-28 policy |
| 30: Numerical methods | f64 drift | Cross-platform last-bit divergence (P14) | Relative tolerance `1e-7` |
| 30: Numerical methods | Determinism | `assert_eq!` on iterated HpNum (P17) | `assert_hp_close!` macro |
| 31: CLI integration | Discovery | XEQ-by-name modal scale (P13) | Add `CATALOG 2`; prefix-tolerant search |
| 31: CLI integration | Coverage | Mid-milestone gate drop (P16) | Test-first commits; ≥ 5 tests per Op |
| 31: CLI integration | Documentation | Citation provenance (P18) | Manual scan in `docs/references/` |
| 32: GUI integration | UI freeze | Long INTEG holds Mutex (P11) | Cancellation channel + lock release |
| 32: GUI integration | Save round-trip | Mid-solver state (P12) | `#[serde(default, skip)]` on all interim |
| 32: GUI integration | Tauri permissions | New commands missing TOML (P21) | Pre-create permission files |
| 32: GUI integration | Slot semantics | Always-loaded vs hardware (P20) | Document; defer to v3.x settings |
| 28+: cross-cutting | Match drift | Four-arm exhaustiveness (P15) | JSON-canonical codegen |
| 28+: cross-cutting | Process | Phase numbering drift (P22) | Lock numbers in ROADMAP |

---

## Sources

**Confidence: HIGH** (Owner's Manual / official HP)
- HP-41C/CV Owner's Handbook (1980, HP part 00041-90325) — built-in op semantics
- HP-41C Math/Stat Pac Owner's Manual (1981, HP part 82143A) — Math 1 algorithms;
  the authoritative spec for v3.0. Public-domain redistribution via hpmuseum.org.
- HP Journal April 1979, "Personal Programmable Calculator Routines" — Romberg
  integration scheme used in Math 1's INTEG.

**Confidence: MEDIUM** (community reverse-engineering, well-cross-referenced)
- Mike Sebastian's calculator pages: https://www.rskey.org/~mwsebastian/miscprj/forensics.htm
  (HP-41 forensic numerical accuracy comparisons; cross-platform drift data).
- Museum of HP Calculators (MoHPC) forum: https://www.hpmuseum.org/forum/
  (community discussions of Math 1 internals, scratch-register usage, edge-case
  behaviours not documented in the manual).
- HP-41 Synthetic Programming Made Easy (Wlodek Mier-Jędrzejowicz) — for the
  XROM-numbering and module-byte layout.

**Confidence: LOW** (single source, unverified — flag for in-phase research)
- Free42 source code (GPL, behaviour-verification only — DO NOT COPY):
  https://thomasokken.com/free42/ — useful as a sanity-check oracle for INTEG /
  SOLVE / PROOT outputs on test inputs, but every algorithm in v3.0 must be
  independently re-derived per Pitfall 19.
- Community-claimed EPSILON value `5e-10` for matrix singularity (Pitfall 7) —
  needs Phase 29 hardware verification or Owner's Manual page citation.

**Cross-referenced project context:**
- v2.0 PITFALLS template: `.planning/milestones/v2.0-research/PITFALLS.md`
- v2.2 settled invariants: `CLAUDE.md` (esp. `is_running` re-entrancy semantics,
  `#[serde(default, skip)]` discipline, SC-4 invariant, four-arm match discipline)
- v3.0 scope: `.planning/PROJECT.md` (Math 1 only; Stat 1 → v3.1)
- Re-entrancy ground truth: `hp41-core/src/ops/program.rs` lines 396–430
  (`run_program` and `run_loop_from_pc` is_running cleanup pattern).
