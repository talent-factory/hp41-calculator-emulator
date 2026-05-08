# Phase 6: Science & Engineering - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-07
**Phase:** 06-science-and-engineering
**Areas discussed:** Linear regression scope, TUI key bindings, HMS arithmetic edge cases, Σ register conflict behavior

---

## Linear Regression Scope

| Option | Description | Selected |
|--------|-------------|----------|
| L.R. only (slope + intercept) | Returns m in Y, b in X. Minimal scope, satisfies roadmap exactly. | |
| L.R. + CORR | Add correlation coefficient. Low cost since computed from same R01–R06. | |
| Full suite: L.R. + YHAT + CORR | Complete HP-41 stats row. | ✓ |

**User's choice:** Full suite — L.R. + YHAT + CORR

---

### CLΣSTAT inclusion

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — practically required | Without it, clearing stats needs 6 manual STO 0 ops. HP-41 has it. | ✓ |
| No — CLREG covers it | CLREG zeros all 100 regs including R01–R06. | |

**User's choice:** Yes — CLΣSTAT included

---

## TUI Key Bindings

| Option | Description | Selected |
|--------|-------------|----------|
| Memorable PC mnemonics | z=Σ+, m=MEAN, d=SDEV, h=HMS→, etc. Consistent with Phase 4 approach. | |
| Follow HP-41 key rows | Map keyboard rows to HP-41 physical positions (F1–F5 or number row). | ✓ |
| You decide (consistent with keys.rs) | Claude picks scheme following lowercase=primary, uppercase=secondary. | |

**User's choice:** Follow HP-41 key rows → then delegated specifics to Claude (consistent with keys.rs)

**Claude's discretion applied:** z/Z (Σ+/Σ−), m/d/y/R/O/V (stats), h/f/j/J (HMS)

---

## HMS Arithmetic Edge Cases

| Option | Description | Selected |
|--------|-------------|----------|
| Return HpError::InvalidInput | Matches HP-41 hardware. Consistent with other domain errors. | ✓ |
| Normalize silently | Treat overflow seconds/minutes as carry. Convenient but departs from HP-41. | |
| Accept any value, no validation | HMS→ is a bijection on arbitrary decimals. Simple. | |

**User's choice:** HpError::InvalidInput for seconds ≥ 60 or minutes ≥ 60

---

## Σ Register Conflict Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Match HP-41 exactly: silent overwrite | R01–R06 are dual-use. Faithful emulation. | ✓ |
| TUI warning in status bar | Non-blocking notice when STO overwrites Σ register. | |
| Block with HpError | Prevent accidental clobber. Departs significantly from HP-41 behavior. | |

**User's choice:** Silent overwrite — match HP-41 hardware exactly

---

## Claude's Discretion

- Specific key binding assignments (see CONTEXT.md D-09 table)
- HMS arithmetic internals: string-split approach (same as ISG/DSE counter extraction)
- MEAN/SDEV stack return semantics (X=x̄/σx, Y=ȳ/σy)
- Σ+/Σ− lift effect (Neutral during accumulation, Enable after pushing n)

## Deferred Ideas

None raised during discussion.
