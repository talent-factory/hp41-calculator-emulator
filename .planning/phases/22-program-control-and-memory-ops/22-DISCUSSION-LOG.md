# Phase 22: Program Control & Memory Ops — Discussion Log

**Date:** 2026-05-14
**Workflow:** `/gsd-discuss-phase 22` (default mode, no flags)
**Output:** `22-CONTEXT.md` (sibling file)

This log captures the question-by-question record of the discussion. It
is for human reference (audits, retrospectives) and is **NOT consumed by
downstream agents** (gsd-phase-researcher, gsd-planner, gsd-executor).

---

## Gray-area selection

**Question:** Which areas do you want to discuss for Phase 22?

**Options presented (multiSelect):**
1. STOP / PSE / R-S resume — interpreter control flow
2. CLP / DEL / INS — program-edit semantics
3. CATALOG output + SIZE register-count — memory ops
4. ASN keymapping model — hardware-style assignments vs existing
   Phase 5 map

**User selected:** ALL FOUR.

---

## Area 1 — STOP / PSE / R-S resume

### Q1.1 — STOP mechanism

**Question:** How should STOP halt run_loop and let R/S resume? (Phase 21's
PROMPT already breaks run_loop and leaves state.pc pointing to the NEXT
step — same pattern reusable here.)

**Options presented:**
- Break-with-state (mirror PROMPT) — *Recommended*
- `paused: bool` field + non-terminating run_loop
- RunState enum (Running/Paused/Stopped/Idle)

**User selected:** Break-with-state (mirror PROMPT) — *Recommended*

**Decision recorded:** D-22.1 — `Op::Stop` breaks run_loop, mirrors
Op::Prompt. state.pc points at next step. is_running reset to false.
R/S = new `resume_program()` entry point.

### Q1.2 — PSE encoding

**Question:** How should PSE encode its ~1 s pause for a buffer-only core?
(PSE = show X briefly, then continue — unlike STOP, PSE does NOT yield to
the user.)

**Options presented:**
- event_buffer marker, no run_loop break — *Recommended*
- Break-with-resume marker
- Synchronous display_override + frontend polls

**User selected:** event_buffer marker, no run_loop break — *Recommended*

**Decision recorded:** D-22.4 — PSE pushes `"PAUSE 1000"` to event_buffer
+ writes display_override. run_loop does NOT break. Frontend drains
event_buffer and inserts the 1 s delay.

### Q1.3 — STOP scope

**Question:** Where should `Op::Stop` actually do something? (FN-PROG-01:
"User can pause a running program (STOP — R/S inside program); execution
resumes on next R/S press".)

**Options presented:**
- Programs only — interactive Op::Stop is a no-op — *Recommended*
- Programs + interactive both meaningful

**User selected:** Programs only — interactive Op::Stop is a no-op —
*Recommended*

**Decision recorded:** D-22.5 — Op::Stop is run_loop-only. Interactive
dispatch = no-op + Neutral lift. R/S key has its own non-Op routing
calling resume_program().

### Q1.4 — Code location

**Question:** Where should `resume_program()` and `Op::Stop`'s run_loop
arm live in the source tree?

**Options presented:**
- All in hp41-core/src/ops/program.rs — *Recommended*
- New file: hp41-core/src/ops/run_control.rs
- Defer the decision to the planner

**User selected:** All in hp41-core/src/ops/program.rs — *Recommended*

**Decision recorded:** D-22.6 — resume_program() lives next to
run_program() in program.rs. Op::Stop arm next to Op::Prompt. No new
file.

---

## Area 2 — CLP / DEL / INS — program-edit semantics

### Q2.1 — CLP boundary

**Question:** How should `CLP "label"` decide where to stop deleting?
Real HP-41 deletes from LBL to the next END/.END. marker; our flat-Vec
model has no END markers.

**Options presented:**
- Delete to next Op::Lbl — *Recommended*
- Introduce Op::End / Op::ProgramEnd variant
- Delete to next LBL OR explicit Rtn boundary

**User selected:** Delete to next Op::Lbl — *Recommended*

**Decision recorded:** D-22.7 — CLP "label": find Op::Lbl(name); delete
from that index up to (but not including) the next Op::Lbl. If target is
the last labelled block, delete to end-of-Vec. Documented divergence
from HP-41 END/.END. semantics.

### Q2.2 — INS blank step

**Question:** What is a "blank step" inserted by `INS`?

**Options presented:**
- Reuse Op::Null — *Recommended*
- Add Op::BlankStep variant

**User selected:** Reuse Op::Null — *Recommended*

**Decision recorded:** D-22.8 — INS inserts Op::Null at state.pc. No new
variant. prgm_display already renders Op::Null as `"NULL"`.

### Q2.3 — DEL bounds

**Question:** How should `DEL nnn` handle out-of-bounds: nnn > remaining
steps from current PC, or nnn = 0?

**Options presented:**
- Clamp silently — delete min(nnn, len-pc); nnn=0 is no-op — *Recommended*
- Error on overflow — HpError::InvalidOp if nnn > remaining
- Error on overflow AND nnn=0

**User selected:** Clamp silently — *Recommended*

**Decision recorded:** D-22.9 — DEL nnn clamps silently. DEL 0 is no-op.
After deletion, leave state.pc at program.len() if it overflowed.

### Q2.4 — Edit-op scope

**Question:** Should `CLP/DEL/INS` only work in PRGM mode (interactive
program editing), or also when called from inside a running program?

**Options presented:**
- PRGM mode only — program-run dispatch returns InvalidOp — *Recommended*
- Allowed anywhere — both interactive and inside run_loop
- Allowed only when is_running=false (interactive only)

**User selected:** PRGM mode only — *Recommended*

**Decision recorded:** D-22.10 — CLP/DEL/INS execute only when
interactive AND prgm_mode == true. Inside run_loop = InvalidOp. They ARE
the program-editing primitives, so they do NOT self-record when
prgm_mode is on.

---

## Area 3 — CATALOG output + SIZE register-count

### Q3.1 — CATALOG format

**Question:** What line format should `CATALOG` emit into print_buffer?
Real HP-41 pages through display one entry at a time; ours dumps to
print_buffer (cli/gui both drain it). 24-char width matches existing
op_prx/op_pra conventions.

**Options presented:**
- Compact: `"R05  3.14000"` / `"LBL ALPHA  12"` — *Recommended*
- Verbose: full register dump including zeros
- Settle in plan-phase — leave both format and which catalogs are no-op
  open

**User selected:** Compact — *Recommended*

**Decision recorded:** D-22.16 — CATALOG header `-- CATALOG n --`, footer
`-- END --`, 24-char-wide payload lines. CATALOG 1: non-zero registers
only. CATALOG 2: LBL + step count. CATALOG 3/4: single `"NOT AVAILABLE"`
line.

### Q3.2 — SIZE semantics

**Question:** What should `SIZE nnn` actually do? Our `regs: Vec<HpNum>`
is currently fixed at 100. Real HP-41CV has 319 registers shared with
program memory; SIZE allocates between data and program.

**Options presented:**
- Resize regs (clamp to 1..=319) — *Recommended*
- Cosmetic only — Op::Size exists, dispatches cleanly, no effect
- Resize with smaller max (1..=100)

**User selected:** Resize regs (clamp to 1..=319) — *Recommended*

**Decision recorded:** D-22.11 — SIZE resizes regs with clamp to
[1, 319]. Shrinking truncates. Growing zero-fills. Out-of-bound → 
InvalidOp. Bounds audit of existing op_sto/op_rcl/op_clreg becomes a
Wave-0 prep task.

### Q3.3 — PACK & clear ops

**Question:** PACK and CLA/CLST: any nuance, or just confirm the
straightforward path?

**Options presented:**
- Straightforward path — *Recommended*
- Make Op::AlphaClear pull double duty as CLA

**User selected:** Straightforward path — *Recommended*

**Decisions recorded:** 
- D-22.12 — PACK is a no-op (flat-Vec has no gaps).
- D-22.13 — NEW Op::Cla variant calls op_alpha_clear; Op::AlphaClear
  kept for v1.0 save-file compat.
- D-22.14 — NEW Op::Clst zeros X/Y/Z/T; preserves lastx and
  lift_enabled.

### Q3.4 — GTO IND / XEQ IND

**Question:** GTO IND / XEQ IND: Phase 22 lands the DIRECT-form ops, but
the real indirect-resolver is Phase 24. What's the right Phase 22 shape?

**Options presented:**
- New variants `Op::GtoInd(u8)` / `Op::XeqInd(u8)` + minimal inline
  resolver — *Recommended*
- Phase 22 adds ONLY the Op enum variants + dispatch arms; bodies are
  stubs that return HpError::InvalidOp until Phase 24

**User selected:** New variants + minimal inline resolver — *Recommended*

**Decision recorded:** D-22.15 — Op::GtoInd(u8), Op::XeqInd(u8) ship with
an inline integer-part check (5 steps: read reg, integer check,
stringify, find_in_program, set pc). Phase 24 extracts into a shared
resolve_indirect() helper.

---

## Area 4 — ASN keymapping model

### Q4.1 — Coexistence with Phase 5 key_assignments

**Question:** How should the new ASN-style assignments (key_code u8 →
target) coexist with the existing Phase 5 `key_assignments:
BTreeMap<char, String>` (which maps keyboard char → label, wired into
hp41-cli USER mode)?

**Options presented:**
- Add NEW field `assignments: BTreeMap<u8, String>` alongside existing
  key_assignments — *Recommended*
- Replace key_assignments with the new u8-keyed map; migrate Phase 5
  wiring now
- Two maps with a unified lookup helper

**User selected:** Add NEW field alongside existing — *Recommended*

**Decision recorded:** D-22.17 — NEW `assignments: BTreeMap<u8, String>`
field with #[serde(default)]. Coexists with Phase 5's key_assignments.
Phase 25/26 will reconcile.

### Q4.2 — ASN target type

**Question:** What's the value type for an `ASN "NAME" key_code`
assignment? FN-KEY-01's example is `ASN "SIN" 11` — SIN is an Op, but the
user could equally write a LABEL name (like `ASN "MYPRG" 11`).

**Options presented:**
- String only — resolved at dispatch time — *Recommended*
- Enum `AsnTarget { Op(Op), Label(String) }` resolved at ASN time

**User selected:** String only — late binding — *Recommended*

**Decision recorded:** D-22.19 — assignments map value is plain String.
USER-mode dispatch resolves by parse-as-Op first, fallback to LBL search
via run_program. Resolution lives in the frontend (Phase 25/26), not
core.

### Q4.3 — Op::Asn flow

**Question:** Op::Asn execution semantics: it's a 2-argument op (NAME +
key_code). How should `dispatch(Op::Asn { name, key_code })` actually
work — fully resolved by the caller, or with a keyboard-modal flow?

**Options presented:**
- Core: fully-formed Op::Asn { name, key_code }; modal flow is
  frontend-only — *Recommended*
- Core: Op::Asn(()) is a marker; uses a pending_asn state field for
  two-step assembly

**User selected:** Fully-formed in core; modal flow frontend-only —
*Recommended*

**Decision recorded:** D-22.18 — `Op::Asn { name: String, key_code: u8 }`
struct variant. dispatch() = assignments.insert + Neutral lift. The 2-
step modal is hp41-cli's PendingInput::AsnNamePrompt → AsnKeyPrompt in
Phase 25.

### Q4.4 — Plan structure

**Question:** Final settling question: plan structure for Phase 22 (13
requirements, ~14 new Op variants, touches run_loop). Phase 20 used 1
plan; Phase 21 used 4 plans by domain.

**Options presented:**
- 4 plans by domain — *Recommended*
- 3 plans — fold catalog into memory-ops
- 2 plans — program-side vs memory-side
- Defer the split to the planner

**User selected:** 4 plans by domain — *Recommended*

**Decision recorded:** D-22.20 — 4 plans:
- 22-01-program-control: Stop, Pse, resume_program, GtoInd, XeqInd
- 22-02-program-edit: Clp, Del, Ins
- 22-03-memory-ops: Size, Cla, Clst, Pack
- 22-04-catalog-and-asn: Catalog, Asn (+ assignments field)

---

## Deferred ideas surfaced during discussion

- Stop-from-keyboard while program is running — needs a
  `state.stop_requested: bool` sentinel. Phase 25 wiring concern.
- Real END / .END. marker — would close CLP boundary divergence. v3.0
  backlog.
- PACK doing something real — moot until we introduce gaps in the
  program Vec.
- CATALOG paging UX — real HP-41 advances per R/S; our bulk-dump
  differs. Phase 26 GUI polish candidate.
- AsnTarget enum (Op | Label) — type-safe alternative to String.
  Revisit in v3.0 if dispatch-time resolution proves error-prone.
- Reconciling `assignments` (u8) and `key_assignments` (char) into one
  canonical map — Phase 25/26.
- SIZE memory model fidelity (shared program/register memory) — v3.0
  Module Emulation milestone.

---

## Scope creep redirected

No scope-creep requests during this session.

---

*Discussion log written: 2026-05-14*
