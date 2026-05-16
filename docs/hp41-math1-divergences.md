# HP-41C Math Pac I Emulator Divergences

This document lists known behavioral divergences between this emulator's implementation
of the HP-41C Math Pac I module and the hardware-faithful behavior described in the
HP-41C Math Pac I Owner's Manual (HP 00041-90034, 1979).

**Status:** First entry added in Plan 28-07. Phase 30 / DOC-04 will expand this
document with a comprehensive divergence catalog.

**Philosophy:** Where divergences exist, this emulator prioritizes:
1. Hardware-faithful behavior where feasible.
2. User-safety (no silent data corruption without documentation).
3. Clear documentation of known divergences.

---

## Divergence 1: User-Program Scratch Register Clobber During INTG/SOLVE/DIFEQ

**Category:** User responsibility / Hardware-faithful

**Status:** Documented — behavior matches HP-41C hardware (NO mitigation).

**Affected operations:** INTG, SOLVE (Plan 28-08), DIFEQ (Plan 28-09)

### Description

Math Pac I uses registers R00–R07 as scratch storage during integration (INTG),
root-finding (SOLVE), and ODE solving (DIFEQ). If the user-provided LBL function
executes `STO` to any of these registers (R00–R07), the solver's internal state
may be corrupted and the result will be wrong.

### Hardware-Faithful Behavior

The emulator faithfully reproduces this hardware behavior — **NO snapshot/restore**
of R00–R07 is performed before/after the user function is called.

This matches the real HP-41C Math Pac I hardware, which also uses these registers
as scratch without protecting them from the user callback.

### OM Reference

HP-41C Math Pac I Owner's Manual (HP 00041-90034, 1979), Chapter 3, p. 35:

> "Important: The INTG program uses registers R00 through R07. Do not use these
> registers in your user function while INTG is active, or the integration
> result will be incorrect."

*(Note: The exact page reference and OM wording should be verified against a
physical copy. The above is a reconstruction from context. Phase 30 / DOC-04
will add verified citations.)*

### Test Coverage

- `hp41-core/tests/math1_user_callback.rs::user_fn_stores_to_scratch_corrupts_integ`

This test explicitly asserts the **wrong-answer** behavior (not an error) when the
user function clobbers a scratch register. The test documentation makes it clear
that this is a user-responsibility divergence, not a bug.

### Impact Assessment

**User impact:** Users who write INTG/SOLVE/DIFEQ callback functions that store to
R00–R07 will get wrong numerical results. No error is raised — the result is simply
incorrect.

**Emulator behavior:** Identical to HP-41C hardware (user must manage register usage).

**Mitigation options considered and rejected:**
- Snapshot/restore R00–R07 around each callback: would diverge from hardware behavior
  and add overhead. Rejected per ADR (Phase 30 / DOC-04 will document this decision).
- Error on STO inside callback: cannot detect this without significant overhead
  and would reject valid user programs. Rejected.
- Documentation only (this file): accepted as the appropriate mitigation.

---

*Last updated: 2026-05-17 (Plan 28-07)*
*Next planned update: Phase 30 / DOC-04 (full divergence catalog)*
