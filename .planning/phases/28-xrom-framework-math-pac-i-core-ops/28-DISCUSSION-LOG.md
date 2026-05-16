# Phase 28: XROM Framework + Math Pac I Core Ops - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-16
**Phase:** 28-xrom-framework-math-pac-i-core-ops
**Areas discussed:** ComplexStack location, Modal-prompt channel, Hyperbolics UX policy, Cancellation field timing

---

## ComplexStack location (CMPLX-01)

### Q1 — Storage representation

| Option | Description | Selected |
|--------|-------------|----------|
| Overlay X/Y/Z/T (ζ = X+iY, τ = Z+iT) | OM-treu; zero new HpNum storage fields; just `complex_mode: bool`. SUMMARY.md §"Complex Stack & Operations" lines 66–68 canonicalizes this. | ✓ |
| Dedicated R02–R05 registers | Uses general registers; stack untouched; matches MATRIX register convention. Risk: user `STO 02` silently clobbers complex stack. | |
| New `ComplexStack { zeta, tau }` struct | Cleanest separation; `#[serde(default)]`. Cost: +4 HpNums per save-file; ABWEICHT vom OM-Modell. | |

**User's choice:** Overlay X/Y/Z/T (Recommended)
**Notes:** OM-faithful, zero save-file growth, no clobbering surface. Number-entry needs `complex_mode` awareness in `flush_entry_buf()` — contained scope. → D-28.1

### Q2 — Complex_mode activation/deactivation

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-on at first complex-op, explicit `XEQ "REAL"` off | First C+/C-/C×/C÷/MAGZ/Z↑N etc. sets `complex_mode = true`; display annunciator hint; save-file default false. | ✓ |
| Always-on while Math Pac I loaded | `xrom_modules & 0b1 != 0` → X/Y/Z/T always complex. Risk: no pure-real arithmetic without unloading XROM. | |
| Per-op transient (no flag) | Stateless; each complex op reads X/Y/Z/T as complex. Risk: Display14Seg can't render annunciator without state bit. | |

**User's choice:** Auto-on first complex-op, explicit XEQ "REAL" off (Recommended)
**Notes:** Matches v2.2 `shift_armed` / `eex_mode` implicit-state-machine pattern. XEQ "REAL" is NEW — not in OM 1979; will be documented as our extension. → D-28.2, D-28.3 (new requirement to add: `CMPLX-18`)

---

## Modal-prompt channel (XROM-09 supersedes)

### Q3 — Which channel carries `ORDER=?` / `A1,1=?` / `FUNCTION NAME?` prompts

| Option | Description | Selected |
|--------|-------------|----------|
| New `modal_prompt: Option<String>` field `#[serde(skip)]` | Dedicated field; CLI renders in `pending_prompt()` line; GUI renders as overlay banner. `print_buffer` stays clean for PRX/PRA output. | ✓ |
| `print_buffer` (REQUIREMENTS XROM-09 default) | Existing channel; zero new fields. Risk: mixes with real PRX/PRA output; after STOP/PSE prompt stuck in scrollback. | |
| LCD direct overwrite via `display_override: Option<String>` | Restores after modal ends; hardware-faithful. Risk: 12-char limit truncates "FUNCTION NAME?" (14 chars) — would require scroll plumbing. | |

**User's choice:** New `modal_prompt` field (Recommended)
**Notes:** Clean separation; transient (`#[serde(skip)]`); REQUIREMENTS XROM-09 wording overridden — to be documented as deviation. CLI/GUI wiring deferred to Phase 29/31. → D-28.4

### Q4 — Modal-prompt numeric-input submit key

| Option | Description | Selected |
|--------|-------------|----------|
| R/S key submits (HP-41 hardware-faithful) | OM 1979 p.13: "Press R/S to continue." Reuses v2.1 `run_stop` Tauri command. ENTER would conflict with MATRIX-Edit stack-push semantics. | ✓ |
| ENTER key submits | Stack-push → modal reads new X-value → advances. Familiar for non-HP-41 users; conflicts with RPN convention in MATRIX. | |
| Auto-advance on complete input | 300ms idle timeout OR 2-digit cap. Risk: timing-dependent tests; user uncertain when auto-fire happens. | |

**User's choice:** R/S key submits (Recommended)
**Notes:** OM 1979 p.13 ground truth; reuses existing run_stop wiring. → D-28.5

---

## Hyperbolics UX policy (HYP-01..06)

### Q5 — Key binding strategy for hyperbolic functions

| Option | Description | Selected |
|--------|-------------|----------|
| XEQ-by-name only | Konsistent mit Rest von Math Pac I (MATRIX, SOLVE, POLY, etc.). User flow: `XEQ ALPHA S I N H ALPHA`. Echtes HP-41C Math Pac I verhält sich genau so. | ✓ |
| f-prefix on SIN/COS/TAN (f+SIN → SINH) | Ergonomic; matches v2.2 ROM built-in pattern. Conflict: f+SIN already wired in v2.2 for SIN⁻¹. | |
| New h-prefix (h+SIN = SINH) | Avoids f-conflict. Divergiert vom HP-41 hardware; verletzt behavioral emulation scope-boundary. | |

**User's choice:** XEQ-by-name only (Recommended)
**Notes:** Real HP-41C with Math Pac I in slot also has no hyp keys — confirmed by user's direct hardware inspection. Plan 28-02 still ships hyperbolics first as proof-of-pattern, reached via resolver chain only. → D-28.6

---

## Cancellation field timing (Pitfall 11)

### Q6 — When does `state.cancel_requested: Arc<AtomicBool>` get plumbed

| Option | Description | Selected |
|--------|-------------|----------|
| Plumbing in Phase 28, wiring in Phase 31 | Field + per-64-samples check land in Plans 28-01/07/08/09. Phase 31 only adds Tauri command + UI button. | ✓ |
| Strict Phase 31 (field AND wiring) | Phase 28 INTG/SOLVE laufen unkillbar. Phase 31 re-edits 3 hp41-core files. Violates "Op variants land before consumers" pattern. | |
| Full cancellation (incl. Tauri cmd) in Phase 28 | Mischt core- und gui-Phasen; verletzt phase boundary. | |

**User's choice:** Plumbing in Phase 28, Wiring in Phase 31 (Recommended)
**Notes:** "Op variants land before consumers" pattern preserved. Per-64-samples cadence per D-28.8; new `HpError::Canceled` variant per D-28.9. → D-28.7, D-28.8, D-28.9

---

## Claude's Discretion

- Number-entry semantics in `complex_mode` (3.5 ENTER 2 → ζ with re=3.5, im=2 per OM convention)
- Stack-lift for complex ops (C+ consumes ζ+τ, T-replicates per OM)
- `integ_state` / `solve_state` struct shape (planner picks from OM-cited algorithm)
- `ModalProgram` enum variant layout (one per top-level program, sub-enums per step)
- POLY multiplicity-as-cluster (no snap-to-zero; documented divergence)
- Triangle SSA ambiguous-case rendering
- Free42 audit policy strictness beyond QUAL-05 baseline

## Deferred Ideas

- GUI cancellation UI (Phase 31 / GUI-05)
- 5 ADR write-ups (Phase 30 / DOC-07)
- CLI modal-prompt rendering (Phase 29 / CLI-05)
- GUI modal-prompt overlay (Phase 31 / GUI-06)
- `hp41-math1-functions.json` + matrix + divergences doc (Phase 30 / DOC-01..04)
- Free42-contamination audit script (Phase 32 / QUAL-05)
- Cross-platform numerical drift assertions (Phase 32 / QUAL-06)
- Math-Pac-I E2E smoke (Phase 32 / QUAL-03)
- CATALOG 2 XROM module listing (Phase 31 / GUI-04)
