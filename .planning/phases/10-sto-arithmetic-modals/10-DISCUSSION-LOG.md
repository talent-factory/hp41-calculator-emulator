# Phase 10: STO Arithmetic Modals - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-08
**Phase:** 10-STO Arithmetic Modals
**Areas discussed:** Stack register design, Step 1 display ambiguity, Help overlay key descriptions

---

## Stack Register Design (STOA-03)

| Option | Description | Selected |
|--------|-------------|----------|
| New Op variant | `Op::StoArithStack { kind, stack_reg: StackReg }` + `op_sto_arith_stack()` in hp41-core — cleanest, follows StoArithKind enum pattern | ✓ |
| Extend Op::StoArith with special reg codes | Reserve reg 100=Y, 101=Z, 102=T, 103=LASTX in u8 space — simpler but leaks encoding into register space | |
| Handle in app.rs (no core changes) | Compute directly on state.stack fields in app.rs — no Op variant, contradicts core-purity pattern | |

**User's choice:** New Op variant

### Follow-up: StackReg enum design

| Option | Description | Selected |
|--------|-------------|----------|
| New StackReg enum (Y/Z/T/LastX) | `pub enum StackReg { Y, Z, T, LastX }` alongside StoArithKind in ops/mod.rs — clearly typed | ✓ |
| Reuse existing patterns — you decide | Let planner decide exact enum shape | |

**User's choice:** New StackReg enum (Y/Z/T/LastX)

**Notes:** None — decision was clear from the options.

---

## Step 1 Display Ambiguity

| Option | Description | Selected |
|--------|-------------|----------|
| Keep `STO [__]` | Simple, consistent with existing STO modal style; `?` overlay is the discovery path | ✓ |
| Show `STO —` | Dash instead of register slots to signal 'expecting input, not necessarily a number yet' | |
| Show `STO +/-/nn` | Explicitly hints both arithmetic and register paths — breaks uniform display progression | |

**User's choice:** Keep `STO [__]`

**Notes:** No change to `ui.rs` display strings for `StoRegister` state.

---

## Help Overlay Key Descriptions

| Option | Description | Selected |
|--------|-------------|----------|
| "S +" / "S -" / "S *" / "S /" | Two-token key column showing both steps; consistent with `S` entry for plain STO | ✓ |
| "S,+" / "S,-" etc. | Comma-separated sequence — slightly more explicit | |
| "S" for all four | Same key column as STO [nn]; distinguish only in function name | |

**User's choice:** "S +" / "S -" / "S *" / "S /"

**Notes:** Description field to include stack register options (Y/Z/T/L) so users know the full input space.

---

## Claude's Discretion

None — all areas had clear user selections.

## Deferred Ideas

None — discussion stayed within phase scope.
