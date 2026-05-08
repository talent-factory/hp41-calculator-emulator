# Phase 3: Programming Engine - Context

**Gathered:** 2026-05-07
**Status:** Ready for planning
**Mode:** Auto (autonomous)

<domain>
## Phase Boundary

Users can record keystroke programs in PRGM mode and execute them with labels, unconditional branches, subroutine calls, conditional tests, and ISG/DSE loop control — all with HP-41-hardware-identical semantics including 4-level call stack depth limit and string-split ISG/DSE counter field parsing.

**Deliverables:**
- `CalcState` gains: `program: Vec<Op>`, `prgm_mode: bool`, `pc: usize`, `call_stack: Vec<usize>`, `is_running: bool`
- New `Op` variants: `Lbl(String)`, `Gto(String)`, `Xeq(String)`, `Rtn`, `PrgmMode`, `Test(TestKind)`, `Isg(u8)`, `Dse(u8)`
- `TestKind` enum with 12 HP-41 conditional tests
- `HpError::CallDepth` ("try again") for 5th-level XEQ
- `run_program(state, label)` public function
- `dispatch()` extended: records ops to program when `prgm_mode = true` instead of executing
- PROG-01 and PROG-02 test suites

</domain>

<decisions>
## Implementation Decisions

### Program Storage Model
- **D-01:** `program: Vec<Op>` flat list in `CalcState` — single contiguous program memory matching HP-41 hardware. Labels are `Op::Lbl(String)` markers inside the Vec; programs are delimited by LBL and RTN.
- **D-02:** Linear label search: `run_program` scans `state.program` for `Op::Lbl(target)` to find entry points. HP-41 hardware does equivalent linear scan.

### PRGM Mode Recording in dispatch()
- **D-03:** `dispatch()` checks `state.prgm_mode` at the top. When true: `flush_entry_buf` records a `Op::PushNum` to `state.program` (instead of pushing to stack), then the current Op is appended to `state.program`, and `dispatch()` returns `Ok(())` without executing. When false: normal execution path unchanged.
- **D-04:** `flush_entry_buf` is extended to accept a `target: &mut Vec<Op>` or checks `prgm_mode` internally to route the PushNum to either stack (execute mode) or program (record mode).

### Execution State in CalcState
- **D-05:** Three fields added to `CalcState`: `pc: usize` (current program counter, 0 at startup), `call_stack: Vec<usize>` (return address stack, max 4 entries), `is_running: bool` (true during `run_program` execution).
- **D-06:** `run_program(state: &mut CalcState, entry_label: &str) -> Result<(), HpError>` sets `state.is_running = true`, finds the label, sets `state.pc`, runs the interpreter loop until RTN from top-level call stack (or end of program), then resets `is_running = false`.

### Conditional Test Representation
- **D-07:** `Op::Test(TestKind)` — single enum covers all 12 HP-41 conditional tests. Cleaner than 12 individual Op variants and symmetric with how StoArith is handled.
- **D-08:** `TestKind` variants: `XEqZero`, `XNeZero`, `XLtZero`, `XGtZero`, `XLeZero`, `XGeZero`, `XEqY`, `XNeY`, `XLtY`, `XGtY`, `XLeY`, `XGeY`.
- **D-09:** Execution semantics: if condition is **TRUE** → execute next step (pc += 1 normally); if condition is **FALSE** → skip next step (pc += 2). This matches HP-41 hardware "skip if false" behavior.

### ISG/DSE Counter Parsing
- **D-10:** Counter format `CCCCC.FFFDD` parsed by string-splitting at decimal point — never `floor()`/`fmod()` on f64 (ADR-001 confirms this). Left of decimal = current count (i64). Right of decimal (padded to 5 chars): first 3 digits = final count, last 2 digits = step (00 → step of 1).
- **D-11:** ISG semantics: increment current by step; if `new_current > final` → skip next (loop exits); else → execute next (loop continues). DSE semantics: decrement; if `new_current <= final` → skip next; else → execute next.
- **D-12:** Counter is stored back to the register as a new `HpNum` constructed from the updated `CCCCC.FFF` (step/DD field is dropped when storing back — HP-41 preserves it, so we keep the full `CCCCC.FFFDD` format with updated CCCCC).

### Call Stack Depth
- **D-13:** `HpError::CallDepth` added to `error.rs`. Message: `"try again"`. Returned when a 5th XEQ is attempted (`call_stack.len() == 4` already).
- **D-14:** `call_stack` max depth: 4 entries (matches HP-41 hardware 4-level return stack). Each entry is a `usize` (return PC = index of the step after XEQ).

### Claude's Discretion
- Stack-lift semantics for new ops: LBL, GTO, XEQ, RTN, PrgmMode = Neutral; Test = Neutral; ISG/DSE = Neutral
- `Op::Rtn` at top-level call (empty call_stack): terminates execution normally (does not error)
- GTO to an unknown label → `HpError::InvalidOp`
- XEQ to an unknown label → `HpError::InvalidOp`
- When `is_running = false`, GTO/XEQ/RTN dispatched in non-prgm mode execute normally (RTN is a no-op or error in run context only)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Architecture Decisions
- `hp41-core/src/state.rs` — ADR-001 (numeric representation, ISG/DSE string-split rule); CalcState and Stack definitions to extend
- `hp41-core/src/error.rs` — existing HpError enum; add `CallDepth` variant here
- `hp41-core/src/ops/mod.rs` — Op enum + dispatch() to extend; PRGM mode gate goes at top of dispatch()
- `.planning/STATE.md` — key decisions from prior phases (stack-lift semantics, no panics, single &mut CalcState)

### Phase Requirements
- `.planning/REQUIREMENTS.md` §PROG-01, §PROG-02 — what must be implemented
- `.planning/ROADMAP.md` §Phase 3 — success criteria (verbatim truth statements for tests)

### HP-41 Programming Reference
- No external HP-41 docs in repo — use success criteria in ROADMAP.md as authoritative behavioral spec. ISG counter format `1.00500` (current=1, final=5, step=1) must increment 4 times before skip.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `HpNum::inner()` → `Decimal` — needed for ISG/DSE counter string parsing
- `HpError` enum in `error.rs` — extend with `CallDepth`
- `Op` enum + `dispatch()` in `ops/mod.rs` — PRGM mode gate and new variants extend this directly
- `flush_entry_buf` in `ops/mod.rs` — must route PushNum to `state.program` when `prgm_mode`
- `apply_lift_effect` in `stack.rs` — all new ops declare LiftEffect (all Neutral for programming ops)

### Established Patterns
- All ops return `Result<(), HpError>` — no panics, no unwrap in non-test code
- `dispatch()` is the single entry point — no direct calls to op functions from outside hp41-core
- `CalcState` is the single `&mut CalcState` — no global mutable state
- Submodule per op family: add `ops/program.rs` for LBL/GTO/XEQ/RTN/Test/ISG/DSE implementation

### Integration Points
- `CalcState` in `state.rs`: add `program`, `prgm_mode`, `pc`, `call_stack`, `is_running`
- `ops/mod.rs`: add Op variants, extend dispatch() with prgm_mode gate and new arms
- New `ops/program.rs` submodule: `op_lbl`, `op_gto`, `op_xeq`, `op_rtn`, `op_test`, `op_isg`, `op_dse`
- `lib.rs`: export `run_program` as public API
- New integration test file: `hp41-core/tests/program_tests.rs`

</code_context>

<specifics>
## Specific Ideas

- ISG counter `1.00500` → current=1, final=5, step=00→1. Loop runs 4 times before skip — this exact case is the success criterion and must be a named test.
- XEQ nesting: 4 levels allowed, 5th returns `HpError::CallDepth` — exact depth limit is 4.
- Conditional tests: HP-41 uses "skip next step if false" — when condition is false, the typically-following GTO is skipped (not the test instruction itself).
- `prgm_mode` is orthogonal to `alpha_mode` — both can be tracked as independent bools in CalcState.

</specifics>

<deferred>
## Deferred Ideas

- `Op::Sto0` / indirect addressing (`STO IND`, `GTO IND`) — HP-41 feature, deferred to v1.1
- Step-number display in PRGM mode (showing "001 LBL A") — TUI concern, Phase 4
- SST/BST (single-step/back-step) navigation — TUI keyboard concern, Phase 4
- R/S (run/stop) key handling — TUI keyboard concern, Phase 4
- Program checksum/size display — Phase 5 persistence concern
- Synthetic programming — explicitly out of scope per PROJECT.md

</deferred>

---

*Phase: 3-Programming-Engine*
*Context gathered: 2026-05-07*
