# Phase 6: Science & Engineering - Context

**Gathered:** 2026-05-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Add HP-41 statistics suite and HMS/H time-angle conversion functions to `hp41-core` and expose them in the TUI.

**Deliverables:**
- `hp41-core/src/ops/stats.rs` — Σ+, Σ−, MEAN, SDEV, L.R. (slope+intercept), YHAT, CORR, CLΣSTAT
- `hp41-core/src/ops/hms.rs` — →HMS, HMS→, HMS+, HMS−
- `hp41-core/src/ops/mod.rs` — new Op variants + dispatch arms
- `hp41-cli/src/keys.rs` — 12 new key bindings
- `hp41-cli/src/help_data.rs` — help text for all new ops
- SCI-01, SCI-02 test coverage

**Not in scope:** ALPHA mode improvements, Phase 7 quality gates, GUI.

</domain>

<decisions>
## Implementation Decisions

### Statistics Operations (SCI-01)

- **D-01:** Full HP-41 statistics suite included: Σ+, Σ−, MEAN, SDEV, L.R., YHAT, CORR, CLΣSTAT. All eight ops are in scope for Phase 6. Not just the bare minimum — faithful emulation of the complete HP-41 stats keyboard row.
- **D-02:** CLΣSTAT is required (zeros R01–R06). Without it, starting a new dataset requires manually clearing 6 registers. Low implementation cost; high practical value.
- **D-03:** Σ registers are R01–R06 in the existing `regs: Vec<HpNum>` — no new CalcState field. R01=Σx², R02=Σx, R03=n (count), R04=Σy², R05=Σy, R06=Σxy (HP-41 standard layout).
- **D-04:** Σ register conflict: silent overwrite. Match HP-41 hardware exactly — R01–R06 are dual-use (stats accumulator AND general-purpose STO/RCL). No warning, no blocking. The emulator is faithful, not paternalistic.
- **D-05:** L.R. returns slope m in Y, intercept b in X (HP-41 convention). YHAT reads x from X, returns ŷ in X. CORR returns correlation coefficient r in X.

### HMS/H Conversions (SCI-02)

- **D-06:** Invalid HMS validation: return `HpError::InvalidInput` when seconds ≥ 60 or minutes ≥ 60. Matches HP-41 hardware (INVALID display, op aborted). Consistent with how other domain errors work (Sqrt of negative, Div by zero).
- **D-07:** HMS+ and HMS− must handle base-60 carry/borrow correctly (e.g., 45s + 20s → 1m 5s; not raw decimal addition). The H.MMSS encode/decode must be implemented with rust_decimal arithmetic to avoid binary float rounding.
- **D-08:** Negative HMS values: follow HP-41 convention — sign applies to the whole value (negative hours with positive minutes/seconds). →HMS and HMS→ must handle negative X correctly.

### TUI Key Bindings (12 new ops)

User delegated binding choices to Claude. Scheme follows existing convention: lowercase = primary op, uppercase = secondary/shifted. All bindings are available (no conflicts with existing keys.rs):

| Op | Key | Mnemonic |
|----|-----|----------|
| Σ+ | `z` | Z ≈ Σ, primary accumulate |
| Σ− | `Z` | uppercase = secondary/remove |
| MEAN | `m` | mean |
| SDEV | `d` | deviation |
| YHAT | `y` | ŷ prediction |
| L.R. | `R` | Regression |
| CORR | `O` | cOrrelation |
| CLΣSTAT | `V` | Void/clear stats |
| HMS→ | `h` | hms→decimal (primary) |
| →HMS | `f` | Format as hms |
| HMS+ | `j` | HMS addition |
| HMS− | `J` | uppercase pair for subtraction |

- **D-09:** Existing keys unchanged. No reassignments. `H` = TenPow stays; `h` (lowercase) = HMS→ is new.

### Claude's Discretion

- HMS arithmetic internals: use rust_decimal string-based H.MMSS parsing (split at decimal point, extract hours/minutes/seconds as integers — same pattern as ISG/DSE counter field extraction). Never use floor()/fmod() on f64 for field extraction.
- MEAN stack semantics: X = x̄ (mean of x values), Y = ȳ (mean of y values). SDEV: X = σx, Y = σy (sample std dev, n-1 denominator per HP-41 hardware).
- Σ+ pushes n (count) into X after accumulating. Σ− pushes n into X after removing.
- Program-mode recording: all 12 new ops record normally to `program: Vec<Op>` when `prgm_mode = true`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Architecture
- `hp41-core/src/state.rs` — CalcState struct; `regs: Vec<HpNum>` (R01–R06 are regs[1..=6]); HpNum = rust_decimal::Decimal
- `hp41-core/src/ops/mod.rs` — Op enum and dispatch() function; all new variants added here
- `hp41-core/src/ops/arithmetic.rs` — pattern for binary ops using binary_result()
- `hp41-core/src/ops/math.rs` — pattern for unary/math ops using unary_result()
- `hp41-core/src/ops/registers.rs` — op_sto, op_rcl: how register reads/writes work
- `hp41-core/src/stack.rs` — enter_number(), LiftEffect semantics

### TUI Integration
- `hp41-cli/src/keys.rs` — existing key→Op mapping; 12 new bindings appended here
- `hp41-cli/src/help_data.rs` — static help array; new entries for all 12 ops

### Prior Phase Context
- `.planning/phases/05-persistence-and-ux/05-CONTEXT.md` — Phase 5 decisions (key binding philosophy, op dispatch pattern, CalcState extension pattern)
- `.planning/phases/01-foundation/01-CONTEXT.md` — ADR-001: HpNum = rust_decimal, ISG/DSE field extraction via string-split (same technique for HMS)

### Requirements
- `.planning/REQUIREMENTS.md` §SCI-01, §SCI-02 — formal requirement text

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `hp41-core/src/ops/arithmetic.rs` — `binary_result()` pattern: reads Y and X, computes result, writes to X and drops Y. Σ+/Σ− read X (and Y for two-variable stats), update R01–R06, push n to X.
- `hp41-core/src/ops/math.rs` — `unary_result()` pattern: reads X, returns result in X. MEAN, SDEV, L.R., YHAT, CORR all consume R01–R06 and push results.
- `hp41-core/src/num.rs` — HpNum::from_str(), to_string(), checked_add/sub/mul/div — needed for HMS field extraction and arithmetic.
- `hp41-core/src/error.rs` — HpError variants; `InvalidInput` for HMS validation (check if this variant exists; add it if not).

### Established Patterns
- **String-split for field extraction** (from ISG/DSE): split at decimal, parse integer fields. Use this for H.MMSS → integer hours/minutes/seconds. Never floor()/fmod().
- **LiftEffect::Enable** on result ops (MEAN, SDEV, etc.) — consuming ops that produce a result always enable lift.
- **LiftEffect::Neutral** on accumulating ops (Σ+, Σ−) — they update R01–R06 and push n, which itself enables lift.
- **Op enum exhaustiveness**: every new Op variant must appear in BOTH the enum definition AND the dispatch() match arm. The compiler enforces this.

### Integration Points
- `hp41-core/src/ops/mod.rs` match arm: add new Op variants in the Science & Engineering section comment block (after UserMode).
- `hp41-cli/src/keys.rs` key_to_op() function: append new bindings at the end of the match.
- `hp41-cli/src/help_data.rs` HELP_ENTRIES array: add entries for all 12 new ops with their keyboard shortcuts and descriptions.

</code_context>

<specifics>
## Specific Requirements

- HP-41 Σ register layout is fixed: R01=Σx², R02=Σx, R03=n, R04=Σy², R05=Σy, R06=Σxy. This matches HP-41 Owner's Handbook. All stat computations read these registers directly.
- Round-trip HMS accuracy: 1.3045 → 1.5125 (the ROADMAP success criteria example) must pass exactly.
- SDEV uses sample standard deviation (divide by n−1), not population std dev (divide by n). This matches HP-41 hardware.
- L.R. slope formula: m = (n·Σxy − Σx·Σy) / (n·Σx² − (Σx)²). Intercept: b = (ȳ − m·x̄). CORR: r = (n·Σxy − Σx·Σy) / sqrt((n·Σx² − (Σx)²)·(n·Σy² − (Σy)²)).

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 6-science-and-engineering*
*Context gathered: 2026-05-07*
