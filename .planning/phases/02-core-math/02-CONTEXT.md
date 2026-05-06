# Phase 2: Core Math - Context

**Gathered:** 2026-05-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can perform the complete HP-41 arithmetic, trigonometric, and formatting operation set, store and recall values in R00–R99 registers, and enter alphanumeric strings in ALPHA mode — all with HP-41-accurate 10-digit results.

**Deliverables:**
- Extended `Op` enum: 1/x, √x, x², Y^X, LN, LOG, e^x, 10^x, SIN/COS/TAN + inverses
- `AngleMode` enum (DEG/RAD/GRAD) in `CalcState`
- `DisplayMode` enum (Fix/Sci/Eng with digit count) in `CalcState`
- String accumulation buffer (`entry_buf`) for digit-by-digit number entry
- `[HpNum; 100]` storage registers (R00–R99) in `CalcState`
- STO/RCL ops + `Op::StoArith { reg, kind }` for STO+/−/×/÷
- `alpha_reg: String` (24-char max) + `alpha_mode: bool` in `CalcState`
- `Op::AlphaAppend(char)` and `Op::AlphaClear` for ALPHA entry

</domain>

<decisions>
## Implementation Decisions

### Scientific Math Implementation
- Use `rust_decimal` with `features = ["maths"]` for ln/exp/pow/trig (ADR-001 specifies this for Phase 2)
- `AngleMode` enum (DEG/RAD/GRAD) stored as field in `CalcState`
- Default angle mode on startup: DEG (HP-41 hardware default)
- Domain errors (log(-1), sqrt(-1), asin(2)) return `Err(HpError::Domain)` — already exists in error.rs

### Display Formatting
- `DisplayMode` enum in `CalcState`: `Fix(u8)`, `Sci(u8)`, `Eng(u8)` where u8 = digit count (0–9)
- Number entry via string accumulation buffer (`entry_buf: String`) in `CalcState`; committed to `HpNum` on Enter or operation
- Show trailing zeros in FIX mode (e.g., FIX 4 of `1` → `1.0000`) — HP-41 hardware behavior
- Uppercase E-notation in SCI mode (`1.23456789E-10`)

### Storage Registers
- `[HpNum; 100]` fixed array in `CalcState` (R00–R99, 0-indexed)
- STO arithmetic: `Op::StoArith { reg: u8, kind: StoArithKind }` with `StoArithKind` enum (Add/Sub/Mul/Div)
- Out-of-range register address returns `Err(HpError::InvalidOp)`
- All registers initialized to zero on startup (HP-41 hardware cold-start behavior)

### ALPHA Mode
- `alpha_reg: String` in `CalcState` with 24-character maximum enforced
- Append mode — each `Op::AlphaAppend(char)` appends to `alpha_reg` (HP-41 hardware behavior)
- `alpha_mode: bool` field in `CalcState` (simple flag; `CalcMode` enum deferred to Phase 3 when PRGM mode is added)
- Number-to-ALPHA conversion (`Op::AlphaFromX`) deferred — Phase 2 covers raw char entry only

### Claude's Discretion
- Internal helper for degree↔radian conversion inside trig dispatch
- `Op::FmtFix(u8)`, `Op::FmtSci(u8)`, `Op::FmtEng(u8)` variants for mode-switching ops
- Stack-lift semantics for all new ops must declare Enable/Disable/Neutral as per Phase 1 convention

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `HpNum` newtype with `checked_add/sub/mul/div`, `rounded()`, `zero()`, `inner()` — extend with new math methods
- `HpError` enum: `Overflow`, `DivideByZero`, `InvalidOp`, `Domain` — all needed variants already exist
- `Op` enum + `dispatch()` pattern in `ops/mod.rs` — Phase 2 extends both without changing the interface
- `Stack` with `lift_enabled` flag — all new ops must declare lift effect

### Established Patterns
- All arithmetic flows through `HpNum` checked methods → `HpNum::rounded()`
- Operations return `Result<(), HpError>` — no panics
- Phase 2 adds `features = ["maths"]` to `rust_decimal` in `hp41-core/Cargo.toml` (ADR-001)
- `CalcState` is the single `&mut CalcState` passed to every op

### Integration Points
- `CalcState` in `state.rs` gains: `regs`, `alpha_reg`, `alpha_mode`, `angle_mode`, `display_mode`, `entry_buf`
- `ops/mod.rs` `Op` enum gains ~30 new variants; `dispatch()` match arm grows accordingly
- New ops submodule: `ops/math.rs` (trig/log/exp), `ops/registers.rs` (STO/RCL), `ops/alpha.rs` (ALPHA ops)

</code_context>

<specifics>
## Specific Ideas

- ADR-001 explicitly calls out: "Phase 2 adds `features = ["maths"]` to rust_decimal for ln/exp/pow"
- All STO operations must correctly set stack-lift: STO and STO+/−/×/÷ are Neutral (don't touch lift flag); RCL is Enable
- `entry_buf` flush: when a non-digit op arrives while `entry_buf` is non-empty, parse and push to stack first, then execute op
- `x²` can be implemented as `x * x` through existing `checked_mul` — no extra math feature needed

</specifics>

<deferred>
## Deferred Ideas

- `Op::AlphaFromX` (append X register as string to alpha_reg) — Phase 2 scope is raw char entry only; numeric→alpha conversion can come in Phase 5 UX
- `CalcMode` enum (Normal/Alpha/Prgm) — `alpha_mode: bool` suffices for Phase 2; the unified enum makes sense once Phase 3 adds PRGM mode
- ALPHA annunciator display — belongs in Phase 4 TUI

</deferred>
